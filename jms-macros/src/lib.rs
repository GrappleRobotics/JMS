use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Visibility, Ident, PatType, braced, parse::{ParseStream, Parse, discouraged::Speculative}, Token, FnArg, parenthesized, Pat, parse_macro_input, spanned::Spanned, Type, token, punctuated::Punctuated, Variant, Path};

/* RPC */

// Elements of this are inspired / ripped from tarpc, but adapted for our purposes.

struct Service {
    attrs: Vec<Attribute>,
    vis: Visibility,
    ident: Ident,
    rpcs: Vec<RpcMethod>,
}

struct RpcMethod {
    attrs: Vec<Attribute>,
    ident: Ident,
    args: Vec<PatType>,
    output: Type,
}

impl Parse for Service {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        let vis = input.parse()?;
        input.parse::<Token![trait]>()?;
        let ident: Ident = input.parse()?;
        let content;
        braced!(content in input);
        let mut rpcs = Vec::<RpcMethod>::new();
        while !content.is_empty() {
            rpcs.push(content.parse()?);
        }

        Ok(Self {
            attrs,
            vis,
            ident,
            rpcs,
        })
    }
}

impl Parse for RpcMethod {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attrs = input.call(Attribute::parse_outer)?;
        input.parse::<Token![async]>()?;
        input.parse::<Token![fn]>()?;
        let ident = input.parse()?;

        let content;
        parenthesized!(content in input);
        let mut args = Vec::new();

        for arg in content.parse_terminated(FnArg::parse, Token![,])? {
            match arg {
                FnArg::Typed(captured) if matches!(&*captured.pat, Pat::Ident(_)) => {
                    args.push(captured);
                }
                FnArg::Typed(_) => {
                    return Err(syn::Error::new(arg.span(), "patterns aren't allowed in RPC args"))
                }
                FnArg::Receiver(_) => {
                    return Err(syn::Error::new(arg.span(), "method args cannot start with self"))
                }
            }
        }

        input.parse::<Token![->]>()?;
        let output = input.parse()?;
        input.parse::<Token![;]>()?;

        Ok(Self {
            attrs,
            ident,
            args,
            output,
        })
    }
}

fn define_service_trait(svc: &Service) -> proc_macro2::TokenStream {
    let Service { attrs, vis, ident, rpcs } = svc;

    let request_variants = rpcs.iter().map(|rpc| {
        let RpcMethod { attrs: _, ident, args, output: _ } = rpc;
        quote! {
            #[allow(non_camel_case_types)]
            #ident { #(#args),* }
        }
    });

    let response_variants = rpcs.iter().map(|rpc| {
        let RpcMethod { attrs: _, ident, args: _, output } = rpc;
        quote! {
            #[allow(non_camel_case_types)]
            #ident(#output)
        }
    });

    let request_enum_ident = syn::Ident::new(&format!("{}Request", ident), ident.span());
    let response_enum_ident = syn::Ident::new(&format!("{}Response", ident), ident.span());

    let rpc_call_body = rpcs.iter().map(|rpc| {
        let RpcMethod { attrs: _, ident, args, output: _ } = rpc;
        let untyped_args = args.iter().map(|a| &a.pat).collect::<Vec<_>>();
        quote! {
            #request_enum_ident::#ident { #(#untyped_args),* } => #response_enum_ident::#ident(self.#ident(#(#untyped_args),*).await)
        }
    });

    let rpc_name_literal = ident.to_string();
    let queue = format!("rpc-{}", rpc_name_literal);
    let rpc_fn = quote! {
        async fn rpc_handle(&self) -> Result<jms_base::mq::MessageQueueSubscriber<#request_enum_ident>, anyhow::Error> {
            let mq = self.mq();
            mq.rpc_subscribe(#rpc_name_literal, #queue, #rpc_name_literal, false).await
        }

        async fn rpc_process(&mut self, msg: Option<Result<jms_base::mq::TypedDelivery<#request_enum_ident>, anyhow::Error>>) -> Result<(), anyhow::Error> {
            match msg {
                Some(Ok(msg)) => {
                    let response = match msg.data {
                        #(#rpc_call_body),*
                    };

                    let mq = self.mq();
                    match mq.rpc_reply(&msg.properties, response).await {
                        Ok(()) => (),
                        Err(e) => log::error!("Error in RPC Response: {}", e)
                    };
                    Ok(())
                },
                Some(Err(e)) => {
                    log::error!("Error in RPC Receive: {}", e);
                    Ok(())
                },
                None => anyhow::bail!("Channel Closed"),
            }
        }
    };
    
    let rpc_server_fns = rpcs.iter().map(|rpc| {
        let RpcMethod { attrs, ident, args, output } = rpc;
        quote! {
            #(#attrs)*
            async fn #ident(&mut self, #(#args),*) -> #output;
        }
    });

    // TODO: Sending. Split out MQ so it's passed between functions (incl in Arena)?
    let rpc_client_fns = rpcs.iter().map(|rpc| {
        let RpcMethod { attrs, ident, args, output } = rpc;
        let untyped_args = args.iter().map(|a| &a.pat).collect::<Vec<_>>();
        quote! {
            #(#attrs)*
            pub async fn #ident(mq: &jms_base::mq::MessageQueueChannel, #(#args),*) -> anyhow::Result<#output> {
                let result: #response_enum_ident = mq.rpc_call(#rpc_name_literal, #request_enum_ident::#ident { #(#untyped_args),* }).await?;
                match result {
                    #response_enum_ident::#ident(out) => Ok(out),
                    _ => Err(anyhow::anyhow!("Incorrect RPC response type"))
                }
            }
        }
    });

    let client_struct = syn::Ident::new(&format!("{}{}", ident, "Client"), ident.span());

    let out = quote! {
        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(tag="method", content="data")]
        #vis enum #request_enum_ident {
            #(#request_variants),*
        }

        #[derive(serde::Serialize, serde::Deserialize)]
        #[serde(tag="method", content="data")]
        #vis enum #response_enum_ident {
            #(#response_variants),*
        }

        #(#attrs)*
        #[async_trait::async_trait]
        #vis trait #ident {
            #(#rpc_server_fns)*

            fn mq(&self) -> &jms_base::mq::MessageQueueChannel;

            #rpc_fn
        }

        #vis struct #client_struct;
        impl #client_struct {
            #(#rpc_client_fns)*
        }
    };
    out
}

#[proc_macro_attribute]
pub fn service(attr: TokenStream, input: TokenStream) -> TokenStream {
    let svc = parse_macro_input!(input as Service);

    let trait_inner = define_service_trait(&svc);

    quote! {
        #trait_inner
    }.into()
}

/* WEBSOCKET */
enum WebsocketMessageDirection {
  Send,
  Recv,
  Both
}

impl WebsocketMessageDirection {
  fn to_derives(&self) -> Vec<&'static str> {
    let mut v = vec!["Debug", "Clone", "schemars::JsonSchema"];
    match self {
      WebsocketMessageDirection::Send => v.push("serde::Serialize"),
      WebsocketMessageDirection::Recv => v.push("serde::Deserialize"),
      WebsocketMessageDirection::Both => { 
        v.push("serde::Serialize");
        v.push("serde::Deserialize")
      },
    }
    v
  }

  fn suffix(&self) -> &str {
    match self {
      WebsocketMessageDirection::Send => "2UI",
      WebsocketMessageDirection::Recv => "2JMS",
      WebsocketMessageDirection::Both => "",
    }
  }

  fn applies(&self, other: &Self) -> bool {
    match (self, other) {
      ( Self::Both, _ ) | ( _, Self::Both ) => true,
      ( Self::Send, Self::Send ) => true,
      ( Self::Recv, Self::Recv ) => true,
      _ => false
    }
  }
}

impl Parse for WebsocketMessageDirection {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    Ok(input.step(|cursor| {
      match cursor.ident() {
        Some((ident, next)) if ident.to_string() == "send" => 
          Ok( ( WebsocketMessageDirection::Send, next ) ),
        Some((ident, next)) if ident.to_string() == "recv" =>
          Ok( ( WebsocketMessageDirection::Recv, next ) ),
        _ => Err(cursor.error("neither send nor recv"))
      }
    }).unwrap_or(WebsocketMessageDirection::Both))
  }
}

enum WebsocketMessageField {
  Msg(WebsocketMessage),
  ExtMsg(ExtWebsocketMessage),
  Data {
    dir: WebsocketMessageDirection,
    var: Variant
  }
}

impl Parse for WebsocketMessageField {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    // Speculate WebsocketMessage
    let fork = input.fork();
    if let Ok(msg) = fork.parse() {
      input.advance_to(&fork);
      Ok(WebsocketMessageField::Msg(msg))
    } else if let Ok(msg) = fork.parse() {
      input.advance_to(&fork);
      Ok(WebsocketMessageField::ExtMsg(msg))
    } else {
      // Speculation was wrong - drop the fork and parse dir / variant
      Ok(WebsocketMessageField::Data {
        dir: input.parse()?,
        var: input.parse()?
      })
    }
  }
}

struct WebsocketMessage {
  dir: WebsocketMessageDirection,
  #[allow(dead_code)]
  dollar_token: token::Dollar,
  name: Ident,
  #[allow(dead_code)]
  brace_token: token::Brace,
  children: Punctuated<WebsocketMessageField, token::Comma>
}

mod kw {
  syn::custom_keyword!(ext);
} 

struct ExtWebsocketMessage {
  #[allow(dead_code)]
  ext: kw::ext,
  dir: WebsocketMessageDirection,
  name: Ident,
  #[allow(dead_code)]
  paren: token::Paren,
  message_type: Ident
}

enum StructuredWebsocketMessageField {
  Msg(Ident, Ident),  // variant name, class name
  Data(Variant),
  // ExtMsg(Ident, Ident)    // variant name, class name
}

struct StructuredWebsocketMessage {
  full_name: Ident,
  children: Vec<StructuredWebsocketMessageField>
}

impl Parse for WebsocketMessage {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let content;
    Ok(WebsocketMessage {
      dir: input.parse()?,
      dollar_token: input.parse()?,
      name: input.parse()?,
      brace_token: braced!(content in input),
      children: content.parse_terminated(WebsocketMessageField::parse, Token![,])?
    })
  }
}

impl Parse for ExtWebsocketMessage {
  fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
    let lookahead = input.lookahead1();
    if lookahead.peek(kw::ext) {
      let content;
      Ok(ExtWebsocketMessage {
        ext: input.parse()?,
        dir: input.parse()?,
        name: input.parse()?,
        paren: parenthesized!(content in input),
        message_type: content.parse()?,
      })
    } else {
      Err(lookahead.error())
    }
  }
}

fn build_messages_vec<'a>(root: &'a WebsocketMessage, prefix: &str, v: &mut Vec<(String, &'a WebsocketMessage)>) {
  let name = format!("{}{}", prefix, root.name.to_string());
  // Depth first
  for child in root.children.iter() {
    if let WebsocketMessageField::Msg(m) = child {
      build_messages_vec(m, &name, v);
    }
  }
  v.push((name.clone(), root));
}

fn define_websocket_msg_inner(target_dir: WebsocketMessageDirection, msg: &WebsocketMessage) -> proc_macro2::TokenStream {
  let mut all_messages = vec![];
  build_messages_vec(msg, "",&mut all_messages);
  
  let mut valid_messages = HashMap::<String, StructuredWebsocketMessage>::new();

  // Filter out empty children and assign names
  for (name, m) in all_messages {
    if target_dir.applies(&m.dir) {
      let fullname = format!("{}{}", name, target_dir.suffix());

      let children = m.children.iter().filter_map(|child| match child {
        WebsocketMessageField::Msg(child_msg) if target_dir.applies(&child_msg.dir) => {
          let child_msg_full_name = format!("{}{}{}", name, child_msg.name.to_string(), target_dir.suffix());
          valid_messages.get(&child_msg_full_name).map(|_| {
            StructuredWebsocketMessageField::Msg(child_msg.name.clone(), Ident::new(&child_msg_full_name, child_msg.name.span()))
          })
        },
        WebsocketMessageField::ExtMsg(ext_msg) if target_dir.applies(&ext_msg.dir) => {
          let child_msg_name = format!("{}{}", ext_msg.message_type.to_string(), target_dir.suffix());
          Some(StructuredWebsocketMessageField::Msg(ext_msg.name.clone(), Ident::new(&child_msg_name, ext_msg.message_type.span())))
        },
        WebsocketMessageField::Data { dir, var } if target_dir.applies(&dir) => {
          Some(StructuredWebsocketMessageField::Data(var.clone()))
        },
        _ => None
      }).collect::<Vec<StructuredWebsocketMessageField>>();

      if children.len() > 0 {
        valid_messages.insert(fullname.clone(), StructuredWebsocketMessage { 
          full_name: Ident::new(&fullname, msg.name.span()), 
          children
        });
      }
    }
  }

  let enums = valid_messages.iter().map(|(_, msg)| {
    // Each entry: (To, From, ToVariant)
    let mut froms = vec![];
    let mut path_maps = vec![];

    // let root_name = &msg.name;
    let root_cls = &msg.full_name;

    let derives = target_dir.to_derives();
    let derive_str = derives.iter().map(|&s| syn::parse_str::<Path>(s).unwrap());
    
    let fields = msg.children.iter().filter_map(|child| {
      match child {
        StructuredWebsocketMessageField::Msg(var_name, cls_name) => {
          let var_name_str = var_name.to_string();

          froms.push( (root_cls.clone(), cls_name.clone(), var_name.clone()) );
          path_maps.push(quote! {
            #root_cls::#var_name(submsg) => [vec![#var_name_str].as_slice(), submsg.ws_path().as_slice()].concat()
          });

          Some(quote! {
            #var_name(#cls_name)
          })
        },
        StructuredWebsocketMessageField::Data(var) => {
          let var_name = &var.ident;
          let var_name_str = var_name.to_string();

          let inner = match &var.fields {
            syn::Fields::Named(_) => "{ .. }",
            syn::Fields::Unnamed(_) => "( .. )",
            syn::Fields::Unit => "",
          };

          let field_tokens: proc_macro2::TokenStream = inner.parse().unwrap();

          // let field_tokens = var.fields.to_token_stream();
          path_maps.push(quote! {
            #root_cls::#var_name #field_tokens => vec![#var_name_str]
          });

          Some(quote! {
            #var
          })
        }
      }
    }).collect::<Vec<proc_macro2::TokenStream>>();

    let from_quotes = froms.iter().map(|(to,from,variant)| {
      quote! {
        impl From<#from> for #to {
          fn from(submsg: #from) -> Self {
            #to::#variant(submsg)
          }
        }
      }
    });

    path_maps.push(quote! { _ => vec![] });

    quote! {
      #[derive(#(#derive_str),*)]
      pub enum #root_cls {
        #(#fields),*
      }

      impl #root_cls {
        pub fn ws_path(&self) -> Vec<&str> {
          match self {
            #(#path_maps),*
          }
        }
      }

      #(#from_quotes)*
    }
  });

  quote! {
    #(#enums)*
  }
}

#[proc_macro]
pub fn define_websocket_msg(input: TokenStream) -> TokenStream {
  let msg = parse_macro_input!(input as WebsocketMessage);

  let send = define_websocket_msg_inner(WebsocketMessageDirection::Send, &msg);
  let recv = define_websocket_msg_inner(WebsocketMessageDirection::Recv, &msg);

  quote! {
    #send
    #recv
  }.into()
}
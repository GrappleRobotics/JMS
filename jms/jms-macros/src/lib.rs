use std::collections::HashMap;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, token, Ident, Variant, parse::{Parse, discouraged::Speculative}, punctuated::Punctuated, Path, braced};
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

enum StructuredWebsocketMessageField {
  Msg(Ident, Ident),  // variant name, class name
  Data(Variant)
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
      children: content.parse_terminated(WebsocketMessageField::parse)?
    })
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

  // println!("{}", send);
  // println!();
  // println!("{}", recv);

  quote! {
    #send
    #recv
  }.into()
}
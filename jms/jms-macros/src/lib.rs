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
  v.push((name.clone(), root));
  for child in root.children.iter() {
    if let WebsocketMessageField::Msg(m) = child {
      build_messages_vec(m, &name, v);
    }
  }
}

fn define_websocket_msg_inner(target_dir: WebsocketMessageDirection, msg: &WebsocketMessage) -> proc_macro2::TokenStream {
  let mut all_messages = vec![];
  build_messages_vec(msg, "",&mut all_messages);

  
  let enums = all_messages.into_iter().filter(|(_, m)| target_dir.applies(&m.dir)).map(|(name, msg)| {
    // Each entry: (To, From, ToVariant)
    let mut froms = vec![];
    let mut path_maps = vec![];

    let name_ident = Ident::new(&format!("{}{}", name, target_dir.suffix()), msg.name.span());
    let derives = target_dir.to_derives();
    let derive_str = derives.iter().map(|&s| syn::parse_str::<Path>(s).unwrap());
    
    let fields = msg.children.iter().filter_map(|child| {
      match child {
        WebsocketMessageField::Msg(submsg) if target_dir.applies(&submsg.dir) => {
          let subname = &submsg.name;
          let subname_str = subname.to_string();
          let subname_full = Ident::new(&format!("{}{}{}", name, submsg.name, target_dir.suffix()), subname.span());
          froms.push( (name_ident.clone(), subname_full.clone(), subname.clone()) );
          path_maps.push(quote! {
            #name_ident::#subname(submsg) => [vec![#subname_str].as_slice(), submsg.ws_path().as_slice()].concat()
          });

          Some(quote! {
            #subname(#subname_full)
          })
        },
        WebsocketMessageField::Data { dir, var } if target_dir.applies(dir) => {
          // let variant_name = &var.ident;
          // let variant_name_str = variant_name.to_string();
          // path_maps.push(quote! {
          //   #name_ident::#variant_name(_) => vec![#variant_name_str]
          // });
          
          Some(quote! {
            #var
          })
        },
        _ => None
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
      pub enum #name_ident {
        #(#fields),*
      }

      impl #name_ident {
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
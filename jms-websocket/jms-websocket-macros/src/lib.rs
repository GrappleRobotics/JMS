use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, ItemTrait, TraitItem, spanned::Spanned, ReturnType, Path, PathSegment, PathArguments, GenericArgument, FnArg};

/* Websocket Handler */

fn gen_websocket_handler_trait(t: &ItemTrait) -> proc_macro2::TokenStream {
  let vis = &t.vis;
  let attrs = &t.attrs;
  let ident = &t.ident;

  let trait_ident = syn::Ident::new(&format!("{}Trait", ident), ident.span());

  let inner_items = t.items.iter().filter_map(|item| {
    if let TraitItem::Fn(f) = item {
      let sig = &f.sig;
      let default = &f.default;
      Some(quote! {
        #sig #default
      })
    } else {
      None
    }
  });

  quote! {
    #(#attrs),*
    #[async_trait::async_trait]
    #vis trait #trait_ident {
      #(#inner_items)*
    }
  }
}

fn gen_websocket_handler_impl(t: &ItemTrait) -> syn::Result<proc_macro2::TokenStream> {
  let vis = &t.vis;
  let ident = &t.ident;

  let trait_ident = syn::Ident::new(&format!("{}Trait", ident), ident.span());

  let mut last_published = vec![];
  let mut last_published_defaults = vec![];
  let mut update_publisher_body = vec![];
  let mut on_subscribe_body = vec![];
  let mut rpc_body = vec![];

  let mut publish_names_body = vec![];
  let mut rpc_names_body = vec![];

  let mut publish_schemas = vec![];
  let mut rpc_response_schemas = vec![];
  let mut rpc_request_schemas = vec![];

  for item in &t.items {
    if let TraitItem::Fn(f) = item {
      if f.attrs.len() > 1 {
        return Err(syn::Error::new(f.span(), "A function in a Websocket Handler may only have one attribute, `publish` or `endpoint`"));
      } else if f.attrs.len() == 1 {
        // We're either a publish or an endpoint
        let f_ident = &f.sig.ident;
        let name = f_ident.to_string();
        let return_type = &f.sig.output;

        if let ReturnType::Type(_, typ) = return_type {
          if let Some(return_type) = extract_type_from_result(typ) {
            if f.attrs[0].path().is_ident("publish") {
              // Publish gets pushed directly
              let last_published_ident = syn::Ident::new(&format!("last_{}", f_ident), f_ident.span());

              last_published.push(quote! {
                #last_published_ident: tokio::sync::RwLock<Option<#return_type>>
              });

              last_published_defaults.push(quote! {
                #last_published_ident: tokio::sync::RwLock::new(None)
              });

              publish_names_body.push(quote! {
                v.push(#name.to_owned());
              });

              update_publisher_body.push(quote! {
                {
                  let v = self.#f_ident(context).await?;
                  let mut last = self.#last_published_ident.write().await;
                  match &*last {
                    Some(v2) if v2 == &v => {},
                    _ => {
                      *last = Some(v.clone());
                      to_publish.push((#name.to_owned(), serde_json::to_value(v)?));
                    }
                  }
                }
              });

              let f_ident_str = f_ident.to_string();

              on_subscribe_body.push(quote! {
                if topic == #f_ident_str {
                  let last = self.#last_published_ident.read().await;
                  if let Some(v) = &*last {
                    to_publish.push((#name.to_owned(), serde_json::to_value(v)?));
                  }
                }
              });

              publish_schemas.push(quote! {
                {
                  // let s = schemars::schema_for!(#return_type);
                  let mut s = schemars::schema::SchemaObject::default();
                  let mut path_schema = schemars::schema::SchemaObject::default();
                  path_schema.const_value = Some(serde_json::to_value(format!("{}/{}", handler_key, #name)).unwrap());
                  s.object().required.insert("path".to_owned());
                  s.object().required.insert("data".to_owned());
                  s.object().properties.insert("path".to_owned(), schemars::schema::Schema::Object(path_schema));
                  s.object().properties.insert("data".to_owned(), gen.subschema_for::<#return_type>());
                  sub.push(schemars::schema::Schema::Object(s));
                }
              });
            } else if f.attrs[0].path().is_ident("endpoint") {
              // Filter the args to only be those that aren't self or the context
              let args = f.sig.inputs.iter().filter_map(|input| {
                if let FnArg::Typed(arg) = input {
                  // Ignore the context
                  match &*arg.ty {
                    syn::Type::Reference(re) => match &*re.elem {
                      syn::Type::Path(path) if path.path.segments.iter().find(|x| x.ident == "WebsocketContext" ||  x.ident == "UserToken").is_some() => None,
                      syn::Type::Path(path) => {
                        match path.path.segments.last() {
                          Some(x) if x.ident == "WebsocketContext" || x.ident == "MaybeToken" => None,
                          _ => Some(arg)
                        }
                      },
                      _ => Some(arg)
                    },
                    _ => Some(arg)
                  }
                } else {
                  None
                }
              }).collect::<Vec<_>>();

              rpc_names_body.push(quote! {
                v.push(#name.to_owned());
              });

              if args.is_empty() {
                rpc_body.push(quote! {
                  #name => {
                    Ok((#name.to_owned(), serde_json::to_value(self.#f_ident(ctx, token).await?)?))
                  }
                });

                rpc_request_schemas.push(quote! {
                  {
                    let mut s = schemars::schema::SchemaObject::default();
                    let mut path_schema = schemars::schema::SchemaObject::default();
                    path_schema.const_value = Some(serde_json::to_value(format!("{}/{}", handler_key, #name)).unwrap());
                    s.object().required.insert("path".to_owned());
                    s.object().properties.insert("path".to_owned(), schemars::schema::Schema::Object(path_schema));

                    let mut data_schema = schemars::schema::SchemaObject::default();
                    data_schema.const_value = Some(serde_json::json!(null));
                    s.object().required.insert("data".to_owned());
                    s.object().properties.insert("data".to_owned(), schemars::schema::Schema::Object(data_schema));

                    sub.push(schemars::schema::Schema::Object(s));
                  }
                });
              } else {
                let destructured_from_msg_args = args.iter().map(|arg| {
                  match &*arg.pat {
                    syn::Pat::Ident(ident) => {
                      let arg_name = ident.ident.to_string();
                      quote! { serde_json::from_value(msg.as_ref().ok_or(anyhow::anyhow!("No arguments presented!"))?.get(#arg_name).ok_or(anyhow::anyhow!("Arguments must be an object!"))?.clone())? }
                    },
                    _ => Err(syn::Error::new(arg.span(), "Argument Pattern should be an ident")).unwrap()
                  }
                });

                rpc_body.push(quote! {
                  #name => {
                    Ok((#name.to_owned(), serde_json::to_value(self.#f_ident(ctx, token, #(#destructured_from_msg_args,)*).await?)?))
                  }
                });

                let arg_schema_builder = args.iter().map(|arg| {
                  match &*arg.pat {
                    syn::Pat::Ident(ident) => {
                      let arg_name: String = ident.ident.to_string();
                      let ty = &*arg.ty;
                      quote! {
                        arg_schema.object().required.insert(#arg_name.to_owned());
                        arg_schema.object().properties.insert(#arg_name.to_owned(), gen.subschema_for::<#ty>());
                      }
                    },
                    _ => Err(syn::Error::new(arg.span(), "Argument Pattern should be an ident")).unwrap()
                  }
                });

                rpc_request_schemas.push(quote! {
                  {
                    let mut s = schemars::schema::SchemaObject::default();
                    let mut path_schema = schemars::schema::SchemaObject::default();
                    path_schema.const_value = Some(serde_json::to_value(format!("{}/{}", handler_key, #name)).unwrap());
                    s.object().required.insert("path".to_owned());
                    s.object().properties.insert("path".to_owned(), schemars::schema::Schema::Object(path_schema));

                    let mut arg_schema = schemars::schema::SchemaObject::default();
                    #(#arg_schema_builder)*
                    s.object().required.insert("data".to_owned());
                    s.object().properties.insert("data".to_owned(), schemars::schema::Schema::Object(arg_schema));
                    sub.push(schemars::schema::Schema::Object(s));
                  }
                });
              }

              rpc_response_schemas.push(quote! {
                {
                  let mut s = schemars::schema::SchemaObject::default();
                  let mut path_schema = schemars::schema::SchemaObject::default();
                  path_schema.const_value = Some(serde_json::to_value(format!("{}/{}", handler_key, #name)).unwrap());
                  s.object().required.insert("path".to_owned());
                  s.object().required.insert("data".to_owned());
                  s.object().properties.insert("path".to_owned(), schemars::schema::Schema::Object(path_schema));
                  s.object().properties.insert("data".to_owned(), gen.subschema_for::<#return_type>());
                  sub.push(schemars::schema::Schema::Object(s));
                }
              });
            } else {
              return Err(syn::Error::new(f.attrs[0].span(), "Unrecognised Attribute, should be either `publish` or `endpoint`"))
            }
          } else {
            return Err(syn::Error::new(typ.span(), "Endpoints and Publish functions should return an anyhow Result"));
          }
        } else {
          return Err(syn::Error::new(f.span(), "All endpoints and publish functions should return an anyhow::Result<>"))
        }
      }
    }
  }

  rpc_body.push(quote!{ _ => unreachable!() });

  Ok(quote! {
    #vis struct #ident {
      #(#last_published),*
    }

    #[async_trait::async_trait]
    impl #trait_ident for #ident {}

    impl #ident {
      pub fn new() -> Self {
        Self {
          #(#last_published_defaults),*
        }
      }
    }

    #[async_trait::async_trait]
    impl crate::handler::WebsocketHandler for #ident {
      fn publish_schema(&self, handler_key: &str, gen: &mut schemars::gen::SchemaGenerator) -> Vec<schemars::schema::Schema> {
        let mut sub = vec![];
        #(#publish_schemas)*
        sub
      }

      fn rpc_request_schema(&self, handler_key: &str, gen: &mut schemars::gen::SchemaGenerator) -> Vec<schemars::schema::Schema> {
        let mut sub = vec![];
        #(#rpc_request_schemas)*
        sub
      }

      fn rpc_response_schema(&self, handler_key: &str, gen: &mut schemars::gen::SchemaGenerator) -> Vec<schemars::schema::Schema> {
        let mut sub = vec![];
        #(#rpc_response_schemas)*
        sub
      }

      fn publishers(&self) -> Vec<String> {
        let mut v = vec![];
        #(#publish_names_body)*
        v
      }

      fn rpcs(&self) -> Vec<String> {
        let mut v = vec![];
        #(#rpc_names_body)*
        v
      }

      async fn update_publishers(&self, context: &WebsocketContext) -> anyhow::Result<Vec<(String, serde_json::Value)>> {
        let mut to_publish = vec![];

        #(#update_publisher_body)*

        Ok(to_publish)
      }

      async fn on_subscribe(&self, topic: &str) -> anyhow::Result<Vec<(String, serde_json::Value)>> {
        let mut to_publish = vec![];

        #(#on_subscribe_body)*

        Ok(to_publish)
      }

      async fn process_rpc_call(&self, ctx: &WebsocketContext, token: &jms_core_lib::models::MaybeToken, path: String, msg: Option<serde_json::Value>) -> anyhow::Result<(String, serde_json::Value)> {
        match path.as_str() {
          #(#rpc_body),*
        }
      }
    }
  })
}

#[proc_macro_attribute]
pub fn websocket_handler(_attr: TokenStream, input: TokenStream) -> TokenStream {
  let t = parse_macro_input!(input as ItemTrait);

  let out_trait = gen_websocket_handler_trait(&t);
  let out_impl = gen_websocket_handler_impl(&t).unwrap();

  quote! {
    #out_trait

    #out_impl
  }.into()
}


/* Helpers */

// Adapted from https://stackoverflow.com/a/56264023
fn extract_type_from_result(ty: &syn::Type) -> Option<&syn::Type> {
  fn extract_type_path(ty: &syn::Type) -> Option<&Path> {
    match *ty {
      syn::Type::Path(ref typepath) if typepath.qself.is_none() => Some(&typepath.path),
      _ => None,
    }
  }

  fn extract_option_segment(path: &Path) -> Option<&PathSegment> {
    let idents_of_path = path
      .segments
      .iter()
      .into_iter()
      .fold(String::new(), |mut acc, v| {
        acc.push_str(&v.ident.to_string());
        acc.push('|');
        acc
      });
    vec!["Result|", "anyhow|Result|"]
      .into_iter()
      .find(|s| &idents_of_path == *s)
      .and_then(|_| path.segments.last())
  }

  extract_type_path(ty)
    .and_then(|path| extract_option_segment(path))
    .and_then(|path_seg| {
      let type_params = &path_seg.arguments;
      // It should have only on angle-bracketed param ("<String>"):
      match *type_params {
        PathArguments::AngleBracketed(ref params) => params.args.first(),
        _ => None,
      }
    })
    .and_then(|generic_arg| match *generic_arg {
      GenericArgument::Type(ref ty) => Some(ty),
      _ => None,
    })
}
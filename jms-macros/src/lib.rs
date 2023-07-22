use proc_macro::TokenStream;
use quote::quote;
use syn::{Attribute, Visibility, Ident, ReturnType, PatType, braced, parse::{ParseStream, Parse}, Token, FnArg, parenthesized, Pat, parse_macro_input, spanned::Spanned, Type};

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
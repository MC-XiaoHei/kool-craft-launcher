use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{ItemStruct, LitStr, Path, parse::Parser, parse_macro_input, spanned::Spanned};

struct ConfigArgs {
    name: Option<(LitStr, Span)>,
    evolution: Option<(Path, Span)>,
    post_process: Option<(Path, Span)>,
    no_default: bool,
}

impl ConfigArgs {
    fn parse(args: TokenStream) -> syn::Result<Self> {
        let mut config = ConfigArgs {
            name: None,
            evolution: None,
            post_process: None,
            no_default: false,
        };

        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("name") {
                config.name = Some((meta.value()?.parse()?, meta.path.span()));
            } else if meta.path.is_ident("evolution") {
                config.evolution = Some((meta.value()?.parse()?, meta.path.span()));
            } else if meta.path.is_ident("post_process") {
                config.post_process = Some((meta.value()?.parse()?, meta.path.span()));
            } else if meta.path.is_ident("no_default") {
                config.no_default = true;
            } else {
                return Err(meta.error("unsupported config property"));
            }
            Ok(())
        });

        parser.parse(args)?;

        Ok(config)
    }

    fn expand(&self, item: &ItemStruct) -> syn::Result<proc_macro2::TokenStream> {
        let (name_lit, name_span) = self
            .name
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(&item.ident, "Missing 'name'"))?;

        let derives = self.generate_derives();
        let ide_helper = self.generate_ide_helper(&item.ident, name_lit, *name_span);
        let trait_impl = self.generate_trait_impl(item, name_lit)?;

        Ok(quote! {
            #ide_helper
            #derives
            #item
            #trait_impl
        })
    }

    fn generate_derives(&self) -> proc_macro2::TokenStream {
        let mut traits = vec![
            quote!(std::fmt::Debug),
            quote!(std::clone::Clone),
            quote!(serde::Serialize),
            quote!(serde::Deserialize),
            quote!(schemars::JsonSchema),
        ];

        if !self.no_default {
            traits.push(quote!(std::default::Default));
        }

        quote! { #[derive(#(#traits),*)] }
    }

    fn generate_ide_helper(
        &self,
        struct_name: &syn::Ident,
        name_val: &LitStr,
        name_span: Span,
    ) -> proc_macro2::TokenStream {
        let (evolution_field, evolution_init) = if let Some((path, span)) = &self.evolution {
            let ident = syn::Ident::new("evolution", *span);
            (
                quote! { evolution: fn(&mut serde_json::Value, &std::collections::HashMap<String, serde_json::Value>) -> anyhow::Result<()> },
                quote_spanned! { *span=> #ident: #path },
            )
        } else {
            (quote! {}, quote! {})
        };

        let (post_process_field, post_process_init) = if let Some((path, span)) = &self.post_process
        {
            (
                quote! { post_process: fn(&mut #struct_name) -> anyhow::Result<()> },
                quote_spanned! { *span=> post_process: #path },
            )
        } else {
            (quote! {}, quote! {})
        };

        let name_ident = syn::Ident::new("name", name_span);

        quote! {
            const _: () = {
                struct __ConfigSchemaHighlight {
                    name: &'static str,
                    #evolution_field
                    #post_process_field
                }

                let _ = __ConfigSchemaHighlight {
                    #name_ident: #name_val,
                    #evolution_init
                    #post_process_init
                };
            };
        }
    }

    fn generate_trait_impl(
        &self,
        item: &ItemStruct,
        name: &LitStr,
    ) -> syn::Result<proc_macro2::TokenStream> {
        let struct_name = &item.ident;
        let name_val = name.value();

        let evolution_body = if let Some((handler_path, span)) = &self.evolution {
            quote_spanned! { *span=> #handler_path(current, all_configs) }
        } else {
            quote! { Ok(()) }
        };

        let post_process_body = if let Some((handler_path, span)) = &self.post_process {
            quote_spanned! { *span=> #handler_path(self) }
        } else {
            quote! { Ok(()) }
        };

        Ok(quote! {
            impl crate::config::traits::ConfigGroup for #struct_name {
                const KEY: &'static str = #name_val;

                fn evolve(current: &mut serde_json::Value, all_configs: &std::collections::HashMap<String, serde_json::Value>) -> anyhow::Result<()> {
                    #evolution_body
                }

                fn post_process(&mut self) -> anyhow::Result<()> {
                    #post_process_body
                }
            }
        })
    }
}

pub fn config_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);

    match ConfigArgs::parse(args).and_then(|args| args.expand(&item_struct)) {
        Ok(expanded) => TokenStream::from(expanded),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

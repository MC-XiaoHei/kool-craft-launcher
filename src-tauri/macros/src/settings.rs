use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{quote, quote_spanned};
use syn::{ItemStruct, LitStr, Path, parse::Parser, parse_macro_input, spanned::Spanned};

struct SettingsArgs {
    name: Option<(LitStr, Span)>,
    evolution: Option<(Path, Span)>,
    post_process: Option<(Path, Span)>,
    update_handler: Option<(Path, Span)>,
    no_default: bool,
}

impl SettingsArgs {
    fn parse(args: TokenStream) -> syn::Result<Self> {
        let mut settings = SettingsArgs {
            name: None,
            evolution: None,
            post_process: None,
            update_handler: None,
            no_default: false,
        };

        let parser = syn::meta::parser(|meta| {
            if meta.path.is_ident("name") {
                settings.name = Some((meta.value()?.parse()?, meta.path.span()));
            } else if meta.path.is_ident("evolution") {
                settings.evolution = Some((meta.value()?.parse()?, meta.path.span()));
            } else if meta.path.is_ident("post_process") {
                settings.post_process = Some((meta.value()?.parse()?, meta.path.span()));
            } else if meta.path.is_ident("update_handler") {
                settings.update_handler = Some((meta.value()?.parse()?, meta.path.span()));
            } else if meta.path.is_ident("no_default") {
                settings.no_default = true;
            } else {
                return Err(meta.error("unsupported settings property"));
            }
            Ok(())
        });

        parser.parse(args)?;

        Ok(settings)
    }

    fn expand(&self, item: &ItemStruct) -> syn::Result<proc_macro2::TokenStream> {
        let (name_lit, name_span) = self
            .name
            .as_ref()
            .ok_or_else(|| syn::Error::new_spanned(&item.ident, "Missing 'name'"))?;

        let ide_helper = self.generate_ide_helper(&item.ident, name_lit, *name_span);
        let derives = self.generate_derives();
        let properties = self.generate_properties();
        let trait_impl = self.generate_trait_impl(item, name_lit)?;
        let inventory = self.generate_inventory_submit(&item.ident, name_lit);

        Ok(quote! {
            #ide_helper
            #derives
            #properties
            #item
            #trait_impl
            #inventory
        })
    }

    fn generate_derives(&self) -> proc_macro2::TokenStream {
        let mut traits = vec![
            quote!(std::fmt::Debug),
            quote!(std::clone::Clone),
            quote!(serde::Serialize),
            quote!(serde::Deserialize),
            quote!(schemars::JsonSchema),
            quote!(specta::Type),
        ];

        if !self.no_default {
            traits.push(quote!(std::default::Default));
        }

        quote! {
            #[derive(#(#traits),*)]
        }
    }

    fn generate_properties(&self) -> proc_macro2::TokenStream {
        quote! {
            #[serde(rename_all = "camelCase")]
        }
    }

    fn generate_ide_helper(
        &self,
        struct_name: &syn::Ident,
        name_val: &LitStr,
        name_span: Span,
    ) -> proc_macro2::TokenStream {
        let mut fields = Vec::new();
        let mut inits = Vec::new();

        let name_ident = syn::Ident::new("name", name_span);
        fields.push(quote! { #name_ident: &'static str });
        inits.push(quote! { #name_ident: #name_val });

        if let Some((path, span)) = &self.evolution {
            fields.push(quote_spanned! { *span => evolution: fn(&mut serde_json::Value, &std::collections::HashMap<String, serde_json::Value>) -> anyhow::Result<()> });
            inits.push(quote_spanned! { *span => evolution: #path });
        }

        if let Some((path, span)) = &self.post_process {
            fields.push(quote_spanned! { *span => post_process: fn(&mut #struct_name) -> anyhow::Result<()> });
            inits.push(quote_spanned! { *span => post_process: #path });
        }

        if let Some((path, span)) = &self.update_handler {
            fields.push(quote_spanned! { *span => update_handler: fn(&#struct_name, #struct_name) -> anyhow::Result<()> });
            inits.push(quote_spanned! { *span => update_handler: #path });
        }

        quote! {
            const _: () = {
                struct __SettingsSchemaHighlight {
                    #(#fields),*
                }

                let _ = __SettingsSchemaHighlight {
                    #(#inits),*
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
            quote_spanned! { *span=> #handler_path(current, all_settingss) }
        } else {
            quote! { Ok(()) }
        };

        let post_process_body = if let Some((handler_path, span)) = &self.post_process {
            quote_spanned! { *span=> #handler_path(self) }
        } else {
            quote! { Ok(()) }
        };

        let update_handler_body = if let Some((handler_path, span)) = &self.update_handler {
            quote_spanned! { *span=> #handler_path(self, old) }
        } else {
            quote! { Ok(()) }
        };

        Ok(quote! {
            impl crate::settings::traits::SettingsGroup for #struct_name {
                const KEY: &'static str = #name_val;

                fn evolve(current: &mut serde_json::Value, all_settingss: &std::collections::HashMap<String, serde_json::Value>) -> anyhow::Result<()> {
                    #evolution_body
                }

                fn post_process(&mut self) -> anyhow::Result<()> {
                    #post_process_body
                }

                fn on_update(&self, old: Self) -> anyhow::Result<()> {
                    #update_handler_body
                }
            }
        })
    }

    fn generate_inventory_submit(
        &self,
        struct_name: &syn::Ident,
        key_lit: &LitStr,
    ) -> proc_macro2::TokenStream {
        let name = struct_name.to_string();
        quote! {
            inventory::submit! {
                crate::settings::codegen::SettingsGroupInfo {
                    name: #name,
                    key: #key_lit,
                }
            }

            inventory::submit! {
                crate::settings::commands::SettingsRegisterHook {
                    handler: |store| {
                        Box::pin(async move {
                            store.register::<#struct_name>().await?;
                            Ok(())
                        })
                    }
                }
            }

            inventory::submit! {
                crate::settings::store::UpdateHandlerInfo {
                    key: #key_lit,
                    handler: |neo, old| {
                        use crate::settings::traits::SettingsGroup;
                        let old = serde_json::from_value::<#struct_name>(old)?;
                        let neo = serde_json::from_value::<#struct_name>(neo)?;
                        neo.on_update(old)?;
                        Ok(())
                    }
                }
            }
        }
    }
}

pub fn settings_impl(args: TokenStream, input: TokenStream) -> TokenStream {
    let item_struct = parse_macro_input!(input as ItemStruct);

    match SettingsArgs::parse(args).and_then(|args| args.expand(&item_struct)) {
        Ok(expanded) => TokenStream::from(expanded),
        Err(err) => TokenStream::from(err.to_compile_error()),
    }
}

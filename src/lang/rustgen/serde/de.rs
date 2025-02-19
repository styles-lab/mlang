use proc_macro2::TokenStream;
use quote::quote;

use crate::lang::{
    ir::{Enum, Node},
    rustgen::{
        mapping::{ComplexTypeMapping, FieldMapping},
        serde::SerdeDisplayName,
    },
};

pub(super) trait DeserializeCodeGen {
    fn gen_deserialize_trait(
        &self,
        opcode_mod: &TokenStream,
        deserialize_fn: TokenStream,
        idx: usize,
    ) -> TokenStream;
}

impl DeserializeCodeGen for Node {
    fn gen_deserialize_trait(
        &self,
        opcode_mod: &TokenStream,
        deserialize_fn: TokenStream,
        type_id: usize,
    ) -> TokenStream {
        let ident = self.to_ident();

        let ty_name = self.display_name().unwrap();

        let mut clauses = vec![];

        for (idx, field) in self.fields.iter().enumerate() {
            let name = if let Some(name) = field.display_name() {
                quote! { Some(#name)}
            } else {
                quote! { None}
            };

            let ty = field.to_type_definition(&quote! {});

            clauses.push(
                field.to_init_clause(
                    &quote! {data.deserialize_field::<#ty>(#ty_name, #idx, #name)?},
                ),
            );
        }

        let body = self.to_struct_body(clauses);

        let name = self.display_name().unwrap();

        quote! {
            impl mlang_rs::rt::serde::de::Deserialize for #opcode_mod #ident {

                type Value = #opcode_mod #ident;

                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: mlang_rs::rt::serde::de::Deserializer
                {
                    use mlang_rs::rt::serde::de::*;

                    let _ = deserializer;

                    struct V;

                    impl Visitor for V {
                        type Value = #opcode_mod #ident;

                        #[allow(unused_mut)]
                        fn visit_node<A>(self, mut data: A) -> Result<Self::Value, A::Error>
                        where
                            A: NodeAccess,
                        {
                            let _ = data;

                            use #opcode_mod *;

                            let value = #ident #body;

                            Ok(value)
                        }
                    }

                    deserializer.#deserialize_fn(#type_id, #name, V)
                }
            }
        }
    }
}

impl DeserializeCodeGen for Enum {
    fn gen_deserialize_trait(
        &self,
        opcode_mod: &TokenStream,
        _: TokenStream,
        type_id: usize,
    ) -> TokenStream {
        let ty = self.to_ident();

        let mut idxs = vec![];
        let mut names = vec![];

        for (idx, node) in self.fields.iter().enumerate() {
            let mut clauses = vec![];
            let ty_name = node.display_name().unwrap();

            for (idx, field) in node.fields.iter().enumerate() {
                let name = if let Some(name) = field.display_name() {
                    quote! { Some(#name)}
                } else {
                    quote! { None}
                };

                let ty = field.to_type_definition(opcode_mod);

                clauses.push(field.to_init_clause(
                    &quote! {node.deserialize_field::<#ty>(#ty_name, #idx, #name)?},
                ));
            }

            let field = node.to_ident();
            let body = node.to_struct_body(clauses);

            let name = node.display_name().unwrap();
            idxs.push(quote! {
                #idx => Ok(#opcode_mod #ty::#field #body)
            });

            names.push(quote! {
                #name => Ok(#opcode_mod #ty::#field #body)
            });
        }

        let name = self.display_name().unwrap();

        quote! {
            impl mlang_rs::rt::serde::de::Deserialize for #opcode_mod #ty {

                type Value = #opcode_mod #ty;

                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: mlang_rs::rt::serde::de::Deserializer
                {
                    use mlang_rs::rt::serde::de::*;

                    let _ = deserializer;

                    struct V;

                    impl Visitor for V {
                        type Value = #opcode_mod #ty;

                        /// Visit enum field.
                        #[allow(unused_mut)]
                        fn visit_enum<A>(self, variant_index: usize, mut node: A) -> Result<Self::Value, A::Error>
                        where
                            A: NodeAccess,
                        {
                            let _ = node;
                            match variant_index {
                                #(#idxs,)*
                                _ => Err(Error::UnknownVariantIndex(#name.to_string(),variant_index).into())
                            }
                        }

                        /// Visit enum field.
                        #[allow(unused_mut)]
                        fn visit_enum_with<A>(self, variant: &str, mut node: A) -> Result<Self::Value, A::Error>
                        where
                            A: NodeAccess,
                        {
                            let _ = node;
                            match variant {
                                #(#names,)*
                                _ => Err(Error::UnknownVariant(#name.to_string(),variant.to_string()).into())
                            }
                        }
                    }

                    deserializer.deserialize_enum(#type_id, #name, V)
                }
            }
        }
    }
}

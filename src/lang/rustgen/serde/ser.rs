use proc_macro2::TokenStream;
use quote::quote;

use crate::lang::{
    ir::{Enum, Node},
    rustgen::{
        mapping::{ComplexTypeMapping, FieldMapping},
        serde::SerdeDisplayName,
    },
};

pub(super) trait SerializeCodeGen {
    fn gen_serialize_trait(
        &self,
        opcode_mod: &TokenStream,
        serialize_fn: TokenStream,
        idx: usize,
    ) -> TokenStream;
}

impl SerializeCodeGen for Node {
    fn gen_serialize_trait(
        &self,
        opcode_mod: &TokenStream,
        serialize_fn: TokenStream,
        idx: usize,
    ) -> TokenStream {
        let ident = self.to_ident();
        let name = self.display_name().unwrap();

        let mut stats = vec![];

        for (idx, field) in self.fields.iter().enumerate() {
            let value = if let Some(ident) = field.to_ident() {
                quote! {self.#ident}
            } else {
                format!("self.{}", idx).parse::<TokenStream>().unwrap()
            };

            let name = if let Some(name) = field.display_name() {
                quote! { Some(#name) }
            } else {
                quote! { None }
            };

            stats.push(quote! {
                serializer.serialize_field(#idx, #name, &#value)?
            });
        }

        let fields = stats.len();

        let mut_token = if stats.is_empty() {
            quote! {}
        } else {
            quote! {mut}
        };

        quote! {
            impl mlang::rt::serde::ser::Serialize for #opcode_mod #ident {
                fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
                where
                    S: mlang::rt::serde::ser::Serializer
                {
                    use mlang::rt::serde::ser::SerializeNode;
                    let #mut_token serializer = serializer.#serialize_fn(#idx, #name, #fields)?;
                    #(#stats;)*

                    serializer.finish()
                }
            }
        }
    }
}

impl SerializeCodeGen for Enum {
    fn gen_serialize_trait(
        &self,
        opcode_mod: &TokenStream,
        serialize_fn: TokenStream,
        type_id: usize,
    ) -> TokenStream {
        let mut stats = vec![];
        let enum_name = self.display_name().unwrap();

        for (idx, node) in self.fields.iter().enumerate() {
            let ident = node.to_ident();

            let mut node_stats = vec![];
            let mut fields = vec![];

            for (idx, field) in node.fields.iter().enumerate() {
                if let Some(ident) = field.to_ident() {
                    let name = ident.to_string();
                    node_stats.push(quote! {
                        serializer.serialize_field(#idx, Some(#name), #ident)?;
                    });
                    fields.push(ident);
                } else {
                    let ident = format!("p{}", idx).parse::<TokenStream>().unwrap();
                    node_stats.push(quote! {
                        serializer.serialize_field(#idx, None, #ident)?;
                    });
                    fields.push(ident);
                }
            }

            let field_count = fields.len();

            let body = node.to_struct_body(fields);

            let variant = node.display_name().unwrap();

            let mut_token = if node_stats.is_empty() {
                quote! {}
            } else {
                quote! {mut}
            };

            let stat = quote! {
                Self::#ident #body => {
                    let #mut_token serializer = serializer.#serialize_fn(#type_id, #enum_name, #variant, #idx, #field_count)?;
                    #(#node_stats)*
                    serializer.finish()
                }
            };

            stats.push(stat);
        }

        let ident = self.to_ident();

        quote! {
            impl mlang::rt::serde::ser::Serialize for #opcode_mod #ident {
                fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
                where
                    S: mlang::rt::serde::ser::Serializer
                {
                    use mlang::rt::serde::ser::SerializeNode;
                    match self {
                        #(#stats),*
                    }
                }
            }

        }
    }
}

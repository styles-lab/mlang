use heck::ToUpperCamelCase;
use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::lang::{
    ir::{Enum, Node, Stat},
    rustgen::mapping::{CommentMapping, ComplexTypeMapping, FieldMapping},
};

trait TypeDefinitionCodeGen {
    fn gen(&self) -> TokenStream;
}

impl TypeDefinitionCodeGen for Node {
    fn gen(&self) -> TokenStream {
        let comments = self.to_comment();
        let ident = self.to_ident();

        let fields = self
            .fields
            .iter()
            .map(|field| {
                let ty = field.to_type_definition(&quote! {});

                field.to_definition_clause(&quote! { pub }, &ty)
            })
            .collect::<Vec<_>>();

        let body = self.to_struct_body(fields);

        let semi_token = self.to_semi_token();

        quote! {
            #comments
            #[derive(Debug, PartialEq, PartialOrd, Clone)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub struct #ident #body #semi_token
        }
    }
}

impl TypeDefinitionCodeGen for Enum {
    fn gen(&self) -> TokenStream {
        let comments = self.to_comment();
        let ident = self.to_ident();

        let fields = self
            .fields
            .iter()
            .map(|node| {
                let comments = node.to_comment();
                let ident = node.to_ident();

                let fields = node
                    .fields
                    .iter()
                    .map(|field| {
                        let ty = field.to_type_definition(&quote! {});

                        field.to_definition_clause(&quote! {}, &ty)
                    })
                    .collect::<Vec<_>>();

                let body = node.to_struct_body(fields);

                quote! {
                    #comments
                    #ident #body
                }
            })
            .collect::<Vec<_>>();

        quote! {
            #comments
            #[derive(Debug, PartialEq, PartialOrd, Clone)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub enum #ident { #(#fields),* }
        }
    }
}

/// opcode module code generator.
#[derive(Default)]
struct CodeGen {
    /// collection of data types.
    data_types: Vec<TokenStream>,
    /// collection of attr types
    attr_types: Vec<TokenStream>,
    /// collection of el types
    el_types: Vec<TokenStream>,
    /// collection of leaf node types.
    leaf_types: Vec<TokenStream>,
}

impl CodeGen {
    fn gen(mut self, stats: &[Stat]) -> TokenStream {
        let mut token_streams = vec![];

        for opcode in stats {
            match opcode {
                Stat::Element(node) => {
                    token_streams.push(node.gen());
                    self.el_types.push(node.to_ident());
                }
                Stat::Leaf(node) => {
                    token_streams.push(node.gen());
                    self.leaf_types.push(node.to_ident());
                }
                Stat::Attr(node) => {
                    token_streams.push(node.gen());
                    self.attr_types.push(node.to_ident());
                }
                Stat::Data(node) => {
                    token_streams.push(node.gen());
                    self.data_types.push(node.to_ident());
                }
                Stat::Enum(node) => {
                    token_streams.push(node.gen());
                    self.data_types.push(node.to_ident());
                }
                _ => {}
            }
        }

        token_streams.push(self.gen_data_definition());
        token_streams.push(self.gen_attr_definition());
        token_streams.push(self.gen_el_definition());
        token_streams.push(self.gen_leaf_definition());
        token_streams.push(self.gen_opcode_definition());
        token_streams.push(self.gen_variable_definition());

        quote! {
            #(#token_streams)*
        }
    }
    fn gen_variable_definition(&self) -> TokenStream {
        quote! {
            pub mod variable {
                pub use mlang::rt::opcode::*;
            }
        }
    }

    fn gen_data_definition(&self) -> TokenStream {
        let mut fields = vec![];
        let mut froms = vec![];

        let builtin_types = vec![
            ("bool", "bool"),
            ("string", "String"),
            ("byte", "i8"),
            ("ubyte", "u8"),
            ("short", "i16"),
            ("ushort", "u16"),
            ("int", "i32"),
            ("uint", "u32"),
            ("long", "i64"),
            ("ulong", "u64"),
            ("float", "f32"),
            ("double", "f64"),
        ];

        for (ident, ty) in builtin_types {
            let name = ident.to_upper_camel_case();
            let ty: TokenStream = ty.parse().unwrap();

            let ident: TokenStream = name.parse().unwrap();
            let list_ident = format_ident!("ListOf{}", name);

            fields.push(quote! {
                #ident(#ty),#list_ident(Box<Vec<#ty>>)
            });

            froms.push(quote! {
                impl From<#ty> for Data {
                    fn from(value: #ty) -> Self {
                        Data::#ident(value)
                    }
                }

                impl<'a> TryFrom<&'a Data> for &'a #ty {
                    type Error = ();

                    fn try_from(value: &'a Data) -> Result<Self,Self::Error> {
                        match value {
                            Data::#ident(v) => Ok(v),
                            _ => Err(())
                        }
                    }
                }

                impl From<Vec<#ty>> for Data {
                    fn from(value: Vec<#ty>) -> Self {
                        Data::#list_ident(Box::new(value))
                    }
                }

                impl<'a> TryFrom<&'a Data> for &'a Vec<#ty> {
                    type Error = ();

                    fn try_from(value: &'a Data) -> Result<Self,Self::Error> {
                        match value {
                            Data::#list_ident(v) => Ok(v),
                            _ => Err(())
                        }
                    }
                }
            });
        }

        for ident in &self.data_types {
            let list_ident = format_ident!("ListOf{}", ident.to_string());

            fields.push(quote! {
                #ident(Box<#ident>),#list_ident(Box<Vec<#ident>>)
            });

            froms.push(quote! {
                impl From<#ident> for Data {
                    fn from(value: #ident) -> Self {
                        Data::#ident(Box::new(value))
                    }
                }

                impl<'a> TryFrom<&'a Data> for &'a #ident {
                    type Error = ();

                    fn try_from(value: &'a Data) -> Result<Self,Self::Error> {
                        match value {
                            Data::#ident(v) => Ok(v),
                            _ => Err(())
                        }
                    }
                }

                impl From<Vec<#ident>> for Data {
                    fn from(value: Vec<#ident>) -> Self {
                        Data::#list_ident(Box::new(value))
                    }
                }

                impl<'a> TryFrom<&'a Data> for &'a Vec<#ident> {
                    type Error = ();

                    fn try_from(value: &'a Data) -> Result<Self,Self::Error> {
                        match value {
                            Data::#list_ident(v) => Ok(v),
                            _ => Err(())
                        }
                    }
                }
            });
        }

        quote! {
            #[derive(Debug, PartialEq, PartialOrd, Clone)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub enum Data {
                #(#fields),*
            }

            #(#froms)*
        }
    }

    fn gen_el_definition(&self) -> TokenStream {
        let mut fields = vec![];
        let mut froms = vec![];

        for el in &self.el_types {
            fields.push(quote! {
                #el(Box<#el>)
            });

            froms.push(quote! {
                impl From<#el> for Element {
                    fn from(value: #el) -> Self {
                        Self::#el(Box::new(value))
                    }
                }
            });
        }

        quote! {

            #[derive(Debug, PartialEq, PartialOrd, Clone)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub enum Element {
                #(#fields),*
            }

            #(#froms)*
        }
    }

    fn gen_leaf_definition(&self) -> TokenStream {
        let mut fields = vec![];
        let mut froms = vec![];

        for el in &self.leaf_types {
            fields.push(quote! {
                #el(Box<#el>)
            });

            froms.push(quote! {
                impl From<#el> for Leaf {
                    fn from(value: #el) -> Self {
                        Self::#el(Box::new(value))
                    }
                }
            });
        }

        quote! {

            #[derive(Debug, PartialEq, PartialOrd, Clone)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub enum Leaf {
                #(#fields),*
            }

            #(#froms)*
        }
    }

    pub(super) fn gen_attr_definition(&self) -> TokenStream {
        let mut fields = vec![];
        let mut froms = vec![];

        for el in &self.attr_types {
            fields.push(quote! {
                #el(Box<#el>)
            });

            froms.push(quote! {
                impl From<#el> for Attr {
                    fn from(value: #el) -> Self {
                        Self::#el(Box::new(value))
                    }
                }
            });
        }

        quote! {

            #[derive(Debug, PartialEq, PartialOrd, Clone)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub enum Attr {
                #(#fields),*
            }

            #(#froms)*
        }
    }

    fn gen_opcode_definition(&self) -> TokenStream {
        quote! {
            #[derive(Debug, Clone, PartialEq)]
            #[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
            pub enum Opcode {
                Apply(Attr),
                Element(Element),
                Pop,
                Leaf(Leaf),
            }

            impl From<Attr> for Opcode {
                fn from(value: Attr) -> Self {
                    Self::Apply(value)
                }
            }

            impl From<Element> for Opcode {
                fn from(value: Element) -> Self {
                    Self::Element(value)
                }
            }

            impl From<Leaf> for Opcode {
                fn from(value: Leaf) -> Self {
                    Self::Leaf(value)
                }
            }
        }
    }
}

/// Generate opcode module from [`stats`](Stat).
pub fn gen_opcode_mod(stats: impl AsRef<[Stat]>) -> TokenStream {
    CodeGen::default().gen(stats.as_ref())
}

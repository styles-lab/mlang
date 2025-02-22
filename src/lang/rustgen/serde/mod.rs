mod de;
mod ser;

use std::collections::{HashMap, HashSet};

use de::DeserializeCodeGen;
use heck::ToLowerCamelCase;
use proc_macro2::TokenStream;
use quote::quote;
use ser::SerializeCodeGen;

use crate::lang::{
    ir::{Enum, Field, Node, Stat},
    rustgen::mapping::ComplexTypeMapping,
};

trait SerdeDisplayName {
    fn display_name(&self) -> Option<String>;
}

impl SerdeDisplayName for Node {
    fn display_name(&self) -> Option<String> {
        if let Some(v) = self.rename() {
            Some(v.to_string())
        } else {
            Some(self.ident.1.to_lower_camel_case())
        }
    }
}

impl SerdeDisplayName for Enum {
    fn display_name(&self) -> Option<String> {
        if let Some(v) = self.rename() {
            Some(v.to_string())
        } else {
            Some(self.ident.1.to_lower_camel_case())
        }
    }
}

impl<'a> SerdeDisplayName for Field<'a> {
    fn display_name(&self) -> Option<String> {
        if let Some(v) = self.rename() {
            Some(v.to_string())
        } else {
            self.ident().map(|v| v.1.to_lower_camel_case())
        }
    }
}

struct CodeGen(TokenStream);

impl CodeGen {
    /// Create new sexpr mode generator
    pub fn new(opcode_mod: impl AsRef<str>) -> Self {
        Self(opcode_mod.as_ref().parse().unwrap())
    }

    /// Generate sexpr mod
    pub fn codegen(self, stats: &[Stat]) -> TokenStream {
        let opcode_mod = &self.0;

        let mut impls: Vec<TokenStream> = vec![];

        let mut attr_fields: HashMap<String, Vec<String>> = Default::default();
        let mut apply_attrs: HashMap<String, HashSet<String>> = Default::default();
        let mut display_names: HashMap<String, String> = Default::default();

        for (idx, stat) in stats.iter().enumerate() {
            match stat {
                Stat::Element(node) => {
                    impls.push(node.gen_serialize_trait(opcode_mod, quote! { serialize_el }, idx));
                    impls.push(node.gen_deserialize_trait(
                        opcode_mod,
                        quote! { deserialize_element },
                        idx,
                    ));

                    display_names.insert(node.ident.1.clone(), node.display_name().unwrap());
                }
                Stat::Leaf(node) => {
                    impls.push(node.gen_serialize_trait(
                        opcode_mod,
                        quote! { serialize_leaf },
                        idx,
                    ));

                    impls.push(node.gen_deserialize_trait(
                        opcode_mod,
                        quote! { deserialize_leaf },
                        idx,
                    ));

                    display_names.insert(node.ident.1.clone(), node.display_name().unwrap());
                }
                Stat::Attr(node) => {
                    impls.push(node.gen_serialize_trait(
                        opcode_mod,
                        quote! { serialize_attr },
                        idx,
                    ));

                    impls.push(node.gen_deserialize_trait(
                        opcode_mod,
                        quote! { deserialize_attr },
                        idx,
                    ));

                    attr_fields.insert(
                        node.ident.1.clone(),
                        node.fields
                            .iter()
                            .filter_map(|field| field.display_name())
                            .collect::<Vec<_>>(),
                    );

                    display_names.insert(node.ident.1.clone(), node.display_name().unwrap());
                }
                Stat::Data(node) => {
                    impls.push(node.gen_serialize_trait(
                        opcode_mod,
                        quote! { serialize_data },
                        idx,
                    ));

                    impls.push(node.gen_deserialize_trait(
                        opcode_mod,
                        quote! { deserialize_data },
                        idx,
                    ));
                }
                Stat::Enum(node) => {
                    impls.push(node.gen_serialize_trait(
                        opcode_mod,
                        quote! { serialize_enum },
                        idx,
                    ));

                    impls.push(node.gen_deserialize_trait(
                        opcode_mod,
                        quote! { deserialize_enum },
                        idx,
                    ));
                }
                Stat::ApplyTo(apply_to) => {
                    for from in &apply_to.from {
                        for to in &apply_to.to {
                            apply_attrs
                                .entry(to.1.clone())
                                .or_insert_with(|| Default::default())
                                .insert(from.1.clone());
                        }
                    }
                }
                _ => {}
            }
        }

        let fileds_to_attrs = self.gen_fields_to_attrs(&apply_attrs, attr_fields, display_names);

        impls.push(self.gen_opcode_serialize_trait(stats));
        impls.push(self.gen_opcode_deserialize_trait(fileds_to_attrs, stats));

        quote! {
            #(#impls)*
        }
    }

    fn gen_fields_to_attrs(
        &self,
        apply_attrs: &HashMap<String, HashSet<String>>,
        attr_fields: HashMap<String, Vec<String>>,
        display_names: HashMap<String, String>,
    ) -> TokenStream {
        let mut clauses = vec![];

        let mut keys = apply_attrs.keys().collect::<Vec<_>>();

        keys.sort();

        for to in keys {
            let ty = display_names.get(to).expect(&format!(
                "apply to node({})'s display name is not found",
                to
            ));

            let mut fields_clauses = vec![];

            let mut attrs = apply_attrs[to].iter().collect::<Vec<_>>();

            attrs.sort();

            for attr in attrs {
                let name = display_names
                    .get(attr)
                    .expect(&format!("attr({})'s display name is not found", attr));

                fields_clauses.push(quote! {
                    #[allow(unreachable_patterns)]
                    #name => { attrs.insert(#name); },
                });

                if let Some(fields) = attr_fields.get(attr) {
                    fields_clauses.push(quote! {
                        #(
                            #[allow(unreachable_patterns)]
                            #fields => { attrs.insert(#name); },
                        )*
                    });
                }
            }

            clauses.push(quote! {
                #ty => {
                    match attr_name {
                        #(#fields_clauses)*
                        _ => {}
                    }
                }
            });
        }

        quote! {
            match name {
                #(#clauses,)*
                _ => {

                }
            }
        }
    }

    fn gen_visit_opcode_clause(
        &self,
        type_id: usize,
        node: &Node,
        from: impl FnOnce(TokenStream) -> TokenStream,
    ) -> TokenStream {
        let ident = node.to_ident();
        let state = from(quote! { #ident::deserialize(deserializer)? });
        quote! {
            #type_id => #state
        }
    }

    fn gen_visit_opcode_with_clause(
        &self,
        node: &Node,
        from: impl FnOnce(TokenStream) -> TokenStream,
    ) -> TokenStream {
        let ident = node.to_ident();
        let name = node.display_name().unwrap();
        let state = from(quote! { #ident::deserialize(deserializer)? });
        quote! {
            #name => #state
        }
    }

    fn gen_opcode_deserialize_trait(
        &self,
        fileds_to_attrs: TokenStream,
        stats: &[Stat],
    ) -> TokenStream {
        let opcode_mod = &self.0;
        let _ = stats;

        let mut visit_opcode_clauses = vec![];
        let mut visit_opcode_with_clauses = vec![];
        let mut element_names = vec![];
        let mut leaf_names = vec![];

        for (type_id, state) in stats.iter().enumerate() {
            match state {
                Stat::Element(node) => {
                    element_names.push(node.display_name().unwrap());
                    visit_opcode_clauses.push(self.gen_visit_opcode_clause(
                        type_id,
                        node,
                        |token_stream| {
                            quote! {
                                Ok(Opcode::from(Element::from(#token_stream)))
                            }
                        },
                    ));
                    visit_opcode_with_clauses.push(self.gen_visit_opcode_with_clause(
                        node,
                        |token_stream| {
                            quote! {
                                Ok(Opcode::from(Element::from(#token_stream)))
                            }
                        },
                    ));
                }
                Stat::Leaf(node) => {
                    leaf_names.push(node.display_name().unwrap());
                    visit_opcode_clauses.push(self.gen_visit_opcode_clause(
                        type_id,
                        node,
                        |token_stream| {
                            quote! {
                                Ok(Opcode::from(Leaf::from(#token_stream)))
                            }
                        },
                    ));
                    visit_opcode_with_clauses.push(self.gen_visit_opcode_with_clause(
                        node,
                        |token_stream| {
                            quote! {
                                Ok(Opcode::from(Leaf::from(#token_stream)))
                            }
                        },
                    ));
                }
                Stat::Attr(node) => {
                    visit_opcode_clauses.push(self.gen_visit_opcode_clause(
                        type_id,
                        node,
                        |token_stream| {
                            quote! {
                                Ok(Opcode::from(Attr::from(#token_stream)))
                            }
                        },
                    ));
                    visit_opcode_with_clauses.push(self.gen_visit_opcode_with_clause(
                        node,
                        |token_stream| {
                            quote! {
                                Ok(Opcode::from(Attr::from(#token_stream)))
                            }
                        },
                    ));
                }
                _ => {}
            }
        }

        quote! {
            impl mlang_rs::rt::serde::de::Deserialize for #opcode_mod Opcode {
                type Value = Option<Vec<#opcode_mod Opcode>>;

                fn deserialize<D>(deserializer: D) -> Result<Self::Value, D::Error>
                where
                    D: mlang_rs::rt::serde::de::Deserializer

                {
                    use mlang_rs::rt::serde::de::*;

                    let _ = deserializer;

                    struct V;

                    impl Visitor for V {
                        type Value = #opcode_mod Opcode;

                        fn is_element(&self, name: &str) -> bool
                        {
                            match name {
                                #(#element_names)|* => true,
                                _ => false
                            }

                        }

                        fn is_leaf(&self, name: &str) -> bool
                        {
                            match name {
                                #(#leaf_names)|* => true,
                                _ => false
                            }

                        }

                        fn visit_opcode<D>(self, type_id: usize, deserializer: D) -> Result<Self::Value, D::Error>
                        where
                            D: Deserializer,
                        {
                            use crate::opcode::*;

                            match type_id {
                                #(#visit_opcode_clauses,)*
                                _ => {
                                    return Err(mlang_rs::rt::serde::de::Error::UnknownType(type_id).into());
                                }
                            }
                        }

                        fn visit_opcode_with<D>(self, name: &str, deserializer: D) -> Result<Self::Value, D::Error>
                        where
                            D: Deserializer,
                        {
                            use crate::opcode::*;

                            match name {
                                #(#visit_opcode_with_clauses,)*
                                _ => {
                                    return Err(mlang_rs::rt::serde::de::Error::UnknownTypeName(name.to_string()).into());
                                }
                            }
                        }

                        fn visit_opcode_with_attrs<D>(
                            self,
                            name: &str,
                            mut deserializer: D,
                        ) -> Result<Vec<Self::Value>, D::Error>
                        where
                            D: AttrsNodeAccess,
                        {
                            let _ = name;
                            let _ = deserializer;

                            let mut attrs = std::collections::HashSet::new();

                            for attr_name in deserializer.attrs() {
                                #fileds_to_attrs
                            }

                            let mut opcodes = vec![];

                            let mut attrs = attrs.into_iter().collect::<Vec<_>>();

                            attrs.sort();

                            for attr in attrs {
                                opcodes.push(deserializer.deserialize_attr(attr,Self)?);
                            }

                            opcodes.push(deserializer.deserialize_attr(name,Self)?);

                            Ok(opcodes)
                        }

                        fn visit_pop<E>(self) -> Result<Self::Value, E>
                        where
                            E: From<Error>,
                        {
                            Ok(Self::Value::Pop)
                        }
                    }

                    deserializer.deserialize_opcode(V)
                }
            }
        }
    }

    fn gen_opcode_serialize_trait(&self, stats: &[Stat]) -> TokenStream {
        let opcode_mod = &self.0;

        let mut ser_els = vec![];
        let mut ser_leaves = vec![];
        let mut ser_attrs = vec![];

        for stat in stats.iter() {
            match stat {
                Stat::Element(node) => {
                    let ident = node.to_ident();

                    ser_els.push(quote! {
                        #opcode_mod Element::#ident(value) => value.serialize(serializer)
                    });
                }
                Stat::Leaf(node) => {
                    let ident = node.to_ident();

                    ser_leaves.push(quote! {
                        #opcode_mod Leaf::#ident(value) => value.serialize(serializer)
                    });
                }
                Stat::Attr(node) => {
                    let ident = node.to_ident();

                    ser_attrs.push(quote! {
                        #opcode_mod Attr::#ident(value) => value.serialize(serializer)
                    });
                }
                _ => {}
            }
        }

        quote! {
            impl mlang_rs::rt::serde::ser::Serialize for #opcode_mod Opcode {
                fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
                where
                    S: mlang_rs::rt::serde::ser::Serializer
                {
                    match self {
                        Self::Apply(v) => {
                            match v {
                                #(#ser_attrs),*
                            }
                        },
                        Self::Element(v) => {
                            match v {
                                #(#ser_els),*
                            }
                        },
                        Self::Leaf(v) => {
                            match v {
                                #(#ser_leaves),*
                            }
                        }
                        Self::Pop => {
                            serializer.serialize_pop()
                        },

                    }
                }
            }
        }
    }
}

/// Generate serde module from [`stats`](Stat).
pub fn gen_serde_mod(stats: impl AsRef<[Stat]>, opcode_mod: impl AsRef<str>) -> TokenStream {
    CodeGen::new(opcode_mod).codegen(stats.as_ref())
}

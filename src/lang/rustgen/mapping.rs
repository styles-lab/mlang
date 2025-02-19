//! This module defines `trait`s and `fn`s that help map `mlang` types to `rust` types.

use heck::{ToSnakeCase, ToUpperCamelCase};
use proc_macro2::TokenStream;
use quote::quote;

use crate::lang::ir::{Comment, Enum, Field, Ident, Node, Type};

/// A trait to help mapping [`Ident`] to rust type/field ident.
pub trait CommentMapping {
    /// Convert self to rust comment lines.
    fn to_comment(&self) -> TokenStream;
}

impl CommentMapping for Comment {
    fn to_comment(&self) -> TokenStream {
        format!("/// {0}", self.1).parse().expect("to_comment")
    }
}

/// A trait to help mapping [`Ident`] to rust type/field ident.
pub trait IdentMapping {
    /// Convert [`Ident`] to rust field name: xMinYMin => x_min_y_min.
    fn to_field_name(&self) -> TokenStream;

    /// Convert [`Ident`] to rust type(struct, enum, enum field) name: xMinYMin => XMinYMin.
    fn to_type_name(&self) -> TokenStream;
}

impl IdentMapping for Ident {
    fn to_field_name(&self) -> TokenStream {
        match self.1.to_snake_case().as_str() {
            "type" => "r#type".parse().expect("to_field_name"),
            "in" => "r#in".parse().expect("to_field_name"),
            "for" => "r#for".parse().expect("to_field_name"),
            ident => ident.parse().expect("to_field_name"),
        }
    }

    fn to_type_name(&self) -> TokenStream {
        self.1.to_upper_camel_case().parse().expect("to_type_name")
    }
}

/// A trait to help mapping [`Type`] to rust type.
pub trait TypeMapping {
    /// Convert [`Type`] to rust type definition.
    fn to_definition(&self, ty_mod: &TokenStream) -> TokenStream;

    /// Convert [`Type`] to rust where clause.
    fn to_from_where_clause(
        &self,
        ty_mod: &TokenStream,
        sexpr_mod: &TokenStream,
        generic_ty: &TokenStream,
    ) -> TokenStream;

    /// Generate rust from clause for this type.
    fn to_from_clause(&self, sexpr_mod: &TokenStream, param: &TokenStream) -> TokenStream;
}

impl TypeMapping for Type {
    fn to_definition(&self, ty_mod: &TokenStream) -> TokenStream {
        match self {
            Type::Bool(_) => quote! {bool},
            Type::String(_) => quote! {String},
            Type::Byte(_) => quote! {i8},
            Type::Ubyte(_) => quote! {u8},
            Type::Short(_) => quote! {i16},
            Type::Ushort(_) => quote! {u16},
            Type::Int(_) => quote! {i32},
            Type::Uint(_) => quote! {u32},
            Type::Long(_) => quote! {i64},
            Type::Ulong(_) => quote! {u64},
            Type::Float(_) => quote! {f32},
            Type::Double(_) => quote! {f64},
            Type::Data(ident) => {
                let ident = ident.to_type_name();

                let ty_mod = ty_mod;

                quote! { #ty_mod #ident }
            }
            Type::ListOf(component, _) => {
                let component = component.to_definition(ty_mod);

                quote! { Vec<#component> }
            }
            Type::ArrayOf(component, lit_num, _) => {
                let component = component.to_definition(ty_mod);
                let num = lit_num.0;

                quote! { [#component;#num] }
            }
        }
    }

    fn to_from_where_clause(
        &self,
        ty_mod: &TokenStream,
        sexpr_mod: &TokenStream,
        generic_ty: &TokenStream,
    ) -> TokenStream {
        match self {
            Type::Data(ident) => {
                let ty = ident.to_type_name();
                let ty_mod = ty_mod;
                let generic_ty = generic_ty;
                quote! {
                   #ty_mod #ty : From<#generic_ty>
                }
            }
            Type::ListOf(component, _) => {
                let ty = component.to_definition(ty_mod);
                let generic_ty = generic_ty;
                quote! {
                    #generic_ty: #sexpr_mod MapCollect<#ty>
                }
            }
            Type::Float(_) => {
                let generic_ty = generic_ty;
                quote! { #sexpr_mod Number: From<#generic_ty> }
            }
            _ => {
                let ty = self.to_definition(ty_mod);
                let generic_ty = generic_ty;
                quote! {
                   #ty : From<#generic_ty>
                }
            }
        }
    }

    fn to_from_clause(&self, sexpr_mod: &TokenStream, param: &TokenStream) -> TokenStream {
        let param = param;

        match self {
            Type::ListOf(_, _) => {
                quote! {
                    #param.map_collect()
                }
            }
            Type::Float(_) => {
                quote! { #sexpr_mod Number::from(#param).0 }
            }
            _ => {
                quote! {
                    #param.into()
                }
            }
        }
    }
}

/// A trait to help mapping [`Field`] to rust field clause.
pub trait FieldMapping: CommentMapping {
    /// Generate rust from clause for this field: Some(value) or Variable::Constant(value)
    fn to_from_clause(&self, sexpr_mod: &TokenStream, param: &TokenStream) -> TokenStream;

    /// Generate rust struct field init clause: `a: 1usize` or `b: value.0`.
    fn to_init_clause(&self, param: &TokenStream) -> TokenStream;

    /// Convert field [`Type`] to rust filed type.
    fn to_type_definition(&self, ty_mod: &TokenStream) -> TokenStream;

    /// Generate rust field ident for [`Field`]
    fn to_ident(&self) -> Option<TokenStream>;

    /// Generate field definition clause for this field.
    fn to_definition_clause(&self, vis: &TokenStream, ty: &TokenStream) -> TokenStream;
}

impl<'a> CommentMapping for Field<'a> {
    fn to_comment(&self) -> TokenStream {
        self.comments().iter().map(|c| c.to_comment()).collect()
    }
}

impl<'a> FieldMapping for Field<'a> {
    fn to_from_clause(&self, sexpr_mod: &TokenStream, param: &TokenStream) -> TokenStream {
        let mut param = self.ty().to_from_clause(sexpr_mod, param);

        if self.is_variable() {
            param = quote! { mlang_rs::rt::opcode::Variable::Constant(#param) };
        }

        if self.is_option() {
            param = quote! { Some(#param) };
        }

        param
    }

    fn to_init_clause(&self, param: &TokenStream) -> TokenStream {
        if let Some(ident) = self.to_ident() {
            quote! { #ident: #param }
        } else {
            quote! { #param }
        }
    }

    fn to_type_definition(&self, ty_mod: &TokenStream) -> TokenStream {
        let mut ty = self.ty().to_definition(ty_mod);

        if self.is_variable() {
            ty = quote! {
                mlang_rs::rt::opcode::Variable<#ty>
            };
        }

        if self.is_option() {
            ty = quote! { Option<#ty> };
        }

        ty
    }

    fn to_definition_clause(&self, vis: &TokenStream, ty: &TokenStream) -> TokenStream {
        let attrs = if self.is_option() {
            quote! { #[serde(skip_serializing_if = "Option::is_none")] }
        } else {
            quote! {}
        };

        if let Some(ident) = self.to_ident() {
            quote! { #attrs #vis #ident: #ty }
        } else {
            quote! { #attrs #vis #ty }
        }
    }

    fn to_ident(&self) -> Option<TokenStream> {
        self.ident().map(|ident| ident.to_field_name())
    }
}

/// A trait to help mapping [`Node`] to rust struct.
pub trait ComplexTypeMapping: CommentMapping {
    /// Generate rust struct ident for [`Node`] .
    fn to_ident(&self) -> TokenStream;

    /// Generate `;` token if necessary
    fn to_semi_token(&self) -> TokenStream;

    /// Generate struct body clause.
    fn to_struct_body(&self, fields: impl AsRef<[TokenStream]>) -> TokenStream;
}

impl CommentMapping for Node {
    fn to_comment(&self) -> TokenStream {
        self.comments.iter().map(|c| c.to_comment()).collect()
    }
}

impl CommentMapping for Enum {
    fn to_comment(&self) -> TokenStream {
        self.comments.iter().map(|c| c.to_comment()).collect()
    }
}

impl ComplexTypeMapping for Node {
    fn to_ident(&self) -> TokenStream {
        self.ident.to_type_name()
    }

    fn to_semi_token(&self) -> TokenStream {
        if self.is_tuple() {
            quote! {;}
        } else {
            quote! {}
        }
    }

    fn to_struct_body(&self, fields: impl AsRef<[TokenStream]>) -> TokenStream {
        let fields = fields.as_ref();

        if fields.is_empty() {
            return quote! {};
        }

        if self.is_tuple() {
            quote! { (#(#fields),*) }
        } else {
            quote! { {#(#fields),*} }
        }
    }
}

impl ComplexTypeMapping for Enum {
    fn to_ident(&self) -> TokenStream {
        self.ident.to_type_name()
    }

    fn to_semi_token(&self) -> TokenStream {
        quote! {}
    }

    fn to_struct_body(&self, fields: impl AsRef<[TokenStream]>) -> TokenStream {
        let fields = fields.as_ref();

        assert!(!fields.is_empty(), "enum fields is empty.");

        if fields.is_empty() {
            return quote! {};
        }

        quote! { {#(#fields),*} }
    }
}

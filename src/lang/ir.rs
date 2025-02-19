//! The intermediate representation for `mlang`.

use std::slice::{Iter, IterMut};

use parserc::Span;

/// Ident token for struct name or field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Ident(
    /// The whole span of the ident.
    pub Span,
    /// name of the ident.
    pub String,
);

/// A line of comment.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Comment(
    /// The whole span includes prefx '///'
    pub Span,
    /// Comment content.
    pub String,
);

/// Literal string expr.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LitStr(
    /// The whole span of the literal string.
    pub Span,
    /// The content of literal string.
    pub String,
);

/// literal unsigned integer.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LitUint(pub usize, pub Span);

impl LitUint {
    /// Create a new `Ident` with span.
    pub fn from_span(name: usize, span: Span) -> Self {
        Self(name.into(), span)
    }
}

/// Call expr of a property.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CallExpr {
    /// The whole span of the call.
    pub span: Span,
    /// Call target name.
    pub target: Ident,
    /// Parameter list of the attribute call.
    pub params: Vec<LitStr>,
}

/// A line of property
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Property {
    pub span: Span,
    /// call-expr list.
    pub calls: Vec<CallExpr>,
}

/// Defines a `type` of one field.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Type {
    Bool(Span),
    String(Span),
    Byte(Span),
    Ubyte(Span),
    Short(Span),
    Ushort(Span),
    Int(Span),
    Uint(Span),
    Long(Span),
    Ulong(Span),
    Float(Span),
    Double(Span),
    /// A data/enum reference.
    Data(Ident),
    /// This type is `vec[T]`.
    ListOf(Box<Type>, Span),
    /// Array type `[T;30]`
    ArrayOf(Box<Type>, LitUint, Span),
}

impl Type {
    /// Return span of the type.
    pub fn span(&self) -> &Span {
        match self {
            Type::Bool(span) => span,
            Type::String(span) => span,
            Type::Byte(span) => span,
            Type::Ubyte(span) => span,
            Type::Short(span) => span,
            Type::Ushort(span) => span,
            Type::Int(span) => span,
            Type::Uint(span) => span,
            Type::Long(span) => span,
            Type::Ulong(span) => span,
            Type::Float(span) => span,
            Type::Double(span) => span,
            Type::Data(ident) => &ident.0,
            Type::ListOf(_, span) => span,
            Type::ArrayOf(_, _, span) => span,
        }
    }
}

/// Named field for nodes.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct NamedField {
    /// The whole span of the named field.
    pub span: Span,
    /// comment of this field.
    pub comments: Vec<Comment>,
    /// custom propert list.
    pub properties: Vec<Property>,
    /// filed name,
    pub ident: Ident,
    /// The type of this field.
    pub ty: Type,
}

/// Unnamed field for tuple body.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct UnnamedField {
    /// The whole span of the unnamed field.
    pub span: Span,
    /// comment of this field.
    pub comments: Vec<Comment>,
    /// custom propert list.
    pub properties: Vec<Property>,
    /// The type of this field.
    pub ty: Type,
}

/// The property of one node or attr.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Fields {
    None,
    Named(Vec<NamedField>),
    Unnamed(Vec<UnnamedField>),
}

impl Fields {
    /// Check if this node is a tuple node.
    pub fn is_tuple(&self) -> bool {
        match &self {
            Fields::Named(_) => false,
            _ => true,
        }
    }

    pub fn is_empty(&self) -> bool {
        match &self {
            Fields::None => true,
            _ => false,
        }
    }
}

impl Fields {
    pub fn append(self, append: Fields) -> Result<Fields, Fields> {
        match self {
            Self::None => Ok(append),
            Self::Named(mut vec) => match append {
                Self::None => Ok(Fields::Named(vec)),
                Self::Named(mut append) => {
                    vec.append(&mut append);

                    Ok(Fields::Named(vec))
                }
                Self::Unnamed(vec) => Err(Self::Unnamed(vec)),
            },
            Self::Unnamed(mut vec) => match append {
                Self::None => Ok(Fields::Unnamed(vec)),
                Self::Unnamed(mut append) => {
                    vec.append(&mut append);

                    Ok(Fields::Unnamed(vec))
                }
                Self::Named(vec) => Err(Self::Named(vec)),
            },
        }
    }
    /// Create a immutable iterator over fields.
    pub fn iter(&self) -> FieldIter<'_> {
        match self {
            Fields::None => FieldIter::None,
            Fields::Named(vec) => FieldIter::Named(vec.iter()),
            Fields::Unnamed(vec) => FieldIter::Unnamed(vec.iter()),
        }
    }

    /// Create a mutable iterator over fields.
    pub fn iter_mut(&mut self) -> FieldIterMut<'_> {
        match self {
            Fields::None => FieldIterMut::None,
            Fields::Named(vec) => FieldIterMut::Named(vec.iter_mut()),
            Fields::Unnamed(vec) => FieldIterMut::Unnamed(vec.iter_mut()),
        }
    }
}

pub enum FieldIterMut<'a> {
    None,
    Named(IterMut<'a, NamedField>),
    Unnamed(IterMut<'a, UnnamedField>),
}

/// The item type used by FieldIter.
pub enum FieldMut<'a> {
    Named(&'a mut NamedField),
    Unnamed(&'a mut UnnamedField),
}

impl<'a> FieldMut<'a> {
    /// Get comments
    pub fn comments(&mut self) -> &mut [Comment] {
        match self {
            FieldMut::Named(named_field) => &mut named_field.comments,
            FieldMut::Unnamed(unnamed_field) => &mut unnamed_field.comments,
        }
    }

    /// Get properties
    pub fn properties(&mut self) -> &mut [Property] {
        match self {
            FieldMut::Named(named_field) => &mut named_field.properties,
            FieldMut::Unnamed(unnamed_field) => &mut unnamed_field.properties,
        }
    }

    /// Get type.
    pub fn ty(&mut self) -> &mut Type {
        match self {
            FieldMut::Named(named_field) => &mut named_field.ty,
            FieldMut::Unnamed(unnamed_field) => &mut unnamed_field.ty,
        }
    }

    /// Get ident.
    pub fn ident(&self) -> Option<&Ident> {
        match self {
            FieldMut::Named(named_field) => Some(&named_field.ident),
            FieldMut::Unnamed(_) => None,
        }
    }
}

impl<'a> Iterator for FieldIterMut<'a> {
    type Item = FieldMut<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            FieldIterMut::None => None,
            FieldIterMut::Named(iter) => iter.next().map(|next| FieldMut::Named(next)),
            FieldIterMut::Unnamed(iter) => iter.next().map(|next| FieldMut::Unnamed(next)),
        }
    }
}

/// The field immutable iterator.
pub enum FieldIter<'a> {
    None,
    Named(Iter<'a, NamedField>),
    Unnamed(Iter<'a, UnnamedField>),
}

/// The item type used by FieldIter.
pub enum Field<'a> {
    Named(&'a NamedField),
    Unnamed(&'a UnnamedField),
}

impl<'a> Field<'a> {
    /// Get comments
    pub fn comments(&self) -> &[Comment] {
        match self {
            Field::Named(named_field) => &named_field.comments,
            Field::Unnamed(unnamed_field) => &unnamed_field.comments,
        }
    }

    /// Get properties
    pub fn properties(&self) -> &[Property] {
        match self {
            Field::Named(named_field) => &named_field.properties,
            Field::Unnamed(unnamed_field) => &unnamed_field.properties,
        }
    }

    /// Get type.
    pub fn ty(&self) -> &Type {
        match self {
            Field::Named(named_field) => &named_field.ty,
            Field::Unnamed(unnamed_field) => &unnamed_field.ty,
        }
    }

    /// Get ident.
    pub fn ident(&self) -> Option<&Ident> {
        match self {
            Field::Named(named_field) => Some(&named_field.ident),
            Field::Unnamed(_) => None,
        }
    }

    pub fn is_option(&self) -> bool {
        for property in self.properties() {
            for callexpr in &property.calls {
                if callexpr.target.1 == "option" {
                    return true;
                }
            }
        }

        return false;
    }

    pub fn is_variable(&self) -> bool {
        for property in self.properties() {
            for callexpr in &property.calls {
                if callexpr.target.1 == "variable" {
                    return true;
                }
            }
        }

        return false;
    }

    pub fn is_init_field(&self) -> bool {
        for property in self.properties() {
            for callexpr in &property.calls {
                if callexpr.target.1 == "init" {
                    return true;
                }
            }
        }

        return false;
    }

    /// Serialize and deserialize this field with the given name instead of its Rust name.
    pub fn rename(&self) -> Option<&str> {
        for prop in self.properties() {
            for param in &prop.calls {
                if param.target.1 == "rename" {
                    return param.params.first().map(|v| v.1.as_str());
                }
            }
        }

        None
    }
}

impl<'a> Iterator for FieldIter<'a> {
    type Item = Field<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        match self {
            FieldIter::None => None,
            FieldIter::Named(iter) => iter.next().map(|next| Field::Named(next)),
            FieldIter::Unnamed(iter) => iter.next().map(|next| Field::Unnamed(next)),
        }
    }
}

/// Defines a node.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Node {
    /// The span of the whole node block.
    pub span: Span,
    /// comment of this node
    pub comments: Vec<Comment>,
    /// The ident of the mixin data.
    pub mixin: Option<Ident>,
    /// custom propert list.
    pub properties: Vec<Property>,
    /// The identifier name of this node
    pub ident: Ident,
    /// the non-inherited properties
    pub fields: Fields,
}

impl Node {
    /// Check if this node is a tuple node.
    pub fn is_tuple(&self) -> bool {
        self.fields.is_tuple()
    }

    /// Serialize and deserialize this node with the given name instead of its Rust name.
    pub fn rename(&self) -> Option<&str> {
        for prop in &self.properties {
            for param in &prop.calls {
                if param.target.1 == "rename" {
                    return param.params.first().map(|v| v.1.as_str());
                }
            }
        }

        None
    }
}

/// Defines an enum data.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Enum {
    pub span: Span,
    /// comment of this node
    pub comments: Vec<Comment>,
    /// custom propert list.
    pub properties: Vec<Property>,
    /// The identifier name of this element.
    pub ident: Ident,
    /// the non-inherited properties
    pub fields: Vec<Node>,
}

impl Enum {
    /// Serialize and deserialize this node with the given name instead of its Rust name.
    pub fn rename(&self) -> Option<&str> {
        for prop in &self.properties {
            for param in &prop.calls {
                if param.target.1 == "rename" {
                    return param.params.first().map(|v| v.1.as_str());
                }
            }
        }

        None
    }
}

/// Defines a group.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Group {
    /// The span of group.
    pub span: Span,
    /// comment of this field.
    pub comments: Vec<Comment>,
    /// custom propert list.
    pub properties: Vec<Property>,
    /// The name of the group.
    pub ident: Ident,
    /// The content of the group.
    pub children: Vec<Ident>,
}

/// Defines apply to link
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ApplyTo {
    /// comment of this field.
    pub comments: Vec<Comment>,
    /// custom propert list.
    pub properties: Vec<Property>,
    /// The span of group.
    pub span: Span,
    /// From target.
    pub from: Vec<Ident>,
    /// To target
    pub to: Vec<Ident>,
}

/// Defines childof link
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChildrenOf {
    /// comment of this field.
    pub comments: Vec<Comment>,
    /// custom propert list.
    pub properties: Vec<Property>,
    /// The span of group.
    pub span: Span,
    /// children group.
    pub from: Vec<Ident>,
    /// parent group.
    pub to: Vec<Ident>,
}

/// Defines a vglang metadata stat.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum Stat {
    Element(Box<Node>),
    Leaf(Box<Node>),
    Attr(Box<Node>),
    Mixin(Box<Node>),
    Data(Box<Node>),
    Enum(Box<Enum>),
    Group(Box<Group>),
    ApplyTo(Box<ApplyTo>),
    ChildrenOf(Box<ChildrenOf>),
}

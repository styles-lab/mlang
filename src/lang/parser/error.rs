/// Error returns by `parser` mod
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum ParseError {
    #[error("Expect stat.")]
    Unparsed,
    #[error("Syntax error of literal number, {0}")]
    Uint(UnitKind),
    #[error("Syntax error of property, {0}")]
    Prop(PropKind),

    #[error("Syntax error of call exp, {0}")]
    Call(CallKind),

    #[error("Syntax error of type declaration, {0}")]
    Type(TypeKind),

    #[error("Syntax error of enum, {0}")]
    Enum(EnumKind),

    #[error("Syntax error of fields, {0}")]
    Fields(FieldsKind),

    #[error("Syntax error of name field, {0}")]
    NamedField(NamedFieldKind),

    #[error("Syntax error of unamed field, expect field type declaration.")]
    UnnamedField,

    #[error("Syntax error of node, {0}")]
    Node(NodeKind),

    #[error("Syntax error of group, {0}")]
    Group(GroupKind),

    #[error("Syntax error of tuple, {0}")]
    Tuple(TupleKind),

    #[error("Syntax error of apply ... to ..., {0}")]
    ApplyTo(ApplyToKind),

    #[error("Syntax error of children ... of ..., {0}")]
    ChildrenOf(ChildrenOfKind),
}

/// Error kind of parsing tuple `(ident,...)` stat.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum TupleKind {
    #[error("expect `(`.")]
    BodyStart,
    #[error("expect `)`.")]
    BodyEnd,
}

/// Error kind of parsing children .. of ... stat.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum ChildrenOfKind {
    #[error("expect keyword `of`.")]
    Of,
    #[error(
        "expect an `ident` or a group of idents `(ident,...)` following by `children` keyword."
    )]
    From,
    #[error("expect an `ident` or a group of idents `(ident,...)` following by `of` keyword.")]
    To,
    #[error("expect `;`.")]
    End,
}

/// Error kind of parsing apply .. to ... stat.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum ApplyToKind {
    #[error("expect keyword `to`.")]
    To,
    #[error("expect an `ident` or a group of idents `(ident,...)` following by `to` keyword.")]
    Target,
    #[error("expect `;`.")]
    End,
}

/// Error kind of node parsing.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum GroupKind {
    #[error("expect `:=`.")]
    Assign,

    #[error("expect `;`.")]
    End,
}

/// Error kind of node parsing.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum NodeKind {
    #[error("epxect mixin `ident`.")]
    MixinIdent,

    #[error("epxect fields.")]
    Fields,

    #[error("expect `;`")]
    End,
}

/// Error kind of enum parsing.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum NamedFieldKind {
    #[error("expect value/type split char `:`")]
    SemiColons,

    #[error("expect field type declaration.")]
    Type,
}

/// Error kind of enum parsing.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum FieldsKind {
    #[error("expect end tag `{0}`")]
    EndTag(char),
}

/// Error kind of enum parsing.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum EnumKind {
    #[error("invalid enum ident.")]
    Ident,

    #[error("expect `{{`")]
    BodyStart,

    #[error("expect `}}`")]
    BodyEnd,
}

/// Error kind of unit parsing.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum TypeKind {
    #[error("unrecognized array length.")]
    Uint,

    #[error("miss `;` for array declaration.")]
    Semicolons,

    #[error("miss array/list start tag `[`")]
    SquareBracketStart,

    #[error("miss array/list end tag `]`")]
    SquareBracketEnd,

    #[error("miss data name.")]
    Data,
}

/// Error kind of unit parsing.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum CallKind {
    #[error("expect call expr parameter list end tag `)`")]
    ParamEnd,
}

/// Error kind of unit parsing.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum PropKind {
    #[error("expect property end tag `]`")]
    MissEnd,
}

/// Error kind of unit parsing.
#[derive(Debug, thiserror::Error, PartialEq, PartialOrd)]
pub enum UnitKind {
    #[error("miss hexadecimal body.")]
    MissBody,
}

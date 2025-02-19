use std::{
    marker::PhantomData,
    num::{ParseFloatError, ParseIntError},
};

use crate::rt::opcode::{Path, Target, Variable};

/// Error used by [`Visitor`]
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Error {
    #[error(transparent)]
    ParseIntError(#[from] ParseIntError),

    #[error(transparent)]
    ParseFloatError(#[from] ParseFloatError),

    #[error("Deserialize sequence out of range({0}), expect {1}")]
    OutOfRange(usize, usize),

    #[error("Unknown opcode type_id: {0}")]
    UnknownType(usize),

    #[error("Unknown opcode: {0}")]
    UnknownTypeName(String),

    #[error("Unexpect {0}")]
    Unexpect(Kind),

    #[error("Unknown variant `{1}` of enum({0})")]
    UnknownVariant(String, String),

    #[error("Unknown variant index({1}) of enum({0})")]
    UnknownVariantIndex(String, usize),
}

/// Unexpect kind .
#[derive(Debug, thiserror::Error, PartialEq)]
pub enum Kind {
    #[error("kind: bool.")]
    Bool,
    #[error("kind: string.")]
    String,
    #[error("kind: byte.")]
    Byte,
    #[error("kind: utype.")]
    Ubyte,
    #[error("kind: short.")]
    Short,
    #[error("kind: ushort.")]
    Ushort,
    #[error("kind: int.")]
    Int,
    #[error("kind: uint.")]
    Uint,
    #[error("kind: long.")]
    Long,
    #[error("kind: ulong.")]
    Ulong,
    #[error("kind: float.")]
    Float,
    #[error("kind: double.")]
    Double,
    #[error("kind: enum.")]
    Enum,
    #[error("kind: data.")]
    Data,
    #[error("kind: element.")]
    Element,
    #[error("kind: leaf.")]
    Leaf,
    #[error("kind: attr.")]
    Attr,
    #[error("kind: opcode({0}).")]
    Opcode(usize),

    #[error("kind: opcode({0}).")]
    NamedOpcode(String),
    #[error("kind: none.")]
    None,
    #[error("kind: some.")]
    Some,
    #[error("kind: seq.")]
    Seq,
    #[error("kind: variable.")]
    Variable,
    #[error("kind: Variable::Constant.")]
    Constant,
    #[error("kind: pop.")]
    Pop,
}

/// This trait represents a visitor that walks through a deserializer.
pub trait Visitor: Sized {
    /// The value produced by this visitor.
    type Value: 'static;

    fn is_element(&self, name: &str) -> bool {
        let _ = name;
        false
    }

    fn is_leaf(&self, name: &str) -> bool {
        let _ = name;
        false
    }

    /// The input contains a opcode.
    ///
    /// The default implementation fails with a type error.
    fn visit_opcode<D>(self, type_id: usize, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer,
    {
        let _ = type_id;
        let _ = deserializer;

        Err(Error::Unexpect(Kind::Opcode(type_id)).into())
    }

    /// The input contains a opcode with `name`.
    ///
    /// The default implementation fails with a type error.
    fn visit_opcode_with<D>(self, name: &str, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer,
    {
        let _ = name;
        let _ = deserializer;

        Err(Error::Unexpect(Kind::NamedOpcode(name.to_string())).into())
    }

    /// The input contains attrs with one opcode.
    ///
    /// The default implementation fails with a type error.
    fn visit_opcode_with_attrs<D>(
        self,
        name: &str,
        deserializer: D,
    ) -> Result<Vec<Self::Value>, D::Error>
    where
        D: AttrsNodeAccess,
    {
        let _ = name;
        let _ = deserializer;

        Err(Error::Unexpect(Kind::NamedOpcode(name.to_string())).into())
    }

    /// The input contains a pop opcode.
    fn visit_pop<E>(self) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        Err(Error::Unexpect(Kind::Pop).into())
    }

    /// The input contains a element node.
    fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer,
    {
        let _ = deserializer;

        Err(Error::Unexpect(Kind::Some).into())
    }

    /// The input contains a element node.
    fn visit_constant<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer,
    {
        let _ = deserializer;

        Err(Error::Unexpect(Kind::Constant).into())
    }

    /// The input contains a element node.
    fn visit_variable<E>(self, path: Path, target: Target) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = path;
        let _ = target;

        Err(Error::Unexpect(Kind::Variable).into())
    }

    /// The input contains a data.
    fn visit_node<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: NodeAccess,
    {
        let _ = deserializer;

        Err(Error::Unexpect(Kind::Data).into())
    }

    /// The input contains a enum data.
    fn visit_enum<D>(self, variant_index: usize, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: NodeAccess,
    {
        let _ = variant_index;
        let _ = deserializer;

        Err(Error::Unexpect(Kind::Enum).into())
    }

    /// The input contains a enum data.
    fn visit_enum_with<D>(
        self,
        variant_name: &str,
        deserializer: D,
    ) -> Result<Self::Value, D::Error>
    where
        D: NodeAccess,
    {
        let _ = variant_name;
        let _ = deserializer;

        Err(Error::Unexpect(Kind::Enum).into())
    }

    /// The input contains a enum data.
    fn visit_seq<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: SeqAccess,
    {
        let _ = deserializer;

        Err(Error::Unexpect(Kind::Seq).into())
    }

    /// The input contains a `string` value.
    fn visit_string<E>(self, value: &str) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = value;

        Err(Error::Unexpect(Kind::String).into())
    }

    /// The input contains a `bool` value.
    fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = value;

        Err(Error::Unexpect(Kind::Bool).into())
    }

    /// The input contains a `byte` value.
    fn visit_byte<E>(self, value: i8) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = value;

        Err(Error::Unexpect(Kind::Byte).into())
    }

    /// The input contains a `ubyte` value.
    fn visit_ubyte<E>(self, value: u8) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = value;

        Err(Error::Unexpect(Kind::Ubyte).into())
    }

    /// The input contains a `short` value.
    fn visit_short<E>(self, value: i16) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = value;

        Err(Error::Unexpect(Kind::Short).into())
    }

    /// The input contains a `ushort` value.
    fn visit_ushort<E>(self, value: u16) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = value;

        Err(Error::Unexpect(Kind::Ushort).into())
    }

    /// The input contains a `int` value.
    fn visit_int<E>(self, value: i32) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = value;

        Err(Error::Unexpect(Kind::Int).into())
    }

    /// The input contains a `uint` value.
    fn visit_uint<E>(self, value: u32) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = value;

        Err(Error::Unexpect(Kind::Uint).into())
    }

    /// The input contains a `long` value.
    fn visit_long<E>(self, value: i64) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = value;

        Err(Error::Unexpect(Kind::Long).into())
    }

    /// The input contains a `ulong` value.
    fn visit_ulong<E>(self, value: u64) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = value;

        Err(Error::Unexpect(Kind::Ulong).into())
    }

    /// The input contains a `float` value.
    fn visit_float<E>(self, value: f32) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = value;

        Err(Error::Unexpect(Kind::Float).into())
    }

    /// The input contains a `double` value.
    fn visit_double<E>(self, value: f64) -> Result<Self::Value, E>
    where
        E: From<Error>,
    {
        let _ = value;

        Err(Error::Unexpect(Kind::Double).into())
    }
}

/// Trait to access a sequence value.
pub trait SeqAccess {
    type Error: From<Error>;

    /// This returns Ok(Some(value)) for the next value in the sequence, or Ok(None) if there are no more remaining items.
    fn next_item<T>(&mut self) -> Result<Option<T::Value>, Self::Error>
    where
        T: Deserialize;
}

/// Trait to access applied attrs.
pub trait AttrsNodeAccess {
    type Error: From<Error>;

    /// Returns a iterator over the attribute names.
    fn attrs(&self) -> impl Iterator<Item = &str>;

    /// This returns Ok(Some(value)) for the next value in the sequence, or Ok(None) if there are no more remaining items.
    fn deserialize_attr<V>(&mut self, name: &str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a element node.
    fn deserialize_node<V>(self, name: &str, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;
}

/// Trait to access a sequence value.
pub trait NodeAccess {
    type Error: From<Error>;

    /// Deserialize next filed.
    fn deserialize_field<T>(
        &mut self,
        ty: &str,
        index: usize,
        field_name: Option<&str>,
    ) -> Result<T::Value, Self::Error>
    where
        T: Deserialize;
}

/// A data format that can deserialize any data structure supported by `mlang`.
pub trait Deserializer {
    /// Error type used by this `deserializer`.
    type Error: From<Error> + 'static;

    /// derserialize a list of opcodes.
    fn deserialize_opcode<V>(self, visitor: V) -> Result<Option<Vec<V::Value>>, Self::Error>
    where
        V: Visitor;

    /// derserialize a element node.
    fn deserialize_element<V>(
        self,
        type_id: usize,
        name: &str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a element node.
    fn deserialize_leaf<V>(
        self,
        type_id: usize,
        name: &str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a element node.
    fn deserialize_attr<V>(
        self,
        type_id: usize,
        name: &str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a element node.
    fn deserialize_data<V>(
        self,
        type_id: usize,
        name: &str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a enum data.
    fn deserialize_enum<V>(
        self,
        type_id: usize,
        name: &str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a sequence data.
    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a option value.
    fn deserialize_option<V>(self, visitor: V) -> Result<Option<V::Value>, Self::Error>
    where
        V: Visitor;

    /// derserialize a variable value.
    fn deserialize_variable<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a string value.
    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a bool value.
    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a byte value.
    fn deserialize_byte<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a ubyte value.
    fn deserialize_ubyte<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a short value.
    fn deserialize_short<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a ushort value.
    fn deserialize_ushort<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a int value.
    fn deserialize_int<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a uint value.
    fn deserialize_uint<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;
    /// derserialize a long value.
    fn deserialize_long<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a ulong value.
    fn deserialize_ulong<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a float value.
    fn deserialize_float<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;

    /// derserialize a double value.
    fn deserialize_double<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor;
}

/// Implement this trait to support derserializing from any data format.
pub trait Deserialize: Sized {
    type Value: 'static;
    /// Derserialize this value from given `derserializer`.
    fn deserialize<D>(deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer;
}

impl Deserialize for String {
    type Value = String;
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer,
    {
        struct V;

        impl Visitor for V {
            type Value = String;

            fn visit_string<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: From<Error>,
            {
                Ok(value.to_string())
            }
        }

        deserializer.deserialize_string(V)
    }
}

impl Deserialize for bool {
    type Value = bool;
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer,
    {
        struct V;

        impl Visitor for V {
            type Value = bool;

            fn visit_string<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: From<Error>,
            {
                Ok(if value == "1" { true } else { false })
            }

            fn visit_bool<E>(self, value: bool) -> Result<Self::Value, E>
            where
                E: From<Error>,
            {
                Ok(value)
            }
        }

        deserializer.deserialize_bool(V)
    }
}

macro_rules! impl_deserilaize_num {
    ($ty:ident, $visit:ident, $deserialize:ident) => {
        impl Deserialize for $ty {
            type Value = $ty;

            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer,
            {
                struct V;

                impl Visitor for V {
                    type Value = $ty;

                    fn visit_string<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: From<Error>,
                    {
                        let value = value.parse::<$ty>().map_err(|err| Error::from(err))?;
                        Ok(value)
                    }

                    fn $visit<E>(self, value: $ty) -> Result<Self::Value, E>
                    where
                        E: From<Error>,
                    {
                        Ok(value)
                    }
                }

                deserializer.$deserialize(V)
            }
        }
    };
}

impl_deserilaize_num!(i8, visit_byte, deserialize_byte);
impl_deserilaize_num!(u8, visit_ubyte, deserialize_ubyte);
impl_deserilaize_num!(i16, visit_short, deserialize_short);
impl_deserilaize_num!(u16, visit_ushort, deserialize_ushort);
impl_deserilaize_num!(i32, visit_int, deserialize_int);
impl_deserilaize_num!(u32, visit_uint, deserialize_uint);
impl_deserilaize_num!(i64, visit_long, deserialize_long);
impl_deserilaize_num!(u64, visit_ulong, deserialize_ulong);
impl_deserilaize_num!(f32, visit_float, deserialize_float);
impl_deserilaize_num!(f64, visit_double, deserialize_double);

impl<'de, T> Deserialize for Option<T>
where
    T: Deserialize,
{
    type Value = Option<T::Value>;
    fn deserialize<D>(deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer,
    {
        struct V<T>(PhantomData<T>);

        impl<'de, T> Visitor for V<T>
        where
            T: Deserialize,
        {
            type Value = T::Value;

            fn visit_some<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer,
            {
                T::deserialize(deserializer)
            }
        }

        deserializer.deserialize_option(V::<T>(PhantomData::default()))
    }
}

impl<'de, T> Deserialize for Variable<T>
where
    T: Deserialize,
{
    type Value = Variable<T::Value>;
    fn deserialize<D>(deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer,
    {
        struct V<T>(PhantomData<T>);

        impl<'de, T> Visitor for V<T>
        where
            T: Deserialize,
        {
            type Value = Variable<T::Value>;

            fn visit_variable<E>(self, path: Path, target: Target) -> Result<Self::Value, E>
            where
                E: From<Error>,
            {
                Ok(Variable::Reference { path, target })
            }

            fn visit_constant<D>(self, deserializer: D) -> Result<Self::Value, D::Error>
            where
                D: Deserializer,
            {
                Ok(Variable::Constant(T::deserialize(deserializer)?))
            }
        }

        deserializer.deserialize_variable(V::<T>(PhantomData::default()))
    }
}

impl<'de, T> Deserialize for Vec<T>
where
    T: Deserialize,
{
    type Value = Vec<T::Value>;
    fn deserialize<D>(deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer,
    {
        struct V<T>(PhantomData<T>);

        impl<'de, T> Visitor for V<T>
        where
            T: Deserialize,
        {
            type Value = Vec<T::Value>;

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess,
            {
                let mut values = vec![];

                while let Some(value) = seq.next_item::<T>()? {
                    values.push(value);
                }

                Ok(values)
            }
        }

        deserializer.deserialize_seq(V::<T>(PhantomData::default()))
    }
}

impl<'de, T, const N: usize> Deserialize for [T; N]
where
    T: Deserialize,
{
    type Value = [T::Value; N];
    fn deserialize<D>(deserializer: D) -> Result<Self::Value, D::Error>
    where
        D: Deserializer,
    {
        struct V<T>(PhantomData<T>);

        impl<'de, T> Visitor for V<T>
        where
            T: Deserialize,
        {
            type Value = Vec<T::Value>;

            fn visit_seq<S>(self, mut seq: S) -> Result<Self::Value, S::Error>
            where
                S: SeqAccess,
            {
                let mut values = vec![];

                while let Some(value) = seq.next_item::<T>()? {
                    values.push(value);
                }

                Ok(values)
            }
        }

        let values = deserializer.deserialize_seq(V::<T>(PhantomData::default()))?;

        Ok(values
            .try_into()
            .map_err(|err: Vec<T::Value>| Error::OutOfRange(err.len(), N))?)
    }
}

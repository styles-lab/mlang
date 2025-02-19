use crate::rt::opcode::{Path, Target, Variable};

/// Serializer for Opcodes.
pub trait Serializer {
    type Error;
    /// Type returns by [`serialize_el`](Serializer::serialize_el),
    /// [`serialize_leaf`](Serializer::serialize_leaf) or [`serialize_attr`](Serializer::serialize_attr)
    /// for serializing the content of node.
    type SerializeNode: SerializeNode<Error = Self::Error>;

    /// Returns by [`serialize_seq`](Serializer::serialize_seq) to help serializing array of vector.
    type SerializeSeq: SerializeSeq<Error = Self::Error>;

    /// Serialize a element node.
    fn serialize_el(
        self,
        type_id: usize,
        name: &str,
        fields: usize,
    ) -> Result<Self::SerializeNode, Self::Error>;

    /// Serialize a leaf node.
    fn serialize_leaf(
        self,
        type_id: usize,
        name: &str,
        fields: usize,
    ) -> Result<Self::SerializeNode, Self::Error>;

    /// Serialize a attr node.
    fn serialize_attr(
        self,
        type_id: usize,
        name: &str,
        fields: usize,
    ) -> Result<Self::SerializeNode, Self::Error>;

    /// Serialize a data.
    fn serialize_data(
        self,
        type_id: usize,
        name: &str,
        fields: usize,
    ) -> Result<Self::SerializeNode, Self::Error>;

    /// Serialize a enum data.
    fn serialize_enum(
        self,
        type_id: usize,
        name: &str,
        variant: &str,
        variant_index: usize,
        fields: usize,
    ) -> Result<Self::SerializeNode, Self::Error>;

    /// Serialize vglang `vec[T]` or `[T;N]`
    fn serialize_seq(self, len: usize) -> Result<Self::SerializeSeq, Self::Error>;

    /// Serialize vglang `string`.
    fn serialize_bool(self, value: bool) -> Result<(), Self::Error>;

    /// Serialize vglang `string`.
    fn serialize_string(self, value: &str) -> Result<(), Self::Error>;

    /// Serialize vglang `byte`.
    fn serialize_byte(self, value: i8) -> Result<(), Self::Error>;

    /// Serialize vglang `ubyte`.
    fn serialize_ubyte(self, value: u8) -> Result<(), Self::Error>;

    /// Serialize vglang `byte`.
    fn serialize_short(self, value: i16) -> Result<(), Self::Error>;

    /// Serialize vglang `ubyte`.
    fn serialize_ushort(self, value: u16) -> Result<(), Self::Error>;

    /// Serialize vglang `int`.
    fn serialize_int(self, value: i32) -> Result<(), Self::Error>;

    /// Serialize vglang `uint`.
    fn serialize_uint(self, value: u32) -> Result<(), Self::Error>;

    /// Serialize vglang `long`.
    fn serialize_long(self, value: i64) -> Result<(), Self::Error>;

    /// Serialize vglang `ulong`.
    fn serialize_ulong(self, value: u64) -> Result<(), Self::Error>;

    /// Serialize vglang `long`.
    fn serialize_float(self, value: f32) -> Result<(), Self::Error>;

    /// Serialize vglang `double`.
    fn serialize_double(self, value: f64) -> Result<(), Self::Error>;

    /// Serialize a none value.
    fn serialize_none(self) -> Result<(), Self::Error>;

    /// Serialize a none value.
    fn serialize_variable(self, path: &Path, target: &Target) -> Result<(), Self::Error>;

    /// Serialize pop directive.
    fn serialize_pop(self) -> Result<(), Self::Error>;
}

pub trait SerializeSeq {
    type Error;

    /// Serialize next item.
    fn next_item<T>(&mut self, value: &T) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize;

    /// Finish serializing a sequence.
    fn finish(self) -> Result<(), Self::Error>;
}

/// A trait to help serialzing a node.
pub trait SerializeNode {
    /// Error type returns by this trait.
    type Error;

    /// Serialize a field.
    fn serialize_field<T>(
        &mut self,
        index: usize,
        name: Option<&str>,
        value: &T,
    ) -> Result<(), Self::Error>
    where
        T: ?Sized + Serialize;

    /// Finish serializing a sequence.
    fn finish(self) -> Result<(), Self::Error>;
}

/// A node/enum must implement this trait to support serde framework.
pub trait Serialize {
    /// serialize self with `serializer`.
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer;
}

impl Serialize for bool {
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_bool(*self)
    }
}

impl Serialize for String {
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_string(&self)
    }
}

impl Serialize for i8 {
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_byte(*self)
    }
}

impl Serialize for u8 {
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_ubyte(*self)
    }
}

impl Serialize for i16 {
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_short(*self)
    }
}

impl Serialize for u16 {
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_ushort(*self)
    }
}

impl Serialize for i32 {
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_int(*self)
    }
}

impl Serialize for u32 {
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_uint(*self)
    }
}

impl Serialize for i64 {
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_long(*self)
    }
}

impl Serialize for u64 {
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_ulong(*self)
    }
}

impl Serialize for f32 {
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_float(*self)
    }
}

impl Serialize for f64 {
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_double(*self)
    }
}

impl<T> Serialize for Option<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        match self {
            Some(v) => v.serialize(serializer),
            None => serializer.serialize_none(),
        }
    }
}

impl<T> Serialize for Variable<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        match self {
            Variable::Constant(v) => v.serialize(serializer),
            Variable::Reference { path, target } => serializer.serialize_variable(path, target),
        }
    }
}

impl<T> Serialize for Vec<T>
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(self.len())?;

        for item in self.iter() {
            seq.next_item(item)?;
        }

        seq.finish()
    }
}

impl<T, const N: usize> Serialize for [T; N]
where
    T: Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<(), S::Error>
    where
        S: Serializer,
    {
        let mut seq = serializer.serialize_seq(self.len())?;

        for item in self.iter() {
            seq.next_item(item)?;
        }

        seq.finish()
    }
}

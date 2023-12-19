mod array;
mod statement;
mod table;
mod value;

use crate::error::{ParseEntryError, ParseItemError, ParseStringError, UnknownError};
use crate::{Item, VdfError};
pub use array::Array;
pub use statement::Statement;
use std::any::type_name;
use std::collections::HashMap;
use std::fmt::Formatter;
use std::slice;
pub use table::Table;
pub use value::Value;

/// The kinds of entry.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
#[serde(untagged)]
pub enum Entry {
    /// A table.
    Table(Table),

    /// An array (entries with the same key).
    Array(Array),

    /// A value.
    Value(Value),

    /// A statement (the values starting with #).
    Statement(Statement),
}

impl From<Item<'_>> for Entry {
    fn from(item: Item) -> Self {
        match item {
            Item::Item { content, .. } => Entry::Value(content.into()),
            Item::Statement { content, .. } => Entry::Statement(content.into()),
        }
    }
}

impl Entry {
    /// Lookup an entry with a path.
    pub fn lookup<S: AsRef<str>>(&self, path: S) -> Option<&Entry> {
        let mut current = self;

        for name in path.as_ref().split('.') {
            if let Some(entry) = current.get(name.trim()) {
                current = entry;
            } else {
                return None;
            }
        }

        Some(current)
    }

    /// Try to get the named entry.
    pub fn get<S: AsRef<str>>(&self, name: S) -> Option<&Entry> {
        match self {
            Entry::Table(value) => value.get(name.as_ref()),

            Entry::Array(value) => name
                .as_ref()
                .parse::<usize>()
                .ok()
                .and_then(|i| value.get(i)),

            _ => None,
        }
    }

    /// Try to convert the entry to the given type.
    pub fn to<T: ParseItem>(self) -> Result<T, ParseEntryError> {
        T::from_entry(self)
    }

    /// Try to take the entry as a table.
    pub fn as_table(&self) -> Option<&Table> {
        if let Entry::Table(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Try to take the entry as a slice.
    pub fn as_slice(&self) -> Option<&[Entry]> {
        if let Entry::Array(value) = self {
            Some(value.as_slice())
        } else {
            unsafe { Some(slice::from_raw_parts(self, 1)) }
        }
    }

    /// Try to take the entry as a statement.
    pub fn as_statement(&self) -> Option<&Statement> {
        if let Entry::Statement(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Try to take the entry as a value.
    pub fn as_value(&self) -> Option<&Value> {
        if let Entry::Value(value) = self {
            Some(value)
        } else {
            None
        }
    }

    /// Try to take the entry as a string.
    pub fn as_str(&self) -> Option<&str> {
        match self {
            Entry::Value(value) => Some(value),

            Entry::Statement(value) => Some(value),

            _ => None,
        }
    }
}

/// Parsable types.
pub trait ParseItem: Sized {
    /// Try to cast the entry into a concrete type
    fn from_entry(entry: Entry) -> Result<Self, ParseEntryError> {
        let string = match entry.as_str() {
            Some(string) => string,
            None => {
                return Err(ParseEntryError::new(type_name::<Self>(), entry));
            }
        };
        Self::from_str(string).map_err(|e| ParseEntryError::new(e.ty, entry))
    }

    /// Try to cast the item into a concrete type
    fn from_item(item: Item) -> Result<Self, ParseItemError> {
        Self::from_str(item.as_str()).map_err(|e| ParseItemError::new(e.ty, item))
    }

    /// Try to cast the string into a concrete type
    fn from_str(item: &str) -> Result<Self, ParseStringError>;
}

macro_rules! from_str {
	(for) => ();

	(for $ty:ident $($rest:tt)*) => (
		from_str!($ty);
		from_str!(for $($rest)*);
	);

	($ty:ident) => (
		impl ParseItem for $ty {
			fn from_entry(entry: Entry) -> Result<Self, ParseEntryError> {
                let string = match entry.as_str() {
                    Some(string) => string,
                    None => {
                        return Err(ParseEntryError::new(type_name::<Self>(), entry));
                    }
                };
				string.parse::<$ty>().map_err(|_| ParseEntryError::new(type_name::<Self>(), entry))
			}

            fn from_item(item: Item) -> Result<Self, ParseItemError> {
                item.as_str().parse::<$ty>().map_err(|_| ParseItemError::new(type_name::<Self>(), item))
            }

            fn from_str(item: &str) -> Result<Self, ParseStringError> {
                item.parse::<$ty>().map_err(|_| ParseStringError::new(type_name::<Self>(), item))
            }
		}
	);
}

use crate::entry::array::ArraySeq;
use crate::entry::table::TableSeq;
use serde::de::{DeserializeSeed, EnumAccess, Error, MapAccess, SeqAccess, VariantAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6};
from_str!(for IpAddr Ipv4Addr Ipv6Addr SocketAddr SocketAddrV4 SocketAddrV6);
from_str!(for i8 i16 i32 i64 isize u8 u16 u32 u64 usize f32 f64);

impl ParseItem for bool {
    fn from_str(item: &str) -> Result<Self, ParseStringError> {
        match item {
            "0" => Ok(false),
            "1" => Ok(true),
            v => v
                .parse::<bool>()
                .map_err(|_| ParseStringError::new(type_name::<Self>(), item)),
        }
    }
}

impl ParseItem for String {
    fn from_entry(entry: Entry) -> Result<Self, ParseEntryError> {
        match entry {
            Entry::Table(entry) => Err(ParseEntryError::new(
                type_name::<Self>(),
                Entry::Table(entry),
            )),
            Entry::Array(entry) => Err(ParseEntryError::new(
                type_name::<Self>(),
                Entry::Array(entry),
            )),
            Entry::Statement(statement) => Ok(statement.into()),
            Entry::Value(value) => Ok(value.into()),
        }
    }

    fn from_item(item: Item) -> Result<Self, ParseItemError> {
        Ok(item.into_content().into())
    }

    fn from_str(item: &str) -> Result<Self, ParseStringError> {
        Ok(item.into())
    }
}

impl<T: ParseItem> ParseItem for Option<T> {
    fn from_entry(entry: Entry) -> Result<Self, ParseEntryError> {
        T::from_entry(entry).map(Some)
    }

    fn from_item(item: Item) -> Result<Self, ParseItemError> {
        T::from_item(item).map(Some)
    }

    fn from_str(item: &str) -> Result<Self, ParseStringError> {
        T::from_str(item).map(Some)
    }
}

impl<'de> Deserialize<'de> for Entry {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct EntryVisitor;

        impl<'v> Visitor<'v> for EntryVisitor {
            type Value = Entry;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "any string like value or group")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Entry::Value(v.to_string().into()))
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Entry::Value(v.to_string().into()))
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(Entry::Value(v.to_string().into()))
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if v.starts_with('#') {
                    Ok(Entry::Statement(v.to_string().into()))
                } else {
                    Ok(Entry::Value(v.to_string().into()))
                }
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                if v.starts_with('#') {
                    Ok(Entry::Statement(v.into()))
                } else {
                    Ok(Entry::Value(v.into()))
                }
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: Error,
            {
                let v = if v { "1" } else { "0" };
                Ok(Entry::Value(v.to_string().into()))
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'v>,
            {
                let mut res = HashMap::new();

                while let Some(entry) = map.next_entry()? {
                    res.insert(entry.0, entry.1);
                }

                Ok(Entry::Table(res.into()))
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'v>,
            {
                let mut res = Vec::new();

                while let Some(entry) = seq.next_element()? {
                    res.push(entry);
                }

                Ok(Entry::Array(res.into()))
            }
        }

        deserializer.deserialize_any(EntryVisitor)
    }
}

impl<'de> Deserializer<'de> for Entry {
    type Error = VdfError;

    fn deserialize_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Table(table) => visitor.visit_map(TableSeq::new(table)),
            Entry::Array(array) => visitor.visit_seq(ArraySeq::new(array)),
            Entry::Value(val) => val.deserialize_any(visitor),
            Entry::Statement(val) => visitor.visit_string(val.into()),
        }
    }

    fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_bool(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_bool(visitor),
            _ => Err(UnknownError::from("bool").into()),
        }
    }

    fn deserialize_i8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_i8(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_i8(visitor),
            _ => Err(UnknownError::from("i8").into()),
        }
    }

    fn deserialize_i16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_i16(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_i16(visitor),
            _ => Err(UnknownError::from("i16").into()),
        }
    }

    fn deserialize_i32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_i32(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_i32(visitor),
            _ => Err(UnknownError::from("i32").into()),
        }
    }

    fn deserialize_i64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_i64(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_i64(visitor),
            _ => Err(UnknownError::from("i64").into()),
        }
    }

    fn deserialize_u8<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_u8(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_u8(visitor),
            _ => Err(UnknownError::from("u8").into()),
        }
    }

    fn deserialize_u16<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_u16(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_u16(visitor),
            _ => Err(UnknownError::from("u16").into()),
        }
    }

    fn deserialize_u32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_u32(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_u32(visitor),
            _ => Err(UnknownError::from("u32").into()),
        }
    }

    fn deserialize_u64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_u64(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_u64(visitor),
            _ => Err(UnknownError::from("u64").into()),
        }
    }

    fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_f32(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_f32(visitor),
            _ => Err(UnknownError::from("f32").into()),
        }
    }

    fn deserialize_f64<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_f64(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_f64(visitor),
            _ => Err(UnknownError::from("f64").into()),
        }
    }

    fn deserialize_char<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_char(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_char(visitor),
            _ => Err(UnknownError::from("char").into()),
        }
    }

    fn deserialize_str<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_str(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_str(visitor),
            _ => Err(UnknownError::from("str").into()),
        }
    }

    fn deserialize_string<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_string(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_string(visitor),
            _ => Err(UnknownError::from("string1").into()),
        }
    }

    fn deserialize_bytes<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_bytes(visitor),
            _ => Err(UnknownError::from("bytes").into()),
        }
    }

    fn deserialize_byte_buf<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_bool(visitor),
            _ => Err(UnknownError::from("bytes buf").into()),
        }
    }

    fn deserialize_option<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_option(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_option(visitor),
            _ => Err(UnknownError::from("option").into()),
        }
    }

    fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_unit(visitor),
            Entry::Statement(val) => Value::from(val).deserialize_unit(visitor),
            _ => Err(UnknownError::from("unit").into()),
        }
    }

    fn deserialize_unit_struct<V>(
        self,
        name: &'static str,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Value(val) => val.deserialize_unit_struct(name, visitor),
            Entry::Statement(val) => Value::from(val).deserialize_unit_struct(name, visitor),
            _ => Err(UnknownError::from("unit_struct").into()),
        }
    }

    fn deserialize_newtype_struct<V>(
        self,
        _name: &'static str,
        _visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        todo!()
    }

    fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Array(arr) => visitor.visit_seq(ArraySeq::new(arr)),
            _ => Err(UnknownError::from("array2").into()),
        }
    }

    fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Array(arr) => visitor.visit_seq(ArraySeq::new(arr)),
            _ => Err(UnknownError::from("tuple").into()),
        }
    }

    fn deserialize_tuple_struct<V>(
        self,
        _name: &'static str,
        _len: usize,
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Array(arr) => visitor.visit_seq(ArraySeq::new(arr)),
            _ => Err(UnknownError::from("tuple_struct").into()),
        }
    }

    fn deserialize_map<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        match self {
            Entry::Table(table) => visitor.visit_map(TableSeq::new(table)),
            _ => Err(UnknownError::from("map").into()),
        }
    }

    fn deserialize_struct<V>(
        self,
        _name: &'static str,
        _fields: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_map(visitor)
    }

    fn deserialize_enum<V>(
        self,
        _name: &'static str,
        _variants: &'static [&'static str],
        visitor: V,
    ) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        struct EnVarAccess {
            variant: Value,
            value: Entry,
        }
        struct EnValAccess {
            value: Entry,
        }

        impl<'de> EnumAccess<'de> for EnVarAccess {
            type Error = VdfError;
            type Variant = EnValAccess;

            fn variant_seed<V>(self, seed: V) -> Result<(V::Value, Self::Variant), Self::Error>
            where
                V: DeserializeSeed<'de>,
            {
                seed.deserialize(self.variant)
                    .map(|v| (v, EnValAccess { value: self.value }))
            }
        }

        impl<'de> VariantAccess<'de> for EnValAccess {
            type Error = VdfError;

            fn unit_variant(self) -> Result<(), Self::Error> {
                Err(UnknownError::from("unit").into())
            }

            fn newtype_variant_seed<T>(self, seed: T) -> Result<T::Value, Self::Error>
            where
                T: DeserializeSeed<'de>,
            {
                seed.deserialize(self.value)
            }

            fn tuple_variant<V>(self, _len: usize, visitor: V) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                self.value.deserialize_seq(visitor)
            }

            fn struct_variant<V>(
                self,
                _fields: &'static [&'static str],
                visitor: V,
            ) -> Result<V::Value, Self::Error>
            where
                V: Visitor<'de>,
            {
                self.value.deserialize_map(visitor)
            }
        }

        match self {
            Entry::Table(table) if table.len() == 1 => {
                let (variant, value) = HashMap::from(table).into_iter().next().unwrap();
                visitor.visit_enum(EnVarAccess {
                    variant: variant.into(),
                    value,
                })
            }
            _ => Err(UnknownError::from("enum").into()),
        }
    }

    fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_string(visitor)
    }

    fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value, Self::Error>
    where
        V: Visitor<'de>,
    {
        self.deserialize_any(visitor)
    }
}

#[cfg(test)]
#[track_caller]
fn unwrap_err<T>(r: Result<T, crate::VdfError>) -> T {
    r.map_err(miette::Error::from).unwrap()
}

#[test]
fn test_serde_entry() {
    use maplit::hashmap;

    let j = r#"1"#;
    assert_eq!(Entry::Value("1".into()), unwrap_err(crate::from_str(j)));

    let j = r#""foo bar""#;
    assert_eq!(
        Entry::Value("foo bar".into()),
        unwrap_err(crate::from_str(j))
    );

    let j = r#"{foo bar}"#;

    assert_eq!(
        Entry::Table(hashmap! {"foo".into() => Entry::Value("bar".into())}.into()),
        unwrap_err(crate::from_str(j))
    );

    let j = r#""[1 2 3]""#;

    assert_eq!(
        Entry::Array(
            vec![
                Value::from("1").into(),
                Value::from("2").into(),
                Value::from("3").into()
            ]
            .into()
        ),
        unwrap_err(crate::from_str(j))
    );

    let j = r#""{1 2 3}""#;

    assert_eq!(
        Entry::Array(
            vec![
                Value::from("1").into(),
                Value::from("2").into(),
                Value::from("3").into()
            ]
            .into()
        ),
        unwrap_err(crate::from_str(j))
    );
}

pub(crate) fn string_is_array(string: &str) -> bool {
    (string.starts_with('[') && string.ends_with(']'))
        || (string.starts_with('{') && string.ends_with('}'))
}

mod array;
mod statement;
mod table;
mod value;

use crate::error::{ParseEntryError, ParseItemError, ParseStringError};
use crate::Item;
pub use array::Array;
pub use statement::Statement;
use std::any::type_name;
use std::slice;
pub use table::Table;
pub use value::Value;

/// The kinds of entry.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub enum Entry {
    /// A table.
    Table(Table),

    /// An array (entries with the same key).
    Array(Array),

    /// A statement (the values starting with #).
    Statement(Statement),

    /// A value.
    Value(Value),
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

use serde::Serialize;
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

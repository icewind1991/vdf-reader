use super::{Entry, Parse};
use serde::Serialize;
use std::borrow::Cow;
use std::ops::Deref;

#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Value(String);

impl From<Cow<'_, str>> for Value {
    fn from(value: Cow<'_, str>) -> Value {
        Value(value.into())
    }
}

impl Into<Entry> for Value {
    fn into(self) -> Entry {
        Entry::Value(self)
    }
}

impl Deref for Value {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl Value {
    /// Try to convert the value to the given type.
    pub fn to<T: Parse>(&self) -> Option<T> {
        T::parse(&self.0)
    }
}

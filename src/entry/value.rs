use super::Entry;
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

impl From<Value> for Entry {
    fn from(value: Value) -> Self {
        Entry::Value(value)
    }
}

impl From<Value> for String {
    fn from(value: Value) -> Self {
        value.0
    }
}

impl Deref for Value {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

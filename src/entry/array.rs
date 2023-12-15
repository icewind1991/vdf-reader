use super::Entry;
use serde::Serialize;
use std::ops::{Deref, DerefMut};

/// An array of entries (items that have the same key).
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Array(Vec<Entry>);

impl From<Entry> for Array {
    fn from(value: Entry) -> Self {
        Array(vec![value])
    }
}

impl From<Array> for Entry {
    fn from(array: Array) -> Self {
        Entry::Array(array)
    }
}

impl Deref for Array {
    type Target = Vec<Entry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Array {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

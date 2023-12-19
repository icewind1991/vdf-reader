use super::Entry;
use crate::entry::Value;
use crate::VdfError;
use serde::de::{DeserializeSeed, SeqAccess};
use serde::{Deserialize, Serialize};
use std::ops::{Deref, DerefMut};

/// An array of entries (items that have the same key).
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct Array(Vec<Entry>);

impl Array {
    pub(crate) fn from_space_separated(str: &str) -> Self {
        let items = str
            .split(' ')
            .filter(|part| !part.is_empty())
            .map(Value::from)
            .map(Entry::from)
            .collect();
        Array(items)
    }
}

impl From<Vec<Entry>> for Array {
    fn from(value: Vec<Entry>) -> Self {
        Array(value)
    }
}
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

pub(crate) struct ArraySeq {
    iter: std::vec::IntoIter<Entry>,
}

impl ArraySeq {
    pub(crate) fn new(array: Array) -> Self {
        ArraySeq {
            iter: array.0.into_iter(),
        }
    }
}

impl<'de> SeqAccess<'de> for ArraySeq {
    type Error = VdfError;

    fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>, Self::Error>
    where
        T: DeserializeSeed<'de>,
    {
        let next = match self.iter.next() {
            Some(next) => next,
            None => return Ok(None),
        };

        seed.deserialize(next).map(Some)
    }
}

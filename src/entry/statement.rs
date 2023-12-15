use super::Entry;
use serde::Serialize;
use std::borrow::Cow;
use std::ops::Deref;

/// A statement.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Statement(String);

impl From<Cow<'_, str>> for Statement {
    fn from(value: Cow<'_, str>) -> Self {
        Statement(value.into())
    }
}

impl Into<Entry> for Statement {
    fn into(self) -> Entry {
        Entry::Statement(self)
    }
}

impl Deref for Statement {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

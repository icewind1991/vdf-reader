use super::Entry;
use serde::{Deserialize, Serialize};
use std::borrow::Cow;
use std::ops::Deref;

/// A statement.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize, Default)]
#[serde(transparent)]
pub struct Statement(String);

impl From<Cow<'_, str>> for Statement {
    fn from(value: Cow<'_, str>) -> Self {
        Statement(value.into())
    }
}

impl From<Statement> for Entry {
    fn from(statement: Statement) -> Self {
        Entry::Statement(statement)
    }
}
impl From<&'_ str> for Statement {
    fn from(value: &str) -> Statement {
        Statement(value.into())
    }
}

impl From<String> for Statement {
    fn from(value: String) -> Statement {
        Statement(value)
    }
}

impl From<Statement> for String {
    fn from(value: Statement) -> Self {
        value.0
    }
}

impl Deref for Statement {
    type Target = str;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

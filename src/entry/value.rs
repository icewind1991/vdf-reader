use super::Entry;
use serde::de::{Error, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use std::borrow::Cow;
use std::fmt::Formatter;
use std::ops::Deref;

#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
#[serde(transparent)]
pub struct Value(String);

impl From<Cow<'_, str>> for Value {
    fn from(value: Cow<'_, str>) -> Value {
        Value(value.into())
    }
}
impl From<&'_ str> for Value {
    fn from(value: &str) -> Value {
        Value(value.into())
    }
}

impl From<String> for Value {
    fn from(value: String) -> Value {
        Value(value)
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

impl<'de> Deserialize<'de> for Value {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct ValueVisitor;

        impl Visitor<'_> for ValueVisitor {
            type Value = String;

            fn expecting(&self, formatter: &mut Formatter) -> std::fmt::Result {
                write!(formatter, "any string like value")
            }

            fn visit_i64<E>(self, v: i64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(v.to_string())
            }

            fn visit_u64<E>(self, v: u64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(v.to_string())
            }

            fn visit_f64<E>(self, v: f64) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(v.to_string())
            }

            fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(v.into())
            }

            fn visit_string<E>(self, v: String) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(v)
            }

            fn visit_bool<E>(self, v: bool) -> Result<Self::Value, E>
            where
                E: Error,
            {
                Ok(if v { "1".into() } else { "0".into() })
            }
        }

        deserializer.deserialize_str(ValueVisitor).map(Value)
    }
}

#[cfg(test)]
#[track_caller]
fn unwrap_err<T>(r: Result<T, crate::VdfError>) -> T {
    r.map_err(miette::Error::from).unwrap()
}

#[test]
fn test_serde_value() {
    let j = r#"1"#;
    assert_eq!(Value("1".into()), unwrap_err(crate::from_str(j)));

    let j = r#""foo bar""#;
    assert_eq!(Value("foo bar".into()), unwrap_err(crate::from_str(j)));
}

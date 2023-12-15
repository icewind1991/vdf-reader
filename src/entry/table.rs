use super::{Array, Entry, Statement, Value};
use crate::error::StatementInTableError;
use crate::{Event, Item, Reader, Result};
use serde::{Serialize, Serializer};
use std::collections::HashMap;
use std::ops::Deref;

/// A table of entries.
#[derive(Clone, PartialEq, Eq, Debug, Serialize)]
pub struct Table(#[serde(serialize_with = "ordered_map")] HashMap<String, Entry>);

fn ordered_map<S, K: Ord + Serialize, V: Serialize>(
    value: &HashMap<K, V>,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    use std::collections::BTreeMap;
    let ordered: BTreeMap<_, _> = value.iter().collect();
    ordered.serialize(serializer)
}

fn insert(map: &mut HashMap<String, Entry>, key: String, value: Entry) {
    if !map.contains_key(&key) {
        map.insert(key, value);
        return;
    }

    if let Some(&mut Entry::Array(ref mut array)) = map.get_mut(&key) {
        array.push(value);
        return;
    }

    let mut array = Array::from(map.remove(&key).unwrap());
    array.push(value);

    map.insert(key, array.into());
}

impl Table {
    /// Load a table from the given `Reader`.
    pub fn load(reader: &mut Reader) -> Result<Table> {
        let mut map = HashMap::new();

        while let Some(event) = reader.event() {
            match event? {
                Event::Entry {
                    key: Item::Statement { .. },
                    span,
                    ..
                } => {
                    return Err(
                        StatementInTableError::new(span.into(), reader.content.into()).into(),
                    )
                }
                Event::Entry {
                    key: Item::Value { content: key, .. },
                    value: Item::Statement { content: value, .. },
                    ..
                } => insert(&mut map, key.into(), Statement::from(value).into()),

                Event::Entry {
                    key: Item::Value { content: key, .. },
                    value: Item::Value { content: value, .. },
                    ..
                } => insert(&mut map, key.into(), Value::from(value).into()),

                Event::GroupStart { name, .. } => {
                    insert(&mut map, name.into(), Table::load(reader)?.into())
                }

                Event::GroupEnd { .. } => break,
            }
        }

        return Ok(Table(map));
    }
}

impl Into<Entry> for Table {
    fn into(self) -> Entry {
        Entry::Table(self)
    }
}

impl Deref for Table {
    type Target = HashMap<String, Entry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

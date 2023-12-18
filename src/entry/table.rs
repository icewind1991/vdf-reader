use super::{Array, Entry};
use crate::entry::{Statement, Value};
use crate::event::{EntryEvent, GroupStartEvent};
use crate::{Event, Item, Reader, Result};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::hash_map;
use std::collections::HashMap;
use std::ops::Deref;

/// A table of entries.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(transparent)]
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

fn insert<K: Into<String>, V: Into<Entry>>(map: &mut HashMap<String, Entry>, key: K, value: V) {
    let key = key.into();
    let value = value.into();
    let entry = map.entry(key);
    match entry {
        hash_map::Entry::Vacant(entry) => {
            entry.insert(value);
        }
        hash_map::Entry::Occupied(mut entry) => match entry.get_mut() {
            Entry::Array(ref mut array) => {
                array.push(value);
            }
            _ => {
                let (key, old_value) = entry.remove_entry();
                let mut array = Array::from(old_value);
                array.push(value);
                map.insert(key, Entry::Array(array));
            }
        },
    }
}

impl Table {
    /// Load a table from the given `Reader`.
    pub fn load(reader: &mut Reader) -> Result<Table> {
        let mut map = HashMap::new();

        while let Some(event) = reader.event() {
            match event? {
                Event::Entry(EntryEvent {
                    key: Item::Item { content: key, .. },
                    value,
                    ..
                }) => insert(&mut map, key, Value::from(value.into_content())),

                Event::Entry(EntryEvent {
                    key: Item::Statement { content: key, .. },
                    value,
                    ..
                }) => insert(&mut map, key, Statement::from(value.into_content())),

                Event::GroupStart(GroupStartEvent { name, .. }) => {
                    insert(&mut map, name, Table::load(reader)?)
                }

                Event::GroupEnd(_) => break,
            }
        }

        Ok(Table(map))
    }
}

impl From<Table> for Entry {
    fn from(table: Table) -> Self {
        Entry::Table(table)
    }
}

impl Deref for Table {
    type Target = HashMap<String, Entry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

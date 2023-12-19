use super::{Array, Entry};
use crate::entry::{string_is_array, Statement, Value};
use crate::error::UnknownError;
use crate::event::{EntryEvent, GroupStartEvent};
use crate::{Event, Item, Reader, Result, VdfError};
use serde::de::{DeserializeSeed, MapAccess};
use serde::{Deserialize, Serialize, Serializer};
use std::collections::hash_map;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

/// A table of entries.
#[derive(Clone, PartialEq, Eq, Debug, Deserialize, Serialize)]
#[serde(transparent)]
pub struct Table(#[serde(serialize_with = "ordered_map")] HashMap<String, Entry>);

impl From<HashMap<String, Entry>> for Table {
    fn from(value: HashMap<String, Entry>) -> Self {
        Table(value)
    }
}

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
    pub fn load_from_str(input: &str) -> Result<Table> {
        let mut reader = Reader::from(input);
        Self::load(&mut reader)
    }

    /// Load a table from the given `Reader`.
    pub fn load(reader: &mut Reader) -> Result<Table> {
        let mut map = HashMap::new();

        while let Some(event) = reader.event() {
            match event? {
                Event::Entry(EntryEvent {
                    key: Item::Item { content: key, .. },
                    value,
                    ..
                }) => {
                    if string_is_array(value.as_str()) {
                        let str = value.as_str();
                        insert(
                            &mut map,
                            key,
                            Array::from_space_separated(&str[1..str.len() - 1]),
                        )
                    } else {
                        insert(&mut map, key, Value::from(value.into_content()))
                    }
                }

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

impl From<Table> for HashMap<String, Entry> {
    fn from(table: Table) -> Self {
        table.0
    }
}

impl Deref for Table {
    type Target = HashMap<String, Entry>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for Table {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

pub(crate) struct TableSeq {
    iter: hash_map::IntoIter<String, Entry>,
    next_item: Option<Entry>,
}

impl TableSeq {
    pub(crate) fn new(table: Table) -> Self {
        TableSeq {
            iter: table.0.into_iter(),
            next_item: None,
        }
    }
}

impl<'de> MapAccess<'de> for TableSeq {
    type Error = VdfError;

    fn next_key_seed<K>(&mut self, seed: K) -> std::result::Result<Option<K::Value>, Self::Error>
    where
        K: DeserializeSeed<'de>,
    {
        let (key, value) = match self.iter.next() {
            Some(pair) => pair,
            None => {
                return Ok(None);
            }
        };
        self.next_item = Some(value);
        seed.deserialize(Value::from(key)).map(Some)
    }

    fn next_value_seed<V>(&mut self, seed: V) -> std::result::Result<V::Value, Self::Error>
    where
        V: DeserializeSeed<'de>,
    {
        let item = match self.next_item.take() {
            Some(item) => item,
            None => return Err(UnknownError::from("double take value").into()),
        };

        seed.deserialize(item)
    }
}

#[cfg(test)]
#[track_caller]
fn unwrap_err<T>(r: Result<T, crate::VdfError>) -> T {
    r.map_err(miette::Error::from).unwrap()
}

#[test]
fn test_serde_table() {
    use maplit::hashmap;

    let j = r#"{foo bar}"#;

    assert_eq!(
        Table(hashmap! {"foo".into() => Entry::Value("bar".into())}),
        unwrap_err(crate::from_str(j))
    );
}

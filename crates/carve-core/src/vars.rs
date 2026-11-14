//! Global variables, kept in memory and saved as JSON.

use std::collections::BTreeMap;

use serde_json::{Map, Value as Json, json};

use crate::value::{PlayerRef, Value};

#[derive(Debug, Default, Clone)]
pub struct Variables {
    map: BTreeMap<String, Value>,
}

impl Variables {
    pub fn get(&self, key: &str) -> Value {
        self.map.get(key).cloned().unwrap_or(Value::None)
    }

    /// The stored value, without cloning. `None` when unset.
    pub fn get_ref(&self, key: &str) -> Option<&Value> {
        self.map.get(key)
    }

    pub fn set(&mut self, key: String, v: Value) {
        if v == Value::None {
            self.map.remove(&key);
        } else {
            self.map.insert(key, v);
        }
    }

    /// Like [`Variables::set`], but only allocates the key when it is new.
    pub fn set_str(&mut self, key: &str, v: Value) {
        if v == Value::None {
            self.map.remove(key);
        } else if let Some(slot) = self.map.get_mut(key) {
            *slot = v;
        } else {
            self.map.insert(key.to_string(), v);
        }
    }

    /// Delete `key`, or every key under `prefix::` when `key` ends with `::*`.
    pub fn delete(&mut self, key: &str) {
        delete_in(&mut self.map, key);
    }

    pub fn len(&self) -> usize {
        self.map.len()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn to_json(&self) -> String {
        let mut obj = Map::new();
        for (k, v) in &self.map {
            let j = match v {
                Value::None => continue,
                Value::Num(n) => json!(n),
                Value::Text(t) => json!(t),
                Value::Bool(b) => json!(b),
                Value::Player(p) => json!({"player": {"uuid": p.uuid, "name": p.name}}),
            };
            obj.insert(k.clone(), j);
        }
        serde_json::to_string_pretty(&Json::Object(obj)).unwrap_or_else(|_| "{}".into())
    }

    pub fn from_json(s: &str) -> Result<Self, String> {
        if s.trim().is_empty() {
            return Ok(Self::default());
        }
        let j: Json =
            serde_json::from_str(s).map_err(|e| format!("variables.json is not valid JSON: {e}"))?;
        let Json::Object(obj) = j else {
            return Err("variables.json must hold a JSON object".into());
        };
        let mut map = BTreeMap::new();
        for (k, v) in obj {
            let val = match v {
                Json::Number(n) => Value::Num(n.as_f64().unwrap_or(0.0)),
                Json::String(s) => Value::Text(s),
                Json::Bool(b) => Value::Bool(b),
                Json::Object(o) => {
                    let p = o.get("player").and_then(Json::as_object);
                    let uuid = p.and_then(|p| p.get("uuid")).and_then(Json::as_str);
                    let name = p.and_then(|p| p.get("name")).and_then(Json::as_str);
                    match (uuid, name) {
                        (Some(u), Some(n)) => Value::Player(PlayerRef {
                            uuid: u.into(),
                            name: n.into(),
                        }),
                        _ => continue,
                    }
                }
                _ => continue,
            };
            map.insert(k, val);
        }
        Ok(Variables { map })
    }
}

/// For a list key like `list::*`, the prefix `list::` that every item starts with.
pub(crate) fn list_prefix(key: &str) -> Option<&str> {
    key.strip_suffix('*').filter(|p| p.ends_with("::"))
}

fn delete_in(map: &mut BTreeMap<String, Value>, key: &str) {
    if let Some(prefix) = list_prefix(key) {
        map.retain(|k, _| !k.starts_with(prefix));
    } else {
        map.remove(key);
    }
}

/// Local variables of one run. Runs hold few locals, so a flat list is
/// faster than a map here.
#[derive(Debug, Default, Clone)]
pub(crate) struct Locals(Vec<(String, Value)>);

impl Locals {
    pub(crate) fn get(&self, key: &str) -> Option<&Value> {
        self.0.iter().find(|(k, _)| k == key).map(|(_, v)| v)
    }

    pub(crate) fn set(&mut self, key: &str, v: Value) {
        let pos = self.0.iter().position(|(k, _)| k == key);
        match (pos, v) {
            (Some(i), Value::None) => {
                self.0.swap_remove(i);
            }
            (None, Value::None) => {}
            (Some(i), v) => {
                if let Some(slot) = self.0.get_mut(i) {
                    slot.1 = v;
                }
            }
            (None, v) => self.0.push((key.to_string(), v)),
        }
    }

    pub(crate) fn delete(&mut self, key: &str) {
        if let Some(prefix) = list_prefix(key) {
            self.0.retain(|(k, _)| !k.starts_with(prefix));
        } else {
            self.set(key, Value::None);
        }
    }
}

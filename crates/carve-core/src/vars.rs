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

    /// Every variable name starting with `prefix`, sorted.
    pub fn keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.map
            .range(prefix.to_string()..)
            .take_while(|(k, _)| k.starts_with(prefix))
            .map(|(k, _)| k.clone())
            .collect()
    }

    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }

    pub fn to_json(&self) -> String {
        let mut obj = Map::new();
        for (k, v) in &self.map {
            let j = match v {
                Value::None | Value::List(_) => continue,
                Value::Num(n) => json!(n),
                Value::Text(t) => json!(t),
                Value::Bool(b) => json!(b),
                Value::Player(p) => json!({"player": {"uuid": p.uuid, "name": p.name}}),
                Value::Item { id, count, meta } => match meta {
                    Some(m) => json!({"item": {"id": id, "count": count, "meta": m.to_json()}}),
                    None => json!({"item": {"id": id, "count": count}}),
                },
                Value::Location { x, y, z, world } => {
                    json!({"location": {"x": x, "y": y, "z": z, "world": world}})
                }
                Value::World(w) => json!({"world": w}),
                Value::Date(d) => json!({"date": d}),
                Value::Console => json!({"console": true}),
                Value::Timespan { millis } => json!({"timespan": millis}),
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
                Json::Object(o) => match object_value(&o) {
                    Some(v) => v,
                    None => continue,
                },
                _ => continue,
            };
            map.insert(k, val);
        }
        Ok(Variables { map })
    }
}

/// A saved player, item, location or world (`{"player": {...}}` and so on).
fn object_value(o: &Map<String, Json>) -> Option<Value> {
    if let Some(p) = o.get("player").and_then(Json::as_object) {
        return Some(Value::Player(PlayerRef {
            uuid: p.get("uuid")?.as_str()?.into(),
            name: p.get("name")?.as_str()?.into(),
        }));
    }
    if let Some(i) = o.get("item").and_then(Json::as_object) {
        return Some(Value::Item {
            id: i.get("id")?.as_str()?.into(),
            count: i
                .get("count")
                .and_then(Json::as_u64)
                .and_then(|c| u32::try_from(c).ok())
                .unwrap_or(1),
            meta: i.get("meta").and_then(crate::item_meta::ItemMeta::from_json),
        });
    }
    if let Some(l) = o.get("location").and_then(Json::as_object) {
        return Some(Value::Location {
            x: l.get("x")?.as_f64()?,
            y: l.get("y")?.as_f64()?,
            z: l.get("z")?.as_f64()?,
            world: l.get("world").and_then(Json::as_str).map(str::to_string),
        });
    }
    if let Some(d) = o.get("date").and_then(Json::as_i64) {
        return Some(Value::Date(d));
    }
    if o.contains_key("console") {
        return Some(Value::Console);
    }
    if let Some(ms) = o.get("timespan").and_then(Json::as_i64) {
        return Some(Value::Timespan { millis: ms });
    }
    o.get("world")
        .and_then(Json::as_str)
        .map(|w| Value::World(w.into()))
}

/// Compare list indexes the way people expect: numbers by value, then text.
pub(crate) fn index_order(a: &str, b: &str) -> std::cmp::Ordering {
    match (a.parse::<f64>(), b.parse::<f64>()) {
        (Ok(x), Ok(y)) => x.partial_cmp(&y).unwrap_or(std::cmp::Ordering::Equal),
        (Ok(_), Err(_)) => std::cmp::Ordering::Less,
        (Err(_), Ok(_)) => std::cmp::Ordering::Greater,
        _ => a.cmp(b),
    }
}

/// The direct items of a list (`prefix` is `name::`), sorted by index.
fn direct<'a>(it: impl Iterator<Item = (&'a String, &'a Value)>, prefix: &str) -> Vec<(String, Value)> {
    let mut out: Vec<(String, Value)> = it
        .filter_map(|(k, v)| {
            let rest = k.strip_prefix(prefix)?;
            (!rest.contains("::")).then(|| (rest.to_string(), v.clone()))
        })
        .collect();
    out.sort_by(|a, b| index_order(&a.0, &b.0));
    out
}

/// Next free number index of a list, counting from size + 1 like Skript.
fn next_index(items: &[(String, Value)]) -> String {
    let mut i = items.len() + 1;
    while items.iter().any(|(k, _)| k.parse::<usize>() == Ok(i)) {
        i += 1;
    }
    i.to_string()
}

impl Variables {
    /// Direct items of the list `prefix` (`name::`), sorted by index.
    pub fn list(&self, prefix: &str) -> Vec<(String, Value)> {
        direct(
            self.map
                .range(prefix.to_string()..)
                .take_while(|(k, _)| k.starts_with(prefix)),
            prefix,
        )
    }

    pub fn list_len(&self, prefix: &str) -> usize {
        self.map
            .range(prefix.to_string()..)
            .take_while(|(k, _)| k.starts_with(prefix))
            .filter(|(k, _)| !k.get(prefix.len()..).unwrap_or("").contains("::"))
            .count()
    }

    pub fn list_push(&mut self, prefix: &str, v: Value) {
        let idx = next_index(&self.list(prefix));
        self.set(format!("{prefix}{idx}"), v);
    }

    pub fn list_remove(&mut self, prefix: &str, v: &Value) {
        for (k, item) in self.list(prefix) {
            if item.loose_eq(v) {
                self.map.remove(&format!("{prefix}{k}"));
            }
        }
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

    pub(crate) fn list(&self, prefix: &str) -> Vec<(String, Value)> {
        direct(self.0.iter().map(|(k, v)| (k, v)), prefix)
    }

    pub(crate) fn list_len(&self, prefix: &str) -> usize {
        self.0
            .iter()
            .filter(|(k, _)| k.strip_prefix(prefix).is_some_and(|r| !r.contains("::")))
            .count()
    }

    pub(crate) fn list_push(&mut self, prefix: &str, v: Value) {
        let idx = next_index(&self.list(prefix));
        self.set(&format!("{prefix}{idx}"), v);
    }

    pub(crate) fn list_remove(&mut self, prefix: &str, v: &Value) {
        self.0.retain(|(k, item)| {
            !(k.strip_prefix(prefix).is_some_and(|r| !r.contains("::")) && item.loose_eq(v))
        });
    }
}

//! Global variables, kept in memory and saved as JSON.

use std::collections::BTreeMap;
use std::fmt::Write as _;

use serde_json::{Map, Value as Json, json};

use crate::value::{PlayerRef, Value};

#[derive(Debug, Default, Clone)]
pub struct Variables {
    store: Store,
}

impl Variables {
    pub fn get(&self, key: &str) -> Value {
        self.store.map.get(key).cloned().unwrap_or(Value::None)
    }

    /// The stored value, without cloning. `None` when unset.
    pub fn get_ref(&self, key: &str) -> Option<&Value> {
        self.store.map.get(key)
    }

    pub fn set(&mut self, key: String, v: Value) {
        self.store.set(key, v);
    }

    /// Like [`Variables::set`], but only allocates the key when it is new.
    pub fn set_str(&mut self, key: &str, v: Value) {
        self.store.set_str(key, v);
    }

    /// Delete `key`, or every key under `prefix::` when `key` ends with `::*`.
    pub fn delete(&mut self, key: &str) {
        self.store.delete(key);
    }

    pub fn len(&self) -> usize {
        self.store.map.len()
    }

    /// Every variable name starting with `prefix`, sorted.
    pub fn keys_with_prefix(&self, prefix: &str) -> Vec<String> {
        self.store.under(prefix).map(|(k, _)| k.clone()).collect()
    }

    pub fn is_empty(&self) -> bool {
        self.store.map.is_empty()
    }

    pub fn to_json(&self) -> String {
        let mut obj = Map::new();
        for (k, v) in &self.store.map {
            let j = match v {
                Value::None | Value::List(_) | Value::Entity(_) | Value::Queue(_) | Value::Ray(_) => continue,
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
                Value::Nbt(c) => json!({"nbt": c.to_snbt()}),
                Value::Vector { x, y, z } => json!({"vector": {"x": x, "y": y, "z": z}}),
                Value::Inventory(i) if i.viewer.is_none() => i.to_json(),
                Value::Inventory(_) => continue,
                other => match crate::types_1::to_json(other) {
                    Some(j) => j,
                    None => continue,
                },
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
        let mut store = Store::default();
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
            store.set(k, val);
        }
        Ok(Variables { store })
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
    if let Some(v) = o.get("vector").and_then(Json::as_object) {
        return Some(Value::Vector {
            x: v.get("x")?.as_f64()?,
            y: v.get("y")?.as_f64()?,
            z: v.get("z")?.as_f64()?,
        });
    }
    if let Some(d) = o.get("date").and_then(Json::as_i64) {
        return Some(Value::Date(d));
    }
    if o.contains_key("console") {
        return Some(Value::Console);
    }
    if let Some(s) = o.get("nbt").and_then(Json::as_str) {
        return crate::skbee_nbt::Compound::parse(s)
            .ok()
            .map(|c| Value::Nbt(Box::new(c)));
    }
    if let Some(ms) = o.get("timespan").and_then(Json::as_i64) {
        return Some(Value::Timespan { millis: ms });
    }
    if let Some(v) = crate::gaps::InvVal::from_json(o) {
        return Some(v);
    }
    if let Some(v) = crate::types_1::from_json(o) {
        return Some(v);
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

/// Order two `a::b::c` paths part by part, numbers by value.
pub(crate) fn path_order(a: &str, b: &str) -> std::cmp::Ordering {
    let mut x = a.split("::");
    let mut y = b.split("::");
    loop {
        match (x.next(), y.next()) {
            (None, None) => return std::cmp::Ordering::Equal,
            (None, Some(_)) => return std::cmp::Ordering::Less,
            (Some(_), None) => return std::cmp::Ordering::Greater,
            (Some(p), Some(q)) => {
                let o = index_order(p, q);
                if o != std::cmp::Ordering::Equal {
                    return o;
                }
            }
        }
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

/// Per list (`name::`): how many direct items it has, and how many of their
/// indexes parse as a whole number without being written plainly (`01`, `+2`).
#[derive(Debug, Default, Clone, Copy)]
struct Meta {
    count: usize,
    odd: usize,
}

/// The lists a key is a direct item of, with its index in each: `a::b::c` is
/// item `c` of `a::b::`. Three colons in a row (`a:::b`) make it an item of
/// both `a::` and `a:::`, same as a prefix scan finds.
fn parents(k: &str) -> impl Iterator<Item = (&str, &str)> {
    let last = k.rfind("::");
    let a = last.map(|i| (&k[..i + 2], &k[i + 2..]));
    let b = last
        .filter(|&i| i > 0 && k.as_bytes()[i - 1] == b':')
        .map(|i| (&k[..i + 1], &k[i + 1..]));
    a.into_iter().chain(b)
}

/// An index that counts as a whole number for `add` but isn't written in
/// plain decimal form.
fn odd_index(rest: &str) -> bool {
    rest.parse::<usize>().is_ok()
        && !(rest.bytes().all(|b| b.is_ascii_digit()) && (rest.len() == 1 || !rest.starts_with('0')))
}

/// Sorted variables plus a per-list item count, so `add` and `size of` don't
/// walk the whole list.
#[derive(Debug, Default, Clone)]
struct Store {
    map: BTreeMap<String, Value>,
    meta: BTreeMap<String, Meta>,
}

impl Store {
    fn added(&mut self, key: &str) {
        for (p, rest) in parents(key) {
            let odd = usize::from(odd_index(rest));
            if let Some(m) = self.meta.get_mut(p) {
                m.count += 1;
                m.odd += odd;
            } else {
                self.meta.insert(p.to_string(), Meta { count: 1, odd });
            }
        }
    }

    fn removed(&mut self, key: &str) {
        for (p, rest) in parents(key) {
            if let Some(m) = self.meta.get_mut(p) {
                m.count = m.count.saturating_sub(1);
                if odd_index(rest) {
                    m.odd = m.odd.saturating_sub(1);
                }
                if m.count == 0 {
                    self.meta.remove(p);
                }
            }
        }
    }

    fn set(&mut self, key: String, v: Value) {
        if matches!(v, Value::None) {
            self.remove(&key);
        } else if let Some(slot) = self.map.get_mut(&key) {
            *slot = v;
        } else {
            self.added(&key);
            self.map.insert(key, v);
        }
    }

    fn set_str(&mut self, key: &str, v: Value) {
        if matches!(v, Value::None) {
            self.remove(key);
        } else if let Some(slot) = self.map.get_mut(key) {
            *slot = v;
        } else {
            self.added(key);
            self.map.insert(key.to_string(), v);
        }
    }

    fn remove(&mut self, key: &str) {
        if self.map.remove(key).is_some() {
            self.removed(key);
        }
    }

    /// Every entry whose key starts with `prefix`, in key order.
    fn under<'a>(&'a self, prefix: &'a str) -> impl Iterator<Item = (&'a String, &'a Value)> {
        self.map
            .range::<str, _>((std::ops::Bound::Included(prefix), std::ops::Bound::Unbounded))
            .take_while(move |(k, _)| k.starts_with(prefix))
    }

    fn delete(&mut self, key: &str) {
        if let Some(prefix) = list_prefix(key) {
            let keys: Vec<String> = self.under(prefix).map(|(k, _)| k.clone()).collect();
            for k in keys {
                self.remove(&k);
            }
        } else {
            self.remove(key);
        }
    }

    fn list(&self, prefix: &str) -> Vec<(String, Value)> {
        direct(self.under(prefix), prefix)
    }

    fn all_under(&self, prefix: &str) -> Vec<(String, Value)> {
        let mut out: Vec<(String, Value)> = self
            .under(prefix)
            .filter_map(|(k, v)| Some((k.strip_prefix(prefix)?.to_string(), v.clone())))
            .collect();
        out.sort_by(|a, b| path_order(&a.0, &b.0));
        out
    }

    fn list_len(&self, prefix: &str) -> usize {
        self.meta.get(prefix).map_or(0, |m| m.count)
    }

    /// The full key of the list's next free number index, counting from
    /// size + 1 like Skript.
    fn next_key(&self, prefix: &str) -> String {
        let m = self.meta.get(prefix).copied().unwrap_or_default();
        let mut i = m.count + 1;
        if m.odd > 0 {
            // Indexes like `01` count as numbers too: check them the slow way.
            let items = self.list(prefix);
            while items.iter().any(|(k, _)| k.parse::<usize>() == Ok(i)) {
                i += 1;
            }
            return format!("{prefix}{i}");
        }
        let mut key = String::with_capacity(prefix.len() + 8);
        loop {
            key.clear();
            key.push_str(prefix);
            let _ = write!(key, "{i}");
            if !self.map.contains_key(&key) {
                return key;
            }
            i += 1;
        }
    }

    fn list_push(&mut self, prefix: &str, v: Value) {
        if matches!(v, Value::None) {
            return;
        }
        // `next_key` only hands out a key that is not in use.
        let key = self.next_key(prefix);
        self.added(&key);
        self.map.insert(key, v);
    }

    fn list_remove(&mut self, prefix: &str, v: &Value) {
        let gone: Vec<String> = self
            .under(prefix)
            .filter(|(k, item)| k.get(prefix.len()..).is_some_and(|r| !r.contains("::")) && item.loose_eq(v))
            .map(|(k, _)| k.clone())
            .collect();
        for k in gone {
            self.remove(&k);
        }
    }
}

impl Variables {
    /// Direct items of the list `prefix` (`name::`), sorted by index.
    pub fn list(&self, prefix: &str) -> Vec<(String, Value)> {
        self.store.list(prefix)
    }

    /// Every value under the list `prefix` (`name::`), at any depth.
    pub fn all_under(&self, prefix: &str) -> Vec<(String, Value)> {
        self.store.all_under(prefix)
    }

    pub fn list_len(&self, prefix: &str) -> usize {
        self.store.list_len(prefix)
    }

    pub fn list_push(&mut self, prefix: &str, v: Value) {
        self.store.list_push(prefix, v);
    }

    pub fn list_remove(&mut self, prefix: &str, v: &Value) {
        self.store.list_remove(prefix, v);
    }
}

/// For a list key like `list::*`, the prefix `list::` that every item starts with.
pub(crate) fn list_prefix(key: &str) -> Option<&str> {
    key.strip_suffix('*').filter(|p| p.ends_with("::"))
}

/// Local variables of one run. Runs hold few plain locals, so those sit in a
/// flat list (faster than a map for a handful). List items (keys with a `:`)
/// go to a sorted store, so big lists stay fast.
#[derive(Debug, Default, Clone)]
pub(crate) struct Locals {
    flat: Vec<(String, Value)>,
    /// Only allocated once the run uses a list.
    lists: Option<Box<Store>>,
}

fn is_list_key(key: &str) -> bool {
    key.as_bytes().contains(&b':')
}

impl Locals {
    fn lists_mut(&mut self) -> &mut Store {
        self.lists.get_or_insert_with(Box::default)
    }

    // Kept out of line so `get` and `set` stay small enough to inline.
    #[inline(never)]
    fn get_list(&self, key: &str) -> Option<&Value> {
        self.lists.as_ref()?.map.get(key)
    }

    #[inline(never)]
    fn set_list(&mut self, key: &str, v: Value) {
        self.lists_mut().set_str(key, v);
    }

    // `flat` never holds a list key, so a hit there needs no further check.
    #[inline]
    pub(crate) fn get(&self, key: &str) -> Option<&Value> {
        match self.flat.iter().find(|(k, _)| k == key) {
            Some((_, v)) => Some(v),
            None if is_list_key(key) => self.get_list(key),
            None => None,
        }
    }

    #[inline]
    pub(crate) fn set(&mut self, key: &str, v: Value) {
        let pos = self.flat.iter().position(|(k, _)| k == key);
        if pos.is_none() && is_list_key(key) {
            self.set_list(key, v);
            return;
        }
        match (pos, v) {
            (Some(i), Value::None) => {
                self.flat.swap_remove(i);
            }
            (None, Value::None) => {}
            (Some(i), v) => {
                if let Some(slot) = self.flat.get_mut(i) {
                    slot.1 = v;
                }
            }
            (None, v) => self.flat.push((key.to_string(), v)),
        }
    }

    /// Remove everything, keeping the memory for reuse.
    pub(crate) fn clear(&mut self) {
        self.flat.clear();
        if let Some(l) = &mut self.lists {
            l.map.clear();
            l.meta.clear();
        }
    }

    pub(crate) fn delete(&mut self, key: &str) {
        if list_prefix(key).is_some() {
            if let Some(l) = &mut self.lists {
                l.delete(key);
            }
        } else {
            self.set(key, Value::None);
        }
    }

    pub(crate) fn list(&self, prefix: &str) -> Vec<(String, Value)> {
        self.lists.as_ref().map(|l| l.list(prefix)).unwrap_or_default()
    }

    pub(crate) fn all_under(&self, prefix: &str) -> Vec<(String, Value)> {
        self.lists
            .as_ref()
            .map(|l| l.all_under(prefix))
            .unwrap_or_default()
    }

    pub(crate) fn list_len(&self, prefix: &str) -> usize {
        self.lists.as_ref().map_or(0, |l| l.list_len(prefix))
    }

    pub(crate) fn list_push(&mut self, prefix: &str, v: Value) {
        self.lists_mut().list_push(prefix, v);
    }

    pub(crate) fn list_remove(&mut self, prefix: &str, v: &Value) {
        if let Some(l) = &mut self.lists {
            l.list_remove(prefix, v);
        }
    }
}

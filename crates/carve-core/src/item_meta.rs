//! Item meta: names, lore and enchantments on item values.
//!
//! An item value carries an optional [`ItemMeta`]. It is boxed, so plain items stay small and
//! cheap. The meta is saved to `variables.json` as JSON.

use serde_json::{Map, Value as Json, json};

use crate::compile::CondKind;
use crate::lexer::Tok;
use crate::value::Value;

/// Name, lore and enchantments of one item.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemMeta {
    /// Custom name, with `&` colour codes as the script wrote it.
    pub name: Option<String>,
    pub lore: Vec<String>,
    /// Enchantment id (`sharpness`, `sweeping_edge`) and level.
    pub enchants: Vec<(String, u32)>,
    /// Used-up durability (the `damage` component), when the item has any.
    pub damage: Option<u32>,
}

impl ItemMeta {
    pub fn is_empty(&self) -> bool {
        self.name.is_none() && self.lore.is_empty() && self.enchants.is_empty() && self.damage.is_none()
    }

    /// Add an enchantment, replacing the level when it is already there.
    pub fn enchant(&mut self, id: &str, level: u32) {
        match self.enchants.iter_mut().find(|(i, _)| i == id) {
            Some(e) => e.1 = level,
            None => self.enchants.push((id.to_string(), level)),
        }
    }

    /// Level of an enchantment, 0 when absent.
    pub fn level_of(&self, id: &str) -> u32 {
        self.enchants.iter().find(|(i, _)| i == id).map_or(0, |e| e.1)
    }

    pub fn to_json(&self) -> Json {
        let mut o = Map::new();
        if let Some(n) = &self.name {
            o.insert("name".into(), json!(n));
        }
        if !self.lore.is_empty() {
            o.insert("lore".into(), json!(self.lore));
        }
        if !self.enchants.is_empty() {
            let e: Map<String, Json> = self.enchants.iter().map(|(k, v)| (k.clone(), json!(v))).collect();
            o.insert("enchants".into(), Json::Object(e));
        }
        if let Some(d) = self.damage {
            o.insert("damage".into(), json!(d));
        }
        Json::Object(o)
    }

    /// Read meta saved by [`ItemMeta::to_json`]. `None` when it holds nothing.
    pub fn from_json(j: &Json) -> Option<Box<ItemMeta>> {
        let o = j.as_object()?;
        let mut m = ItemMeta {
            name: o.get("name").and_then(Json::as_str).map(str::to_string),
            ..ItemMeta::default()
        };
        if let Some(l) = o.get("lore").and_then(Json::as_array) {
            m.lore = l.iter().filter_map(|x| x.as_str().map(str::to_string)).collect();
        }
        if let Some(e) = o.get("enchants").and_then(Json::as_object) {
            for (k, v) in e {
                if let Some(n) = v.as_u64().and_then(|n| u32::try_from(n).ok()) {
                    m.enchants.push((k.clone(), n));
                }
            }
        }
        m.damage = o
            .get("damage")
            .and_then(Json::as_u64)
            .and_then(|n| u32::try_from(n).ok());
        (!m.is_empty()).then(|| Box::new(m))
    }
}

/// Vanilla enchantment ids, sorted.
pub const ENCHANTS: &[&str] = &[
    "aqua_affinity",
    "bane_of_arthropods",
    "binding_curse",
    "blast_protection",
    "breach",
    "channeling",
    "density",
    "depth_strider",
    "efficiency",
    "feather_falling",
    "fire_aspect",
    "fire_protection",
    "flame",
    "fortune",
    "frost_walker",
    "impaling",
    "infinity",
    "knockback",
    "looting",
    "loyalty",
    "luck_of_the_sea",
    "lunge",
    "lure",
    "mending",
    "multishot",
    "piercing",
    "power",
    "projectile_protection",
    "protection",
    "punch",
    "quick_charge",
    "respiration",
    "riptide",
    "sharpness",
    "silk_touch",
    "smite",
    "soul_speed",
    "sweeping_edge",
    "swift_sneak",
    "thorns",
    "unbreaking",
    "vanishing_curse",
    "wind_burst",
];

/// Other ways to write an enchantment.
const ALIASES: &[(&str, &str)] = &[
    ("curse_of_binding", "binding_curse"),
    ("curse_of_vanishing", "vanishing_curse"),
    ("sweeping", "sweeping_edge"),
];

fn canonical(joined: &str) -> Option<&'static str> {
    if let Ok(i) = ENCHANTS.binary_search(&joined) {
        return ENCHANTS.get(i).copied();
    }
    ALIASES.iter().find(|(a, _)| *a == joined).map(|(_, c)| *c)
}

/// Could this word start an enchantment name? Cheap test before a longer look.
pub fn could_start(word: &str) -> bool {
    let w = word.to_ascii_lowercase();
    let i = ENCHANTS.partition_point(|n| *n < w.as_str());
    ENCHANTS
        .get(i)
        .is_some_and(|n| n.starts_with(&w) && (n.len() == w.len() || n.as_bytes()[w.len()] == b'_'))
        || ALIASES.iter().any(|(a, _)| a.starts_with(&w))
}

fn roman(w: &str) -> Option<u32> {
    Some(match w.to_ascii_lowercase().as_str() {
        "i" => 1,
        "ii" => 2,
        "iii" => 3,
        "iv" => 4,
        "v" => 5,
        "vi" => 6,
        "vii" => 7,
        "viii" => 8,
        "ix" => 9,
        "x" => 10,
        _ => return None,
    })
}

/// Read `sharpness`, `sharpness 5`, `sweeping edge iii` starting at `at`.
/// Returns the id, the level (0 when absent) and the number of tokens used.
pub fn enchant_at(toks: &[Tok], at: usize) -> Option<(&'static str, u32, usize)> {
    let Some(Tok::Word(first)) = toks.get(at) else {
        return None;
    };
    if !could_start(first) {
        return None;
    }
    let mut words: Vec<String> = Vec::new();
    for t in toks.get(at..)?.iter().take(4) {
        match t {
            Tok::Word(w) => words.push(w.to_ascii_lowercase()),
            _ => break,
        }
    }
    for n in (1..=words.len()).rev() {
        let Some(id) = canonical(&words[..n].join("_")) else {
            continue;
        };
        let used = n;
        return Some(match toks.get(at + used) {
            Some(Tok::Num(l)) if l.fract() == 0.0 && (1.0..=255.0).contains(l) => (id, *l as u32, used + 1),
            Some(Tok::Word(w)) if roman(w).is_some() => (id, roman(w).unwrap_or(1), used + 1),
            _ => (id, 0, used),
        });
    }
    None
}

/// Split the text an enchantment expression makes (`sharpness 5`) into id and level.
/// The level is 0 when the text names no level.
pub fn split_spec(s: &str) -> Option<(String, u32)> {
    let s = s.trim().to_ascii_lowercase().replace(' ', "_");
    let (name, level) = match s.rsplit_once('_') {
        Some((n, l)) if l.chars().all(|c| c.is_ascii_digit()) && !l.is_empty() => {
            (n.to_string(), l.parse().ok()?)
        }
        _ => (s, 0),
    };
    let id = canonical(&name)?;
    Some((id.to_string(), level))
}

/// Condition patterns. Tried before the main list.
pub const CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    (
        "%item% is enchanted with %text%",
        CondKind::ItemEnchanted {
            any: false,
            negate: false,
        },
    ),
    (
        "%item% (isn't|is not) enchanted with %text%",
        CondKind::ItemEnchanted {
            any: false,
            negate: true,
        },
    ),
    (
        "%item% is enchanted",
        CondKind::ItemEnchanted {
            any: true,
            negate: false,
        },
    ),
    (
        "%item% (isn't|is not) enchanted",
        CondKind::ItemEnchanted {
            any: true,
            negate: true,
        },
    ),
];

/// Text of a JSON text component, without styling. Used to read names and lore.
pub fn plain_from_json(json: &str) -> String {
    fn walk(j: &Json, out: &mut String) {
        match j {
            Json::String(s) => out.push_str(s),
            Json::Array(a) => a.iter().for_each(|x| walk(x, out)),
            Json::Object(o) => {
                if let Some(t) = o.get("text").and_then(Json::as_str) {
                    out.push_str(t);
                }
                if let Some(e) = o.get("extra") {
                    walk(e, out);
                }
            }
            _ => {}
        }
    }
    let mut out = String::new();
    match serde_json::from_str::<Json>(json) {
        Ok(j) => walk(&j, &mut out),
        Err(_) => out.push_str(json),
    }
    out
}

fn snbt_str(s: &str) -> String {
    let mut o = String::with_capacity(s.len() + 2);
    o.push('"');
    for c in s.chars() {
        if c == '"' || c == '\\' {
            o.push('\\');
        }
        o.push(c);
    }
    o.push('"');
    o
}

/// The vanilla `give` command for an item with meta. Pumpkin reads names and lore as plain
/// strings, so colour codes go in as `§` codes.
pub fn give_command(player: &str, id: &str, count: u32, meta: &ItemMeta) -> String {
    let mut parts: Vec<String> = Vec::new();
    if let Some(n) = &meta.name {
        parts.push(format!("custom_name={}", snbt_str(&crate::colors::colorize(n))));
    }
    if !meta.lore.is_empty() {
        let l: Vec<String> = meta
            .lore
            .iter()
            .map(|l| snbt_str(&crate::colors::colorize(l)))
            .collect();
        parts.push(format!("lore=[{}]", l.join(",")));
    }
    if !meta.enchants.is_empty() {
        let e: Vec<String> = meta
            .enchants
            .iter()
            .map(|(k, v)| format!("\"minecraft:{k}\":{v}"))
            .collect();
        parts.push(format!("enchantments={{{}}}", e.join(",")));
    }
    if let Some(d) = meta.damage {
        parts.push(format!("damage={d}"));
    }
    if parts.is_empty() {
        format!("give {player} minecraft:{id} {count}")
    } else {
        format!("give {player} minecraft:{id}[{}] {count}", parts.join(","))
    }
}

/// What `<thing> of <item>` reads.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ItemInfo {
    Name,
    Lore,
    Enchants,
    /// The item type alone: no amount, no meta.
    Type,
    Amount,
}

impl ItemInfo {
    /// `name`, `lore`, `enchantments` and so on, as one word.
    pub fn from_word(w: &str) -> Option<ItemInfo> {
        Some(match w {
            "name" => ItemInfo::Name,
            "lore" => ItemInfo::Lore,
            "enchantments" | "enchants" => ItemInfo::Enchants,
            "type" => ItemInfo::Type,
            "amount" => ItemInfo::Amount,
            _ => return None,
        })
    }
}

/// `<item> named "x" with lore "a" and "b" enchanted with sharpness 5`.
#[derive(Debug, Clone, PartialEq)]
pub struct MetaExpr {
    pub base: crate::expr::Expr,
    pub name: Option<crate::expr::Expr>,
    pub lore: Vec<crate::expr::Expr>,
    pub enchants: Vec<crate::expr::Expr>,
}

/// Build an item value with meta added to `base`. Non-items pass through unchanged.
/// An enchantment spec without a level gets level 1.
pub fn with_meta(base: Value, name: Option<String>, lore: Vec<String>, specs: &[String]) -> Value {
    let Value::Item { id, count, meta } = base else {
        return base;
    };
    let mut m = meta.map(|b| *b).unwrap_or_default();
    if name.is_some() {
        m.name = name;
    }
    m.lore.extend(lore);
    for s in specs {
        if let Some((e, l)) = split_spec(s) {
            m.enchant(&e, l.max(1));
        }
    }
    Value::Item {
        id,
        count,
        meta: (!m.is_empty()).then(|| Box::new(m)),
    }
}

/// Value of `name of`, `lore of`, ... for an item value.
pub fn info(what: ItemInfo, v: &Value) -> Value {
    match (what, v) {
        (ItemInfo::Amount, Value::Item { count, .. }) => Value::Num(f64::from(*count)),
        (ItemInfo::Amount, Value::List(l)) => Value::Num(l.len() as f64),
        (ItemInfo::Type, Value::Item { id, .. }) => Value::Item {
            id: id.clone(),
            count: 1,
            meta: None,
        },
        (ItemInfo::Name, Value::Item { meta, .. }) => match meta.as_ref().and_then(|m| m.name.clone()) {
            Some(n) => Value::Text(n),
            None => Value::None,
        },
        (ItemInfo::Lore, Value::Item { meta, .. }) => Value::List(
            meta.as_ref()
                .map(|m| m.lore.iter().cloned().map(Value::Text).collect())
                .unwrap_or_default(),
        ),
        (ItemInfo::Enchants, Value::Item { meta, .. }) => Value::List(
            meta.as_ref()
                .map(|m| {
                    m.enchants
                        .iter()
                        .map(|(k, l)| Value::Text(spec_text(k, *l)))
                        .collect()
                })
                .unwrap_or_default(),
        ),
        (ItemInfo::Lore | ItemInfo::Enchants, _) => Value::List(Vec::new()),
        _ => Value::None,
    }
}

/// `item is enchanted [with spec]`. A spec without a level matches any level. A spec with a
/// level matches that level or higher.
pub fn has_enchant(v: &Value, spec: Option<&str>) -> bool {
    let Value::Item { meta, .. } = v else {
        return false;
    };
    let Some(m) = meta else {
        return false;
    };
    let Some(spec) = spec else {
        return !m.enchants.is_empty();
    };
    match split_spec(spec) {
        Some((id, 0)) => m.level_of(&id) > 0,
        Some((id, l)) => m.level_of(&id) >= l,
        None => false,
    }
}

/// `sharpness 5`, as `enchantments of` and enchantment expressions print it.
pub fn spec_text(id: &str, level: u32) -> String {
    if level == 0 {
        id.replace('_', " ")
    } else {
        format!("{} {level}", id.replace('_', " "))
    }
}

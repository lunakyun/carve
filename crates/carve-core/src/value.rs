use std::fmt;

/// A handle to a player. Resolved through the host on every use, so a stored
/// handle for a player who went offline resolves to `<none>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerRef {
    pub uuid: String,
    pub name: String,
}

/// Where a plain `send` goes when no `to` is given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recipient {
    Player(PlayerRef),
    Console,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    None,
    Num(f64),
    Text(String),
    Bool(bool),
    Player(PlayerRef),
    /// An item or block type (`id` without `minecraft:`), with an amount.
    Item {
        id: String,
        count: u32,
        /// Name, lore and enchantments, when the item has any.
        meta: Option<Box<crate::item_meta::ItemMeta>>,
    },
    Location {
        x: f64,
        y: f64,
        z: f64,
        world: Option<String>,
    },
    World(String),
    /// A point in time: milliseconds since 1970-01-01 UTC.
    Date(i64),
    /// The server console (`console`, or `sender` when the console ran the command).
    Console,
    /// A length of time in milliseconds.
    Timespan {
        millis: i64,
    },
    /// A vector (x, y, z).
    Vector {
        x: f64,
        y: f64,
        z: f64,
    },
    /// A list produced by a list operation or a function. Not stored in variables.
    List(Vec<Value>),
    /// An NBT compound (`nbt from "{a:1}"`).
    Nbt(Box<crate::skbee_nbt::Compound>),
    /// A non-player entity (mob, item, ...). Not saved in global variables.
    Entity(EntityRef),
    /// A chunk by chunk coordinates. Not saved in global variables.
    Chunk {
        world: Option<String>,
        x: i32,
        z: i32,
    },
    /// The result of `ray trace ...`. Not saved in global variables.
    Ray(Box<crate::w4_skbee_raytrace_world::RayHit>),
}

/// A handle to an entity. Looked up through the host on every use, so a
/// handle for an entity that is gone does nothing.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EntityRef {
    pub uuid: String,
    /// Entity type id without `minecraft:`, like `zombie`.
    pub kind: String,
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::None => "<none>",
            Value::Num(_) => "number",
            Value::Text(_) => "text",
            Value::Bool(_) => "boolean",
            Value::Player(_) => "player",
            Value::Item { .. } => "item",
            Value::Location { .. } => "location",
            Value::World(_) => "world",
            Value::Date(_) => "date",
            Value::Console => "console",
            Value::Timespan { .. } => "timespan",
            Value::Vector { .. } => "vector",
            Value::List(_) => "list",
            Value::Nbt(_) => "nbt compound",
            Value::Entity(_) => "entity",
            Value::Chunk { .. } => "chunk",
            Value::Ray(_) => "ray trace result",
        }
    }

    pub fn as_num(&self) -> Option<f64> {
        match self {
            Value::Num(n) => Some(*n),
            _ => None,
        }
    }

    /// Skript-like equality: numbers by value, text ignoring case,
    /// players by UUID, a player equals text matching its name.
    pub fn loose_eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::None, Value::None) => true,
            (Value::Num(a), Value::Num(b)) => (a - b).abs() < 1e-9,
            (Value::Text(a), Value::Text(b)) => a.eq_ignore_ascii_case(b),
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Console, Value::Console) => true,
            (Value::Player(a), Value::Player(b)) => a.uuid == b.uuid,
            (Value::Player(p), Value::Text(t)) | (Value::Text(t), Value::Player(p)) => {
                p.name.eq_ignore_ascii_case(t)
            }
            (Value::Num(n), Value::Text(t)) | (Value::Text(t), Value::Num(n)) => t
                .trim()
                .parse::<f64>()
                .map(|v| (v - n).abs() < 1e-9)
                .unwrap_or(false),
            (Value::Item { id: a, .. }, Value::Item { id: b, .. }) => a == b,
            (Value::Item { id, .. }, Value::Text(t)) | (Value::Text(t), Value::Item { id, .. }) => {
                let t = t.trim().to_ascii_lowercase();
                let t = t.strip_prefix("minecraft:").unwrap_or(&t);
                t.replace(' ', "_") == *id
            }
            (
                Value::Location {
                    x: ax,
                    y: ay,
                    z: az,
                    world: aw,
                },
                Value::Location {
                    x: bx,
                    y: by,
                    z: bz,
                    world: bw,
                },
            ) => {
                (ax - bx).abs() < 1e-9
                    && (ay - by).abs() < 1e-9
                    && (az - bz).abs() < 1e-9
                    && (aw.is_none() || bw.is_none() || aw == bw)
            }
            (Value::Entity(a), Value::Entity(b)) => a.uuid == b.uuid,
            (Value::Chunk { world: aw, x: ax, z: az }, Value::Chunk { world: bw, x: bx, z: bz }) => {
                ax == bx && az == bz && (aw.is_none() || bw.is_none() || aw == bw)
            }
            (Value::Vector { x: ax, y: ay, z: az }, Value::Vector { x: bx, y: by, z: bz }) => {
                (ax - bx).abs() < 1e-9 && (ay - by).abs() < 1e-9 && (az - bz).abs() < 1e-9
            }
            (Value::Date(a), Value::Date(b)) => a == b,
            (Value::Nbt(a), Value::Nbt(b)) => a == b,
            (Value::Timespan { millis: a }, Value::Timespan { millis: b })
            => a == b,
            (Value::World(a), Value::World(b)) => a.eq_ignore_ascii_case(b),
            (Value::World(w), Value::Text(t)) | (Value::Text(t), Value::World(w)) => {
                w.eq_ignore_ascii_case(t)
            }
            _ => false,
        }
    }
}

pub fn format_num(n: f64) -> String {
    let mut s = String::new();
    push_num(&mut s, n);
    s
}

/// Append a number the way scripts print it: whole numbers without a
/// fraction, others with at most 4 decimals.
pub fn push_num(out: &mut String, n: f64) {
    use std::fmt::Write;
    if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e15 {
        let _ = write!(out, "{}", n as i64);
    } else {
        let start = out.len();
        let _ = write!(out, "{n:.4}");
        let keep = out
            .get(start..)
            .map_or(0, |s| s.trim_end_matches('0').trim_end_matches('.').len());
        out.truncate(start + keep);
    }
}

impl Value {
    /// Append the printed form of this value to `out`.
    pub fn push_to(&self, out: &mut String) {
        match self {
            Value::None => out.push_str("<none>"),
            Value::Num(n) => push_num(out, *n),
            Value::Text(t) => out.push_str(t),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::Player(p) => out.push_str(&p.name),
            Value::Item { id, count, meta } => {
                if *count != 1 {
                    push_num(out, f64::from(*count));
                    out.push(' ');
                }
                out.push_str(&id.replace('_', " "));
                if let Some(n) = meta.as_ref().and_then(|m| m.name.as_ref()) {
                    out.push_str(" named \"");
                    out.push_str(n);
                    out.push('"');
                }
            }
            Value::Location { x, y, z, world } => {
                out.push_str("x: ");
                push_num(out, *x);
                out.push_str(", y: ");
                push_num(out, *y);
                out.push_str(", z: ");
                push_num(out, *z);
                if let Some(w) = world {
                    out.push_str(" in '");
                    out.push_str(w);
                    out.push('\'');
                }
            }
            Value::World(w) => out.push_str(w),
            Value::Date(d) => out.push_str(&crate::time_dates::format_date(*d, crate::time_dates::DEFAULT_FORMAT)),
            Value::Console => out.push_str("console"),
            Value::Entity(e) => out.push_str(&e.kind.replace('_', " ")),
            Value::Chunk { world, x, z } => {
                out.push_str("chunk ");
                push_num(out, f64::from(*x));
                out.push_str(", ");
                push_num(out, f64::from(*z));
                if let Some(w) = world {
                    out.push_str(" in '");
                    out.push_str(w);
                    out.push('\'');
                }
            }
            Value::Ray(r) => {
                out.push_str("ray trace result");
                if let Some(e) = &r.entity {
                    out.push_str(" (hit ");
                    e.push_to(out);
                    out.push(')');
                } else if let Some((x, y, z)) = r.block {
                    out.push_str(&format!(" (hit block {x}, {y}, {z})"));
                }
            }
            Value::Vector { x, y, z } => {
                out.push_str("x: ");
                push_num(out, *x);
                out.push_str(", y: ");
                push_num(out, *y);
                out.push_str(", z: ");
                push_num(out, *z);
            }
            Value::Timespan { millis } => crate::timespans::push_span(out, *millis),
            Value::Nbt(c) => c.push_snbt(out),
            Value::List(items) => {
                if items.is_empty() {
                    out.push_str("<none>");
                }
                for (i, v) in items.iter().enumerate() {
                    if i > 0 {
                        out.push_str(if i + 1 == items.len() { " and " } else { ", " });
                    }
                    v.push_to(out);
                }
            }
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut s = String::new();
        self.push_to(&mut s);
        f.write_str(&s)
    }
}

//! Wave 5, chunk 6: entity, display, block, border, boss bar and inventory
//! properties, plus a few event value names and the `shoot` section.
//!
//! - `scale of <entity>`, `target of <mob>`, `spectator target of <player>`,
//!   `text alignment` / `line width` / `text opacity of <text display>`.
//! - `sea pickles of <block>`, `block data of <block>`, `temperature of <block>`.
//! - `world border [of <world|player>]`, `world border warning distance|time`.
//! - `title`, `style` and `viewers` of boss bars and inventory values.
//! - `all <type>s`, `index of <slot>`, `spawn egg entity of <item>`.
//! - `source block`, `tamer`, `shooter`, `spawning reason` in their events.
//! - `shoot <projectile> from <entity> [at speed <n>]:`.
//!
//! Every expression is one boxed [`Node`] (`Expr::C6`) and every change is
//! one boxed [`C6Effect`] (`Effect::C6`). Both run out of line, so the hot
//! paths of the engine do not grow.

use crate::block_ops::BlockPos;
use crate::compile::Effect;
use crate::engine::{Engine, Exec, online};
use crate::expr::{Env, Expr, P, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::types_1::{ENUMS, EnumVal};
use crate::value::{EntityRef, PlayerRef, Value};
use crate::w5_chunk6_blockdata::{BLOCKS, PROPS};

/// Fewest and most pickles a sea pickle block holds.
pub const MIN_PICKLES: i64 = 1;
pub const MAX_PICKLES: i64 = 4;
/// Vanilla defaults that `reset` goes back to.
pub const DEFAULT_WARNING_DISTANCE: f64 = 5.0;
pub const DEFAULT_WARNING_SECONDS: f64 = 15.0;
pub const DEFAULT_LINE_WIDTH: f64 = 200.0;
pub const DEFAULT_OPACITY: f64 = 255.0;
/// Speed of `shoot` when the script gives none, in blocks per tick.
pub const DEFAULT_SHOOT_SPEED: f64 = 5.0;
/// Largest scale a script may set (the vanilla attribute limit is 16).
pub const MAX_SCALE: f64 = 16.0;
pub const MIN_SCALE: f64 = 0.0625;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Pick {
    Count,
    Min,
    Max,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextField {
    Alignment,
    LineWidth,
    Opacity,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Warn {
    /// Blocks.
    Distance,
    /// Seconds.
    Time,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    Scale,
    Pickles(Pick),
    /// Every value of a named-value type (index into `ENUMS`).
    AllOf(u8),
    SpawnEgg,
    Spectator,
    Target,
    Temperature,
    Text(TextField),
    /// Title of an inventory value.
    Title,
    BarTitle,
    BarStyle,
    /// Viewers of an inventory value.
    Viewers,
    BarViewers,
    Warning(Warn),
    Border,
    BlockData,
}

impl Op {
    /// Can `set`, `add`, `remove`, `reset` or `delete` change it?
    fn changeable(self) -> bool {
        !matches!(
            self,
            Op::AllOf(_)
                | Op::Temperature
                | Op::Pickles(Pick::Min | Pick::Max)
                | Op::Border
                | Op::BarTitle
                | Op::BarStyle
        )
    }

    fn name(self) -> &'static str {
        match self {
            Op::Scale => "scale",
            Op::Pickles(_) => "sea pickles",
            Op::AllOf(_) => "all values",
            Op::SpawnEgg => "spawn egg entity",
            Op::Spectator => "spectator target",
            Op::Target => "target",
            Op::Temperature => "temperature",
            Op::Text(TextField::Alignment) => "text alignment",
            Op::Text(TextField::LineWidth) => "line width",
            Op::Text(TextField::Opacity) => "text opacity",
            Op::Title | Op::BarTitle => "title",
            Op::BarStyle => "style",
            Op::Viewers | Op::BarViewers => "viewers",
            Op::Warning(Warn::Distance) => "world border warning distance",
            Op::Warning(Warn::Time) => "world border warning time",
            Op::Border => "world border",
            Op::BlockData => "block data",
        }
    }
}

/// One expression of this module: what to read and the argument (at most one).
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub op: Op,
    pub args: Vec<Expr>,
    pub ty: Ty,
}

fn node(op: Op, args: Vec<Expr>, ty: Ty) -> Expr {
    Expr::C6(Box::new(Node { op, args, ty }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Change {
    Set,
    Add,
    Remove,
    Reset,
}

/// `set <node> to <value>`, `add <value> to <node>`, `reset <node>`, ...
#[derive(Debug, Clone, PartialEq)]
pub struct C6Effect {
    pub change: Change,
    pub target: Node,
    pub value: Option<Expr>,
}

// ---- host interface ----------------------------------------------------------

/// A question for the server.
#[derive(Debug, Clone, Copy)]
pub enum Get<'a> {
    /// Display scale (a vector) or the scale attribute of a living entity (a number).
    Scale(&'a EntityRef),
    /// What a mob targets, or the entity a player looks at.
    Target(&'a EntityRef),
    /// The entity a player's camera follows; `None` for their own view.
    Camera(&'a PlayerRef),
    /// A text display field: alignment as text, line width and opacity (0-255) as numbers.
    Text(&'a EntityRef, TextField),
    /// World border warning distance (blocks) or time (seconds).
    Warning(&'a str, Warn),
    BarTitle(&'a str),
    /// Style id like `segmented_6`.
    BarStyle(&'a str),
    BarViewers(&'a str),
    /// Players with a menu open that was built from this inventory id.
    MenuViewers(u64),
}

/// A change on the server. The host returns whether it was done.
#[derive(Debug, Clone, Copy)]
pub enum Set<'a> {
    /// A number (uniform) or a vector (displays only). `None`: back to normal.
    Scale(&'a EntityRef, Option<&'a Value>),
    Target(&'a EntityRef, Option<&'a EntityRef>),
    Camera(&'a PlayerRef, Option<&'a EntityRef>),
    /// Alignment as `left` / `center` / `right`, the others as numbers.
    Text(&'a EntityRef, TextField, &'a Value),
    Warning(&'a str, Warn, f64),
    BlockState(&'a BlockPos, u16),
}

/// A block's state: id without `minecraft:`, its state id and the first
/// state id of that block.
pub type BlockState = (String, u16, u16);

// ---- block data --------------------------------------------------------------

fn block_props(id: &str) -> &'static [u8] {
    BLOCKS
        .binary_search_by(|(n, _)| (*n).cmp(id))
        .map_or(&[], |i| BLOCKS[i].1)
}

/// Property values of a state, in property order.
pub fn decode_state(id: &str, state: u16, first: u16) -> Vec<(&'static str, &'static str)> {
    let props = block_props(id);
    let mut rest = u32::from(state.saturating_sub(first));
    let mut out = Vec::with_capacity(props.len());
    for &p in props.iter().rev() {
        let (name, vals) = PROPS[usize::from(p)];
        let n = vals.len() as u32;
        out.push((name, vals[(rest % n) as usize]));
        rest /= n;
    }
    out.reverse();
    out
}

/// The state id for property values, starting from `current` for the ones not given.
pub fn encode_state(id: &str, first: u16, current: u16, set: &[(String, String)]) -> Result<u16, String> {
    let props = block_props(id);
    let mut vals: Vec<&str> = decode_state(id, current, first)
        .into_iter()
        .map(|(_, v)| v)
        .collect();
    for (k, v) in set {
        let Some(i) = props.iter().position(|&p| PROPS[usize::from(p)].0 == k) else {
            let names: Vec<&str> = props.iter().map(|&p| PROPS[usize::from(p)].0).collect();
            return Err(if names.is_empty() {
                format!("{id} has no block data")
            } else {
                format!("{id} has no property '{k}' (it has: {})", names.join(", "))
            });
        };
        let allowed = PROPS[usize::from(props[i])].1;
        let Some(&found) = allowed.iter().find(|a| **a == v) else {
            return Err(format!(
                "{k} of {id} must be one of {}, not '{v}'",
                allowed.join(", ")
            ));
        };
        vals[i] = found;
    }
    let mut off: u32 = 0;
    for (i, &p) in props.iter().enumerate() {
        let allowed = PROPS[usize::from(p)].1;
        let idx = allowed.iter().position(|a| *a == vals[i]).unwrap_or(0) as u32;
        off = off * allowed.len() as u32 + idx;
    }
    u16::try_from(u32::from(first) + off).map_err(|_| "block state out of range".into())
}

/// `sea_pickle[pickles=2,waterlogged=true]`, or the plain id for blocks with no data.
pub fn format_data(id: &str, state: u16, first: u16) -> String {
    let vals = decode_state(id, state, first);
    if vals.is_empty() {
        return id.to_string();
    }
    let inner: Vec<String> = vals.iter().map(|(k, v)| format!("{k}={v}")).collect();
    format!("{id}[{}]", inner.join(","))
}

/// `minecraft:oak_log[axis=x]` into `oak_log` and its properties.
pub fn parse_data(text: &str) -> Result<(String, Vec<(String, String)>), String> {
    let t = text.trim();
    let (id, rest) = match t.find('[') {
        Some(i) => (&t[..i], Some(&t[i + 1..])),
        None => (t, None),
    };
    let id = id.trim().strip_prefix("minecraft:").unwrap_or(id.trim());
    let id = id.to_ascii_lowercase().replace(' ', "_");
    if id.is_empty() {
        return Err(format!(
            "'{text}' is not block data (expected like oak_log[axis=x])"
        ));
    }
    let mut props = Vec::new();
    if let Some(r) = rest {
        let Some(body) = r.trim_end().strip_suffix(']') else {
            return Err(format!("missing ']' in block data '{text}'"));
        };
        for part in body.split(',').map(str::trim).filter(|p| !p.is_empty()) {
            let Some((k, v)) = part.split_once('=') else {
                return Err(format!("expected key=value in block data, got '{part}'"));
            };
            props.push((k.trim().to_ascii_lowercase(), v.trim().to_ascii_lowercase()));
        }
    }
    Ok((id, props))
}

/// Pickle count of a sea pickle state.
pub fn pickles_of(state: u16, first: u16) -> Option<i64> {
    decode_state("sea_pickle", state, first)
        .first()
        .and_then(|(_, v)| v.parse().ok())
}

// ---- stored enchantments ---------------------------------------------------------

fn varint(b: &[u8], at: &mut usize) -> Option<u32> {
    let mut n = 0u32;
    for shift in 0..5 {
        let byte = *b.get(*at)?;
        *at += 1;
        n |= u32::from(byte & 0x7f) << (7 * shift);
        if byte & 0x80 == 0 {
            return Some(n);
        }
    }
    None
}

fn put_varint(mut n: u32, out: &mut Vec<u8>) {
    loop {
        let b = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

/// The `stored_enchantments` component (a count, then registry id and level
/// pairs; 26.3 enchantment ids are alphabetical) as texts like `sharpness 5`.
pub fn stored_decode(b: &[u8]) -> Value {
    let names = crate::types_1_data::ENCHANTMENTS;
    let mut at = 0;
    let Some(n) = varint(b, &mut at) else {
        return Value::List(Vec::new());
    };
    let mut out = Vec::new();
    for _ in 0..n.min(256) {
        let (Some(id), Some(level)) = (varint(b, &mut at), varint(b, &mut at)) else {
            break;
        };
        if let Some(name) = names.get(id as usize) {
            out.push(Value::Text(format!("{name} {level}")));
        }
    }
    Value::List(out)
}

/// One `sharpness 5` (or `fire aspect`, level 1) into an id and level.
fn enchant_of(text: &str) -> Result<(u32, u32), String> {
    let t = text.trim().to_ascii_lowercase();
    let (name, level) = match t.rsplit_once(' ') {
        Some((n, l)) if l.parse::<u32>().is_ok() => (n.trim(), l.parse::<u32>().unwrap_or(1)),
        _ => (t.as_str(), 1),
    };
    let id = crate::types_1::norm(name);
    let id = id.strip_prefix("minecraft:").unwrap_or(&id);
    let i = crate::types_1_data::ENCHANTMENTS
        .iter()
        .position(|e| *e == id)
        .ok_or_else(|| format!("unknown enchantment '{name}'"))?;
    if !(1..=255).contains(&level) {
        return Err(format!("enchantment level must be 1 to 255, not {level}"));
    }
    Ok((i as u32, level))
}

/// Component bytes for one enchantment text or a list of them.
pub fn stored_encode(v: &Value) -> Result<Vec<u8>, String> {
    let items: Vec<String> = match v {
        Value::List(l) => l.iter().map(ToString::to_string).collect(),
        Value::None => Vec::new(),
        v => vec![v.to_string()],
    };
    let mut pairs: Vec<(u32, u32)> = Vec::new();
    for it in &items {
        let (id, level) = enchant_of(it)?;
        match pairs.iter_mut().find(|p| p.0 == id) {
            Some(p) => p.1 = level,
            None => pairs.push((id, level)),
        }
    }
    let mut out = Vec::new();
    put_varint(pairs.len() as u32, &mut out);
    for (id, level) in pairs {
        put_varint(id, &mut out);
        put_varint(level, &mut out);
    }
    Ok(out)
}

// ---- other pure helpers --------------------------------------------------------

/// Base temperature of each vanilla biome (from the 26.3 data pack).
const BIOME_TEMPS: &[(&str, f64)] = &[
    ("badlands", 2.0),
    ("bamboo_jungle", 0.95),
    ("basalt_deltas", 2.0),
    ("beach", 0.8),
    ("birch_forest", 0.6),
    ("cherry_grove", 0.5),
    ("cold_ocean", 0.5),
    ("crimson_forest", 2.0),
    ("dappled_forest", 0.6),
    ("dark_forest", 0.7),
    ("deep_cold_ocean", 0.5),
    ("deep_dark", 0.8),
    ("deep_frozen_ocean", 0.5),
    ("deep_lukewarm_ocean", 0.5),
    ("deep_ocean", 0.5),
    ("desert", 2.0),
    ("dripstone_caves", 0.8),
    ("end_barrens", 0.5),
    ("end_highlands", 0.5),
    ("end_midlands", 0.5),
    ("eroded_badlands", 2.0),
    ("flower_forest", 0.7),
    ("forest", 0.7),
    ("frozen_ocean", 0.0),
    ("frozen_peaks", -0.7),
    ("frozen_river", 0.0),
    ("grove", -0.2),
    ("ice_spikes", 0.0),
    ("jagged_peaks", -0.7),
    ("jungle", 0.95),
    ("lukewarm_ocean", 0.5),
    ("lush_caves", 0.5),
    ("mangrove_swamp", 0.8),
    ("meadow", 0.5),
    ("mushroom_fields", 0.9),
    ("nether_wastes", 2.0),
    ("ocean", 0.5),
    ("old_growth_birch_forest", 0.6),
    ("old_growth_pine_taiga", 0.3),
    ("old_growth_spruce_taiga", 0.25),
    ("pale_garden", 0.7),
    ("plains", 0.8),
    ("river", 0.5),
    ("savanna", 2.0),
    ("savanna_plateau", 2.0),
    ("small_end_islands", 0.5),
    ("snowy_beach", 0.05),
    ("snowy_plains", 0.0),
    ("snowy_slopes", -0.3),
    ("snowy_taiga", -0.5),
    ("soul_sand_valley", 2.0),
    ("sparse_jungle", 0.95),
    ("stony_peaks", 1.0),
    ("stony_shore", 0.2),
    ("sulfur_caves", 0.8),
    ("sunflower_plains", 0.8),
    ("swamp", 0.8),
    ("taiga", 0.25),
    ("the_end", 0.5),
    ("the_void", 0.5),
    ("warm_ocean", 0.5),
    ("warped_forest", 2.0),
    ("windswept_forest", 0.2),
    ("windswept_gravelly_hills", 0.2),
    ("windswept_hills", 0.2),
    ("windswept_savanna", 2.0),
    ("wooded_badlands", 2.0),
];

/// Temperature at a block: the biome's base temperature, lower above y 80 the
/// way vanilla does it (without vanilla's small random noise).
pub fn temperature(biome: &str, y: i32) -> Option<f64> {
    let b = biome.strip_prefix("minecraft:").unwrap_or(biome);
    let base = BIOME_TEMPS.iter().find(|(n, _)| *n == b)?.1;
    let t = if y > 80 {
        base - f64::from(y - 80) * 0.05 / 40.0
    } else {
        base
    };
    Some((t * 1.0e6).round() / 1.0e6)
}

/// Entity a spawn egg item makes (`zombie_spawn_egg` -> `zombie`).
pub fn egg_entity(item: &str) -> Option<&'static str> {
    let kind = item.strip_suffix("_spawn_egg")?;
    crate::entity_names::ENTITY_TYPES
        .iter()
        .copied()
        .find(|k| *k == kind)
}

/// The spawn egg item for an entity type, if there is one.
pub fn egg_for(kind: &str) -> Option<String> {
    let k = crate::types_1::norm(kind);
    let k = k.strip_prefix("minecraft:").unwrap_or(&k);
    let egg = format!("{k}_spawn_egg");
    crate::items::is_known(&egg).then_some(egg)
}

/// Index of a named-value type whose plural (or alias plural) is `words`.
fn enum_type_plural(words: &str) -> Option<u8> {
    let mut forms: Vec<String> = Vec::with_capacity(3);
    if let Some(s) = words.strip_suffix("ies") {
        forms.push(format!("{s}y"));
    }
    if let Some(s) = words.strip_suffix("es") {
        forms.push(s.to_string());
    }
    forms.push(words.strip_suffix('s')?.to_string());
    forms.iter().find_map(|single| {
        ENUMS
            .iter()
            .position(|d| d.name == single || d.aliases.contains(&single.as_str()))
            .and_then(|i| u8::try_from(i).ok())
    })
}

/// Every value of a named-value type, as a list.
pub fn all_of(ty: u8) -> Value {
    let n = ENUMS.get(usize::from(ty)).map_or(0, |d| d.values.len());
    Value::List(
        (0..n)
            .map(|i| Value::Enum(EnumVal { ty, idx: i as u16 }))
            .collect(),
    )
}

fn enum_named(type_name: &str, id: &str) -> Value {
    match ENUMS.iter().position(|d| d.name == type_name) {
        Some(i) => match crate::types_1::parse_text(i as u8, id) {
            Value::None => Value::Text(id.to_string()),
            v => v,
        },
        None => Value::Text(id.to_string()),
    }
}

// ---- parsing ---------------------------------------------------------------------

/// Head words that can start one of our expressions (`the` is skipped first).
const HEADS: &[&str] = &[
    "scale",
    "scales",
    "min",
    "max",
    "minimum",
    "maximum",
    "sea",
    "pickle",
    "pickles",
    "all",
    "every",
    "raw",
    "unique",
    "index",
    "spawn",
    "spectator",
    "target",
    "targeted",
    "temperature",
    "temperatures",
    "text",
    "line",
    "display",
    "opacity",
    "title",
    "titles",
    "style",
    "styles",
    "viewer",
    "viewers",
    "world",
    "block",
    "source",
    "tamer",
    "shooter",
    "spawning",
];

fn lit(toks: &[Tok], i: usize, w: &str) -> bool {
    toks.get(i).is_some_and(|t| t.is_lit(w))
}

impl P<'_> {
    fn c6_lit(&self, off: usize, w: &str) -> bool {
        if self.pos + off >= self.toks.len() {
            self.hit_end.set(true);
        }
        lit(self.toks, self.pos + off, w)
    }

    /// `of <x>` after the head words: the operand, or `None` when no `of` follows.
    fn c6_of(&mut self) -> Result<Option<Expr>, String> {
        if !self.c6_lit(0, "of") {
            return Ok(None);
        }
        self.pos += 1;
        if self.pos >= self.toks.len() {
            self.hit_end.set(true);
            return Err("missing what to read after 'of'".into());
        }
        self.primary().map(Some)
    }

    /// `bossbar "id"` / `boss bar "id"` at the cursor: the id expression.
    fn c6_bar(&mut self) -> Result<Option<Expr>, String> {
        let n = if self.c6_lit(0, "bossbar") {
            1
        } else if self.c6_lit(0, "boss") && self.c6_lit(1, "bar") {
            2
        } else {
            return Ok(None);
        };
        self.pos += n;
        if self.pos >= self.toks.len() {
            self.hit_end.set(true);
            return Err("missing the boss bar id".into());
        }
        let id = self.primary()?;
        if !Ty::Text.accepts(id.ty(self.env)) {
            return Err("a boss bar id is text, like \"raid\"".into());
        }
        Ok(Some(id))
    }

    /// `<op> of <x>`; with no `of`, `implicit` is the operand (or the head is not ours).
    fn c_unary(&mut self, op: Op, ty: Ty, implicit: Option<Expr>) -> Result<Option<Expr>, String> {
        match self.c6_of()? {
            Some(x) => Ok(Some(node(op, vec![x], ty))),
            None => Ok(implicit.map(|x| node(op, vec![x], ty))),
        }
    }

    /// Called with the head word read. `Ok(None)`: not one of ours, position kept.
    pub(crate) fn c6_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        if !HEADS.contains(&lw) {
            return Ok(None);
        }
        let start = self.pos;
        let r = self.c6_inner(lw);
        if matches!(r, Ok(None)) {
            self.pos = start;
        }
        r
    }

    fn c6_inner(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let env = self.env;
        let in_event = !env.in_function;
        // Event values under the names Skript uses.
        if in_event && let Some(ev) = env.ext4 {
            use crate::w2_events_4::ExtEvent;
            let slot = match (lw, ev) {
                ("source", ExtEvent::Spread) if self.c6_lit(0, "block") => {
                    self.pos += 1;
                    ev.slot_index("source-block")
                }
                ("tamer", ExtEvent::Tame) => ev.slot_index("owner"),
                ("shooter", ExtEvent::Shoot) => ev.slot_index("entity"),
                ("target", ExtEvent::Target) if !self.c6_lit(0, "of") => ev.slot_index("target"),
                _ => None,
            };
            if let Some(i) = slot {
                return Ok(Some(Expr::Ev4Val(i as u8)));
            }
        }
        if lw == "spawning" && self.c6_lit(0, "reason") && in_event && env.event_vals != 0 {
            let reason = [Tok::Word("reason".into())];
            if let Some((v, _)) = crate::events_items::parse_val(env.event_vals, "spawn", &reason) {
                self.pos += 1;
                return Ok(Some(Expr::EventVal(v)));
            }
        }
        match lw {
            "scale" | "scales" => self.c_unary(Op::Scale, Ty::Object, None),
            "min" | "max" | "minimum" | "maximum" | "sea" | "pickle" | "pickles" => self.c_pickles(lw),
            "all" | "every" => Ok(self.c_all()),
            "raw" | "unique" | "index" => self.c_index(lw),
            "spawn" if self.c6_lit(0, "egg") && self.c6_lit(1, "entity") => {
                self.pos += 2;
                self.c_unary(Op::SpawnEgg, Ty::Text, None)
            }
            "spectator" if self.c6_lit(0, "target") => {
                self.pos += 1;
                self.c_unary(Op::Spectator, Ty::Object, Some(Expr::Player))
            }
            "target" | "targeted" => {
                if self.c6_lit(0, "block") {
                    return Ok(None);
                }
                if self.c6_lit(0, "entity") {
                    self.pos += 1;
                }
                let implicit = (!env.no_player).then_some(Expr::Player);
                self.c_unary(Op::Target, Ty::Object, implicit)
            }
            "temperature" | "temperatures" => self.c_unary(Op::Temperature, Ty::Number, None),
            "text" if self.c6_lit(0, "alignment") || self.c6_lit(0, "alignments") => {
                self.pos += 1;
                self.c_unary(Op::Text(TextField::Alignment), Ty::Object, None)
            }
            "text" if self.c6_lit(0, "opacity") => {
                self.pos += 1;
                self.c_unary(Op::Text(TextField::Opacity), Ty::Number, None)
            }
            "line" if self.c6_lit(0, "width") => {
                self.pos += 1;
                self.c_unary(Op::Text(TextField::LineWidth), Ty::Number, None)
            }
            "display"
                if self.c6_lit(0, "opacity") || (self.c6_lit(0, "text") && self.c6_lit(1, "opacity")) =>
            {
                self.pos += if self.c6_lit(0, "text") { 2 } else { 1 };
                self.c_unary(Op::Text(TextField::Opacity), Ty::Number, None)
            }
            "opacity" => self.c_unary(Op::Text(TextField::Opacity), Ty::Number, None),
            "title" | "titles" | "style" | "styles" | "viewer" | "viewers" => {
                if !self.c6_lit(0, "of") {
                    return Ok(None);
                }
                self.pos += 1;
                let (bar_op, inv_op, ty) = match lw {
                    "title" | "titles" => (Op::BarTitle, Some(Op::Title), Ty::Text),
                    "style" | "styles" => (Op::BarStyle, None, Ty::Object),
                    _ => (Op::Viewers, Some(Op::Viewers), Ty::List),
                };
                if let Some(id) = self.c6_bar()? {
                    let op = if bar_op == Op::Viewers {
                        Op::BarViewers
                    } else {
                        bar_op
                    };
                    return Ok(Some(node(op, vec![id], ty)));
                }
                let Some(op) = inv_op else {
                    return Ok(None);
                };
                // Only values that can hold an inventory; the rest is someone else's `title of`.
                let at = self.pos;
                let Ok(x) = self.primary() else {
                    self.pos = at;
                    return Ok(None);
                };
                let fits = matches!(x, Expr::Var(_) | Expr::Gap(_))
                    || matches!(x.ty(env), Ty::T1(crate::types_1::T_INVENTORY));
                if !fits {
                    self.pos = at;
                    return Ok(None);
                }
                Ok(Some(node(op, vec![x], ty)))
            }
            "world" if self.c6_lit(0, "border") => self.c_border(),
            "block" if self.c6_lit(0, "data") => {
                self.pos += 1;
                self.c_unary(Op::BlockData, Ty::Text, None)
            }
            _ => Ok(None),
        }
    }

    fn c_pickles(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let pick = match lw {
            "min" | "minimum" => Pick::Min,
            "max" | "maximum" => Pick::Max,
            _ => Pick::Count,
        };
        // The head word is read; walk the rest of `[sea] pickle(s| count| amount)`.
        let mut i = 0;
        let mut saw_pickle = matches!(lw, "pickle" | "pickles");
        if pick != Pick::Count || lw == "sea" {
            if pick != Pick::Count && self.c6_lit(i, "sea") {
                i += 1;
            }
            if self.c6_lit(i, "pickle") || self.c6_lit(i, "pickles") {
                i += 1;
                saw_pickle = true;
            }
        }
        if !saw_pickle {
            return Ok(None);
        }
        if self.c6_lit(i, "count") || self.c6_lit(i, "amount") {
            i += 1;
        }
        if !self.c6_lit(i, "of") {
            return Ok(None);
        }
        self.pos += i;
        self.c_unary(Op::Pickles(pick), Ty::Number, None)
    }

    fn c_all(&mut self) -> Option<Expr> {
        let mut words: Vec<String> = Vec::new();
        let mut skip = 0;
        if self.c6_lit(0, "of") && self.c6_lit(1, "the") {
            skip = 2;
        } else if self.c6_lit(0, "the") {
            skip = 1;
        }
        for t in self.toks.get(self.pos + skip..).unwrap_or(&[]).iter().take(5) {
            match t {
                Tok::Word(w) => words.push(w.to_ascii_lowercase()),
                _ => break,
            }
        }
        for n in (1..=words.len()).rev() {
            if let Some(ty) = enum_type_plural(&words[..n].join(" ")) {
                self.pos += skip + n;
                return Some(node(Op::AllOf(ty), Vec::new(), Ty::List));
            }
        }
        None
    }

    /// `[raw|unique] index of <slot>`: a slot is its number already.
    fn c_index(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        if lw != "index" {
            if !self.c6_lit(0, "index") {
                return Ok(None);
            }
            self.pos += 1;
        }
        if !self.c6_lit(0, "of") {
            return Ok(None);
        }
        self.pos += 1;
        let at = self.pos;
        let Ok(x) = self.primary() else {
            self.pos = at;
            return Ok(None);
        };
        // `index of "a" in "abc"` is the text function.
        if self.c6_lit(0, "in") {
            return Ok(None);
        }
        use crate::inventories::InvExpr;
        match x {
            Expr::Inv(b) => match *b {
                InvExpr::ClickedSlot => Ok(Some(Expr::Inv(Box::new(InvExpr::ClickedSlot)))),
                InvExpr::Slot { slot, .. } => Ok(Some(slot)),
                _ => Ok(None),
            },
            _ => Ok(None),
        }
    }

    /// After `world`, at `border`.
    fn c_border(&mut self) -> Result<Option<Expr>, String> {
        if self.c6_lit(1, "warning") && (self.c6_lit(2, "distance") || self.c6_lit(2, "time")) {
            let w = if self.c6_lit(2, "distance") {
                Warn::Distance
            } else {
                Warn::Time
            };
            self.pos += 3;
            let ty = if w == Warn::Time { Ty::Timespan } else { Ty::Number };
            let of = self.c6_of()?.unwrap_or(Expr::Text(Vec::new()));
            return Ok(Some(node(Op::Warning(w), vec![of], ty)));
        }
        // `world border size` and friends belong to other modules.
        let next = self.toks.get(self.pos + 1);
        if next.is_some_and(|t| matches!(t, Tok::Word(_)) && !t.is_lit("of")) {
            return Ok(None);
        }
        self.pos += 1;
        let of = self.c6_of()?.unwrap_or(Expr::Text(Vec::new()));
        Ok(Some(node(
            Op::Border,
            vec![of],
            crate::expr::Ty::T1(crate::types_1::T_BORDER),
        )))
    }
}

/// `<x>'s scale`, `<x>'s world border`, ... with `at` on the word after `'s`.
/// Gives the expression and the position after it.
pub(crate) fn possessive(toks: &[Tok], at: usize, base: &Expr) -> Option<(Expr, usize)> {
    let w = |i: usize, s: &str| lit(toks, at + i, s);
    let one = |op: Op, ty: Ty, n: usize| Some((node(op, vec![base.clone()], ty), at + n));
    if w(0, "scale") || w(0, "scales") {
        return one(Op::Scale, Ty::Object, 1);
    }
    if w(0, "spectator") && w(1, "target") {
        return one(Op::Spectator, Ty::Object, 2);
    }
    if (w(0, "target") || w(0, "targeted")) && !w(1, "block") {
        let n = if w(1, "entity") { 2 } else { 1 };
        return one(Op::Target, Ty::Object, n);
    }
    if w(0, "temperature") || w(0, "temperatures") {
        return one(Op::Temperature, Ty::Number, 1);
    }
    if w(0, "text") && (w(1, "alignment") || w(1, "alignments")) {
        return one(Op::Text(TextField::Alignment), Ty::Object, 2);
    }
    if w(0, "line") && w(1, "width") {
        return one(Op::Text(TextField::LineWidth), Ty::Number, 2);
    }
    if w(0, "text") && w(1, "opacity") {
        return one(Op::Text(TextField::Opacity), Ty::Number, 2);
    }
    if w(0, "spawn") && w(1, "egg") && w(2, "entity") {
        return one(Op::SpawnEgg, Ty::Text, 3);
    }
    if w(0, "block") && w(1, "data") {
        return one(Op::BlockData, Ty::Text, 2);
    }
    if w(0, "title") || w(0, "titles") {
        return one(Op::Title, Ty::Text, 1);
    }
    if w(0, "viewer") || w(0, "viewers") {
        return one(Op::Viewers, Ty::List, 1);
    }
    if w(0, "world") && w(1, "border") {
        if w(2, "warning") && (w(3, "distance") || w(3, "time")) {
            let k = if w(3, "time") { Warn::Time } else { Warn::Distance };
            let ty = if k == Warn::Time { Ty::Timespan } else { Ty::Number };
            return one(Op::Warning(k), ty, 4);
        }
        return one(Op::Border, Ty::T1(crate::types_1::T_BORDER), 2);
    }
    let (pick, n) = if w(0, "sea") && (w(1, "pickles") || w(1, "pickle")) {
        (Pick::Count, 2)
    } else if w(0, "pickles") || w(0, "pickle") {
        (Pick::Count, 1)
    } else {
        return None;
    };
    let n = if w(n, "count") || w(n, "amount") { n + 1 } else { n };
    one(Op::Pickles(pick), Ty::Number, n)
}

/// Does this token run start one of our list expressions (for `loop`)?
pub fn is_list_expr(toks: &[Tok]) -> bool {
    let start = usize::from(lit(toks, 0, "the"));
    if lit(toks, start, "viewers") || lit(toks, start, "viewer") {
        return lit(toks, start + 1, "of");
    }
    if lit(toks, start, "all") || lit(toks, start, "every") {
        let words: Vec<String> = toks
            .iter()
            .skip(start + 1)
            .take(5)
            .map_while(|t| match t {
                Tok::Word(w) => Some(w.to_ascii_lowercase()),
                _ => None,
            })
            .collect();
        let from = usize::from(words.first().is_some_and(|w| w == "the"));
        return (from + 1..=words.len()).any(|n| enum_type_plural(&words[from..n].join(" ")).is_some());
    }
    false
}

/// Can a change line target one of ours? Cheap test on the target's first word.
fn may_target(toks: &[Tok]) -> bool {
    let i = usize::from(lit(toks, 0, "the"));
    matches!(toks.get(i), Some(Tok::Word(w)) if HEADS.iter().any(|h| w.eq_ignore_ascii_case(h)))
        || toks.iter().any(|t| t.is_lit("'s"))
}

/// `set/add/remove/reset/delete/clear` of one of our expressions. `None`: not ours.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Option<Result<Effect, String>> {
    let verb = match toks.first() {
        Some(Tok::Word(w)) => w.to_ascii_lowercase(),
        _ => return None,
    };
    let (change, target, value) = match verb.as_str() {
        "set" => {
            let to = toks.iter().position(|t| t.is_lit("to"))?;
            (Change::Set, toks.get(1..to)?, toks.get(to + 1..))
        }
        "add" | "remove" => {
            let kw = if verb == "add" { "to" } else { "from" };
            let at = toks.iter().rposition(|t| t.is_lit(kw))?;
            let ch = if verb == "add" {
                Change::Add
            } else {
                Change::Remove
            };
            (ch, toks.get(at + 1..)?, toks.get(1..at))
        }
        "reset" | "delete" | "clear" => (Change::Reset, toks.get(1..)?, None),
        _ => return None,
    };
    if target.is_empty() || !may_target(target) {
        return None;
    }
    let Ok(Expr::C6(n)) = parse_expr(target, env) else {
        return None;
    };
    // Boss bar title and style have their own effects (`scoreboards_bossbars`).
    if !n.op.changeable() {
        return None;
    }
    let value = match value {
        Some(v) if !v.is_empty() => match (parse_expr(v, env), v) {
            (Ok(e), _) => Some(e),
            // A bare word like `cow` or `left` is its own text.
            (Err(_), [Tok::Word(w)]) => {
                Some(Expr::Text(vec![crate::expr::Part::Lit(w.to_ascii_lowercase())]))
            }
            (Err(m), _) => return Some(Err(m)),
        },
        Some(_) => return Some(Err("missing the value".into())),
        None => None,
    };
    let ok = match (n.op, change) {
        (_, Change::Reset) => !matches!(n.op, Op::Title | Op::SpawnEgg | Op::BlockData),
        (Op::Viewers | Op::BarViewers, Change::Add | Change::Remove) => true,
        (Op::Viewers | Op::BarViewers, _) => false,
        (Op::Pickles(_) | Op::Scale | Op::Warning(_), _) => true,
        (Op::Text(TextField::Alignment), Change::Set) => true,
        (Op::Text(TextField::Alignment), _) => false,
        (Op::Text(_), _) => true,
        (_, Change::Set) => true,
        _ => false,
    };
    // `reset player's title` clears the on-screen title (`w2_effects_3`).
    if !ok && n.op == Op::Title {
        return None;
    }
    if !ok {
        let verb = match change {
            Change::Set => "set",
            Change::Add => "add to",
            Change::Remove => "remove from",
            Change::Reset => "reset",
        };
        return Some(Err(format!("can't {verb} the {}", n.op.name())));
    }
    Some(Ok(Effect::C6(Box::new(C6Effect {
        change,
        target: *n,
        value,
    }))))
}

// ---- the shoot section -----------------------------------------------------------

/// `shoot <projectile> [from <shooter>] [(at|with) (speed|velocity) <n>]:`.
/// Turns into a spawn, a velocity and the body, with `event-entity`,
/// `event-projectile` and `shot entity` meaning the projectile.
pub(crate) fn shoot_section(cx: &mut crate::compile::Cx, l: &crate::blocks::Line) {
    let head = l.head();
    let lower = head.to_ascii_lowercase();
    let body = head.get("shoot ".len()..).unwrap_or("").trim();
    let lbody = lower.get("shoot ".len()..).unwrap_or("").trim().to_string();
    let find = |w: &str| lbody.find(w);
    let speed_at = ["at speed", "with speed", "at velocity", "with velocity"]
        .iter()
        .filter_map(|w| find(&format!(" {w} ")).map(|i| (i, w.len() + 2)))
        .min();
    let from_at = find(" from ");
    let proj_end = [from_at, speed_at.map(|s| s.0)]
        .into_iter()
        .flatten()
        .min()
        .unwrap_or(body.len());
    let mut proj = body.get(..proj_end).unwrap_or("").trim();
    for a in ["a ", "an ", "the ", "1 "] {
        if let Some(p) = proj.strip_prefix(a) {
            proj = p.trim();
        }
    }
    let shooter = match from_at {
        Some(i) => {
            let end = speed_at.filter(|s| s.0 > i).map_or(body.len(), |s| s.0);
            body.get(i + " from ".len()..end).unwrap_or("").trim()
        }
        None => "player",
    };
    let speed = match speed_at {
        Some((i, n)) => {
            let end = from_at.filter(|f| *f > i).unwrap_or(body.len());
            body.get(i + n..end).unwrap_or("").trim()
        }
        None => "",
    };
    if proj.is_empty() || shooter.is_empty() || (speed_at.is_some() && speed.is_empty()) {
        cx.err(
            l.num,
            "expected 'shoot <projectile> [from <entity>] [at speed <number>]:'",
        );
        return;
    }
    let speed = if speed.is_empty() {
        format!("{DEFAULT_SHOOT_SPEED}")
    } else {
        speed.to_string()
    };
    // Brackets only around phrases: `(player)` reads as something else in a velocity line.
    let wrap = |s: &str| {
        if s.contains(' ') {
            format!("({s})")
        } else {
            s.to_string()
        }
    };
    let (shooter, speed) = (wrap(shooter), wrap(&speed));
    let dir = format!("(vector from direction of {shooter})");
    cx.effect(
        l.num,
        &format!("spawn {proj} at (eye location of {shooter}) offset by {dir}"),
    );
    cx.effect(
        l.num,
        &format!("set velocity of last spawned entity to {dir} * {speed}"),
    );
    let body = crate::w4_sections::rewrite_lines(
        &l.children,
        &[
            ("event-entity", "last spawned entity"),
            ("event-projectile", "last spawned entity"),
            ("shot entity", "last spawned entity"),
            ("shot projectile", "last spawned entity"),
        ],
    );
    cx.block(&body);
}

// ---- running -------------------------------------------------------------------------

/// The entity handle of a player or entity value.
fn handle(v: &Value, ex: &mut Exec, host: &mut dyn Host) -> Option<EntityRef> {
    match v {
        Value::Entity(r) => Some(r.clone()),
        Value::Player(_) | Value::Text(_) => online(v, &mut ex.seen, host).map(|p| EntityRef {
            uuid: p.uuid,
            kind: "player".into(),
        }),
        _ => None,
    }
}

fn world_of(v: &Value, ex: &mut Exec, host: &mut dyn Host) -> Option<String> {
    match v {
        Value::World(w) | Value::Border(w) => Some(w.clone()),
        Value::Location { world: Some(w), .. } => Some(w.clone()),
        Value::Player(_) => online(v, &mut ex.seen, host).and_then(|p| host.player_location(&p).map(|l| l.3)),
        Value::Entity(e) => host.entity_location(e).map(|l| l.3),
        Value::Text(t) if !t.is_empty() => {
            if host.world_names().iter().any(|n| n == t) {
                Some(t.clone())
            } else {
                online(v, &mut ex.seen, host).and_then(|p| host.player_location(&p).map(|l| l.3))
            }
        }
        _ => host.default_world(),
    }
}

fn block_of(v: &Value, host: &mut dyn Host) -> Option<BlockPos> {
    match v {
        Value::Location { x, y, z, world } => {
            let world = world.clone().or_else(|| host.default_world());
            crate::block_ops::pos_of(*x, *y, *z, world)
        }
        _ => None,
    }
}

fn num_of(v: &Value, what: &str) -> Result<f64, String> {
    match v {
        Value::Num(n) if n.is_finite() => Ok(*n),
        Value::Timespan { millis } => Ok(*millis as f64 / 1000.0),
        _ => Err(format!(
            "expected a number for the {what}, got {} ({v})",
            v.type_name()
        )),
    }
}

fn players_of(v: Value) -> Vec<Value> {
    match v {
        Value::List(l) => l,
        Value::None => Vec::new(),
        v => vec![v],
    }
}

impl Engine {
    #[cold]
    #[inline(never)]
    pub(crate) fn eval_c6(&mut self, n: &Node, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        let a = match n.args.first() {
            Some(e) => self.eval(e, ex, host)?,
            None => Value::None,
        };
        Ok(read(n.op, &a, ex, host).unwrap_or(Value::None))
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn exec_c6(&mut self, e: &C6Effect, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        let value = match &e.value {
            Some(v) => self.eval(v, ex, host)?,
            None => Value::None,
        };
        let arg = e.target.args.first();
        let a = match arg {
            Some(x) => self.eval(x, ex, host)?,
            None => Value::None,
        };
        let op = e.target.op;
        match op {
            Op::Title | Op::SpawnEgg => {
                let Some(Expr::Var(var)) = arg else {
                    return Err(format!(
                        "the {} can only be changed in a variable, like {{_x}}",
                        op.name()
                    ));
                };
                let new = match (op, a) {
                    (Op::Title, Value::Inventory(mut inv)) => {
                        inv.title = value.to_string();
                        Value::Inventory(inv)
                    }
                    (Op::SpawnEgg, Value::Item { count, meta, .. }) => {
                        let kind = match &value {
                            Value::Entity(r) => r.kind.clone(),
                            v => v.to_string(),
                        };
                        let id =
                            egg_for(&kind).ok_or_else(|| format!("there is no spawn egg for '{kind}'"))?;
                        Value::Item { id, count, meta }
                    }
                    (_, v) => {
                        return Err(format!(
                            "can't change the {} of {} ({v})",
                            op.name(),
                            v.type_name()
                        ));
                    }
                };
                let k = self.key(var, ex, host)?.into_owned();
                self.store(var.local, &k, new, ex, host);
            }
            Op::Viewers | Op::BarViewers => {
                let add = e.change == Change::Add;
                for v in players_of(value) {
                    let Some(p) = online(&v, &mut ex.seen, host) else {
                        continue;
                    };
                    match (&a, op) {
                        (Value::Text(id), Op::BarViewers) => {
                            if add {
                                host.bossbar_show(id, &p);
                            } else {
                                host.bossbar_hide(id, &p);
                            }
                        }
                        (Value::Inventory(inv), _) if add => host.open_inventory(&p, inv),
                        (Value::Inventory(_), _) => host.close_inventory(&p),
                        (x, _) => return Err(format!("{} has no viewers", x.type_name())),
                    }
                }
            }
            _ => change(op, e.change, &a, &value, ex, host)?,
        }
        Ok(())
    }
}

/// Read one of our expressions from its evaluated argument.
fn read(op: Op, a: &Value, ex: &mut Exec, host: &mut dyn Host) -> Option<Value> {
    use Get as G;
    match op {
        Op::Scale => {
            let e = handle(a, ex, host)?;
            host.c6_get(G::Scale(&e))
        }
        Op::Pickles(Pick::Min) => Some(Value::Num(MIN_PICKLES as f64)),
        Op::Pickles(Pick::Max) => Some(Value::Num(MAX_PICKLES as f64)),
        Op::Pickles(Pick::Count) => {
            let at = block_of(a, host)?;
            let (id, state, first) = host.c6_block_state(&at)?;
            (id == "sea_pickle").then(|| pickles_of(state, first).map(|n| Value::Num(n as f64)))?
        }
        Op::AllOf(ty) => Some(all_of(ty)),
        Op::SpawnEgg => match a {
            Value::Item { id, .. } => egg_entity(id).map(|k| Value::Text(k.into())),
            _ => None,
        },
        Op::Spectator => {
            let p = online(a, &mut ex.seen, host)?;
            host.c6_get(G::Camera(&p))
        }
        Op::Target => {
            let e = handle(a, ex, host)?;
            host.c6_get(G::Target(&e))
        }
        Op::Temperature => {
            let at = block_of(a, host)?;
            let world = at.world.clone()?;
            let biome = host.biome_at(&world, at.x, at.y, at.z)?;
            temperature(&biome, at.y).map(Value::Num)
        }
        Op::Text(f) => {
            let e = handle(a, ex, host)?;
            let v = host.c6_get(G::Text(&e, f))?;
            Some(match (f, v) {
                (TextField::Alignment, Value::Text(t)) => enum_named("display text alignment", &t),
                (_, v) => v,
            })
        }
        Op::Title => match a {
            Value::Inventory(inv) => Some(Value::Text(inv.title.clone())),
            _ => None,
        },
        Op::BarTitle => host.c6_get(G::BarTitle(&a.to_string())),
        Op::BarStyle => match host.c6_get(G::BarStyle(&a.to_string()))? {
            Value::Text(t) => Some(enum_named("boss bar style", &t)),
            v => Some(v),
        },
        Op::Viewers => match a {
            Value::Inventory(inv) => {
                if let Some(u) = &inv.viewer {
                    let p = Value::Player(PlayerRef {
                        uuid: u.clone(),
                        name: String::new(),
                    });
                    return Some(Value::List(
                        online(&p, &mut ex.seen, host)
                            .map(Value::Player)
                            .into_iter()
                            .collect(),
                    ));
                }
                host.c6_get(G::MenuViewers(inv.id))
            }
            _ => None,
        },
        Op::BarViewers => host.c6_get(G::BarViewers(&a.to_string())),
        Op::Warning(w) => {
            let world = world_of(a, ex, host)?;
            let v = host.c6_get(G::Warning(&world, w))?;
            match (w, v) {
                (Warn::Time, Value::Num(s)) => Some(Value::Timespan {
                    millis: (s * 1000.0) as i64,
                }),
                (_, v) => Some(v),
            }
        }
        Op::Border => world_of(a, ex, host).map(Value::Border),
        Op::BlockData => {
            let at = block_of(a, host)?;
            let (id, state, first) = host.c6_block_state(&at)?;
            Some(Value::Text(format_data(&id, state, first)))
        }
    }
}

/// Apply a change to a host-backed expression.
fn change(
    op: Op,
    how: Change,
    a: &Value,
    value: &Value,
    ex: &mut Exec,
    host: &mut dyn Host,
) -> Result<(), String> {
    // Add and remove start from the current number.
    let delta = |cur: Option<Value>, what: &str| -> Result<f64, String> {
        let d = num_of(value, what)?;
        let c = match cur {
            Some(Value::Num(n)) => n,
            Some(Value::Timespan { millis }) => millis as f64 / 1000.0,
            _ => 0.0,
        };
        Ok(match how {
            Change::Add => c + d,
            Change::Remove => c - d,
            _ => d,
        })
    };
    match op {
        Op::Scale => {
            let Some(e) = handle(a, ex, host) else {
                return Ok(());
            };
            let new = match how {
                Change::Reset => None,
                Change::Set if matches!(value, Value::Vector { .. }) => Some(value.clone()),
                _ => {
                    let cur = host.c6_get(Get::Scale(&e));
                    let n = delta(cur, "scale")?;
                    Some(Value::Num(n.clamp(MIN_SCALE, MAX_SCALE)))
                }
            };
            host.c6_set(Set::Scale(&e, new.as_ref()));
        }
        Op::Pickles(_) => {
            let Some(at) = block_of(a, host) else {
                return Ok(());
            };
            let Some((id, state, first)) = host.c6_block_state(&at) else {
                return Ok(());
            };
            if id != "sea_pickle" {
                return Ok(());
            }
            let n = match how {
                Change::Reset => MIN_PICKLES as f64,
                _ => delta(
                    pickles_of(state, first).map(|n| Value::Num(n as f64)),
                    "sea pickles",
                )?,
            };
            let n = (n.round() as i64).clamp(MIN_PICKLES, MAX_PICKLES);
            let s = encode_state("sea_pickle", first, state, &[("pickles".into(), n.to_string())])?;
            host.c6_set(Set::BlockState(&at, s));
        }
        Op::BlockData => {
            let Some(at) = block_of(a, host) else {
                return Ok(());
            };
            let (id, props) = parse_data(&value.to_string())?;
            let mut cur = host.c6_block_state(&at);
            if cur.as_ref().is_none_or(|c| c.0 != id) {
                if !crate::items::is_known(&id) && block_props(&id).is_empty() {
                    return Err(format!("unknown block '{id}'"));
                }
                host.set_block(&at, &id);
                cur = host.c6_block_state(&at);
            }
            let Some((cid, state, first)) = cur else {
                return Ok(());
            };
            if cid != id {
                return Err(format!("'{id}' is not a block"));
            }
            if !props.is_empty() {
                let s = encode_state(&id, first, state, &props)?;
                host.c6_set(Set::BlockState(&at, s));
            }
        }
        Op::Spectator | Op::Target => {
            let to = match how {
                Change::Set => match handle(value, ex, host) {
                    Some(r) => Some(r),
                    None => return Err(format!("expected an entity, got {} ({value})", value.type_name())),
                },
                Change::Reset => None,
                _ => return Err(format!("can't add to or remove from the {}", op.name())),
            };
            if op == Op::Spectator {
                if let Some(p) = online(a, &mut ex.seen, host) {
                    host.c6_set(Set::Camera(&p, to.as_ref()));
                }
            } else if let Some(e) = handle(a, ex, host) {
                host.c6_set(Set::Target(&e, to.as_ref()));
            }
        }
        Op::Text(f) => {
            let Some(e) = handle(a, ex, host) else {
                return Ok(());
            };
            let new = match (f, how) {
                (TextField::Alignment, Change::Reset) => Value::Text("center".into()),
                (TextField::Alignment, _) => {
                    let t = crate::types_1::norm(&value.to_string());
                    if !matches!(t.as_str(), "left" | "center" | "centered" | "right") {
                        return Err(format!(
                            "text alignment must be left, center or right, not '{value}'"
                        ));
                    }
                    Value::Text(if t == "centered" { "center".into() } else { t })
                }
                (TextField::LineWidth, Change::Reset) => Value::Num(DEFAULT_LINE_WIDTH),
                (TextField::Opacity, Change::Reset) => Value::Num(DEFAULT_OPACITY),
                (_, _) => {
                    let cur = host.c6_get(Get::Text(&e, f));
                    let n = delta(cur, op.name())?.round();
                    Value::Num(if f == TextField::Opacity {
                        n.clamp(0.0, 255.0)
                    } else {
                        n.clamp(0.0, f64::from(i32::MAX))
                    })
                }
            };
            host.c6_set(Set::Text(&e, f, &new));
        }
        Op::Warning(w) => {
            let Some(world) = world_of(a, ex, host) else {
                return Ok(());
            };
            let n = match how {
                Change::Reset if w == Warn::Distance => DEFAULT_WARNING_DISTANCE,
                Change::Reset => DEFAULT_WARNING_SECONDS,
                _ => {
                    let cur = host.c6_get(Get::Warning(&world, w));
                    delta(cur, op.name())?
                }
            };
            host.c6_set(Set::Warning(&world, w, n.round().clamp(0.0, f64::from(i32::MAX))));
        }
        _ => return Err(format!("can't change the {}", op.name())),
    }
    Ok(())
}

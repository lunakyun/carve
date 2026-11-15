//! Wave 6, chunk 3: conditions the game can answer after all (`is adult`,
//! `is baby`, `is charged`, `is climbing`, `is conditional`, `is lootable`,
//! `is occluding`, `is pathfinding`, `is persistent`, `is left handed`,
//! `is pressing <key>`, `is holding`, `plugin "x" is enabled`, `running
//! minecraft "x"`, ...). The old notes said the API had nothing for these.
//!
//! Everything compiles to one boxed `Cond::W6c3`, so the hot condition enum
//! does not grow. Lines without a key word never touch the pattern table. The
//! game answers through [`Host::w6c3`](crate::host::Host::w6c3); a question the
//! game cannot answer makes the condition false, negated or not.

use std::sync::OnceLock;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::compile::Cond;
use crate::engine::{Engine, Exec, online};
use crate::expr::{Env, Expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::pattern::{Capture, Memo, Pattern};
use crate::value::{PlayerRef, Value};

/// Which hand `is holding` looks at.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hand {
    Any,
    Main,
    Off,
}

/// How `running minecraft` compares versions.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Cmp {
    Eq,
    Above,
    Below,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Adult,
    Baby,
    Charged,
    Climbing,
    Conditional,
    Spawner,
    Infinite,
    Lootable,
    Normalized,
    Occluding,
    Pathfinding,
    Persistent,
    LeftHanded,
    RightHanded,
    Igniting,
    IndirectPowered,
    Powered,
    Silent,
    Unbreakable,
    Jumping,
    RidingType,
    Riding,
    Holding(Hand),
    Plugin,
    Feature,
    WorldFeature,
    /// Pressing a key; the number is the protocol's input bit.
    Key(u8),
    Version(Cmp),
    /// `the respawn location is a bed spawn` (`false`) or `... a respawn anchor spawn` (`true`).
    SpawnPoint(bool),
    /// `<entity> is of type pig`: the type is a word, not a value.
    OfType,
}

/// A yes/no question about an entity (or player).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ck {
    /// Ageable mob (or baby zombie) is a baby; `adult` is the opposite.
    Baby,
    /// A creeper is charged (struck by lightning), or a crossbow is loaded.
    Charged,
    /// On a climbable block and off the ground.
    Climbing,
    /// Spawned by a spawner block while the plugin was loaded.
    Spawner,
    Pathfinding,
    LeftHanded,
    /// A creeper has lit its fuse.
    Igniting,
    Silent,
    /// Riding any vehicle.
    Riding,
    /// Has a loot table (no entity loot table is readable: always `None`).
    Lootable,
    /// Entity is kept loaded (no persistence flag exists: always `None`).
    Persistent,
    /// Jumped within the last few ticks.
    Jumping,
}

/// A yes/no question about the block at a position.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Bk {
    Conditional,
    Lootable,
    Persistent,
    Occluding,
    Powered,
    Solid,
    Transparent,
    Passable,
}

/// What the core asks the game.
#[derive(Debug, Clone, Copy)]
pub enum Q<'a> {
    /// `Bool`.
    Check(Ck, &'a Value),
    /// `Bool`: block at world, x, y, z.
    Block(Bk, &'a str, i32, i32, i32),
    /// `Bool`: a plugin of that name is loaded.
    Plugin(&'a str),
    /// `Bool`: the server runs with that feature flag (`trade_rebalance`).
    Feature(&'a str),
    /// `Bool`: the player holds the input bit now.
    Key(&'a PlayerRef, u8),
    /// `Text`: the Minecraft version, like `26.3`.
    Version,
    /// List of two `Text` ids (`""` for an empty hand): main hand, off hand.
    Hands(&'a Value),
    /// `Text`: kind of the vehicle the entity rides, or `None` when it rides nothing.
    Vehicle(&'a Value),
    /// `Bool`: an active effect of a player has duration 0 (infinite).
    InfiniteEffect(&'a PlayerRef, &'a str),
    /// `Bool`: the world border of the named world holds this x and z.
    Border(&'a str, f64, f64),
    /// `Text`: id of the block at a position.
    BlockId(&'a str, i32, i32, i32),
    /// `Text`: `bed`, `anchor` or `none`: what the player's respawn point is.
    SpawnKind(&'a PlayerRef),
}

/// Set when a script uses `is pressing <key>`, so the game side listens to
/// player input (a high-volume event it only subscribes to on demand).
static KEYS_USED: AtomicBool = AtomicBool::new(false);

/// Does any compiled script use `is pressing <key>`?
pub fn keys_used() -> bool {
    KEYS_USED.load(Ordering::Relaxed)
}

/// Set when a script uses `running minecraft`: the version is read once at start.
static VERSION_USED: AtomicBool = AtomicBool::new(false);

/// Does any compiled script use `running minecraft`?
pub fn version_used() -> bool {
    VERSION_USED.load(Ordering::Relaxed)
}

/// Set when a script asks whether an entity came from a spawner.
static SPAWNER_USED: AtomicBool = AtomicBool::new(false);

/// Does any compiled script use `is from a spawner`?
pub fn spawner_used() -> bool {
    SPAWNER_USED.load(Ordering::Relaxed)
}

#[derive(Debug, Clone, PartialEq)]
pub struct C3Cond {
    pub kind: Kind,
    pub negate: bool,
    pub a: Option<Expr>,
    pub b: Option<Expr>,
}

/// Condition patterns: pattern, what it asks, negated.
pub const CONDITION_PATTERNS: &[(&str, Kind, bool)] = &[
    ("%object% (is|are) [an] adult", Kind::Adult, false),
    (
        "%object% (isn't|is not|aren't|are not) [an] adult",
        Kind::Adult,
        true,
    ),
    ("%object% (is|are) [a] baby", Kind::Baby, false),
    (
        "%object% (isn't|is not|aren't|are not) [a] baby",
        Kind::Baby,
        true,
    ),
    ("%object% (is|are) charged", Kind::Charged, false),
    (
        "%object% (isn't|is not|aren't|are not) charged",
        Kind::Charged,
        true,
    ),
    ("%object% (is|are) climbing", Kind::Climbing, false),
    (
        "%object% (isn't|is not|aren't|are not) climbing",
        Kind::Climbing,
        true,
    ),
    ("%object% (is|are) conditional", Kind::Conditional, false),
    (
        "%object% (isn't|is not|aren't|are not) conditional",
        Kind::Conditional,
        true,
    ),
    (
        "%object% (is|are) (from|spawned from|spawned by) [(a|the)] [(mob|trial|monster)] spawner",
        Kind::Spawner,
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) (from|spawned from|spawned by) [(a|the)] [(mob|trial|monster)] spawner",
        Kind::Spawner,
        true,
    ),
    ("%object% (is|are) infinite", Kind::Infinite, false),
    (
        "%object% (isn't|is not|aren't|are not) infinite",
        Kind::Infinite,
        true,
    ),
    ("%object% (is|are) lootable", Kind::Lootable, false),
    (
        "%object% (isn't|is not|aren't|are not) lootable",
        Kind::Lootable,
        true,
    ),
    ("%object% (is|are) normalized", Kind::Normalized, false),
    (
        "%object% (isn't|is not|aren't|are not) normalized",
        Kind::Normalized,
        true,
    ),
    ("%object% (is|are) occluding", Kind::Occluding, false),
    (
        "%object% (isn't|is not|aren't|are not) occluding",
        Kind::Occluding,
        true,
    ),
    ("%object% (is|are) pathfinding", Kind::Pathfinding, false),
    (
        "%object% (isn't|is not|aren't|are not) pathfinding",
        Kind::Pathfinding,
        true,
    ),
    ("%object% (is|are) persistent", Kind::Persistent, false),
    (
        "%object% (isn't|is not|aren't|are not) persistent",
        Kind::Persistent,
        true,
    ),
    ("%object% (is|are) left( |-)handed", Kind::LeftHanded, false),
    (
        "%object% (isn't|is not|aren't|are not) left( |-)handed",
        Kind::LeftHanded,
        true,
    ),
    ("%object% (is|are) right( |-)handed", Kind::RightHanded, false),
    (
        "%object% (isn't|is not|aren't|are not) right( |-)handed",
        Kind::RightHanded,
        true,
    ),
    (
        "%object% (is|are) (igniting|ignited|in [the] ignition process)",
        Kind::Igniting,
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) (igniting|ignited|in [the] ignition process)",
        Kind::Igniting,
        true,
    ),
    (
        "%object% (is|are) indirectly [block] [redstone] powered",
        Kind::IndirectPowered,
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) indirectly [block] [redstone] powered",
        Kind::IndirectPowered,
        true,
    ),
    (
        "%object% (is|are) [block] [redstone] powered",
        Kind::Powered,
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) [block] [redstone] powered",
        Kind::Powered,
        true,
    ),
    ("%object% (is|are) silent", Kind::Silent, false),
    (
        "%object% (isn't|is not|aren't|are not) silent",
        Kind::Silent,
        true,
    ),
    ("%object% (is|are) unbreakable", Kind::Unbreakable, false),
    (
        "%object% (isn't|is not|aren't|are not) unbreakable",
        Kind::Unbreakable,
        true,
    ),
    ("%object% (is|are) jumping", Kind::Jumping, false),
    (
        "%object% (isn't|is not|aren't|are not) jumping",
        Kind::Jumping,
        true,
    ),
    ("%object% (is|are) riding %object%", Kind::RidingType, false),
    (
        "%object% (isn't|is not|aren't|are not) riding %object%",
        Kind::RidingType,
        true,
    ),
    ("%object% (is|are) riding", Kind::Riding, false),
    (
        "%object% (isn't|is not|aren't|are not) riding",
        Kind::Riding,
        true,
    ),
    (
        "%object% (is|are) holding %object%",
        Kind::Holding(Hand::Any),
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) holding %object%",
        Kind::Holding(Hand::Any),
        true,
    ),
    (
        "%object% (is|are) holding %object% in [the] main hand",
        Kind::Holding(Hand::Main),
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) holding %object% in [the] main hand",
        Kind::Holding(Hand::Main),
        true,
    ),
    (
        "%object% (is|are) holding %object% in [the] off hand",
        Kind::Holding(Hand::Off),
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) holding %object% in [the] off hand",
        Kind::Holding(Hand::Off),
        true,
    ),
    ("[the] plugin %object% (is|are) enabled", Kind::Plugin, false),
    (
        "[the] plugin %object% (isn't|is not|aren't|are not) enabled",
        Kind::Plugin,
        true,
    ),
    ("plugin %object% (is|are) enabled", Kind::Plugin, false),
    (
        "plugin %object% (isn't|is not|aren't|are not) enabled",
        Kind::Plugin,
        true,
    ),
    (
        "[the] [experimental] feature %object% (is|are) enabled",
        Kind::Feature,
        false,
    ),
    (
        "[the] [experimental] feature %object% (isn't|is not|aren't|are not) enabled",
        Kind::Feature,
        true,
    ),
    (
        "%object% (is|are) using [the] [experimental] feature %object%",
        Kind::WorldFeature,
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) using [the] [experimental] feature %object%",
        Kind::WorldFeature,
        true,
    ),
    (
        "%object% (is|are) pressing [the] forward [key]",
        Kind::Key(1),
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) pressing [the] forward [key]",
        Kind::Key(1),
        true,
    ),
    (
        "%object% (is|are) pressing [the] backward [key]",
        Kind::Key(2),
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) pressing [the] backward [key]",
        Kind::Key(2),
        true,
    ),
    ("%object% (is|are) pressing [the] left [key]", Kind::Key(4), false),
    (
        "%object% (isn't|is not|aren't|are not) pressing [the] left [key]",
        Kind::Key(4),
        true,
    ),
    (
        "%object% (is|are) pressing [the] right [key]",
        Kind::Key(8),
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) pressing [the] right [key]",
        Kind::Key(8),
        true,
    ),
    (
        "%object% (is|are) pressing [the] jump [key]",
        Kind::Key(16),
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) pressing [the] jump [key]",
        Kind::Key(16),
        true,
    ),
    (
        "%object% (is|are) pressing [the] sneak [key]",
        Kind::Key(32),
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) pressing [the] sneak [key]",
        Kind::Key(32),
        true,
    ),
    (
        "%object% (is|are) pressing [the] sprint [key]",
        Kind::Key(64),
        false,
    ),
    (
        "%object% (isn't|is not|aren't|are not) pressing [the] sprint [key]",
        Kind::Key(64),
        true,
    ),
    (
        "running [(minecraft|mc)] [version] %object%",
        Kind::Version(Cmp::Eq),
        false,
    ),
    (
        "running (above|newer than|later than) [(minecraft|mc)] [version] %object%",
        Kind::Version(Cmp::Above),
        false,
    ),
    (
        "running (below|older than|earlier than) [(minecraft|mc)] [version] %object%",
        Kind::Version(Cmp::Below),
        false,
    ),
    (
        "(isn't|is not|not) running [(minecraft|mc)] [version] %object%",
        Kind::Version(Cmp::Eq),
        true,
    ),
    (
        "(isn't|is not|not) running (above|newer than|later than) [(minecraft|mc)] [version] %object%",
        Kind::Version(Cmp::Above),
        true,
    ),
    (
        "(isn't|is not|not) running (below|older than|earlier than) [(minecraft|mc)] [version] %object%",
        Kind::Version(Cmp::Below),
        true,
    ),
];

/// Patterns for effects (`player's speed effect is infinite`) and the held item.
pub const POTION_PATTERNS: &[(&str, Kind, bool)] = &[
    (
        "[the] respawn location (is|was) [a] bed spawn",
        Kind::SpawnPoint(false),
        false,
    ),
    (
        "[the] respawn location (isn't|is not|wasn't|was not) [a] bed spawn",
        Kind::SpawnPoint(false),
        true,
    ),
    (
        "[the] respawn location (is|was) [a] respawn anchor spawn",
        Kind::SpawnPoint(true),
        false,
    ),
    (
        "[the] respawn location (isn't|is not|wasn't|was not) [a] respawn anchor spawn",
        Kind::SpawnPoint(true),
        true,
    ),
    ("%object% (is|are) infinite", Kind::Infinite, false),
    (
        "%object% (isn't|is not|aren't|are not) infinite",
        Kind::Infinite,
        true,
    ),
    (
        "%player%'s %object% [potion] effect (is|are) infinite",
        Kind::Infinite,
        false,
    ),
    (
        "%player%'s %object% [potion] effect (isn't|is not|aren't|are not) infinite",
        Kind::Infinite,
        true,
    ),
    (
        "%player%'s (tool|held item) (is|are) unbreakable",
        Kind::Unbreakable,
        false,
    ),
    (
        "%player%'s (tool|held item) (isn't|is not|aren't|are not) unbreakable",
        Kind::Unbreakable,
        true,
    ),
    (
        "[the] (tool|held item) of %player% (is|are) unbreakable",
        Kind::Unbreakable,
        false,
    ),
    (
        "[the] (tool|held item) of %player% (isn't|is not|aren't|are not) unbreakable",
        Kind::Unbreakable,
        true,
    ),
    ("%object% (is|are) infinite", Kind::Infinite, false),
    (
        "%object% (isn't|is not|aren't|are not) infinite",
        Kind::Infinite,
        true,
    ),
];

/// Words one of the patterns above needs. A line without any of them skips
/// the whole list, which keeps compiling fast.
const KEY_WORDS: &[&str] = &[
    "adult",
    "baby",
    "charged",
    "climbing",
    "conditional",
    "spawner",
    "infinite",
    "lootable",
    "normalized",
    "occluding",
    "pathfinding",
    "persistent",
    "handed",
    "igniting",
    "ignited",
    "ignition",
    "powered",
    "silent",
    "unbreakable",
    "jumping",
    "riding",
    "holding",
    "plugin",
    "feature",
    "pressing",
    "running",
    "respawn",
    "type",
];

fn patterns() -> &'static [Pattern<(Kind, bool)>] {
    static PATS: OnceLock<Vec<Pattern<(Kind, bool)>>> = OnceLock::new();
    PATS.get_or_init(|| {
        CONDITION_PATTERNS
            .iter()
            .chain(POTION_PATTERNS)
            .filter_map(|(s, k, n)| Pattern::new(s, (*k, *n)).ok())
            .collect()
    })
}

/// Checks every pattern compiles. Used by tests.
pub fn check_patterns() -> Result<(), String> {
    for (s, k, n) in CONDITION_PATTERNS.iter().chain(POTION_PATTERNS) {
        Pattern::new(s, (*k, *n))?;
    }
    Ok(())
}

/// All the pattern sources, for the docs.
pub fn pattern_sources() -> Vec<&'static str> {
    CONDITION_PATTERNS
        .iter()
        .chain(POTION_PATTERNS)
        .map(|(s, _, _)| *s)
        .collect()
}

fn cap(c: &mut [Option<Capture>], i: usize) -> Option<Expr> {
    match c.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    }
}

/// Try the patterns on one condition line.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    if !toks.iter().any(|t| KEY_WORDS.iter().any(|w| t.is_lit(w))) {
        return None;
    }
    if let Some(c) = riding_type(toks, env).or_else(|| of_type(toks, env)) {
        return Some(c);
    }
    let mut memo = Memo::default();
    let ((kind, negate), mut caps) = patterns()
        .iter()
        .find_map(|pat| pat.matches_memo(toks, env, &mut memo).ok().map(|c| (pat.kind, c)))?;
    match kind {
        Kind::Key(_) => KEYS_USED.store(true, Ordering::Relaxed),
        Kind::Spawner => SPAWNER_USED.store(true, Ordering::Relaxed),
        Kind::Version(_) => VERSION_USED.store(true, Ordering::Relaxed),
        _ => {}
    }
    let (a, b) = (cap(&mut caps, 0), cap(&mut caps, 1));
    Some(Cond::W6c3(Box::new(C3Cond { kind, negate, a, b })))
}

/// `<entity> is of type pig`, `<entity> isn't of type zombie`: an entity type word.
fn of_type(toks: &[Tok], env: &Env) -> Option<Cond> {
    let o = toks
        .windows(2)
        .position(|w| w[0].is_lit("of") && w[1].is_lit("type"))?;
    let kind = crate::entities::kind_words(toks.get(o + 2..)?)?;
    let is_at = toks
        .iter()
        .take(o)
        .position(|t| t.is_lit("is") || t.is_lit("are") || t.is_lit("isn't") || t.is_lit("aren't"))?;
    let mut negate = toks[is_at].is_lit("isn't") || toks[is_at].is_lit("aren't");
    if toks.get(is_at + 1).is_some_and(|t| t.is_lit("not")) {
        negate = !negate;
    }
    let subject = crate::expr::parse_expr(toks.get(..is_at)?, env).ok()?;
    Some(Cond::W6c3(Box::new(C3Cond {
        kind: Kind::OfType,
        negate,
        a: Some(subject),
        b: Some(Expr::Text(vec![crate::expr::Part::Lit(kind.to_string())])),
    })))
}

/// `<entity> is riding a pig`: the vehicle's type is a word, not a value.
fn riding_type(toks: &[Tok], env: &Env) -> Option<Cond> {
    let r = toks.iter().position(|t| t.is_lit("riding"))?;
    let rest = toks.get(r + 1..)?;
    let rest = match rest.first() {
        Some(t) if t.is_lit("a") || t.is_lit("an") || t.is_lit("the") => &rest[1..],
        _ => rest,
    };
    let kind = crate::entities::kind_words(rest)?;
    let is_at = toks
        .iter()
        .take(r)
        .position(|t| t.is_lit("is") || t.is_lit("are") || t.is_lit("isn't") || t.is_lit("aren't"))?;
    let mut negate = toks[is_at].is_lit("isn't") || toks[is_at].is_lit("aren't");
    if toks.get(is_at + 1).is_some_and(|t| t.is_lit("not")) {
        negate = !negate;
    }
    let subject = crate::expr::parse_expr(toks.get(..is_at)?, env).ok()?;
    Some(Cond::W6c3(Box::new(C3Cond {
        kind: Kind::RidingType,
        negate,
        a: Some(subject),
        b: Some(Expr::Text(vec![crate::expr::Part::Lit(kind.to_string())])),
    })))
}

// ---------------------------------------------------------------- helpers

fn norm(s: &str) -> String {
    crate::conditions_entity_block::norm_id(s)
}

/// Every value of a list, or the value itself.
fn each(v: &Value) -> Vec<&Value> {
    match v {
        Value::List(l) => l.iter().collect(),
        Value::None => Vec::new(),
        v => vec![v],
    }
}

fn block_of(v: &Value, host: &mut dyn Host) -> Option<(String, i32, i32, i32)> {
    let Value::Location { x, y, z, world } = v else {
        return None;
    };
    let w = match world {
        Some(w) => w.clone(),
        None => host.default_world()?,
    };
    Some((w, x.floor() as i32, y.floor() as i32, z.floor() as i32))
}

fn as_bool(v: Option<Value>) -> Option<bool> {
    match v? {
        Value::Bool(b) => Some(b),
        _ => None,
    }
}

fn item_name(v: &Value) -> Option<String> {
    match v {
        Value::Item { id, .. } => Some(norm(id)),
        Value::Text(t) if !t.trim().is_empty() => Some(norm(t)),
        Value::Enum(e) => Some(norm(e.id())),
        _ => None,
    }
}

/// Dotted version numbers as comparable parts (`1.20.4`, `26.3`).
pub fn version_parts(s: &str) -> Vec<u32> {
    s.trim()
        .trim_start_matches(['v', 'V'])
        .split(['.', '-', '+', ' '])
        .map_while(|p| p.parse::<u32>().ok())
        .collect()
}

/// Compare two versions part by part; a missing part counts as 0.
pub fn version_cmp(have: &str, want: &str) -> std::cmp::Ordering {
    let (a, b) = (version_parts(have), version_parts(want));
    for i in 0..a.len().max(b.len()) {
        let o = a.get(i).unwrap_or(&0).cmp(b.get(i).unwrap_or(&0));
        if o.is_ne() {
            return o;
        }
    }
    std::cmp::Ordering::Equal
}

/// Is the position inside chunk (cx, cz)?
pub fn in_chunk(x: f64, z: f64, cx: i32, cz: i32) -> bool {
    (x / 16.0).floor() as i32 == cx && (z / 16.0).floor() as i32 == cz
}

/// Vectors are normalized when their length is 1 (Bukkit's tolerance).
pub fn is_normalized(x: f64, y: f64, z: f64) -> bool {
    (x * x + y * y + z * z - 1.0).abs() < 1e-6
}

/// Every value of a subject must pass; a value the game cannot answer for
/// makes the whole check unanswered.
fn all(a: &Value, mut f: impl FnMut(&Value) -> Option<bool>) -> Option<bool> {
    let mut out = None;
    for v in each(a) {
        match f(v) {
            Some(true) => out = Some(out.unwrap_or(true)),
            Some(false) => return Some(false),
            None => return None,
        }
    }
    out
}

/// Entity type id named by a value (an entity, a player, a type name).
fn entity_kind(v: &Value) -> Option<String> {
    match v {
        Value::Entity(e) => Some(norm(&e.kind)),
        Value::Player(_) => Some("player".into()),
        Value::Text(t) => Some(norm(t)),
        Value::Enum(e) => Some(norm(e.id())),
        _ => None,
    }
}

impl Engine {
    fn c6_eval(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        match e {
            Expr::Var(v) if v.list => Ok(Value::List(
                self.list_items(v, ex, host)?
                    .into_iter()
                    .map(|(_, v)| v)
                    .collect(),
            )),
            e => self.eval(e, ex, host),
        }
    }

    /// Run one of these conditions. `None` from the game counts as false,
    /// negated or not.
    #[cold]
    pub(crate) fn cond_w6c3(
        &mut self,
        c: &C3Cond,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<bool, String> {
        let a = match &c.a {
            Some(e) => self.c6_eval(e, ex, host)?,
            None => Value::None,
        };
        let b = match &c.b {
            Some(e) => Some(self.c6_eval(e, ex, host)?),
            None => None,
        };
        let hit = match c.kind {
            Kind::Plugin => match &a {
                Value::Text(n) => as_bool(host.w6c3(Q::Plugin(n.trim()))),
                _ => None,
            },
            Kind::Feature => match &a {
                Value::Text(n) => as_bool(host.w6c3(Q::Feature(&norm(n)))),
                Value::Enum(e) => as_bool(host.w6c3(Q::Feature(&norm(e.id())))),
                _ => None,
            },
            Kind::WorldFeature => match b.as_ref() {
                Some(Value::Text(n)) => as_bool(host.w6c3(Q::Feature(&norm(n)))),
                Some(Value::Enum(e)) => as_bool(host.w6c3(Q::Feature(&norm(e.id())))),
                _ => None,
            },
            Kind::Version(cmp) => {
                let want = match &a {
                    Value::Text(t) => t.clone(),
                    Value::Num(n) => n.to_string(),
                    _ => String::new(),
                };
                let w = version_parts(&want);
                match host.w6c3(Q::Version) {
                    Some(Value::Text(have)) if !w.is_empty() => Some(match cmp {
                        // `running minecraft "26"` matches every 26.x.
                        Cmp::Eq => {
                            let h = version_parts(&have);
                            h.len() >= w.len() && h[..w.len()] == w[..]
                        }
                        Cmp::Above => version_cmp(&have, &want).is_gt(),
                        Cmp::Below => version_cmp(&have, &want).is_lt(),
                    }),
                    _ => None,
                }
            }
            Kind::SpawnPoint(anchor) => {
                let who = match &c.a {
                    Some(e) => self.eval(e, ex, host)?,
                    None => self.eval(&Expr::Player, ex, host)?,
                };
                match online(&who, &mut ex.seen, host) {
                    Some(p) => match host.w6c3(Q::SpawnKind(&p)) {
                        Some(Value::Text(k)) => Some(k == if anchor { "anchor" } else { "bed" }),
                        _ => None,
                    },
                    None => None,
                }
            }
            Kind::OfType => {
                let want = b.as_ref().and_then(entity_kind);
                all(&a, |v| {
                    let want = want.as_ref()?;
                    match v {
                        Value::Entity(_) | Value::Player(_) => Some(entity_kind(v).as_ref() == Some(want)),
                        Value::Location { .. } => Some(block_id_at(v, host).as_ref() == Some(want)),
                        Value::Item { .. } => Some(item_name(v).as_ref() == Some(want)),
                        _ => None,
                    }
                })
            }
            Kind::Key(bit) => match online(&a, &mut ex.seen, host) {
                Some(p) => as_bool(host.w6c3(Q::Key(&p, bit))),
                None => None,
            },
            Kind::Normalized => all(&a, |v| match v {
                Value::Vector { x, y, z } => Some(is_normalized(*x, *y, *z)),
                _ => None,
            }),
            Kind::Infinite => match (&a, b.as_ref()) {
                (Value::W5(o), _) => match &**o {
                    crate::potions_particles_queues::Obj::Potion(p) => {
                        Some(p.ticks == crate::potions_particles_queues::INFINITE)
                    }
                    _ => None,
                },
                // `player's speed effect is infinite`: the player first, then the effect.
                (who, Some(eff)) => {
                    let p = online(who, &mut ex.seen, host);
                    let name = match eff {
                        Value::Text(t) => Some(norm(t)),
                        Value::Enum(e) => Some(norm(e.id())),
                        _ => None,
                    };
                    match (p, name) {
                        (Some(p), Some(n)) => as_bool(host.w6c3(Q::InfiniteEffect(&p, &n))),
                        _ => None,
                    }
                }
                _ => None,
            },
            Kind::Unbreakable => match &a {
                Value::Item { meta, .. } => Some(
                    meta.as_ref()
                        .is_some_and(|m| m.comps.iter().any(|(k, _)| norm(k) == "unbreakable")),
                ),
                Value::Player(_) | Value::Text(_) => online(&a, &mut ex.seen, host).map(|p| {
                    host.held_component(&p, crate::skbee_nbt::ItemComp::Unbreakable)
                        .is_some()
                }),
                _ => None,
            },
            Kind::Holding(hand) => {
                let want: Vec<String> = b
                    .as_ref()
                    .map(|b| each(b).into_iter().filter_map(item_name).collect())
                    .unwrap_or_default();
                if want.is_empty() {
                    None
                } else {
                    all(&a, |v| {
                        let Value::List(h) = host.w6c3(Q::Hands(v))? else {
                            return None;
                        };
                        let slot = |i: usize| match h.get(i) {
                            Some(Value::Text(t)) if !t.is_empty() => Some(norm(t)),
                            _ => None,
                        };
                        let held: Vec<String> = match hand {
                            Hand::Main => slot(0).into_iter().collect(),
                            Hand::Off => slot(1).into_iter().collect(),
                            Hand::Any => slot(0).into_iter().chain(slot(1)).collect(),
                        };
                        Some(want.iter().any(|w| held.contains(w)))
                    })
                }
            }
            Kind::RidingType => {
                let want = b.as_ref().and_then(entity_kind);
                all(&a, |v| {
                    let want = want.as_ref()?;
                    match host.w6c3(Q::Vehicle(v))? {
                        Value::Text(k) => Some(norm(&k) == *want),
                        Value::None => Some(false),
                        _ => None,
                    }
                })
            }
            kind => all(&a, |v| check_one(kind, v, host)),
        };
        Ok(hit.is_some_and(|h| h != c.negate))
    }
}

fn block_check(k: Bk, v: &Value, host: &mut dyn Host) -> Option<bool> {
    let (w, x, y, z) = block_of(v, host)?;
    as_bool(host.w6c3(Q::Block(k, &w, x, y, z)))
}

/// One subject of a check (before negation).
fn check_one(kind: Kind, v: &Value, host: &mut dyn Host) -> Option<bool> {
    let mut ent = |c: Ck| as_bool(host.w6c3(Q::Check(c, v)));
    match kind {
        Kind::Baby => ent(Ck::Baby),
        Kind::Adult => ent(Ck::Baby).map(|b| !b),
        Kind::Charged => ent(Ck::Charged),
        Kind::Climbing => ent(Ck::Climbing),
        Kind::Spawner => ent(Ck::Spawner),
        Kind::Pathfinding => ent(Ck::Pathfinding),
        Kind::LeftHanded => ent(Ck::LeftHanded),
        Kind::RightHanded => ent(Ck::LeftHanded).map(|b| !b),
        Kind::Igniting => ent(Ck::Igniting),
        // Players keep the older host call; other entities ask the new one.
        Kind::Silent | Kind::Riding if matches!(v, Value::Player(_)) => {
            let Value::Player(p) = v else { return None };
            let st = if kind == Kind::Silent {
                crate::conditions_entity_block::PlayerState::Silent
            } else {
                crate::conditions_entity_block::PlayerState::Riding
            };
            Some(host.player_state(p, st))
        }
        Kind::Silent => ent(Ck::Silent),
        Kind::Riding => ent(Ck::Riding),
        Kind::Jumping => ent(Ck::Jumping),
        Kind::Conditional => block_check(Bk::Conditional, v, host),
        Kind::Occluding => block_check(Bk::Occluding, v, host),
        Kind::Powered | Kind::IndirectPowered => block_check(Bk::Powered, v, host),
        Kind::Lootable => match v {
            Value::Location { .. } => block_check(Bk::Lootable, v, host),
            _ => ent(Ck::Lootable),
        },
        Kind::Persistent => match v {
            Value::Location { .. } => block_check(Bk::Persistent, v, host),
            _ => ent(Ck::Persistent),
        },
        _ => None,
    }
}

/// Id of the block at a location value.
pub fn block_id_at(v: &Value, host: &mut dyn Host) -> Option<String> {
    let (w, x, y, z) = block_of(v, host)?;
    match host.w6c3(Q::BlockId(&w, x, y, z))? {
        Value::Text(t) => Some(norm(&t)),
        _ => None,
    }
}

/// `is solid`, `is transparent` and `is passable` for a placed block, from its own state.
pub fn placed_block_trait(
    at: &Value,
    t: crate::conditions_entity_block::BlockTrait,
    host: &mut dyn Host,
) -> Option<bool> {
    use crate::conditions_entity_block::BlockTrait as T;
    block_check(
        match t {
            T::Solid => Bk::Solid,
            T::Transparent => Bk::Transparent,
            T::Passable => Bk::Passable,
        },
        at,
        host,
    )
}

/// `<location|entity|player> is within <chunk|world border>`.
pub fn within_area(
    who: &Value,
    area: &Value,
    seen: &mut Vec<(String, Option<String>)>,
    host: &mut dyn Host,
) -> bool {
    let pos = match crate::conditions_entity_block::loc_parts(who) {
        Some((x, _, z, w)) => Some((x, z, w.map(str::to_string))),
        None => online(who, seen, host)
            .and_then(|p| host.player_location(&p))
            .map(|(x, _, z, w)| (x, z, Some(w))),
    };
    let Some((x, z, world)) = pos else {
        return false;
    };
    match area {
        Value::Chunk {
            x: cx,
            z: cz,
            world: cw,
        } => {
            let same = match (&world, cw) {
                (Some(a), Some(b)) => a.eq_ignore_ascii_case(b),
                _ => true,
            };
            same && in_chunk(x, z, *cx, *cz)
        }
        Value::Border(name) => {
            let same = world.as_ref().is_none_or(|w| w.eq_ignore_ascii_case(name));
            same && as_bool(host.w6c3(Q::Border(name, x, z))).unwrap_or(false)
        }
        _ => false,
    }
}

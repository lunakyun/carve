//! `damage was caused by`, `egg will hatch`,
//! `entity storage is full`, the equippable checks (`can be dispensed`, ...),
//! `has boss bar flag`, `has tooltip`, `has loot table`, `has metadata`,
//! `has persistent data tag`, `has played before`, `is preferred tool` and
//! `is responsive`.
//!
//! Everything compiles to one boxed `Cond::FlagsMetadata`, so the hot condition enum
//! does not grow. Only lines that hold one of the key words are matched
//! against the patterns. A check the game cannot answer is false, negated or
//! not.

use std::sync::OnceLock;

use crate::compile::Cond;
use crate::expr::{Env, Expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::pattern::{Capture, Memo, Pattern};
use crate::scoreboards_bossbars::BarFlag;
use crate::value::Value;

/// Equippable component flags (see [`EQUIPPABLE`]).
pub const EQ_DISPENSABLE: u8 = 1;
pub const EQ_SWAPPABLE: u8 = 2;
pub const EQ_DAMAGE_ON_HURT: u8 = 4;
pub const EQ_EQUIP_ON_INTERACT: u8 = 8;
pub const EQ_SHEARABLE: u8 = 16;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    /// `damage was caused by fall`: the cause of the running damage event.
    DamageCause,
    /// `egg will hatch` in `on throwing of an egg`.
    EggHatch,
    /// `entity storage of <block> is full` (a beehive or bee nest).
    StorageFull,
    /// A flag of the item's equippable component.
    Equip(u8),
    /// `bossbar "id" has darken sky flag`.
    BarFlag(BarFlag),
    /// `tooltip of <item> is shown`.
    Tooltip,
    /// `<block> has loot table`.
    LootTable,
    /// `<thing> has metadata "key"`.
    Metadata,
    /// `<thing> has persistent data tag "key"`.
    Persistent,
    /// `player has played before`.
    PlayedBefore,
    /// `<item> is preferred tool for <block>`.
    PreferredTool,
    /// `<interaction> is responsive`.
    Responsive,
}

/// One condition: what to check, negation, and up to two values.
#[derive(Debug, Clone, PartialEq)]
pub struct FlagsCond {
    pub kind: Kind,
    pub negate: bool,
    pub a: Option<Expr>,
    pub b: Option<Expr>,
}

/// A question only the game can answer.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Query<'a> {
    /// Has this uuid joined before this session (saved in Carve's `carve.redb`)?
    PlayedBefore(&'a str),
    /// Bees in the hive or nest at a block.
    BeeCount {
        world: Option<&'a str>,
        x: i32,
        y: i32,
        z: i32,
    },
    /// Is this flag on for the boss bar with this id?
    BarFlag(&'a str, BarFlag),
    /// Does this block type (`stone`, no namespace) need the right tool for drops?
    ToolRequired(&'a str),
    /// Is the interaction entity with this uuid responsive?
    Responsive(&'a str),
}

type P = (Kind, bool);
use Kind::*;

const fn p(kind: Kind) -> P {
    (kind, false)
}
const fn n(kind: Kind) -> P {
    (kind, true)
}

/// Patterns. `%..%` slots fill `a` then `b`, in order.
pub const CONDITION_PATTERNS: &[(&str, P)] = &[
    (
        "[the] damage (was|is|has) [been] (caused|done|made) by %object%",
        p(DamageCause),
    ),
    (
        "[the] damage (wasn't|was not|isn't|is not|hasn't|has not) [been] (caused|done|made) by %object%",
        n(DamageCause),
    ),
    ("[the] egg (will|is going to) hatch", p(EggHatch)),
    (
        "[the] egg (won't|will not|isn't going to|is not going to) hatch",
        n(EggHatch),
    ),
    ("[the] entity storage of %location% (is|are) full", p(StorageFull)),
    (
        "[the] entity storage of %location% (isn't|is not|aren't|are not) full",
        n(StorageFull),
    ),
    ("%location%'s entity storage (is|are) full", p(StorageFull)),
    (
        "%location%'s entity storage (isn't|is not|aren't|are not) full",
        n(StorageFull),
    ),
    ("%object% can be dispensed", p(Equip(EQ_DISPENSABLE))),
    (
        "%object% (can't|cannot|can not) be dispensed",
        n(Equip(EQ_DISPENSABLE)),
    ),
    ("%object% can be sheared off", p(Equip(EQ_SHEARABLE))),
    (
        "%object% (can't|cannot|can not) be sheared off",
        n(Equip(EQ_SHEARABLE)),
    ),
    (
        "%object% can be (equipped|put) (on|onto) entities",
        p(Equip(EQ_EQUIP_ON_INTERACT)),
    ),
    (
        "%object% can equip (on|onto) entities",
        p(Equip(EQ_EQUIP_ON_INTERACT)),
    ),
    (
        "%object% (can't|cannot|can not) be (equipped|put) (on|onto) entities",
        n(Equip(EQ_EQUIP_ON_INTERACT)),
    ),
    (
        "%object% (can't|cannot|can not) equip (on|onto) entities",
        n(Equip(EQ_EQUIP_ON_INTERACT)),
    ),
    ("%object% can (swap|switch) equipment", p(Equip(EQ_SWAPPABLE))),
    ("%object% can be (swapped|switched)", p(Equip(EQ_SWAPPABLE))),
    (
        "%object% (can't|cannot|can not) (swap|switch) equipment",
        n(Equip(EQ_SWAPPABLE)),
    ),
    (
        "%object% (can't|cannot|can not) be (swapped|switched)",
        n(Equip(EQ_SWAPPABLE)),
    ),
    (
        "%object% will (lose durability|be damaged) [(on [wearer's] injury|when [the] wearer [is] (hurt|injured|damaged))]",
        p(Equip(EQ_DAMAGE_ON_HURT)),
    ),
    (
        "%object% (won't|will not) (lose durability|be damaged) [(on [wearer's] injury|when [the] wearer [is] (hurt|injured|damaged))]",
        n(Equip(EQ_DAMAGE_ON_HURT)),
    ),
    (
        "bossbar %text% (has|have) [the] [flag] (darken sky|darken the sky|sky darkening) [flag]",
        p(BarFlag(crate::scoreboards_bossbars::BarFlag::DarkenSky)),
    ),
    (
        "bossbar %text% (doesn't|does not) have [the] [flag] (darken sky|darken the sky|sky darkening) [flag]",
        n(BarFlag(crate::scoreboards_bossbars::BarFlag::DarkenSky)),
    ),
    (
        "bossbar %text% (has|have) [the] [flag] [create] fog [flag]",
        p(BarFlag(crate::scoreboards_bossbars::BarFlag::Fog)),
    ),
    (
        "bossbar %text% (doesn't|does not) have [the] [flag] [create] fog [flag]",
        n(BarFlag(crate::scoreboards_bossbars::BarFlag::Fog)),
    ),
    (
        "bossbar %text% (has|have) [the] [flag] (dragon bar|[play] boss music) [flag]",
        p(BarFlag(crate::scoreboards_bossbars::BarFlag::Dragon)),
    ),
    (
        "bossbar %text% (doesn't|does not) have [the] [flag] (dragon bar|[play] boss music) [flag]",
        n(BarFlag(crate::scoreboards_bossbars::BarFlag::Dragon)),
    ),
    (
        "[the] [(entire|additional)] (tooltip|tooltips) of %object% (is|are) shown",
        p(Tooltip),
    ),
    (
        "[the] [(entire|additional)] (tooltip|tooltips) of %object% (is|are) hidden",
        n(Tooltip),
    ),
    (
        "%object%'s [(entire|additional)] (tooltip|tooltips) (is|are) shown",
        p(Tooltip),
    ),
    (
        "%object%'s [(entire|additional)] (tooltip|tooltips) (is|are) hidden",
        n(Tooltip),
    ),
    (
        "%object% (has|have) [(a|an|the)] (entire|additional) tooltip",
        p(Tooltip),
    ),
    (
        "%object% (doesn't|does not|don't|do not) have [(a|an|the)] (entire|additional) tooltip",
        n(Tooltip),
    ),
    ("%location% (has|have) [a] loot table", p(LootTable)),
    ("%location% (has|have) [a] loottable", p(LootTable)),
    (
        "%location% (doesn't|does not|don't|do not) have [a] loot table",
        n(LootTable),
    ),
    (
        "%location% (doesn't|does not|don't|do not) have [a] loottable",
        n(LootTable),
    ),
    ("%object% (has|have) metadata [(value|tag)] %text%", p(Metadata)),
    (
        "%object% (doesn't|does not|don't|do not) have metadata [(value|tag)] %text%",
        n(Metadata),
    ),
    (
        "%object% (has|have) [persistent] data (tag|tags|value|values) %text%",
        p(Persistent),
    ),
    (
        "%object% (doesn't|does not|don't|do not) have [persistent] data (tag|tags|value|values) %text%",
        n(Persistent),
    ),
    (
        "%player% (has|did) [already] (play|played) [on (this|the) server] (before|already)",
        p(PlayedBefore),
    ),
    (
        "%player% (hasn't|has not|didn't|did not) [(already|yet)] (play|played) [on (this|the) server] [(before|already|yet)]",
        n(PlayedBefore),
    ),
    (
        "%object% (is|are) %location%'s preferred (tool|tools)",
        p(PreferredTool),
    ),
    (
        "%object% (is|are) [(the|a)] preferred (tool|tools) (for|of) %object%",
        p(PreferredTool),
    ),
    (
        "%object% (isn't|is not|aren't|are not) %location%'s preferred (tool|tools)",
        n(PreferredTool),
    ),
    (
        "%object% (isn't|is not|aren't|are not) [(the|a)] preferred (tool|tools) (for|of) %object%",
        n(PreferredTool),
    ),
    ("%object% (is|are) responsive", p(Responsive)),
    ("%object% (isn't|is not|aren't|are not) responsive", n(Responsive)),
];

/// Words one of which every pattern above holds. Lines without any of them
/// skip the pattern table, so compiling ordinary conditions costs nothing.
const KEY_WORDS: &[&str] = &[
    "damage",
    "hatch",
    "storage",
    "dispensed",
    "sheared",
    "entities",
    "equipment",
    "swapped",
    "switched",
    "durability",
    "damaged",
    "bossbar",
    "tooltip",
    "tooltips",
    "loot",
    "loottable",
    "metadata",
    "data",
    "played",
    "play",
    "preferred",
    "responsive",
];

fn patterns() -> &'static [Pattern<P>] {
    static PATS: OnceLock<Vec<Pattern<P>>> = OnceLock::new();
    PATS.get_or_init(|| {
        CONDITION_PATTERNS
            .iter()
            .filter_map(|(s, k)| Pattern::new(s, *k).ok())
            .collect()
    })
}

/// Checks every pattern compiles. Used by tests.
pub fn check_patterns() -> Result<(), String> {
    for (s, k) in CONDITION_PATTERNS {
        Pattern::new(s, *k)?;
    }
    Ok(())
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
    let mut memo = Memo::default();
    let (kind, negate, mut caps) = patterns().iter().find_map(|pat| {
        pat.matches_memo(toks, env, &mut memo)
            .ok()
            .map(|c| (pat.kind.0, pat.kind.1, c))
    })?;
    let (a, b) = match kind {
        // The cause is the only slot; the event gives the rest.
        DamageCause => {
            if !env.has_damage || env.in_function {
                return None;
            }
            (None, cap(&mut caps, 0))
        }
        EggHatch => {
            let ev = crate::events_world_vehicles::ExtEvent::EggThrow;
            if env.world_event != Some(ev) || env.in_function {
                return None;
            }
            let i = ev.slot_index("hatching")?;
            (Some(Expr::WorldEventVal(i as u8)), None)
        }
        _ => (cap(&mut caps, 0), cap(&mut caps, 1)),
    };
    Some(Cond::FlagsMetadata(Box::new(FlagsCond { kind, negate, a, b })))
}

// ---------------------------------------------------------------- helpers

fn norm_id(s: &str) -> String {
    crate::conditions_entity_block::norm_id(s)
}

/// The item id of an item value or an item name.
pub fn item_id(v: &Value) -> Option<String> {
    match v {
        Value::Item { id, .. } => Some(norm_id(id)),
        Value::Text(t) if !t.trim().is_empty() => Some(norm_id(t)),
        _ => None,
    }
}

/// Default equippable flags of an item, `None` when it has no equippable
/// component.
pub fn equippable(id: &str) -> Option<u8> {
    EQUIPPABLE
        .binary_search_by(|(k, _)| (*k).cmp(id))
        .ok()
        .map(|i| EQUIPPABLE[i].1)
}

/// Damage causes by their Skript (Bukkit) names, mapped to the game's damage
/// types. The game's own names always match themselves.
const CAUSE_ALIASES: &[(&str, &[&str])] = &[
    (
        "entity_attack",
        &["mob_attack", "mob_attack_no_aggro", "player_attack"],
    ),
    ("entity_sweep_attack", &["player_attack"]),
    (
        "projectile",
        &[
            "arrow",
            "trident",
            "mob_projectile",
            "fireball",
            "unattributed_fireball",
            "wither_skull",
            "thrown",
            "wind_charge",
            "spit",
        ],
    ),
    ("fire", &["in_fire", "campfire"]),
    ("fire_tick", &["on_fire"]),
    ("drowning", &["drown"]),
    ("suffocation", &["in_wall"]),
    ("starvation", &["starve"]),
    ("lightning", &["lightning_bolt"]),
    ("block_explosion", &["explosion", "bad_respawn_point"]),
    (
        "entity_explosion",
        &["explosion", "player_explosion", "fireworks"],
    ),
    ("contact", &["cactus", "sweet_berry_bush"]),
    (
        "falling_block",
        &["falling_block", "falling_anvil", "falling_stalactite"],
    ),
    ("void", &["out_of_world"]),
    ("kill", &["generic_kill"]),
    ("world_border", &["outside_border"]),
    ("dryout", &["dry_out"]),
    ("magic", &["magic", "indirect_magic"]),
    ("custom", &["generic"]),
];

/// Does the event's damage cause (a game damage type) match the wanted name?
pub fn cause_matches(actual: &str, wanted: &str) -> bool {
    let actual = norm_id(actual);
    let wanted = norm_id(wanted);
    if actual == wanted {
        return true;
    }
    CAUSE_ALIASES
        .iter()
        .find(|(k, _)| *k == wanted)
        .is_some_and(|(_, v)| v.contains(&actual.as_str()))
}

fn in_tag(tag: &str, id: &str) -> bool {
    crate::skbee_raytrace_world::tag_members(crate::skbee_raytrace_world::TagKind::Block, tag)
        .is_some_and(|m| m.split(' ').any(|x| x == id))
}

/// Is this item the right tool to get drops from this block type? The first
/// rule that names the block decides; no rule means no.
pub fn correct_for_drops(item: &str, block: &str) -> bool {
    let Some((_, rules)) = TOOL_RULES.iter().find(|(items, _)| items.contains(&item)) else {
        return false;
    };
    for (blocks, ok) in rules.iter() {
        let hit = match blocks.strip_prefix('#') {
            Some(tag) => in_tag(tag, block),
            None => blocks.split(' ').any(|b| b == block),
        };
        if hit {
            return *ok;
        }
    }
    false
}

/// A block type from a location (read from the world) or an item or name.
fn block_id(v: &Value, host: &mut dyn Host) -> Option<String> {
    match v {
        Value::Location { x, y, z, world } => {
            let at = crate::block_ops::BlockPos {
                x: x.floor() as i32,
                y: y.floor() as i32,
                z: z.floor() as i32,
                world: world.clone(),
            };
            host.block_type(&at).map(|id| norm_id(&id))
        }
        other => item_id(other),
    }
}

fn block_at(v: &Value) -> Option<(Option<&str>, i32, i32, i32)> {
    match v {
        Value::Location { x, y, z, world } => Some((
            world.as_deref(),
            x.floor() as i32,
            y.floor() as i32,
            z.floor() as i32,
        )),
        _ => None,
    }
}

fn loot_table(v: &Value, host: &mut dyn Host) -> Option<bool> {
    let (world, x, y, z) = block_at(v)?;
    let w = match world {
        Some(w) => w.to_string(),
        None => host.default_world()?,
    };
    let Some(bytes) = host.block_entity_nbt(&w, x, y, z) else {
        return Some(false);
    };
    let nbt = crate::skbee_nbt::Compound::from_bytes(&bytes).ok()?;
    Some(nbt.get_path("LootTable").is_some())
}

/// Checks that need only the values and the game (not engine state).
/// `None` means the game could not answer.
pub fn check(kind: Kind, a: &Value, b: Option<&Value>, host: &mut dyn Host) -> Option<bool> {
    match kind {
        StorageFull => {
            let (world, x, y, z) = block_at(a)?;
            match host.flags_query(Query::BeeCount { world, x, y, z })? {
                Value::Num(n) => Some(n >= 3.0),
                _ => None,
            }
        }
        Equip(flag) => {
            let id = item_id(a)?;
            Some(equippable(&id).is_some_and(|f| f & flag != 0))
        }
        BarFlag(f) => {
            let Value::Text(id) = a else { return None };
            match host.flags_query(Query::BarFlag(id, f))? {
                Value::Bool(on) => Some(on),
                _ => None,
            }
        }
        // Item values carry no tooltip settings, so a tooltip is always shown.
        Tooltip => item_id(a).map(|_| true),
        LootTable => loot_table(a, host),
        PlayedBefore => {
            let Value::Player(pl) = a else { return None };
            match host.flags_query(Query::PlayedBefore(&pl.uuid))? {
                Value::Bool(b) => Some(b),
                _ => None,
            }
        }
        PreferredTool => {
            let item = item_id(a)?;
            let block = block_id(b?, host)?;
            let needs_tool = match host.flags_query(Query::ToolRequired(&block))? {
                Value::Bool(b) => b,
                _ => return None,
            };
            Some(!needs_tool || correct_for_drops(&item, &block))
        }
        Responsive => {
            let Value::Entity(e) = a else { return Some(false) };
            match host.flags_query(Query::Responsive(&e.uuid))? {
                Value::Bool(b) => Some(b),
                _ => None,
            }
        }
        // Answered by the engine.
        DamageCause | EggHatch | Metadata | Persistent => None,
    }
}

include!("data/equippable.rs");

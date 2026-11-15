//! Wave 4 conditions, part 1: `is alphanumeric`, `event is cancelled`,
//! `can hold`, `can build`, `can see`, `has line of sight`, `has item cooldown`,
//! `is sedated`, `respawn anchors work`, `has glowing text`, and a group of
//! entity and player checks the game answers through `Host::entity_check`
//! (`is sheared`, `has ai`, `can breed`, `bell is ringing`, ...).
//!
//! All of them run through one boxed `Cond` variant so the hot condition enum
//! does not grow. A check the game cannot answer is false, negated or not.

use crate::compile::CondKind;
use crate::expr::Expr;
use crate::host::Host;
use crate::value::{PlayerRef, Value};

/// A yes/no question the game answers about a player, entity or block.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Check {
    /// The bell block at a location is ringing.
    BellRinging,
    /// An animal is an adult and not in breeding cooldown.
    CanBreed,
    /// An enderman is being stared at.
    Stared,
    /// The entity is inside water or lava.
    InLiquid,
    /// The entity is inside water.
    Wet,
    /// A sheep has no wool.
    Sheared,
    /// A mob has its AI switched on.
    HasAi,
    /// The player's client shows chat colors.
    ChatColors,
    /// The player's client shows chat messages.
    ChatMessages,
    /// The player's client filters chat text.
    ChatFiltering,
    /// The player has their own weather set.
    ClientWeather,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Kind {
    Alphanumeric,
    EventCancelled,
    CanHold,
    CanBuild,
    CanPickUp,
    CanSee,
    LineOfSight,
    ItemCooldown,
    Sedated,
    AnchorsWork,
    GlowingText,
    CustomModelData,
    Host(Check),
}

/// One condition: what to check, its subject, and an optional second value.
#[derive(Debug, Clone, PartialEq)]
pub struct EntityCheckCond {
    pub kind: Kind,
    pub negate: bool,
    pub a: Expr,
    pub b: Option<Expr>,
}

const fn k(kind: Kind, negate: bool) -> CondKind {
    CondKind::EntityChecks(kind, negate)
}

const fn h(c: Check, negate: bool) -> CondKind {
    CondKind::EntityChecks(Kind::Host(c), negate)
}

/// Patterns. Tried before the plain `is` comparison.
pub const CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    ("%object% (is|are) alphanumeric", k(Kind::Alphanumeric, false)),
    (
        "%object% (is not|isn't|are not|aren't) alphanumeric",
        k(Kind::Alphanumeric, true),
    ),
    ("[the] event is cancelled", k(Kind::EventCancelled, false)),
    (
        "[the] event (is not|isn't) cancelled",
        k(Kind::EventCancelled, true),
    ),
    ("%player% can hold %item%", k(Kind::CanHold, false)),
    (
        "%player% (can't|cannot|can not) hold %item%",
        k(Kind::CanHold, true),
    ),
    ("%player%'s inventory can hold %item%", k(Kind::CanHold, false)),
    (
        "%player%'s inventory (can't|cannot|can not) hold %item%",
        k(Kind::CanHold, true),
    ),
    (
        "%player% can build [(at|in|on)] %location%",
        k(Kind::CanBuild, false),
    ),
    (
        "%player% (can't|cannot|can not) build [(at|in|on)] %location%",
        k(Kind::CanBuild, true),
    ),
    ("%player% can pick up items", k(Kind::CanPickUp, false)),
    (
        "%player% (can't|cannot|can not) pick up items",
        k(Kind::CanPickUp, true),
    ),
    (
        "%player% can see chat (colors|colours)",
        h(Check::ChatColors, false),
    ),
    (
        "%player% (can't|cannot|can not) see chat (colors|colours)",
        h(Check::ChatColors, true),
    ),
    ("%player% can see [chat] messages", h(Check::ChatMessages, false)),
    (
        "%player% (can't|cannot|can not) see [chat] messages",
        h(Check::ChatMessages, true),
    ),
    ("%player% can see %player%", k(Kind::CanSee, false)),
    (
        "%player% (can't|cannot|can not) see %player%",
        k(Kind::CanSee, true),
    ),
    (
        "%player% (has|have) chat filtering",
        h(Check::ChatFiltering, false),
    ),
    (
        "%player% (doesn't|does not|don't|do not) have chat filtering",
        h(Check::ChatFiltering, true),
    ),
    (
        "%object% (has|have) [a] [direct] line of sight (to|of) %object%",
        k(Kind::LineOfSight, false),
    ),
    (
        "%object% (doesn't|does not|don't|do not) have [a] [direct] line of sight (to|of) %object%",
        k(Kind::LineOfSight, true),
    ),
    (
        "%player% (has|have) [(a|an)] [item] cooldown (for|on) %item%",
        k(Kind::ItemCooldown, false),
    ),
    (
        "%player% (doesn't|does not|don't|do not) have [(a|an)] [item] cooldown (for|on) %item%",
        k(Kind::ItemCooldown, true),
    ),
    ("%location% (is|are) sedated", k(Kind::Sedated, false)),
    (
        "%location% (is not|isn't|are not|aren't) sedated",
        k(Kind::Sedated, true),
    ),
    (
        "respawn anchors (work|are active) in %world%",
        k(Kind::AnchorsWork, false),
    ),
    (
        "respawn anchors (don't|do not|doesn't|does not) work in %world%",
        k(Kind::AnchorsWork, true),
    ),
    ("%location% (has|have) glowing text", k(Kind::GlowingText, false)),
    (
        "%location% (doesn't|does not|don't|do not) have glowing text",
        k(Kind::GlowingText, true),
    ),
    (
        "%player% (has|have) [a] custom model data",
        k(Kind::CustomModelData, false),
    ),
    (
        "%player% (doesn't|does not|don't|do not) have [a] custom model data",
        k(Kind::CustomModelData, true),
    ),
    ("%location% (is|are) ringing", h(Check::BellRinging, false)),
    (
        "%location% (is not|isn't|are not|aren't) ringing",
        h(Check::BellRinging, true),
    ),
    ("%object% can breed", h(Check::CanBreed, false)),
    ("%object% (can't|cannot|can not) breed", h(Check::CanBreed, true)),
    ("%object% (has|have) been stared at", h(Check::Stared, false)),
    (
        "%object% (has not|hasn't|have not|haven't) been stared at",
        h(Check::Stared, true),
    ),
    (
        "%object% (is|are) in (liquid|a liquid)",
        h(Check::InLiquid, false),
    ),
    (
        "%object% (is not|isn't|are not|aren't) in (liquid|a liquid)",
        h(Check::InLiquid, true),
    ),
    ("%object% (is|are) wet", h(Check::Wet, false)),
    ("%object% (is not|isn't|are not|aren't) wet", h(Check::Wet, true)),
    ("%object% (is|are) sheared", h(Check::Sheared, false)),
    (
        "%object% (is not|isn't|are not|aren't) sheared",
        h(Check::Sheared, true),
    ),
    (
        "%object% (has|have) (ai|artificial intelligence)",
        h(Check::HasAi, false),
    ),
    (
        "%object% (doesn't|does not|don't|do not) have (ai|artificial intelligence)",
        h(Check::HasAi, true),
    ),
    (
        "%player% (has|have) (custom|client) weather",
        h(Check::ClientWeather, false),
    ),
    (
        "%player% (doesn't|does not|don't|do not) have (custom|client) weather",
        h(Check::ClientWeather, true),
    ),
];

pub fn build(kind: Kind, negate: bool, a: Expr, b: Option<Expr>) -> EntityCheckCond {
    EntityCheckCond { kind, negate, a, b }
}

/// Does this kind read a second value?
pub fn has_b(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::CanHold | Kind::CanBuild | Kind::CanSee | Kind::LineOfSight | Kind::ItemCooldown
    )
}

/// Does this kind read the first value? (`event is cancelled` has none.)
pub fn has_a(kind: Kind) -> bool {
    !matches!(kind, Kind::EventCancelled)
}

// ---------------------------------------------------------------- helpers

fn norm_id(s: &str) -> String {
    crate::conditions_entity_block::norm_id(s)
}

fn is_air(id: &str) -> bool {
    matches!(id, "air" | "cave_air" | "void_air")
}

fn live_player(v: &Value, host: &mut dyn Host) -> Option<PlayerRef> {
    match v {
        Value::Player(p) => host.resolve_player(&p.uuid).map(|name| PlayerRef {
            uuid: p.uuid.clone(),
            name,
        }),
        _ => None,
    }
}

fn item_of(v: &Value) -> Option<(String, u32)> {
    match v {
        Value::Item { id, count, .. } => Some((norm_id(id), (*count).max(1))),
        Value::Text(t) => Some((norm_id(t), 1)),
        _ => None,
    }
}

/// How many of an item fit in one slot. Names are matched by pattern, so new
/// items fall back to 64.
pub fn max_stack(id: &str) -> u32 {
    const ONE: &[&str] = &[
        "_sword",
        "_pickaxe",
        "_axe",
        "_shovel",
        "_hoe",
        "_helmet",
        "_chestplate",
        "_leggings",
        "_boots",
        "_boat",
        "_raft",
        "_minecart",
        "_bed",
        "_shulker_box",
        "_banner",
        "_horse_armor",
        "_potion",
    ];
    const ONE_EXACT: &[&str] = &[
        "bow",
        "crossbow",
        "trident",
        "shield",
        "elytra",
        "fishing_rod",
        "carrot_on_a_stick",
        "warped_fungus_on_a_stick",
        "shears",
        "flint_and_steel",
        "brush",
        "mace",
        "saddle",
        "totem_of_undying",
        "potion",
        "splash_potion",
        "lingering_potion",
        "water_bucket",
        "lava_bucket",
        "milk_bucket",
        "powder_snow_bucket",
        "cod_bucket",
        "salmon_bucket",
        "pufferfish_bucket",
        "tropical_fish_bucket",
        "axolotl_bucket",
        "tadpole_bucket",
        "enchanted_book",
        "written_book",
        "cake",
        "mushroom_stew",
        "rabbit_stew",
        "beetroot_soup",
        "suspicious_stew",
        "bundle",
        "compass",
        "recovery_compass",
        "spyglass",
        "goat_horn",
        "minecart",
        "chest_minecart",
        "furnace_minecart",
        "hopper_minecart",
        "tnt_minecart",
        "command_block_minecart",
        "knowledge_book",
        "filled_map",
    ];
    const SIXTEEN_EXACT: &[&str] = &[
        "ender_pearl",
        "snowball",
        "egg",
        "bucket",
        "honey_bottle",
        "armor_stand",
        "name_tag",
        "writable_book",
    ];
    if id.starts_with("music_disc_") || id.ends_with("_sign") || id.ends_with("_hanging_sign") {
        return if id.starts_with("music_disc_") { 1 } else { 16 };
    }
    if ONE_EXACT.contains(&id) || ONE.iter().any(|s| id.ends_with(s)) {
        return 1;
    }
    if SIXTEEN_EXACT.contains(&id) {
        return 16;
    }
    64
}

/// Can a player's main inventory take `count` of `id`? Slots 0..36 are the
/// hotbar and main inventory. A slot that reads as nothing is empty (the
/// Pumpkin host gives `None` for an empty slot).
fn can_hold(host: &mut dyn Host, p: &PlayerRef, id: &str, count: u32) -> Option<bool> {
    let cap = max_stack(id);
    let mut room: u64 = 0;
    let mut known = host.resolve_player(&p.uuid).is_some();
    for slot in 0..36 {
        match host.inv_get(p, false, slot) {
            None => room += u64::from(cap),
            Some((sid, n)) => {
                known = true;
                if sid.is_empty() || is_air(&sid) || n == 0 {
                    room += u64::from(cap);
                } else if norm_id(&sid) == id {
                    room += u64::from(cap.saturating_sub(n));
                }
            }
        }
    }
    known.then_some(room >= u64::from(count))
}

/// A position for line-of-sight checks: eye height for players.
fn sight_point(v: &Value, host: &mut dyn Host) -> Option<(f64, f64, f64, String)> {
    match v {
        Value::Player(p) => {
            let p = live_player(v, host).unwrap_or_else(|| p.clone());
            host.eye_location(&p)
                .or_else(|| host.player_location(&p).map(|(x, y, z, w)| (x, y + 1.62, z, w)))
        }
        Value::Entity(e) => host.entity_location(e).map(|(x, y, z, w)| (x, y + 1.0, z, w)),
        Value::Location { x, y, z, world } => {
            let w = match world {
                Some(w) => w.clone(),
                None => host.default_world()?,
            };
            Some((*x, *y, *z, w))
        }
        _ => None,
    }
}

const SIGHT_RANGE: f64 = 128.0;
const SIGHT_STEP: f64 = 0.25;

/// Walk the line between two points; any solid block on the way blocks it.
fn line_clear(host: &mut dyn Host, a: (f64, f64, f64, String), b: (f64, f64, f64, String)) -> Option<bool> {
    if !a.3.eq_ignore_ascii_case(&b.3) {
        return Some(false);
    }
    let (dx, dy, dz) = (b.0 - a.0, b.1 - a.1, b.2 - a.2);
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    if len > SIGHT_RANGE {
        return Some(false);
    }
    let steps = (len / SIGHT_STEP).ceil() as u32;
    let mut solid: Vec<(String, bool)> = Vec::new();
    let mut last = (i32::MIN, i32::MIN, i32::MIN);
    for i in 0..=steps {
        let t = if steps == 0 {
            0.0
        } else {
            f64::from(i) / f64::from(steps)
        };
        let (x, y, z) = (a.0 + dx * t, a.1 + dy * t, a.2 + dz * t);
        let cell = (x.floor() as i32, y.floor() as i32, z.floor() as i32);
        if cell == last {
            continue;
        }
        last = cell;
        let at = crate::block_ops::BlockPos {
            x: cell.0,
            y: cell.1,
            z: cell.2,
            world: Some(a.3.clone()),
        };
        let id = norm_id(&host.block_type(&at)?);
        if is_air(&id) {
            continue;
        }
        let s = match solid.iter().find(|(n, _)| *n == id) {
            Some((_, s)) => *s,
            None => {
                let s = host
                    .block_trait(&id, crate::conditions_entity_block::BlockTrait::Solid)
                    .unwrap_or(false);
                solid.push((id, s));
                s
            }
        };
        if s {
            return Some(false);
        }
    }
    Some(true)
}

fn block_at(v: &Value, host: &mut dyn Host, dy: i32) -> Option<(crate::block_ops::BlockPos, String)> {
    let Value::Location { x, y, z, world } = v else {
        return None;
    };
    let at = crate::block_ops::BlockPos {
        x: x.floor() as i32,
        y: y.floor() as i32 + dy,
        z: z.floor() as i32,
        world: world.clone(),
    };
    let id = norm_id(&host.block_type(&at)?);
    Some((at, id))
}

/// A hive is sedated when a campfire burns up to five blocks below it.
fn sedated(v: &Value, host: &mut dyn Host) -> Option<bool> {
    let (_, id) = block_at(v, host, 0)?;
    if id != "beehive" && id != "bee_nest" {
        return Some(false);
    }
    for d in 1..=5 {
        let (_, below) = block_at(v, host, -d)?;
        if below == "campfire" || below == "soul_campfire" {
            return Some(true);
        }
    }
    Some(false)
}

fn glowing_text(v: &Value, host: &mut dyn Host) -> Option<bool> {
    use crate::skbee_nbt::{Compound, Tag};
    let Value::Location { x, y, z, world } = v else {
        return None;
    };
    let w = match world {
        Some(w) => w.clone(),
        None => host.default_world()?,
    };
    let bytes = host.block_entity_nbt(&w, x.floor() as i32, y.floor() as i32, z.floor() as i32)?;
    let nbt = Compound::from_bytes(&bytes).ok()?;
    let on = |side: &str| {
        matches!(
            nbt.get_path(&format!("{side};has_glowing_text")),
            Some(Tag::Byte(b)) if *b != 0
        )
    };
    Some(on("front_text") || on("back_text"))
}

/// Check a condition (before negation). `None` means the game could not
/// answer, which counts as false whether the condition is negated or not.
pub fn check(kind: Kind, a: &Value, b: Option<&Value>, cancelled: bool, host: &mut dyn Host) -> Option<bool> {
    match kind {
        Kind::EventCancelled => Some(cancelled),
        Kind::Alphanumeric => {
            let ok = |v: &Value| {
                match v {
                Value::Text(t) => !t.is_empty() && t.chars().all(char::is_alphanumeric),
                Value::List(l) => !l.is_empty() && l.iter().all(|x| matches!(x, Value::Text(t) if !t.is_empty() && t.chars().all(char::is_alphanumeric))),
                _ => false,
            }
            };
            Some(ok(a))
        }
        Kind::CanHold => {
            let p = live_player(a, host)?;
            let (id, n) = item_of(b?)?;
            can_hold(host, &p, &id, n)
        }
        Kind::CanBuild => {
            let p = live_player(a, host)?;
            let mode = host.gamemode(&p)?;
            Some(!matches!(
                mode.to_ascii_lowercase().as_str(),
                "adventure" | "spectator"
            ))
        }
        Kind::CanPickUp => {
            let p = live_player(a, host)?;
            let mode = host.gamemode(&p)?;
            Some(!mode.eq_ignore_ascii_case("spectator"))
        }
        Kind::CanSee => {
            let viewer = live_player(a, host)?;
            let other = live_player(b?, host)?;
            Some(host.can_see(&viewer, &other))
        }
        Kind::LineOfSight => {
            let from = sight_point(a, host)?;
            let to = sight_point(b?, host)?;
            line_clear(host, from, to)
        }
        Kind::ItemCooldown => {
            let p = live_player(a, host)?;
            let (id, _) = item_of(b?)?;
            Some(host.item_cooldown(&p, &id).is_some_and(|t| t > 0))
        }
        Kind::Sedated => sedated(a, host),
        Kind::AnchorsWork => {
            let name = match a {
                Value::World(n) | Value::Text(n) => n.as_str(),
                _ => return None,
            };
            match host.server_query(crate::expr_vectors_server::Query::Environment(name))? {
                Value::Text(d) => {
                    Some(norm_id(&d).ends_with("nether") || norm_id(&d).ends_with("the_nether"))
                }
                _ => None,
            }
        }
        Kind::GlowingText => glowing_text(a, host),
        Kind::CustomModelData => {
            let p = live_player(a, host)?;
            Some(
                host.held_component(&p, crate::skbee_nbt::ItemComp::CustomModelData)
                    .is_some(),
            )
        }
        Kind::Host(c) => host.entity_check(c, a),
    }
}

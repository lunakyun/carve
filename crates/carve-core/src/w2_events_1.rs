//! Extra block, player and server events (anvils, beds, bells, brewing,
//! buckets, books, chunks, ...) and `at <time>:` triggers.
//!
//! All of them share one `EventKind::Ext(ExtEvent)` variant, so the hot
//! enums stay small. Event values (`event-block`, `event-location`, ...) are
//! stored in `EventCtx::ev` in the order of [`ExtEvent::fields`] and read by
//! the single `Expr::ExtVal(index)` variant.

use crate::expr::Ty;

/// One event from this module.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtEvent {
    /// `at 18:00:` with the time in world ticks (0..24000).
    At(u16),
    AnvilPrepare,
    ArmSwing,
    BedEnter,
    BedLeave,
    BellRing,
    BlockDamage,
    BlockDrop,
    BlockFertilize,
    BlockGrowth,
    BookEdit,
    BookSign,
    BrewingComplete,
    BrewingFuel,
    BrewingStart,
    Broadcast,
    BucketCatch,
    BucketEmpty,
    BucketFill,
    Burn,
    CanBuild,
    ChunkGenerate,
    ChunkUnload,
    Combust,
    Connect,
    Dispense,
    EggThrow,
    ElytraBoost,
}

type F = &'static [(&'static str, Ty)];

const LOC: (&str, Ty) = ("event-location", Ty::Location);
const WORLD: (&str, Ty) = ("event-world", Ty::World);
const NUM: (&str, Ty) = ("event-number", Ty::Number);
const TEXT: (&str, Ty) = ("event-text", Ty::Text);
const ITEM: (&str, Ty) = ("event-item", Ty::Item);

impl ExtEvent {
    /// Name used in messages and `/carve info`.
    pub fn name(self) -> &'static str {
        match self {
            ExtEvent::At(_) => "at time",
            ExtEvent::AnvilPrepare => "anvil prepare",
            ExtEvent::ArmSwing => "arm swing",
            ExtEvent::BedEnter => "bed enter",
            ExtEvent::BedLeave => "bed leave",
            ExtEvent::BellRing => "bell ring",
            ExtEvent::BlockDamage => "block damage",
            ExtEvent::BlockDrop => "block drop",
            ExtEvent::BlockFertilize => "block fertilize",
            ExtEvent::BlockGrowth => "block growth",
            ExtEvent::BookEdit => "book edit",
            ExtEvent::BookSign => "book sign",
            ExtEvent::BrewingComplete => "brewing complete",
            ExtEvent::BrewingFuel => "brewing fuel",
            ExtEvent::BrewingStart => "brewing start",
            ExtEvent::Broadcast => "broadcast",
            ExtEvent::BucketCatch => "bucket catch entity",
            ExtEvent::BucketEmpty => "bucket empty",
            ExtEvent::BucketFill => "bucket fill",
            ExtEvent::Burn => "burn",
            ExtEvent::CanBuild => "can build check",
            ExtEvent::ChunkGenerate => "chunk generate",
            ExtEvent::ChunkUnload => "chunk unload",
            ExtEvent::Combust => "combust",
            ExtEvent::Connect => "connect",
            ExtEvent::Dispense => "dispense",
            ExtEvent::EggThrow => "egg throw",
            ExtEvent::ElytraBoost => "elytra boost",
        }
    }

    /// Parse the text after `on ` (lowercase, trimmed).
    pub fn parse(s: &str) -> Option<ExtEvent> {
        let s = s.trim();
        Some(match s {
            "anvil prepare" | "prepare anvil" | "anvil prepare result" => ExtEvent::AnvilPrepare,
            "arm swing" | "swing" | "player arm swing" | "arm animation" => ExtEvent::ArmSwing,
            "bed enter" | "bed entering" | "player bed enter" => ExtEvent::BedEnter,
            "bed leave" | "bed leaving" | "player bed leave" => ExtEvent::BedLeave,
            "bell ring" | "bell rung" | "ring of bell" => ExtEvent::BellRing,
            "block damage" | "block damaging" | "damage of block" => ExtEvent::BlockDamage,
            "block drop" | "block drops" | "block drop item" | "drop of block" => ExtEvent::BlockDrop,
            "block fertilize" | "block fertilise" | "fertilize" | "fertilise" => ExtEvent::BlockFertilize,
            "block growth" | "block growing" | "growth" => ExtEvent::BlockGrowth,
            "book edit" | "book editing" | "edit book" | "player edit book" => ExtEvent::BookEdit,
            "book sign" | "book signing" | "sign book" | "player sign book" => ExtEvent::BookSign,
            "brewing complete" | "brewing finish" | "brew complete" | "brewing completing" => {
                ExtEvent::BrewingComplete
            }
            "brewing fuel" | "brewing fueling" | "brew fuel" => ExtEvent::BrewingFuel,
            "brewing start" | "brewing starting" | "brew start" => ExtEvent::BrewingStart,
            "broadcast" | "server broadcast" | "broadcasting" => ExtEvent::Broadcast,
            "bucket catch entity" | "bucket entity" | "entity bucket" | "bucket catch" => {
                ExtEvent::BucketCatch
            }
            "bucket empty" | "bucket emptying" | "empty bucket" | "player bucket empty" => {
                ExtEvent::BucketEmpty
            }
            "bucket fill" | "bucket filling" | "fill bucket" | "player bucket fill" => ExtEvent::BucketFill,
            "burn" | "block burn" | "block burning" => ExtEvent::Burn,
            "can build check" | "can build" | "build check" => ExtEvent::CanBuild,
            "chunk generate" | "chunk generation" | "chunk populate" | "chunk generating" => {
                ExtEvent::ChunkGenerate
            }
            "chunk unload" | "chunk unloading" => ExtEvent::ChunkUnload,
            "combust" | "combustion" | "entity combust" | "ignite entity" => ExtEvent::Combust,
            "connect" | "player connect" | "connection" => ExtEvent::Connect,
            "dispense" | "block dispense" | "dispenser dispense" => ExtEvent::Dispense,
            "egg throw" | "throwing of an egg" | "throw of egg" | "player egg throw" => ExtEvent::EggThrow,
            "elytra boost" | "elytra boosting" | "firework boost" => ExtEvent::ElytraBoost,
            _ => return None,
        })
    }

    /// Whether the event always has a player.
    pub fn has_player(self) -> bool {
        use ExtEvent::*;
        matches!(
            self,
            AnvilPrepare
                | ArmSwing
                | BedEnter
                | BedLeave
                | BlockDamage
                | BookEdit
                | BookSign
                | BucketCatch
                | BucketEmpty
                | BucketFill
                | CanBuild
                | Connect
                | EggThrow
                | ElytraBoost
        )
    }

    /// Whether the event has a player only sometimes (`player` may be unset).
    pub fn maybe_player(self) -> bool {
        matches!(self, ExtEvent::BlockDrop | ExtEvent::BlockFertilize)
    }

    /// Whether `cancel event` means anything.
    pub fn cancellable(self) -> bool {
        !matches!(
            self,
            ExtEvent::At(_) | ExtEvent::AnvilPrepare | ExtEvent::BedLeave
        )
    }

    /// The `event-...` values of the event, in the order the host fills
    /// `EventCtx::ev`.
    pub fn fields(self) -> F {
        use ExtEvent::*;
        match self {
            At(_) | Connect => &[],
            ArmSwing => &[TEXT],
            AnvilPrepare => &[TEXT, NUM],
            BedEnter | BedLeave => &[LOC],
            BellRing => &[LOC, WORLD, NUM, TEXT],
            BlockDamage => &[LOC, ("event-boolean", Ty::Boolean)],
            BlockDrop => &[LOC, WORLD, ITEM, NUM],
            BlockFertilize => &[LOC, WORLD, NUM],
            BlockGrowth => &[
                LOC,
                WORLD,
                ("event-block", Ty::Item),
                ("event-old-block", Ty::Item),
            ],
            BookEdit | BookSign => &[TEXT, NUM, ("event-pages", Ty::Text)],
            BrewingComplete | BrewingFuel => &[LOC, NUM],
            BrewingStart => &[LOC, WORLD, NUM],
            Broadcast => &[TEXT, ("event-sender", Ty::Text)],
            BucketCatch => &[ITEM, NUM],
            BucketEmpty | BucketFill => &[LOC, ITEM],
            Burn => &[
                LOC,
                WORLD,
                ("event-block", Ty::Item),
                ("event-igniting-block", Ty::Item),
            ],
            CanBuild => &[
                ("event-block", Ty::Item),
                ("event-existing-block", Ty::Item),
                ("event-boolean", Ty::Boolean),
            ],
            ChunkGenerate | ChunkUnload => &[("event-chunk-x", Ty::Number), ("event-chunk-z", Ty::Number)],
            Combust => &[NUM, ("event-timespan", Ty::Timespan)],
            Dispense => &[LOC, ITEM],
            EggThrow => &[("event-boolean", Ty::Boolean), NUM, TEXT],
            ElytraBoost => &[NUM],
        }
    }
}

/// Index and type of an `event-...` name in the fields of `ev`.
pub fn field_index(fields: F, name: &str) -> Option<(u8, Ty)> {
    fields
        .iter()
        .position(|(n, _)| *n == name)
        .map(|i| (i as u8, fields[i].1))
}

/// Parse the time of `at <time>:` (the text after `at `, lowercase, without
/// the colon). Accepts `18:00`, `6:30 pm`, `6pm`, `noon`, `midnight`.
/// Returns Minecraft ticks: 0 is 06:00, 6000 is noon, 18000 is midnight.
pub fn parse_at_time(s: &str) -> Option<u16> {
    let s = s.trim();
    let (hour, minute): (u32, u32) = match s {
        "noon" | "midday" => (12, 0),
        "midnight" => (0, 0),
        _ => {
            let (body, pm) = if let Some(b) = s.strip_suffix("am") {
                (b.trim(), Some(false))
            } else if let Some(b) = s.strip_suffix("pm") {
                (b.trim(), Some(true))
            } else {
                (s, None)
            };
            let (h, m) = match body.split_once(':') {
                Some((h, m)) => (h.trim().parse::<u32>().ok()?, m.trim().parse::<u32>().ok()?),
                None if pm.is_some() => (body.parse::<u32>().ok()?, 0),
                None => return None,
            };
            if m > 59 {
                return None;
            }
            let h = match pm {
                Some(p) => {
                    if !(1..=12).contains(&h) {
                        return None;
                    }
                    (h % 12) + if p { 12 } else { 0 }
                }
                None => {
                    if h > 23 {
                        return None;
                    }
                    h
                }
            };
            (h, m)
        }
    };
    // 1000 ticks per hour, and 06:00 is tick 0.
    let ticks = (hour * 1000 + minute * 1000 / 60 + 18000) % 24000;
    Some(ticks as u16)
}

/// The `at` times (in ticks) that fall after `prev` up to and including `cur`
/// when the world clock went from `prev` to `cur` (it may wrap past 24000).
pub fn crossed(prev: i64, cur: i64, target: u16) -> bool {
    let (p, c, t) = (prev.rem_euclid(24000), cur.rem_euclid(24000), i64::from(target));
    if prev == cur {
        false
    } else if p < c {
        p < t && t <= c
    } else {
        // Wrapped (or time was set back): fire only on a forward wrap.
        t > p || t <= c
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn times() {
        assert_eq!(parse_at_time("6:00"), Some(0));
        assert_eq!(parse_at_time("18:00"), Some(12000));
        assert_eq!(parse_at_time("12:00"), Some(6000));
        assert_eq!(parse_at_time("6pm"), Some(12000));
        assert_eq!(parse_at_time("12am"), Some(18000));
        assert_eq!(parse_at_time("noon"), Some(6000));
        assert_eq!(parse_at_time("25:00"), None);
        assert_eq!(parse_at_time("abc"), None);
    }

    #[test]
    fn crossing() {
        assert!(crossed(5999, 6000, 6000));
        assert!(!crossed(6000, 6001, 6000));
        assert!(crossed(23999, 1, 0));
        assert!(crossed(23999, 1, 1));
        assert!(!crossed(100, 100, 100));
    }
}

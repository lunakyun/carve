//! More events: server list ping, sign change, smelting, vehicles, worlds,
//! weather, targeting, taming, and `at 14:30 in real time:`.
//!
//! Every event here is one `ExtEvent`. It travels inside `EventKind::Ext4`, so
//! the shared event enum grows by one variant only. The values an event
//! offers (`event-motd`, `event-world`, ...) live in `EventCtx::args`, in the
//! order of [`ExtEvent::slots`]. Scripts read them with `event-<name>`, and
//! write the writable ones with `set event-<name> to <value>`.

use crate::compile::{Effect, EventKind};
use crate::engine::{Engine, EventCtx};
use crate::expr::{Env, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ExtEvent {
    ServerListPing,
    SheepRegrowWool,
    Shoot,
    SignChange,
    SlimeSplit,
    Smelt,
    StartSmelt,
    SpawnChange,
    SpongeAbsorb,
    Spread,
    SwimToggle,
    /// `at HH:MM in real time`, as minutes since midnight (UTC).
    SystemTime(u16),
    Tame,
    Target,
    EggThrow,
    ToolChange,
    VaultDisplayItem,
    VehicleCollision,
    VehicleCreate,
    VehicleDamage,
    VehicleDestroy,
    VehicleEnter,
    VehicleExit,
    VehicleMove,
    VillagerCareerChange,
    WeatherChange,
    WeatherToRain,
    WeatherToClear,
    WorldInit,
    WorldLoad,
    WorldSave,
    WorldUnload,
    ZombieBreakDoor,
    /// `on placeholder request for prefix "x"` (see `placeholders`).
    PlaceholderRequest,
    /// `on preparing craft` (see `w5_chunk1`).
    PrepareCraft,
    // Wave 6 chunk 1 (see `w6_chunk1`).
    PickupArrow,
    Riptide,
    ExperienceSpawn,
    ProjectileCollide,
    PressurePlate,
    ChunkEnter,
    StepOn,
    Jump,
    BorderBounds,
    BorderCenter,
    ArmorChange,
    Spectate,
    ScriptUnload,
    ServerStop,
}

/// One named value of an event. Writable ones can be changed with `set`.
pub type Slot = (&'static str, bool);

macro_rules! r {
    ($n:literal) => {
        ($n, false)
    };
}
macro_rules! w {
    ($n:literal) => {
        ($n, true)
    };
}

/// Header texts (after `on `) and the event they name.
const NAMES: &[(&str, ExtEvent)] = &[
    ("server list ping", ExtEvent::ServerListPing),
    ("prepare craft", ExtEvent::PrepareCraft),
    ("preparing craft", ExtEvent::PrepareCraft),
    ("prepare crafting", ExtEvent::PrepareCraft),
    ("preparing crafting", ExtEvent::PrepareCraft),
    ("beginning craft", ExtEvent::PrepareCraft),
    ("beginning crafting", ExtEvent::PrepareCraft),
    ("player preparing craft", ExtEvent::PrepareCraft),
    ("player beginning craft", ExtEvent::PrepareCraft),
    ("list ping", ExtEvent::ServerListPing),
    ("server ping", ExtEvent::ServerListPing),
    ("ping", ExtEvent::ServerListPing),
    ("sheep regrow wool", ExtEvent::SheepRegrowWool),
    ("sheep wool regrow", ExtEvent::SheepRegrowWool),
    ("shoot", ExtEvent::Shoot),
    ("entity shoot", ExtEvent::Shoot),
    ("shoot bow", ExtEvent::Shoot),
    ("sign change", ExtEvent::SignChange),
    ("sign edit", ExtEvent::SignChange),
    ("slime split", ExtEvent::SlimeSplit),
    ("smelt", ExtEvent::Smelt),
    ("furnace smelt", ExtEvent::Smelt),
    ("start smelt", ExtEvent::StartSmelt),
    ("start smelting", ExtEvent::StartSmelt),
    ("furnace start smelt", ExtEvent::StartSmelt),
    ("spawn change", ExtEvent::SpawnChange),
    ("world spawn change", ExtEvent::SpawnChange),
    ("sponge absorb", ExtEvent::SpongeAbsorb),
    ("spread", ExtEvent::Spread),
    ("block spread", ExtEvent::Spread),
    ("swim toggle", ExtEvent::SwimToggle),
    ("toggle swim", ExtEvent::SwimToggle),
    ("entity toggle swim", ExtEvent::SwimToggle),
    ("tame", ExtEvent::Tame),
    ("entity tame", ExtEvent::Tame),
    ("target", ExtEvent::Target),
    ("entity target", ExtEvent::Target),
    ("throwing of an egg", ExtEvent::EggThrow),
    ("throwing of egg", ExtEvent::EggThrow),
    ("throw of an egg", ExtEvent::EggThrow),
    ("tool change", ExtEvent::ToolChange),
    ("player tool change", ExtEvent::ToolChange),
    ("held item change", ExtEvent::ToolChange),
    ("vault display item", ExtEvent::VaultDisplayItem),
    ("vehicle collision", ExtEvent::VehicleCollision),
    ("vehicle create", ExtEvent::VehicleCreate),
    ("vehicle damage", ExtEvent::VehicleDamage),
    ("vehicle destroy", ExtEvent::VehicleDestroy),
    ("vehicle enter", ExtEvent::VehicleEnter),
    ("vehicle exit", ExtEvent::VehicleExit),
    ("vehicle move", ExtEvent::VehicleMove),
    ("villager career change", ExtEvent::VillagerCareerChange),
    ("weather change", ExtEvent::WeatherChange),
    ("weather change to rain", ExtEvent::WeatherToRain),
    ("weather change to rainy", ExtEvent::WeatherToRain),
    ("weather change to raining", ExtEvent::WeatherToRain),
    ("weather change to clear", ExtEvent::WeatherToClear),
    ("weather change to sun", ExtEvent::WeatherToClear),
    ("weather change to sunny", ExtEvent::WeatherToClear),
    ("world init", ExtEvent::WorldInit),
    ("world load", ExtEvent::WorldLoad),
    ("world save", ExtEvent::WorldSave),
    ("world unload", ExtEvent::WorldUnload),
    ("zombie break door", ExtEvent::ZombieBreakDoor),
    ("entity break door", ExtEvent::ZombieBreakDoor),
    ("break door", ExtEvent::ZombieBreakDoor),
    ("placeholder request", ExtEvent::PlaceholderRequest),
    ("player pickup arrow", ExtEvent::PickupArrow),
    ("pickup arrow", ExtEvent::PickupArrow),
    ("arrow pickup", ExtEvent::PickupArrow),
    ("riptide", ExtEvent::Riptide),
    ("player riptide", ExtEvent::Riptide),
    ("experience spawn", ExtEvent::ExperienceSpawn),
    ("xp spawn", ExtEvent::ExperienceSpawn),
    ("exp spawn", ExtEvent::ExperienceSpawn),
    ("projectile collide", ExtEvent::ProjectileCollide),
    ("pressure plate", ExtEvent::PressurePlate),
    ("pressure plate press", ExtEvent::PressurePlate),
    ("trip", ExtEvent::PressurePlate),
    ("pressure plate trip", ExtEvent::PressurePlate),
    ("player chunk enter", ExtEvent::ChunkEnter),
    ("player chunk enters", ExtEvent::ChunkEnter),
    ("chunk enter", ExtEvent::ChunkEnter),
    ("player enters chunk", ExtEvent::ChunkEnter),
    ("step on", ExtEvent::StepOn),
    ("walk on", ExtEvent::StepOn),
    ("move on", ExtEvent::StepOn),
    ("jump", ExtEvent::Jump),
    ("player jump", ExtEvent::Jump),
    ("world border bounds change", ExtEvent::BorderBounds),
    ("border bounds change", ExtEvent::BorderBounds),
    ("world border center change", ExtEvent::BorderCenter),
    ("border center change", ExtEvent::BorderCenter),
    ("armor change", ExtEvent::ArmorChange),
    ("armour change", ExtEvent::ArmorChange),
    ("player armor change", ExtEvent::ArmorChange),
    ("spectate", ExtEvent::Spectate),
    ("player spectate", ExtEvent::Spectate),
    ("unload", ExtEvent::ScriptUnload),
    ("script unload", ExtEvent::ScriptUnload),
    ("server stop", ExtEvent::ServerStop),
    ("server shutdown", ExtEvent::ServerStop),
];

impl ExtEvent {
    /// Event for a header like `server list ping` (already lowercase, no `on `).
    pub fn parse(s: &str) -> Option<ExtEvent> {
        NAMES.iter().find(|(n, _)| *n == s).map(|(_, e)| *e)
    }

    pub fn name(self) -> &'static str {
        match self {
            ExtEvent::SystemTime(_) => "system time",
            e => NAMES.iter().find(|(_, x)| *x == e).map_or("event", |(n, _)| n),
        }
    }

    /// The named values, in `EventCtx::args` order.
    pub fn slots(self) -> &'static [Slot] {
        use ExtEvent::*;
        match self {
            ServerListPing => &[
                w!("motd"),
                w!("max-players"),
                w!("players"),
                r!("hostname"),
                r!("address"),
                w!("icon"),
            ],
            PrepareCraft => &[r!("item"), r!("recipe")],
            SheepRegrowWool => &[r!("entity")],
            Shoot => &[r!("entity"), r!("weapon"), r!("force"), r!("projectile")],
            SignChange => &[
                r!("block"),
                w!("line-1"),
                w!("line-2"),
                w!("line-3"),
                w!("line-4"),
            ],
            SlimeSplit => &[r!("entity"), w!("count")],
            Smelt => &[r!("block"), r!("source"), r!("result")],
            StartSmelt => &[r!("block"), r!("source"), w!("time")],
            SpawnChange => &[r!("world"), r!("previous-location"), r!("new-location")],
            SpongeAbsorb => &[r!("block")],
            Spread => &[r!("block"), r!("source-block"), r!("world")],
            SwimToggle => &[r!("entity"), r!("swimming")],
            SystemTime(_) => &[],
            Tame => &[r!("entity"), r!("owner")],
            Target => &[r!("entity"), r!("target")],
            EggThrow => &[w!("hatching"), w!("hatches"), r!("hatch-type")],
            ToolChange => &[
                r!("previous-slot"),
                r!("new-slot"),
                r!("item"),
                r!("previous-item"),
            ],
            VaultDisplayItem => &[r!("block"), r!("world"), r!("item")],
            VehicleCollision => &[r!("vehicle"), r!("entity"), r!("block")],
            VehicleCreate => &[r!("vehicle")],
            VehicleDamage => &[r!("vehicle"), r!("damage"), r!("attacker")],
            VehicleDestroy => &[r!("vehicle"), r!("attacker")],
            VehicleEnter => &[r!("vehicle"), r!("entity")],
            VehicleExit => &[r!("vehicle"), r!("entity")],
            VehicleMove => &[r!("vehicle"), r!("from"), r!("to")],
            VillagerCareerChange => &[r!("entity"), r!("profession"), r!("reason")],
            WeatherChange | WeatherToRain | WeatherToClear => &[r!("world"), r!("weather")],
            WorldInit | WorldLoad | WorldUnload => &[r!("world")],
            WorldSave => &[r!("world")],
            ZombieBreakDoor => &[r!("entity"), r!("block")],
            PlaceholderRequest => &[r!("identifier"), w!("result"), r!("prefix")],
            PickupArrow => &[r!("entity")],
            Riptide => &[r!("item")],
            ExperienceSpawn => &[r!("entity"), r!("location")],
            ProjectileCollide => &[r!("entity"), r!("hit-entity"), r!("location")],
            PressurePlate => &[r!("location"), r!("power")],
            ChunkEnter => &[
                r!("chunk-x"),
                r!("chunk-z"),
                r!("previous-chunk-x"),
                r!("previous-chunk-z"),
            ],
            StepOn => &[r!("block"), r!("location")],
            Jump => &[],
            BorderBounds => &[r!("world"), r!("old-size"), r!("new-size")],
            BorderCenter => &[r!("world"), r!("old-x"), r!("old-z"), r!("new-x"), r!("new-z")],
            ArmorChange => &[r!("slot"), r!("item"), r!("previous-item")],
            Spectate => &[r!("entity")],
            ScriptUnload | ServerStop => &[],
        }
    }

    /// Position of `event-<name>` in `EventCtx::args`.
    pub fn slot_index(self, name: &str) -> Option<usize> {
        self.slots().iter().position(|(n, _)| *n == name)
    }

    /// Whether `cancel event` means anything here.
    pub fn cancellable(self) -> bool {
        use ExtEvent::*;
        !matches!(
            self,
            ServerListPing
                | SystemTime(_)
                | SpawnChange
                | WorldInit
                | WorldLoad
                | PlaceholderRequest
                | ExperienceSpawn
                | PressurePlate
                | ChunkEnter
                | StepOn
                | Jump
                | BorderBounds
                | BorderCenter
                | ArmorChange
                | Spectate
                | ScriptUnload
                | ServerStop
        )
    }

    /// Events that can name a player as `player`.
    pub fn has_player(self) -> bool {
        use ExtEvent::*;
        matches!(
            self,
            SignChange
                | EggThrow
                | ToolChange
                | SwimToggle
                | Tame
                | Shoot
                | VehicleDamage
                | VehicleDestroy
                | VehicleEnter
                | VehicleExit
                | PrepareCraft
                | PickupArrow
                | Riptide
                | ChunkEnter
                | StepOn
                | Jump
                | ArmorChange
                | Spectate
        )
    }
}

/// `at 14:30 in real time` / `at 6pm in real time`: minutes since midnight.
pub fn parse_at_real_time(lower: &str) -> Option<u16> {
    let rest = lower.strip_prefix("at ")?.trim();
    let t = rest
        .strip_suffix("in real time")
        .or_else(|| rest.strip_suffix("in real-time"))?
        .trim();
    parse_clock(t)
}

fn parse_clock(t: &str) -> Option<u16> {
    let (body, pm) = if let Some(b) = t.strip_suffix("pm") {
        (b.trim(), Some(true))
    } else if let Some(b) = t.strip_suffix("am") {
        (b.trim(), Some(false))
    } else {
        (t, None)
    };
    let (h, m) = match body.split_once(':') {
        Some((h, m)) => (h.trim().parse::<u32>().ok()?, m.trim().parse::<u32>().ok()?),
        None => (body.parse::<u32>().ok()?, 0),
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
    Some((h * 60 + m) as u16)
}

/// What a `Effect::Ext4` does.
#[derive(Debug, Clone, PartialEq)]
pub enum ExtEffect {
    /// `set event-<name> to <value>`: slot index and the new value.
    SetVal(u8, crate::expr::Expr),
}

/// `set event-motd to "x"` inside an event of this module.
/// `None`: this line is not one of ours.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Option<Result<Effect, String>> {
    let ev = env.ext4?;
    if !toks.first()?.is_lit("set") {
        return None;
    }
    let at = if ev == ExtEvent::PlaceholderRequest && toks.get(1)?.is_lit("the") {
        2
    } else {
        1
    };
    let name = match toks.get(at)? {
        Tok::Word(w) => w.to_ascii_lowercase(),
        _ => return None,
    };
    let name = match name.strip_prefix("event-") {
        Some(n) => n,
        None if ev == ExtEvent::PlaceholderRequest => crate::placeholders::slot_alias(&name)?,
        None => return None,
    };
    let idx = match ev.slot_index(name) {
        Some(i) => i,
        None => {
            let names: Vec<_> = ev.slots().iter().map(|(n, _)| format!("event-{n}")).collect();
            return Some(Err(format!(
                "'on {}' has no value 'event-{name}' (it has: {})",
                ev.name(),
                if names.is_empty() {
                    "none".into()
                } else {
                    names.join(", ")
                }
            )));
        }
    };
    if !ev.slots()[idx].1 {
        return Some(Err(format!(
            "'event-{name}' can't be changed in 'on {}'",
            ev.name()
        )));
    }
    if !toks.get(at + 1)?.is_lit("to") {
        return None;
    }
    let value = match parse_expr(toks.get(at + 2..)?, env) {
        Ok(e) => e,
        Err(e) => return Some(Err(e)),
    };
    Some(Ok(Effect::Ext4(Box::new(ExtEffect::SetVal(idx as u8, value)))))
}

/// A fresh context for an event of this module. `vals` must follow the slot
/// order of the event.
pub fn ctx(player: Option<crate::value::PlayerRef>, vals: Vec<Value>) -> EventCtx {
    let mut c = match player {
        Some(p) => EventCtx::player(p),
        None => EventCtx::console(),
    };
    c.args = vals;
    c
}

/// Fire `at HH:MM in real time` triggers. Call once per tick; `last` is the
/// minute (since 1970) that was handled before, start it at 0. Runs the
/// triggers once when the minute changes. The clock is UTC.
pub fn poll_system_time(engine: &mut Engine, last: &mut i64, host: &mut dyn Host) {
    let minute = host.wall_clock_millis().div_euclid(60_000);
    if minute == *last {
        return;
    }
    let first = *last == 0;
    *last = minute;
    if first {
        return;
    }
    let tod = minute.rem_euclid(1440) as u16;
    let ev = EventKind::Ext4(ExtEvent::SystemTime(tod));
    if engine.table().has_event(ev) {
        engine.fire(ev, EventCtx::console(), host);
    }
}

//! Movement and command events: `on first join`, `on command`, `on move`,
//! `on swap hand items`, `on teleport` and `on world change`.
//!
//! Every event is one [`MvEvent`] inside `EventKind::Mv`. The values a script
//! can read (`event-location`, `command`, ...) are listed in [`MvEvent::fields`]
//! and sit in `EventCtx::ev` in that order, like the events of `events_blocks_players`.

use crate::expr::{Expr, Part, Ty};
use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MvEvent {
    /// A player runs a command (`command`, `full command`).
    Command,
    /// A player joins for the first time ever.
    FirstJoin,
    /// A player moves to another block.
    Move,
    /// A player swaps the main hand and off hand items.
    SwapHands,
    /// A player is teleported.
    Teleport,
    /// A player arrives in another world.
    WorldChange,
    /// A player presses tab in a command: never fires, the plugin API has no such event.
    TabComplete,
}

const NAMES: &[(&str, MvEvent)] = &[
    ("command", MvEvent::Command),
    ("player command", MvEvent::Command),
    ("first join", MvEvent::FirstJoin),
    ("first login", MvEvent::FirstJoin),
    ("player first join", MvEvent::FirstJoin),
    ("move", MvEvent::Move),
    ("player move", MvEvent::Move),
    ("walk", MvEvent::Move),
    ("swap hand items", MvEvent::SwapHands),
    ("swap hands", MvEvent::SwapHands),
    ("hand swap", MvEvent::SwapHands),
    ("player swap hand items", MvEvent::SwapHands),
    ("teleport", MvEvent::Teleport),
    ("player teleport", MvEvent::Teleport),
    ("world change", MvEvent::WorldChange),
    ("player world change", MvEvent::WorldChange),
    ("world switch", MvEvent::WorldChange),
    ("changed world", MvEvent::WorldChange),
    ("tab complete", MvEvent::TabComplete),
    ("tab completion", MvEvent::TabComplete),
    ("player tab complete", MvEvent::TabComplete),
];

type F = &'static [(&'static str, Ty)];

const COMMAND_FIELDS: F = &[("event-command", Ty::Text), ("event-full-command", Ty::Text)];
const LOC_FIELDS: F = &[
    ("event-location", Ty::Location),
    ("event-past-location", Ty::Location),
];
const WORLD_FIELDS: F = &[("event-world", Ty::World), ("event-past-world", Ty::World)];

impl MvEvent {
    /// Event for a header like `first join` (lowercase, without `on `).
    pub fn parse(s: &str) -> Option<MvEvent> {
        NAMES.iter().find(|(n, _)| *n == s).map(|(_, e)| *e)
    }

    pub fn name(self) -> &'static str {
        NAMES.iter().find(|(_, e)| *e == self).map_or("event", |(n, _)| n)
    }

    /// Whether `cancel event` means anything here.
    pub fn cancellable(self) -> bool {
        matches!(
            self,
            MvEvent::Command | MvEvent::Move | MvEvent::SwapHands | MvEvent::Teleport | MvEvent::TabComplete
        )
    }

    /// The `event-...` values, in `EventCtx::ev` order.
    pub fn fields(self) -> F {
        match self {
            MvEvent::Command => COMMAND_FIELDS,
            MvEvent::Move | MvEvent::Teleport => LOC_FIELDS,
            MvEvent::WorldChange => WORLD_FIELDS,
            MvEvent::FirstJoin | MvEvent::SwapHands | MvEvent::TabComplete => &[],
        }
    }

    pub fn all() -> impl Iterator<Item = MvEvent> {
        [
            MvEvent::Command,
            MvEvent::FirstJoin,
            MvEvent::Move,
            MvEvent::SwapHands,
            MvEvent::Teleport,
            MvEvent::WorldChange,
            MvEvent::TabComplete,
        ]
        .into_iter()
    }
}

/// `command "home"` -> (`command`, Some(`home`)). Also `command /home`.
/// Other headers come back unchanged.
pub fn split_command_filter(ev: &str) -> (&str, Option<String>) {
    for base in ["player command", "command"] {
        let Some(rest) = ev.strip_prefix(base) else {
            continue;
        };
        if !rest.starts_with(' ') {
            continue;
        }
        let text = rest
            .trim()
            .trim_matches('"')
            .trim()
            .trim_start_matches('/')
            .trim();
        if !text.is_empty() {
            return (base, Some(text.to_string()));
        }
    }
    (ev, None)
}

/// The guard for `on command "home":`: what to compare, and with what. A
/// filter with a space (`"gamemode creative"`) compares the full command.
pub fn command_guard(filter: &str) -> (Expr, Expr) {
    let f = filter.to_ascii_lowercase();
    let idx = u8::from(f.contains(' '));
    (Expr::ExtVal(idx), Expr::Text(vec![Part::Lit(f)]))
}

/// `/Home  a b` -> (`home`, `home  a b`): the command name and the whole
/// command, lowercase name, no leading slash.
pub fn split_command(raw: &str) -> (String, String) {
    let full = raw.trim().trim_start_matches('/').trim_start();
    let label = full.split_whitespace().next().unwrap_or("").to_ascii_lowercase();
    let rest = full.get(label.len()..).unwrap_or("");
    let full = format!("{label}{rest}");
    (label, full)
}

/// `on move` is throttled: it only runs when the player enters another block.
pub fn moved_enough(from: (f64, f64, f64), to: (f64, f64, f64)) -> bool {
    from.0.floor() != to.0.floor() || from.1.floor() != to.1.floor() || from.2.floor() != to.2.floor()
}

/// `EventCtx::ev` of `on command`.
pub fn command_vals(raw: &str) -> Vec<Value> {
    let (label, full) = split_command(raw);
    vec![Value::Text(label), Value::Text(full)]
}

/// `EventCtx::ev` of `on move` and `on teleport`: where to, where from.
pub fn loc_vals(to: (f64, f64, f64), from: (f64, f64, f64), world: Option<String>) -> Vec<Value> {
    let loc = |p: (f64, f64, f64)| Value::Location {
        x: p.0,
        y: p.1,
        z: p.2,
        world: world.clone(),
    };
    vec![loc(to), loc(from)]
}

/// `EventCtx::ev` of `on world change`: the new world, the old world.
pub fn world_vals(to: String, from: String) -> Vec<Value> {
    vec![Value::World(to), Value::World(from)]
}

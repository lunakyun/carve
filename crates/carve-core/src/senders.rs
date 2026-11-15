//! Senders: `sender`, `console`, `me`, `make X execute command`, and the
//! "is a player / is console" conditions. The expressions live in `expr.rs`;
//! this file holds the extra patterns and the small kinds they build.

use crate::compile::{CondKind, EffKind};

/// Extra effect patterns. Tried before the main table.
pub const SENDER_EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    (
        "make %object% (execute|run) [the] [command] %text%",
        EffKind::ExecAs,
    ),
    (
        "force %object% to (execute|run) [the] [command] %text%",
        EffKind::ExecAs,
    ),
    ("[(execute|run)] player command %text%", EffKind::ExecAsPlayer),
    ("[(execute|run)] console command %text%", EffKind::ConsoleCommand),
    (
        "message %text% to (all|every) [online] (player|players)",
        EffKind::SendAllPlayers,
    ),
    (
        "send %text% to (all|every) [online] (player|players)",
        EffKind::SendAllPlayers,
    ),
    ("message %text% [to %player%]", EffKind::Send),
    ("clear %variable%", EffKind::Delete),
    (
        "[(force|forcibly)] teleport %player% at %location%",
        EffKind::TeleportTo,
    ),
    ("[execute] [the] command %text% by %object%", EffKind::ExecAsBy),
    ("execute [the] command %text%", EffKind::ConsoleCommand),
    ("command %text%", EffKind::ConsoleCommand),
];

/// Extra condition patterns. Tried before the main table, so they win over
/// the plain `is` comparison.
pub const SENDER_CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    (
        "%player% (doesn't|does not|do not|don't) have [the] permission %text%",
        CondKind::NotHasPerm,
    ),
    ("%player% (has|have) [the] permission %text%", CondKind::HasPerm),
    ("%text% (do not|don't) contain %text%", CondKind::NotContains),
    ("%object% not set", CondKind::NotSet),
    ("%object% is (a|an) player", CondKind::IsPlayer),
    ("%object% (is not|isn't) (a|an) player", CondKind::IsNotPlayer),
    ("%object% is [the] console", CondKind::IsConsole),
    ("%object% (is not|isn't) [the] console", CondKind::IsNotConsole),
];

//! Senders: `sender`, `console`, `me`, `make X execute command`, and the
//! "is a player / is console" conditions. The expressions live in `expr.rs`;
//! this file holds the extra patterns and the small kinds they build.

use crate::compile::{CondKind, EffKind};

/// Extra effect patterns. Tried before the main table.
pub const SENDER_EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    ("make %object% execute [the] [command] %text%", EffKind::ExecAs),
    (
        "force %object% to execute [the] [command] %text%",
        EffKind::ExecAs,
    ),
    ("execute [the] command %text% by %object%", EffKind::ExecAsBy),
    ("execute [the] command %text%", EffKind::ConsoleCommand),
];

/// Extra condition patterns. Tried before the main table, so they win over
/// the plain `is` comparison.
pub const SENDER_CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    ("%object% is (a|an) player", CondKind::IsPlayer),
    ("%object% (is not|isn't) (a|an) player", CondKind::IsNotPlayer),
    ("%object% is [the] console", CondKind::IsConsole),
    ("%object% (is not|isn't) [the] console", CondKind::IsNotConsole),
];

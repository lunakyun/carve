//! Game modes. A mode is plain text in scripts: `survival`, `creative`,
//! `adventure` or `spectator`.

use crate::compile::{CondKind, EffKind};

/// Effect patterns for game modes, tried before the main list.
pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    ("set gamemode of %player% to %text% [mode]", EffKind::SetGamemode),
    ("set game mode of %player% to %text% [mode]", EffKind::SetGamemode),
    ("set %player%'s gamemode to %text% [mode]", EffKind::SetGamemode),
    ("set %player%'s game mode to %text% [mode]", EffKind::SetGamemode),
];

/// Condition patterns for game modes, tried before the main list.
pub const CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    ("%player% is in %text% [mode]", CondKind::InGamemode),
    ("%player% (isn't|is not) in %text% [mode]", CondKind::NotInGamemode),
];

/// The four modes, in vanilla order.
pub const MODES: [&str; 4] = ["survival", "creative", "adventure", "spectator"];

/// Canonical mode name for a word, or `None`. Accepts the numbers 0 to 3 and
/// one-letter short forms (`s`, `c`, `a`, `sp`) like the vanilla command.
pub fn normalize(s: &str) -> Option<&'static str> {
    let t = s.trim().to_ascii_lowercase();
    let t = t.strip_suffix(" mode").unwrap_or(&t);
    match t {
        "survival" | "s" | "0" => Some(MODES[0]),
        "creative" | "c" | "1" => Some(MODES[1]),
        "adventure" | "a" | "2" => Some(MODES[2]),
        "spectator" | "sp" | "3" => Some(MODES[3]),
        _ => None,
    }
}

/// True for the bare words that name a mode in script source.
pub fn is_mode_word(w: &str) -> bool {
    MODES.iter().any(|m| m.eq_ignore_ascii_case(w))
}

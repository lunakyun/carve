//! Sounds: patterns and helpers for `play sound` and `stop sound`.
//!
//! Pumpkin's plugin API has no sound call, so the default host methods run
//! the vanilla `playsound` and `stopsound` console commands.

use crate::compile::EffKind;

/// Registered next to the main effect patterns.
pub const SOUND_PATTERNS: &[(&str, EffKind)] = &[
    (
        "play [the] (sound|sounds) %text% [(at|with) volume %number%] [(and|and at|and with|at|with) pitch %number%] [(to|for) %player%] [at %location%]",
        EffKind::PlaySound,
    ),
    ("stop all sounds [(for|from) %player%]", EffKind::StopAllSounds),
    (
        "stop [the] (sound|sounds) %text% [(for|from) %player%]",
        EffKind::StopSound,
    ),
];

/// A sound id is safe to put in a command: lowercase letters, digits and `_ . : / -`.
/// Returns the cleaned id (trimmed, lowercased).
pub fn clean_id(raw: &str) -> Result<String, String> {
    let id = raw.trim().to_ascii_lowercase();
    let ok = !id.is_empty()
        && id.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | ':' | '/' | '-')
        });
    if ok {
        Ok(id)
    } else {
        Err(format!(
            "'{raw}' is not a valid sound id (try \"entity.experience_orb.pickup\")"
        ))
    }
}

fn num(n: f64) -> String {
    if n.is_finite() { format!("{n}") } else { "0".into() }
}

/// The vanilla command for one player.
pub fn play_command(player: &str, sound: &str, x: f64, y: f64, z: f64, volume: f64, pitch: f64) -> String {
    format!(
        "playsound {sound} master {player} {} {} {} {} {}",
        num(x),
        num(y),
        num(z),
        num(volume.max(0.0)),
        num(pitch.clamp(0.0, 2.0)),
    )
}

pub fn stop_command(player: &str, sound: Option<&str>) -> String {
    match sound {
        Some(s) => format!("stopsound {player} * {s}"),
        None => format!("stopsound {player}"),
    }
}

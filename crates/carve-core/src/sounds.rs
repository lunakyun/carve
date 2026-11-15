//! Sounds: patterns and helpers for `play sound` and `stop sound`.
//!
//! The plugin sends the sound packet itself. Hosts with no direct call (and
//! the tests) fall back on the vanilla `playsound` and `stopsound` commands,
//! which the default host methods build with [`play_command`] and [`stop_command`].

use std::sync::OnceLock;

use crate::compile::EffKind;

/// Every vanilla sound id (`entity.experience_orb.pickup`), sorted, from Pumpkin's data.
static VANILLA: &str = include_str!("sound_ids.txt");

fn vanilla_ids() -> &'static [&'static str] {
    static IDS: OnceLock<Vec<&'static str>> = OnceLock::new();
    IDS.get_or_init(|| VANILLA.lines().collect())
}

/// What a cleaned sound id names.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SoundKind {
    /// A vanilla sound, with or without the `minecraft:` prefix.
    Vanilla,
    /// A sound from another namespace (a resource pack). The server can't check it.
    Custom,
    /// No namespace or `minecraft:`, but not a vanilla sound: the client plays nothing.
    Unknown,
}

/// Sort a cleaned sound id (see [`clean_id`]) into vanilla, resource pack or unknown.
pub fn classify(id: &str) -> SoundKind {
    let name = match id.split_once(':') {
        Some((ns, _)) if ns != "minecraft" => return SoundKind::Custom,
        Some((_, name)) => name,
        None => id,
    };
    if vanilla_ids().binary_search(&name).is_ok() {
        SoundKind::Vanilla
    } else {
        SoundKind::Unknown
    }
}

/// The id with its namespace: `minecraft:` is added when there is none.
pub fn namespaced(id: &str) -> String {
    if id.contains(':') {
        id.to_string()
    } else {
        format!("minecraft:{id}")
    }
}

/// Volume and pitch the way the vanilla command takes them: no negative
/// volume, pitch from 0 to 2.
pub fn clamp_levels(volume: f64, pitch: f64) -> (f32, f32) {
    let v = if volume.is_finite() { volume.max(0.0) } else { 0.0 };
    let p = if pitch.is_finite() {
        pitch.clamp(0.0, 2.0)
    } else {
        0.0
    };
    (v as f32, p as f32)
}

/// Registered next to the main effect patterns.
pub const SOUND_PATTERNS: &[(&str, EffKind)] = &[
    (
        "play [the] (sound|sounds) %text% [(at|with) volume %number%] [(and|and at|and with|at|with) pitch %number%] at %location% (to|for) %object%",
        EffKind::PlaySoundAtTo,
    ),
    (
        "play [the] (sound|sounds) %text% [(at|with) volume %number%] [(and|and at|and with|at|with) pitch %number%] [(to|for) %object%] [at %location%]",
        EffKind::PlaySound,
    ),
    ("stop all sounds [(for|from) %player%]", EffKind::StopAllSounds),
    (
        "stop [the] (sound|sounds) %text% [(for|from) %player%]",
        EffKind::StopSound,
    ),
];

/// Bukkit enum names (`ENTITY_TNT_PRIMED`, `BLOCK_NOTE_BLOCK_PLING`) as
/// Skript accepts them: the vanilla id that has the same letters with `.` as `_`.
fn from_enum_name(id: String) -> String {
    if id.contains('.') || id.contains(':') || !id.contains('_') {
        return id;
    }
    let id = id.strip_prefix("minecraft_").unwrap_or(&id).to_string();
    match vanilla_ids().iter().find(|v| v.replace('.', "_") == id) {
        Some(v) => (*v).to_string(),
        None => id,
    }
}

/// A sound id is safe to put in a command: lowercase letters, digits and `_ . : / -`.
/// Returns the cleaned id (trimmed, lowercased).
pub fn clean_id(raw: &str) -> Result<String, String> {
    let id = raw.trim().to_ascii_lowercase();
    let ok = !id.is_empty()
        && id.chars().all(|c| {
            c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, '_' | '.' | ':' | '/' | '-')
        });
    if ok {
        Ok(from_enum_name(id))
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

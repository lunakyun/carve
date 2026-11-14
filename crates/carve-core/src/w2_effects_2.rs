//! Player effects: `make player fly`, `make player invulnerable`, `ignite`,
//! `knock`, `make player look at`, `make player say`, `open book`, `log`.
//!
//! One effect variant carries all of them (`Effect::W2`), so the hot enums
//! stay small. The engine runs them from its rare path.

use crate::compile::{EffKind, Effect};
use crate::expr::Expr;
use crate::pattern::Capture;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum W2Op {
    /// `make p fly` / `make p stop flying`
    Fly(bool),
    /// `make p invulnerable` / `vulnerable`
    Invulnerable(bool),
    /// `make p invisible` / `visible`
    Invisible(bool),
    /// `ignite p [for 5 seconds]`; args: player, span
    Ignite,
    Extinguish,
    /// `knock p away from loc [with strength n]`; args: player, location, strength
    KnockAway,
    /// `knock p toward loc [with strength n]`
    KnockToward,
    /// `make p look at loc`; args: player, location
    LookAt,
    /// `make p say text`; args: player, text
    Say,
    /// `open book (in main|off hand) to p`; true is the off hand
    OpenBook(bool),
    /// `log text [to file name]`; args: text, file
    Log,
    /// `locally suppress ... warnings`: accepted and ignored
    Suppress,
}

#[derive(Debug, Clone, PartialEq)]
pub struct W2Effect {
    pub op: W2Op,
    pub args: Vec<Option<Expr>>,
}

use EffKind::W2;
use W2Op::*;

/// Registered after the main effect patterns. Each starts with a literal.
pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    ("make %player% (fly|start flying|be able to fly)", W2(Fly(true))),
    ("make %player% (stop flying|not fly|stop fly|unable to fly)", W2(Fly(false))),
    ("make %player% invulnerable", W2(Invulnerable(true))),
    ("make %player% (vulnerable|not invulnerable)", W2(Invulnerable(false))),
    ("make %player% invisible", W2(Invisible(true))),
    ("make %player% (visible|not invisible)", W2(Invisible(false))),
    ("ignite %player% [for %timespan%]", W2(Ignite)),
    ("set %player% on fire [for %timespan%]", W2(Ignite)),
    ("(extinguish|put out) %player%", W2(Extinguish)),
    (
        "knock %player% (away from|away) %location% [with (strength|force) %number%]",
        W2(KnockAway),
    ),
    (
        "knock %player% (toward|towards|to) %location% [with (strength|force) %number%]",
        W2(KnockToward),
    ),
    ("make %player% (look|face) at %location%", W2(LookAt)),
    ("make %player% say %text%", W2(Say)),
    ("open [the] book [in [the] main hand] to %player%", W2(OpenBook(false))),
    ("open [the] book in [the] off hand to %player%", W2(OpenBook(true))),
    ("log %text% to [file] %text%", W2(Log)),
    ("log %text%", W2(Log)),
    (
        "[locally] suppress (conflict|conjunction|missing conjunction|variable save|variable saved|deprecation|deprecated syntax|unreachable code|empty script) (warning|warnings)",
        W2(Suppress),
    ),
];

/// Build the effect from the captures, which come in slot order.
pub fn build(op: W2Op, caps: &mut [Option<Capture>]) -> Option<Effect> {
    let args = caps
        .iter_mut()
        .map(|c| match c.take() {
            Some(Capture::Expr(e)) => Some(e),
            _ => None,
        })
        .collect();
    Some(Effect::W2(Box::new(W2Effect { op, args })))
}

/// Fire ticks for `ignite` with no time given (Skript uses 5 seconds).
pub const DEFAULT_FIRE_TICKS: i32 = 100;

/// Yaw and pitch (degrees) for looking from one point at another, using
/// Minecraft's convention: yaw 0 faces south (+z), pitch positive looks down.
pub fn look_angles(from: (f64, f64, f64), to: (f64, f64, f64)) -> (f32, f32) {
    let (dx, dy, dz) = (to.0 - from.0, to.1 - from.1, to.2 - from.2);
    let horiz = dx.hypot(dz);
    let yaw = (-dx).atan2(dz).to_degrees();
    let pitch = (-dy).atan2(horiz).to_degrees();
    ((yaw + 0.0) as f32, (pitch + 0.0) as f32)
}

/// Clamp a knockback strength to something sane.
pub fn clean_strength(s: Option<f64>) -> f64 {
    match s {
        Some(v) if v.is_finite() => v.clamp(0.0, 20.0),
        Some(_) => 0.0,
        None => 0.4,
    }
}

/// A log file name is safe when it is a plain file name: no folders, no `..`.
pub fn clean_log_name(raw: &str) -> Result<String, String> {
    let name = raw.trim();
    let ok = !name.is_empty()
        && name.len() <= 100
        && !name.starts_with('.')
        && name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | ' '));
    if !ok {
        return Err(format!(
            "'{raw}' is not a valid log file name (use letters, digits, - _ . and spaces, no folders)"
        ));
    }
    Ok(if name.contains('.') { name.to_string() } else { format!("{name}.log") })
}

/// `log "x"` with no file goes to this file.
pub const DEFAULT_LOG_FILE: &str = "server.log";

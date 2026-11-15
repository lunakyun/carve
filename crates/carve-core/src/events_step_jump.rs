//! Helpers for the events added to `events_world_vehicles` (step on,
//! chunk enter, jump, world border changes, armor change and more).
//!
//! Everything here is plain logic with no game calls, so it is unit tested
//! in `tests/events_step_jump.rs`. The game side lives in `carve-wasm/src/events_step_jump.rs`.

use crate::expr::{Expr, Part};

/// `step on stone` -> (`step on`, Some(("stepon", "stone"))). Other headers
/// come back unchanged.
pub fn split_step_on(ev: &str) -> (&str, Option<(&'static str, String)>) {
    for base in ["step on", "walk on", "move on"] {
        if let Some(rest) = ev.strip_prefix(base)
            && rest.starts_with(' ')
            && !rest.trim().is_empty()
        {
            return (base, Some(("stepon", rest.trim().to_string())));
        }
    }
    (ev, None)
}

/// The block id a `step on <block>` header compares with: `Oak Planks` ->
/// `oak_planks`. `None` for text that cannot be a block name.
pub fn step_rhs(text: &str) -> Option<Expr> {
    let t = text.trim().trim_matches('"').trim().to_ascii_lowercase();
    let t = t
        .strip_prefix("a ")
        .or_else(|| t.strip_prefix("an "))
        .unwrap_or(&t);
    let t = t.strip_prefix("minecraft:").unwrap_or(t).replace(' ', "_");
    if t.is_empty()
        || !t
            .chars()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_')
    {
        return None;
    }
    Some(Expr::Text(vec![Part::Lit(t)]))
}

/// The chunk a block position is in (`floor(x / 16)`).
pub fn chunk_of(v: f64) -> i32 {
    (v / 16.0).floor() as i32
}

/// Whether moving between these two points crosses into another chunk.
pub fn chunk_changed(from: (f64, f64), to: (f64, f64)) -> bool {
    chunk_of(from.0) != chunk_of(to.0) || chunk_of(from.1) != chunk_of(to.1)
}

/// The jump bit of a `player input` bitmask (forward 1, backward 2, left 4,
/// right 8, jump 16, sneak 32, sprint 64).
pub const JUMP_BIT: u32 = 16;

/// A rising edge of the jump bit: it is set now and was not set before.
pub fn jump_started(before: u32, now: u32) -> bool {
    now & JUMP_BIT != 0 && before & JUMP_BIT == 0
}

/// Whether a redstone change on a pressure plate is a press (power rises from
/// zero).
pub fn plate_pressed(old: i32, new: i32) -> bool {
    old == 0 && new > 0
}

/// Whether an entity type is one of the mobs that break doors like a zombie.
pub fn is_zombie_kind(kind: &str) -> bool {
    matches!(
        kind.strip_prefix("minecraft:").unwrap_or(kind),
        "zombie" | "husk" | "drowned" | "zombie_villager"
    )
}

/// `set event-boolean to false` in `on can build check`: whether the player
/// can build. The host reads value slot 0 of the outcome back.
pub fn parse_can_build_set(
    toks: &[crate::lexer::Tok],
    env: &crate::expr::Env,
) -> Option<Result<crate::compile::Effect, String>> {
    use crate::lexer::Tok;
    if !env.ev_fields.iter().any(|(n, _)| *n == "event-existing-block") {
        return None;
    }
    if !toks.first()?.is_lit("set") {
        return None;
    }
    let Tok::Word(w) = toks.get(1)? else { return None };
    if !matches!(
        w.to_ascii_lowercase().as_str(),
        "event-boolean" | "event-buildable"
    ) || !toks.get(2)?.is_lit("to")
    {
        return None;
    }
    let v = match crate::expr::parse_expr(toks.get(3..)?, env) {
        Ok(e) => e,
        Err(e) => return Some(Err(e)),
    };
    Some(Ok(crate::compile::Effect::WorldVehicles(Box::new(
        crate::events_world_vehicles::ExtEffect::SetVal(0, v),
    ))))
}

/// Whether a block id is a pressure plate.
pub fn is_pressure_plate(block: &str) -> bool {
    block
        .strip_prefix("minecraft:")
        .unwrap_or(block)
        .ends_with("_pressure_plate")
}

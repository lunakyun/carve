//! Combat values of `on damage` and `on death` that go beyond players: mob
//! victims and attackers, and the settable `damage` and `death message`.
//!
//! The entity behind a victim or attacker is often never used by a script,
//! so the host passes its numeric id and it is looked up on first use.

use crate::compile::Effect;
use crate::engine::{Engine, EventCtx, Exec, online};
use crate::expr::{Env, Expr, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::{EntityRef, Value};

/// `FireOutcome::set` slot of a changed `damage` (a number).
pub const SLOT_DAMAGE: u8 = 250;
/// `FireOutcome::set` slot of a changed `death message` (a text).
pub const SLOT_DEATH_MESSAGE: u8 = 251;

/// An entity of the event that is not a player.
#[derive(Debug, Clone, PartialEq)]
pub enum EntSlot {
    /// The server's entity id, not looked up yet.
    Id(i32),
    Ent(EntityRef),
    /// Looked up and not found.
    Gone,
}

/// The non-player victim and attacker of a hit.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Combat {
    pub victim: Option<EntSlot>,
    pub attacker: Option<EntSlot>,
}

impl EventCtx {
    /// A hit on an entity that is not a player.
    pub fn entity_victim(victim: EntSlot) -> Self {
        let mut c = Self::console();
        c.combat = Some(Box::new(Combat {
            victim: Some(victim),
            attacker: None,
        }));
        c
    }

    /// Set a non-player attacker.
    pub fn set_entity_attacker(&mut self, attacker: EntSlot) {
        self.combat.get_or_insert_with(Default::default).attacker = Some(attacker);
    }
}

fn resolve(slot: &mut EntSlot, host: &mut dyn Host) -> Value {
    if let EntSlot::Id(id) = *slot {
        *slot = host.entity_by_id(id).map_or(EntSlot::Gone, EntSlot::Ent);
    }
    match slot {
        EntSlot::Ent(e) => Value::Entity(e.clone()),
        _ => Value::None,
    }
}

/// `attacker` when it is not a player.
pub(crate) fn attacker(ctx: &mut EventCtx, host: &mut dyn Host) -> Value {
    match ctx.combat.as_mut().and_then(|c| c.attacker.as_mut()) {
        Some(s) => resolve(s, host),
        None => Value::None,
    }
}

/// `victim` in `on damage`: the player, else the hurt entity.
pub(crate) fn victim(ex: &mut Exec, host: &mut dyn Host) -> Value {
    if let Some(p) = ex.ctx.player.clone() {
        return online(&Value::Player(p), &mut ex.seen, host).map_or(Value::None, Value::Player);
    }
    match ex.ctx.combat.as_mut().and_then(|c| c.victim.as_mut()) {
        Some(s) => resolve(s, host),
        None => Value::None,
    }
}

/// Hand changes of one trigger to the triggers that run after it.
pub(crate) fn carry(ctx: &mut EventCtx, out: &[(u8, Value)]) {
    for (slot, v) in out {
        match (*slot, v) {
            (SLOT_DAMAGE, Value::Num(n)) => ctx.damage = Some(*n),
            (SLOT_DEATH_MESSAGE, Value::Text(t)) => ctx.death_message = Some(t.clone()),
            _ => {}
        }
    }
}

// ---------- effects ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Set,
    Add,
    Remove,
}

#[derive(Debug, Clone, PartialEq)]
pub enum CombatFx {
    Damage(Mode, Expr),
    DeathMessage(Expr),
}

fn is(t: Option<&Tok>, w: &str) -> bool {
    t.is_some_and(|t| t.is_lit(w))
}

/// Skip `the`, then expect `damage` / `death message`. Returns tokens used.
fn target(toks: &[Tok], death: bool) -> Option<usize> {
    let s = usize::from(is(toks.first(), "the"));
    if death {
        (is(toks.get(s), "death") && is(toks.get(s + 1), "message")).then_some(s + 2)
    } else {
        is(toks.get(s), "damage").then_some(s + 1)
    }
}

fn pos(toks: &[Tok], w: &str) -> Option<usize> {
    toks.iter().position(|t| t.is_lit(w))
}

fn value(toks: &[Tok], env: &Env, want: Ty) -> Result<Expr, String> {
    if toks.is_empty() {
        return Err("missing a value".into());
    }
    let e = parse_expr(toks, env)?;
    let t = e.ty(env);
    if !want.accepts(t) {
        return Err(format!("expected a {}, got a {}", want.name(), t.name()));
    }
    Ok(e)
}

/// `set damage to 5`, `add 2 to damage`, `remove 1 from the damage`,
/// `set death message to "..."`. `None`: not ours.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Option<Result<Effect, String>> {
    if env.in_function || !env.has_attacker {
        return None;
    }
    let first = toks.first()?;
    let damage = env.has_damage;
    let fx = if first.is_lit("set") {
        let rest = toks.get(1..)?;
        let n = target(rest, !damage)?;
        if !is(rest.get(n), "to") {
            return None;
        }
        let v = rest.get(n + 1..)?;
        if damage {
            value(v, env, Ty::Number).map(|e| CombatFx::Damage(Mode::Set, e))
        } else {
            value(v, env, Ty::Text).map(CombatFx::DeathMessage)
        }
    } else if damage && (first.is_lit("add") || first.is_lit("remove")) {
        let (mode, word) = if first.is_lit("add") {
            (Mode::Add, "to")
        } else {
            (Mode::Remove, "from")
        };
        let at = pos(toks, word)?;
        let rest = toks.get(at + 1..)?;
        if target(rest, false)? != rest.len() {
            return None;
        }
        value(toks.get(1..at)?, env, Ty::Number).map(|e| CombatFx::Damage(mode, e))
    } else {
        return None;
    };
    Some(fx.map(|f| Effect::Combat(Box::new(f))))
}

impl Engine {
    pub(crate) fn combat_effect(
        &mut self,
        f: &CombatFx,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        let (slot, v) = match f {
            CombatFx::Damage(mode, e) => {
                let n = self.num(e, ex, host)?;
                let old = ex.ctx.damage.unwrap_or(0.0);
                let new = match mode {
                    Mode::Set => n,
                    Mode::Add => old + n,
                    Mode::Remove => old - n,
                };
                let new = if new.is_finite() { new.max(0.0) } else { old };
                ex.ctx.damage = Some(new);
                (SLOT_DAMAGE, Value::Num(new))
            }
            CombatFx::DeathMessage(e) => {
                let t = self.eval(e, ex, host)?.to_string();
                ex.ctx.death_message = Some(t.clone());
                (SLOT_DEATH_MESSAGE, Value::Text(t))
            }
        };
        // Like `cancel event`: only a change before the first wait reaches the server.
        if !ex.waited {
            self.ext_out.push((slot, v));
        }
        Ok(())
    }
}

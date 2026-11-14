//! World effects: lightning, explosions, particles, potion effects and fire.
//!
//! These lines have free-form words in them (`flame particles`, `speed 2`),
//! so they are parsed by hand here instead of by the pattern registry.
//! `parse_effect` and `parse_cond` are called from the compiler before the
//! registry is tried; the `Engine` methods at the bottom run them.

use crate::compile::{Cond, Effect};
use crate::engine::{Engine, Exec, online};
use crate::expr::{Env, Expr, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::{PlayerRef, Value};

pub use crate::fx_names::{EFFECTS, PARTICLES};
pub use crate::skbee_particles_display::{ParticleData, ParticleOpts};

/// A point in a world. `world: None` means the default world.
#[derive(Debug, Clone, PartialEq)]
pub struct FxPos {
    pub x: f64,
    pub y: f64,
    pub z: f64,
    pub world: Option<String>,
}

/// One particle burst, already checked and clamped.
#[derive(Debug, Clone, PartialEq)]
pub struct ParticleFx {
    pub name: String,
    pub at: FxPos,
    pub count: u32,
    /// Random spread on each axis.
    pub spread: f64,
    pub speed: f64,
    /// A different spread on each axis; wins over `spread`.
    pub offset: Option<[f64; 3]>,
    /// Show to players far away too.
    pub force: bool,
    pub data: ParticleData,
}

impl ParticleFx {
    /// The spread on each axis.
    pub fn offsets(&self) -> [f64; 3] {
        self.offset.unwrap_or([self.spread; 3])
    }
}

/// A duration: an amount and how many ticks one unit is.
type Dur = (Expr, u64);

#[derive(Debug, Clone, PartialEq)]
pub enum WorldFx {
    Lightning {
        at: Expr,
        effect_only: bool,
    },
    Explosion {
        at: Expr,
        power: Option<Expr>,
        fire: bool,
    },
    Particles {
        name: String,
        count: Option<Expr>,
        at: Expr,
        spread: Option<Expr>,
        speed: Option<Expr>,
        /// Extra options (force, per-axis offset, color, block, item).
        opts: Option<Box<ParticleOpts>>,
    },
    /// `spawn text display at <place> with ...`.
    Display(crate::skbee_particles_display::DisplayFx),
    Apply {
        effect: String,
        tier: Option<Expr>,
        to: Expr,
        dur: Option<Dur>,
    },
    ClearEffects {
        from: Expr,
        effect: Option<String>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct WorldCond {
    pub player: Expr,
    pub effect: String,
    pub negate: bool,
}

const DEFAULT_POTION_TICKS: f64 = 15.0 * 20.0;
const DEFAULT_POWER: f64 = 3.0;
const MAX_PARTICLES: f64 = 1000.0;
const MAX_POWER: f64 = 100.0;
const MAX_TICKS: f64 = 20_000_000.0;

// ---------- parsing ----------

fn is(t: &Tok, lit: &str) -> bool {
    t.is_lit(lit)
}

fn find(toks: &[Tok], from: usize, lits: &[&str]) -> Option<usize> {
    toks.iter()
        .enumerate()
        .skip(from)
        .find(|(_, t)| lits.iter().any(|l| is(t, l)))
        .map(|(i, _)| i)
}

fn expr_of(toks: &[Tok], env: &Env, want: &[Ty]) -> Option<Expr> {
    let e = parse_expr(toks, env).ok()?;
    let got = e.ty(env);
    (got == Ty::Object || want.contains(&got)).then_some(e)
}

fn number(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Number])
}

/// A place: a location, or a player (their position).
fn place(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Location, Ty::Player])
}

fn player(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Player])
}

/// Words joined with `_`, lowercase. `None` when a token is not a plain word.
fn name_of(toks: &[Tok]) -> Option<String> {
    let mut out = String::new();
    for t in toks {
        let Tok::Word(w) = t else {
            return None;
        };
        if !out.is_empty() {
            out.push('_');
        }
        out.push_str(&w.to_ascii_lowercase());
    }
    (!out.is_empty()).then_some(out)
}

/// `speed`, `fire resistance`, `regen` to the vanilla id.
pub fn effect_id(words: &str) -> Option<&'static str> {
    let alias = match words {
        "regen" => "regeneration",
        "jump" => "jump_boost",
        "instant_heal" | "healing" => "instant_health",
        "harming" => "instant_damage",
        "slow" => "slowness",
        "fire_res" => "fire_resistance",
        other => other,
    };
    EFFECTS.iter().copied().find(|e| *e == alias)
}

fn particle_id(words: &str) -> Option<&'static str> {
    PARTICLES.iter().copied().find(|p| *p == words)
}

/// `for 10 seconds`: the tokens after `for`.
fn duration(toks: &[Tok], env: &Env) -> Option<Dur> {
    let (unit, amount) = toks.split_last()?;
    let Tok::Word(u) = unit else {
        return None;
    };
    let per = match u.to_ascii_lowercase().as_str() {
        "tick" | "ticks" => 1,
        "second" | "seconds" => 20,
        "minute" | "minutes" => 1200,
        "hour" | "hours" => 72_000,
        _ => return None,
    };
    Some((number(amount, env)?, per))
}

/// Split `... [for <duration>]` at the last top-level `for`.
fn split_for<'a>(toks: &'a [Tok], env: &Env) -> Option<(&'a [Tok], Option<Dur>)> {
    match toks.iter().rposition(|t| is(t, "for")) {
        Some(i) => Some((toks.get(..i)?, Some(duration(toks.get(i + 1..)?, env)?))),
        None => Some((toks, None)),
    }
}

/// `Ok(None)`: not a world effect line. `Err`: it is one, but wrong.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    let first = first.to_ascii_lowercase();
    let fx = match first.as_str() {
        "strike" => lightning(toks, env),
        "show" | "play" | "create" | "make" | "spawn" | "draw" | "summon" => {
            if toks.iter().any(|t| is(t, "display")) {
                crate::skbee_particles_display::parse_display(toks, env)?.map(WorldFx::Display)
            } else if toks.iter().any(|t| is(t, "particle") || is(t, "particles")) {
                particles(toks, env)?
            } else if toks.iter().any(|t| is(t, "explosion")) {
                explosion(toks, env)?
            } else {
                lightning(toks, env)
            }
        }
        "apply" => apply(toks, env)?,
        "remove" | "clear" => clear(toks, env)?,
        // `ignite`, `extinguish`, `set <player> on fire` are `w2_effects_2`'s.
        _ => None,
    };
    Ok(fx.map(|f| Effect::World(Box::new(f))))
}

/// `strike lightning [effect] at <place>`, `show a lightning effect at <place>`.
fn lightning(toks: &[Tok], env: &Env) -> Option<WorldFx> {
    let l = find(toks, 0, &["lightning"])?;
    // Only filler words may come before `lightning`.
    let before = toks.get(..l)?;
    let filler = |t: &Tok| {
        [
            "strike", "show", "play", "create", "make", "spawn", "summon", "a", "the",
        ]
        .iter()
        .any(|w| is(t, w))
    };
    if !before.iter().all(filler) {
        return None;
    }
    let mut i = l + 1;
    let mut effect_only = false;
    match toks.get(i) {
        Some(t) if is(t, "effect") => {
            effect_only = true;
            i += 1;
        }
        Some(t) if is(t, "bolt") || is(t, "strike") => i += 1,
        _ => {}
    }
    if !toks.get(i).is_some_and(|t| is(t, "at") || is(t, "on")) {
        return None;
    }
    let at = place(toks.get(i + 1..)?, env)?;
    Some(WorldFx::Lightning { at, effect_only })
}

/// `create [an] explosion [of force N] [with fire] at <place>`.
fn explosion(toks: &[Tok], env: &Env) -> Result<Option<WorldFx>, String> {
    let Some(e) = find(toks, 0, &["explosion"]) else {
        return Ok(None);
    };
    let before_ok = toks.get(..e).is_some_and(|b| {
        b.iter().all(|t| {
            ["create", "make", "spawn", "summon", "a", "an", "the"]
                .iter()
                .any(|w| is(t, w))
        })
    });
    let Some(at_i) = find(toks, e, &["at", "on"]) else {
        return Ok(None);
    };
    if !before_ok {
        return Ok(None);
    }
    let Some(at) = toks.get(at_i + 1..).and_then(|t| place(t, env)) else {
        return Ok(None);
    };
    let mut mid = toks.get(e + 1..at_i).unwrap_or_default();
    let mut fire = false;
    if let [rest @ .., w, f] = mid
        && is(w, "with")
        && is(f, "fire")
    {
        fire = true;
        mid = rest;
    }
    let power = match mid {
        [] => None,
        [a, b, p @ ..]
            if (is(a, "of") || is(a, "with"))
                && ["force", "power", "strength", "size"].iter().any(|w| is(b, w)) =>
        {
            Some(number(p, env).ok_or("explosion force must be a number")?)
        }
        _ => return Ok(None),
    };
    Ok(Some(WorldFx::Explosion { at, power, fire }))
}

/// `play 5 flame particles at <place> [with spread N] [with speed N]`.
fn particles(toks: &[Tok], env: &Env) -> Result<Option<WorldFx>, String> {
    let Some(at_i) = find(toks, 1, &["at", "on"]) else {
        return Ok(None);
    };
    let head = toks.get(1..at_i).unwrap_or_default();
    let Some((last, words)) = head.split_last() else {
        return Ok(None);
    };
    if !(is(last, "particle") || is(last, "particles")) {
        return Ok(None);
    }
    let (count, name_toks) = match words.split_first() {
        Some((c @ (Tok::Num(_) | Tok::Var(_)), rest)) => (number(std::slice::from_ref(c), env), rest),
        _ => (None, words),
    };
    let Some(name) = name_of(name_toks) else {
        return Ok(None);
    };
    let Some(name) = particle_id(&name) else {
        return Err(format!("unknown particle '{}'", name.replace('_', " ")));
    };
    let tail = toks.get(at_i + 1..).unwrap_or_default();
    let mut parts = tail.split(|t| is(t, "with"));
    let Some(at) = parts.next().and_then(|t| place(t, env)) else {
        return Ok(None);
    };
    let (mut spread, mut speed) = (None, None);
    let mut opts = ParticleOpts::default();
    for seg in parts {
        let seg = match seg.split_last() {
            Some((a, v)) if is(a, "and") => v,
            _ => seg,
        };
        let Some((kind, val)) = seg.split_first() else {
            return Ok(None);
        };
        let one = || number(val, env);
        if (is(kind, "spread") || is(kind, "offset")) && let Some(n) = one() {
            spread = Some(n);
        } else if (is(kind, "speed") || is(kind, "extra")) && let Some(n) = one() {
            speed = Some(n);
        } else if !crate::skbee_particles_display::particle_opt(seg, env, &mut opts)? {
            return Ok(None);
        }
    }
    let opts = (opts != ParticleOpts::default()).then(|| Box::new(opts));
    Ok(Some(WorldFx::Particles {
        name: name.to_string(),
        count,
        at,
        spread,
        speed,
        opts,
    }))
}

/// `apply [potion of] speed [2] to <player> [for 10 seconds]`.
fn apply(toks: &[Tok], env: &Env) -> Result<Option<WorldFx>, String> {
    let mut i = 1;
    for w in ["potion", "effect", "of"] {
        if toks.get(i).is_some_and(|t| is(t, w)) {
            i += 1;
        }
    }
    let Some(to_i) = find(toks, i, &["to"]) else {
        return Ok(None);
    };
    let mut head = toks.get(i..to_i).unwrap_or_default();
    // `of tier N` or a trailing number is the tier.
    let mut tier = None;
    if let Some(p) = head.iter().position(|t| is(t, "of")) {
        let t = head.get(p + 1..).unwrap_or_default();
        let t = match t.split_first() {
            Some((a, rest)) if is(a, "tier") => rest,
            _ => t,
        };
        tier = Some(number(t, env).ok_or("potion tier must be a number")?);
        head = head.get(..p).unwrap_or_default();
    } else if let Some((last @ (Tok::Num(_) | Tok::Var(_)), rest)) = head.split_last() {
        tier = number(std::slice::from_ref(last), env);
        head = rest;
    }
    let Some(name) = name_of(head) else {
        return Ok(None);
    };
    let Some(effect) = effect_id(&name) else {
        return Err(format!("unknown potion effect '{}'", name.replace('_', " ")));
    };
    let Some((target, dur)) = split_for(toks.get(to_i + 1..).unwrap_or_default(), env) else {
        return Ok(None);
    };
    let Some(to) = player(target, env) else {
        return Ok(None);
    };
    Ok(Some(WorldFx::Apply {
        effect: effect.to_string(),
        tier,
        to,
        dur,
    }))
}

/// `remove all potion effects from <player>`, `remove speed potion effect from <player>`.
fn clear(toks: &[Tok], env: &Env) -> Result<Option<WorldFx>, String> {
    if !toks.iter().any(|t| is(t, "potion")) {
        return Ok(None);
    }
    let Some(from_i) = find(toks, 1, &["from", "of"]) else {
        return Ok(None);
    };
    let Some(from) = toks.get(from_i + 1..).and_then(|t| player(t, env)) else {
        return Ok(None);
    };
    let words: Vec<Tok> = toks
        .get(1..from_i)
        .unwrap_or_default()
        .iter()
        .filter(|t| {
            !["potion", "effect", "effects", "the", "all", "active"]
                .iter()
                .any(|w| is(t, w))
        })
        .cloned()
        .collect();
    let effect = if words.is_empty() {
        None
    } else {
        let Some(name) = name_of(&words) else {
            return Ok(None);
        };
        let Some(id) = effect_id(&name) else {
            return Err(format!("unknown potion effect '{}'", name.replace('_', " ")));
        };
        Some(id.to_string())
    };
    Ok(Some(WorldFx::ClearEffects { from, effect }))
}

/// `<player> has [the] [potion] effect speed`, `... doesn't have potion effect speed`.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    let eff = toks.iter().position(|t| is(t, "effect"))?;
    let h = find(toks, 0, &["have", "has"])?;
    if h > eff {
        return None;
    }
    let before = toks.get(..h)?;
    let (subject, negate) = match before.split_last() {
        Some((n, s)) if is(n, "not") => match s.split_last() {
            Some((d, s)) if is(d, "does") || is(d, "do") => (s, true),
            _ => return None,
        },
        Some((n, s)) if is(n, "doesn't") || is(n, "don't") => (s, true),
        _ => (before, false),
    };
    // Only `[the] [potion] effect` may sit between `has` and the name.
    let between = toks.get(h + 1..eff)?;
    if !between
        .iter()
        .all(|t| is(t, "the") || is(t, "potion") || is(t, "a"))
    {
        return None;
    }
    let name = name_of(toks.get(eff + 1..)?)?;
    let effect = effect_id(&name)?.to_string();
    Some(Cond::World(Box::new(WorldCond {
        player: player(subject, env)?,
        effect,
        negate,
    })))
}

// ---------- running ----------

fn ticks_of(
    dur: &Option<Dur>,
    default: f64,
    engine: &mut Engine,
    ex: &mut Exec,
    host: &mut dyn Host,
) -> Result<u32, String> {
    let t = match dur {
        Some((e, per)) => engine.num(e, ex, host)? * *per as f64,
        None => default,
    };
    Ok(if t.is_finite() {
        t.clamp(1.0, MAX_TICKS) as u32
    } else {
        1
    })
}

impl Engine {
    /// The position a `place` expression stands for.
    pub(crate) fn fx_pos(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<Option<FxPos>, String> {
        let v = self.eval(e, ex, host)?;
        match v {
            Value::Location { x, y, z, world } => Ok(Some(FxPos { x, y, z, world })),
            Value::Player(_) | Value::Text(_) => {
                let Some(p) = online(&v, &mut ex.seen, host) else {
                    return Ok(None);
                };
                Ok(host.player_location(&p).map(|(x, y, z, w)| FxPos {
                    x,
                    y,
                    z,
                    world: Some(w),
                }))
            }
            Value::None => Ok(None),
            other => Err(format!(
                "expected a location or player, got {} ({other})",
                other.type_name()
            )),
        }
    }

    fn fx_player(
        &mut self,
        e: &Expr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Option<PlayerRef>, String> {
        self.player(e, ex, host)
    }

    fn fx_num(
        &mut self,
        e: &Option<Expr>,
        default: f64,
        max: f64,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<f64, String> {
        match e {
            Some(e) => {
                let n = self.num(e, ex, host)?;
                Ok(if n.is_finite() { n.clamp(0.0, max) } else { default })
            }
            None => Ok(default),
        }
    }

    #[inline(never)]
    pub(crate) fn world_effect(
        &mut self,
        w: &WorldFx,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        match w {
            WorldFx::Lightning { at, effect_only } => {
                if let Some(p) = self.fx_pos(at, ex, host)? {
                    host.strike_lightning(&p, *effect_only);
                }
            }
            WorldFx::Explosion { at, power, fire } => {
                let power = self.fx_num(power, DEFAULT_POWER, MAX_POWER, ex, host)?;
                if let Some(p) = self.fx_pos(at, ex, host)? {
                    host.explode(&p, power, *fire);
                }
            }
            WorldFx::Particles {
                name,
                count,
                at,
                spread,
                speed,
                opts,
            } => {
                let count = self.fx_num(count, 1.0, MAX_PARTICLES, ex, host)? as u32;
                let spread = self.fx_num(spread, 0.0, 64.0, ex, host)?;
                let speed = self.fx_num(speed, 0.0, 64.0, ex, host)?;
                let (mut offset, mut force, mut data) = (None, false, ParticleData::None);
                if let Some(o) = opts {
                    offset = self.particle_offset(o, ex, host)?;
                    force = o.force;
                    data = self.particle_data(name, o, ex, host)?;
                } else if crate::skbee_particles_display::needs_data(name) {
                    data = self.particle_data(name, &ParticleOpts::default(), ex, host)?;
                }
                if crate::skbee_particles_display::needs_data(name) && !host.supports_particle_data() {
                    crate::skbee_particles_display::warn_no_data(host, name);
                    return Ok(());
                }
                if let Some(at) = self.fx_pos(at, ex, host)? {
                    host.spawn_particle(&ParticleFx {
                        name: name.clone(),
                        at,
                        count,
                        spread,
                        speed,
                        offset,
                        force,
                        data,
                    });
                }
            }
            WorldFx::Display(d) => self.display_effect(d, ex, host)?,
            WorldFx::Apply {
                effect,
                tier,
                to,
                dur,
            } => {
                let tier = self.fx_num(tier, 1.0, 256.0, ex, host)?.max(1.0);
                let ticks = ticks_of(dur, DEFAULT_POTION_TICKS, self, ex, host)?;
                if let Some(p) = self.fx_player(to, ex, host)? {
                    host.add_effect(&p, effect, ticks, (tier as u32 - 1).min(255) as u8);
                }
            }
            WorldFx::ClearEffects { from, effect } => {
                if let Some(p) = self.fx_player(from, ex, host)? {
                    host.clear_effects(&p, effect.as_deref());
                }
            }
        }
        Ok(())
    }

    #[inline(never)]
    pub(crate) fn world_cond(
        &mut self,
        c: &WorldCond,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<bool, String> {
        let Some(p) = self.fx_player(&c.player, ex, host)? else {
            return Ok(false);
        };
        Ok(host.has_effect(&p, &c.effect) != c.negate)
    }
}

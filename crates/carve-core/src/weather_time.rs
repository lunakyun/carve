//! World time, weather, game rules and difficulty.
//!
//! ```text
//! time in world "world"            weather in world "world"
//! set time to 6:00 pm              set weather to rain for 5 minutes
//! set gamerule keepInventory to true
//! set difficulty to hard
//! if it is raining:                if it is night:
//! ```
//!
//! The expression forms are parsed in `expr.rs`, the effect and condition
//! forms by hand here. The `Engine` methods at the bottom run them. Every
//! game call goes through `Host` methods whose defaults use the vanilla
//! `/time`, `/weather`, `/gamerule` and `/difficulty` commands.

use crate::compile::{Cond, Effect};
use crate::engine::{Engine, Exec, online};
use crate::expr::{Env, Expr, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::Value;

/// Ticks in one Minecraft day.
pub const DAY: i64 = 24_000;
/// Longest weather length we ask the server for, in ticks.
const MAX_WEATHER_TICKS: f64 = 1_000_000.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Weather {
    Clear,
    Rain,
    Thunder,
}

impl Weather {
    pub fn name(self) -> &'static str {
        match self {
            Weather::Clear => "clear",
            Weather::Rain => "rain",
            Weather::Thunder => "thunder",
        }
    }

    /// `rain`, `raining`, `rainy`, `sunny`, `storm`, ... to a weather.
    pub fn parse(word: &str) -> Option<Weather> {
        match word.to_ascii_lowercase().as_str() {
            "clear" | "sun" | "sunny" | "sunshine" | "fair" => Some(Weather::Clear),
            "rain" | "raining" | "rainy" => Some(Weather::Rain),
            "thunder" | "thundering" | "thunderstorm" | "storm" | "stormy" => Some(Weather::Thunder),
            _ => None,
        }
    }

    /// Weather from the two vanilla flags.
    pub fn from_flags(raining: bool, thundering: bool) -> Weather {
        if thundering {
            Weather::Thunder
        } else if raining {
            Weather::Rain
        } else {
            Weather::Clear
        }
    }

    /// The vanilla `/weather` command.
    pub fn command(self, ticks: Option<u32>) -> String {
        match ticks {
            Some(t) => format!("weather {} {t}", self.name()),
            None => format!("weather {}", self.name()),
        }
    }
}

/// The four difficulties, in vanilla order.
pub const DIFFICULTIES: [&str; 4] = ["peaceful", "easy", "normal", "hard"];

/// Canonical difficulty name for a word or the numbers 0 to 3.
pub fn difficulty_name(s: &str) -> Option<&'static str> {
    match s.trim().to_ascii_lowercase().as_str() {
        "peaceful" | "0" => Some(DIFFICULTIES[0]),
        "easy" | "1" => Some(DIFFICULTIES[1]),
        "normal" | "2" => Some(DIFFICULTIES[2]),
        "hard" | "3" => Some(DIFFICULTIES[3]),
        _ => None,
    }
}

/// Named times of day, as vanilla ticks.
pub fn time_word(w: &str) -> Option<i64> {
    match w.to_ascii_lowercase().as_str() {
        "day" | "morning" => Some(1000),
        "noon" | "midday" => Some(6000),
        "sunset" | "dusk" | "evening" => Some(12000),
        "night" => Some(13000),
        "midnight" => Some(18000),
        "sunrise" | "dawn" => Some(23000),
        _ => None,
    }
}

/// Ticks of a clock time. Tick 0 is 6:00, like Skript and vanilla.
/// `ampm`: `Some(false)` for am, `Some(true)` for pm.
pub fn clock_ticks(hour: f64, minute: f64, ampm: Option<bool>) -> Option<f64> {
    if hour.fract() != 0.0 || minute.fract() != 0.0 || !(0.0..60.0).contains(&minute) {
        return None;
    }
    let mut h = hour as i64;
    match ampm {
        Some(pm) => {
            if !(1..=12).contains(&h) {
                return None;
            }
            h %= 12;
            if pm {
                h += 12;
            }
        }
        None => {
            if !(0..=24).contains(&h) {
                return None;
            }
        }
    }
    let ticks = h * 1000 + (minute as i64 * 1000) / 60;
    Some((ticks - 6000).rem_euclid(DAY) as f64)
}

/// Game rules by vanilla name.
pub const RULES: &[&str] = &[
    "advance_time",
    "advance_weather",
    "allow_entering_nether_using_portals",
    "block_drops",
    "block_explosion_drop_decay",
    "command_block_output",
    "command_blocks_work",
    "drowning_damage",
    "elytra_movement_check",
    "ender_pearls_vanish_on_death",
    "entity_drops",
    "fall_damage",
    "fire_damage",
    "fire_spread_radius_around_player",
    "forgive_dead_players",
    "freeze_damage",
    "global_sound_events",
    "immediate_respawn",
    "keep_inventory",
    "lava_source_conversion",
    "limited_crafting",
    "locator_bar",
    "log_admin_commands",
    "max_block_modifications",
    "max_command_forks",
    "max_command_sequence_length",
    "max_entity_cramming",
    "max_minecart_speed",
    "max_snow_accumulation_height",
    "mob_drops",
    "mob_explosion_drop_decay",
    "mob_griefing",
    "natural_health_regeneration",
    "player_movement_check",
    "players_nether_portal_creative_delay",
    "players_nether_portal_default_delay",
    "players_sleeping_percentage",
    "projectiles_can_break_blocks",
    "pvp",
    "raids",
    "random_tick_speed",
    "reduced_debug_info",
    "respawn_radius",
    "send_command_feedback",
    "show_advancement_messages",
    "show_death_messages",
    "spawn_mobs",
    "spawn_monsters",
    "spawn_patrols",
    "spawn_phantoms",
    "spawn_wandering_traders",
    "spawn_wardens",
    "spawner_blocks_work",
    "spectators_generate_chunks",
    "spread_vines",
    "tnt_explodes",
    "tnt_explosion_drop_decay",
    "universal_anger",
    "water_source_conversion",
];

/// Older names that now have a different vanilla name.
const RULE_ALIASES: &[(&str, &str)] = &[
    ("do_daylight_cycle", "advance_time"),
    ("do_weather_cycle", "advance_weather"),
    ("do_mob_spawning", "spawn_mobs"),
    ("do_mob_loot", "mob_drops"),
    ("do_tile_drops", "block_drops"),
    ("do_entity_drops", "entity_drops"),
    ("natural_regeneration", "natural_health_regeneration"),
    ("announce_advancements", "show_advancement_messages"),
    ("do_immediate_respawn", "immediate_respawn"),
    ("do_insomnia", "spawn_phantoms"),
    ("do_patrol_spawning", "spawn_patrols"),
    ("do_trader_spawning", "spawn_wandering_traders"),
    ("do_warden_spawning", "spawn_wardens"),
    ("do_limited_crafting", "limited_crafting"),
    ("do_vines_spread", "spread_vines"),
    ("do_raids", "raids"),
    ("spawn_radius", "respawn_radius"),
    ("max_command_chain_length", "max_command_sequence_length"),
    ("snow_accumulation_height", "max_snow_accumulation_height"),
];

/// `keepInventory`, `keep-inventory`, `KEEP_INVENTORY` to `keep_inventory`.
fn snake(raw: &str) -> String {
    let mut out = String::with_capacity(raw.len() + 4);
    let mut prev_lower = false;
    for c in raw.chars() {
        if c == '-' || c == ' ' {
            out.push('_');
            prev_lower = false;
        } else if c.is_ascii_uppercase() {
            if prev_lower {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
            prev_lower = false;
        } else {
            out.push(c);
            prev_lower = c.is_ascii_lowercase() || c.is_ascii_digit();
        }
    }
    out
}

/// Vanilla name of a game rule, or `None` when it is not a known rule.
pub fn rule_name(raw: &str) -> Option<&'static str> {
    let s = snake(raw.trim().trim_start_matches("minecraft:"));
    let s = RULE_ALIASES
        .iter()
        .find(|(old, _)| *old == s)
        .map_or(s.as_str(), |(_, new)| *new);
    RULES.iter().copied().find(|r| *r == s)
}

// ---------- syntax tree ----------

#[derive(Debug, Clone, PartialEq)]
pub enum WtKind {
    /// Ticks since sunrise (0 to 23999); tick 0 is 6:00.
    Time,
    /// `clear`, `rain` or `thunder`.
    Weather,
    Difficulty,
    /// A rule by vanilla name: `true`, `false` or a number.
    Gamerule(String),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WtExpr {
    pub kind: WtKind,
    /// `None`: the default world.
    pub world: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum WtEffect {
    SetTime {
        world: Option<Expr>,
        to: Expr,
    },
    /// Add `amount` (ticks or a timespan) times `sign`.
    AddTime {
        world: Option<Expr>,
        amount: Expr,
        sign: i64,
    },
    SetWeather {
        world: Option<Expr>,
        to: Weather,
        dur: Option<Expr>,
    },
    SetRule {
        world: Option<Expr>,
        rule: String,
        value: Expr,
    },
    SetDifficulty {
        world: Option<Expr>,
        to: Expr,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum WtTest {
    Raining,
    Thundering,
    Clear,
    Day,
    Night,
    WeatherIs(Weather),
    DifficultyIs(&'static str),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WtCond {
    pub test: WtTest,
    pub world: Option<Expr>,
    pub negate: bool,
}

// ---------- parsing ----------

fn is(t: &Tok, lit: &str) -> bool {
    t.is_lit(lit)
}

fn pos_of(toks: &[Tok], from: usize, lit: &str) -> Option<usize> {
    toks.iter()
        .enumerate()
        .skip(from)
        .find(|(_, t)| is(t, lit))
        .map(|(i, _)| i)
}

fn typed(toks: &[Tok], env: &Env, want: &[Ty]) -> Result<Expr, String> {
    let e = parse_expr(toks, env)?;
    let got = e.ty(env);
    if got == Ty::Object || want.contains(&got) {
        Ok(e)
    } else {
        Err(format!(
            "expected {}, got {}",
            want.first().map_or("a value", |t| t.name()),
            got.name()
        ))
    }
}

/// The part after a keyword: empty, or `in|of|at <world>`.
pub fn world_part(toks: &[Tok], env: &Env) -> Result<Option<Expr>, String> {
    let Some((first, rest)) = toks.split_first() else {
        return Ok(None);
    };
    if !(is(first, "in") || is(first, "of") || is(first, "at")) || rest.is_empty() {
        return Err("expected 'in world \"name\"'".into());
    }
    typed(rest, env, &[Ty::World, Ty::Text, Ty::Player]).map(Some)
}

/// Strip a leading `the`.
fn skip_the(toks: &[Tok]) -> &[Tok] {
    match toks.split_first() {
        Some((t, rest)) if is(t, "the") => rest,
        _ => toks,
    }
}

fn wt_effect(e: WtEffect) -> Option<Effect> {
    Some(Effect::Wt(Box::new(e)))
}

/// `Ok(None)`: not a weather/time line. `Err`: it is one, but wrong.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    // `add 4 seconds to "world"'s time` reads like `add 4 seconds to time in "world"`.
    if let Some(i) = toks
        .windows(2)
        .position(|w| matches!(&w[0], Tok::Sym("'s")) && is(&w[1], "time"))
        && i > 1
        && matches!(
            first.to_ascii_lowercase().as_str(),
            "set" | "add" | "remove" | "subtract"
        )
    {
        let start = toks
            .get(..i)
            .and_then(|h| {
                h.iter()
                    .rposition(|t| is(t, "to") || is(t, "from") || is(t, "set") || is(t, "the"))
            })
            .map_or(1, |p| p + 1);
        if start < i {
            let mut t: Vec<Tok> = toks.get(..start).unwrap_or_default().to_vec();
            t.push(Tok::Word("time".into()));
            t.push(Tok::Word("in".into()));
            t.extend(toks.get(start..i).unwrap_or_default().iter().cloned());
            t.extend(toks.get(i + 2..).unwrap_or_default().iter().cloned());
            return parse_effect(&t, env);
        }
    }
    match first.to_ascii_lowercase().as_str() {
        "set" => parse_set(toks.get(1..).unwrap_or_default(), env),
        "add" => parse_add(toks, env, 1),
        "remove" | "subtract" => parse_add(toks, env, -1),
        _ => Ok(None),
    }
}

fn parse_set(rest: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let rest = skip_the(rest);
    let Some(Tok::Word(kw)) = rest.first() else {
        return Ok(None);
    };
    let kw = kw.to_ascii_lowercase();
    if !matches!(kw.as_str(), "time" | "weather" | "gamerule" | "difficulty") {
        return Ok(None);
    }
    let body = rest.get(1..).unwrap_or_default();
    let (rule, body) = if kw == "gamerule" {
        let raw = match body.first() {
            Some(Tok::Word(w) | Tok::Str(w)) => w.as_str(),
            _ => return Err("expected a game rule name, like 'set gamerule keepInventory to true'".into()),
        };
        let Some(r) = rule_name(raw) else {
            return Err(format!("unknown game rule '{raw}'"));
        };
        (Some(r), body.get(1..).unwrap_or_default())
    } else {
        (None, body)
    };
    let Some(to_i) = pos_of(body, 0, "to") else {
        return Err(format!("expected 'set {kw} ... to <value>'"));
    };
    let world = world_part(body.get(..to_i).unwrap_or_default(), env)?;
    let value = body.get(to_i + 1..).unwrap_or_default();
    if value.is_empty() {
        return Err(format!("missing value after 'set {kw} to'"));
    }
    let eff = match kw.as_str() {
        "time" => {
            let to = match value {
                [Tok::Word(w)] if time_word(w).is_some() => Expr::Num(time_word(w).unwrap_or(0) as f64),
                _ => typed(value, env, &[Ty::Number])?,
            };
            WtEffect::SetTime { world, to }
        }
        "weather" => {
            let (w_toks, dur) = match value.iter().rposition(|t| is(t, "for")) {
                Some(i) => {
                    let d = typed(
                        value.get(i + 1..).unwrap_or_default(),
                        env,
                        &[Ty::Timespan, Ty::Number],
                    )?;
                    (value.get(..i).unwrap_or_default(), Some(d))
                }
                None => (value, None),
            };
            let to = match w_toks {
                [Tok::Word(w)] => Weather::parse(w),
                _ => None,
            };
            let Some(to) = to else {
                return Err("weather must be rain, thunder or clear".into());
            };
            WtEffect::SetWeather { world, to, dur }
        }
        "gamerule" => WtEffect::SetRule {
            world,
            rule: rule.unwrap_or_default().to_string(),
            value: typed(value, env, &[Ty::Boolean, Ty::Number, Ty::Text])?,
        },
        _ => {
            let to = match value {
                [Tok::Word(w)] if difficulty_name(w).is_some() => {
                    Expr::Text(vec![crate::expr::Part::Lit(w.to_ascii_lowercase())])
                }
                _ => typed(value, env, &[Ty::Text, Ty::Number])?,
            };
            WtEffect::SetDifficulty { world, to }
        }
    };
    Ok(wt_effect(eff))
}

/// `add 100 ticks to time`, `remove 5 seconds from the time in world "w"`.
fn parse_add(toks: &[Tok], env: &Env, sign: i64) -> Result<Option<Effect>, String> {
    let sep = if sign > 0 { "to" } else { "from" };
    let Some(sep_i) = pos_of(toks, 2, sep) else {
        return Ok(None);
    };
    let target = skip_the(toks.get(sep_i + 1..).unwrap_or_default());
    let Some((kw, tail)) = target.split_first() else {
        return Ok(None);
    };
    if !is(kw, "time") || tail.first().is_some_and(|t| is(t, "since") || is(t, "until")) {
        return Ok(None);
    }
    let world = world_part(tail, env)?;
    let amount = typed(
        toks.get(1..sep_i).unwrap_or_default(),
        env,
        &[Ty::Number, Ty::Timespan],
    )?;
    Ok(wt_effect(WtEffect::AddTime { world, amount, sign }))
}

/// Conditions: `it is raining`, `it is night in world "w"`,
/// `weather is thunder`, `difficulty is not hard`.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    let Tok::Word(w) = toks.first()? else {
        return None;
    };
    let w = w.to_ascii_lowercase();
    match w.as_str() {
        "weather" | "difficulty" => {
            let r = toks.get(1..)?;
            let i = r.iter().position(|t| is(t, "is") || is(t, "isn't"))?;
            let world = world_part(r.get(..i)?, env).ok()?;
            finish_named(&w, world, r.get(i..)?)
        }
        "it" | "is" | "isn't" => {
            let rest = if w == "it" { toks.get(1..)? } else { toks };
            let (negate, rest) = take_is(rest)?;
            let (Tok::Word(word), tail) = rest.split_first()? else {
                return None;
            };
            let test = match word.to_ascii_lowercase().as_str() {
                "raining" | "rainy" => WtTest::Raining,
                "thundering" | "stormy" => WtTest::Thundering,
                "sunny" | "clear" | "fair" => WtTest::Clear,
                "day" | "daytime" => WtTest::Day,
                "night" | "nighttime" => WtTest::Night,
                _ => return None,
            };
            let world = world_part(tail, env).ok()?;
            Some(Cond::Wt(Box::new(WtCond { test, world, negate })))
        }
        _ => None,
    }
}

/// `is`, `is not`, `isn't` at the front: whether it negates, and the rest.
fn take_is(toks: &[Tok]) -> Option<(bool, &[Tok])> {
    let (head, rest) = toks.split_first()?;
    if is(head, "isn't") {
        return Some((true, rest));
    }
    if !is(head, "is") {
        return None;
    }
    match rest.split_first() {
        Some((n, r)) if is(n, "not") => Some((true, r)),
        _ => Some((false, rest)),
    }
}

fn finish_named(kw: &str, world: Option<Expr>, rest: &[Tok]) -> Option<Cond> {
    let (negate, rest) = take_is(rest)?;
    let [Tok::Word(word)] = rest else {
        return None;
    };
    let test = if kw == "weather" {
        WtTest::WeatherIs(Weather::parse(word)?)
    } else {
        WtTest::DifficultyIs(difficulty_name(word)?)
    };
    Some(Cond::Wt(Box::new(WtCond { test, world, negate })))
}

// ---------- running ----------

impl Engine {
    /// Name of the world an expression stands for. `None`: the default world.
    fn wt_world(
        &mut self,
        e: &Option<Expr>,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Option<String>, String> {
        let Some(e) = e else {
            return Ok(None);
        };
        let v = self.eval(e, ex, host)?;
        match v {
            Value::World(n) | Value::Text(n) => Ok(Some(n)),
            Value::Location { world, .. } => Ok(world),
            Value::Player(_) => {
                let Some(p) = online(&v, &mut ex.seen, host) else {
                    return Err("that player is not online".into());
                };
                Ok(host.player_location(&p).map(|l| l.3))
            }
            other => Err(format!("expected a world, got {} ({other})", other.type_name())),
        }
    }

    #[inline(never)]
    pub(crate) fn wt_eval(
        &mut self,
        w: &WtExpr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        let world = self.wt_world(&w.world, ex, host)?;
        let world = world.as_deref();
        Ok(match &w.kind {
            WtKind::Time => host
                .world_time(world)
                .map_or(Value::None, |t| Value::Num(t.rem_euclid(DAY) as f64)),
            WtKind::Weather => host
                .weather(world)
                .map_or(Value::None, |w| Value::Text(w.name().into())),
            WtKind::Difficulty => host.difficulty(world).map_or(Value::None, Value::Text),
            WtKind::Gamerule(r) => match host.game_rule(world, r) {
                None => Value::None,
                Some(s) => match s.as_str() {
                    "true" => Value::Bool(true),
                    "false" => Value::Bool(false),
                    _ => s.parse::<f64>().map_or(Value::Text(s), Value::Num),
                },
            },
        })
    }

    #[inline(never)]
    pub(crate) fn wt_effect(
        &mut self,
        e: &WtEffect,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        match e {
            WtEffect::SetTime { world, to } => {
                let world = self.wt_world(world, ex, host)?;
                let n = self.num(to, ex, host)?;
                if !n.is_finite() {
                    return Err("time must be a number".into());
                }
                let target = (n.round() as i64).rem_euclid(DAY);
                // Keep the day count when the host can read it.
                let base = host
                    .world_time(world.as_deref())
                    .map_or(0, |t| t - t.rem_euclid(DAY));
                host.set_world_time(world.as_deref(), base + target);
            }
            WtEffect::AddTime { world, amount, sign } => {
                let world = self.wt_world(world, ex, host)?;
                let delta = match self.eval(amount, ex, host)? {
                    Value::Num(n) if n.is_finite() => n.round() as i64,
                    Value::Timespan { millis } => (millis as f64 / 50.0).round() as i64,
                    other => return Err(format!("can't add {} to the time", other.type_name())),
                } * sign;
                match host.world_time(world.as_deref()) {
                    Some(t) => {
                        let next = t.saturating_add(delta);
                        let next = if next < 0 { next.rem_euclid(DAY) } else { next };
                        host.set_world_time(world.as_deref(), next);
                    }
                    // Only a host that can't read the time (no direct API) gets here.
                    None => host.execute_command(&format!("time add {delta}")),
                }
            }
            WtEffect::SetWeather { world, to, dur } => {
                let world = self.wt_world(world, ex, host)?;
                let ticks = match dur {
                    None => None,
                    Some(d) => {
                        let t = match self.eval(d, ex, host)? {
                            Value::Num(n) => n,
                            Value::Timespan { millis } => millis as f64 / 50.0,
                            other => return Err(format!("weather length can't be {}", other.type_name())),
                        };
                        Some(if t.is_finite() {
                            t.clamp(1.0, MAX_WEATHER_TICKS) as u32
                        } else {
                            1
                        })
                    }
                };
                host.set_weather(world.as_deref(), *to, ticks);
            }
            WtEffect::SetRule { world, rule, value } => {
                let world = self.wt_world(world, ex, host)?;
                let text = match self.eval(value, ex, host)? {
                    Value::Bool(b) => b.to_string(),
                    Value::Num(n) if n.is_finite() && n.fract() == 0.0 => (n as i64).to_string(),
                    Value::Text(t) if matches!(t.to_ascii_lowercase().as_str(), "true" | "false") => {
                        t.to_ascii_lowercase()
                    }
                    Value::Text(t) if t.parse::<i64>().is_ok() => t,
                    other => {
                        return Err(format!(
                            "game rule {rule} needs true, false or a whole number, got {other}"
                        ));
                    }
                };
                host.set_game_rule(world.as_deref(), rule, &text);
            }
            WtEffect::SetDifficulty { world, to } => {
                let world = self.wt_world(world, ex, host)?;
                let v = self.eval(to, ex, host)?;
                let name = match &v {
                    Value::Text(t) => difficulty_name(t),
                    Value::Num(n) if n.fract() == 0.0 => difficulty_name(&(*n as i64).to_string()),
                    _ => None,
                };
                let Some(name) = name else {
                    return Err("difficulty must be peaceful, easy, normal or hard".into());
                };
                host.set_difficulty(world.as_deref(), name);
            }
        }
        Ok(())
    }

    #[inline(never)]
    pub(crate) fn wt_cond(&mut self, c: &WtCond, ex: &mut Exec, host: &mut dyn Host) -> Result<bool, String> {
        let world = self.wt_world(&c.world, ex, host)?;
        let world = world.as_deref();
        let got = match c.test {
            WtTest::Raining => host.weather(world).map(|w| w != Weather::Clear),
            WtTest::Thundering => host.weather(world).map(|w| w == Weather::Thunder),
            WtTest::Clear => host.weather(world).map(|w| w == Weather::Clear),
            WtTest::WeatherIs(w) => host.weather(world).map(|x| x == w),
            WtTest::Day => host.world_time(world).map(is_day),
            WtTest::Night => host.world_time(world).map(|t| !is_day(t)),
            WtTest::DifficultyIs(d) => host.difficulty(world).map(|x| x.eq_ignore_ascii_case(d)),
        };
        // A host that cannot answer makes the plain test false and a negated one true.
        Ok(got.unwrap_or(false) != c.negate)
    }
}

/// Night is tick 13000 up to (not including) 23000, as in vanilla's `time set night`.
fn is_day(t: i64) -> bool {
    !(13_000..23_000).contains(&t.rem_euclid(DAY))
}

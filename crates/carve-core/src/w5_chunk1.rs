//! Wave 5, chunk 1: command cooldowns and failing a command, drops and kept
//! inventories in death and break events, dyeing and glint of the held item,
//! creepers and other explosive entities, dropping items, the whitelist
//! enforcement, growing and shrinking the world border, leashes, the server
//! icon, babies, glowing sign text, pathfinding, persistence, potion effect
//! flags and `can age`.
//!
//! The compiler calls `parse_effect` and `parse_cond` (both behind a cheap
//! first-word check); the engine runs the results through `Engine::w5c1_effect`
//! and `Engine::w5c1_cond`, which are out of line so the hot paths stay small.
//! Game-side work goes through two `Host` methods: `w5c1` (mobs, signs,
//! potion flags) and `read_data_file`. Leashes, persistence and dropped items
//! use console commands Pumpkin implements (`data merge entity`, `summon`).

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::compile::{Cond, Effect};
use crate::engine::{Engine, Exec, online};
use crate::expr::{Env, Expr, Prop};
use crate::host::Host;
use crate::lexer::Tok;
use crate::pattern::{Capture, Pattern};
use crate::value::{EntityRef, PlayerRef, Value};
use crate::world_effects::FxPos;

/// `FireOutcome::set` slots this module writes (value: `true`).
pub const SLOT_NO_ITEM_DROPS: u8 = 240;
pub const SLOT_NO_XP_DROPS: u8 = 241;
pub const SLOT_KEEP_INVENTORY: u8 = 242;
pub const SLOT_KEEP_XP: u8 = 243;

/// Slot of `event-icon` in `on server list ping` (see `w2_events_4`).
const ICON_SLOT: u8 = 5;

/// What starts or stops a creeper's fuse.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ignite {
    /// Explode on the next tick.
    Instant,
    Start,
    Stop,
}

/// A potion effect flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PotionFlag {
    Ambient,
    Particles,
    Icon,
    Infinite,
}

/// Where a mob should walk.
#[derive(Debug, Clone, Copy)]
pub enum PathTarget<'a> {
    Pos(f64, f64, f64),
    Entity(&'a Value),
}

/// A game-side change (`Host::w5c1`). `Some(true)`: done, `Some(false)`: the
/// target exists but can't do it (not a creeper, not ageable, ...).
#[derive(Debug, Clone, Copy)]
pub enum HostOp<'a> {
    Creeper(&'a Value, Ignite),
    /// Is this an ageable mob? (Pumpkin has no age lock, so these all age.)
    Ageable(&'a Value),
    Pathfind(&'a Value, PathTarget<'a>, f64),
    StopPath(&'a Value),
    /// Glowing text on the sign at a block (world, x, y, z).
    SignGlow(&'a str, i32, i32, i32, bool),
    /// Change a flag on the player's active potion effects.
    Potion(&'a PlayerRef, PotionFlag, bool),
    /// Is the server's whitelist switched on?
    WhitelistOn,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Fx {
    /// `cancel the cooldown` (true) / `uncancel the cooldown`.
    Cooldown(bool),
    /// `cancel drops [of items|of xp]`: (items, xp).
    Drops(bool, bool),
    /// `keep the inventory [and experience]`: (inventory, xp).
    Keep(bool, bool),
    Dye {
        player: Expr,
        color: Vec<Expr>,
    },
    /// `force <tool> to glint`: Some(on), or None to clear the override.
    Glint(Expr, Option<bool>),
    Detonate(Expr),
    Creeper(Expr, Ignite),
    Drop {
        what: Expr,
        at: Option<Expr>,
        still: bool,
    },
    Enforce(bool),
    Border {
        world: Option<Expr>,
        shrink: bool,
        radius: bool,
        to: bool,
        amount: Expr,
        over: Option<Expr>,
    },
    Fail(Expr),
    Leash(Expr, Expr),
    Unleash(Expr),
    LoadIcon(Expr),
    /// `set the server icon to the last loaded server icon` (in a ping).
    SetLastIcon,
    Baby(Expr, bool),
    /// `make <mob> breedable` / `not breedable` (breeding cooldown).
    Breedable(Expr, bool),
    SignGlow(Expr, bool),
    Pathfind {
        who: Expr,
        to: Expr,
        speed: Option<Expr>,
    },
    StopPath(Expr),
    Persist(Expr, bool),
    Potion(Expr, PotionFlag, bool),
    /// `launch a firework at <location>`: a plain rocket (Pumpkin ignores
    /// firework NBT on summoned rockets, so effects and flight time can't be set).
    Firework(Expr),
}

/// `%entity% can age`.
#[derive(Debug, Clone, PartialEq)]
pub struct CanAge {
    pub who: Expr,
    pub negate: bool,
}

/// A command's `cooldown:` entries.
#[derive(Debug, Clone, PartialEq)]
pub struct Cooldown {
    pub millis: i64,
    pub message: Option<String>,
    pub bypass: Option<String>,
}

/// Run-time state kept by the engine.
#[derive(Debug, Default)]
pub struct State {
    /// (command, player uuid) -> wall clock millis of the last use.
    last_use: HashMap<(String, String), i64>,
    /// The command run in progress: key, the use before this one, and now.
    current: Option<((String, String), Option<i64>, i64)>,
    /// Set by `fail the command execution`.
    pub(crate) failed: Option<String>,
    /// `last loaded server icon`, as a `data:` URL.
    pub(crate) icon: Option<String>,
}

// ------------------------------------------------------------------ parsing

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum K {
    CdCancel,
    CdUncancel,
    DropsAll,
    DropsItems,
    DropsXp,
    KeepInv,
    KeepBoth,
    KeepXp,
    Dye,
    GlintOn,
    GlintOff,
    GlintClear,
    Detonate,
    Creeper(Ignite),
    Drop,
    DropAt,
    DropStill,
    DropAtStill,
    Enforce,
    Unenforce,
    /// shrink, radius, to, has world, has time
    Border(bool, bool, bool, bool, bool),
    Fail,
    Leash,
    LeashMake,
    Unleash,
    LoadIcon,
    SetLastIcon,
    SetIcon,
    Adult,
    Baby,
    Breedable,
    Unbreedable,
    SignGlow,
    SignNormal,
    Path,
    PathSpeed,
    StopPath,
    Persist,
    NoPersist,
    Potion(PotionFlag, bool),
    /// Index of the location capture.
    Firework(usize),
    PotionShow(PotionFlag, bool),
}

const BORDER: &str = "(world border|worldborder|border)";

fn patterns() -> &'static [Pattern<K>] {
    static P: OnceLock<Vec<Pattern<K>>> = OnceLock::new();
    P.get_or_init(|| {
        // Longer patterns first: `drop x at y` before `drop x`.
        let mut r = raw();
        r.sort_by_key(|(s, _)| std::cmp::Reverse(s.len()));
        r.into_iter()
            .filter_map(|(s, k)| Pattern::new(s, k).ok())
            .collect()
    })
}

/// The words a pattern source can start with: `(a|b) ...` or `word ...`.
/// `None` when the start is not a plain word (always tried).
fn first_words(src: &str) -> Option<Vec<String>> {
    if let Some(rest) = src.strip_prefix('(') {
        let group = &rest[..rest.find(')')?];
        if group.contains(['(', '[', ' ', '%']) {
            return None;
        }
        return Some(group.split('|').map(str::to_string).collect());
    }
    let w = src.split(' ').next()?;
    (!w.is_empty() && w.chars().all(|c| c.is_ascii_alphabetic() || c == '-')).then(|| vec![w.to_string()])
}

/// Pattern indexes by first word (lowercase), plus the ones always tried.
type HeadIndex = (HashMap<String, Vec<usize>>, Vec<usize>);

fn by_head() -> &'static HeadIndex {
    static I: OnceLock<HeadIndex> = OnceLock::new();
    I.get_or_init(|| {
        let mut map: HashMap<String, Vec<usize>> = HashMap::new();
        let mut any = Vec::new();
        for (i, p) in patterns().iter().enumerate() {
            match first_words(p.src) {
                Some(ws) => {
                    for w in ws {
                        map.entry(w).or_default().push(i);
                    }
                }
                None => any.push(i),
            }
        }
        (map, any)
    })
}

/// Checks every pattern of this module compiles. Used by tests.
pub fn check_patterns() -> Result<usize, String> {
    let r = raw();
    for (s, k) in &r {
        Pattern::new(s, *k).map_err(|e| format!("{s}: {e}"))?;
    }
    for s in CONDITION_SOURCES {
        Pattern::new(s, false).map_err(|e| format!("{s}: {e}"))?;
    }
    Ok(r.len())
}

fn raw() -> Vec<(&'static str, K)> {
    {
        use Ignite::*;
        use K::*;
        use PotionFlag::*;
        let mut v: Vec<(&'static str, K)> = vec![
            ("(cancel|ignore) [the] [current] [command] cooldown", CdCancel),
            (
                "(uncancel|unignore) [the] [current] [command] cooldown",
                CdUncancel,
            ),
            ("(cancel|clear|delete) [the] drops", DropsAll),
            ("(cancel|clear|delete) [the] drops of items", DropsItems),
            (
                "(cancel|clear|delete) [the] drops of (xp|exp|experience|experiences)",
                DropsXp,
            ),
            ("keep [the] (inventory|items)", KeepInv),
            (
                "keep [the] (inventory|items) and (xp|exp|experience|experiences) [(point|points)]",
                KeepBoth,
            ),
            (
                "keep [the] (xp|exp|experience|experiences) [(point|points)]",
                KeepXp,
            ),
            (
                "keep [the] (xp|exp|experience|experiences) [(point|points)] and (inventory|items)",
                KeepBoth,
            ),
            ("(dye|color|colour|paint) %item% %object%", Dye),
            ("(force|make) %item% [to] [start] (glint|glinting)", GlintOn),
            ("(force|make) %item% [to] (not|stop) (glint|glinting)", GlintOff),
            (
                "(clear|delete|reset) [the] enchantment glint override of %item%",
                GlintClear,
            ),
            ("detonate %object%", Detonate),
            (
                "(instantly|immediately) explode [(creeper|creepers)] %object%",
                Creeper(Instant),
            ),
            (
                "explode [(creeper|creepers)] %object% (instantly|immediately)",
                Creeper(Instant),
            ),
            ("ignite (creeper|creepers) %object%", Creeper(Start)),
            (
                "start (ignition|explosion) [process] of [(creeper|creepers)] %object%",
                Creeper(Start),
            ),
            (
                "stop (ignition|explosion) [process] of [(creeper|creepers)] %object%",
                Creeper(Stop),
            ),
            ("launch [a] firework at %location%", Firework(0)),
            ("launch [a] firework [effect] %object% at %location%", Firework(1)),
            (
                "launch [a] firework [effect] %object% at %location% (timed|with (duration|power)) %number%",
                Firework(1),
            ),
            ("drop %object%", Drop),
            ("drop %object% (at|on|near|above) %location%", DropAt),
            ("drop %object% (without|with no) velocity", DropStill),
            (
                "drop %object% (at|on|near|above) %location% (without|with no) velocity",
                DropAtStill,
            ),
            ("enforce [the] [server] (whitelist|white list)", Enforce),
            ("unenforce [the] [server] (whitelist|white list)", Unenforce),
            (
                "fail [the] command execution with [the] error [message] %text%",
                Fail,
            ),
            ("(leash|lead) %object% to %object%", Leash),
            ("make %object% (leash|lead) %object%", LeashMake),
            ("(unleash|unlead) [holder of] %object%", Unleash),
            (
                "load [the] server icon (from|of) [the] [image] [file] %text%",
                LoadIcon,
            ),
            (
                "set [the] [(shown|sent)] server icon to [the] last loaded server icon",
                SetLastIcon,
            ),
            ("set [the] [(shown|sent)] server icon to %object%", SetIcon),
            ("make %object% [(a|an)] adult", Adult),
            ("make %object% [(a|an)] (baby|child)", Baby),
            ("force %object% to (be|become) [(a|an)] adult", Adult),
            ("force %object% to (be|become) [(a|an)] (baby|child)", Baby),
            ("make %object% breedable", Breedable),
            (
                "make %object% (unbreedable|not breedable|non-breedable)",
                Unbreedable,
            ),
            ("make %object% have glowing text", SignGlow),
            (
                "make %object% have (normal|non-glowing|non glowing) text",
                SignNormal,
            ),
            ("make %object% (pathfind|move) (to|towards) %object%", Path),
            (
                "make %object% (pathfind|move) (to|towards) %object% at speed %number%",
                PathSpeed,
            ),
            ("make %object% stop (pathfinding|moving)", StopPath),
            ("make %object% (persist|persistent)", Persist),
            ("make %object% not (persist|persistent)", NoPersist),
            ("force %object% to persist", Persist),
            ("force %object% to not persist", NoPersist),
            ("prevent %object% from persisting", NoPersist),
        ];
        for (word, flag) in [("ambient", Ambient), ("(infinite|permanent)", Infinite)] {
            for (src, on) in [
                ("make %player%'s [active] potion effects ", true),
                ("make %player%'s [active] potion effects not ", false),
                ("make [the] [active] potion effects of %player% ", true),
                ("make [the] [active] potion effects of %player% not ", false),
            ] {
                let s: &'static str = Box::leak(format!("{src}{word}").into_boxed_str());
                v.push((s, Potion(flag, on)));
            }
        }
        for (word, flag) in [("particles", Particles), ("(icon|icons)", Icon)] {
            for (verb, on) in [("show", true), ("hide", false)] {
                for of in [
                    "(of|for) %player%'s [active] potion effects",
                    "(of|for) [the] [active] potion effects of %player%",
                ] {
                    let s: &'static str =
                        Box::leak(format!("{verb} [the] [potion] {word} {of}").into_boxed_str());
                    v.push((s, PotionShow(flag, on)));
                }
            }
        }
        for shrink in [false, true] {
            for radius in [false, true] {
                for to in [false, true] {
                    for world in [false, true] {
                        for time in [false, true] {
                            let verb = if shrink {
                                "(shrink|contract)"
                            } else {
                                "(expand|grow)"
                            };
                            let what = if radius { "radius" } else { "diameter" };
                            let of = if world {
                                format!("[the] {BORDER} of %world%")
                            } else {
                                format!("[the] {BORDER}")
                            };
                            let by = if to { "to" } else { "by" };
                            let over = if time {
                                " over [a period of] %timespan%"
                            } else {
                                ""
                            };
                            let head = if radius {
                                format!("{verb} [the] {what} of {of}")
                            } else {
                                format!("{verb} [[the] {what} of] {of}")
                            };
                            let s = format!("{head} {by} %number%{over}");
                            let s: &'static str = Box::leak(s.into_boxed_str());
                            v.push((s, Border(shrink, radius, to, world, time)));
                        }
                    }
                }
            }
        }
        v
    }
}

/// Every pattern source, for tests and suggestions.
pub fn pattern_sources() -> Vec<&'static str> {
    patterns().iter().map(|p| p.src).collect()
}

const HEADS: &[&str] = &[
    "cancel",
    "ignore",
    "uncancel",
    "unignore",
    "clear",
    "delete",
    "keep",
    "dye",
    "color",
    "colour",
    "paint",
    "force",
    "make",
    "reset",
    "detonate",
    "instantly",
    "immediately",
    "explode",
    "ignite",
    "start",
    "stop",
    "drop",
    "enforce",
    "unenforce",
    "expand",
    "grow",
    "shrink",
    "contract",
    "fail",
    "leash",
    "lead",
    "unleash",
    "unlead",
    "load",
    "set",
    "prevent",
    "show",
    "hide",
    "launch",
];

fn head_matches(toks: &[Tok]) -> bool {
    matches!(toks.first(), Some(Tok::Word(w)) if HEADS.iter().any(|h| w.eq_ignore_ascii_case(h)))
}

fn take(c: &mut [Option<Capture>], i: usize) -> Option<Expr> {
    match c.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    }
}

/// The player of `player's tool` / `tool of player`.
fn tool_player(e: Expr) -> Result<Expr, String> {
    match e {
        Expr::Prop(Prop::Tool, p) => Ok(*p),
        _ => Err("only the held item can be changed here (use player's tool)".into()),
    }
}

fn event_is(env: &Env, names: &[&str]) -> bool {
    names.iter().any(|n| env.trigger_name == format!("'on {n}'"))
}

/// An effect of this module: the effect and whether the trigger stops after
/// it (`fail the command execution`). `Ok(None)`: not one of ours.
pub fn parse_effect(toks: &[Tok], env: &Env, is_command: bool) -> Result<Option<(Effect, bool)>, String> {
    if !head_matches(toks) {
        return Ok(None);
    }
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    let (map, any) = by_head();
    let ps = patterns();
    let mut idx: Vec<usize> = map
        .get(first.to_ascii_lowercase().as_str())
        .map_or_else(Vec::new, Clone::clone);
    idx.extend_from_slice(any);
    // Keep the longest-first order of `patterns()`.
    idx.sort_unstable();
    let mut hit = None;
    for i in idx {
        let p = &ps[i];
        if let Ok(c) = p.matches(toks, env) {
            hit = Some((p.kind, c));
            break;
        }
    }
    let Some((k, mut c)) = hit else {
        return Ok(None);
    };
    let need = |c: &mut [Option<Capture>], i: usize| take(c, i).ok_or_else(|| "missing value".to_string());
    let fx = match k {
        K::CdCancel | K::CdUncancel => {
            if !is_command {
                return Err("only a command has a cooldown to cancel".into());
            }
            Fx::Cooldown(k == K::CdCancel)
        }
        K::DropsAll | K::DropsItems | K::DropsXp => {
            if !event_is(env, &["break", "death"]) {
                return Err("drops can only be cancelled in 'on break' and 'on death'".into());
            }
            Fx::Drops(k != K::DropsXp, k != K::DropsItems)
        }
        K::KeepInv | K::KeepBoth | K::KeepXp => {
            if !event_is(env, &["death"]) {
                return Err("'keep the inventory' only works in 'on death'".into());
            }
            Fx::Keep(k != K::KeepXp, k != K::KeepInv)
        }
        K::Dye => {
            let item = tool_player(need(&mut c, 0)?)?;
            let color = need(&mut c, 1)?;
            Fx::Dye {
                player: item,
                color: vec![color],
            }
        }
        K::GlintOn | K::GlintOff | K::GlintClear => {
            let p = tool_player(need(&mut c, 0)?)?;
            Fx::Glint(
                p,
                match k {
                    K::GlintOn => Some(true),
                    K::GlintOff => Some(false),
                    _ => None,
                },
            )
        }
        K::Detonate => Fx::Detonate(need(&mut c, 0)?),
        K::Creeper(i) => Fx::Creeper(need(&mut c, 0)?, i),
        K::Drop | K::DropAt | K::DropStill | K::DropAtStill => Fx::Drop {
            what: need(&mut c, 0)?,
            at: take(&mut c, 1),
            still: matches!(k, K::DropStill | K::DropAtStill),
        },
        K::Enforce | K::Unenforce => Fx::Enforce(k == K::Enforce),
        K::Border(shrink, radius, to, world, time) => {
            let mut i = 0;
            let w = if world {
                i += 1;
                take(&mut c, 0)
            } else {
                None
            };
            let amount = need(&mut c, i)?;
            Fx::Border {
                world: w,
                shrink,
                radius,
                to,
                amount,
                over: if time { take(&mut c, i + 1) } else { None },
            }
        }
        K::Fail => {
            if !is_command {
                return Err("only a command can fail".into());
            }
            Fx::Fail(need(&mut c, 0)?)
        }
        K::Leash => Fx::Leash(need(&mut c, 0)?, need(&mut c, 1)?),
        K::LeashMake => {
            let holder = need(&mut c, 0)?;
            Fx::Leash(need(&mut c, 1)?, holder)
        }
        K::Unleash => Fx::Unleash(need(&mut c, 0)?),
        K::LoadIcon => Fx::LoadIcon(need(&mut c, 0)?),
        K::SetLastIcon | K::SetIcon if env.ext4 != Some(crate::w2_events_4::ExtEvent::ServerListPing) => {
            return Err("the server icon can only be set in 'on server list ping'".into());
        }
        K::SetLastIcon => Fx::SetLastIcon,
        K::SetIcon => {
            if env.ext4 != Some(crate::w2_events_4::ExtEvent::ServerListPing) {
                return Err("the server icon can only be set in 'on server list ping'".into());
            }
            let v = need(&mut c, 0)?;
            return Ok(Some((
                Effect::Ext4(Box::new(crate::w2_events_4::ExtEffect::SetVal(ICON_SLOT, v))),
                false,
            )));
        }
        K::Adult | K::Baby => Fx::Baby(need(&mut c, 0)?, k == K::Baby),
        K::Breedable | K::Unbreedable => Fx::Breedable(need(&mut c, 0)?, k == K::Breedable),
        K::SignGlow | K::SignNormal => Fx::SignGlow(need(&mut c, 0)?, k == K::SignGlow),
        K::Path | K::PathSpeed => Fx::Pathfind {
            who: need(&mut c, 0)?,
            to: need(&mut c, 1)?,
            speed: take(&mut c, 2),
        },
        K::StopPath => Fx::StopPath(need(&mut c, 0)?),
        K::Persist | K::NoPersist => Fx::Persist(need(&mut c, 0)?, k == K::Persist),
        K::Potion(f, on) | K::PotionShow(f, on) => Fx::Potion(need(&mut c, 0)?, f, on),
        K::Firework(i) => Fx::Firework(need(&mut c, i)?),
    };
    let stop = matches!(fx, Fx::Fail(_));
    Ok(Some((Effect::W5c1(Box::new(fx)), stop)))
}

/// `%object% can age` / `can grow up` and the negated forms.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    static P: OnceLock<Vec<Pattern<bool>>> = OnceLock::new();
    if !toks.iter().any(|t| t.is_lit("age") || t.is_lit("grow")) {
        return None;
    }
    let ps = P.get_or_init(|| {
        [
            ("%object% can (age|grow up|grow old|grow older)", false),
            (
                "%object% (can't|cannot|can not) (age|grow up|grow old|grow older)",
                true,
            ),
        ]
        .into_iter()
        .filter_map(|(s, k)| Pattern::new(s, k).ok())
        .collect()
    });
    ps.iter().find_map(|p| {
        let mut c = p.matches(toks, env).ok()?;
        Some(Cond::W5c1(Box::new(CanAge {
            who: take(&mut c, 0)?,
            negate: p.kind,
        })))
    })
}

/// Condition pattern sources, for tests.
pub const CONDITION_SOURCES: &[&str] = &[
    "%object% can (age|grow up|grow old|grow older)",
    "%object% (can't|cannot|can not) (age|grow up|grow old|grow older)",
];

/// Read a command's `cooldown...:` entry. `Ok(false)`: not a cooldown entry.
pub fn command_entry(cd: &mut Option<Box<Cooldown>>, lower: &str, raw: &str) -> Result<bool, String> {
    let value = |key: &str| raw.get(key.len()..).unwrap_or("").trim().to_string();
    let slot = cd.get_or_insert_with(|| {
        Box::new(Cooldown {
            millis: 0,
            message: None,
            bypass: None,
        })
    });
    if lower.starts_with("cooldown message:") {
        slot.message = Some(value("cooldown message:"));
    } else if lower.starts_with("cooldown bypass:") {
        slot.bypass = Some(value("cooldown bypass:")).filter(|s| !s.is_empty());
    } else if lower.starts_with("cooldown storage:") {
        return Err("'cooldown storage:' is not supported; Carve keeps cooldowns in memory".into());
    } else if lower.starts_with("cooldown:") {
        let v = value("cooldown:");
        slot.millis =
            parse_span(&v).ok_or_else(|| format!("can't understand the cooldown '{v}', use '10 seconds'"))?;
    } else {
        return Ok(false);
    }
    Ok(true)
}

/// `10 seconds`, `1 minute and 30 seconds`, `2.5 hours`.
pub fn parse_span(s: &str) -> Option<i64> {
    let words: Vec<&str> = s
        .split_whitespace()
        .filter(|w| !w.eq_ignore_ascii_case("and"))
        .flat_map(|w| w.split(','))
        .filter(|w| !w.is_empty())
        .collect();
    if words.is_empty() || !words.len().is_multiple_of(2) {
        return None;
    }
    let mut total = 0i64;
    for pair in words.chunks(2) {
        let n: f64 = if pair[0].eq_ignore_ascii_case("a") || pair[0].eq_ignore_ascii_case("an") {
            1.0
        } else {
            pair[0].parse().ok()?
        };
        let unit = crate::timespans::unit_ms(&pair[1].to_ascii_lowercase())?;
        total = total.checked_add(crate::timespans::span_ms(n, unit)?)?;
    }
    (total > 0).then_some(total)
}

fn span_text(ms: i64) -> String {
    let mut s = String::new();
    crate::timespans::push_span(&mut s, ms.max(0));
    s
}

/// The PNG `data:` URL Minecraft shows as a server icon. The image must be a
/// 64x64 PNG.
pub fn icon_data_url(bytes: &[u8]) -> Result<String, String> {
    const MAGIC: &[u8] = b"\x89PNG\r\n\x1a\n";
    if !bytes.starts_with(MAGIC) || bytes.len() < 24 || &bytes[12..16] != b"IHDR" {
        return Err("the server icon must be a PNG image".into());
    }
    let w = u32::from_be_bytes([bytes[16], bytes[17], bytes[18], bytes[19]]);
    let h = u32::from_be_bytes([bytes[20], bytes[21], bytes[22], bytes[23]]);
    if (w, h) != (64, 64) {
        return Err(format!(
            "the server icon must be 64x64 pixels, this one is {w}x{h}"
        ));
    }
    Ok(format!("data:image/png;base64,{}", base64(bytes)))
}

fn base64(b: &[u8]) -> String {
    const T: &[u8; 64] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut out = String::with_capacity(b.len().div_ceil(3) * 4);
    for c in b.chunks(3) {
        let n = (u32::from(c[0]) << 16)
            | (u32::from(*c.get(1).unwrap_or(&0)) << 8)
            | u32::from(*c.get(2).unwrap_or(&0));
        out.push(T[(n >> 18) as usize & 63] as char);
        out.push(T[(n >> 12) as usize & 63] as char);
        out.push(if c.len() > 1 {
            T[(n >> 6) as usize & 63] as char
        } else {
            '='
        });
        out.push(if c.len() > 2 {
            T[n as usize & 63] as char
        } else {
            '='
        });
    }
    out
}

/// A uuid as the four ints of an NBT int array.
pub fn uuid_ints(u: &str) -> Option<[i32; 4]> {
    let hex: String = u.chars().filter(|c| *c != '-').collect();
    if hex.len() != 32 {
        return None;
    }
    let mut out = [0i32; 4];
    for (i, o) in out.iter_mut().enumerate() {
        *o = u32::from_str_radix(hex.get(i * 8..i * 8 + 8)?, 16).ok()? as i32;
    }
    Some(out)
}

/// An `Item` NBT compound for `summon item`.
fn item_nbt(id: &str, count: u32) -> String {
    let id = id.strip_prefix("minecraft:").unwrap_or(id);
    format!("{{id:\"minecraft:{id}\",count:{}}}", count.clamp(1, 99))
}

fn uuid_of(v: &Value) -> Option<&str> {
    match v {
        Value::Player(p) => Some(&p.uuid),
        Value::Entity(e) => Some(&e.uuid),
        _ => None,
    }
}

fn each(v: Value) -> Vec<Value> {
    match v {
        Value::List(l) => l,
        Value::None => Vec::new(),
        v => vec![v],
    }
}

// -------------------------------------------------------------- the engine

impl Engine {
    /// Before a command with a `cooldown:` runs. `Some(message)`: the sender
    /// has to wait.
    #[cold]
    pub(crate) fn w5c1_cooldown_gate(
        &mut self,
        name: &str,
        cd: &Cooldown,
        player: Option<&PlayerRef>,
        host: &mut dyn Host,
    ) -> Option<String> {
        let p = player?;
        if cd.millis <= 0 || cd.bypass.as_deref().is_some_and(|b| host.has_permission(p, b)) {
            return None;
        }
        let now = host.wall_clock_millis();
        let key = (name.to_string(), p.uuid.clone());
        let prev = self.w5c1.last_use.get(&key).copied();
        if let Some(t) = prev {
            let elapsed = now - t;
            if elapsed < cd.millis {
                let left = span_text(cd.millis - elapsed);
                return Some(match &cd.message {
                    Some(m) => m
                        .replace("%remaining time%", &left)
                        .replace("%elapsed time%", &span_text(elapsed))
                        .replace("%cooldown time%", &span_text(cd.millis)),
                    None => format!("You have to wait {left} before using this command again"),
                });
            }
        }
        self.w5c1.last_use.insert(key.clone(), now);
        self.w5c1.current = Some((key, prev, now));
        None
    }

    /// After the first slice of a command run.
    #[cold]
    pub(crate) fn w5c1_command_done(&mut self) {
        self.w5c1.current = None;
    }

    #[inline(never)]
    pub(crate) fn w5c1_cond(
        &mut self,
        c: &CanAge,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<bool, String> {
        let v = self.eval(&c.who, ex, host)?;
        Ok(host.w5c1(HostOp::Ageable(&v)).is_some_and(|b| b != c.negate))
    }

    fn w5c1_pos(&mut self, v: &Value, ex: &mut Exec, host: &mut dyn Host) -> Option<FxPos> {
        let fx = |(x, y, z, w): (f64, f64, f64, String)| FxPos {
            x,
            y,
            z,
            world: Some(w),
        };
        match v {
            Value::Location { x, y, z, world } => Some(FxPos {
                x: *x,
                y: *y,
                z: *z,
                world: world.clone(),
            }),
            Value::Entity(e) => host.entity_location(e).map(fx),
            Value::Player(_) => {
                let p = online(v, &mut ex.seen, host)?;
                host.player_location(&p).map(fx)
            }
            _ => None,
        }
    }

    #[inline(never)]
    pub(crate) fn w5c1_effect(&mut self, f: &Fx, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        match f {
            Fx::Cooldown(cancel) => {
                if let Some((key, prev, now)) = self.w5c1.current.clone() {
                    match (cancel, prev) {
                        (true, Some(t)) => {
                            self.w5c1.last_use.insert(key, t);
                        }
                        (true, None) => {
                            self.w5c1.last_use.remove(&key);
                        }
                        (false, _) => {
                            self.w5c1.last_use.insert(key, now);
                        }
                    }
                }
            }
            Fx::Drops(items, xp) | Fx::Keep(items, xp) => {
                let (a, b) = if matches!(f, Fx::Drops(..)) {
                    (SLOT_NO_ITEM_DROPS, SLOT_NO_XP_DROPS)
                } else {
                    (SLOT_KEEP_INVENTORY, SLOT_KEEP_XP)
                };
                // Like `cancel event`: only before the first wait.
                if !ex.waited {
                    if *items {
                        self.ext_out.push((a, Value::Bool(true)));
                    }
                    if *xp {
                        self.ext_out.push((b, Value::Bool(true)));
                    }
                }
            }
            Fx::Dye { player, color } => {
                let Some(p) = self.player(player, ex, host)? else {
                    return Ok(());
                };
                let c = self.eval(&color[0], ex, host)?;
                let bytes = crate::skbee_nbt::ItemComp::DyedColor.encode(&c)?;
                if host.held_item(&p).is_some()
                    && !host.set_held_component(&p, crate::skbee_nbt::ItemComp::DyedColor, bytes)
                {
                    return Err("this server can't dye items".into());
                }
            }
            Fx::Glint(player, on) => {
                let Some(p) = self.player(player, ex, host)? else {
                    return Ok(());
                };
                let comp = crate::skbee_nbt::ItemComp::GlintOverride;
                let bytes = match on {
                    Some(b) => comp.encode(&Value::Bool(*b))?,
                    None => None,
                };
                if host.held_item(&p).is_some() && !host.set_held_component(&p, comp, bytes) {
                    return Err("this server can't change the enchantment glint".into());
                }
            }
            Fx::Detonate(who) => {
                for v in each(self.eval(who, ex, host)?) {
                    if host.w5c1(HostOp::Creeper(&v, Ignite::Instant)) == Some(true) {
                        continue;
                    }
                    let Value::Entity(e) = &v else { continue };
                    let power = match e.kind.as_str() {
                        "tnt" | "tnt_minecart" => 4.0,
                        "end_crystal" => 6.0,
                        "wind_charge" | "breeze_wind_charge" => 1.2,
                        "fireball" | "small_fireball" => 1.0,
                        "wither_skull" => 1.0,
                        "firework_rocket" => 0.0,
                        _ => continue,
                    };
                    if let Some(pos) = self.w5c1_pos(&v, ex, host) {
                        host.entity_remove(e);
                        if power > 0.0 {
                            host.explode(&pos, power, false);
                        }
                    }
                }
            }
            Fx::Creeper(who, mode) => {
                for v in each(self.eval(who, ex, host)?) {
                    host.w5c1(HostOp::Creeper(&v, *mode));
                }
            }
            Fx::Drop { what, at, still } => {
                let pos = match at {
                    Some(a) => {
                        let v = self.eval(a, ex, host)?;
                        self.w5c1_pos(&v, ex, host)
                    }
                    None => match ex.ctx.player.clone() {
                        Some(p) => self.w5c1_pos(&Value::Player(p), ex, host),
                        None => return Err("drop needs 'at <location>' here".into()),
                    },
                };
                let Some(pos) = pos else { return Ok(()) };
                let motion = if *still { ",Motion:[0.0d,0.0d,0.0d]" } else { "" };
                for v in each(self.eval(what, ex, host)?) {
                    let (id, count) = match &v {
                        Value::Item { id, count, .. } => (id.clone(), *count),
                        Value::Text(t) => (crate::w2_conditions_3::norm_id(t), 1),
                        _ => return Err(format!("can't drop {} ({v})", v.type_name())),
                    };
                    host.execute_command(&format!(
                        "summon minecraft:item {:.3} {:.3} {:.3} {{Item:{}{motion}}}",
                        pos.x,
                        pos.y,
                        pos.z,
                        item_nbt(&id, count)
                    ));
                }
            }
            Fx::Firework(at) => {
                let v = self.eval(at, ex, host)?;
                if let Some(pos) = self.w5c1_pos(&v, ex, host) {
                    host.execute_command(&format!(
                        "summon minecraft:firework_rocket {:.3} {:.3} {:.3}",
                        pos.x, pos.y, pos.z
                    ));
                }
            }
            Fx::Enforce(on) => {
                if *on && host.w5c1(HostOp::WhitelistOn) != Some(false) {
                    for p in host.online_players() {
                        if !host.is_whitelisted(&p.name) && !host.is_op(&p) {
                            host.kick(&p.name, Some("You are not whitelisted on this server!"));
                        }
                    }
                }
            }
            Fx::Border {
                world,
                shrink,
                radius,
                to,
                amount,
                over,
            } => {
                let w = match world {
                    Some(w) => match self.eval(w, ex, host)? {
                        Value::World(n) | Value::Text(n) => Some(n),
                        _ => None,
                    },
                    None => host.default_world(),
                };
                let Some(w) = w else { return Ok(()) };
                let mut n = self.num(amount, ex, host)?;
                if !n.is_finite() {
                    return Ok(());
                }
                if *radius {
                    n *= 2.0;
                }
                let secs = match over {
                    Some(o) => match self.eval(o, ex, host)? {
                        Value::Timespan { millis } => millis as f64 / 1000.0,
                        v => v.as_num().unwrap_or(0.0),
                    },
                    None => 0.0,
                };
                let cur = host.border_size(&w).unwrap_or(5.999_996_8e7);
                let new = match (to, shrink) {
                    (true, _) => n,
                    (false, false) => cur + n,
                    (false, true) => cur - n,
                };
                host.set_border_size(&w, new.clamp(1.0, 5.999_996_8e7), secs.max(0.0));
            }
            Fx::Fail(msg) => {
                let m = self.text(msg, ex, host)?;
                if !ex.waited {
                    self.w5c1.failed = Some(m);
                }
            }
            Fx::Leash(who, holder) => {
                let h = self.eval(holder, ex, host)?;
                let Some(ints) = uuid_of(&h).and_then(uuid_ints) else {
                    return Ok(());
                };
                for v in each(self.eval(who, ex, host)?) {
                    if let Value::Entity(e) = &v {
                        host.execute_command(&format!(
                            "data merge entity {} {{leash:{{UUID:[I;{},{},{},{}]}}}}",
                            e.uuid, ints[0], ints[1], ints[2], ints[3]
                        ));
                    }
                }
            }
            Fx::Unleash(who) => {
                for v in each(self.eval(who, ex, host)?) {
                    if let Value::Entity(e) = &v {
                        host.execute_command(&format!("data merge entity {} {{leash:0b}}", e.uuid));
                    }
                }
            }
            Fx::LoadIcon(path) => {
                let path = self.text(path, ex, host)?;
                if path.contains("..") {
                    return Err("the icon file must be inside the Carve data folder".into());
                }
                let bytes = host
                    .read_data_file(&path)
                    .ok_or_else(|| format!("can't read the icon file '{path}'"))?;
                self.w5c1.icon = Some(icon_data_url(&bytes)?);
            }
            Fx::SetLastIcon => {
                if !ex.waited
                    && let Some(i) = &self.w5c1.icon
                {
                    self.ext_out.push((ICON_SLOT, Value::Text(i.clone())));
                }
            }
            Fx::Baby(who, baby) => {
                // Through the NBT, so Pumpkin sends the baby flag to players.
                let nbt = if *baby {
                    "{Age:-24000,IsBaby:1b}"
                } else {
                    "{Age:0,IsBaby:0b}"
                };
                for v in each(self.eval(who, ex, host)?) {
                    if let Value::Entity(e) = &v {
                        host.execute_command(&format!("data merge entity {} {nbt}", e.uuid));
                    }
                }
            }
            Fx::Breedable(who, on) => {
                // Like Bukkit's `setBreed`: age 0 can breed, 6000 is the cooldown.
                for v in each(self.eval(who, ex, host)?) {
                    if let Value::Entity(e) = &v {
                        let age = if *on { 0 } else { 6000 };
                        host.execute_command(&format!("data merge entity {} {{Age:{age}}}", e.uuid));
                    }
                }
            }
            Fx::SignGlow(at, on) => {
                for v in each(self.eval(at, ex, host)?) {
                    let Some(pos) = self.w5c1_pos(&v, ex, host) else {
                        continue;
                    };
                    let Some(w) = pos.world.clone().or_else(|| host.default_world()) else {
                        continue;
                    };
                    host.w5c1(HostOp::SignGlow(
                        &w,
                        pos.x.floor() as i32,
                        pos.y.floor() as i32,
                        pos.z.floor() as i32,
                        *on,
                    ));
                }
            }
            Fx::Pathfind { who, to, speed } => {
                let target = self.eval(to, ex, host)?;
                let speed = match speed {
                    Some(s) => self.num(s, ex, host)?,
                    None => 1.0,
                };
                let speed = if speed.is_finite() {
                    speed.clamp(0.0, 10.0)
                } else {
                    1.0
                };
                let t = match &target {
                    Value::Location { x, y, z, .. } => PathTarget::Pos(*x, *y, *z),
                    Value::Player(_) | Value::Entity(_) => PathTarget::Entity(&target),
                    _ => return Ok(()),
                };
                for v in each(self.eval(who, ex, host)?) {
                    host.w5c1(HostOp::Pathfind(&v, t, speed));
                }
            }
            Fx::StopPath(who) => {
                for v in each(self.eval(who, ex, host)?) {
                    host.w5c1(HostOp::StopPath(&v));
                }
            }
            Fx::Persist(who, on) => {
                for v in each(self.eval(who, ex, host)?) {
                    match &v {
                        Value::Entity(e) => host.execute_command(&format!(
                            "data merge entity {} {{PersistenceRequired:{}b}}",
                            e.uuid,
                            u8::from(*on)
                        )),
                        Value::Location { .. } => self.w5c1_leaves(&v, *on, ex, host),
                        _ => {}
                    }
                }
            }
            Fx::Potion(who, flag, on) => {
                for v in each(self.eval(who, ex, host)?) {
                    if let Some(p) = online(&v, &mut ex.seen, host) {
                        host.w5c1(HostOp::Potion(&p, *flag, *on));
                    }
                }
            }
        }
        Ok(())
    }

    /// Leaves keep (`persistent=true`) or lose their leaves when no log is near.
    fn w5c1_leaves(&mut self, v: &Value, on: bool, ex: &mut Exec, host: &mut dyn Host) {
        let Some(pos) = self.w5c1_pos(v, ex, host) else {
            return;
        };
        let at = crate::block_ops::BlockPos {
            x: pos.x.floor() as i32,
            y: pos.y.floor() as i32,
            z: pos.z.floor() as i32,
            world: pos.world.clone(),
        };
        let Some(id) = host.block_type(&at) else { return };
        if !id.ends_with("_leaves") {
            return;
        }
        host.execute_command(&format!(
            "setblock {} {} {} minecraft:{id}[persistent={on}]",
            at.x, at.y, at.z
        ));
    }
}

/// An entity reference for tests and hosts.
pub fn entity(uuid: &str, kind: &str) -> Value {
    Value::Entity(EntityRef {
        uuid: uuid.into(),
        kind: kind.into(),
    })
}

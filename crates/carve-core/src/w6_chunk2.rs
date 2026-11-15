//! Wave 6, chunk 2: effects that were marked missing but the server can do.
//!
//! Goat horns, silence, screaming, growing trees, zombifying,
//! allay duplication and script switching go through console commands Pumpkin
//! implements (`data merge entity`, `place feature`, `carve ...`). `reset`,
//! `remove all` and the vector rotations are plain Carve logic. The effects
//! that need the game API (flight, fake blocks, interaction and text display
//! flags, sound categories, shearing, taming) go through `Host::w6c2`.
//!
//! The compiler calls [`parse_effect`] before its other effect parsers; the
//! run side is [`Engine::w6c2_effect`], out of line so the hot paths stay small.

use std::collections::HashMap;
use std::sync::OnceLock;

use crate::compile::Effect;
use crate::engine::{Engine, Exec};
use crate::expr::{Env, Expr, VarRef};
use crate::host::Host;
use crate::lexer::Tok;
use crate::pattern::{Capture, Pattern};
use crate::value::{PlayerRef, Value};
use crate::world_effects::FxPos;

/// Which script action (`enable script "x"` ...).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ScriptOp {
    Enable,
    Disable,
    Reload,
}

/// A flag of a text display entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayFlag {
    Shadow,
    SeeThrough,
}

/// Game-side work for `Host::w6c2`. `Some(true)` done, `Some(false)` the
/// target can't do it, `None` this host can't.
#[derive(Debug, Clone, Copy)]
pub enum HostOp<'a> {
    /// `allow flight for <player>` (true) / `disallow flight` (false).
    AllowFlight(&'a PlayerRef, bool),
    /// A fake block for one player: world, x, y, z, block id (`None`: the real block).
    FakeBlock(&'a PlayerRef, &'a str, i32, i32, i32, Option<&'a str>),
    /// `make <interaction> responsive`.
    Responsive(&'a Value, bool),
    /// Drop shadow or see-through flag of a text display.
    Display(&'a Value, DisplayFlag, bool),
    /// `stop sound <name> in <category> for <player>`.
    StopSound(&'a PlayerRef, Option<&'a str>, &'a str),
    /// Tame (true) or untame a wolf or cat.
    Tame(&'a Value, bool, Option<&'a PlayerRef>),
}

/// A rotation axis for [`rotate`].
#[derive(Debug, Clone, PartialEq)]
pub enum Axis {
    X,
    Y,
    Z,
    Vec(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Fx {
    /// Remove (false) or regrow (true) goat horns: who, left, right.
    Horn(Expr, bool, bool, bool),
    Silent(Expr, bool),
    Scream(Expr, bool),
    Tree(&'static str, Expr),
    /// `zombify` (true) / `unzombify` (false).
    Zombify(Expr, bool),
    Duplicate(Expr),
    Script(ScriptOp, Expr),
    /// `reset {x}`: back to the default of the held value.
    Reset(VarRef),
    Rotate(VarRef, Axis, Expr),
    Allow(Expr, bool),
    FakeBlock {
        who: Expr,
        at: Expr,
        block: Option<Expr>,
    },
    Responsive(Expr, bool),
    Display(Expr, DisplayFlag, bool),
    StopSound {
        sound: Option<Expr>,
        category: &'static str,
        who: Option<Expr>,
    },
    Shear(Expr, bool),
    Tame(Expr, bool),
    /// `make <who> ride <vehicle>` (no vehicle: dismount).
    Ride(Expr, Option<Expr>),
    /// `make <who> swing [their] main/off hand`.
    Swing(Expr, bool),
    /// `equip [<who>] with <items>`.
    Equip(Option<Expr>, Vec<Expr>),
    /// `ban <who> [because of <reason>] for <timespan>` (ip: ban the address).
    BanFor {
        who: Expr,
        reason: Option<Expr>,
        span: Expr,
        ip: bool,
    },
}

// ------------------------------------------------------------------ pure logic

/// Rotate a vector. Angles are degrees, like Skript's `VectorMath`.
pub fn rotate(v: (f64, f64, f64), axis: &Axis2, deg: f64) -> (f64, f64, f64) {
    let (x, y, z) = v;
    let (s, c) = deg.to_radians().sin_cos();
    match *axis {
        Axis2::X => (x, y * c - z * s, y * s + z * c),
        Axis2::Y => (x * c + z * s, y, -x * s + z * c),
        Axis2::Z => (x * c - y * s, x * s + y * c, z),
        Axis2::Vec(ax, ay, az) => {
            // Rodrigues' rotation formula.
            let len = (ax * ax + ay * ay + az * az).sqrt();
            if len == 0.0 {
                return v;
            }
            let (kx, ky, kz) = (ax / len, ay / len, az / len);
            let dot = kx * x + ky * y + kz * z;
            let cross = (ky * z - kz * y, kz * x - kx * z, kx * y - ky * x);
            (
                x * c + cross.0 * s + kx * dot * (1.0 - c),
                y * c + cross.1 * s + ky * dot * (1.0 - c),
                z * c + cross.2 * s + kz * dot * (1.0 - c),
            )
        }
    }
}

/// A resolved rotation axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Axis2 {
    X,
    Y,
    Z,
    Vec(f64, f64, f64),
}

/// Sound categories of `stop sound ... in <category>`.
pub const CATEGORIES: &[&str] = &[
    "master", "music", "record", "weather", "block", "hostile", "neutral", "player", "ambient", "voice",
];

/// `place feature` ids for the tree words of `grow <type> at <location>`.
pub const TREES: &[(&str, &str)] = &[
    ("oak", "oak_checked"),
    ("tree", "oak_checked"),
    ("big tree", "fancy_oak_checked"),
    ("birch", "birch_checked"),
    ("jungle", "jungle_tree"),
    ("acacia", "acacia_checked"),
    ("dark oak", "dark_oak_checked"),
    ("spruce", "spruce_checked"),
    ("redwood", "spruce_checked"),
    ("cherry", "cherry_checked"),
    ("mangrove", "mangrove_checked"),
];

/// Is this text a UUID? Checked before it goes on a console command line.
pub fn uuid_ok(u: &str) -> bool {
    u.len() == 36 && u.chars().all(|c| c.is_ascii_hexdigit() || c == '-')
}

/// A script name that is safe to put on a console command line.
pub fn script_arg(name: &str) -> Option<&str> {
    let n = name.trim();
    (!n.is_empty()
        && n.chars()
            .all(|c| c.is_alphanumeric() || matches!(c, '_' | '-' | '.' | '/')))
    .then_some(n)
}

// -------------------------------------------------------------------- parsing

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum K {
    /// left, right, regrow
    Horn(bool, bool, bool),
    Silent(bool),
    Scream(bool),
    Tree(&'static str),
    Zombify(bool),
    Duplicate,
    Script(ScriptOp),
    Reset,
    RemoveAll,
    /// 0 x, 1 y, 2 z, 3 vector
    Rotate(u8),
    Allow(bool),
    FakeBlock,
    FakeReset,
    Responsive(bool),
    Display(DisplayFlag, bool),
    StopSound(&'static str),
    StopAllCat(&'static str),
    Shear(bool),
    Tame(bool),
    BanFor(bool),
    Ride,
    Dismount,
    Swing(bool),
    Equip(usize),
    EquipOwn(usize),
}

fn raw() -> Vec<(&'static str, K)> {
    use K::*;
    let mut v: Vec<(&'static str, K)> = Vec::new();
    for (side, l, r) in [
        ("left", true, false),
        ("right", false, true),
        ("both", true, true),
    ] {
        for (verb, regrow) in [("remove", false), ("(regrow|replace)", true)] {
            let a: &'static str =
                Box::leak(format!("{verb} [the] {side} (horn|horns) of %object%").into_boxed_str());
            v.push((a, Horn(l, r, regrow)));
            let b: &'static str =
                Box::leak(format!("{verb} %object%'s {side} (horn|horns)").into_boxed_str());
            v.push((b, Horn(l, r, regrow)));
        }
    }
    for (verb, regrow) in [("remove", false), ("(regrow|replace)", true)] {
        let a: &'static str = Box::leak(format!("{verb} [the] horns of %object%").into_boxed_str());
        v.push((a, Horn(true, true, regrow)));
        let b: &'static str = Box::leak(format!("{verb} %object%'s horns").into_boxed_str());
        v.push((b, Horn(true, true, regrow)));
    }
    v.extend([
        ("silence %object%", Silent(true)),
        ("unsilence %object%", Silent(false)),
        ("make %object% silent", Silent(true)),
        ("make %object% not silent", Silent(false)),
        ("make %object% (start screaming|scream)", Scream(true)),
        ("force %object% to (start screaming|scream)", Scream(true)),
        ("make %object% stop screaming", Scream(false)),
        ("force %object% to stop screaming", Scream(false)),
        ("zombify %object%", Zombify(true)),
        ("unzombify %object%", Zombify(false)),
        ("make %object% (duplicate|clone)", Duplicate),
        (
            "(enable|load) script [(file|named)] %text% [with errors]",
            Script(ScriptOp::Enable),
        ),
        (
            "(enable|load) skript file %text% [with errors]",
            Script(ScriptOp::Enable),
        ),
        (
            "reload script [(file|named)] %text% [with errors]",
            Script(ScriptOp::Reload),
        ),
        (
            "reload skript file %text% [with errors]",
            Script(ScriptOp::Reload),
        ),
        (
            "(disable|unload) script [(file|named)] %text% [with errors]",
            Script(ScriptOp::Disable),
        ),
        (
            "(disable|unload) skript file %text% [with errors]",
            Script(ScriptOp::Disable),
        ),
        ("reset %variable%", Reset),
        ("remove all %object% from %variable%", RemoveAll),
        ("rotate %variable% around [the] x [-] axis by %number%", Rotate(0)),
        ("rotate %variable% around [the] y [-] axis by %number%", Rotate(1)),
        ("rotate %variable% around [the] z [-] axis by %number%", Rotate(2)),
        ("rotate %variable% around %object% by %number%", Rotate(3)),
        ("(allow|enable) (fly|flight) (for|to) %object%", Allow(true)),
        ("(disallow|disable) (fly|flight) (for|to) %object%", Allow(false)),
        ("make %object% see %location% as %object%", FakeBlock),
        (
            "make %object% see %location% as [(the|its|their)] (original|normal|actual) [(block|blocks)]",
            FakeReset,
        ),
        ("make %object% responsive", Responsive(true)),
        ("make %object% (unresponsive|not responsive)", Responsive(false)),
        (
            "(apply|add) (drop|text) shadow to [[the] text of] %object%",
            Display(DisplayFlag::Shadow, true),
        ),
        (
            "(apply|add) (drop|text) shadow to %object%'s text",
            Display(DisplayFlag::Shadow, true),
        ),
        (
            "(remove|clear) (drop|text) shadow from [[the] text of] %object%",
            Display(DisplayFlag::Shadow, false),
        ),
        (
            "(remove|clear) (drop|text) shadow from %object%'s text",
            Display(DisplayFlag::Shadow, false),
        ),
        (
            "make %object% visible through (blocks|walls)",
            Display(DisplayFlag::SeeThrough, true),
        ),
        (
            "force %object% to be visible through (blocks|walls)",
            Display(DisplayFlag::SeeThrough, true),
        ),
        (
            "(prevent|block) %object% from being (visible|seen) through (blocks|walls)",
            Display(DisplayFlag::SeeThrough, false),
        ),
        ("shear %object%", Shear(true)),
        ("unshear %object%", Shear(false)),
        ("tame %object%", Tame(true)),
        ("domesticate %object%", Tame(true)),
        ("untame %object%", Tame(false)),
        ("undomesticate %object%", Tame(false)),
    ]);
    v.extend([
        ("(make|let|force) %object% [to] (ride|mount) [(in|on)] %object%", Ride),
        ("(make|let|force) %object% [to] dismount", Dismount),
        (
            "(make|let|force) %object% [to] (dismount|leave) [(from|of)] [(any|the|its|their|his|her)] (vehicle|vehicles)",
            Dismount,
        ),
        ("make %object% swing [their] [main] hand", Swing(true)),
        ("make %object% swing [their] off hand", Swing(false)),
        ("make %object% swing [their] offhand", Swing(false)),
    ]);
    // Up to four items: `a`, `a and b`, `a, b and c`, ...
    for n in 1..=4usize {
        let mut items = String::new();
        for i in 0..n {
            if i > 0 {
                items.push_str(if i + 1 == n { " and " } else { ", " });
            }
            items.push_str("%object%");
        }
        let a: &'static str = Box::leak(format!("equip %object% with {items}").into_boxed_str());
        v.push((a, Equip(n)));
        let b: &'static str = Box::leak(format!("make %object% wear {items}").into_boxed_str());
        v.push((b, Equip(n)));
        let c: &'static str = Box::leak(format!("equip with {items}").into_boxed_str());
        v.push((c, EquipOwn(n)));
    }
    for (src, ip) in [
        (
            "ban %object% [(by reason of|because of|because|on account of|due to|with reason) %text%] for %timespan%",
            false,
        ),
        (
            "ban ip %object% [(by reason of|because of|because|on account of|due to|with reason) %text%] for %timespan%",
            true,
        ),
        (
            "ban %object% by ip [(by reason of|because of|because|on account of|due to|with reason) %text%] for %timespan%",
            true,
        ),
    ] {
        v.push((src, BanFor(ip)));
    }
    for cat in CATEGORIES {
        // `from player` is the old way to name the listener, so that category needs `in`.
        let from = if *cat == "player" { "in" } else { "(in|from)" };
        let a: &'static str = Box::leak(
            format!("stop (sound|sounds) %text% {from} [the] {cat} [(from playing to|for|to) %object%]")
                .into_boxed_str(),
        );
        v.push((a, StopSound(cat)));
        let b: &'static str = Box::leak(
            format!("stop playing (sound|sounds) %text% {from} [the] {cat} [(for|to) %object%]")
                .into_boxed_str(),
        );
        v.push((b, StopSound(cat)));
        let c: &'static str = Box::leak(
            format!("stop all (sound|sounds) {from} [the] {cat} [(from playing to|for|to) %object%]")
                .into_boxed_str(),
        );
        v.push((c, StopAllCat(cat)));
    }
    for (word, feature) in TREES {
        let s: &'static str = Box::leak(
            format!("(grow|create|generate) [(a|an)] {word} [tree] (at|on|above) %location%")
                .into_boxed_str(),
        );
        v.push((s, Tree(feature)));
    }
    v
}

fn patterns() -> &'static [Pattern<K>] {
    static P: OnceLock<Vec<Pattern<K>>> = OnceLock::new();
    P.get_or_init(|| {
        let mut r = raw();
        r.sort_by_key(|(s, _)| std::cmp::Reverse(s.len()));
        r.into_iter()
            .filter_map(|(s, k)| Pattern::new(s, k).ok())
            .collect()
    })
}

/// Checks every pattern compiles. Used by tests.
pub fn check_patterns() -> Result<usize, String> {
    let r = raw();
    for (s, k) in &r {
        Pattern::new(s, *k).map_err(|e| format!("{s}: {e}"))?;
    }
    Ok(r.len())
}

/// Every pattern source, for tests and suggestions.
pub fn pattern_sources() -> Vec<&'static str> {
    patterns().iter().map(|p| p.src).collect()
}

fn first_word(src: &str) -> Vec<String> {
    if let Some(rest) = src.strip_prefix('(')
        && let Some(end) = rest.find(')')
    {
        return rest[..end].split('|').map(str::to_string).collect();
    }
    vec![src.split(' ').next().unwrap_or("").to_string()]
}

fn by_head() -> &'static HashMap<String, Vec<usize>> {
    static I: OnceLock<HashMap<String, Vec<usize>>> = OnceLock::new();
    I.get_or_init(|| {
        let mut map: HashMap<String, Vec<usize>> = HashMap::new();
        for (i, p) in patterns().iter().enumerate() {
            for w in first_word(p.src) {
                map.entry(w).or_default().push(i);
            }
        }
        map
    })
}

fn take(c: &mut [Option<Capture>], i: usize) -> Option<Expr> {
    match c.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    }
}

fn take_var(c: &mut [Option<Capture>], i: usize) -> Option<VarRef> {
    match c.get_mut(i).and_then(Option::take) {
        Some(Capture::Var(v)) => Some(v),
        _ => None,
    }
}

/// An effect of this module. `Ok(None)`: not one of ours.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    let Some(idx) = by_head().get(first.to_ascii_lowercase().as_str()) else {
        return Ok(None);
    };
    let ps = patterns();
    let mut idx = idx.clone();
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
        K::Horn(l, r, regrow) => Fx::Horn(need(&mut c, 0)?, l, r, regrow),
        K::Silent(on) => Fx::Silent(need(&mut c, 0)?, on),
        K::Scream(on) => Fx::Scream(need(&mut c, 0)?, on),
        K::Tree(f) => Fx::Tree(f, need(&mut c, 0)?),
        K::Zombify(on) => Fx::Zombify(need(&mut c, 0)?, on),
        K::Duplicate => Fx::Duplicate(need(&mut c, 0)?),
        K::Script(op) => Fx::Script(op, need(&mut c, 0)?),
        K::Reset => Fx::Reset(take_var(&mut c, 0).ok_or("expected a variable")?),
        K::RemoveAll => {
            let what = need(&mut c, 0)?;
            let var = take_var(&mut c, 1).ok_or("expected a variable")?;
            if !var.list {
                return Err("'remove all' needs a list variable like {x::*}".into());
            }
            return Ok(Some(Effect::ListRemoveAll(var, vec![what])));
        }
        K::Rotate(a) => {
            let var = take_var(&mut c, 0).ok_or("expected a vector variable")?;
            if a == 3 {
                let axis = need(&mut c, 1)?;
                Fx::Rotate(var, Axis::Vec(axis), need(&mut c, 2)?)
            } else {
                let axis = [Axis::X, Axis::Y, Axis::Z][usize::from(a)].clone();
                Fx::Rotate(var, axis, need(&mut c, 1)?)
            }
        }
        K::Allow(on) => Fx::Allow(need(&mut c, 0)?, on),
        K::FakeBlock => Fx::FakeBlock {
            who: need(&mut c, 0)?,
            at: need(&mut c, 1)?,
            block: Some(need(&mut c, 2)?),
        },
        K::FakeReset => Fx::FakeBlock {
            who: need(&mut c, 0)?,
            at: need(&mut c, 1)?,
            block: None,
        },
        K::Responsive(on) => Fx::Responsive(need(&mut c, 0)?, on),
        K::Display(f, on) => Fx::Display(need(&mut c, 0)?, f, on),
        K::Shear(on) => Fx::Shear(need(&mut c, 0)?, on),
        K::Tame(on) => Fx::Tame(need(&mut c, 0)?, on),
        K::Ride => Fx::Ride(need(&mut c, 0)?, Some(need(&mut c, 1)?)),
        K::Dismount => Fx::Ride(need(&mut c, 0)?, None),
        K::Swing(main) => Fx::Swing(need(&mut c, 0)?, main),
        K::Equip(n) => {
            let who = need(&mut c, 0)?;
            let items = (1..=n).map(|i| need(&mut c, i)).collect::<Result<Vec<_>, _>>()?;
            Fx::Equip(Some(who), items)
        }
        K::EquipOwn(n) => Fx::Equip(
            None,
            (0..n).map(|i| need(&mut c, i)).collect::<Result<Vec<_>, _>>()?,
        ),
        K::BanFor(ip) => Fx::BanFor {
            who: need(&mut c, 0)?,
            reason: take(&mut c, 1),
            span: need(&mut c, 2)?,
            ip,
        },
        K::StopSound(cat) => Fx::StopSound {
            sound: Some(need(&mut c, 0)?),
            category: cat,
            who: take(&mut c, 1),
        },
        K::StopAllCat(cat) => Fx::StopSound {
            sound: None,
            category: cat,
            who: take(&mut c, 0),
        },
    };
    Ok(Some(Effect::W6c2(Box::new(fx))))
}

// -------------------------------------------------------------------- running

fn each(v: Value) -> Vec<Value> {
    match v {
        Value::List(l) => l,
        Value::None => Vec::new(),
        v => vec![v],
    }
}

/// What a command line calls this player or entity.
fn selector(v: &Value) -> Option<String> {
    match v {
        Value::Player(p) if !p.name.is_empty() => Some(p.name.clone()),
        Value::Player(p) => Some(p.uuid.clone()),
        Value::Entity(e) if uuid_ok(&e.uuid) => Some(e.uuid.clone()),
        _ => None,
    }
}

/// Run a game-side change; say so in the log when the game could not do it.
fn w6c2_run(host: &mut dyn Host, op: HostOp<'_>, what: &str) {
    match host.w6c2(op) {
        Some(true) => {}
        Some(false) => host.log(
            crate::host::LogLevel::Warn,
            &format!("{what}: that target can't do it"),
        ),
        None => host.log(
            crate::host::LogLevel::Warn,
            &format!("{what}: not possible on this server"),
        ),
    }
}

impl Engine {
    /// The players a value names (online ones).
    fn w6c2_players(
        &mut self,
        e: &Expr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Vec<PlayerRef>, String> {
        let mut out = Vec::new();
        for v in each(self.eval(e, ex, host)?) {
            if let Some(p) = crate::engine::online(&v, &mut ex.seen, host) {
                out.push(p);
            }
        }
        Ok(out)
    }

    /// Run `data merge entity` for every entity the expression names.
    fn w6c2_merge(
        &mut self,
        who: &Expr,
        nbt: &str,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        for v in each(self.eval(who, ex, host)?) {
            if let Value::Entity(e) = &v
                && uuid_ok(&e.uuid)
            {
                host.execute_command(&format!("data merge entity {} {nbt}", e.uuid));
            }
        }
        Ok(())
    }

    #[inline(never)]
    #[cold]
    pub(crate) fn w6c2_effect(&mut self, f: &Fx, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        match f {
            Fx::Horn(who, left, right, regrow) => {
                let on = u8::from(*regrow);
                let mut nbt = String::from("{");
                if *left {
                    nbt.push_str(&format!("HasLeftHorn:{on}b"));
                }
                if *right {
                    if *left {
                        nbt.push(',');
                    }
                    nbt.push_str(&format!("HasRightHorn:{on}b"));
                }
                nbt.push('}');
                self.w6c2_merge(who, &nbt, ex, host)?;
            }
            Fx::Silent(who, on) => {
                self.w6c2_merge(who, &format!("{{Silent:{}b}}", u8::from(*on)), ex, host)?
            }
            Fx::Scream(who, on) => {
                self.w6c2_merge(who, &format!("{{IsScreamingGoat:{}b}}", u8::from(*on)), ex, host)?;
            }
            Fx::Tree(feature, at) => {
                let v = self.eval(at, ex, host)?;
                if let Some((x, y, z, _)) = self.as_location(&v, ex, host) {
                    host.execute_command(&format!(
                        "place feature minecraft:{feature} {} {} {}",
                        x.floor() as i64,
                        y.floor() as i64,
                        z.floor() as i64
                    ));
                }
            }
            Fx::Zombify(who, on) => {
                let (from, to) = if *on {
                    ("villager", "zombie_villager")
                } else {
                    ("zombie_villager", "villager")
                };
                for v in each(self.eval(who, ex, host)?) {
                    let Value::Entity(e) = &v else { continue };
                    if e.kind != from {
                        continue;
                    }
                    let Some((x, y, z, world)) = host.entity_location(e) else {
                        continue;
                    };
                    let name = host.entity_name(e);
                    let health = host.entity_health(e);
                    host.entity_remove(e);
                    let pos = FxPos {
                        x,
                        y,
                        z,
                        world: Some(world),
                    };
                    if let Some(z) = host.spawn_entity(to, &pos) {
                        // The new entity is only findable on the next tick, so name and health go
                        // through a console command that runs then.
                        let mut nbt = String::new();
                        if let Some(n) = name.as_deref().filter(|n| *n != "Villager" && !n.is_empty()) {
                            let n = n.replace('\\', "\\\\").replace('"', "\\\"");
                            nbt.push_str(&format!("CustomName:\"{n}\",CustomNameVisible:1b,"));
                        }
                        if let Some(h) = health {
                            nbt.push_str(&format!("Health:{:.1}f,", h.min(20.0)));
                        }
                        if uuid_ok(&z.uuid) && !nbt.is_empty() {
                            nbt.pop();
                            host.execute_command(&format!("data merge entity {} {{{nbt}}}", z.uuid));
                        }
                    }
                }
            }
            Fx::Duplicate(who) => {
                for v in each(self.eval(who, ex, host)?) {
                    let Value::Entity(e) = &v else { continue };
                    if e.kind != "allay" {
                        continue;
                    }
                    let Some((x, y, z, world)) = host.entity_location(e) else {
                        continue;
                    };
                    let pos = FxPos {
                        x,
                        y,
                        z,
                        world: Some(world),
                    };
                    if let Some(copy) = host.spawn_entity("allay", &pos) {
                        // Both allays wait before they can duplicate again.
                        for t in [e, &copy] {
                            if uuid_ok(&t.uuid) {
                                host.execute_command(&format!(
                                    "data merge entity {} {{DuplicationCooldown:6000L}}",
                                    t.uuid
                                ));
                            }
                        }
                    }
                }
            }
            Fx::Script(op, name) => {
                let n = self.text(name, ex, host)?;
                let Some(n) = script_arg(&n) else {
                    return Err(format!("'{n}' is not a script name"));
                };
                let verb = match op {
                    ScriptOp::Enable => "enable",
                    ScriptOp::Disable => "disable",
                    ScriptOp::Reload => "reload",
                };
                host.execute_command(&format!("carve {verb} {n}"));
            }
            Fx::Reset(v) => {
                let k = self.key(v, ex, host)?.into_owned();
                if v.list {
                    self.w6c2_delete(v, &k, ex, host);
                } else {
                    let cur = if v.local {
                        ex.locals.get(&k).cloned()
                    } else {
                        self.vars.get_ref(&k).cloned()
                    };
                    match cur {
                        Some(Value::Num(_)) => self.store(v.local, &k, Value::Num(0.0), ex, host),
                        Some(Value::Text(_)) => self.store(v.local, &k, Value::Text(String::new()), ex, host),
                        Some(Value::Bool(_)) => self.store(v.local, &k, Value::Bool(false), ex, host),
                        Some(_) => self.w6c2_delete(v, &k, ex, host),
                        None => {}
                    }
                }
            }
            Fx::Rotate(v, axis, deg) => {
                let deg = self.num(deg, ex, host)?;
                let ax = match axis {
                    Axis::X => Axis2::X,
                    Axis::Y => Axis2::Y,
                    Axis::Z => Axis2::Z,
                    Axis::Vec(e) => match self.eval(e, ex, host)? {
                        Value::Vector { x, y, z } => Axis2::Vec(x, y, z),
                        other => {
                            return Err(format!(
                                "can't rotate around {} ({other}), expected a vector",
                                other.type_name()
                            ));
                        }
                    },
                };
                let keys: Vec<String> = if v.list {
                    self.list_items(v, ex, host)?
                        .into_iter()
                        .map(|(k, _)| {
                            format!(
                                "{}{k}",
                                self.key(v, ex, host)
                                    .map(|b| b.trim_end_matches('*').to_string())
                                    .unwrap_or_default()
                            )
                        })
                        .collect()
                } else {
                    vec![self.key(v, ex, host)?.into_owned()]
                };
                for k in keys {
                    let cur = if v.local {
                        ex.locals.get(&k).cloned()
                    } else {
                        self.vars.get_ref(&k).cloned()
                    };
                    if let Some(Value::Vector { x, y, z }) = cur {
                        let (x, y, z) = rotate((x, y, z), &ax, deg);
                        self.store(v.local, &k, Value::Vector { x, y, z }, ex, host);
                    }
                }
            }
            Fx::Allow(who, on) => {
                for p in self.w6c2_players(who, ex, host)? {
                    w6c2_run(host, HostOp::AllowFlight(&p, *on), "allow flight");
                }
            }
            Fx::FakeBlock { who, at, block } => {
                let target = self.eval(at, ex, host)?;
                let Some((x, y, z, world)) = self.as_location(&target, ex, host) else {
                    return Ok(());
                };
                let id = match block {
                    Some(b) => match self.eval(b, ex, host)? {
                        Value::Item { id, .. } => Some(crate::w2_conditions_3::norm_id(&id)),
                        Value::Text(t) => Some(crate::w2_conditions_3::norm_id(&t)),
                        other => {
                            return Err(format!("can't show {} ({other}) as a block", other.type_name()));
                        }
                    },
                    None => None,
                };
                let world = world.or_else(|| host.default_world()).unwrap_or_default();
                for p in self.w6c2_players(who, ex, host)? {
                    w6c2_run(
                        host,
                        HostOp::FakeBlock(
                            &p,
                            &world,
                            x.floor() as i32,
                            y.floor() as i32,
                            z.floor() as i32,
                            id.as_deref(),
                        ),
                        "fake block",
                    );
                }
            }
            Fx::Responsive(who, on) => {
                for v in each(self.eval(who, ex, host)?) {
                    w6c2_run(host, HostOp::Responsive(&v, *on), "responsive");
                }
            }
            Fx::Display(who, flag, on) => {
                for v in each(self.eval(who, ex, host)?) {
                    w6c2_run(host, HostOp::Display(&v, *flag, *on), "display");
                }
            }
            Fx::StopSound { sound, category, who } => {
                let sound = match sound {
                    Some(s) => Some(self.text(s, ex, host)?),
                    None => None,
                };
                let players = match who {
                    Some(w) => self.w6c2_players(w, ex, host)?,
                    None => match ex.ctx.player.clone() {
                        Some(p) => vec![p],
                        None => host.online_players(),
                    },
                };
                for p in players {
                    w6c2_run(
                        host,
                        HostOp::StopSound(&p, sound.as_deref(), category),
                        "stop sound",
                    );
                }
            }
            Fx::Ride(who, vehicle) => {
                let vehicle = match vehicle {
                    Some(v) => match self.eval(v, ex, host)? {
                        Value::List(l) => l.into_iter().next(),
                        Value::None => return Ok(()),
                        v => Some(v),
                    },
                    None => None,
                };
                let target = match &vehicle {
                    Some(v) => match selector(v) {
                        Some(s) => Some(s),
                        None => return Ok(()),
                    },
                    None => None,
                };
                for v in each(self.eval(who, ex, host)?) {
                    let Some(rider) = selector(&v) else { continue };
                    // Nobody rides themselves.
                    if target.as_ref() == Some(&rider) {
                        continue;
                    }
                    match &target {
                        Some(t) => host.execute_command(&format!("ride {rider} mount {t}")),
                        None => host.execute_command(&format!("ride {rider} dismount")),
                    }
                }
            }
            Fx::Swing(who, main) => {
                for v in each(self.eval(who, ex, host)?) {
                    if let Some(s) = selector(&v) {
                        host.execute_command(&crate::w2_effects_3::swing_command(&s, *main));
                    }
                }
            }
            Fx::Equip(who, items) => {
                let who = match who {
                    Some(w) => self.eval(w, ex, host)?,
                    None => match ex.ctx.player.clone() {
                        Some(p) => Value::Player(p),
                        None => return Err("equip needs a target here, like 'equip player with ...'".into()),
                    },
                };
                let mut list = Vec::new();
                for e in items {
                    list.extend(each(self.eval(e, ex, host)?));
                }
                let items = list;
                for target in each(who) {
                    let Some(sel) = selector(&target) else { continue };
                    for it in &items {
                        let id = match it {
                            Value::Item { id, .. } => crate::w2_conditions_3::norm_id(id),
                            Value::Text(t) => crate::w2_conditions_3::norm_id(t),
                            _ => continue,
                        };
                        let slot = crate::w2_effects_1::equip_slot(&id);
                        host.execute_command(&format!(
                            "item replace entity {sel} {slot} with minecraft:{id}"
                        ));
                    }
                }
            }
            Fx::BanFor {
                who,
                reason,
                span,
                ip,
            } => {
                let reason = match reason {
                    Some(r) => Some(self.text(r, ex, host)?),
                    None => None,
                };
                let millis = match self.eval(span, ex, host)? {
                    Value::Timespan { millis } => millis,
                    other => {
                        return Err(format!(
                            "can't ban for {} ({other}), expected a timespan",
                            other.type_name()
                        ));
                    }
                };
                let secs = (millis.max(0) / 1000) as u64;
                for v in each(self.eval(who, ex, host)?) {
                    let name = match &v {
                        Value::Player(p) => host.resolve_player(&p.uuid).unwrap_or_else(|| p.name.clone()),
                        Value::Text(t) => t.clone(),
                        _ => continue,
                    };
                    if !host.ban_for(&name, reason.as_deref(), *ip, secs) {
                        host.log(
                            crate::host::LogLevel::Warn,
                            &format!(
                                "ban for a time: '{name}' has to be online (a permanent ban was not made)"
                            ),
                        );
                    }
                }
            }
            Fx::Shear(who, on) => {
                // Through the NBT: a sheep that was spawned a moment ago is only
                // findable by the game on the next tick, the console command runs then.
                for v in each(self.eval(who, ex, host)?) {
                    if let Value::Entity(e) = &v
                        && e.kind == "sheep"
                        && uuid_ok(&e.uuid)
                    {
                        host.execute_command(&format!(
                            "data merge entity {} {{Sheared:{}b}}",
                            e.uuid,
                            u8::from(*on)
                        ));
                    }
                }
            }
            Fx::Tame(who, on) => {
                let owner = ex.ctx.player.clone();
                for v in each(self.eval(who, ex, host)?) {
                    w6c2_run(host, HostOp::Tame(&v, *on, owner.as_ref()), "tame");
                }
            }
        }
        Ok(())
    }

    fn w6c2_delete(&mut self, v: &VarRef, key: &str, ex: &mut Exec, host: &mut dyn Host) {
        if v.local {
            ex.locals.delete(key);
        } else {
            self.vars.delete(key);
            self.mark_dirty(host);
        }
    }
}

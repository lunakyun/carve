//! Misc player effects: poison/cure, hide/reveal, swing, sprint, clear title,
//! dismount/eject/ride, custom name visibility, resource packs, save world,
//! stop server, `toggle` and `transform`.
//!
//! All of them share one effect variant ([`W2Eff`]) so the hot enums stay small.

use crate::compile::{EffKind, Effect};
use crate::expr::{Env, Expr, VarRef, parse_expr, parse_varref};
use crate::host::Host;
use crate::lexer::Tok;
use crate::list_ops::INPUT_KEY;
use crate::pattern::Capture;
use crate::value::PlayerRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum W2Op {
    /// `poison player [for 10 seconds]`
    Poison,
    /// `cure player [of poison]`
    Cure,
    /// `hide player [from viewer]`
    Hide,
    /// `reveal player [to viewer]`
    Reveal,
    SaveWorld,
    StopServer,
    /// `make player start sprinting`; the bool is the new state.
    Sprint(bool),
    /// `make player swing main hand`; the bool is "main hand".
    Swing(bool),
    /// `clear title of player` (false) or `reset title of player` (true).
    ClearTitle(bool),
    Dismount,
    Eject,
    /// `make rider ride vehicle`
    Ride,
    /// `show` (true) or `hide` (false) a custom name.
    NameVisible(bool),
    /// `send resource pack "url" [with hash "sha1"] to player`
    ResourcePack,
    /// `toggle {flag}`
    Toggle,
}

/// What a capture slot holds, in pattern order.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Arg {
    Player,
    Span,
    Text,
    Var,
}

impl W2Op {
    pub fn layout(self) -> &'static [Arg] {
        use Arg::*;
        match self {
            W2Op::Poison => &[Player, Span],
            W2Op::Hide | W2Op::Reveal | W2Op::Ride => &[Player, Player],
            W2Op::SaveWorld | W2Op::StopServer => &[],
            W2Op::ResourcePack => &[Text, Text, Player],
            W2Op::Toggle => &[Var],
            _ => &[Player],
        }
    }
}

/// One compiled effect. `args` follows [`W2Op::layout`] (without the
/// variable slot); missing optional parts are `None`.
#[derive(Debug, Clone, PartialEq)]
pub struct W2Eff {
    pub op: W2Op,
    pub args: Vec<Option<Expr>>,
    pub var: Option<VarRef>,
}

/// `transform {list::*} with (input * 2)`: each item is replaced by the result.
#[derive(Debug, Clone, PartialEq)]
pub struct Transform {
    pub list: VarRef,
    pub with: Expr,
}

use EffKind::W2e3 as W2;
use W2Op::*;

pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    ("poison %player% [for %timespan%]", W2(Poison)),
    ("(cure|unpoison) %player% [of poison]", W2(Cure)),
    ("show [the] custom name of %player%", W2(NameVisible(true))),
    ("show %player%'s custom name", W2(NameVisible(true))),
    ("hide [the] custom name of %player%", W2(NameVisible(false))),
    ("hide %player%'s custom name", W2(NameVisible(false))),
    ("hide %player%", W2(Hide)),
    ("reveal %player%", W2(Reveal)),
    ("save [the] (world|worlds)", W2(SaveWorld)),
    ("save all worlds", W2(SaveWorld)),
    ("stop [the] server", W2(StopServer)),
    ("shut down [the] server", W2(StopServer)),
    ("shutdown [the] server", W2(StopServer)),
    ("make %player% start sprinting", W2(Sprint(true))),
    ("make %player% sprint", W2(Sprint(true))),
    ("make %player% stop sprinting", W2(Sprint(false))),
    ("make %player% swing [their] [main] hand", W2(Swing(true))),
    ("make %player% swing [their] off hand", W2(Swing(false))),
    ("make %player% swing", W2(Swing(true))),
    ("clear [the] title of %player%", W2(ClearTitle(false))),
    ("clear %player%'s title", W2(ClearTitle(false))),
    ("clear [the] title", W2(ClearTitle(false))),
    ("reset [the] title of %player%", W2(ClearTitle(true))),
    ("reset %player%'s title", W2(ClearTitle(true))),
    ("reset [the] title", W2(ClearTitle(true))),
    ("dismount %player%", W2(Dismount)),
    ("make %player% dismount", W2(Dismount)),
    ("eject %player%", W2(Eject)),
    ("eject [all] passengers of %player%", W2(Eject)),
    ("make %player% (ride|mount) %player%", W2(Ride)),
    (
        "send [the] resource pack %text% [with hash %text%] to %player%",
        W2(ResourcePack),
    ),
    ("(toggle|switch) %variable%", W2(Toggle)),
];

fn take(caps: &mut [Option<Capture>], i: usize) -> Option<Capture> {
    caps.get_mut(i).and_then(Option::take)
}

/// Build the effect from pattern captures.
pub fn build(op: W2Op, caps: &mut [Option<Capture>]) -> Option<Effect> {
    let mut args = Vec::new();
    let mut var = None;
    for (i, a) in op.layout().iter().enumerate() {
        match (a, take(caps, i)) {
            (Arg::Var, Some(Capture::Var(v))) => var = Some(v),
            (Arg::Var, _) => return None,
            (_, Some(Capture::Expr(e))) => args.push(Some(e)),
            (_, _) => args.push(None),
        }
    }
    // `clear title` with no target means the player of the event.
    if matches!(op, ClearTitle(_)) && args.first().is_some_and(Option::is_none) {
        args = vec![Some(Expr::Player)];
    }
    let first_needed = !matches!(op, SaveWorld | StopServer | Toggle);
    let missing = |i: usize| args.get(i).is_none_or(Option::is_none);
    if first_needed && missing(0) {
        return None;
    }
    if op == ResourcePack && missing(2) {
        return None;
    }
    if op == Ride && missing(1) {
        return None;
    }
    Some(Effect::W2e3(Box::new(W2Eff { op, args, var })))
}

/// `transform {list::*} with <expr using input>`. `None` means this line is
/// not a transform.
pub fn parse_transform(toks: &[Tok], env: &Env) -> Option<Result<Effect, String>> {
    if !toks.first().is_some_and(|t| t.is_lit("transform")) {
        return None;
    }
    let Some(Tok::Var(raw)) = toks.get(1) else {
        return None;
    };
    let list = match parse_varref(raw, env) {
        Ok(v) if v.is_list() => v,
        _ => return None,
    };
    if !toks.get(2).is_some_and(|t| t.is_lit("with")) {
        return Some(Err("use 'transform {list::*} with (input * 2)'".into()));
    }
    let rest: Vec<Tok> = toks
        .get(3..)?
        .iter()
        .map(|t| match t {
            Tok::Word(w) if w.eq_ignore_ascii_case("input") => Tok::Var(INPUT_KEY.into()),
            t => t.clone(),
        })
        .collect();
    Some(
        parse_expr(&rest, env)
            .map(|with| Effect::W2Transform(Box::new(Transform { list, with })))
            .map_err(|e| format!("can't understand the 'transform' expression: {e}")),
    )
}

/// Deterministic UUID text from a pack URL, so sending the same pack again
/// replaces it instead of stacking.
pub fn pack_uuid(url: &str) -> String {
    let fnv = |seed: u64| {
        url.bytes().fold(seed, |h, b| {
            (h ^ u64::from(b)).wrapping_mul(0x0000_0100_0000_01b3)
        })
    };
    let a = fnv(0xcbf2_9ce4_8422_2325);
    let b = fnv(0x8422_2325_cbf2_9ce4);
    // Version 4 and variant bits so parsers accept it.
    let a = (a & 0xffff_ffff_ffff_0fff) | 0x4000;
    let b = (b & 0x3fff_ffff_ffff_ffff) | 0x8000_0000_0000_0000;
    format!(
        "{:08x}-{:04x}-{:04x}-{:04x}-{:012x}",
        a >> 32,
        (a >> 16) & 0xffff,
        a & 0xffff,
        b >> 48,
        b & 0xffff_ffff_ffff
    )
}

/// Evaluated arguments: players and texts in the order the layout lists them.
#[derive(Default)]
pub struct Args {
    pub players: [Option<PlayerRef>; 2],
    pub span_ms: Option<i64>,
    pub texts: [Option<String>; 2],
}

/// Run an effect on evaluated arguments (`toggle` is handled by the engine).
pub fn run(op: W2Op, a: &Args, host: &mut dyn Host) {
    let p0 = a.players[0].as_ref();
    match op {
        Poison => {
            if let Some(p) = p0 {
                let secs = a.span_ms.map_or(1, |ms| ((ms.max(0) + 999) / 1000).max(1));
                host.poison(p, Some(secs as u64));
            }
        }
        Cure => {
            if let Some(p) = p0 {
                host.poison(p, None);
            }
        }
        Hide | Reveal => {
            let show = op == Reveal;
            let Some(target) = p0 else { return };
            let viewers = match a.players[1].clone() {
                Some(v) => vec![v],
                None => host.online_players(),
            };
            for v in viewers.iter().filter(|v| v.uuid != target.uuid) {
                host.set_visible_to(v, target, show);
            }
        }
        SaveWorld => host.save_world(),
        StopServer => host.stop_server(),
        Sprint(on) => {
            if let Some(p) = p0 {
                host.set_sprinting(p, on);
            }
        }
        Swing(main) => {
            if let Some(p) = p0 {
                host.swing_hand(p, main);
            }
        }
        ClearTitle(reset) => {
            if let Some(p) = p0 {
                host.clear_title(p, reset);
            }
        }
        Dismount => {
            if let Some(p) = p0 {
                host.dismount(p);
            }
        }
        Eject => {
            if let Some(p) = p0 {
                host.eject_passengers(p);
            }
        }
        Ride => {
            if let (Some(rider), Some(vehicle)) = (p0, a.players[1].as_ref())
                && rider.uuid != vehicle.uuid
            {
                host.ride(rider, vehicle);
            }
        }
        NameVisible(v) => {
            if let Some(p) = p0 {
                host.set_name_visible(p, v);
            }
        }
        ResourcePack => {
            if let (Some(p), Some(url)) = (p0, a.texts[0].as_deref()) {
                let hash = a.texts[1].as_deref().unwrap_or("");
                host.send_resource_pack(p, url, hash, &pack_uuid(url));
            }
        }
        Toggle => {}
    }
}

/// Default Host commands, kept here so `host.rs` stays small.
pub fn swing_command(name: &str, main: bool) -> String {
    format!("swing {name} {}", if main { "mainhand" } else { "offhand" })
}

pub fn title_command(name: &str, reset: bool) -> String {
    format!("title {name} {}", if reset { "reset" } else { "clear" })
}

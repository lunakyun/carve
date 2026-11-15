//! Skript syntax that real servers use and Carve was missing: number
//! comparisons in words (`is greater than`), `subtract`, `make console
//! execute`, `set message to` / `set chat format to` in `on chat`, `offhand
//! tool`, `x meters in front of`, `formatted`, and the lines that need another plugin or something Pumpkin can't do
//! (`region at`, `player's prefix`, `explosive yield`, tab
//! suggestions). Those load with a warning (see `docs/skript-parity.md`).
//!
//! The rare effects and expressions sit behind boxes (`Effect::Compat`,
//! `Expr::Compat`), so the hot enums do not grow.

use crate::compile::{CondKind, EffKind, Effect};
use crate::engine::{Engine, Exec};
use crate::error::compile_warn;
use crate::expr::{Expr, P, Ty};
use crate::host::Host;
use crate::lexer::Tok;
use crate::pattern::Capture;
use crate::value::Value;

/// `FireOutcome::set` slots written by `on chat` effects (value: text).
pub const SLOT_CHAT_MESSAGE: u8 = 238;
pub const SLOT_CHAT_FORMAT: u8 = 239;

/// What an effect of this module does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EOp {
    /// `set message to <text>` in `on chat`.
    SetMessage,
    /// `set chat format to <text>` in `on chat`.
    SetFormat,
    /// `replace all "a" in message with "b"`.
    ReplaceMessage,
    /// `set explosive yield of <entity> to 3`: the game has no such setting.
    Yield,
    /// `set arg-1's suggestions to ...`: the game has no custom completions.
    Suggest,
}

/// Registered with the effects, ahead of the main list.
pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    (
        "make console (execute|run) [command] %text%",
        EffKind::ConsoleCommand,
    ),
    ("subtract %object% from %variable%", EffKind::Remove),
    ("set [the] message to %text%", EffKind::Compat(EOp::SetMessage)),
    ("set [the] chat format to %text%", EffKind::Compat(EOp::SetFormat)),
    (
        "replace [all] %text% in [the] message with %text%",
        EffKind::Compat(EOp::ReplaceMessage),
    ),
    (
        "set [the] explosive yield of %object% to %number%",
        EffKind::Compat(EOp::Yield),
    ),
];

/// `x is greater than y` and the other word forms of `>`, `<`, `>=`, `<=`.
pub const CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    (
        "%number% (is|are) (greater|more|larger|bigger|higher) than or equal to %number%",
        CondKind::Ge,
    ),
    ("%number% (is|are) at least %number%", CondKind::Ge),
    (
        "%number% (is|are) (less|smaller|fewer|lower) than or equal to %number%",
        CondKind::Le,
    ),
    ("%number% (is|are) at most %number%", CondKind::Le),
    (
        "%number% (is|are) (greater|more|larger|bigger|higher) than %number%",
        CondKind::Gt,
    ),
    (
        "%number% (is|are) (less|smaller|fewer|lower) than %number%",
        CondKind::Lt,
    ),
    (
        "%number% (isn't|is not|aren't|are not) (greater|more|larger|bigger|higher) than %number%",
        CondKind::Le,
    ),
    (
        "%number% (isn't|is not|aren't|are not) (less|smaller|fewer|lower) than %number%",
        CondKind::Ge,
    ),
];

#[derive(Debug, Clone, PartialEq)]
pub struct Fx {
    pub op: EOp,
    pub a: Option<Expr>,
    pub b: Option<Expr>,
}

fn cap(c: &mut [Option<Capture>], i: usize) -> Option<Expr> {
    match c.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    }
}

/// Build the effect of a pattern. Warns for the ones the server can't carry out.
pub fn build_effect(op: EOp, c: &mut [Option<Capture>]) -> Option<Effect> {
    let (a, b) = match op {
        EOp::SetMessage | EOp::SetFormat => (cap(c, 0)?, None),
        EOp::ReplaceMessage => (cap(c, 0)?, cap(c, 1)),
        EOp::Yield => (cap(c, 0)?, cap(c, 1)),
        EOp::Suggest => return None,
    };
    if op == EOp::Yield {
        compile_warn(
            "'explosive yield' does nothing on this server: Pumpkin has no explosion power setting for entities",
        );
    }
    Some(Effect::Compat(Box::new(Fx { op, a: Some(a), b })))
}

impl Engine {
    #[cold]
    pub(crate) fn compat_effect(&mut self, f: &Fx, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        match f.op {
            EOp::SetMessage | EOp::SetFormat => {
                let Some(e) = &f.a else { return Ok(()) };
                let t = self.text(e, ex, host)?;
                if f.op == EOp::SetMessage {
                    ex.ctx.message = Some(t.clone());
                    self.ext_out.push((SLOT_CHAT_MESSAGE, Value::Text(t)));
                } else {
                    self.ext_out.push((SLOT_CHAT_FORMAT, Value::Text(t)));
                }
            }
            EOp::ReplaceMessage => {
                let (Some(from), Some(to)) = (&f.a, &f.b) else {
                    return Ok(());
                };
                let from = self.text(from, ex, host)?;
                let to = self.text(to, ex, host)?;
                let Some(cur) = ex.ctx.message.clone() else {
                    return Ok(());
                };
                if from.is_empty() {
                    return Ok(());
                }
                let t = cur.replace(&from, &to);
                ex.ctx.message = Some(t.clone());
                self.ext_out.push((SLOT_CHAT_MESSAGE, Value::Text(t)));
            }
            // Parsed and checked at load, with a warning there; nothing to do.
            EOp::Yield | EOp::Suggest => {}
        }
        Ok(())
    }
}

// ---------------------------------------------------------------- expressions

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// `region at <place>`: needs WorldGuard; always `<none>`.
    RegionAt,
    /// `<player>'s prefix`: needs a chat plugin; always empty.
    Prefix,
    Suffix,
    /// `<player>'s offhand tool`.
    Offhand,
    /// `current input` in a command's `suggestions:`.
    CurrentInput,
    /// `random element out of "a" and "b"`: args: the list.
    RandomOf,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub op: Op,
    pub args: Vec<Expr>,
}

impl Node {
    pub fn ty(&self) -> Ty {
        match self.op {
            Op::RegionAt | Op::RandomOf => Ty::Object,
            Op::Prefix | Op::Suffix | Op::CurrentInput => Ty::Text,
            Op::Offhand => Ty::Item,
        }
    }
}

fn node(op: Op, args: Vec<Expr>) -> Expr {
    Expr::Compat(Box::new(Node { op, args }))
}

impl Engine {
    #[cold]
    pub(crate) fn eval_compat(
        &mut self,
        n: &Node,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        Ok(match n.op {
            Op::RegionAt => Value::None,
            Op::RandomOf => {
                let Some(a) = n.args.first() else {
                    return Ok(Value::None);
                };
                let items = match self.eval(a, ex, host)? {
                    Value::List(l) => l,
                    Value::None => Vec::new(),
                    v => vec![v],
                };
                if items.is_empty() {
                    Value::None
                } else {
                    let i = ((self.next_random(host) * items.len() as f64) as usize).min(items.len() - 1);
                    items.into_iter().nth(i).unwrap_or(Value::None)
                }
            }
            Op::Prefix | Op::Suffix | Op::CurrentInput => Value::Text(String::new()),
            Op::Offhand => {
                let Some(a) = n.args.first() else {
                    return Ok(Value::None);
                };
                let v = self.eval(a, ex, host)?;
                match crate::engine::online(&v, &mut ex.seen, host) {
                    Some(p) => match host.inv_get(&p, false, 40) {
                        Some((id, count)) => Value::Item {
                            id,
                            count,
                            meta: None,
                        },
                        None => Value::Item {
                            id: "air".into(),
                            count: 1,
                            meta: None,
                        },
                    },
                    None => Value::None,
                }
            }
        })
    }
}

impl P<'_> {
    fn compat_lit(&self, off: usize, w: &str) -> bool {
        if self.pos + off >= self.toks.len() {
            self.hit_end.set(true);
        }
        matches!(self.toks.get(self.pos + off), Some(t) if t.is_lit(w))
    }

    /// `<player>'s prefix`, `'s offhand tool`: after the `'s`, at `lw`. Returns
    /// the new expression and the tokens used after the `'s`.
    pub(crate) fn compat_poss(&mut self, lw: &str, e: &Expr) -> Option<(Expr, usize)> {
        match lw {
            "prefix" | "suffix" => {
                compile_warn(format!(
                    "'{lw}' needs a chat or permission plugin: it is always empty on this server"
                ));
                let op = if lw == "prefix" { Op::Prefix } else { Op::Suffix };
                Some((node(op, vec![e.clone()]), 1))
            }
            "offhand" | "off" => {
                let skip = usize::from(lw == "off" && self.compat_lit(2, "hand"));
                let w = self.toks.get(self.pos + 2 + skip);
                if matches!(w, Some(t) if t.is_lit("tool") || t.is_lit("item")) {
                    Some((node(Op::Offhand, vec![e.clone()]), 2 + skip))
                } else {
                    None
                }
            }
            _ => None,
        }
    }

    /// `a` / `an` before an item (`an egg`), `a random number`: the last thing tried.
    pub(crate) fn compat_article(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        if lw != "a" && lw != "an" {
            return Ok(None);
        }
        let Some(Tok::Word(w)) = self.toks.get(self.pos).cloned() else {
            return Ok(None);
        };
        if w.eq_ignore_ascii_case("random") {
            return Ok(Some(self.primary()?));
        }
        let save = self.pos;
        self.pos += 1;
        match self.item_after_article(&w) {
            Some(e) => Ok(Some(e)),
            None => {
                self.pos = save;
                Ok(None)
            }
        }
    }

    /// `location 3 meters in front of x`: tried before the other heads.
    pub(crate) fn compat_early(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        // `a random number between 1 and 5`, `a random element out of x`
        if (lw == "a" || lw == "an") && self.compat_lit(0, "random") {
            return Ok(Some(self.primary()?));
        }
        if lw != "location" {
            return Ok(None);
        }
        let Some(Tok::Num(n)) = self.toks.get(self.pos).cloned() else {
            return Ok(None);
        };
        self.pos += 1;
        match self.textdir_offset(n)? {
            Some(e) => Ok(Some(e)),
            None => Err("expected '<n> meters in front of <place>' after 'location'".into()),
        }
    }

    /// Expressions of this module by their first word. `Ok(None)`: not ours.
    pub(crate) fn compat_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let e = match lw {
            "region" | "regions" if self.compat_lit(0, "at") => {
                self.pos += 1;
                let at = self.primary()?;
                compile_warn("'region at' needs WorldGuard: it is always <none> on this server");
                node(Op::RegionAt, vec![at])
            }
            "random"
                if self.compat_lit(0, "element") && self.compat_lit(1, "out") && self.compat_lit(2, "of") =>
            {
                self.pos += 3;
                if self.pos >= self.toks.len() {
                    self.hit_end.set(true);
                }
                let (first, mut end) = crate::list_ops::parse_src_at(self.toks, self.pos, self.env, false)?;
                let mut parts = vec![first];
                let has_and = self.toks.iter().skip(end).any(|t| t.is_lit("and"));
                while self
                    .toks
                    .get(end)
                    .is_some_and(|t| t.is_lit("and") || (t.is_lit(",") && has_and))
                {
                    if end + 1 >= self.toks.len() {
                        self.hit_end.set(true);
                    }
                    let (s, e) = crate::list_ops::parse_src_at(self.toks, end + 1, self.env, false)?;
                    parts.push(s);
                    end = e;
                }
                self.pos = end;
                let src = if parts.len() == 1 {
                    parts
                        .pop()
                        .unwrap_or(crate::list_ops::ListSrc::Concat(Vec::new()))
                } else {
                    crate::list_ops::ListSrc::Concat(parts)
                };
                node(Op::RandomOf, vec![Expr::Items(Box::new(src))])
            }
            // `formatted message`, `formatted "&aHi"` (`formatted now as ...` is a date).
            "formatted"
                if self.compat_lit(0, "message") || matches!(self.toks.get(self.pos), Some(Tok::Str(_))) =>
            {
                let arg = self.unary()?;
                Expr::Color(true, Box::new(arg))
            }
            "tab" if matches!(self.toks.get(self.pos), Some(Tok::Word(w)) if w.to_ascii_lowercase().starts_with("arg")) => {
                self.primary()?
            }
            "current" if self.compat_lit(0, "input") => {
                self.pos += 1;
                node(Op::CurrentInput, Vec::new())
            }
            "prefix" | "suffix" if self.compat_lit(0, "of") && !self.compat_lit(1, "team") => {
                self.pos += 1;
                let who = self.primary()?;
                compile_warn(format!(
                    "'{lw}' needs a chat or permission plugin: it is always empty on this server"
                ));
                node(if lw == "prefix" { Op::Prefix } else { Op::Suffix }, vec![who])
            }
            "offhand" if self.compat_lit(0, "tool") || self.compat_lit(0, "item") => {
                if !self.compat_lit(1, "of") {
                    return Err("use 'offhand tool of <player>' or '<player>'s offhand tool'".into());
                }
                self.pos += 2;
                let who = self.primary()?;
                node(Op::Offhand, vec![who])
            }
            _ => return Ok(None),
        };
        Ok(Some(e))
    }
}

// ---------------------------------------------------------------- hand-parsed lines

/// `set arg-1's suggestions to ...` (checked, then never run) and
/// `add 1 to y-coordinate of {_loc}` (and `remove`, `subtract`, `set`).
pub fn parse_effect(toks: &[Tok], env: &crate::expr::Env) -> Result<Option<Effect>, String> {
    use crate::expr::{BinOp, VarRef, parse_expr, parse_varref};
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    let first = first.to_ascii_lowercase();
    // `set arg-1's suggestions to "a", "b" and "c"`
    if first == "set"
        && matches!(toks.get(1), Some(Tok::Word(w)) if w.to_ascii_lowercase().starts_with("arg"))
        && toks.get(2).is_some_and(|t| t.is_lit("'s"))
        && toks
            .get(3)
            .is_some_and(|t| t.is_lit("suggestions") || t.is_lit("suggestion"))
        && toks.get(4).is_some_and(|t| t.is_lit("to"))
    {
        let rest = toks.get(5..).unwrap_or_default();
        crate::list_ops::parse_src_all(rest, env, true)
            .map_err(|e| format!("can't understand the suggestions: {e}"))?;
        compile_warn(
            "tab suggestions do nothing on this server: the plugin API can't change command completions",
        );
        return Ok(Some(Effect::Compat(Box::new(Fx {
            op: EOp::Suggest,
            a: None,
            b: None,
        }))));
    }
    // Coordinates of a location variable.
    let (axis_at, value, sign): (usize, Option<&[Tok]>, i8) = match first.as_str() {
        "add" | "remove" | "subtract" => {
            let sep = if first == "add" { "to" } else { "from" };
            let Some(i) = toks.iter().position(|t| t.is_lit(sep)) else {
                return Ok(None);
            };
            (i + 1, toks.get(1..i), if first == "add" { 1 } else { -1 })
        }
        "set" => {
            let Some(i) = toks.iter().position(|t| t.is_lit("to")) else {
                return Ok(None);
            };
            (1, toks.get(i + 1..), 0)
        }
        _ => return Ok(None),
    };
    let axis = match toks.get(axis_at) {
        Some(Tok::Word(w)) => match w.to_ascii_lowercase().as_str() {
            "x" => 0,
            "y" => 1,
            "z" => 2,
            _ => return Ok(None),
        },
        _ => return Ok(None),
    };
    let mut i = axis_at + 1;
    if toks.get(i).is_some_and(|t| t.is_lit("-")) {
        i += 1;
    }
    if !(toks.get(i).is_some_and(|t| t.is_lit("coordinate"))
        && toks.get(i + 1).is_some_and(|t| t.is_lit("of")))
    {
        return Ok(None);
    }
    let var_end = if first == "set" {
        toks.iter().position(|t| t.is_lit("to")).unwrap_or(toks.len())
    } else {
        toks.len()
    };
    let Some([Tok::Var(raw)]) = toks.get(i + 2..var_end) else {
        return Ok(None);
    };
    let var: VarRef = parse_varref(raw, env)?;
    if var.is_list() {
        return Ok(None);
    }
    let num = parse_expr(value.unwrap_or_default(), env)?;
    let comp = |ax: u8, prop: crate::expr::Prop| -> Expr {
        let cur = Expr::Prop(prop, Box::new(Expr::Var(var.clone())));
        if ax != axis {
            return cur;
        }
        match sign {
            0 => num.clone(),
            1 => Expr::Bin(Box::new(cur), BinOp::Add, Box::new(num.clone())),
            _ => Expr::Bin(Box::new(cur), BinOp::Sub, Box::new(num.clone())),
        }
    };
    let loc = Expr::Loc(
        Box::new(comp(0, crate::expr::Prop::X)),
        Box::new(comp(1, crate::expr::Prop::Y)),
        Box::new(comp(2, crate::expr::Prop::Z)),
        Some(Box::new(Expr::Prop(
            crate::expr::Prop::World,
            Box::new(Expr::Var(var.clone())),
        ))),
    );
    Ok(Some(Effect::Set(var, loc)))
}

/// `damage cause is entity explosion or primed tnt` (and `is not`): each name
/// is matched with Skript's cause names, not by plain text equality.
pub fn damage_cause_cond(toks: &[Tok], env: &crate::expr::Env) -> Option<crate::compile::Cond> {
    use crate::compile::Cond;
    use crate::conditions_flags_metadata::{FlagsCond, Kind};
    if !env.has_damage || env.in_function {
        return None;
    }
    if !(toks.first()?.is_lit("damage") && toks.get(1)?.is_lit("cause")) {
        return None;
    }
    let (negate, rest_at) = match toks.get(2)? {
        t if t.is_lit("is") && toks.get(3).is_some_and(|n| n.is_lit("not")) => (true, 4),
        t if t.is_lit("isn't") => (true, 3),
        t if t.is_lit("is") || t.is_lit("=") => (false, 3),
        _ => return None,
    };
    let rest = toks.get(rest_at..)?;
    let mut conds = Vec::new();
    for part in rest.split(|t| t.is_lit("or") || t.is_lit(",")) {
        let e = crate::expr::parse_expr(part, env).ok()?;
        conds.push(Cond::FlagsMetadata(Box::new(FlagsCond {
            kind: Kind::DamageCause,
            negate,
            a: None,
            b: Some(e),
        })));
    }
    if conds.is_empty() {
        return None;
    }
    if conds.len() == 1 {
        return conds.pop();
    }
    Some(if negate {
        Cond::All(conds)
    } else {
        Cond::Any(conds)
    })
}

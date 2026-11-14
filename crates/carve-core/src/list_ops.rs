//! List operations: `sorted`, `reversed`, `shuffled`, `where` filters,
//! `random N elements of`, `sum of`, `min of`, `max of`, and the `sort` effect.
//!
//! Lists are variables like `{x::*}`, so a "list expression" is a small tree
//! ([`ListSrc`]) that the engine turns into a `Vec<Value>` when it runs.

use crate::compile::{CmpOp, Cond, Effect, TextOp};
use crate::expr::{Env, Expr, Ty, VarRef, parse_expr, parse_sum_at, parse_varref};
use crate::lexer::Tok;

/// Local variable that holds `input` while a `where` filter runs. The
/// control character keeps scripts from clashing with it.
pub const INPUT_KEY: &str = "_\u{1}input";

#[derive(Debug, Clone, PartialEq)]
pub enum ListSrc {
    /// `{x::*}`
    Var(VarRef),
    /// One expression. A function result that is a list is spread out.
    One(Expr),
    /// `a, b and c` where each part is a list or a value.
    Concat(Vec<ListSrc>),
    Sorted(Box<ListSrc>),
    /// `alphabetically sorted list`: everything compared as text, ignoring case.
    SortedAlpha(Box<ListSrc>),
    Reversed(Box<ListSrc>),
    Shuffled(Box<ListSrc>),
    /// `list where [input > 2]`
    Filter(Box<ListSrc>, Box<Cond>),
    /// `random 2 elements of list`
    Random(Box<Expr>, Box<ListSrc>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Agg {
    Sum,
    Avg,
    Min,
    Max,
    Size,
}

fn word(t: Option<&Tok>) -> Option<String> {
    match t {
        Some(Tok::Word(w)) => Some(w.to_ascii_lowercase()),
        _ => None,
    }
}

struct Cur<'a> {
    toks: &'a [Tok],
    env: &'a Env,
    /// Allow `a, b and c` at this level.
    commas: bool,
}

/// Parse a list expression starting at `pos`. Returns it and the position
/// after it. With `commas`, `a, b and c` joins several parts.
pub fn parse_src_at(toks: &[Tok], pos: usize, env: &Env, commas: bool) -> Result<(ListSrc, usize), String> {
    Cur { toks, env, commas }.concat(pos)
}

/// Like [`parse_src_at`] but the whole token list must be the list.
pub fn parse_src_all(toks: &[Tok], env: &Env, commas: bool) -> Result<ListSrc, String> {
    let (s, end) = parse_src_at(toks, 0, env, commas)?;
    match toks.get(end) {
        None => Ok(s),
        Some(t) => Err(format!("unexpected {t:?} after the list")),
    }
}

/// Does this look like a list operation (not a plain `a, b and c`)?
pub fn looks_like_list_op(toks: &[Tok], env: &Env) -> bool {
    if let Some(Tok::Word(w)) = toks.first() {
        let lw = w.to_ascii_lowercase();
        if matches!(lw.as_str(), "sorted" | "reversed" | "shuffled" | "alphabetically" | "alphabetical") {
            return true;
        }
        if lw == "all" && toks.get(1).is_some_and(|t| t.is_lit("scripts")) && toks.len() == 2 {
            return true;
        }
        if lw == "random" && matches!(toks.get(1), Some(Tok::Num(_) | Tok::Var(_))) {
            return true;
        }
        if matches!(toks.get(1), Some(t) if t.is_lit("("))
            && env
                .functions
                .iter()
                .any(|s| s.name.eq_ignore_ascii_case(w) && s.ret == Some(Ty::List))
        {
            return true;
        }
    }
    crate::w2_expressions_2::starts_list(toks) || toks.iter().any(|t| t.is_lit("where"))
}

impl Cur<'_> {
    fn concat(&self, mut pos: usize) -> Result<(ListSrc, usize), String> {
        let (first, np) = self.item(pos)?;
        pos = np;
        if !self.commas {
            return Ok((first, pos));
        }
        let mut parts = vec![first];
        while self.toks.get(pos).is_some_and(|t| t.is_lit(",") || t.is_lit("and")) {
            let (s, np) = self.item(pos + 1)?;
            parts.push(s);
            pos = np;
        }
        if parts.len() == 1
            && let Some(only) = parts.pop()
        {
            return Ok((only, pos));
        }
        Ok((ListSrc::Concat(parts), pos))
    }

    fn item(&self, pos: usize) -> Result<(ListSrc, usize), String> {
        let (mut s, mut pos) = self.base(pos)?;
        while self.toks.get(pos).is_some_and(|t| t.is_lit("where")) {
            let (c, np) = self.pred(pos + 1)?;
            s = ListSrc::Filter(Box::new(s), Box::new(c));
            pos = np;
        }
        Ok((s, pos))
    }

    fn one(&self, pos: usize) -> Result<(ListSrc, usize), String> {
        let (e, np) = parse_sum_at(self.toks, pos, self.env)?;
        Ok((ListSrc::One(e), np))
    }

    fn base(&self, pos: usize) -> Result<(ListSrc, usize), String> {
        match self.toks.get(pos) {
            None => Err("missing list".into()),
            Some(Tok::Var(raw)) => {
                let v = parse_varref(raw, self.env)?;
                if v.is_list() {
                    Ok((ListSrc::Var(v), pos + 1))
                } else {
                    self.one(pos)
                }
            }
            Some(Tok::Sym("(")) => {
                let inner = Cur {
                    toks: self.toks,
                    env: self.env,
                    commas: true,
                };
                if let Ok((s, np)) = inner.concat(pos + 1)
                    && self.toks.get(np).is_some_and(|t| t.is_lit(")"))
                {
                    return Ok((s, np + 1));
                }
                self.one(pos)
            }
            Some(Tok::Word(w)) => match w.to_ascii_lowercase().as_str() {
                "alphabetically" | "alphabetical"
                    if word(self.toks.get(pos + 1)).as_deref() == Some("sorted") =>
                {
                    let mut p = pos + 2;
                    if word(self.toks.get(p)).as_deref() == Some("list") {
                        p += 1;
                    }
                    let (inner, np) = self.concat(p)?;
                    Ok((ListSrc::SortedAlpha(Box::new(inner)), np))
                }
                lw @ ("sorted" | "reversed" | "shuffled") => {
                    let mut p = pos + 1;
                    if word(self.toks.get(p)).as_deref() == Some("list") {
                        p += 1;
                    }
                    let (inner, np) = self.concat(p)?;
                    let b = Box::new(inner);
                    Ok((
                        match lw {
                            "sorted" => ListSrc::Sorted(b),
                            "reversed" => ListSrc::Reversed(b),
                            _ => ListSrc::Shuffled(b),
                        },
                        np,
                    ))
                }
                "random" if matches!(self.toks.get(pos + 1), Some(Tok::Num(_) | Tok::Var(_))) => {
                    let n = match self.toks.get(pos + 1) {
                        Some(Tok::Num(n)) => Expr::Num(*n),
                        Some(Tok::Var(raw)) => Expr::Var(parse_varref(raw, self.env)?),
                        _ => return Err("expected a number after 'random'".into()),
                    };
                    let mut p = pos + 2;
                    if !matches!(
                        word(self.toks.get(p)).as_deref(),
                        Some("elements" | "items" | "element" | "item")
                    ) {
                        return Err("expected 'random N elements of {list::*}'".into());
                    }
                    p += 1;
                    if word(self.toks.get(p)).as_deref() == Some("out") {
                        p += 1;
                    }
                    if word(self.toks.get(p)).as_deref() != Some("of") {
                        return Err("expected 'random N elements of {list::*}'".into());
                    }
                    let (inner, np) = self.item(p + 1)?;
                    Ok((ListSrc::Random(Box::new(n), Box::new(inner)), np))
                }
                _ => self.one(pos),
            },
            Some(_) => self.one(pos),
        }
    }

    /// The condition after `where`: `[input > 2]`, or unbracketed up to the end.
    fn pred(&self, pos: usize) -> Result<(Cond, usize), String> {
        let (inner, np) = if self.toks.get(pos).is_some_and(|t| t.is_lit("[")) {
            let mut depth = 0i32;
            let mut close = None;
            for (i, t) in self.toks.iter().enumerate().skip(pos) {
                if t.is_lit("[") {
                    depth += 1;
                } else if t.is_lit("]") {
                    depth -= 1;
                    if depth == 0 {
                        close = Some(i);
                        break;
                    }
                }
            }
            let close = close.ok_or("missing ']' after 'where ['")?;
            (self.toks.get(pos + 1..close).unwrap_or(&[]), close + 1)
        } else {
            let mut depth = 0i32;
            let mut end = self.toks.len();
            for (i, t) in self.toks.iter().enumerate().skip(pos) {
                if t.is_lit("(") {
                    depth += 1;
                } else if t.is_lit(")") {
                    depth -= 1;
                    if depth < 0 {
                        end = i;
                        break;
                    }
                }
            }
            (self.toks.get(pos..end).unwrap_or(&[]), end)
        };
        Ok((parse_filter(inner, self.env)?, np))
    }
}

/// A `where` condition. `input` is the item being tested.
fn parse_filter(toks: &[Tok], env: &Env) -> Result<Cond, String> {
    if toks.is_empty() {
        return Err("empty 'where' condition".into());
    }
    let toks: Vec<Tok> = toks
        .iter()
        .map(|t| match t {
            Tok::Word(w) if w.eq_ignore_ascii_case("input") => Tok::Var(INPUT_KEY.into()),
            t => t.clone(),
        })
        .collect();
    let mut ors = Vec::new();
    for or_part in toks.split(|t| t.is_lit("or")) {
        let mut ands = Vec::new();
        for and_part in or_part.split(|t| t.is_lit("and")) {
            ands.push(filter_atom(and_part, env)?);
        }
        ors.push(if ands.len() == 1 {
            ands.remove(0)
        } else {
            Cond::All(ands)
        });
    }
    Ok(if ors.len() == 1 { ors.remove(0) } else { Cond::Any(ors) })
}

fn filter_atom(toks: &[Tok], env: &Env) -> Result<Cond, String> {
    let bad = || {
        "can't understand the 'where' condition (try 'input > 2', 'input is \"a\"', 'input contains \"a\"')"
            .to_string()
    };
    let expr = |t: &[Tok]| parse_expr(t, env);
    let sym = toks
        .iter()
        .position(|t| matches!(t, Tok::Sym(">=" | "<=" | "!=" | ">" | "<" | "=")));
    if let Some(i) = sym {
        let op = match toks.get(i) {
            Some(Tok::Sym(">=")) => CmpOp::Ge,
            Some(Tok::Sym("<=")) => CmpOp::Le,
            Some(Tok::Sym("!=")) => CmpOp::Ne,
            Some(Tok::Sym(">")) => CmpOp::Gt,
            Some(Tok::Sym("<")) => CmpOp::Lt,
            _ => CmpOp::Eq,
        };
        let l = expr(toks.get(..i).ok_or_else(bad)?)?;
        let r = expr(toks.get(i + 1..).ok_or_else(bad)?)?;
        return Ok(Cond::Cmp(l, op, r));
    }
    let text_op = |name: &str| match name {
        "contains" | "contain" => Some((TextOp::Contains, 1)),
        "starts" => Some((TextOp::StartsWith, 2)),
        "ends" => Some((TextOp::EndsWith, 2)),
        _ => None,
    };
    if let Some((i, (op, skip))) = toks
        .iter()
        .enumerate()
        .find_map(|(i, t)| word(Some(t)).and_then(|w| text_op(&w)).map(|o| (i, o)))
    {
        let mut left = toks.get(..i).ok_or_else(bad)?;
        let mut negate = false;
        if left.last().is_some_and(|t| t.is_lit("not")) {
            negate = true;
            left = left.get(..left.len() - 1).unwrap_or(left);
        }
        if left.last().is_some_and(|t| t.is_lit("does")) {
            left = left.get(..left.len() - 1).unwrap_or(left);
        }
        return Ok(Cond::Text {
            left: expr(left)?,
            op,
            right: expr(toks.get(i + skip..).ok_or_else(bad)?)?,
            negate,
        });
    }
    let is = toks.iter().position(|t| t.is_lit("is")).ok_or_else(bad)?;
    let left = expr(toks.get(..is).ok_or_else(bad)?)?;
    let mut rest = toks.get(is + 1..).ok_or_else(bad)?;
    let mut neg = false;
    if rest.first().is_some_and(|t| t.is_lit("not")) {
        neg = true;
        rest = rest.get(1..).unwrap_or(&[]);
    }
    let w0 = word(rest.first()).unwrap_or_default();
    let w1 = word(rest.get(1)).unwrap_or_default();
    let cmp = |op: CmpOp, skip: usize| -> Result<Cond, String> {
        let r = expr(rest.get(skip..).ok_or_else(bad)?)?;
        Ok(Cond::Cmp(left.clone(), op, r))
    };
    match (w0.as_str(), w1.as_str()) {
        ("greater" | "more" | "bigger" | "larger", "than") => cmp(if neg { CmpOp::Le } else { CmpOp::Gt }, 2),
        ("less" | "smaller" | "fewer", "than") => cmp(if neg { CmpOp::Ge } else { CmpOp::Lt }, 2),
        ("at", "least") => cmp(if neg { CmpOp::Lt } else { CmpOp::Ge }, 2),
        ("at", "most") => cmp(if neg { CmpOp::Gt } else { CmpOp::Le }, 2),
        ("above", _) => cmp(if neg { CmpOp::Le } else { CmpOp::Gt }, 1),
        ("below", _) => cmp(if neg { CmpOp::Ge } else { CmpOp::Lt }, 1),
        ("set", "") => Ok(Cond::IsSet(left, !neg)),
        _ => cmp(if neg { CmpOp::Ne } else { CmpOp::Eq }, 0),
    }
}

/// `set {x::*} to <list>` and `sort {x::*} [in descending order]`.
/// `None` means this line is not one of them.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Option<Result<Effect, String>> {
    let first = word(toks.first())?;
    let Some(Tok::Var(raw)) = toks.get(1) else {
        return None;
    };
    let v = match parse_varref(raw, env) {
        Ok(v) if v.is_list() => v,
        _ => return None,
    };
    match first.as_str() {
        "set" if toks.get(2).is_some_and(|t| t.is_lit("to")) => {
            Some(parse_src_all(toks.get(3..)?, env, true).map(|s| Effect::SetList(v, s)))
        }
        "sort" => {
            let tail: Vec<String> = toks.get(2..)?.iter().filter_map(|t| word(Some(t))).collect();
            if tail.len() != toks.len() - 2 {
                return None;
            }
            let tail: Vec<&str> = tail
                .iter()
                .map(String::as_str)
                .filter(|w| *w != "in" && *w != "order")
                .collect();
            match tail.as_slice() {
                [] | ["ascending"] => Some(Ok(Effect::SortList(v, false))),
                ["descending"] => Some(Ok(Effect::SortList(v, true))),
                _ => Some(Err(
                    "use 'sort {list::*}' or 'sort {list::*} in descending order'".into(),
                )),
            }
        }
        _ => None,
    }
}

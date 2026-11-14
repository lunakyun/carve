//! List literals: `1, 2 and 3`, and the effects and expressions that use them.

use crate::compile::Effect;
use crate::expr::{Env, Expr, VarRef, parse_expr, parse_varref};
use crate::lexer::Tok;

/// Which element of a list an `Expr::Elem` picks.
#[derive(Debug, Clone, PartialEq)]
pub enum ElemSel {
    First,
    Last,
    /// `element N of {x::*}`, counting from 1.
    Nth(Box<Expr>),
}

fn is_sep(t: &Tok) -> bool {
    t.is_lit(",") || t.is_lit("and") || t.is_lit("or")
}

/// Split `a, b and c` into expressions. A span that parses as one expression
/// stays whole, so `random integer between 1 and 6` is one item.
pub fn split_items(toks: &[Tok], env: &Env) -> Option<Vec<Expr>> {
    if toks.is_empty() {
        return None;
    }
    if let Ok(e) = parse_expr(toks, env) {
        return Some(vec![e]);
    }
    for i in 1..toks.len().saturating_sub(1) {
        if !is_sep(&toks[i]) {
            continue;
        }
        let Ok(head) = parse_expr(&toks[..i], env) else {
            continue;
        };
        // `a, b and c`: an `and`/`or` right after a comma is part of the list.
        let rest = if toks[i].is_lit(",") && (toks[i + 1].is_lit("and") || toks[i + 1].is_lit("or")) {
            &toks[i + 2..]
        } else {
            &toks[i + 1..]
        };
        if let Some(mut tail) = split_items(rest, env) {
            tail.insert(0, head);
            return Some(tail);
        }
    }
    None
}

/// Does this expression stand for many values (a whole list)?
pub fn is_many(e: &Expr) -> bool {
    matches!(e, Expr::Var(v) if v.list) || matches!(e, Expr::Indices(_))
}

fn list_var(tok: Option<&Tok>, env: &Env) -> Option<VarRef> {
    match tok {
        Some(Tok::Var(raw)) => parse_varref(raw, env).ok().filter(VarRef::is_list),
        _ => None,
    }
}

/// `set {x::*} to a, b and c`, `add a, b and c to {x::*}` and
/// `remove a, b from {x::*}`. `None` when the line is not one of these (or
/// is a plain single-item add/remove, which the normal patterns handle).
pub fn parse_list_effect(toks: &[Tok], env: &Env) -> Option<Result<Effect, String>> {
    let first = toks.first()?;
    if first.is_lit("set") {
        let v = list_var(toks.get(1), env)?;
        if !toks.get(2)?.is_lit("to") {
            return None;
        }
        return Some(match split_items(toks.get(3..)?, env) {
            Some(mut items) if items.len() == 1 && matches!(items[0], Expr::Split(..)) => {
                let Some(Expr::Split(a, b)) = items.pop() else {
                    unreachable!()
                };
                Ok(Effect::SetSplit(v, *a, *b))
            }
            Some(items) => Ok(Effect::ListSet(v, items)),
            None => Err("can't understand the list items (use 'set {x::*} to a, b and c')".into()),
        });
    }
    let (word, add) = if first.is_lit("add") {
        ("to", true)
    } else if first.is_lit("remove") {
        ("from", false)
    } else {
        return None;
    };
    let n = toks.len();
    if n < 4 || !toks.get(n - 2)?.is_lit(word) {
        return None;
    }
    let v = list_var(toks.get(n - 1), env)?;
    let items = split_items(toks.get(1..n - 2)?, env)?;
    if items.len() == 1 && !items.first().is_some_and(is_many) {
        return None;
    }
    Some(Ok(if add {
        Effect::ListAddAll(v, items)
    } else {
        Effect::ListRemoveAll(v, items)
    }))
}

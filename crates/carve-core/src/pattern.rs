//! Patterns are data: strings like `send %text% [to %player%]` compiled into
//! node lists, expanded into plain sequences, and matched with backtracking.

use std::collections::HashMap;

use crate::expr::{Env, Expr, Ty, VarRef, parse_expr_span, parse_varref};
use crate::lexer::Tok;

/// The apostrophe, for `doesn't` and possessive `'s` in patterns.
const APOS: char = 0x27 as char;

/// Parse results for one line, shared by every pattern tried on that line.
/// Backtracking asks for the same token span many times; this parses each
/// span once.
#[derive(Default)]
pub struct Memo {
    exprs: HashMap<(usize, usize), Option<Expr>>,
    vars: HashMap<usize, Option<VarRef>>,
    /// start -> first end from which no longer span can parse.
    dead: HashMap<usize, usize>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SlotTy {
    Ty(Ty),
    Variable,
}

#[derive(Debug, Clone, PartialEq)]
enum Node {
    Lit(String),
    Slot(SlotTy, usize),
    Opt(Vec<Node>),
    Choice(Vec<Vec<Node>>),
}

#[derive(Debug, Clone, PartialEq)]
enum Flat {
    Lit(String),
    Slot(SlotTy, usize),
}

#[derive(Debug, Clone)]
pub struct Pattern<K> {
    pub src: &'static str,
    pub kind: K,
    pub slots: usize,
    seqs: Vec<Vec<Flat>>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Capture {
    Expr(Expr),
    Var(VarRef),
}

impl<K: Copy> Pattern<K> {
    pub fn new(src: &'static str, kind: K) -> Result<Self, String> {
        let chars: Vec<char> = src.chars().collect();
        let mut pos = 0;
        let mut slots = 0;
        let nodes = parse_seq(&chars, &mut pos, &mut slots, None)?;
        if pos != chars.len() {
            return Err(format!("pattern '{src}': unexpected '{}'", chars[pos]));
        }
        let seqs = expand(&nodes);
        Ok(Pattern {
            src,
            kind,
            slots,
            seqs,
        })
    }

    /// Try to match tokens. On success returns captures indexed by slot.
    /// On failure returns a type error if the words matched but a value had the
    /// wrong type.
    pub fn matches(&self, toks: &[Tok], env: &Env) -> Result<Vec<Option<Capture>>, Option<String>> {
        self.matches_memo(toks, env, &mut Memo::default())
    }

    /// [`Pattern::matches`] with a parse cache shared across patterns.
    pub fn matches_memo(
        &self,
        toks: &[Tok],
        env: &Env,
        memo: &mut Memo,
    ) -> Result<Vec<Option<Capture>>, Option<String>> {
        let mut type_err = None;
        for seq in &self.seqs {
            // A literal must be one of the tokens; skip sequences that can't match.
            let impossible = seq
                .iter()
                .any(|f| matches!(f, Flat::Lit(l) if !toks.iter().any(|t| t.is_lit(l))));
            if impossible {
                continue;
            }
            let mut caps: Vec<Option<Capture>> = vec![None; self.slots];
            let mut m = Matcher {
                toks,
                env,
                memo: &mut *memo,
                caps: &mut caps,
                type_err: &mut type_err,
            };
            if m.seq(seq, 0, 0) {
                return Ok(caps);
            }
        }
        Err(type_err)
    }
}

fn parse_seq(
    c: &[char],
    pos: &mut usize,
    slots: &mut usize,
    close: Option<char>,
) -> Result<Vec<Node>, String> {
    let mut out = Vec::new();
    while *pos < c.len() {
        let ch = c[*pos];
        if Some(ch) == close || ch == '|' {
            return Ok(out);
        }
        match ch {
            ' ' => *pos += 1,
            '[' => {
                *pos += 1;
                let inner = parse_seq(c, pos, slots, Some(']'))?;
                if c.get(*pos) != Some(&']') {
                    return Err("pattern: missing ']'".into());
                }
                *pos += 1;
                out.push(Node::Opt(inner));
            }
            '(' => {
                *pos += 1;
                let mut alts = Vec::new();
                loop {
                    alts.push(parse_seq(c, pos, slots, Some(')'))?);
                    match c.get(*pos) {
                        Some('|') => *pos += 1,
                        Some(')') => {
                            *pos += 1;
                            break;
                        }
                        _ => return Err("pattern: missing ')'".into()),
                    }
                }
                out.push(Node::Choice(alts));
            }
            '%' => {
                *pos += 1;
                let start = *pos;
                while *pos < c.len() && c[*pos] != '%' {
                    *pos += 1;
                }
                let name: String = c[start..*pos].iter().collect();
                *pos += 1;
                let ty = match name.as_str() {
                    "number" => SlotTy::Ty(Ty::Number),
                    "text" => SlotTy::Ty(Ty::Text),
                    "player" => SlotTy::Ty(Ty::Player),
                    "boolean" => SlotTy::Ty(Ty::Boolean),
                    "item" => SlotTy::Ty(Ty::Item),
                    "location" => SlotTy::Ty(Ty::Location),
                    "world" => SlotTy::Ty(Ty::World),
                    "timespan" => SlotTy::Ty(Ty::Timespan),
                    "object" => SlotTy::Ty(Ty::Object),
                    "variable" => SlotTy::Variable,
                    _ => return Err(format!("pattern: unknown type %{name}%")),
                };
                out.push(Node::Slot(ty, *slots));
                *slots += 1;
            }
            _ => {
                let start = *pos;
                if ch.is_alphanumeric() || ch == '_' || ch == '-' {
                    while *pos < c.len()
                        && (c[*pos].is_alphanumeric() || c[*pos] == '_' || c[*pos] == '-' || c[*pos] == APOS)
                    {
                        *pos += 1;
                    }
                } else if ch == APOS && c.get(*pos + 1) == Some(&'s') {
                    // possessive `'s`
                    *pos += 2;
                } else {
                    *pos += 1;
                    // two-char symbols
                    if *pos < c.len() && c[*pos] == '=' && matches!(ch, '>' | '<' | '!') {
                        *pos += 1;
                    }
                }
                let lit: String = c[start..*pos].iter().collect();
                out.push(Node::Lit(lit.to_ascii_lowercase()));
            }
        }
    }
    if close.is_some() {
        return Err("pattern: unclosed group".into());
    }
    Ok(out)
}

fn expand(nodes: &[Node]) -> Vec<Vec<Flat>> {
    let mut acc: Vec<Vec<Flat>> = vec![Vec::new()];
    for n in nodes {
        match n {
            Node::Lit(l) => acc.iter_mut().for_each(|s| s.push(Flat::Lit(l.clone()))),
            Node::Slot(t, i) => acc.iter_mut().for_each(|s| s.push(Flat::Slot(*t, *i))),
            Node::Opt(inner) => {
                let mut next = Vec::new();
                for tail in expand(inner) {
                    for s in &acc {
                        let mut s2 = s.clone();
                        s2.extend(tail.iter().cloned());
                        next.push(s2);
                    }
                }
                // keep the "with optional part" variants first, so the greedy
                // reading wins (`send "a" to player` uses the `to` form).
                next.extend(acc);
                acc = next;
            }
            Node::Choice(alts) => {
                let mut next = Vec::new();
                for alt in alts {
                    for tail in expand(alt) {
                        for s in &acc {
                            let mut s2 = s.clone();
                            s2.extend(tail.iter().cloned());
                            next.push(s2);
                        }
                    }
                }
                acc = next;
            }
        }
    }
    acc
}

struct Matcher<'a> {
    toks: &'a [Tok],
    env: &'a Env,
    memo: &'a mut Memo,
    caps: &'a mut Vec<Option<Capture>>,
    type_err: &'a mut Option<String>,
}

impl Matcher<'_> {
    fn set_cap(&mut self, idx: usize, c: Option<Capture>) {
        if let Some(slot) = self.caps.get_mut(idx) {
            *slot = c;
        }
    }

    fn seq(&mut self, seq: &[Flat], si: usize, ti: usize) -> bool {
        let Some(node) = seq.get(si) else {
            return ti == self.toks.len();
        };
        match node {
            Flat::Lit(l) => match self.toks.get(ti) {
                Some(t) if t.is_lit(l) => self.seq(seq, si + 1, ti + 1),
                _ => false,
            },
            Flat::Slot(SlotTy::Variable, idx) => {
                let Some(Tok::Var(raw)) = self.toks.get(ti) else {
                    return false;
                };
                let env = self.env;
                let parsed = self
                    .memo
                    .vars
                    .entry(ti)
                    .or_insert_with(|| parse_varref(raw, env).ok());
                let Some(v) = parsed.clone() else {
                    return false;
                };
                self.set_cap(*idx, Some(Capture::Var(v)));
                if self.seq(seq, si + 1, ti + 1) {
                    return true;
                }
                self.set_cap(*idx, None);
                false
            }
            Flat::Slot(SlotTy::Ty(want), idx) => {
                for end in ti + 1..=self.toks.len() {
                    if self.memo.dead.get(&ti).is_some_and(|d| end > *d) {
                        break;
                    }
                    let env = self.env;
                    if !self.memo.exprs.contains_key(&(ti, end)) {
                        let parsed = match self.toks.get(ti..end).map(|span| parse_expr_span(span, env)) {
                            Some(Ok(e)) => Some(e),
                            Some(Err((_, saw_end))) => {
                                if !saw_end {
                                    self.memo.dead.insert(ti, end);
                                }
                                None
                            }
                            None => None,
                        };
                        self.memo.exprs.insert((ti, end), parsed);
                    }
                    let Some(Some(e)) = self.memo.exprs.get(&(ti, end)) else {
                        continue;
                    };
                    let got = e.ty(env);
                    if !want.accepts(got) {
                        if self.type_err.is_none() {
                            *self.type_err = Some(format!(
                                "expected a {} here but got a {}",
                                want.name(),
                                got.name()
                            ));
                        }
                        continue;
                    }
                    let e = e.clone();
                    self.set_cap(*idx, Some(Capture::Expr(e)));
                    if self.seq(seq, si + 1, end) {
                        return true;
                    }
                    self.set_cap(*idx, None);
                }
                false
            }
        }
    }
}

pub fn levenshtein(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    let mut cur = vec![0; b.len() + 1];
    for i in 1..=a.len() {
        cur[0] = i;
        for j in 1..=b.len() {
            let cost = usize::from(a[i - 1] != b[j - 1]);
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        std::mem::swap(&mut prev, &mut cur);
    }
    prev[b.len()]
}

/// The `n` pattern sources closest to `line` by edit distance.
pub fn closest<'a>(line: &str, srcs: impl Iterator<Item = &'a str>, n: usize) -> Vec<String> {
    let line = line.to_ascii_lowercase();
    let mut scored: Vec<(usize, &str)> = srcs.map(|s| (levenshtein(&line, s), s)).collect();
    scored.sort_by_key(|(d, _)| *d);
    scored.into_iter().take(n).map(|(_, s)| s.to_string()).collect()
}

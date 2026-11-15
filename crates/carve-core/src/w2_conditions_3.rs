//! Rare conditions: `is valid`, `script "x" is loaded`, `matches` (regex),
//! `is within`, `is within N blocks of`, `is poisoned`, `is silent`,
//! `is riding`, `pvp is enabled`, `is instant`, and block checks
//! (`is solid`, `is transparent`, `is passable`).
//!
//! The conditions run through one boxed `Cond` variant, so the hot condition
//! enum does not grow.

use crate::compile::CondKind;
use crate::expr::Expr;
use crate::pattern::Capture;
use crate::value::Value;

/// A yes/no state of a player that is not covered by `PlayerFlag`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerState {
    Poisoned,
    Silent,
    Riding,
}

/// A yes/no property of a block type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockTrait {
    Solid,
    Transparent,
    Passable,
}

/// What a pattern means. Copy, so it can sit inside `CondKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum W2Kind {
    Valid(bool),
    ScriptLoaded(bool),
    /// `matches`: the whole text must match.
    Matches(bool),
    /// `contains regex`: a match anywhere.
    FindsRegex(bool),
    WithinBox(bool),
    WithinRadius(bool),
    WithinWorld(bool),
    State(PlayerState, bool),
    /// `pvp is enabled`; the bool is `true` for the negated form (`disabled`).
    Pvp(bool, bool),
    Instant(bool),
    Block(BlockTrait, bool),
}

/// The built condition. `negate` is already folded in where it applies.
#[derive(Debug, Clone, PartialEq)]
pub enum W2Cond {
    Valid(Expr, bool),
    ScriptLoaded(Expr, bool),
    Regex {
        text: Expr,
        pattern: Expr,
        whole: bool,
        negate: bool,
    },
    WithinBox {
        loc: Expr,
        a: Expr,
        b: Expr,
        negate: bool,
    },
    WithinRadius {
        loc: Expr,
        radius: Expr,
        center: Expr,
        negate: bool,
    },
    WithinWorld {
        loc: Expr,
        world: Expr,
        negate: bool,
    },
    State(Expr, PlayerState, bool),
    Pvp {
        world: Option<Expr>,
        negate: bool,
    },
    Instant(Expr, bool),
    Block(Expr, BlockTrait, bool),
}

pub const W2_CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    ("%object% is valid", CondKind::W3(W2Kind::Valid(false))),
    ("%object% (is not|isn't) valid", CondKind::W3(W2Kind::Valid(true))),
    (
        "script %text% is loaded",
        CondKind::W3(W2Kind::ScriptLoaded(false)),
    ),
    (
        "script %text% (is not|isn't) loaded",
        CondKind::W3(W2Kind::ScriptLoaded(true)),
    ),
    (
        "pvp is enabled in %world%",
        CondKind::W3(W2Kind::Pvp(false, true)),
    ),
    (
        "pvp is (disabled|not enabled) in %world%",
        CondKind::W3(W2Kind::Pvp(true, true)),
    ),
    ("pvp is enabled", CondKind::W3(W2Kind::Pvp(false, false))),
    (
        "pvp is (disabled|not enabled)",
        CondKind::W3(W2Kind::Pvp(true, false)),
    ),
    (
        "%player% (is|are) poisoned",
        CondKind::W3(W2Kind::State(PlayerState::Poisoned, false)),
    ),
    (
        "%player% (is not|isn't|are not|aren't) poisoned",
        CondKind::W3(W2Kind::State(PlayerState::Poisoned, true)),
    ),
    (
        "%player% (is|are) silent",
        CondKind::W3(W2Kind::State(PlayerState::Silent, false)),
    ),
    (
        "%player% (is not|isn't|are not|aren't) silent",
        CondKind::W3(W2Kind::State(PlayerState::Silent, true)),
    ),
    (
        "%player% (is|are) riding",
        CondKind::W3(W2Kind::State(PlayerState::Riding, false)),
    ),
    (
        "%player% (is not|isn't|are not|aren't) riding",
        CondKind::W3(W2Kind::State(PlayerState::Riding, true)),
    ),
    ("%text% (is|are) instant", CondKind::W3(W2Kind::Instant(false))),
    (
        "%text% (is not|isn't|are not|aren't) instant",
        CondKind::W3(W2Kind::Instant(true)),
    ),
    (
        "%object% is solid",
        CondKind::W3(W2Kind::Block(BlockTrait::Solid, false)),
    ),
    (
        "%object% (is not|isn't) solid",
        CondKind::W3(W2Kind::Block(BlockTrait::Solid, true)),
    ),
    (
        "%object% is transparent",
        CondKind::W3(W2Kind::Block(BlockTrait::Transparent, false)),
    ),
    (
        "%object% (is not|isn't) transparent",
        CondKind::W3(W2Kind::Block(BlockTrait::Transparent, true)),
    ),
    (
        "%object% is passable",
        CondKind::W3(W2Kind::Block(BlockTrait::Passable, false)),
    ),
    (
        "%object% (is not|isn't) passable",
        CondKind::W3(W2Kind::Block(BlockTrait::Passable, true)),
    ),
    (
        "%object% is within %number% (block|blocks|meter|meters|metre|metres) (around|of|from) %object%",
        CondKind::W3(W2Kind::WithinRadius(false)),
    ),
    (
        "%object% (is not|isn't) within %number% (block|blocks|meter|meters|metre|metres) (around|of|from) %object%",
        CondKind::W3(W2Kind::WithinRadius(true)),
    ),
    (
        "%object% is within %object% (and|to) %object%",
        CondKind::W3(W2Kind::WithinBox(false)),
    ),
    (
        "%object% (is not|isn't) within %object% (and|to) %object%",
        CondKind::W3(W2Kind::WithinBox(true)),
    ),
    (
        "%object% is within %object%",
        CondKind::W3(W2Kind::WithinWorld(false)),
    ),
    (
        "%object% (is not|isn't) within %object%",
        CondKind::W3(W2Kind::WithinWorld(true)),
    ),
    (
        "%text% (matches|match) %text%",
        CondKind::W3(W2Kind::Matches(false)),
    ),
    (
        "%text% (doesn't|does not|don't|do not) match %text%",
        CondKind::W3(W2Kind::Matches(true)),
    ),
    (
        "%text% (contains|contain) regex %text%",
        CondKind::W3(W2Kind::FindsRegex(false)),
    ),
    (
        "%text% (doesn't|does not|don't|do not) contain regex %text%",
        CondKind::W3(W2Kind::FindsRegex(true)),
    ),
];

fn take(c: &mut [Option<Capture>], i: usize) -> Option<Expr> {
    match c.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    }
}

/// Build the condition from the captured slots. Returns `None` if a slot is missing.
pub fn build(kind: W2Kind, mut c: Vec<Option<Capture>>) -> Option<W2Cond> {
    let c = &mut c;
    Some(match kind {
        W2Kind::Valid(n) => W2Cond::Valid(take(c, 0)?, n),
        W2Kind::ScriptLoaded(n) => W2Cond::ScriptLoaded(take(c, 0)?, n),
        W2Kind::Matches(n) | W2Kind::FindsRegex(n) => W2Cond::Regex {
            text: take(c, 0)?,
            pattern: take(c, 1)?,
            whole: matches!(kind, W2Kind::Matches(_)),
            negate: n,
        },
        W2Kind::WithinBox(n) => W2Cond::WithinBox {
            loc: take(c, 0)?,
            a: take(c, 1)?,
            b: take(c, 2)?,
            negate: n,
        },
        W2Kind::WithinRadius(n) => W2Cond::WithinRadius {
            loc: take(c, 0)?,
            radius: take(c, 1)?,
            center: take(c, 2)?,
            negate: n,
        },
        W2Kind::WithinWorld(n) => W2Cond::WithinWorld {
            loc: take(c, 0)?,
            world: take(c, 1)?,
            negate: n,
        },
        W2Kind::State(s, n) => W2Cond::State(take(c, 0)?, s, n),
        W2Kind::Pvp(negate, with_world) => W2Cond::Pvp {
            world: if with_world { Some(take(c, 0)?) } else { None },
            negate,
        },
        W2Kind::Instant(n) => W2Cond::Instant(take(c, 0)?, n),
        W2Kind::Block(t, n) => W2Cond::Block(take(c, 0)?, t, n),
    })
}

// ---------------------------------------------------------------- helpers

/// A block id without namespace, lowercase, spaces as underscores.
pub fn norm_id(s: &str) -> String {
    let s = s.trim().to_ascii_lowercase();
    let s = s.strip_prefix("minecraft:").unwrap_or(&s);
    s.replace([' ', '-'], "_")
}

/// Instant potion effects: they act once instead of over time.
pub fn is_instant_effect(name: &str) -> bool {
    matches!(
        norm_id(name).as_str(),
        "instant_health" | "instant_damage" | "healing" | "heal" | "harming" | "harm" | "saturation"
    )
}

/// Does the script file name match what the user wrote? Accepts the name with
/// or without `.carve`, and ignores case and folder prefixes.
pub fn script_name_matches(file: &str, wanted: &str) -> bool {
    fn base(s: &str) -> String {
        let s = s.replace('\\', "/");
        let s = s.rsplit('/').next().unwrap_or("").to_ascii_lowercase();
        s.strip_suffix(".carve").map(str::to_string).unwrap_or(s)
    }
    let w = wanted.trim();
    if w.is_empty() {
        return false;
    }
    base(file) == base(w)
}

/// The position of a value that is a location. Players need the host.
pub fn loc_parts(v: &Value) -> Option<(f64, f64, f64, Option<&str>)> {
    match v {
        Value::Location { x, y, z, world } => Some((*x, *y, *z, world.as_deref())),
        _ => None,
    }
}

/// Worlds that are both known and different.
fn worlds_differ(a: Option<&str>, b: Option<&str>) -> bool {
    matches!((a, b), (Some(x), Some(y)) if !x.eq_ignore_ascii_case(y))
}

pub fn in_box(
    p: (f64, f64, f64, Option<&str>),
    a: (f64, f64, f64, Option<&str>),
    b: (f64, f64, f64, Option<&str>),
) -> bool {
    if worlds_differ(p.3, a.3) || worlds_differ(p.3, b.3) || worlds_differ(a.3, b.3) {
        return false;
    }
    let within = |v: f64, l: f64, h: f64| v >= l.min(h) && v <= l.max(h);
    within(p.0, a.0, b.0) && within(p.1, a.1, b.1) && within(p.2, a.2, b.2)
}

pub fn in_radius(p: (f64, f64, f64, Option<&str>), c: (f64, f64, f64, Option<&str>), r: f64) -> bool {
    if worlds_differ(p.3, c.3) || r < 0.0 {
        return false;
    }
    let (dx, dy, dz) = (p.0 - c.0, p.1 - c.1, p.2 - c.2);
    dx * dx + dy * dy + dz * dz <= r * r
}

// ---------------------------------------------------------------- regex

/// A small backtracking regex engine (Java-like syntax, no captures).
/// Supports: literals, `.`, classes `[a-z]`, `\d \w \s \D \W \S \b \B`, `^ $`,
/// groups `(...)` and `(?:...)`, alternation `|`, and the quantifiers
/// `* + ? {n} {n,} {n,m}` with a lazy `?` suffix. A leading `(?i)` ignores case.
/// A step limit stops runaway patterns.
pub mod regex {
    use std::cell::Cell;

    const STEP_LIMIT: usize = 200_000;
    const MAX_REPEAT: usize = 1000;
    /// Longest pattern, in characters. Patterns nest as deep as they are long.
    const MAX_PATTERN: usize = 2000;
    /// Most rounds of a repeated group. Each round is a deeper call, and the
    /// plugin has a small stack.
    const MAX_GROUP_ROUNDS: usize = 300;

    #[derive(Debug, Clone)]
    enum Node {
        Char(char),
        Any,
        Class(Vec<Item>, bool),
        Start,
        End,
        WordB(bool),
        Alt(Vec<Node>),
        Seq(Vec<Node>),
        Repeat {
            node: Box<Node>,
            min: usize,
            max: usize,
            lazy: bool,
        },
    }

    #[derive(Debug, Clone)]
    enum Item {
        Range(char, char),
        Digit(bool),
        Word(bool),
        Space(bool),
    }

    #[derive(Debug)]
    pub struct Regex {
        root: Node,
        icase: bool,
        steps: Cell<usize>,
    }

    struct Parser<'a> {
        c: &'a [char],
        i: usize,
    }

    impl Parser<'_> {
        fn peek(&self) -> Option<char> {
            self.c.get(self.i).copied()
        }

        fn alt(&mut self, depth: usize) -> Result<Node, String> {
            if depth > 50 {
                return Err("pattern is nested too deeply".into());
            }
            let mut alts = vec![self.seq(depth)?];
            while self.peek() == Some('|') {
                self.i += 1;
                alts.push(self.seq(depth)?);
            }
            Ok(if alts.len() == 1 {
                alts.remove(0)
            } else {
                Node::Alt(alts)
            })
        }

        fn seq(&mut self, depth: usize) -> Result<Node, String> {
            let mut items = Vec::new();
            while let Some(ch) = self.peek() {
                if ch == '|' || ch == ')' {
                    break;
                }
                let atom = self.atom(depth)?;
                items.push(self.quant(atom)?);
            }
            Ok(Node::Seq(items))
        }

        fn number(&mut self) -> Option<usize> {
            let start = self.i;
            while self.peek().is_some_and(|c| c.is_ascii_digit()) {
                self.i += 1;
            }
            if start == self.i {
                return None;
            }
            self.c[start..self.i].iter().collect::<String>().parse().ok()
        }

        fn quant(&mut self, atom: Node) -> Result<Node, String> {
            let (min, max) = match self.peek() {
                Some('*') => (0, usize::MAX),
                Some('+') => (1, usize::MAX),
                Some('?') => (0, 1),
                Some('{') => {
                    let save = self.i;
                    self.i += 1;
                    let Some(lo) = self.number() else {
                        self.i = save;
                        return Ok(atom);
                    };
                    let hi = if self.peek() == Some(',') {
                        self.i += 1;
                        self.number().unwrap_or(usize::MAX)
                    } else {
                        lo
                    };
                    if self.peek() != Some('}') {
                        return Err("unclosed '{' in pattern".into());
                    }
                    if lo > MAX_REPEAT || (hi != usize::MAX && hi > MAX_REPEAT) || hi < lo {
                        return Err("bad repeat count in pattern".into());
                    }
                    (lo, hi)
                }
                _ => return Ok(atom),
            };
            self.i += 1;
            let lazy = self.peek() == Some('?');
            if lazy {
                self.i += 1;
            } else if self.peek() == Some('+') {
                // Possessive quantifiers are treated as greedy.
                self.i += 1;
            }
            Ok(Node::Repeat {
                node: Box::new(atom),
                min,
                max,
                lazy,
            })
        }

        fn escape_item(ch: char) -> Option<Item> {
            Some(match ch {
                'd' => Item::Digit(true),
                'D' => Item::Digit(false),
                'w' => Item::Word(true),
                'W' => Item::Word(false),
                's' => Item::Space(true),
                'S' => Item::Space(false),
                _ => return None,
            })
        }

        fn escaped_char(ch: char) -> char {
            match ch {
                'n' => '\n',
                't' => '\t',
                'r' => '\r',
                other => other,
            }
        }

        fn atom(&mut self, depth: usize) -> Result<Node, String> {
            let ch = self.peek().ok_or("unexpected end of pattern")?;
            self.i += 1;
            Ok(match ch {
                '.' => Node::Any,
                '^' => Node::Start,
                '$' => Node::End,
                '(' => {
                    if self.peek() == Some('?') {
                        // Only `(?:` is supported.
                        if self.c.get(self.i + 1) == Some(&':') {
                            self.i += 2;
                        } else {
                            return Err("only '(?:' groups are supported".into());
                        }
                    }
                    let inner = self.alt(depth + 1)?;
                    if self.peek() != Some(')') {
                        return Err("unclosed '(' in pattern".into());
                    }
                    self.i += 1;
                    inner
                }
                '[' => self.class()?,
                '\\' => {
                    let e = self.peek().ok_or("pattern ends with a backslash")?;
                    self.i += 1;
                    match e {
                        'b' => Node::WordB(true),
                        'B' => Node::WordB(false),
                        _ => match Self::escape_item(e) {
                            Some(item) => Node::Class(vec![item], false),
                            None => Node::Char(Self::escaped_char(e)),
                        },
                    }
                }
                '*' | '+' | '?' => return Err(format!("nothing to repeat before '{ch}'")),
                ')' => return Err("unmatched ')' in pattern".into()),
                other => Node::Char(other),
            })
        }

        fn class(&mut self) -> Result<Node, String> {
            let neg = self.peek() == Some('^');
            if neg {
                self.i += 1;
            }
            let mut items = Vec::new();
            let mut first = true;
            loop {
                let ch = self.peek().ok_or("unclosed '[' in pattern")?;
                self.i += 1;
                if ch == ']' && !first {
                    break;
                }
                first = false;
                let lo = if ch == '\\' {
                    let e = self.peek().ok_or("pattern ends with a backslash")?;
                    self.i += 1;
                    if let Some(item) = Self::escape_item(e) {
                        items.push(item);
                        continue;
                    }
                    Self::escaped_char(e)
                } else {
                    ch
                };
                if self.peek() == Some('-') && self.c.get(self.i + 1).is_some_and(|c| *c != ']') {
                    self.i += 1;
                    let mut hi = self.peek().ok_or("unclosed '[' in pattern")?;
                    self.i += 1;
                    if hi == '\\' {
                        hi = Self::escaped_char(self.peek().ok_or("pattern ends with a backslash")?);
                        self.i += 1;
                    }
                    if hi < lo {
                        return Err("bad range in character class".into());
                    }
                    items.push(Item::Range(lo, hi));
                } else {
                    items.push(Item::Range(lo, lo));
                }
            }
            Ok(Node::Class(items, neg))
        }
    }

    fn is_word(c: char) -> bool {
        c.is_ascii_alphanumeric() || c == '_'
    }

    fn item_hits(it: &Item, c: char, icase: bool) -> bool {
        match it {
            Item::Range(lo, hi) => {
                if (*lo..=*hi).contains(&c) {
                    return true;
                }
                if icase {
                    let l = c.to_ascii_lowercase();
                    let u = c.to_ascii_uppercase();
                    (*lo..=*hi).contains(&l) || (*lo..=*hi).contains(&u)
                } else {
                    false
                }
            }
            Item::Digit(p) => c.is_ascii_digit() == *p,
            Item::Word(p) => is_word(c) == *p,
            Item::Space(p) => c.is_whitespace() == *p,
        }
    }

    impl Regex {
        pub fn new(src: &str) -> Result<Regex, String> {
            let (body, icase) = match src.strip_prefix("(?i)") {
                Some(rest) => (rest, true),
                None => (src, false),
            };
            let chars: Vec<char> = body.chars().take(MAX_PATTERN + 1).collect();
            if chars.len() > MAX_PATTERN {
                return Err(format!("pattern is longer than {MAX_PATTERN} characters"));
            }
            let mut p = Parser { c: &chars, i: 0 };
            let root = p.alt(0)?;
            if p.i < chars.len() {
                return Err("unmatched ')' in pattern".into());
            }
            Ok(Regex {
                root,
                icase,
                steps: Cell::new(0),
            })
        }

        fn tick(&self) -> bool {
            let n = self.steps.get() + 1;
            self.steps.set(n);
            n <= STEP_LIMIT
        }

        /// One character of a single-character node, if it matches.
        fn single(&self, node: &Node, t: &[char], i: usize) -> Option<bool> {
            let c = t.get(i).copied();
            Some(match node {
                Node::Char(x) => {
                    c.is_some_and(|c| c == *x || (self.icase && c.to_lowercase().eq(x.to_lowercase())))
                }
                Node::Any => c.is_some_and(|c| c != '\n'),
                Node::Class(items, neg) => {
                    c.is_some_and(|c| items.iter().any(|it| item_hits(it, c, self.icase)) != *neg)
                }
                _ => return None,
            })
        }

        fn m(&self, node: &Node, t: &[char], i: usize, k: &mut dyn FnMut(usize) -> bool) -> bool {
            if !self.tick() {
                return false;
            }
            match node {
                Node::Char(_) | Node::Any | Node::Class(..) => {
                    self.single(node, t, i) == Some(true) && k(i + 1)
                }
                Node::Start => i == 0 && k(i),
                Node::End => i == t.len() && k(i),
                Node::WordB(want) => {
                    let before = i > 0 && is_word(t[i - 1]);
                    let after = t.get(i).is_some_and(|c| is_word(*c));
                    ((before != after) == *want) && k(i)
                }
                Node::Alt(alts) => alts.iter().any(|a| self.m(a, t, i, k)),
                Node::Seq(items) => self.seq(items, t, i, k),
                Node::Repeat { node, min, max, lazy } => {
                    if matches!(**node, Node::Char(_) | Node::Any | Node::Class(..)) {
                        // Single-character body: count, then try each length.
                        let mut n = 0;
                        while n < *max && self.single(node, t, i + n) == Some(true) {
                            n += 1;
                            if !self.tick() {
                                return false;
                            }
                        }
                        if n < *min {
                            return false;
                        }
                        if *lazy {
                            (*min..=n).any(|c| k(i + c))
                        } else {
                            (*min..=n).rev().any(|c| k(i + c))
                        }
                    } else {
                        self.rep(node, *min, *max, *lazy, 0, t, i, k)
                    }
                }
            }
        }

        fn seq(&self, items: &[Node], t: &[char], i: usize, k: &mut dyn FnMut(usize) -> bool) -> bool {
            match items.split_first() {
                None => k(i),
                Some((first, rest)) => self.m(first, t, i, &mut |j| self.seq(rest, t, j, k)),
            }
        }

        #[allow(clippy::too_many_arguments)]
        fn rep(
            &self,
            node: &Node,
            min: usize,
            max: usize,
            lazy: bool,
            count: usize,
            t: &[char],
            i: usize,
            k: &mut dyn FnMut(usize) -> bool,
        ) -> bool {
            if !self.tick() {
                return false;
            }
            if count > MAX_GROUP_ROUNDS {
                self.steps.set(STEP_LIMIT + 1);
                return false;
            }
            let more = |this: &Self, k: &mut dyn FnMut(usize) -> bool| {
                count < max
                    && this.m(node, t, i, &mut |j| {
                        // An empty iteration cannot make progress.
                        j != i && this.rep(node, min, max, lazy, count + 1, t, j, k)
                    })
            };
            if count < min {
                return more(self, k);
            }
            #[allow(clippy::if_same_then_else)]
            if lazy {
                k(i) || more(self, k)
            } else {
                more(self, k) || k(i)
            }
        }

        /// `Err` when the step limit was hit.
        fn run(&self, text: &str, whole: bool) -> Result<bool, String> {
            let t: Vec<char> = text.chars().collect();
            self.steps.set(0);
            let starts: Box<dyn Iterator<Item = usize>> = if whole {
                Box::new(0..1)
            } else {
                Box::new(0..=t.len())
            };
            for s in starts {
                let hit = self.m(&self.root, &t, s, &mut |e| !whole || e == t.len());
                if self.steps.get() > STEP_LIMIT {
                    return Err("regex is too complex for this text".into());
                }
                if hit {
                    return Ok(true);
                }
            }
            Ok(false)
        }

        /// The whole text matches.
        pub fn is_match(&self, text: &str) -> Result<bool, String> {
            self.run(text, true)
        }

        /// A match exists anywhere in the text.
        pub fn find(&self, text: &str) -> Result<bool, String> {
            self.run(text, false)
        }

        /// The first match starting at or after `from`, as char positions.
        pub fn find_span(&self, t: &[char], from: usize) -> Result<Option<(usize, usize)>, String> {
            self.steps.set(0);
            for s in from..=t.len() {
                let mut end = None;
                let hit = self.m(&self.root, t, s, &mut |e| {
                    end = Some(e);
                    true
                });
                if self.steps.get() > STEP_LIMIT {
                    return Err("regex is too complex for this text".into());
                }
                if let (true, Some(e)) = (hit, end) {
                    return Ok(Some((s, e)));
                }
            }
            Ok(None)
        }
    }
}

/// `matches` / `contains regex` with a readable error for a bad pattern.
pub fn regex_test(text: &str, pattern: &str, whole: bool) -> Result<bool, String> {
    let re = regex::Regex::new(pattern).map_err(|e| format!("invalid regex '{pattern}': {e}"))?;
    if whole { re.is_match(text) } else { re.find(text) }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn full(p: &str, t: &str) -> bool {
        regex_test(t, p, true).unwrap()
    }

    #[test]
    fn regex_basics() {
        assert!(full("abc", "abc"));
        assert!(!full("abc", "abcd"));
        assert!(full("a.c", "abc"));
        assert!(full("[a-c]+", "abcabc"));
        assert!(!full("[^a-c]+", "abc"));
        assert!(full("\\d{3}-\\d{2,}", "123-4567"));
        assert!(full("(foo|bar)+x?", "foobarfoo"));
        assert!(full("(?i)hello", "HeLLo"));
        assert!(full("a*", ""));
        assert!(full(".*end", "the end"));
        assert!(full("(ab)*c", "ababc"));
        assert!(regex_test("say hello there", "\\bhello\\b", false).unwrap());
        assert!(!regex_test("shelloo", "\\bhello\\b", false).unwrap());
        assert!(regex_test("abc", "^b", false).is_ok_and(|b| !b));
    }

    #[test]
    fn regex_errors_and_limits() {
        assert!(regex_test("a", "(", true).is_err());
        assert!(regex_test("a", "[a", true).is_err());
        assert!(regex_test("a", "*a", true).is_err());
        assert!(regex_test("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaab", "(a+)+$", false).is_err());
    }

    #[test]
    fn helpers() {
        assert!(script_name_matches("scripts/Shop.carve", "shop"));
        assert!(script_name_matches("shop.carve", "SHOP.carve"));
        assert!(!script_name_matches("shop.carve", "shops"));
        assert!(is_instant_effect("Instant Health"));
        assert!(is_instant_effect("minecraft:saturation"));
        assert!(!is_instant_effect("poison"));
        assert!(!is_instant_effect("health"));
    }
}

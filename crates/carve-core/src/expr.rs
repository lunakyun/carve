//! Expressions: literals, variables, text templates, event values, arithmetic.

use std::cell::Cell;

use crate::lexer::{Tok, lex};
use crate::list_literals::ElemSel;
use crate::list_ops::{Agg, ListSrc, parse_src_at};
use crate::value::format_num;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Number,
    Text,
    Player,
    Boolean,
    Item,
    Location,
    World,
    Timespan,
    Date,
    /// Unknown until runtime (variables).
    Object,
    /// A list (function parameters and results like `numbers`).
    List,
}

impl Ty {
    pub fn name(self) -> &'static str {
        match self {
            Ty::Number => "number",
            Ty::Text => "text",
            Ty::Player => "player",
            Ty::Boolean => "boolean",
            Ty::Item => "item",
            Ty::Location => "location",
            Ty::World => "world",
            Ty::Timespan => "timespan",
            Ty::Date => "date",
            Ty::Object => "object",
            Ty::List => "list",
        }
    }

    /// Can a value of type `got` fill a slot of type `self`?
    pub fn accepts(self, got: Ty) -> bool {
        self == Ty::Object
            || got == Ty::Object
            || self == Ty::Text
            || self == got
            || (self == Ty::Number && got == Ty::Date)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Part {
    Lit(String),
    Expr(Expr),
}

/// A variable name, lowercased. Names made only of plain text are lowercased
/// once at compile time; names with `%...%` parts are built on every use.
#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Static(String),
    Dynamic(Vec<Part>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarRef {
    pub key: Key,
    pub local: bool,
    /// `{x::*}`: the whole list.
    pub list: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Bool(bool),
    Text(Vec<Part>),
    Var(VarRef),
    Player,
    /// `sender`, `executor`: whoever started the trigger (a player or the console).
    Sender,
    /// `console`
    Console,
    Message,
    /// 1-based command argument.
    Arg(usize),
    NameOf(Box<Expr>),
    Neg(Box<Expr>),
    Bin(Box<Expr>, BinOp, Box<Expr>),
    /// Current value of the loop at this depth (0 = outermost loop).
    LoopValue(usize),
    /// List index (`loop-index`) of the list loop at this depth.
    LoopIndex(usize),
    /// 1-based round counter (`loop-iteration`) of the loop at this depth.
    LoopIter(usize),
    /// Number of items in a list variable (`size of {x::*}`).
    Size(VarRef),
    /// Call of a script function, by id, with its arguments.
    Call(usize, Vec<Expr>),
    /// Item literal like `diamond sword` or `2 diamonds`.
    Item {
        id: String,
        count: u32,
    },
    /// `location of player`, `player's world`, `x of {_loc}` and so on.
    Prop(Prop, Box<Expr>),
    /// `location(x, y, z[, world])`.
    Loc(Box<Expr>, Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    /// `world "name"`.
    WorldNamed(Box<Expr>),
    /// `distance between a and b`.
    Distance(Box<Expr>, Box<Expr>),
    /// `random integer between a and b` (true) / `random number between a and b`.
    Random(bool, Box<Expr>, Box<Expr>),
    /// `event-<name>` of a `w2_events_4` event: index into the event values.
    EventVal(u8),
    /// `damage` in `on damage`.
    Damage,
    /// `attacker` in `on damage` / `on death` (a player, or `<none>`).
    Attacker,
    /// `clicked block` in click events (an item value of the block type).
    ClickedBlock,
    /// `a ? b`: a, or b when a is not set.
    Default(Box<Expr>, Box<Expr>),
    /// `floor(x)`, `max(a, b)`, ...
    Math(MathFn, Vec<Expr>),
    /// `x parsed as number`: the number, or `<none>`.
    ParseNum(Box<Expr>),
    /// `random element of {list::*}`.
    RandomOf(VarRef),
    /// `join {list::*} with ", "`.
    Join(VarRef, Box<Expr>),
    /// `split "a,b" at ","`: a list, only usable in `set {x::*} to` and `loop`.
    Split(Box<Expr>, Box<Expr>),
    /// `length of x` / `amount of characters in x`.
    Length(Box<Expr>),
    /// `x with all "a" replaced with "b"`: text, from, to, replace all.
    Replaced(Box<Expr>, Box<Expr>, Box<Expr>, bool),
    /// Text functions: case, substring, characters, index of, trimmed, reversed.
    TextFn(crate::text_case_substring::TextFn, Vec<Expr>),
    /// `indices of {x::*}`: the index of every item, as text.
    Indices(VarRef),
    /// `first element of {x::*}`, `element 2 of {x::*}`.
    Elem(ElemSel, VarRef),
    /// `now`, `unix timestamp of d`, `date from unix timestamp n`, `d formatted as f`.
    Date(DateOp, Vec<Expr>),
    /// `colored x` (true) / `uncolored x` (false).
    Color(bool, Box<Expr>),
    /// A time span literal like `1 hour and 30 seconds`, in milliseconds.
    Span(i64),
    /// `time since <date>` (true) / `time until <date>` (false).
    Since(Box<Expr>, bool),
    /// Math where a side is a time span or date.
    TBin(Box<Expr>, BinOp, Box<Expr>),
    /// `minutes of <span>` / `total minutes of <span>`: ms per unit, total?
    SpanPart(i64, bool, Box<Expr>),
    /// `sorted {x::*}`, `{x::*} where [...]`, ...: a list value.
    Items(Box<ListSrc>),
    /// `sum of {x::*}`, `max of ...`, `size of {x::*} where [...]`.
    ListAgg(Agg, Box<ListSrc>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateOp {
    Now,
    /// Seconds since 1970 of a date.
    ToUnix,
    /// Date from seconds since 1970.
    FromUnix,
    /// Date, optional format text.
    Format,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathFn {
    Floor,
    Ceil,
    Round,
    Abs,
    Sqrt,
    Min,
    Max,
    Mod,
}

impl MathFn {
    fn from_name(n: &str) -> Option<(MathFn, usize)> {
        Some(match n {
            "floor" => (MathFn::Floor, 1),
            "ceil" | "ceiling" => (MathFn::Ceil, 1),
            "round" => (MathFn::Round, 1),
            "abs" => (MathFn::Abs, 1),
            "sqrt" => (MathFn::Sqrt, 1),
            "min" => (MathFn::Min, 2),
            "max" => (MathFn::Max, 2),
            "mod" => (MathFn::Mod, 2),
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prop {
    Location,
    World,
    Uuid,
    Name,
    Tool,
    Health,
    Gamemode,
    Level,
    Xp,
    LevelProgress,
    MaxHealth,
    Food,
    Saturation,
    X,
    Y,
    Z,
}

impl Prop {
    fn from_word(w: &str) -> Option<Prop> {
        Some(match w {
            "location" | "position" => Prop::Location,
            "world" => Prop::World,
            "uuid" => Prop::Uuid,
            "name" | "display" => Prop::Name,
            "tool" | "held" => Prop::Tool,
            "health" => Prop::Health,
            "gamemode" => Prop::Gamemode,
            "level" => Prop::Level,
            "xp" | "exp" | "experience" => Prop::Xp,
            "x" => Prop::X,
            "y" => Prop::Y,
            "z" => Prop::Z,
            _ => return None,
        })
    }

    pub fn ty(self) -> Ty {
        match self {
            Prop::Location => Ty::Location,
            Prop::World => Ty::World,
            Prop::Uuid | Prop::Name | Prop::Gamemode => Ty::Text,
            Prop::Tool => Ty::Item,
            Prop::Health
            | Prop::MaxHealth
            | Prop::Food
            | Prop::Saturation
            | Prop::Level
            | Prop::Xp
            | Prop::LevelProgress
            | Prop::X
            | Prop::Y
            | Prop::Z => Ty::Number,
        }
    }

    /// Types this property can be read from.
    fn takes(self, t: Ty) -> bool {
        t == Ty::Object
            || match self {
                Prop::X | Prop::Y | Prop::Z | Prop::World => t == Ty::Player || t == Ty::Location,
                _ => t == Ty::Player,
            }
    }
}

/// A function's parameters and return type, known to every script at load.
#[derive(Debug, Clone, PartialEq)]
pub struct Sig {
    pub name: String,
    /// (local variable key like `_name`, type, has a default value)
    pub params: Vec<(String, Ty, bool)>,
    pub ret: Option<Ty>,
}

/// What a loop goes over. Decides the type of `loop-value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopKind {
    Times,
    List,
    Players,
    Values,
}

/// What the current trigger offers to expressions.
#[derive(Debug, Clone, Default)]
pub struct Env {
    pub has_message: bool,
    pub arg_types: Vec<Ty>,
    /// `Some("chat")` etc, used in error messages.
    pub trigger_name: String,
    /// Loops around the current line, outermost first (`while` not included).
    pub loops: Vec<LoopKind>,
    /// Every function of every script, by id.
    pub functions: std::sync::Arc<Vec<Sig>>,
    /// Inside a function body: no event values (`player`, `message`, `arg`).
    pub in_function: bool,
    /// The trigger has no player (`on load`, `every`).
    pub no_player: bool,
    pub has_damage: bool,
    pub has_attacker: bool,
    pub has_block: bool,
    /// The trigger is an event of `w2_events_4` (gives `event-<name>`).
    pub ext: Option<crate::w2_events_4::ExtEvent>,
}

impl Expr {
    pub fn ty(&self, env: &Env) -> Ty {
        match self {
            Expr::Num(_) | Expr::Neg(_) | Expr::Bin(..) | Expr::LoopIter(_) | Expr::Size(_) => Ty::Number,
            Expr::Span(_) | Expr::Since(..) => Ty::Timespan,
            Expr::SpanPart(..) => Ty::Number,
            Expr::TBin(a, op, b) => crate::timespans::result_ty(a.ty(env), *op, b.ty(env)),
            Expr::LoopIndex(_) => Ty::Text,
            Expr::Bool(_) => Ty::Boolean,
            Expr::Call(id, _) => env.functions.get(*id).and_then(|s| s.ret).unwrap_or(Ty::Object),
            Expr::Item { .. } => Ty::Item,
            Expr::Prop(p, _) => p.ty(),
            Expr::Loc(..) => Ty::Location,
            Expr::WorldNamed(_) => Ty::World,
            Expr::Distance(..) | Expr::Random(..) | Expr::Damage => Ty::Number,
            Expr::Attacker => Ty::Player,
            Expr::EventVal(_) => Ty::Object,
            Expr::Math(..) | Expr::ParseNum(_) => Ty::Number,
            Expr::Color(..) => Ty::Text,
            Expr::RandomOf(_) | Expr::Split(..) | Expr::Elem(..) => Ty::Object,
            Expr::Join(..) | Expr::Replaced(..) => Ty::Text,
            Expr::Length(_) => Ty::Number,
            Expr::TextFn(f, _) if f.returns_number() => Ty::Number,
            Expr::TextFn(..) => Ty::Text,
            Expr::Indices(_) => Ty::Text,
            Expr::Date(op, _) => match op {
                DateOp::Now | DateOp::FromUnix => Ty::Date,
                DateOp::ToUnix => Ty::Number,
                DateOp::Format => Ty::Text,
            },
            Expr::Items(_) => Ty::List,
            Expr::ListAgg(Agg::Size | Agg::Sum, _) => Ty::Number,
            Expr::ListAgg(..) => Ty::Object,
            Expr::Default(a, b) => {
                let (x, y) = (a.ty(env), b.ty(env));
                if x == y { x } else { Ty::Object }
            }
            Expr::ClickedBlock => Ty::Item,
            Expr::LoopValue(d) => match env.loops.get(*d) {
                Some(LoopKind::Times) => Ty::Number,
                Some(LoopKind::Players) => Ty::Player,
                _ => Ty::Object,
            },
            Expr::Text(_) | Expr::Message | Expr::NameOf(_) => Ty::Text,
            Expr::Var(_) => Ty::Object,
            Expr::Player => Ty::Player,
            Expr::Sender | Expr::Console => Ty::Object,
            Expr::Arg(i) => env
                .arg_types
                .get(i.wrapping_sub(1))
                .copied()
                .unwrap_or(Ty::Object),
        }
    }
}

/// Parse one arithmetic expression starting at `pos`; returns it and where it ended.
pub(crate) fn parse_sum_at(toks: &[Tok], pos: usize, env: &Env) -> Result<(Expr, usize), String> {
    let mut p = P {
        toks,
        pos,
        env,
        hit_end: Cell::new(false),
    };
    let e = p.sum()?;
    Ok((e, p.pos))
}

pub fn parse_expr(toks: &[Tok], env: &Env) -> Result<Expr, String> {
    parse_expr_span(toks, env).map_err(|(msg, _)| msg)
}

/// Like [`parse_expr`]. On failure, also says whether the parser looked past
/// the last token. If it did not, adding more tokens can't make the parse
/// succeed, which lets the pattern matcher stop trying longer spans.
pub fn parse_expr_span(toks: &[Tok], env: &Env) -> Result<Expr, (String, bool)> {
    if toks.is_empty() {
        return Err(("missing expression".into(), true));
    }
    let mut p = P {
        toks,
        pos: 0,
        env,
        hit_end: Cell::new(false),
    };
    let e = p.top().map_err(|m| (m, p.hit_end.get()))?;
    if let Some(t) = toks.get(p.pos) {
        return Err((format!("unexpected {}", describe(t)), p.hit_end.get()));
    }
    Ok(e)
}

pub fn parse_expr_str(s: &str, env: &Env) -> Result<Expr, String> {
    parse_expr(&lex(s)?, env)
}

fn describe(t: &Tok) -> String {
    match t {
        Tok::Word(w) => format!("'{w}'"),
        Tok::Num(n) => format!("number {n}"),
        Tok::Str(_) => "text".into(),
        Tok::Var(v) => format!("{{{v}}}"),
        Tok::Sym(s) => format!("'{s}'"),
    }
}

struct P<'a> {
    toks: &'a [Tok],
    pos: usize,
    env: &'a Env,
    /// Set when the parser asked for a token past the end.
    hit_end: Cell<bool>,
}

impl P<'_> {
    fn peek(&self) -> Option<&Tok> {
        let t = self.toks.get(self.pos);
        if t.is_none() {
            self.hit_end.set(true);
        }
        t
    }

    /// Like `peek`, `n` tokens further on; running off the end counts as "need more tokens".
    fn peek_at(&self, n: usize) -> Option<&Tok> {
        let t = self.toks.get(self.pos + n);
        if t.is_none() {
            self.hit_end.set(true);
        }
        t
    }

    fn eat_sym(&mut self, s: &str) -> bool {
        if matches!(self.peek(), Some(Tok::Sym(x)) if *x == s) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// `a ? b ? c`: the lowest priority, below math.
    fn top(&mut self) -> Result<Expr, String> {
        let mut e = self.sum()?;
        while self.eat_sym("?") {
            let b = self.sum()?;
            e = Expr::Default(Box::new(e), Box::new(b));
        }
        Ok(e)
    }

    fn sum(&mut self) -> Result<Expr, String> {
        let mut left = self.product()?;
        loop {
            let op = if self.eat_sym("+") {
                BinOp::Add
            } else if self.eat_sym("-") {
                BinOp::Sub
            } else {
                return Ok(left);
            };
            let right = self.product()?;
            left = self.bin(left, op, right)?;
        }
    }

    fn product(&mut self) -> Result<Expr, String> {
        let mut left = self.unary()?;
        loop {
            let op = if self.eat_sym("*") {
                BinOp::Mul
            } else if self.eat_sym("/") {
                BinOp::Div
            } else {
                return Ok(left);
            };
            let right = self.unary()?;
            left = self.bin(left, op, right)?;
        }
    }

    fn bin(&self, l: Expr, op: BinOp, r: Expr) -> Result<Expr, String> {
        let (lt, rt) = (l.ty(self.env), r.ty(self.env));
        if matches!(lt, Ty::Timespan | Ty::Date) || matches!(rt, Ty::Timespan | Ty::Date) {
            if let (Expr::Span(x), Expr::Span(y)) = (&l, &r) {
                match op {
                    BinOp::Add => return Ok(Expr::Span(x.saturating_add(*y))),
                    BinOp::Sub => return Ok(Expr::Span(x.saturating_sub(*y))),
                    _ => {}
                }
            }
            if !crate::timespans::ok_types(lt, op, rt) {
                return Err(format!(
                    "can't do {} and {} with that operator",
                    lt.name(),
                    rt.name()
                ));
            }
            return Ok(Expr::TBin(Box::new(l), op, Box::new(r)));
        }
        for side in [&l, &r] {
            let t = side.ty(self.env);
            if !Ty::Number.accepts(t) {
                return Err(format!("math needs numbers, got {}", t.name()));
            }
        }
        // Fold constant math. Division by zero stays a runtime error so it is
        // reported with its line, like any other runtime error.
        if let (Expr::Num(x), Expr::Num(y)) = (&l, &r) {
            let folded = match op {
                BinOp::Add => Some(x + y),
                BinOp::Sub => Some(x - y),
                BinOp::Mul => Some(x * y),
                BinOp::Div => (*y != 0.0).then(|| x / y),
            };
            if let Some(n) = folded {
                return Ok(Expr::Num(n));
            }
        }
        Ok(Expr::Bin(Box::new(l), op, Box::new(r)))
    }

    fn unary(&mut self) -> Result<Expr, String> {
        if self.eat_sym("-") {
            let inner = self.unary()?;
            if !Ty::Number.accepts(inner.ty(self.env)) {
                return Err("'-' needs a number".into());
            }
            return Ok(match inner {
                Expr::Num(n) => Expr::Num(-n),
                e => Expr::Neg(Box::new(e)),
            });
        }
        self.primary()
    }

    fn primary(&mut self) -> Result<Expr, String> {
        let mut e = self.primary_base()?;
        // `5 minutes ago`, `2 hours later`
        if let Some(Tok::Word(w)) = self.toks.get(self.pos)
            && (w.eq_ignore_ascii_case("ago") || w.eq_ignore_ascii_case("later"))
            && matches!(e.ty(self.env), Ty::Timespan | Ty::Object)
        {
            let op = if w.eq_ignore_ascii_case("ago") {
                BinOp::Sub
            } else {
                BinOp::Add
            };
            self.pos += 1;
            e = Expr::TBin(Box::new(Expr::Date(DateOp::Now, Vec::new())), op, Box::new(e));
        }
        // `arg-1 parsed as number`
        if matches!(self.toks.get(self.pos), Some(t) if t.is_lit("parsed")) && self.pos + 1 >= self.toks.len()
        {
            self.hit_end.set(true);
        }
        if matches!(self.toks.get(self.pos), Some(t) if t.is_lit("parsed"))
            && matches!(self.toks.get(self.pos + 1), Some(t) if t.is_lit("as"))
        {
            match self.toks.get(self.pos + 2) {
                Some(t) if t.is_lit("number") || t.is_lit("integer") || t.is_lit("num") => {
                    self.pos += 3;
                    e = Expr::ParseNum(Box::new(e));
                }
                Some(_) => return Err("only 'parsed as number' is supported".into()),
                None => {
                    self.hit_end.set(true);
                    return Err("expected 'parsed as number'".into());
                }
            }
        }
        e = self.replaced_postfix(e);
        e = self.in_case(e);
        // `now formatted as "yyyy-MM-dd"`
        while matches!(self.toks.get(self.pos), Some(t) if t.is_lit("formatted"))
            && matches!(e.ty(self.env), Ty::Date | Ty::Object)
        {
            self.pos += 1;
            e = self.date_format(e)?;
        }
        // `player's location`, `player's world`, ...
        while matches!(self.toks.get(self.pos), Some(Tok::Sym("'s"))) {
            let Some(Tok::Word(w)) = self.toks.get(self.pos + 1) else {
                if self.pos + 1 >= self.toks.len() {
                    self.hit_end.set(true);
                }
                return Err("expected a property after 's, like player's location".into());
            };
            let lw = w.to_ascii_lowercase();
            if self.pos + 2 >= self.toks.len() {
                self.hit_end.set(true);
            }
            let (prop, extra) = match crate::health_food::words_from(self.toks, self.pos + 1) {
                Some((p, n)) => (p, n - 1),
                None => match lw.as_str() {
                    "inventory" => {
                        return Err("use 'player's inventory contains <item>' as a condition".into());
                    }
                    _ => match self.toks.get(self.pos + 2) {
                        Some(Tok::Word(w2)) if crate::xp_level::two_word(&lw, &w2.to_ascii_lowercase()).is_some() => (
                            crate::xp_level::two_word(&lw, &w2.to_ascii_lowercase()).unwrap_or(Prop::Level),
                            1,
                        ),
                        _ => (
                            Prop::from_word(&lw).ok_or_else(|| format!("unknown property '{w}'"))?,
                            0,
                        ),
                    },
                },
            };
            self.pos += 2 + extra;
            if prop == Prop::Tool && matches!(self.peek(), Some(t) if t.is_lit("item")) {
                self.pos += 1;
            }
            e = self.prop(prop, e)?;
        }
        Ok(e)
    }

    /// `<text> with [all|first] "a" replaced (with|by) "b"`. Leaves the
    /// position alone and returns `e` when the words after it do not fit.
    fn replaced_postfix(&mut self, e: Expr) -> Expr {
        if !matches!(self.toks.get(self.pos), Some(t) if t.is_lit("with")) {
            return e;
        }
        let save = self.pos;
        self.pos += 1;
        let mut all = true;
        if matches!(self.peek(), Some(t) if t.is_lit("all")) {
            self.pos += 1;
        } else if matches!(self.peek(), Some(t) if t.is_lit("first")) {
            self.pos += 1;
            all = false;
        }
        if let Ok(from) = self.primary_base()
            && matches!(self.peek(), Some(t) if t.is_lit("replaced"))
        {
            self.pos += 1;
            if matches!(self.peek(), Some(t) if t.is_lit("with") || t.is_lit("by")) {
                self.pos += 1;
                if let Ok(to) = self.primary_base() {
                    return Expr::Replaced(Box::new(e), Box::new(from), Box::new(to), all);
                }
            }
        }
        self.pos = save;
        e
    }

    /// After a number: `5 minutes`, `1 hour and 30 seconds`, `1 hour, 5 minutes`.
    fn span_literal(&mut self, first: f64) -> Option<Expr> {
        let unit_at = |p: usize| match self.toks.get(p) {
            Some(Tok::Word(w)) => crate::timespans::unit_ms(w),
            _ => None,
        };
        let mut total = crate::timespans::span_ms(first, unit_at(self.pos)?)?;
        let mut pos = self.pos + 1;
        loop {
            let mut p = pos;
            if matches!(self.toks.get(p), Some(t) if t.is_lit("and") || t.is_lit(",")) {
                p += 1;
            }
            // Running out of tokens mid-literal means a longer span may parse.
            if p + 1 >= self.toks.len() && p > pos {
                self.hit_end.set(true);
            }
            let (Some(Tok::Num(n)), Some(u)) = (self.toks.get(p), unit_at(p + 1)) else {
                break;
            };
            total = total.checked_add(crate::timespans::span_ms(*n, u)?)?;
            pos = p + 2;
        }
        self.pos = pos;
        Some(Expr::Span(total))
    }

    /// `minutes of`, `total seconds of`.
    fn span_part_ahead(&self, first: &str) -> bool {
        let of_at = if first == "total" {
            if !matches!(self.toks.get(self.pos), Some(Tok::Word(w)) if crate::timespans::unit_ms(w).is_some())
            {
                return false;
            }
            self.pos + 1
        } else {
            self.pos
        };
        matches!(self.toks.get(of_at), Some(t) if t.is_lit("of"))
    }

    fn span_part(&mut self, first: &str) -> Result<Expr, String> {
        let (total, unit) = if first == "total" {
            let Some(Tok::Word(w)) = self.toks.get(self.pos).cloned() else {
                return Err("expected a unit after 'total'".into());
            };
            self.pos += 1;
            (true, w)
        } else {
            (false, first.to_string())
        };
        let ms = crate::timespans::unit_ms(&unit).ok_or("unknown time unit")?;
        self.pos += 1; // `of`
        let inner = self.primary()?;
        if !matches!(inner.ty(self.env), Ty::Timespan | Ty::Object) {
            return Err(format!(
                "'{unit} of' needs a timespan, got {}",
                inner.ty(self.env).name()
            ));
        }
        Ok(Expr::SpanPart(ms, total, Box::new(inner)))
    }

    fn prop(&self, prop: Prop, of: Expr) -> Result<Expr, String> {
        let t = of.ty(self.env);
        if !prop.takes(t) {
            return Err(format!("can't take the {prop:?} of a {}", t.name()).to_lowercase());
        }
        if prop == Prop::Name && t == Ty::Player {
            return Ok(Expr::NameOf(Box::new(of)));
        }
        Ok(Expr::Prop(prop, Box::new(of)))
    }

    /// `max health of`, `food level of`, ... starting at the word just read.
    /// Marks the end as hit when more tokens could complete it.
    fn hf_head(&self) -> Option<(Prop, usize)> {
        let start = self.pos - 1;
        if start + 2 >= self.toks.len() {
            self.hit_end.set(true);
        }
        let (p, n) = crate::health_food::words_from(self.toks, start)?;
        matches!(self.toks.get(start + n), Some(t) if t.is_lit("of")).then_some((p, n))
    }

    /// `<word> of <value>` for the properties above.
    fn prop_of(&mut self, prop: Prop) -> Result<Expr, String> {
        // already past the property word
        if prop == Prop::Tool && matches!(self.peek(), Some(t) if t.is_lit("item")) {
            self.pos += 1;
        }
        if !matches!(self.peek(), Some(t) if t.is_lit("of")) {
            return Err("expected 'of'".into());
        }
        self.pos += 1;
        let inner = self.primary()?;
        self.prop(prop, inner)
    }

    /// Longest run of words that names an item, like `diamond sword`. Plural
    /// `diamonds` and `torches` work too.
    fn item(&mut self, first: &str) -> Option<Expr> {
        let lower = first.to_ascii_lowercase();
        if !crate::items::could_start(&lower) {
            return None;
        }
        let start = self.pos - 1;
        let mut words = vec![lower];
        let mut i = self.pos;
        while let Some(Tok::Word(w)) = self.toks.get(i) {
            if words.len() >= 5 {
                break;
            }
            words.push(w.to_ascii_lowercase());
            i += 1;
        }
        // More words after the end could make a longer item name.
        if i >= self.toks.len() && words.len() < 5 {
            self.hit_end.set(true);
        }
        for n in (1..=words.len()).rev() {
            let joined = words.get(..n).unwrap_or_default().join("_");
            let mut candidates = vec![joined.clone()];
            if let Some(s) = joined.strip_suffix("es") {
                candidates.push(s.to_string());
            }
            if let Some(s) = joined.strip_suffix('s') {
                candidates.push(s.to_string());
            }
            if let Some(id) = candidates.into_iter().find(|c| crate::items::is_known(c)) {
                self.pos = start + n;
                return Some(Expr::Item { id, count: 1 });
            }
        }
        None
    }

    fn primary_base(&mut self) -> Result<Expr, String> {
        let Some(tok) = self.peek().cloned() else {
            return Err("missing expression".into());
        };
        self.pos += 1;
        match tok {
            Tok::Num(n) => {
                if let Some(span) = self.span_literal(n) {
                    return Ok(span);
                }
                // `2 diamonds`: an amount of an item.
                if self.pos >= self.toks.len() {
                    self.hit_end.set(true);
                }
                if let Some(Tok::Word(w)) = self.toks.get(self.pos).cloned()
                    && n >= 1.0
                    && n.fract() == 0.0
                    && n <= f64::from(u32::MAX)
                {
                    let save = self.pos;
                    self.pos += 1;
                    if let Some(Expr::Item { id, .. }) = self.item(&w) {
                        return Ok(Expr::Item { id, count: n as u32 });
                    }
                    self.pos = save;
                }
                Ok(Expr::Num(n))
            }
            Tok::Str(s) => Ok(Expr::Text(parse_template(&s, self.env)?)),
            Tok::Var(v) => {
                let var = parse_varref(&v, self.env)?;
                if var.is_list() && matches!(self.peek(), Some(t) if t.is_lit("where")) {
                    return self.list_expr(self.pos - 1);
                }
                Ok(Expr::Var(var))
            }
            Tok::Sym("(") => {
                let e = self.sum()?;
                if !self.eat_sym(")") {
                    return Err("missing ')'".into());
                }
                Ok(e)
            }
            Tok::Word(w) => {
                let lw = w.to_ascii_lowercase();
                if let Some(e) = self.text_fn(&lw)? {
                    return Ok(e);
                }
                match lw.as_str() {
                    "the" => self.primary(),
                    _ if crate::gamemode::is_mode_word(&lw) => Ok(Expr::Text(vec![Part::Lit(lw)])),
                    "game"
                        if matches!(self.peek(), Some(t) if t.is_lit("mode"))
                            && matches!(self.toks.get(self.pos + 1), Some(t) if t.is_lit("of")) =>
                    {
                        self.pos += 1;
                        self.prop_of(Prop::Gamemode)
                    }
                    "colored" | "coloured" | "uncolored" | "uncoloured" | "unformatted" => {
                        if self.pos >= self.toks.len() {
                            self.hit_end.set(true);
                        }
                        let inner = self.primary()?;
                        Ok(Expr::Color(!lw.starts_with('u'), Box::new(inner)))
                    }
                    "time" if matches!(self.peek(), Some(t) if t.is_lit("since") || t.is_lit("until")) => {
                        let since = matches!(self.peek(), Some(t) if t.is_lit("since"));
                        self.pos += 1;
                        let d = self.primary()?;
                        if !matches!(d.ty(self.env), Ty::Date | Ty::Object) {
                            return Err(format!(
                                "'time since' needs a date, got {}",
                                d.ty(self.env).name()
                            ));
                        }
                        Ok(Expr::Since(Box::new(d), since))
                    }
                    "total" | "tick" | "ticks" | "millisecond" | "milliseconds" | "second" | "seconds"
                    | "minute" | "minutes" | "hour" | "hours" | "day" | "days" | "week" | "weeks"
                    | "month" | "months" | "year" | "years"
                        if self.span_part_ahead(&lw) =>
                    {
                        self.span_part(&lw)
                    }
                    "true" | "yes" => Ok(Expr::Bool(true)),
                    "false" | "no" => Ok(Expr::Bool(false)),
                    "player" if self.env.in_function => {
                        Err("'player' doesn't exist inside a function; pass the player as a parameter".into())
                    }
                    "player" | "victim" if self.env.no_player => Err(format!(
                        "'{lw}' doesn't exist in {}: nobody started it",
                        self.env.trigger_name
                    )),
                    "sender" | "executor" | "console" if self.env.in_function => Err(format!(
                        "'{lw}' doesn't exist inside a function; pass it as a parameter"
                    )),
                    "sender" | "executor" => Ok(Expr::Sender),
                    "console" => Ok(Expr::Console),
                    "command"
                        if !self.env.in_function
                            && matches!(self.peek(), Some(t) if t.is_lit("sender") || t.is_lit("executor")) =>
                    {
                        self.pos += 1;
                        Ok(Expr::Sender)
                    }
                    "me" | "myself" if self.env.in_function => {
                        Err("'me' doesn't exist inside a function; pass the player as a parameter".into())
                    }
                    "me" | "myself" if self.env.no_player => Err(format!(
                        "'{lw}' doesn't exist in {}: nobody started it",
                        self.env.trigger_name
                    )),
                    "me" | "myself" => Ok(Expr::Player),
                    "player" => Ok(Expr::Player),
                    "victim" if self.env.has_damage || self.env.has_attacker => Ok(Expr::Player),
                    "damage" if self.env.has_damage && !self.env.in_function => Ok(Expr::Damage),
                    "attacker" | "damager" if self.env.has_attacker && !self.env.in_function => {
                        Ok(Expr::Attacker)
                    }
                    w if w.starts_with("event-") && !self.env.in_function && self.env.ext.is_some() => {
                        let ev = self.env.ext.unwrap_or(crate::w2_events_4::ExtEvent::WorldInit);
                        match ev.slot_index(&w["event-".len()..]) {
                            Some(i) => Ok(Expr::EventVal(i as u8)),
                            None => Err(format!(
                                "'on {}' has no value '{w}' (it has: {})",
                                ev.name(),
                                if ev.slots().is_empty() {
                                    "none".to_string()
                                } else {
                                    ev.slots().iter().map(|(n, _)| format!("event-{n}")).collect::<Vec<_>>().join(", ")
                                }
                            )),
                        }
                    }
                    "clicked" | "event-block"
                        if self.env.has_block
                            && !self.env.in_function
                            && (lw == "event-block"
                                || matches!(self.peek(), Some(t) if t.is_lit("block"))) =>
                    {
                        if lw == "clicked" {
                            self.pos += 1;
                        }
                        Ok(Expr::ClickedBlock)
                    }
                    "message" => {
                        if self.env.has_message && !self.env.in_function {
                            Ok(Expr::Message)
                        } else {
                            Err(format!(
                                "'message' only exists in 'on chat', not in {}",
                                self.env.trigger_name
                            ))
                        }
                    }
                    "now" => Ok(Expr::Date(DateOp::Now, Vec::new())),
                    "unix"
                        if matches!(self.peek(), Some(t) if t.is_lit("timestamp"))
                            && matches!(self.peek_at(1), Some(t) if t.is_lit("of")) =>
                    {
                        self.pos += 2;
                        let d = self.primary()?;
                        if !matches!(d.ty(self.env), Ty::Date | Ty::Object) {
                            return Err("'unix timestamp of' needs a date, like 'unix timestamp of now'".into());
                        }
                        Ok(Expr::Date(DateOp::ToUnix, vec![d]))
                    }
                    "unix"
                        if matches!(self.peek(), Some(t) if t.is_lit("date"))
                            && matches!(self.peek_at(1), Some(t) if t.is_lit("of")) =>
                    {
                        self.pos += 2;
                        self.date_from_unix()
                    }
                    "date"
                        if matches!(self.peek(), Some(t) if t.is_lit("from"))
                            && matches!(self.peek_at(1), Some(t) if t.is_lit("unix"))
                            && matches!(self.peek_at(2), Some(t) if t.is_lit("timestamp")) =>
                    {
                        self.pos += 3;
                        self.date_from_unix()
                    }
                    "formatted" => {
                        // `formatted now as "..."` / `formatted {date}`
                        let d = self.primary_base()?;
                        let as_next = matches!(self.peek(), Some(t) if t.is_lit("as"));
                        match d.ty(self.env) {
                            Ty::Date => self.date_format(d),
                            Ty::Object | Ty::Text if as_next => self.date_format(d),
                            Ty::Object | Ty::Text => Ok(Expr::Color(true, Box::new(d))),
                            _ => Err("'formatted' needs a date or text, like 'formatted now'".into()),
                        }
                    }
                    "arg" | "argument" => self.arg(1),
                    "join" if matches!(self.peek(), Some(Tok::Var(_))) => {
                        let Some(Tok::Var(v)) = self.peek().cloned() else {
                            return Err("expected 'join {list::*} with \", \"'".into());
                        };
                        self.pos += 1;
                        let var = parse_varref(&v, self.env)?;
                        if !var.is_list() {
                            return Err("'join' needs a list variable like {x::*}".into());
                        }
                        if !matches!(self.peek(), Some(t) if t.is_lit("with") || t.is_lit("by")) {
                            return Err("expected 'join {list::*} with \", \"'".into());
                        }
                        self.pos += 1;
                        let sep = self.primary_base()?;
                        Ok(Expr::Join(var, Box::new(sep)))
                    }
                    "split" if self.peek().is_some() && !matches!(self.peek(), Some(t) if t.is_lit("(")) => {
                        let text = self.primary_base()?;
                        if !matches!(self.peek(), Some(t) if t.is_lit("at") || t.is_lit("by")) {
                            return Err("expected 'split <text> at <separator>'".into());
                        }
                        self.pos += 1;
                        let sep = self.primary_base()?;
                        Ok(Expr::Split(Box::new(text), Box::new(sep)))
                    }
                    "length" if matches!(self.peek(), Some(t) if t.is_lit("of")) => {
                        self.pos += 1;
                        let inner = self.primary_base()?;
                        Ok(Expr::Length(Box::new(inner)))
                    }
                    "size" | "amount" | "count" | "number"
                        if matches!(self.peek(), Some(t) if t.is_lit("of"))
                            && matches!(self.toks.get(self.pos + 1), Some(t) if t.is_lit("characters") || t.is_lit("letters")) =>
                    {
                        self.pos += 2;
                        if !matches!(self.peek(), Some(t) if t.is_lit("in") || t.is_lit("of")) {
                            return Err(format!("expected '{lw} of characters in <text>'"));
                        }
                        self.pos += 1;
                        let inner = self.primary_base()?;
                        Ok(Expr::Length(Box::new(inner)))
                    }
                    "number" if matches!(self.peek(), Some(t) if t.is_lit("of")) => self.size_of(&lw),
                    "indices" | "indexes" if matches!(self.peek(), Some(t) if t.is_lit("of")) => {
                        Ok(Expr::Indices(self.list_after_of(&lw)?))
                    }
                    "first" | "last" if matches!(self.peek(), Some(t) if t.is_lit("element")) => {
                        self.pos += 1;
                        let sel = if lw == "first" { ElemSel::First } else { ElemSel::Last };
                        Ok(Expr::Elem(sel, self.list_after_of(&format!("{lw} element"))?))
                    }
                    "element" if !matches!(self.peek(), Some(t) if t.is_lit("of")) => {
                        let n = self.unary()?;
                        if !Ty::Number.accepts(n.ty(self.env)) {
                            return Err("'element N of' needs a number".into());
                        }
                        Ok(Expr::Elem(ElemSel::Nth(Box::new(n)), self.list_after_of("element N")?))
                    }
                    "size" | "amount" | "count" => self.size_of(&lw),
                    "sorted" | "reversed" | "shuffled" if self.peek().is_some() => {
                        self.list_expr(self.pos - 1)
                    }
                    "random" if matches!(self.peek(), Some(Tok::Num(_) | Tok::Var(_))) => {
                        self.list_expr(self.pos - 1)
                    }
                    "sum" | "average" | "avg" | "min" | "max" | "minimum" | "maximum"
                        if matches!(self.peek(), Some(t) if t.is_lit("of")) =>
                    {
                        self.pos += 1;
                        let agg = match lw.as_str() {
                            "sum" => Agg::Sum,
                            "average" | "avg" => Agg::Avg,
                            "min" | "minimum" => Agg::Min,
                            _ => Agg::Max,
                        };
                        self.try_agg(agg).unwrap_or_else(|| Err("missing list".into()))
                    }
                    "location"
                        if matches!(self.peek(), Some(t) if t.is_lit("("))
                            && !self.has_function("location") =>
                    {
                        self.pos += 1;
                        let x = self.sum()?;
                        let mut parts = vec![x];
                        while self.eat_sym(",") {
                            parts.push(self.sum()?);
                        }
                        if !self.eat_sym(")") {
                            return Err("missing ')' after location(x, y, z)".into());
                        }
                        if !(3..=4).contains(&parts.len()) {
                            return Err("location(x, y, z) or location(x, y, z, world)".into());
                        }
                        for p in parts.iter().take(3) {
                            if !Ty::Number.accepts(p.ty(self.env)) {
                                return Err("location() needs numbers for x, y and z".into());
                            }
                        }
                        let mut it = parts.into_iter();
                        let (Some(x), Some(y), Some(z)) = (it.next(), it.next(), it.next()) else {
                            return Err("location(x, y, z)".into());
                        };
                        let w = it.next().map(Box::new);
                        Ok(Expr::Loc(Box::new(x), Box::new(y), Box::new(z), w))
                    }
                    "world" if matches!(self.peek(), Some(Tok::Str(_))) => {
                        let name = self.primary()?;
                        Ok(Expr::WorldNamed(Box::new(name)))
                    }
                    "random" if matches!(self.peek(), Some(t) if t.is_lit("element")) => {
                        self.pos += 1;
                        if !matches!(self.peek(), Some(t) if t.is_lit("of")) {
                            return Err("expected 'random element of {list::*}'".into());
                        }
                        self.pos += 1;
                        match self.peek().cloned() {
                            Some(Tok::Var(v)) => {
                                self.pos += 1;
                                let var = parse_varref(&v, self.env)?;
                                if !var.is_list() {
                                    return Err(
                                        "'random element of' needs a list variable like {x::*}".into()
                                    );
                                }
                                Ok(Expr::RandomOf(var))
                            }
                            _ => Err("'random element of' needs a list variable like {x::*}".into()),
                        }
                    }
                    "x" | "y" | "z"
                        if matches!(self.peek(), Some(t) if t.is_lit("-") || t.is_lit("coordinate"))
                            && matches!(self.toks.get(self.pos + 1), Some(t) if t.is_lit("coordinate") || t.is_lit("of")) =>
                    {
                        // `x-coordinate of` / `x coordinate of`
                        self.eat_sym("-");
                        if matches!(self.peek(), Some(t) if t.is_lit("coordinate")) {
                            self.pos += 1;
                        }
                        let prop = Prop::from_word(&lw).ok_or("unknown property")?;
                        self.prop_of(prop)
                    }
                    "random" if matches!(self.peek(), Some(Tok::Word(w)) if matches!(w.to_ascii_lowercase().as_str(), "integer" | "number" | "int")) =>
                    {
                        let int = !matches!(self.peek(), Some(t) if t.is_lit("number"));
                        self.pos += 1;
                        if !matches!(self.peek(), Some(t) if t.is_lit("between")) {
                            return Err("expected 'random integer between <number> and <number>'".into());
                        }
                        self.pos += 1;
                        let a = self.unary()?;
                        if !matches!(self.peek(), Some(t) if t.is_lit("and")) {
                            return Err("expected 'random integer between <number> and <number>'".into());
                        }
                        self.pos += 1;
                        let b = self.unary()?;
                        for side in [&a, &b] {
                            if !Ty::Number.accepts(side.ty(self.env)) {
                                return Err("random needs numbers".into());
                            }
                        }
                        Ok(Expr::Random(int, Box::new(a), Box::new(b)))
                    }
                    "distance" if matches!(self.peek(), Some(t) if t.is_lit("between")) => {
                        self.pos += 1;
                        let a = self.primary()?;
                        if !matches!(self.peek(), Some(t) if t.is_lit("and")) {
                            return Err("expected 'distance between <location> and <location>'".into());
                        }
                        self.pos += 1;
                        let b = self.primary()?;
                        for side in [&a, &b] {
                            let t = side.ty(self.env);
                            if !matches!(t, Ty::Location | Ty::Player | Ty::Object) {
                                return Err(format!("distance needs locations, got a {}", t.name()));
                            }
                        }
                        Ok(Expr::Distance(Box::new(a), Box::new(b)))
                    }
                    "level" | "total"
                        if matches!(self.peek(), Some(Tok::Word(w2)) if crate::xp_level::two_word(&lw, &w2.to_ascii_lowercase()).is_some()) =>
                    {
                        let prop = match self.peek() {
                            Some(Tok::Word(w2)) => crate::xp_level::two_word(&lw, &w2.to_ascii_lowercase()),
                            _ => None,
                        }
                        .ok_or("unknown property")?;
                        self.pos += 1;
                        self.prop_of(prop)
                    }
                    "location" | "position" | "world" | "uuid" | "tool" | "health" | "gamemode" | "x" | "y" | "z"
                    | "held" | "level" | "xp" | "exp" | "experience"
                        if matches!(self.peek(), Some(t) if t.is_lit("of") || t.is_lit("item")) =>
                    {
                        let prop = Prop::from_word(&lw).ok_or("unknown property")?;
                        self.prop_of(prop)
                    }
                    "max" | "food" | "hunger" | "saturation" if self.hf_head().is_some() => {
                        let (prop, n) = self.hf_head().ok_or("unknown property")?;
                        self.pos += n - 1;
                        self.prop_of(prop)
                    }
                    "name" => {
                        if !matches!(self.peek(), Some(t) if t.is_lit("of")) {
                            return Err("expected 'name of <player>'".into());
                        }
                        self.pos += 1;
                        let inner = self.primary()?;
                        let t = inner.ty(self.env);
                        if !(t == Ty::Player || t == Ty::Object) {
                            return Err(format!("'name of' needs a player, got {}", t.name()));
                        }
                        Ok(Expr::NameOf(Box::new(inner)))
                    }
                    _ if lw.starts_with("loop-") => self.loop_expr(&lw),
                    _ if matches!(self.peek(), Some(t) if t.is_lit("("))
                        && !self.has_function(&lw)
                        && MathFn::from_name(&lw).is_some() =>
                    {
                        let (f, n) = MathFn::from_name(&lw).ok_or("unknown math function")?;
                        self.pos += 1;
                        let mut args = vec![self.top()?];
                        while self.eat_sym(",") {
                            args.push(self.top()?);
                        }
                        if !self.eat_sym(")") {
                            return Err(format!("missing ')' after {lw}("));
                        }
                        if args.len() != n {
                            return Err(format!("{lw}() takes {n} number(s), got {}", args.len()));
                        }
                        if args.iter().any(|a| !Ty::Number.accepts(a.ty(self.env))) {
                            return Err(format!("{lw}() needs numbers"));
                        }
                        Ok(Expr::Math(f, args))
                    }
                    _ if matches!(self.peek(), Some(t) if t.is_lit("(")) => self.call(&w),
                    _ => {
                        if let Some(d) = lw.strip_prefix("arg-") {
                            let i: usize = d.parse().map_err(|_| format!("bad argument '{w}'"))?;
                            return self.arg(i);
                        }
                        if let Some(item) = self.item(&w) {
                            return Ok(item);
                        }
                        Err(format!("unknown expression '{w}'"))
                    }
                }
            }
            other => Err(format!("unexpected {}", describe(&other))),
        }
    }

    fn is_word(&self, at: usize, w: &str) -> bool {
        if self.pos + at >= self.toks.len() {
            self.hit_end.set(true);
        }
        matches!(self.toks.get(self.pos + at), Some(t) if t.is_lit(w))
    }

    /// `x in upper case`, `x in lower case`, `x in proper case`.
    fn in_case(&mut self, e: Expr) -> Expr {
        use crate::text_case_substring::TextFn;
        if !matches!(self.toks.get(self.pos), Some(t) if t.is_lit("in")) {
            return e;
        }
        if !self.is_word(2, "case") {
            return e;
        }
        let f = match self.toks.get(self.pos + 1) {
            Some(t) if t.is_lit("upper") => TextFn::Upper,
            Some(t) if t.is_lit("lower") => TextFn::Lower,
            Some(t) if t.is_lit("proper") => TextFn::Proper,
            _ => return e,
        };
        self.pos += 3;
        Expr::TextFn(f, vec![e])
    }

    fn text_arg(&mut self, what: &str) -> Result<Expr, String> {
        let e = self.primary()?;
        if !Ty::Text.accepts(e.ty(self.env)) {
            return Err(format!("{what} needs text"));
        }
        Ok(e)
    }

    fn num_arg(&mut self, what: &str) -> Result<Expr, String> {
        let e = self.sum()?;
        if !Ty::Number.accepts(e.ty(self.env)) {
            return Err(format!("{what} needs a number"));
        }
        Ok(e)
    }

    fn expected(&self, usage: &str) -> String {
        if self.pos >= self.toks.len() {
            self.hit_end.set(true);
        }
        format!("expected '{usage}'")
    }

    /// Text functions that start with a word (`uppercase of x`, `substring of x from 1 to 3`,
    /// `first 3 characters of x`, `index of "a" in x`, `nl`, `trimmed x`, `reversed x`).
    /// Returns `None` when the words are not one of these forms.
    fn text_fn(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        use crate::text_case_substring::TextFn;
        if !matches!(
            lw,
            "nl" | "newline"
                | "new"
                | "uppercase"
                | "lowercase"
                | "upper"
                | "lower"
                | "capitalized"
                | "capitalised"
                | "trimmed"
                | "trim"
                | "reversed"
                | "substring"
                | "subtext"
                | "first"
                | "last"
                | "character"
                | "char"
                | "index"
        ) || matches!(self.toks.get(self.pos), Some(t) if t.is_lit("("))
        {
            return Ok(None);
        }
        let f = |f: TextFn, args: Vec<Expr>| Ok(Some(Expr::TextFn(f, args)));
        match lw {
            "nl" | "newline" => Ok(Some(Expr::Text(vec![Part::Lit("\n".into())]))),
            "new" if self.is_word(0, "line") => {
                self.pos += 1;
                Ok(Some(Expr::Text(vec![Part::Lit("\n".into())])))
            }
            "uppercase" | "lowercase" if self.is_word(0, "of") => {
                self.pos += 1;
                let a = self.text_arg(&format!("{lw} of"))?;
                f(if lw == "uppercase" { TextFn::Upper } else { TextFn::Lower }, vec![a])
            }
            "upper" | "lower" if self.is_word(0, "case") && self.is_word(1, "of") => {
                self.pos += 2;
                let a = self.text_arg(&format!("{lw} case of"))?;
                f(if lw == "upper" { TextFn::Upper } else { TextFn::Lower }, vec![a])
            }
            "capitalized" | "capitalised" | "trimmed" | "trim"
                if matches!(self.toks.get(self.pos), Some(Tok::Str(_) | Tok::Var(_) | Tok::Word(_))) =>
            {
                if self.is_word(0, "of") {
                    self.pos += 1;
                }
                let a = self.text_arg(lw)?;
                let k = if lw.starts_with("cap") { TextFn::Capitalized } else { TextFn::Trim };
                f(k, vec![a])
            }
            "reversed" => {
                let save = self.pos;
                if self.is_word(0, "text") {
                    self.pos += 1;
                    if self.is_word(0, "of") {
                        self.pos += 1;
                    }
                } else if !matches!(self.toks.get(self.pos), Some(Tok::Str(_))) {
                    // `reversed {x::*}` and friends belong to lists.
                    return Ok(None);
                }
                let a = self.text_arg("reversed")?;
                if matches!(&a, Expr::Var(v) if v.is_list()) {
                    self.pos = save;
                    return Ok(None);
                }
                f(TextFn::Reverse, vec![a])
            }
            "substring" | "subtext" if self.is_word(0, "of") => {
                self.pos += 1;
                let t = self.text_arg(lw)?;
                if !self.is_word(0, "from") {
                    return Err(self.expected(&format!("{lw} of <text> from <number> to <number>")));
                }
                self.pos += 1;
                if self.is_word(0, "characters") || self.is_word(0, "character") {
                    self.pos += 1;
                }
                let a = self.num_arg(lw)?;
                let mut args = vec![t, a];
                if self.is_word(0, "to") {
                    self.pos += 1;
                    args.push(self.num_arg(lw)?);
                }
                f(TextFn::Sub, args)
            }
            "first" | "last"
                if matches!(self.toks.get(self.pos), Some(Tok::Num(_) | Tok::Var(_)))
                    || (self.is_word(0, "index") && self.is_word(1, "of")) =>
            {
                if self.is_word(0, "index") {
                    self.pos += 2;
                    return self.index_args(lw == "last");
                }
                let n = self.unary()?;
                if !Ty::Number.accepts(n.ty(self.env)) {
                    return Err(format!("'{lw}' needs a number of characters"));
                }
                let usage = format!("{lw} <number> characters of <text>");
                if !(self.is_word(0, "characters") || self.is_word(0, "character")) {
                    return Err(self.expected(&usage));
                }
                self.pos += 1;
                if !self.is_word(0, "of") {
                    return Err(self.expected(&usage));
                }
                self.pos += 1;
                let t = self.text_arg(lw)?;
                f(if lw == "first" { TextFn::First } else { TextFn::Last }, vec![n, t])
            }
            "character" | "char"
                if self.is_word(0, "at")
                    && (self.is_word(1, "index") || matches!(self.toks.get(self.pos + 1), Some(Tok::Num(_)))) =>
            {
                self.pos += 1;
                if self.is_word(0, "index") {
                    self.pos += 1;
                }
                let n = self.unary()?;
                if !Ty::Number.accepts(n.ty(self.env)) {
                    return Err("'character at index' needs a number".into());
                }
                if !self.is_word(0, "of") {
                    return Err(self.expected("character at index <number> of <text>"));
                }
                self.pos += 1;
                let t = self.text_arg("character at index")?;
                f(TextFn::CharAt, vec![n, t])
            }
            "index" if self.is_word(0, "of") => {
                self.pos += 1;
                self.index_args(false)
            }
            _ => Ok(None),
        }
    }

    /// After `index of`: `<text> in <text>`.
    fn index_args(&mut self, last: bool) -> Result<Option<Expr>, String> {
        use crate::text_case_substring::TextFn;
        let a = self.text_arg("index of")?;
        if !self.is_word(0, "in") {
            return Err(self.expected("index of <text> in <text>"));
        }
        self.pos += 1;
        let b = self.text_arg("index of")?;
        let k = if last { TextFn::IndexLast } else { TextFn::IndexFirst };
        Ok(Some(Expr::TextFn(k, vec![a, b])))
    }

    fn date_from_unix(&mut self) -> Result<Expr, String> {
        let n = self.primary()?;
        if !Ty::Number.accepts(n.ty(self.env)) {
            return Err("'date from unix timestamp' needs a number of seconds".into());
        }
        Ok(Expr::Date(DateOp::FromUnix, vec![n]))
    }

    /// After a date and the word `formatted`: optional `as "pattern"`.
    fn date_format(&mut self, d: Expr) -> Result<Expr, String> {
        let mut args = vec![d];
        if matches!(self.peek(), Some(t) if t.is_lit("as")) {
            self.pos += 1;
            let f = self.primary()?;
            if !Ty::Text.accepts(f.ty(self.env)) {
                return Err("'formatted as' needs a format text like \"yyyy-MM-dd\"".into());
            }
            args.push(f);
        }
        Ok(Expr::Date(DateOp::Format, args))
    }

    /// `loop-value`, `loop-value-2`, `loop-player`, `loop-index`, `loop-number`, `loop-iteration`.
    /// `of {x::*}` after a word like `indices`; the cursor is on `of`.
    fn list_after_of(&mut self, what: &str) -> Result<VarRef, String> {
        if !matches!(self.peek(), Some(t) if t.is_lit("of")) {
            return Err(format!("expected '{what} of {{list::*}}'"));
        }
        self.pos += 1;
        if let Some(Tok::Var(v)) = self.peek().cloned() {
            self.pos += 1;
            let var = parse_varref(&v, self.env)?;
            if var.is_list() {
                return Ok(var);
            }
        }
        Err(format!("'{what} of' needs a list variable like {{x::*}}"))
    }

    /// `size of`, `amount of`, `number of`.
    fn size_of(&mut self, word: &str) -> Result<Expr, String> {
        if matches!(self.peek(), Some(t) if t.is_lit("of")) {
            let plain_var = matches!(self.toks.get(self.pos + 1), Some(Tok::Var(_)))
                && !matches!(self.toks.get(self.pos + 2), Some(t) if t.is_lit("where"));
            if !plain_var {
                let save = self.pos;
                self.pos += 1;
                if let Some(e) = self.try_agg(Agg::Size) {
                    return e;
                }
                self.pos = save;
            }
        }
        Ok(Expr::Size(self.list_after_of(word)?))
    }

    fn loop_expr(&self, word: &str) -> Result<Expr, String> {
        let loops = &self.env.loops;
        if loops.is_empty() {
            return Err(format!("'{word}' only works inside a loop"));
        }
        let (name, nth) = match word.rsplit_once('-') {
            Some((head, d)) if d.chars().all(|c| c.is_ascii_digit()) && head.contains('-') => {
                (head, d.parse::<usize>().ok())
            }
            _ => (word, None),
        };
        let want: Option<LoopKind> = match name {
            "loop-value" | "loop-iteration" | "loop-counter" => None,
            "loop-player" => Some(LoopKind::Players),
            "loop-index" => Some(LoopKind::List),
            "loop-number" => Some(LoopKind::Times),
            _ => {
                return Err(format!(
                    "unknown loop value '{word}' (use loop-value, loop-player, loop-index, loop-number or loop-iteration)"
                ));
            }
        };
        let depth = match nth {
            Some(n) => {
                if n == 0 || n > loops.len() {
                    return Err(format!("'{word}': there are only {} loop(s) here", loops.len()));
                }
                n - 1
            }
            None => match want {
                None => loops.len() - 1,
                Some(k) => match loops.iter().rposition(|l| *l == k) {
                    Some(d) => d,
                    None => {
                        return Err(format!(
                            "'{word}' needs a surrounding {} loop",
                            match k {
                                LoopKind::Players => "'loop all players'",
                                LoopKind::List => "list ('loop {x::*}')",
                                LoopKind::Times => "'loop N times'",
                                LoopKind::Values => "value",
                            }
                        ));
                    }
                },
            },
        };
        if let (Some(k), Some(got)) = (want, loops.get(depth))
            && *got != k
        {
            return Err(format!("'{word}': loop {} is not that kind of loop", depth + 1));
        }
        Ok(match name {
            "loop-index" => Expr::LoopIndex(depth),
            "loop-iteration" | "loop-counter" => Expr::LoopIter(depth),
            _ => Expr::LoopValue(depth),
        })
    }

    /// A list expression starting at `start` (a word or list variable).
    fn list_expr(&mut self, start: usize) -> Result<Expr, String> {
        match parse_src_at(self.toks, start, self.env, false) {
            Ok((s, end)) => {
                self.pos = end;
                Ok(Expr::Items(Box::new(s)))
            }
            Err(e) => {
                self.hit_end.set(true);
                Err(e)
            }
        }
    }

    /// The list after `sum of` / `size of`: a number computed from it.
    /// `None` when it is a single plain value (not a list at all).
    fn try_agg(&mut self, agg: Agg) -> Option<Result<Expr, String>> {
        match parse_src_at(self.toks, self.pos, self.env, false) {
            Ok((ListSrc::One(_), _)) if agg == Agg::Size => None,
            Ok((s, end)) => {
                self.pos = end;
                Some(Ok(Expr::ListAgg(agg, Box::new(s))))
            }
            Err(e) => {
                self.hit_end.set(true);
                Some(Err(e))
            }
        }
    }

    fn has_function(&self, name: &str) -> bool {
        self.env
            .functions
            .iter()
            .any(|s| s.name.eq_ignore_ascii_case(name))
    }

    /// `name(a, b)`: a call of a script function. The `(` is next.
    fn call(&mut self, name: &str) -> Result<Expr, String> {
        let env = self.env;
        let Some(id) = env
            .functions
            .iter()
            .position(|s| s.name.eq_ignore_ascii_case(name))
        else {
            return Err(format!("unknown function '{name}'"));
        };
        self.pos += 1; // (
        let mut args = Vec::new();
        if !self.eat_sym(")") {
            loop {
                let want_list = env.functions.get(id).and_then(|s| s.params.get(args.len())).map(|p| p.1) == Some(Ty::List);
                if want_list {
                    let (src, end) = parse_src_at(self.toks, self.pos, env, false)?;
                    self.pos = end;
                    args.push(Expr::Items(Box::new(src)));
                } else {
                    args.push(self.top()?);
                }
                if self.eat_sym(")") {
                    break;
                }
                if !self.eat_sym(",") {
                    return Err(format!("expected ',' or ')' in the call of {name}"));
                }
            }
        }
        let sig = env.functions.get(id).ok_or("unknown function")?;
        let required = sig.params.iter().filter(|p| !p.2).count();
        if args.len() < required || args.len() > sig.params.len() {
            return Err(format!(
                "{}() takes {} argument(s), got {}",
                sig.name,
                if required == sig.params.len() {
                    required.to_string()
                } else {
                    format!("{required} to {}", sig.params.len())
                },
                args.len()
            ));
        }
        for (i, (a, (pname, pty, _))) in args.iter().zip(&sig.params).enumerate() {
            let got = a.ty(env);
            if !pty.accepts(got) {
                return Err(format!(
                    "argument {} ({}) of {}() must be a {}, got a {}",
                    i + 1,
                    pname.trim_start_matches('_'),
                    sig.name,
                    pty.name(),
                    got.name()
                ));
            }
        }
        Ok(Expr::Call(id, args))
    }

    fn arg(&self, i: usize) -> Result<Expr, String> {
        let n = if self.env.in_function {
            0
        } else {
            self.env.arg_types.len()
        };
        if n == 0 {
            return Err(format!(
                "'arg' only exists in commands with arguments, not in {}",
                self.env.trigger_name
            ));
        }
        if i == 0 || i > n {
            return Err(format!(
                "arg-{i} does not exist, this command has {n} argument(s)"
            ));
        }
        Ok(Expr::Arg(i))
    }
}

/// Parse a `"hello %player%!"` template body. `%%` is a literal percent sign.
pub fn parse_template(s: &str, env: &Env) -> Result<Vec<Part>, String> {
    let mut parts: Vec<Part> = Vec::new();
    let mut lit = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c != '%' {
            lit.push(c);
            i += 1;
            continue;
        }
        if chars.get(i + 1) == Some(&'%') {
            lit.push('%');
            i += 2;
            continue;
        }
        // find closing %, skipping braces
        let mut j = i + 1;
        let mut depth = 0;
        while j < chars.len() {
            match chars[j] {
                '{' => depth += 1,
                '}' => depth -= 1,
                '%' if depth <= 0 => break,
                _ => {}
            }
            j += 1;
        }
        if j >= chars.len() {
            return Err("unclosed '%' in text (use %% for a percent sign)".into());
        }
        let inner: String = chars.get(i + 1..j).unwrap_or_default().iter().collect();
        let e = parse_expr_str(&inner, env).map_err(|e| format!("in %{inner}%: {e}"))?;
        // Constant parts are folded into the surrounding text.
        match e {
            Expr::Num(n) => lit.push_str(&format_num(n)),
            Expr::Text(inner_parts) => {
                for p in inner_parts {
                    match p {
                        Part::Lit(l) => lit.push_str(&l),
                        e @ Part::Expr(_) => {
                            if !lit.is_empty() {
                                parts.push(Part::Lit(std::mem::take(&mut lit)));
                            }
                            parts.push(e);
                        }
                    }
                }
            }
            e => {
                if !lit.is_empty() {
                    parts.push(Part::Lit(std::mem::take(&mut lit)));
                }
                parts.push(Part::Expr(e));
            }
        }
        i = j + 1;
    }
    if !lit.is_empty() {
        parts.push(Part::Lit(lit));
    }
    Ok(parts)
}

impl VarRef {
    /// `{x::*}`: the whole list rather than one variable.
    pub fn is_list(&self) -> bool {
        self.list
    }
}

pub fn parse_varref(raw: &str, env: &Env) -> Result<VarRef, String> {
    let raw = raw.trim();
    let mut parts = parse_template(raw, env)?;
    let local = raw.starts_with('_');
    let list = raw.ends_with("::*");
    let key = match parts.as_mut_slice() {
        [] => Key::Static(String::new()),
        [Part::Lit(l)] => Key::Static(std::mem::take(l).to_ascii_lowercase()),
        _ => Key::Dynamic(parts),
    };
    Ok(VarRef { key, local, list })
}

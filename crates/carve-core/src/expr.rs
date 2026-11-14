//! Expressions: literals, variables, text templates, event values, arithmetic.

use std::cell::Cell;

use crate::lexer::{Tok, lex};
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
    /// Unknown until runtime (variables).
    Object,
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
            Ty::Object => "object",
        }
    }

    /// Can a value of type `got` fill a slot of type `self`?
    pub fn accepts(self, got: Ty) -> bool {
        self == Ty::Object || got == Ty::Object || self == Ty::Text || self == got
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
    /// `colored x` (true) / `uncolored x` (false).
    Color(bool, Box<Expr>),
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
            Prop::Uuid | Prop::Name => Ty::Text,
            Prop::Tool => Ty::Item,
            Prop::Health | Prop::X | Prop::Y | Prop::Z => Ty::Number,
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
}

impl Expr {
    pub fn ty(&self, env: &Env) -> Ty {
        match self {
            Expr::Num(_) | Expr::Neg(_) | Expr::Bin(..) | Expr::LoopIter(_) | Expr::Size(_) => Ty::Number,
            Expr::LoopIndex(_) => Ty::Text,
            Expr::Bool(_) => Ty::Boolean,
            Expr::Call(id, _) => env.functions.get(*id).and_then(|s| s.ret).unwrap_or(Ty::Object),
            Expr::Item { .. } => Ty::Item,
            Expr::Prop(p, _) => p.ty(),
            Expr::Loc(..) => Ty::Location,
            Expr::WorldNamed(_) => Ty::World,
            Expr::Distance(..) | Expr::Random(..) | Expr::Damage => Ty::Number,
            Expr::Attacker => Ty::Player,
            Expr::Math(..) | Expr::ParseNum(_) => Ty::Number,
            Expr::RandomOf(_) => Ty::Object,
            Expr::Color(..) => Ty::Text,
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
            Expr::Arg(i) => env
                .arg_types
                .get(i.wrapping_sub(1))
                .copied()
                .unwrap_or(Ty::Object),
        }
    }
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
        // `player's location`, `player's world`, ...
        while matches!(self.toks.get(self.pos), Some(Tok::Sym("'s"))) {
            let Some(Tok::Word(w)) = self.toks.get(self.pos + 1) else {
                if self.pos + 1 >= self.toks.len() {
                    self.hit_end.set(true);
                }
                return Err("expected a property after 's, like player's location".into());
            };
            let lw = w.to_ascii_lowercase();
            let prop = match lw.as_str() {
                "inventory" => return Err("use 'player's inventory contains <item>' as a condition".into()),
                _ => Prop::from_word(&lw).ok_or_else(|| format!("unknown property '{w}'"))?,
            };
            self.pos += 2;
            if prop == Prop::Tool && matches!(self.peek(), Some(t) if t.is_lit("item")) {
                self.pos += 1;
            }
            e = self.prop(prop, e)?;
        }
        Ok(e)
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
            Tok::Var(v) => Ok(Expr::Var(parse_varref(&v, self.env)?)),
            Tok::Sym("(") => {
                let e = self.sum()?;
                if !self.eat_sym(")") {
                    return Err("missing ')'".into());
                }
                Ok(e)
            }
            Tok::Word(w) => {
                let lw = w.to_ascii_lowercase();
                match lw.as_str() {
                    "the" => self.primary(),
                    "colored" | "coloured" | "formatted" | "uncolored" | "uncoloured" | "unformatted" => {
                        if self.pos >= self.toks.len() {
                            self.hit_end.set(true);
                        }
                        let inner = self.primary()?;
                        Ok(Expr::Color(!lw.starts_with('u'), Box::new(inner)))
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
                    "player" => Ok(Expr::Player),
                    "victim" if self.env.has_damage || self.env.has_attacker => Ok(Expr::Player),
                    "damage" if self.env.has_damage && !self.env.in_function => Ok(Expr::Damage),
                    "attacker" | "damager" if self.env.has_attacker && !self.env.in_function => {
                        Ok(Expr::Attacker)
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
                    "arg" | "argument" => self.arg(1),
                    "size" | "amount" | "count" => {
                        if !matches!(self.peek(), Some(t) if t.is_lit("of")) {
                            return Err(format!("expected '{lw} of {{list::*}}'"));
                        }
                        self.pos += 1;
                        match self.peek().cloned() {
                            Some(Tok::Var(v)) => {
                                self.pos += 1;
                                let var = parse_varref(&v, self.env)?;
                                if !var.is_list() {
                                    return Err(format!("'{lw} of' needs a list variable like {{x::*}}"));
                                }
                                Ok(Expr::Size(var))
                            }
                            _ => Err(format!("'{lw} of' needs a list variable like {{x::*}}")),
                        }
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
                    "location" | "position" | "world" | "uuid" | "tool" | "health" | "x" | "y" | "z"
                    | "held"
                        if matches!(self.peek(), Some(t) if t.is_lit("of") || t.is_lit("item")) =>
                    {
                        let prop = Prop::from_word(&lw).ok_or("unknown property")?;
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

    /// `loop-value`, `loop-value-2`, `loop-player`, `loop-index`, `loop-number`, `loop-iteration`.
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
                args.push(self.top()?);
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

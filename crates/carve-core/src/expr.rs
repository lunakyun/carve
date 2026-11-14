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
}

impl Expr {
    pub fn ty(&self, env: &Env) -> Ty {
        match self {
            Expr::Num(_) | Expr::Neg(_) | Expr::Bin(..) | Expr::LoopIter(_) | Expr::Size(_) => Ty::Number,
            Expr::LoopIndex(_) => Ty::Text,
            Expr::Bool(_) => Ty::Boolean,
            Expr::Call(id, _) => env.functions.get(*id).and_then(|s| s.ret).unwrap_or(Ty::Object),
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
    let e = p.sum().map_err(|m| (m, p.hit_end.get()))?;
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
        let Some(tok) = self.peek().cloned() else {
            return Err("missing expression".into());
        };
        self.pos += 1;
        match tok {
            Tok::Num(n) => Ok(Expr::Num(n)),
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
                    "true" | "yes" => Ok(Expr::Bool(true)),
                    "false" | "no" => Ok(Expr::Bool(false)),
                    "player" if self.env.in_function => {
                        Err("'player' doesn't exist inside a function; pass the player as a parameter".into())
                    }
                    "player" => Ok(Expr::Player),
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
                    _ if matches!(self.peek(), Some(t) if t.is_lit("(")) => self.call(&w),
                    _ => {
                        if let Some(d) = lw.strip_prefix("arg-") {
                            let i: usize = d.parse().map_err(|_| format!("bad argument '{w}'"))?;
                            return self.arg(i);
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
                args.push(self.sum()?);
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

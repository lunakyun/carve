//! Expressions: literals, variables, text templates, event values, arithmetic.

use crate::lexer::{Tok, lex};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Number,
    Text,
    Player,
    /// Unknown until runtime (variables).
    Object,
}

impl Ty {
    pub fn name(self) -> &'static str {
        match self {
            Ty::Number => "number",
            Ty::Text => "text",
            Ty::Player => "player",
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

#[derive(Debug, Clone, PartialEq)]
pub struct VarRef {
    pub name: Vec<Part>,
    pub local: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Text(Vec<Part>),
    Var(VarRef),
    Player,
    Message,
    /// 1-based command argument.
    Arg(usize),
    NameOf(Box<Expr>),
    Neg(Box<Expr>),
    Bin(Box<Expr>, BinOp, Box<Expr>),
}

/// What the current trigger offers to expressions.
#[derive(Debug, Clone, Default)]
pub struct Env {
    pub has_message: bool,
    pub arg_types: Vec<Ty>,
    /// `Some("chat")` etc, used in error messages.
    pub trigger_name: String,
}

impl Expr {
    pub fn ty(&self, env: &Env) -> Ty {
        match self {
            Expr::Num(_) | Expr::Neg(_) | Expr::Bin(..) => Ty::Number,
            Expr::Text(_) | Expr::Message | Expr::NameOf(_) => Ty::Text,
            Expr::Var(_) => Ty::Object,
            Expr::Player => Ty::Player,
            Expr::Arg(i) => env.arg_types.get(i.wrapping_sub(1)).copied().unwrap_or(Ty::Object),
        }
    }
}

pub fn parse_expr(toks: &[Tok], env: &Env) -> Result<Expr, String> {
    if toks.is_empty() {
        return Err("missing expression".into());
    }
    let mut p = P { toks, pos: 0, env };
    let e = p.sum()?;
    if p.pos != toks.len() {
        return Err(format!("unexpected {}", describe(&toks[p.pos])));
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
}

impl P<'_> {
    fn peek(&self) -> Option<&Tok> {
        self.toks.get(self.pos)
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
                    "player" => Ok(Expr::Player),
                    "message" => {
                        if self.env.has_message {
                            Ok(Expr::Message)
                        } else {
                            Err(format!("'message' only exists in 'on chat', not in {}", self.env.trigger_name))
                        }
                    }
                    "arg" | "argument" => self.arg(1),
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

    fn arg(&self, i: usize) -> Result<Expr, String> {
        let n = self.env.arg_types.len();
        if n == 0 {
            return Err(format!("'arg' only exists in commands with arguments, not in {}", self.env.trigger_name));
        }
        if i == 0 || i > n {
            return Err(format!("arg-{i} does not exist, this command has {n} argument(s)"));
        }
        Ok(Expr::Arg(i))
    }
}

/// Parse a `"hello %player%!"` template body. `%%` is a literal percent sign.
pub fn parse_template(s: &str, env: &Env) -> Result<Vec<Part>, String> {
    let mut parts = Vec::new();
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
        let inner: String = chars[i + 1..j].iter().collect();
        let e = parse_expr_str(&inner, env).map_err(|e| format!("in %{inner}%: {e}"))?;
        if !lit.is_empty() {
            parts.push(Part::Lit(std::mem::take(&mut lit)));
        }
        parts.push(Part::Expr(e));
        i = j + 1;
    }
    if !lit.is_empty() {
        parts.push(Part::Lit(lit));
    }
    Ok(parts)
}

pub fn parse_varref(raw: &str, env: &Env) -> Result<VarRef, String> {
    let raw = raw.trim();
    let name = parse_template(raw, env)?;
    let local = raw.starts_with('_');
    Ok(VarRef { name, local })
}

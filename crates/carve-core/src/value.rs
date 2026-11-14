use std::fmt;

/// A handle to a player. Resolved through the host on every use, so a stored
/// handle for a player who went offline resolves to `<none>`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PlayerRef {
    pub uuid: String,
    pub name: String,
}

/// Where a plain `send` goes when no `to` is given.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Recipient {
    Player(PlayerRef),
    Console,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Value {
    None,
    Num(f64),
    Text(String),
    Bool(bool),
    Player(PlayerRef),
}

impl Value {
    pub fn type_name(&self) -> &'static str {
        match self {
            Value::None => "<none>",
            Value::Num(_) => "number",
            Value::Text(_) => "text",
            Value::Bool(_) => "boolean",
            Value::Player(_) => "player",
        }
    }

    pub fn as_num(&self) -> Option<f64> {
        match self {
            Value::Num(n) => Some(*n),
            _ => None,
        }
    }

    /// Skript-like equality: numbers by value, text ignoring case,
    /// players by UUID, a player equals text matching its name.
    pub fn loose_eq(&self, other: &Value) -> bool {
        match (self, other) {
            (Value::None, Value::None) => true,
            (Value::Num(a), Value::Num(b)) => (a - b).abs() < 1e-9,
            (Value::Text(a), Value::Text(b)) => a.eq_ignore_ascii_case(b),
            (Value::Bool(a), Value::Bool(b)) => a == b,
            (Value::Player(a), Value::Player(b)) => a.uuid == b.uuid,
            (Value::Player(p), Value::Text(t)) | (Value::Text(t), Value::Player(p)) => {
                p.name.eq_ignore_ascii_case(t)
            }
            (Value::Num(n), Value::Text(t)) | (Value::Text(t), Value::Num(n)) => t
                .trim()
                .parse::<f64>()
                .map(|v| (v - n).abs() < 1e-9)
                .unwrap_or(false),
            _ => false,
        }
    }
}

pub fn format_num(n: f64) -> String {
    let mut s = String::new();
    push_num(&mut s, n);
    s
}

/// Append a number the way scripts print it: whole numbers without a
/// fraction, others with at most 4 decimals.
pub fn push_num(out: &mut String, n: f64) {
    use std::fmt::Write;
    if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e15 {
        let _ = write!(out, "{}", n as i64);
    } else {
        let start = out.len();
        let _ = write!(out, "{n:.4}");
        let keep = out
            .get(start..)
            .map_or(0, |s| s.trim_end_matches('0').trim_end_matches('.').len());
        out.truncate(start + keep);
    }
}

impl Value {
    /// Append the printed form of this value to `out`.
    pub fn push_to(&self, out: &mut String) {
        match self {
            Value::None => out.push_str("<none>"),
            Value::Num(n) => push_num(out, *n),
            Value::Text(t) => out.push_str(t),
            Value::Bool(b) => out.push_str(if *b { "true" } else { "false" }),
            Value::Player(p) => out.push_str(&p.name),
        }
    }
}

impl fmt::Display for Value {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Value::None => write!(f, "<none>"),
            Value::Num(n) => write!(f, "{}", format_num(*n)),
            Value::Text(t) => write!(f, "{t}"),
            Value::Bool(b) => write!(f, "{b}"),
            Value::Player(p) => write!(f, "{}", p.name),
        }
    }
}

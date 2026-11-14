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
            (Value::Num(n), Value::Text(t)) | (Value::Text(t), Value::Num(n)) => {
                t.trim().parse::<f64>().map(|v| (v - n).abs() < 1e-9).unwrap_or(false)
            }
            _ => false,
        }
    }
}

pub fn format_num(n: f64) -> String {
    if n.is_finite() && n.fract() == 0.0 && n.abs() < 1e15 {
        format!("{}", n as i64)
    } else {
        let s = format!("{n:.4}");
        let s = s.trim_end_matches('0').trim_end_matches('.');
        s.to_string()
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

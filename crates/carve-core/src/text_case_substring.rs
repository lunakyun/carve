//! Text functions: case changes, substrings, character access, index search,
//! trimming and reversing. Pure helpers; parsing lives in `expr.rs`.

use crate::value::Value;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextFn {
    Upper,
    Lower,
    /// First letter of every word upper case, the rest lower case.
    Proper,
    /// First character upper case, the rest unchanged.
    Capitalized,
    Trim,
    Reverse,
    /// `substring of x from a to b`; args: text, from, to (to optional).
    Sub,
    /// `first n characters of x`; args: n, text.
    First,
    /// `last n characters of x`; args: n, text.
    Last,
    /// `character at index n of x`; args: n, text.
    CharAt,
    /// `[first] index of a in b`; args: needle, haystack.
    IndexFirst,
    IndexLast,
}

impl TextFn {
    pub fn returns_number(self) -> bool {
        matches!(self, TextFn::IndexFirst | TextFn::IndexLast)
    }
}

fn proper(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut start = true;
    for c in s.chars() {
        if c.is_whitespace() {
            start = true;
            out.push(c);
        } else if start {
            start = false;
            out.extend(c.to_uppercase());
        } else {
            out.extend(c.to_lowercase());
        }
    }
    out
}

fn capitalized(s: &str) -> String {
    let mut it = s.chars();
    match it.next() {
        Some(c) => c.to_uppercase().chain(it).collect(),
        None => String::new(),
    }
}

/// Substring by 1-based inclusive char positions, clamped to the text.
fn sub(s: &str, from: f64, to: f64) -> String {
    let len = s.chars().count() as f64;
    let a = from.max(1.0);
    let b = to.min(len);
    if !a.is_finite() || !b.is_finite() || a > b {
        return String::new();
    }
    s.chars()
        .skip(a as usize - 1)
        .take((b - a) as usize + 1)
        .collect()
}

fn index_of(needle: &str, hay: &str, last: bool) -> f64 {
    if needle.is_empty() {
        return 0.0;
    }
    let pos = if last { hay.rfind(needle) } else { hay.find(needle) };
    match pos {
        Some(b) => (hay[..b].chars().count() + 1) as f64,
        None => 0.0,
    }
}

/// Apply `f`. A `<none>` input gives `<none>`; a bad number gives `<none>`.
pub fn apply(f: TextFn, args: &[Value]) -> Value {
    if args.iter().any(|a| matches!(a, Value::None)) {
        return Value::None;
    }
    let num = |i: usize| args.get(i).and_then(Value::as_num);
    let txt = |i: usize| args.get(i).map(|v| v.to_string()).unwrap_or_default();
    match f {
        TextFn::Upper => Value::Text(txt(0).to_uppercase()),
        TextFn::Lower => Value::Text(txt(0).to_lowercase()),
        TextFn::Proper => Value::Text(proper(&txt(0))),
        TextFn::Capitalized => Value::Text(capitalized(&txt(0))),
        TextFn::Trim => Value::Text(txt(0).trim().to_string()),
        TextFn::Reverse => Value::Text(txt(0).chars().rev().collect()),
        TextFn::Sub => match (num(1), args.get(2).map_or(Some(f64::INFINITY), Value::as_num)) {
            (Some(a), Some(b)) => Value::Text(sub(&txt(0), a.trunc(), b.trunc())),
            _ => Value::None,
        },
        TextFn::First | TextFn::Last => match num(0) {
            Some(n) if n >= 0.0 => {
                let s = txt(1);
                let len = s.chars().count();
                let n = (n.trunc() as usize).min(len);
                if f == TextFn::First {
                    Value::Text(s.chars().take(n).collect())
                } else {
                    Value::Text(s.chars().skip(len - n).collect())
                }
            }
            _ => Value::None,
        },
        TextFn::CharAt => match num(0) {
            Some(n) if n >= 1.0 => txt(1)
                .chars()
                .nth(n.trunc() as usize - 1)
                .map_or(Value::None, |c| Value::Text(c.to_string())),
            _ => Value::None,
        },
        TextFn::IndexFirst => Value::Num(index_of(&txt(0), &txt(1), false)),
        TextFn::IndexLast => Value::Num(index_of(&txt(0), &txt(1), true)),
    }
}

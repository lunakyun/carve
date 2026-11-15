//! Variadic `min`/`max`, `round` with places, `entity(uuid)`, `axisAngle` and alias patterns.
//!
//! `min(a, b, c, ...)`, `round(n, places)`, `entity(uuid)`, `axisAngle(...)`,
//! the `for each` loop wording and the plural forms and `[optional]` /
//! `(a|b)` patterns of the `aliases:` section.

use crate::blocks::Line;
use crate::expr::{Expr, MathFn};
use crate::host::Host;
use crate::skbee_utilities::UtilFn;
use crate::value::{PlayerRef, Value};

/// `min`/`max` with any number of arguments (folded into pairs) and
/// `round(n, places)`.
pub fn math_call(f: MathFn, mut args: Vec<Expr>) -> Expr {
    if f == MathFn::Round && args.len() == 2 {
        return Expr::Util(UtilFn::Round, args);
    }
    if matches!(f, MathFn::Min | MathFn::Max) && args.len() > 2 {
        let mut acc = args.pop();
        while let Some(prev) = args.pop() {
            acc = acc.map(|a| Expr::Math(f, vec![prev, a]));
        }
        if let Some(e) = acc {
            return e;
        }
        return Expr::Num(0.0);
    }
    Expr::Math(f, args)
}

/// `entity from uuid "..."`: an online player first, then any live entity.
pub fn entity_from_uuid(arg: &Value, host: &mut dyn Host) -> Value {
    let u = arg.to_string();
    let u = u.trim();
    if let Some(name) = host.resolve_player(u) {
        return Value::Player(PlayerRef {
            uuid: u.to_string(),
            name,
        });
    }
    match host.entity_by_uuid(u) {
        Some(e) => Value::Entity(e),
        None => Value::None,
    }
}

/// `axisAngle(angle, x, y, z)`: a rotation of `angle` degrees around the axis
/// (the axis is normalised, as in Skript).
pub fn axis_angle(angle_deg: f64, x: f64, y: f64, z: f64) -> Result<Value, String> {
    let len = (x * x + y * y + z * z).sqrt();
    if len == 0.0 || !len.is_finite() {
        return Err("axisAngle needs an axis that is not 0, 0, 0".into());
    }
    let half = angle_deg.to_radians() / 2.0;
    let s = half.sin() / len;
    Ok(Value::Quaternion {
        w: half.cos(),
        x: x * s,
        y: y * s,
        z: z * s,
    })
}

/// `for each {_x} in <source>:` and `for each <type> in <source>:` as a
/// `loop <source>:` whose first line sets `{_x}` to `loop-value`.
pub fn for_each_line(line: &Line) -> Option<Line> {
    let head = line.head();
    let lower = head.to_ascii_lowercase();
    let rest = lower.strip_prefix("for each ")?;
    let off = head.len() - rest.len();
    let body = head.get(off..)?;
    let at = find_in(body)?;
    let (who, src) = (body.get(..at)?.trim(), body.get(at + 4..)?.trim());
    if who.is_empty() || src.is_empty() {
        return None;
    }
    let mut children = Vec::with_capacity(line.children.len() + 1);
    if who.starts_with('{') && who.ends_with('}') {
        children.push(Line {
            num: line.num,
            text: format!("set {who} to loop-value"),
            children: Vec::new(),
        });
    }
    children.extend(line.children.iter().cloned());
    Some(Line {
        num: line.num,
        text: format!("loop {src}:"),
        children,
    })
}

/// Byte offset of the first ` in ` outside quotes and braces.
fn find_in(s: &str) -> Option<usize> {
    let (mut q, mut depth) = (false, 0usize);
    for (i, c) in s.char_indices() {
        match c {
            '"' => q = !q,
            '{' if !q => depth += 1,
            '}' if !q => depth = depth.saturating_sub(1),
            ' ' if !q && depth == 0 && s.get(i..i + 4).is_some_and(|w| w.eq_ignore_ascii_case(" in ")) => {
                return Some(i);
            }
            _ => {}
        }
    }
    None
}

/// All spellings of an alias name: `[a|b]` is optional, `(a|b)` is a choice.
/// Capped so a bad pattern cannot blow up.
pub fn alias_names(name: &str) -> Vec<String> {
    let mut out = vec![String::new()];
    let chars: Vec<char> = name.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        let close = match c {
            '[' => ']',
            '(' => ')',
            _ => {
                for o in &mut out {
                    o.push(c);
                }
                i += 1;
                continue;
            }
        };
        let mut depth = 0;
        let mut j = i;
        while j < chars.len() {
            if chars[j] == c {
                depth += 1;
            } else if chars[j] == close {
                depth -= 1;
                if depth == 0 {
                    break;
                }
            }
            j += 1;
        }
        if j >= chars.len() {
            for o in &mut out {
                o.push(c);
            }
            i += 1;
            continue;
        }
        let inner: String = chars[i + 1..j].iter().collect();
        let mut opts: Vec<String> = Vec::new();
        for part in split_top(&inner) {
            opts.extend(alias_names(&part));
        }
        if c == '[' {
            opts.push(String::new());
        }
        let mut next = Vec::new();
        for o in &out {
            for p in &opts {
                if next.len() < 256 {
                    next.push(format!("{o}{p}"));
                }
            }
        }
        out = next;
        i = j + 1;
    }
    let mut seen: Vec<String> = Vec::new();
    for o in out {
        let t = o.split_whitespace().collect::<Vec<_>>().join(" ");
        if !t.is_empty() && !seen.contains(&t) {
            seen.push(t);
        }
    }
    seen
}

fn split_top(s: &str) -> Vec<String> {
    let mut parts = vec![String::new()];
    let mut depth = 0i32;
    for c in s.chars() {
        match c {
            '[' | '(' => depth += 1,
            ']' | ')' => depth -= 1,
            '|' if depth == 0 => {
                parts.push(String::new());
                continue;
            }
            _ => {}
        }
        if let Some(l) = parts.last_mut() {
            l.push(c);
        }
    }
    parts
}

/// Give every alias its plural (`my sword` also matches `my swords`) unless
/// the script defined that name itself.
pub fn add_plurals(aliases: &mut Vec<(String, String)>) {
    let extra: Vec<(String, String)> = aliases
        .iter()
        .filter(|(k, _)| !k.ends_with('s'))
        .map(|(k, v)| (format!("{k}s"), v.clone()))
        .filter(|(p, _)| !aliases.iter().any(|(k, _)| k == p))
        .collect();
    aliases.extend(extra);
}

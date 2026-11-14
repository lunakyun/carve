//! Wave 4: the built-in functions of Skript that are not plain math:
//! trigonometry (in degrees), `exp`, `ln`, `log`, `root`, `factorial`,
//! `combinations`, `permutations`, `clamp`, `median`, `product`, `concat`,
//! `formatNumber`, `fromBase`, `toBase`, `isNaN`, `calcExperience`, `rgb`,
//! `uuid`, `player` and `offlineplayer`.
//!
//! Every call is one boxed [`Node`] (`Expr::W4F`), so the hot expression
//! enum stays small. The engine evaluates the arguments and calls [`apply`].

use crate::expr::{Expr, Ty};
use crate::host::Host;
use crate::value::{PlayerRef, Value};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum F {
    Sin,
    Cos,
    Tan,
    Asin,
    Acos,
    Atan,
    Atan2,
    Exp,
    Ln,
    Log,
    Root,
    Factorial,
    Combinations,
    Permutations,
    Clamp,
    Median,
    Product,
    Concat,
    FormatNumber,
    FromBase,
    ToBase,
    IsNan,
    CalcExperience,
    Rgb,
    Uuid,
    Player,
    OfflinePlayer,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub f: F,
    pub args: Vec<Expr>,
}

/// Function name (lowercase), what it is, fewest and most arguments.
const TABLE: &[(&str, F, usize, usize)] = &[
    ("sin", F::Sin, 1, 1),
    ("cos", F::Cos, 1, 1),
    ("tan", F::Tan, 1, 1),
    ("asin", F::Asin, 1, 1),
    ("acos", F::Acos, 1, 1),
    ("atan", F::Atan, 1, 1),
    ("atan2", F::Atan2, 2, 2),
    ("exp", F::Exp, 1, 1),
    ("ln", F::Ln, 1, 1),
    ("log", F::Log, 1, 2),
    ("root", F::Root, 1, 2),
    ("factorial", F::Factorial, 1, 1),
    ("combinations", F::Combinations, 2, 2),
    ("permutations", F::Permutations, 2, 2),
    ("clamp", F::Clamp, 3, 3),
    ("median", F::Median, 1, usize::MAX),
    ("product", F::Product, 1, usize::MAX),
    ("concat", F::Concat, 1, usize::MAX),
    ("formatnumber", F::FormatNumber, 2, 2),
    ("frombase", F::FromBase, 2, 2),
    ("tobase", F::ToBase, 2, 2),
    ("isnan", F::IsNan, 1, 1),
    ("calcexperience", F::CalcExperience, 1, 1),
    ("rgb", F::Rgb, 3, 4),
    ("uuid", F::Uuid, 1, 1),
    ("player", F::Player, 1, 2),
    ("offlineplayer", F::OfflinePlayer, 1, 2),
];

pub fn lookup(lower: &str) -> Option<(F, usize, usize)> {
    TABLE.iter().find(|r| r.0 == lower).map(|r| (r.1, r.2, r.3))
}

impl F {
    pub fn ty(self) -> Ty {
        match self {
            F::IsNan => Ty::Boolean,
            F::Concat | F::FormatNumber | F::ToBase | F::Rgb | F::Uuid => Ty::Text,
            F::Player | F::OfflinePlayer => Ty::Player,
            _ => Ty::Number,
        }
    }

    /// Functions that read their arguments as lists (`{x::*}` allowed).
    pub fn takes_lists(self) -> bool {
        matches!(self, F::Median | F::Product | F::Concat)
    }

    /// Arguments that must be numbers, by position.
    pub fn number_arg(self, i: usize) -> bool {
        match self {
            F::Sin
            | F::Cos
            | F::Tan
            | F::Asin
            | F::Acos
            | F::Atan
            | F::Atan2
            | F::Exp
            | F::Ln
            | F::Log
            | F::Root
            | F::Factorial
            | F::Combinations
            | F::Permutations
            | F::Clamp
            | F::IsNan
            | F::CalcExperience
            | F::Rgb => true,
            F::FormatNumber | F::ToBase => i == 0,
            F::FromBase => i == 1,
            F::Median | F::Product | F::Concat | F::Uuid | F::Player | F::OfflinePlayer => false,
        }
    }
}

fn num(v: &Value) -> Option<f64> {
    match v {
        Value::Num(n) => Some(*n),
        _ => None,
    }
}

fn flatten(vals: &[Value]) -> Vec<&Value> {
    let mut out = Vec::new();
    for v in vals {
        match v {
            Value::List(items) => out.extend(items.iter()),
            Value::None => {}
            other => out.push(other),
        }
    }
    out
}

fn factorial(n: f64) -> f64 {
    if n < 0.0 || n.fract() != 0.0 {
        return f64::NAN;
    }
    let mut r = 1.0f64;
    let mut i = 2.0;
    while i <= n {
        r *= i;
        if r.is_infinite() {
            break;
        }
        i += 1.0;
    }
    r
}

/// Total experience points needed to reach a level from zero.
fn total_xp(l: f64) -> f64 {
    if l <= 16.0 {
        l * l + 6.0 * l
    } else if l <= 31.0 {
        2.5 * l * l - 40.5 * l + 360.0
    } else {
        4.5 * l * l - 162.5 * l + 2220.0
    }
}

fn to_base(mut n: i64, base: u32) -> String {
    if n == 0 {
        return "0".into();
    }
    let neg = n < 0;
    let mut digits = Vec::new();
    while n != 0 {
        let d = (n % i64::from(base)).unsigned_abs() as u32;
        digits.push(char::from_digit(d, base).unwrap_or('0'));
        n /= i64::from(base);
    }
    if neg {
        digits.push('-');
    }
    digits.iter().rev().collect()
}

/// Java `DecimalFormat`-style patterns: `#,##0.00`, `0.#`, `#%`, `$0.00`.
pub fn format_number(n: f64, pattern: &str) -> Option<String> {
    if !n.is_finite() {
        return Some(if n.is_nan() {
            "NaN".into()
        } else if n > 0.0 {
            "∞".into()
        } else {
            "-∞".into()
        });
    }
    let pattern = pattern.split(';').next().unwrap_or(pattern);
    let is_core = |c: char| matches!(c, '#' | '0' | ',' | '.');
    let start = pattern.find(is_core)?;
    let end = pattern.rfind(is_core)? + 1;
    let prefix = pattern.get(..start)?.replace('\'', "");
    let core = pattern.get(start..end)?;
    let suffix = pattern.get(end..)?.replace('\'', "");
    let percent = prefix.contains('%') || suffix.contains('%');
    let value = if percent { n * 100.0 } else { n };
    let (int_pat, frac_pat) = core.split_once('.').unwrap_or((core, ""));
    if frac_pat.contains(',') || frac_pat.contains('.') {
        return None;
    }
    let min_int = int_pat.chars().filter(|c| *c == '0').count();
    let group = int_pat
        .rfind(',')
        .map(|i| int_pat.get(i + 1..).map_or(0, str::len));
    let min_frac = frac_pat.chars().filter(|c| *c == '0').count();
    let max_frac = frac_pat.chars().filter(|c| matches!(c, '0' | '#')).count();
    let mut s = format!("{:.*}", max_frac, value.abs());
    let mut frac = String::new();
    if let Some((i, f)) = s.split_once('.') {
        frac = f.to_string();
        s = i.to_string();
    }
    while frac.len() > min_frac && frac.ends_with('0') {
        frac.pop();
    }
    let mut int = s;
    if int == "0" && min_int == 0 && !frac.is_empty() {
        int.clear();
    }
    while int.len() < min_int {
        int.insert(0, '0');
    }
    if let Some(g) = group.filter(|g| *g > 0) {
        let chars: Vec<char> = int.chars().collect();
        let mut out = String::new();
        for (i, c) in chars.iter().enumerate() {
            if i > 0 && (chars.len() - i) % g == 0 {
                out.push(',');
            }
            out.push(*c);
        }
        int = out;
    }
    let mut res = String::new();
    let is_zero = int.chars().all(|c| c == '0' || c == ',') && frac.chars().all(|c| c == '0');
    if value < 0.0 && !is_zero {
        res.push('-');
    }
    res.push_str(&prefix);
    res.push_str(&int);
    if !frac.is_empty() {
        res.push('.');
        res.push_str(&frac);
    }
    res.push_str(&suffix);
    Some(res)
}

fn parse_uuid(s: &str) -> Option<String> {
    let t = s.trim().to_ascii_lowercase();
    let hex: String = t.chars().filter(|c| *c != '-').collect();
    if hex.len() != 32 || !hex.chars().all(|c| c.is_ascii_hexdigit()) {
        return None;
    }
    let dashed_ok = t.len() == 32
        || (t.len() == 36
            && [8, 13, 18, 23]
                .iter()
                .all(|&i| t.as_bytes().get(i) == Some(&b'-')));
    dashed_ok.then(|| {
        format!(
            "{}-{}-{}-{}-{}",
            &hex[..8],
            &hex[8..12],
            &hex[12..16],
            &hex[16..20],
            &hex[20..]
        )
    })
}

fn find(text: &str, host: &mut dyn Host) -> Option<PlayerRef> {
    if let Some(p) = host.find_player(text) {
        return Some(p);
    }
    let id = parse_uuid(text)?;
    host.resolve_player(&id).map(|name| PlayerRef { uuid: id, name })
}

fn base_arg(b: f64) -> Result<u32, String> {
    if (2.0..=36.0).contains(&b) && b.fract() == 0.0 {
        Ok(b as u32)
    } else {
        Err("the base must be a whole number from 2 to 36".into())
    }
}

pub fn apply(f: F, a: &[Value], host: &mut dyn Host) -> Result<Value, String> {
    let n0 = a.first().and_then(num);
    let n1 = a.get(1).and_then(num);
    let n2 = a.get(2).and_then(num);
    let one = |g: fn(f64) -> f64| n0.map_or(Value::None, |x| Value::Num(g(x)));
    Ok(match f {
        F::Sin => one(|x| x.to_radians().sin()),
        F::Cos => one(|x| x.to_radians().cos()),
        F::Tan => one(|x| x.to_radians().tan()),
        F::Asin => one(|x| x.asin().to_degrees()),
        F::Acos => one(|x| x.acos().to_degrees()),
        F::Atan => one(|x| x.atan().to_degrees()),
        F::Atan2 => match (n0, n1) {
            (Some(y), Some(x)) => Value::Num(y.atan2(x).to_degrees()),
            _ => Value::None,
        },
        F::Exp => one(f64::exp),
        F::Ln => one(f64::ln),
        F::Log => match n0 {
            Some(x) => {
                let base = if a.len() > 1 { n1 } else { Some(10.0) };
                base.map_or(Value::None, |b| Value::Num(x.ln() / b.ln()))
            }
            None => Value::None,
        },
        F::Root => match n0 {
            Some(x) => {
                let d = if a.len() > 1 { n1 } else { Some(2.0) };
                d.map_or(Value::None, |d| {
                    Value::Num(if x < 0.0 && d.fract() == 0.0 && d % 2.0 != 0.0 {
                        -(-x).powf(1.0 / d)
                    } else {
                        x.powf(1.0 / d)
                    })
                })
            }
            None => Value::None,
        },
        F::Factorial => one(factorial),
        F::Combinations | F::Permutations => match (n0, n1) {
            (Some(n), Some(k)) if n >= 0.0 && k >= 0.0 && k <= n && n.fract() == 0.0 && k.fract() == 0.0 => {
                let perm = f == F::Permutations;
                let k = if perm { k } else { k.min(n - k) };
                let mut r = 1.0f64;
                let mut i = 0.0;
                while i < k {
                    r *= n - i;
                    if !perm {
                        r /= i + 1.0;
                    }
                    i += 1.0;
                }
                Value::Num(if perm { r } else { r.round() })
            }
            (Some(_), Some(_)) => Value::Num(f64::NAN),
            _ => Value::None,
        },
        F::Clamp => match (n0, n1, n2) {
            (Some(v), Some(lo), Some(hi)) => {
                if lo > hi {
                    return Err(format!(
                        "clamp() needs a minimum that is not above the maximum ({} > {})",
                        crate::value::format_num(lo),
                        crate::value::format_num(hi)
                    ));
                }
                Value::Num(v.clamp(lo, hi))
            }
            _ => Value::None,
        },
        F::Median => {
            let mut xs: Vec<f64> = flatten(a).into_iter().filter_map(num).collect();
            if xs.is_empty() {
                Value::None
            } else {
                xs.sort_by(f64::total_cmp);
                let m = xs.len() / 2;
                Value::Num(if xs.len() % 2 == 1 {
                    xs[m]
                } else {
                    (xs[m - 1] + xs[m]) / 2.0
                })
            }
        }
        F::Product => {
            let xs: Vec<f64> = flatten(a).into_iter().filter_map(num).collect();
            if xs.is_empty() {
                Value::None
            } else {
                Value::Num(xs.iter().product())
            }
        }
        F::Concat => {
            let mut s = String::new();
            for v in flatten(a) {
                match v {
                    Value::Num(n) => crate::value::push_num(&mut s, *n),
                    Value::Text(t) => s.push_str(t),
                    other => s.push_str(&format!("{other}")),
                }
            }
            Value::Text(s)
        }
        F::FormatNumber => match (n0, a.get(1)) {
            (Some(n), Some(Value::Text(p))) => match format_number(n, p) {
                Some(s) => Value::Text(s),
                None => return Err(format!("'{p}' is not a number format like \"#,##0.00\"")),
            },
            _ => Value::None,
        },
        F::FromBase => match (a.first(), n1) {
            (Some(Value::Text(t)), Some(b)) => {
                let b = base_arg(b)?;
                i64::from_str_radix(t.trim(), b).map_or(Value::None, |n| Value::Num(n as f64))
            }
            _ => Value::None,
        },
        F::ToBase => match (n0, n1) {
            (Some(n), Some(b)) => Value::Text(to_base(n as i64, base_arg(b)?)),
            _ => Value::None,
        },
        F::IsNan => Value::Bool(n0.is_none_or(f64::is_nan)),
        F::CalcExperience => one(|l| total_xp(l.max(0.0)).round()),
        F::Rgb => match (n0, n1, n2) {
            (Some(r), Some(g), Some(b)) => {
                let c = |x: f64| x.clamp(0.0, 255.0).round() as u8;
                Value::Text(format!("#{:02x}{:02x}{:02x}", c(r), c(g), c(b)))
            }
            _ => Value::None,
        },
        F::Uuid => match a.first() {
            Some(Value::Text(t)) => parse_uuid(t).map_or(Value::None, Value::Text),
            _ => Value::None,
        },
        F::Player => match a.first() {
            Some(Value::Text(t)) => find(t, host).map_or(Value::None, Value::Player),
            _ => Value::None,
        },
        F::OfflinePlayer => match a.first() {
            Some(Value::Text(t)) => Value::Player(find(t, host).unwrap_or_else(|| match parse_uuid(t) {
                Some(uuid) => PlayerRef {
                    uuid,
                    name: t.clone(),
                },
                None => PlayerRef {
                    uuid: String::new(),
                    name: t.clone(),
                },
            })),
            _ => Value::None,
        },
    })
}

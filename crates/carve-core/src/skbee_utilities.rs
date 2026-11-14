//! SkBee-style utilities: number formatting, roman numerals, hex, JSON text,
//! random UUIDs and extra client properties of a player. Pure helpers;
//! parsing lives in `expr.rs`, evaluation calls [`apply`].

use crate::host::Host;
use crate::value::{PlayerRef, Value};

/// A client property of an online player.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Info {
    Brand,
    Locale,
    ViewDistance,
    Pose,
    MainHand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UtilFn {
    /// `n with commas [and 2 decimal places]`; args: n, optional places.
    Commas,
    /// `n rounded to 2 decimal places`; args: n, places.
    Round,
    /// `abbreviated n`: `1.5K`, `2M`.
    Abbrev,
    /// `roman numeral of n`.
    Roman,
    /// `number from roman numeral "XIV"`.
    FromRoman,
    /// `hex of 255` is `ff`.
    Hex,
    /// `number from hex "ff"`.
    FromHex,
    /// `json of x`: a JSON string, number, `true`, `false` or `null`.
    Json,
    /// `random uuid`. The engine passes four random numbers in [0, 1).
    RandomUuid,
    /// A client property of the player in the first argument.
    Info(Info),
}

impl UtilFn {
    pub fn returns_number(self) -> bool {
        matches!(
            self,
            UtilFn::Round | UtilFn::FromRoman | UtilFn::FromHex | UtilFn::Info(Info::ViewDistance)
        )
    }

    pub fn needs_player(self) -> bool {
        matches!(self, UtilFn::Info(_))
    }
}

/// The player properties by their leading words, like `client brand`,
/// `view distance`, `main hand`. Returns the property and how many words it
/// uses, or `None`.
pub fn info_words(first: &str, second: Option<&str>, third: Option<&str>) -> Option<(Info, usize)> {
    Some(match (first, second, third) {
        ("client", Some("brand"), _) => (Info::Brand, 2),
        ("client", Some("locale" | "language"), _) => (Info::Locale, 2),
        ("client", Some("view"), Some("distance")) => (Info::ViewDistance, 3),
        ("view", Some("distance"), _) => (Info::ViewDistance, 2),
        ("main", Some("hand"), _) => (Info::MainHand, 2),
        ("brand", ..) => (Info::Brand, 1),
        ("locale" | "language", ..) => (Info::Locale, 1),
        ("pose", ..) => (Info::Pose, 1),
        _ => return None,
    })
}

/// Insert `,` between groups of three digits of the whole part.
pub fn group_commas(s: &str) -> String {
    let (sign, rest) = match s.strip_prefix('-') {
        Some(r) => ("-", r),
        None => ("", s),
    };
    let (int, frac) = match rest.find('.') {
        Some(i) => rest.split_at(i),
        None => (rest, ""),
    };
    if int.is_empty() || !int.bytes().all(|b| b.is_ascii_digit()) {
        return s.to_string();
    }
    let mut out = String::with_capacity(s.len() + int.len() / 3 + 1);
    out.push_str(sign);
    for (i, c) in int.chars().enumerate() {
        if i > 0 && (int.len() - i) % 3 == 0 {
            out.push(',');
        }
        out.push(c);
    }
    out.push_str(frac);
    out
}

fn places(v: Option<&Value>) -> Option<usize> {
    let n = v?.as_num()?;
    (n.is_finite() && n >= 0.0).then(|| (n.trunc() as usize).min(15))
}

fn round_to(n: f64, p: usize) -> f64 {
    let m = 10f64.powi(p as i32);
    let r = (n * m).round() / m;
    if r.is_finite() { r } else { n }
}

fn abbreviate(n: f64) -> String {
    const UNITS: [&str; 5] = ["", "K", "M", "B", "T"];
    let mut v = n;
    let mut i = 0;
    while v.abs() >= 1000.0 && i < UNITS.len() - 1 {
        v /= 1000.0;
        i += 1;
    }
    let mut r = round_to(v, 1);
    if r.abs() >= 1000.0 && i < UNITS.len() - 1 {
        r /= 1000.0;
        i += 1;
    }
    let mut s = format!("{r:.1}");
    if s.ends_with(".0") {
        s.truncate(s.len() - 2);
    }
    s.push_str(UNITS[i]);
    s
}

const ROMAN: [(u32, &str); 13] = [
    (1000, "M"),
    (900, "CM"),
    (500, "D"),
    (400, "CD"),
    (100, "C"),
    (90, "XC"),
    (50, "L"),
    (40, "XL"),
    (10, "X"),
    (9, "IX"),
    (5, "V"),
    (4, "IV"),
    (1, "I"),
];

pub fn to_roman(mut n: u32) -> Option<String> {
    if !(1..=3999).contains(&n) {
        return None;
    }
    let mut s = String::new();
    for (v, r) in ROMAN {
        while n >= v {
            s.push_str(r);
            n -= v;
        }
    }
    Some(s)
}

/// Strict: only the canonical spelling is accepted (`IIII` is not a number).
pub fn from_roman(s: &str) -> Option<u32> {
    let s = s.trim().to_ascii_uppercase();
    let val = |c: char| match c {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        'D' => Some(500),
        'M' => Some(1000),
        _ => None,
    };
    let digits: Vec<i64> = s.chars().map(val).collect::<Option<_>>()?;
    let mut total = 0;
    for (i, &d) in digits.iter().enumerate() {
        if digits.get(i + 1).is_some_and(|&next| next > d) {
            total -= d;
        } else {
            total += d;
        }
    }
    let total = u32::try_from(total).ok()?;
    (to_roman(total).as_deref() == Some(s.as_str())).then_some(total)
}

pub fn json_text(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => out.push_str(&format!("\\u{:04x}", c as u32)),
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

fn json_value(v: &Value) -> String {
    match v {
        Value::None => "null".into(),
        Value::Num(n) if n.is_finite() => v.to_string(),
        Value::Num(_) => "null".into(),
        Value::Bool(b) => b.to_string(),
        other => json_text(&other.to_string()),
    }
}

fn uuid_from(r: &[Value]) -> Value {
    let mut b = [0u8; 16];
    for (i, v) in r.iter().take(4).enumerate() {
        let x = (v.as_num().unwrap_or(0.0).clamp(0.0, 0.999_999_999) * 4_294_967_296.0) as u32;
        b[i * 4..i * 4 + 4].copy_from_slice(&x.to_be_bytes());
    }
    b[6] = (b[6] & 0x0f) | 0x40;
    b[8] = (b[8] & 0x3f) | 0x80;
    let h: String = b.iter().map(|x| format!("{x:02x}")).collect();
    Value::Text(format!("{}-{}-{}-{}-{}", &h[..8], &h[8..12], &h[12..16], &h[16..20], &h[20..]))
}

/// Run `f`. A `<none>` input, or one that makes no sense, gives `<none>`.
pub fn apply(f: UtilFn, args: &[Value], host: &mut dyn Host) -> Value {
    if f == UtilFn::RandomUuid {
        return uuid_from(args);
    }
    if f == UtilFn::Json {
        return Value::Text(args.first().map_or_else(|| "null".into(), json_value));
    }
    let Some(first) = args.first() else {
        return Value::None;
    };
    if matches!(first, Value::None) {
        return Value::None;
    }
    let num = first.as_num().filter(|n| n.is_finite());
    match f {
        UtilFn::Commas => {
            let Some(n) = num else { return Value::None };
            let s = match places(args.get(1)) {
                Some(p) => format!("{n:.p$}"),
                None => first.to_string(),
            };
            Value::Text(group_commas(&s))
        }
        UtilFn::Round => match (num, places(args.get(1))) {
            (Some(n), Some(p)) => Value::Num(round_to(n, p)),
            _ => Value::None,
        },
        UtilFn::Abbrev => num.map_or(Value::None, |n| Value::Text(abbreviate(n))),
        UtilFn::Roman => num
            .filter(|n| n.fract() == 0.0 && *n >= 1.0 && *n <= 3999.0)
            .and_then(|n| to_roman(n as u32))
            .map_or(Value::None, Value::Text),
        UtilFn::FromRoman => from_roman(&first.to_string()).map_or(Value::None, |n| Value::Num(f64::from(n))),
        UtilFn::Hex => match num {
            Some(n) if n.fract() == 0.0 && n.abs() < 9e15 => {
                let i = n as i64;
                Value::Text(if i < 0 { format!("-{:x}", -i) } else { format!("{i:x}") })
            }
            _ => Value::None,
        },
        UtilFn::FromHex => {
            let t = first.to_string();
            let t = t.trim().trim_start_matches('#');
            let t = t.strip_prefix("0x").or_else(|| t.strip_prefix("0X")).unwrap_or(t);
            i64::from_str_radix(t, 16).map_or(Value::None, |n| Value::Num(n as f64))
        }
        UtilFn::Info(i) => {
            let Value::Player(p) = first else {
                return Value::None;
            };
            player_info(host, p, i)
        }
        UtilFn::RandomUuid | UtilFn::Json => Value::None,
    }
}

fn player_info(host: &mut dyn Host, p: &PlayerRef, i: Info) -> Value {
    match host.player_info(p, i) {
        Some(s) if i == Info::ViewDistance => s.parse::<f64>().map_or(Value::None, Value::Num),
        Some(s) => Value::Text(s),
        None => Value::None,
    }
}

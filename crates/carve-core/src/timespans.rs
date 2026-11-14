//! Time spans and a minimal date: unit parsing, arithmetic and printing.
//! Spans and dates are whole milliseconds. One tick is 50 ms.

use crate::expr::{BinOp, Ty};
use crate::value::{Value, push_num};

pub const MS_TICK: i64 = 50;
pub const MS_SECOND: i64 = 1000;
pub const MS_MINUTE: i64 = 60_000;
pub const MS_HOUR: i64 = 3_600_000;
pub const MS_DAY: i64 = 86_400_000;
pub const MS_WEEK: i64 = 7 * MS_DAY;
pub const MS_MONTH: i64 = 30 * MS_DAY;
pub const MS_YEAR: i64 = 365 * MS_DAY;

/// `second`, `minutes`, `ticks`... -> milliseconds per unit.
pub fn unit_ms(word: &str) -> Option<i64> {
    let w = word.to_ascii_lowercase();
    let w = w.strip_suffix('s').unwrap_or(&w);
    Some(match w {
        "millisecond" => 1,
        "tick" => MS_TICK,
        "second" => MS_SECOND,
        "minute" => MS_MINUTE,
        "hour" => MS_HOUR,
        "day" => MS_DAY,
        "week" => MS_WEEK,
        "month" => MS_MONTH,
        "year" => MS_YEAR,
        _ => return None,
    })
}

/// Milliseconds of `n` units, rounded. `None` if it does not fit.
pub fn span_ms(n: f64, unit: i64) -> Option<i64> {
    let v = (n * unit as f64).round();
    (v.is_finite() && v.abs() < 9.0e15).then_some(v as i64)
}

/// Ticks for a wait: rounded to the nearest tick.
pub fn to_ticks(ms: i64) -> u64 {
    if ms <= 0 {
        0
    } else {
        ((ms + MS_TICK / 2) / MS_TICK) as u64
    }
}

/// `<unit> of <span>` (component) or `total <unit> of <span>`.
pub fn span_part(ms: i64, unit: i64, total: bool) -> f64 {
    if total {
        return ms as f64 / unit as f64;
    }
    let a = ms.abs();
    let v = match unit {
        1 => a % MS_SECOND,
        MS_TICK => (a % MS_SECOND) / MS_TICK,
        MS_SECOND => (a / MS_SECOND) % 60,
        MS_MINUTE => (a / MS_MINUTE) % 60,
        MS_HOUR => (a / MS_HOUR) % 24,
        MS_DAY => (a / MS_DAY) % 7,
        MS_WEEK => (a / MS_WEEK) % 4,
        _ => a / unit,
    };
    v as f64
}

fn push_unit(out: &mut String, n: f64, one: &str) {
    push_num(out, n);
    out.push(' ');
    out.push_str(one);
    if (n - 1.0).abs() > 1e-9 {
        out.push('s');
    }
}

/// `5 minutes`, `1 hour and 30 seconds`, `1 day, 2 hours and 3 minutes`.
pub fn push_span(out: &mut String, ms: i64) {
    let mut ms = ms;
    if ms < 0 {
        out.push('-');
        ms = -ms;
    }
    let mut parts: Vec<(f64, &str)> = Vec::new();
    for (unit, name) in [(MS_DAY, "day"), (MS_HOUR, "hour"), (MS_MINUTE, "minute")] {
        let n = ms / unit;
        ms %= unit;
        if n > 0 {
            parts.push((n as f64, name));
        }
    }
    if ms > 0 || parts.is_empty() {
        parts.push((ms as f64 / 1000.0, "second"));
    }
    let last = parts.len() - 1;
    for (i, (n, name)) in parts.iter().enumerate() {
        if i > 0 {
            out.push_str(if i == last { " and " } else { ", " });
        }
        push_unit(out, *n, name);
    }
}

/// `2026-11-14 12:30:05` (UTC).
pub fn push_date(out: &mut String, millis: i64) {
    use std::fmt::Write;
    let days = millis.div_euclid(MS_DAY);
    let rem = millis.rem_euclid(MS_DAY);
    let (y, m, d) = civil_from_days(days);
    let _ = write!(
        out,
        "{y:04}-{m:02}-{d:02} {:02}:{:02}:{:02}",
        rem / MS_HOUR,
        (rem / MS_MINUTE) % 60,
        (rem / MS_SECOND) % 60
    );
}

fn civil_from_days(z: i64) -> (i64, i64, i64) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = doy - (153 * mp + 2) / 5 + 1;
    let m = if mp < 10 { mp + 3 } else { mp - 9 };
    (yoe + era * 400 + i64::from(m <= 2), m, d)
}

/// Numeric view for `<`, `>`, `<=`, `>=`: numbers, text numbers, spans, dates.
pub fn cmp_num(v: &Value) -> Option<f64> {
    match v {
        Value::Num(n) => Some(*n),
        Value::Text(t) => t.trim().parse().ok(),
        Value::Timespan { millis } | Value::Date { millis } => Some(*millis as f64),
        _ => None,
    }
}

/// Arithmetic where at least one side is a span or a date.
pub fn arith(a: &Value, op: BinOp, b: &Value) -> Result<Value, String> {
    use Value::{Date, Num, Timespan};
    let over = || "time overflow".to_string();
    Ok(match (a, op, b) {
        (Timespan { millis: x }, BinOp::Add, Timespan { millis: y }) => Timespan {
            millis: x.checked_add(*y).ok_or_else(over)?,
        },
        (Timespan { millis: x }, BinOp::Sub, Timespan { millis: y }) => Timespan {
            millis: x.checked_sub(*y).ok_or_else(over)?,
        },
        (Date { millis: x }, BinOp::Add, Timespan { millis: y })
        | (Timespan { millis: y }, BinOp::Add, Date { millis: x }) => Date {
            millis: x.checked_add(*y).ok_or_else(over)?,
        },
        (Date { millis: x }, BinOp::Sub, Timespan { millis: y }) => Date {
            millis: x.checked_sub(*y).ok_or_else(over)?,
        },
        (Date { millis: x }, BinOp::Sub, Date { millis: y }) => Timespan {
            millis: x.checked_sub(*y).ok_or_else(over)?,
        },
        (Timespan { millis: x }, BinOp::Mul, Num(n)) | (Num(n), BinOp::Mul, Timespan { millis: x }) => {
            Timespan {
                millis: span_ms(*n, *x).ok_or_else(over)?,
            }
        }
        (Timespan { millis: x }, BinOp::Div, Num(n)) => {
            if *n == 0.0 {
                return Err("division by zero".into());
            }
            Timespan {
                millis: span_ms(1.0 / n, *x).ok_or_else(over)?,
            }
        }
        (Timespan { millis: x }, BinOp::Div, Timespan { millis: y }) => {
            if *y == 0 {
                return Err("division by zero".into());
            }
            Num(*x as f64 / *y as f64)
        }
        _ => {
            let sym = match op {
                BinOp::Add => '+',
                BinOp::Sub => '-',
                BinOp::Mul => '*',
                BinOp::Div => '/',
            };
            return Err(format!("can't do {} {sym} {}", a.type_name(), b.type_name()));
        }
    })
}

/// Can this operator work on these static types? `Object` could be anything.
pub fn ok_types(l: Ty, op: BinOp, r: Ty) -> bool {
    use Ty::{Date, Number, Object, Timespan};
    let timey = |t: Ty| matches!(t, Timespan | Date);
    match op {
        BinOp::Add => {
            matches!((l, r), (Timespan, Timespan) | (Date, Timespan) | (Timespan, Date))
                || (l == Object && timey(r))
                || (r == Object && timey(l))
        }
        BinOp::Sub => {
            matches!((l, r), (Timespan, Timespan) | (Date, Timespan) | (Date, Date))
                || (l == Object && timey(r))
                || (r == Object && timey(l))
        }
        BinOp::Mul => matches!((l, r), (Timespan, Number | Object) | (Number | Object, Timespan)),
        BinOp::Div => {
            matches!((l, r), (Timespan, Number | Object | Timespan)) || (l == Object && r == Timespan)
        }
    }
}

/// Static result type of a span/date operation.
pub fn result_ty(l: Ty, op: BinOp, r: Ty) -> Ty {
    use Ty::{Date, Number, Timespan};
    match (l, op, r) {
        (Timespan, BinOp::Add | BinOp::Sub, Timespan) => Timespan,
        (Date, BinOp::Add | BinOp::Sub, Timespan) | (Timespan, BinOp::Add, Date) => Date,
        (Date, BinOp::Sub, Date) => Timespan,
        (Timespan, BinOp::Mul | BinOp::Div, Number) | (Number, BinOp::Mul, Timespan) => Timespan,
        (Timespan, BinOp::Div, Timespan) => Number,
        _ => Ty::Object,
    }
}

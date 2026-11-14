//! Dates: UTC calendar math and Java-style date formatting. No dependencies.
//!
//! A date is a count of milliseconds since 1970-01-01T00:00:00Z.

/// The format used when a date is printed without `formatted as`.
pub const DEFAULT_FORMAT: &str = "yyyy-MM-dd HH:mm:ss";

/// Days since 1970-01-01 to (year, month 1-12, day 1-31).
pub fn civil_from_days(z: i64) -> (i64, u32, u32) {
    let z = z + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

const MONTHS: [&str; 12] = [
    "January",
    "February",
    "March",
    "April",
    "May",
    "June",
    "July",
    "August",
    "September",
    "October",
    "November",
    "December",
];
const DAYS: [&str; 7] = [
    "Monday",
    "Tuesday",
    "Wednesday",
    "Thursday",
    "Friday",
    "Saturday",
    "Sunday",
];

fn pad(out: &mut String, n: i64, width: usize) {
    use std::fmt::Write;
    let _ = write!(out, "{n:0width$}");
}

/// Format `millis` (UTC) with a Java-style pattern: `yyyy MM dd HH mm ss SSS`,
/// `yy M d H h m s`, `a` (AM/PM), `MMM`/`MMMM` (month name), `EEE`/`EEEE`
/// (weekday name) and `'quoted text'`.
pub fn format_date(millis: i64, pattern: &str) -> String {
    let days = millis.div_euclid(86_400_000);
    let ms_of_day = millis.rem_euclid(86_400_000);
    let (year, month, day) = civil_from_days(days);
    let hour = ms_of_day / 3_600_000;
    let minute = ms_of_day / 60_000 % 60;
    let second = ms_of_day / 1000 % 60;
    let milli = ms_of_day % 1000;
    // 1970-01-01 was a Thursday (index 3 with Monday = 0).
    let weekday = (days + 3).rem_euclid(7) as usize;

    let chars: Vec<char> = pattern.chars().collect();
    let mut out = String::new();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c == '\'' {
            i += 1;
            if chars.get(i) == Some(&'\'') {
                out.push('\'');
                i += 1;
                continue;
            }
            while i < chars.len() && chars[i] != '\'' {
                out.push(chars[i]);
                i += 1;
            }
            i += 1;
            continue;
        }
        if !c.is_ascii_alphabetic() {
            out.push(c);
            i += 1;
            continue;
        }
        let mut n = 1;
        while chars.get(i + n) == Some(&c) {
            n += 1;
        }
        i += n;
        match c {
            'y' | 'u' => {
                if n == 2 {
                    pad(&mut out, year.rem_euclid(100), 2);
                } else {
                    pad(&mut out, year, n);
                }
            }
            'M' | 'L' => match n {
                1 | 2 => pad(&mut out, i64::from(month), n),
                3 => out.push_str(&MONTHS[month as usize - 1][..3]),
                _ => out.push_str(MONTHS[month as usize - 1]),
            },
            'd' => pad(&mut out, i64::from(day), n),
            'H' => pad(&mut out, hour, n),
            'h' => pad(&mut out, if hour % 12 == 0 { 12 } else { hour % 12 }, n),
            'm' => pad(&mut out, minute, n),
            's' => pad(&mut out, second, n),
            'S' => {
                let s = format!("{milli:03}");
                out.push_str(&s[..n.min(3)]);
                for _ in 3..n {
                    out.push('0');
                }
            }
            'a' => out.push_str(if hour < 12 { "AM" } else { "PM" }),
            'E' => {
                if n >= 4 {
                    out.push_str(DAYS[weekday]);
                } else {
                    out.push_str(&DAYS[weekday][..3]);
                }
            }
            _ => out.extend(std::iter::repeat_n(c, n)),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epoch_and_known_dates() {
        assert_eq!(format_date(0, DEFAULT_FORMAT), "1970-01-01 00:00:00");
        // 2024-02-29 13:05:09.042 UTC
        let ms = 1_709_211_909_042;
        assert_eq!(
            format_date(ms, "yyyy-MM-dd HH:mm:ss.SSS"),
            "2024-02-29 13:05:09.042"
        );
        assert_eq!(
            format_date(ms, "EEEE, d MMMM yy h:mm a"),
            "Thursday, 29 February 24 1:05 PM"
        );
        assert_eq!(format_date(ms, "'on' EEE 'at' HH''mm"), "on Thu at 13'05");
        assert_eq!(format_date(-1, DEFAULT_FORMAT), "1969-12-31 23:59:59");
    }
}

//! Expressions, part 1 (A to C): `an eternity`, `5 degrees`, `altitude of`,
//! `red value of`, `amount of diamonds in player's inventory`, `biome at`,
//! `helmet of player`, `beacon tier of`, `brewing time of`, `burn time of`,
//! `all scripts`.
//!
//! Everything here reaches the game through three optional [`Host`] methods
//! (`biome_at`, `armor_item`, `block_entity_nbt`) and hangs off one `Expr`
//! variant, so the hot expression enum stays small.

use crate::expr::{Env, Expr, parse_expr, parse_sum_at};
use crate::lexer::Tok;

/// The "forever" time span, in milliseconds. 2^53 - 1 keeps it exact as a number.
pub const ETERNITY_MS: i64 = 9_007_199_254_740_991;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Channel {
    Alpha,
    Red,
    Green,
    Blue,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArmorSlot {
    Helmet,
    Chestplate,
    Leggings,
    Boots,
}

/// What to read from a block entity.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BeField {
    BeaconTier,
    BeaconRange,
    BrewTime,
    BrewFuel,
    BurnTime,
    CookTime,
    CookTimeTotal,
}

impl BeField {
    /// NBT keys that can hold the value, newest name first.
    fn keys(self) -> &'static [&'static str] {
        match self {
            BeField::BeaconTier | BeField::BeaconRange => &["Levels"],
            BeField::BrewTime => &["BrewTime"],
            BeField::BrewFuel => &["Fuel"],
            BeField::BurnTime => &["lit_time_remaining", "BurnTime"],
            BeField::CookTime => &["cooking_time_spent", "CookTime"],
            BeField::CookTimeTotal => &["cooking_total_time", "CookTimeTotal"],
        }
    }

    /// Is the result a time span (ticks in the block, milliseconds here)?
    pub fn is_span(self) -> bool {
        matches!(
            self,
            BeField::BrewTime | BeField::BurnTime | BeField::CookTime | BeField::CookTimeTotal
        )
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum W2 {
    Eternity,
    /// `x radians` (true) or `x degrees` (false): the angle in degrees.
    Angle(bool, Expr),
    Altitude(Expr),
    Channel(Channel, Expr),
    /// `amount of <item> in <player>'s inventory`
    ItemCount(Expr, Expr),
    Biome(Expr),
    Armor(ArmorSlot, Expr),
    BlockEntity(BeField, Expr),
    AllScripts,
}

impl W2 {
    pub fn ty(&self) -> crate::expr::Ty {
        use crate::expr::Ty;
        match self {
            W2::Eternity => Ty::Timespan,
            W2::BlockEntity(f, _) if f.is_span() => Ty::Timespan,
            W2::Angle(..) | W2::Altitude(_) | W2::Channel(..) | W2::ItemCount(..) | W2::BlockEntity(..) => Ty::Number,
            W2::Biome(_) => Ty::Text,
            W2::Armor(..) => Ty::Item,
            W2::AllScripts => Ty::List,
        }
    }
}

fn is(t: Option<&Tok>, w: &str) -> bool {
    t.is_some_and(|t| t.is_lit(w))
}

/// Words that can start one of the expressions. A cheap check so ordinary
/// words never reach the real parser.
pub(crate) fn starts(lw: &str) -> bool {
    matches!(
        lw,
        "eternity"
            | "an"
            | "altitude"
            | "red"
            | "green"
            | "blue"
            | "alpha"
            | "amount"
            | "number"
            | "biome"
            | "helmet"
            | "chestplate"
            | "leggings"
            | "boots"
            | "beacon"
            | "brewing"
            | "burn"
            | "cook"
            | "total"
            | "all"
    )
}

/// Try to parse one of the expressions. `pos` is the position after the first
/// word `lw` (already lowercase). Returns the expression and the end position,
/// or `None` when the words are something else.
pub(crate) fn parse(toks: &[Tok], pos: usize, lw: &str, env: &Env) -> Result<Option<(W2, usize)>, String> {
    if !starts(lw) {
        return Ok(None);
    }
    let at = |i: usize| toks.get(pos + i);
    // `of <expr>` after `n` words.
    let of = |n: usize| -> Result<Option<(Expr, usize)>, String> {
        if !is(toks.get(pos + n), "of") {
            return Ok(None);
        }
        parse_sum_at(toks, pos + n + 1, env).map(Some)
    };
    let hit = |r: Option<(Expr, usize)>, f: &dyn Fn(Expr) -> W2| r.map(|(e, p)| (f(e), p));
    Ok(match lw {
        "eternity" => Some((W2::Eternity, pos)),
        "an" if is(at(0), "eternity") => Some((W2::Eternity, pos + 1)),
        "altitude" => hit(of(0)?, &W2::Altitude),
        "red" | "green" | "blue" | "alpha" => {
            let ch = match lw {
                "red" => Channel::Red,
                "green" => Channel::Green,
                "blue" => Channel::Blue,
                _ => Channel::Alpha,
            };
            // `red value of x`, `red component of x`. A bare `red of` is too
            // likely to clash with other words, so `value` or `component` is required.
            if !(is(at(0), "value") || is(at(0), "component")) {
                return Ok(None);
            }
            hit(of(1)?, &|e| W2::Channel(ch, e))
        }
        "amount" | "number" if is(at(0), "of") => item_count(toks, pos + 1, env)?,
        "biome" if is(at(0), "at") || is(at(0), "of") => {
            parse_sum_at(toks, pos + 1, env).map(|(e, p)| Some((W2::Biome(e), p)))?
        }
        "helmet" | "chestplate" | "leggings" | "boots" => {
            let slot = match lw {
                "helmet" => ArmorSlot::Helmet,
                "chestplate" => ArmorSlot::Chestplate,
                "leggings" => ArmorSlot::Leggings,
                _ => ArmorSlot::Boots,
            };
            hit(of(0)?, &|e| W2::Armor(slot, e))
        }
        "beacon" if is(at(0), "tier") || is(at(0), "level") => hit(of(1)?, &|e| W2::BlockEntity(BeField::BeaconTier, e)),
        "beacon" if is(at(0), "range") => hit(of(1)?, &|e| W2::BlockEntity(BeField::BeaconRange, e)),
        "brewing" => {
            // `brewing [stand] time of`, `brewing [stand] fuel [level] of`
            let s = usize::from(is(at(0), "stand"));
            if is(at(s), "time") {
                hit(of(s + 1)?, &|e| W2::BlockEntity(BeField::BrewTime, e))
            } else if is(at(s), "fuel") {
                let l = usize::from(is(at(s + 1), "level"));
                hit(of(s + 1 + l)?, &|e| W2::BlockEntity(BeField::BrewFuel, e))
            } else {
                None
            }
        }
        "burn" if is(at(0), "time") => hit(of(1)?, &|e| W2::BlockEntity(BeField::BurnTime, e)),
        "cook" if is(at(0), "time") => hit(of(1)?, &|e| W2::BlockEntity(BeField::CookTime, e)),
        "total" if is(at(0), "cook") && is(at(1), "time") => {
            hit(of(2)?, &|e| W2::BlockEntity(BeField::CookTimeTotal, e))
        }
        "all" if is(at(0), "scripts") => Some((W2::AllScripts, pos + 1)),
        _ => None,
    })
}

/// `amount of <item> in <player>'s inventory` / `... in inventory of <player>`.
/// `pos` is just after `of`. Returns `None` for anything else, so
/// `amount of {list::*}` keeps working.
fn item_count(toks: &[Tok], pos: usize, env: &Env) -> Result<Option<(W2, usize)>, String> {
    let Some(in_at) = (pos..toks.len()).find(|&i| is(toks.get(i), "in")) else {
        return Ok(None);
    };
    if in_at == pos {
        return Ok(None);
    }
    let (player_toks_end, end) = if is(toks.get(in_at + 1), "inventory") && is(toks.get(in_at + 2), "of") {
        let (_, end) = parse_sum_at(toks, in_at + 3, env)?;
        (None, end)
    } else {
        let Some(j) = (in_at + 1..toks.len().saturating_sub(1))
            .find(|&j| matches!(toks.get(j), Some(Tok::Sym("'s"))) && is(toks.get(j + 1), "inventory"))
        else {
            return Ok(None);
        };
        (Some(j), j + 2)
    };
    let item = parse_expr(toks.get(pos..in_at).unwrap_or_default(), env)?;
    let player = match player_toks_end {
        Some(j) => parse_expr(toks.get(in_at + 1..j).unwrap_or_default(), env)?,
        None => parse_sum_at(toks, in_at + 3, env)?.0,
    };
    Ok(Some((W2::ItemCount(item, player), end)))
}

/// `5 degrees`, `1.5 radians` after a number. Returns the angle in degrees
/// and the position after the unit word.
pub(crate) fn angle_suffix(toks: &[Tok], pos: usize, n: f64) -> Option<(Expr, usize)> {
    let Some(Tok::Word(w)) = toks.get(pos) else {
        return None;
    };
    match w.to_ascii_lowercase().as_str() {
        "degree" | "degrees" | "deg" => Some((Expr::Num(n), pos + 1)),
        "radian" | "radians" | "rad" => Some((Expr::Num(n.to_degrees()), pos + 1)),
        _ => None,
    }
}

// ---------------------------------------------------------------- values

/// Red, green, blue or alpha of a color given as text: `#rrggbb`, `&#rrggbb`,
/// `rgb(1, 2, 3)` is not supported, but legacy names like `red` and `dark_blue` are.
pub fn channel_of(text: &str, ch: Channel) -> Option<f64> {
    let t = text.trim();
    let tag = {
        let bare = t.trim_start_matches('&').trim_start_matches('<').trim_end_matches('>');
        if bare.starts_with('#') {
            format!("<{bare}>x")
        } else {
            format!("<{}>x", bare.replace(' ', "_").to_ascii_lowercase())
        }
    };
    let spans = crate::colors::parse(&tag);
    let (r, g, b) = spans.first()?.style.color?;
    Some(match ch {
        Channel::Red => f64::from(r),
        Channel::Green => f64::from(g),
        Channel::Blue => f64::from(b),
        Channel::Alpha => 255.0,
    })
}

/// Turn a CamelCase debug name into a `snake_case` id (`BirchForest` to `birch_forest`).
pub fn snake(name: &str) -> String {
    let mut out = String::with_capacity(name.len() + 4);
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Beacon range in blocks for a pyramid tier.
pub fn beacon_range(tier: f64) -> f64 {
    if tier <= 0.0 { 0.0 } else { tier * 10.0 + 10.0 }
}

/// Read one number from a block entity. `tier * 10 + 10` for the range,
/// ticks to milliseconds for time fields.
pub fn block_entity_value(field: BeField, nbt: &[u8]) -> Option<f64> {
    let raw = nbt_number(nbt, field.keys())?;
    let v = raw as f64;
    Some(match field {
        BeField::BeaconRange => beacon_range(v),
        f if f.is_span() => v * 50.0,
        _ => v,
    })
}

// ------------------------------------------------------------ NBT reader

struct Rd<'a> {
    b: &'a [u8],
    p: usize,
}

impl Rd<'_> {
    fn take(&mut self, n: usize) -> Option<&[u8]> {
        let s = self.b.get(self.p..self.p.checked_add(n)?)?;
        self.p += n;
        Some(s)
    }
    fn u8(&mut self) -> Option<u8> {
        self.take(1).map(|s| s[0])
    }
    fn u16(&mut self) -> Option<usize> {
        self.take(2).map(|s| usize::from(u16::from_be_bytes([s[0], s[1]])))
    }
    fn i32(&mut self) -> Option<i32> {
        self.take(4).map(|s| i32::from_be_bytes([s[0], s[1], s[2], s[3]]))
    }
    fn skip_len(&mut self, n: i64, unit: usize) -> Option<()> {
        let n = usize::try_from(n).ok()?;
        self.take(n.checked_mul(unit)?).map(|_| ())
    }
    /// Skip the payload of a tag.
    fn skip(&mut self, tag: u8, depth: u32) -> Option<()> {
        if depth > 32 {
            return None;
        }
        match tag {
            1 => self.skip_len(1, 1),
            2 => self.skip_len(2, 1),
            3 | 5 => self.skip_len(4, 1),
            4 | 6 => self.skip_len(8, 1),
            7 => {
                let n = self.i32()?;
                self.skip_len(i64::from(n), 1)
            }
            8 => {
                let n = self.u16()?;
                self.take(n).map(|_| ())
            }
            9 => {
                let t = self.u8()?;
                let n = self.i32()?;
                for _ in 0..n.max(0) {
                    self.skip(t, depth + 1)?;
                }
                Some(())
            }
            10 => {
                self.entries(depth + 1, &mut |_, _, _| {})?;
                Some(())
            }
            11 => {
                let n = self.i32()?;
                self.skip_len(i64::from(n), 4)
            }
            12 => {
                let n = self.i32()?;
                self.skip_len(i64::from(n), 8)
            }
            _ => None,
        }
    }
    fn number(&mut self, tag: u8) -> Option<i64> {
        Some(match tag {
            1 => i64::from(self.u8()? as i8),
            2 => {
                let s = self.take(2)?;
                i64::from(i16::from_be_bytes([s[0], s[1]]))
            }
            3 => i64::from(self.i32()?),
            4 => {
                let s = self.take(8)?;
                i64::from_be_bytes(s.try_into().ok()?)
            }
            _ => return None,
        })
    }
    /// Read compound entries up to the end tag. Calls `f(name, tag, value)` for
    /// whole-number entries (value is `Some`), and skips everything else.
    fn entries(&mut self, depth: u32, f: &mut dyn FnMut(&str, u8, Option<i64>)) -> Option<()> {
        loop {
            let tag = self.u8()?;
            if tag == 0 {
                return Some(());
            }
            let n = self.u16()?;
            let name = String::from_utf8_lossy(self.take(n)?).into_owned();
            if (1..=4).contains(&tag) {
                let v = self.number(tag)?;
                f(&name, tag, Some(v));
            } else {
                self.skip(tag, depth)?;
                f(&name, tag, None);
            }
        }
    }
}

/// The first top-level whole number whose key is in `keys` (earlier keys win).
/// Accepts both a named root (files) and a nameless one (network format).
pub fn nbt_number(bytes: &[u8], keys: &[&str]) -> Option<i64> {
    for named in [false, true] {
        let mut r = Rd { b: bytes, p: 0 };
        if r.u8()? != 10 {
            return None;
        }
        if named {
            let n = r.u16()?;
            if r.take(n).is_none() {
                continue;
            }
        }
        let mut found: Vec<(String, i64)> = Vec::new();
        let ok = r
            .entries(0, &mut |name, _, v| {
                if let (Some(v), true) = (v, keys.contains(&name)) {
                    found.push((name.to_string(), v));
                }
            })
            .is_some()
            && r.p == bytes.len();
        if ok {
            return keys
                .iter()
                .find_map(|k| found.iter().find(|(n, _)| n == k).map(|(_, v)| *v));
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn compound(entries: &[(&str, u8, Vec<u8>)], named: bool) -> Vec<u8> {
        let mut b = vec![10u8];
        if named {
            b.extend([0, 0]);
        }
        for (k, t, v) in entries {
            b.push(*t);
            b.extend((k.len() as u16).to_be_bytes());
            b.extend(k.as_bytes());
            b.extend(v);
        }
        b.push(0);
        b
    }

    #[test]
    fn nbt_reads_numbers_skipping_other_tags() {
        for named in [false, true] {
            let nbt = compound(
                &[
                    ("id", 8, [&[0u8, 5][..], b"hello"].concat()),
                    ("Items", 9, [&[10u8][..], &[0, 0, 0, 1], &[0]].concat()),
                    ("BrewTime", 2, 400i16.to_be_bytes().to_vec()),
                    ("Fuel", 1, vec![17]),
                    ("Levels", 3, 4i32.to_be_bytes().to_vec()),
                ],
                named,
            );
            assert_eq!(nbt_number(&nbt, &["BrewTime"]), Some(400));
            assert_eq!(nbt_number(&nbt, &["Fuel"]), Some(17));
            assert_eq!(nbt_number(&nbt, &["Missing", "Levels"]), Some(4));
            assert_eq!(nbt_number(&nbt, &["nope"]), None);
            assert_eq!(block_entity_value(BeField::BeaconRange, &nbt), Some(50.0));
            assert_eq!(block_entity_value(BeField::BrewTime, &nbt), Some(20_000.0));
        }
    }

    #[test]
    fn nbt_rejects_garbage() {
        assert_eq!(nbt_number(&[], &["a"]), None);
        assert_eq!(nbt_number(&[3, 1, 2], &["a"]), None);
        assert_eq!(nbt_number(&[10, 1, 0, 1, b'a'], &["a"]), None);
    }

    #[test]
    fn channels() {
        assert_eq!(channel_of("#ff8000", Channel::Red), Some(255.0));
        assert_eq!(channel_of("#ff8000", Channel::Green), Some(128.0));
        assert_eq!(channel_of("&#0000ff", Channel::Blue), Some(255.0));
        assert_eq!(channel_of("red", Channel::Red), Some(255.0));
        assert_eq!(channel_of("not a color", Channel::Red), None);
        assert_eq!(channel_of("#102030", Channel::Alpha), Some(255.0));
    }

    #[test]
    fn snake_names() {
        assert_eq!(snake("BirchForest"), "birch_forest");
        assert_eq!(snake("Plains"), "plains");
    }

    #[test]
    fn ranges() {
        assert_eq!(beacon_range(0.0), 0.0);
        assert_eq!(beacon_range(4.0), 50.0);
    }
}

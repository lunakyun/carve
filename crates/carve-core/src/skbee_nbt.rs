//! SkBee-style NBT and item components.
//!
//! Two parts live here:
//!
//! 1. A pure-Rust NBT value type ([`Tag`], [`Compound`]) with an SNBT parser
//!    and printer, tag paths (`display;Name`, `list[0]`) and the binary
//!    network format. Scripts hold these as `Value::Nbt`.
//! 2. The script syntax and engine glue: `nbt from "{a:1}"`,
//!    `int tag "a" of {_nbt}`, `set tag "a" of nbt of player's tool to 5`,
//!    `custom model data of player's tool`, and so on.
//!
//! Items are only reachable through a player's main hand, because that is
//! what the Pumpkin plugin API can read and write. Item data goes through the
//! two `Host` methods `held_component` and `set_held_component`, which move
//! raw component bytes; the byte formats are handled here, so hosts stay thin.

use crate::compile::{Cond, Effect};
use crate::engine::{Engine, Exec};
use crate::expr::{Env, Expr, Part, Prop, Ty, VarRef, parse_expr, parse_template, parse_varref};
use crate::host::Host;
use crate::lexer::Tok;
use crate::value::{PlayerRef, Value};

// ---------------------------------------------------------------- NBT type

#[derive(Debug, Clone, PartialEq)]
pub enum Tag {
    Byte(i8),
    Short(i16),
    Int(i32),
    Long(i64),
    Float(f32),
    Double(f64),
    ByteArray(Vec<i8>),
    Str(String),
    List(Vec<Tag>),
    Compound(Compound),
    IntArray(Vec<i32>),
    LongArray(Vec<i64>),
}

/// A compound keeps its keys in insertion order.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Compound(pub Vec<(String, Tag)>);

const MAX_DEPTH: usize = 64;

impl Compound {
    pub fn get(&self, k: &str) -> Option<&Tag> {
        self.0.iter().find(|(n, _)| n == k).map(|(_, t)| t)
    }

    fn get_mut(&mut self, k: &str) -> Option<&mut Tag> {
        self.0.iter_mut().find(|(n, _)| n == k).map(|(_, t)| t)
    }

    pub fn insert(&mut self, k: String, t: Tag) {
        match self.get_mut(&k) {
            Some(slot) => *slot = t,
            None => self.0.push((k, t)),
        }
    }

    pub fn remove(&mut self, k: &str) -> Option<Tag> {
        let i = self.0.iter().position(|(n, _)| n == k)?;
        Some(self.0.remove(i).1)
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }

    pub fn keys(&self) -> impl Iterator<Item = &str> {
        self.0.iter().map(|(k, _)| k.as_str())
    }

    /// Copy every key of `other` into `self`. Compounds on both sides merge deeply.
    pub fn merge(&mut self, other: &Compound) {
        for (k, v) in &other.0 {
            if let (Some(Tag::Compound(mine)), Tag::Compound(theirs)) = (self.get_mut(k), v) {
                mine.merge(theirs);
            } else {
                self.insert(k.clone(), v.clone());
            }
        }
    }

    pub fn parse(s: &str) -> Result<Compound, String> {
        match parse_snbt(s)? {
            Tag::Compound(c) => Ok(c),
            _ => Err("NBT text must be a compound like {a:1}".into()),
        }
    }

    pub fn get_path(&self, path: &str) -> Option<&Tag> {
        get_in(self, &split_path(path)?)
    }

    /// Set a tag by path, making missing compounds on the way. False when
    /// the path runs into something that is not a compound or list.
    pub fn set_path(&mut self, path: &str, tag: Tag) -> bool {
        match split_path(path) {
            Some(s) => set_in(self, &s, tag),
            None => false,
        }
    }

    pub fn remove_path(&mut self, path: &str) -> bool {
        match split_path(path) {
            Some(s) => remove_in(self, &s),
            None => false,
        }
    }
}

type Segs = Vec<(String, Option<usize>)>;

/// `a;b[2];c` into `[(a, None), (b, Some(2)), (c, None)]`.
fn split_path(path: &str) -> Option<Segs> {
    let mut out = Vec::new();
    for part in path.split(';') {
        let part = part.trim();
        if part.is_empty() {
            return None;
        }
        match part.strip_suffix(']').and_then(|p| p.split_once('[')) {
            Some((name, idx)) if !name.is_empty() => {
                out.push((name.to_string(), Some(idx.trim().parse().ok()?)))
            }
            _ => out.push((part.to_string(), None)),
        }
    }
    (!out.is_empty()).then_some(out)
}

fn get_in<'a>(c: &'a Compound, segs: &[(String, Option<usize>)]) -> Option<&'a Tag> {
    let (k, idx) = segs.first()?;
    let mut t = c.get(k)?;
    if let Some(i) = idx {
        match t {
            Tag::List(l) => t = l.get(*i)?,
            _ => return None,
        }
    }
    if segs.len() == 1 {
        return Some(t);
    }
    match t {
        Tag::Compound(cc) => get_in(cc, segs.get(1..)?),
        _ => None,
    }
}

fn set_in(c: &mut Compound, segs: &[(String, Option<usize>)], tag: Tag) -> bool {
    let Some((k, idx)) = segs.first() else {
        return false;
    };
    let last = segs.len() == 1;
    if last && idx.is_none() {
        c.insert(k.clone(), tag);
        return true;
    }
    if c.get(k).is_none() {
        if idx.is_some() {
            return false;
        }
        c.insert(k.clone(), Tag::Compound(Compound::default()));
    }
    let Some(t) = c.get_mut(k) else {
        return false;
    };
    let slot = match idx {
        Some(i) => match t {
            Tag::List(l) => {
                if last && *i == l.len() && l.first().is_none_or(|f| same_kind(f, &tag)) {
                    l.push(tag);
                    return true;
                }
                match l.get_mut(*i) {
                    Some(s) => s,
                    None => return false,
                }
            }
            _ => return false,
        },
        None => t,
    };
    if last {
        if !same_kind(slot, &tag) {
            return false;
        }
        *slot = tag;
        return true;
    }
    match (slot, segs.get(1..)) {
        (Tag::Compound(cc), Some(rest)) => set_in(cc, rest, tag),
        _ => false,
    }
}

fn remove_in(c: &mut Compound, segs: &[(String, Option<usize>)]) -> bool {
    let Some((k, idx)) = segs.first() else {
        return false;
    };
    let last = segs.len() == 1;
    if last {
        return match idx {
            None => c.remove(k).is_some(),
            Some(i) => match c.get_mut(k) {
                Some(Tag::List(l)) if *i < l.len() => {
                    l.remove(*i);
                    true
                }
                _ => false,
            },
        };
    }
    let Some(t) = c.get_mut(k) else {
        return false;
    };
    let slot = match idx {
        Some(i) => match t {
            Tag::List(l) => match l.get_mut(*i) {
                Some(s) => s,
                None => return false,
            },
            _ => return false,
        },
        None => t,
    };
    match (slot, segs.get(1..)) {
        (Tag::Compound(cc), Some(rest)) => remove_in(cc, rest),
        _ => false,
    }
}

fn same_kind(a: &Tag, b: &Tag) -> bool {
    std::mem::discriminant(a) == std::mem::discriminant(b)
}

// ------------------------------------------------------------------- SNBT

struct Snbt {
    c: Vec<char>,
    i: usize,
}

/// Parse SNBT text such as `{a:1,b:"x",c:[1,2],d:1b,e:2.5f,f:[I;1,2]}`.
pub fn parse_snbt(s: &str) -> Result<Tag, String> {
    let mut p = Snbt {
        c: s.chars().collect(),
        i: 0,
    };
    let t = p.value(0)?;
    p.ws();
    if p.i < p.c.len() {
        return Err(format!("unexpected '{}' in NBT text", p.c[p.i]));
    }
    Ok(t)
}

impl Snbt {
    fn ws(&mut self) {
        while self.c.get(self.i).is_some_and(|c| c.is_whitespace()) {
            self.i += 1;
        }
    }

    fn peek(&mut self) -> Option<char> {
        self.ws();
        self.c.get(self.i).copied()
    }

    fn eat(&mut self, ch: char) -> bool {
        if self.peek() == Some(ch) {
            self.i += 1;
            true
        } else {
            false
        }
    }

    fn expect(&mut self, ch: char) -> Result<(), String> {
        if self.eat(ch) {
            Ok(())
        } else {
            Err(match self.c.get(self.i) {
                Some(c) => format!("expected '{ch}' but found '{c}' in NBT text"),
                None => format!("expected '{ch}' but the NBT text ended"),
            })
        }
    }

    fn value(&mut self, depth: usize) -> Result<Tag, String> {
        if depth > MAX_DEPTH {
            return Err("NBT text is nested too deeply".into());
        }
        match self.peek() {
            None => Err("the NBT text ended too soon".into()),
            Some('{') => {
                self.i += 1;
                self.compound(depth)
            }
            Some('[') => {
                self.i += 1;
                self.list(depth)
            }
            Some(q @ ('"' | '\'')) => {
                self.i += 1;
                Ok(Tag::Str(self.quoted(q)?))
            }
            Some(_) => {
                let w = self.bare();
                if w.is_empty() {
                    return Err(format!("unexpected '{}' in NBT text", self.c[self.i]));
                }
                Ok(classify(&w))
            }
        }
    }

    fn bare(&mut self) -> String {
        let start = self.i;
        while self
            .c
            .get(self.i)
            .is_some_and(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+'))
        {
            self.i += 1;
        }
        self.c[start..self.i].iter().collect()
    }

    fn quoted(&mut self, q: char) -> Result<String, String> {
        let mut out = String::new();
        while let Some(&ch) = self.c.get(self.i) {
            self.i += 1;
            if ch == q {
                return Ok(out);
            }
            if ch == '\\' {
                let Some(&n) = self.c.get(self.i) else {
                    break;
                };
                self.i += 1;
                out.push(match n {
                    'n' => '\n',
                    't' => '\t',
                    'r' => '\r',
                    other => other,
                });
            } else {
                out.push(ch);
            }
        }
        Err("a quoted string in the NBT text is not closed".into())
    }

    fn compound(&mut self, depth: usize) -> Result<Tag, String> {
        let mut c = Compound::default();
        if self.eat('}') {
            return Ok(Tag::Compound(c));
        }
        loop {
            let key = match self.peek() {
                Some(q @ ('"' | '\'')) => {
                    self.i += 1;
                    self.quoted(q)?
                }
                _ => {
                    let k = self.bare();
                    if k.is_empty() {
                        return Err("expected a key in the NBT text".into());
                    }
                    k
                }
            };
            self.expect(':')?;
            let v = self.value(depth + 1)?;
            c.insert(key, v);
            if self.eat(',') {
                if self.eat('}') {
                    break;
                }
                continue;
            }
            self.expect('}')?;
            break;
        }
        Ok(Tag::Compound(c))
    }

    fn list(&mut self, depth: usize) -> Result<Tag, String> {
        self.ws();
        let kind = match (self.c.get(self.i), self.c.get(self.i + 1)) {
            (Some(k @ ('B' | 'I' | 'L')), Some(';')) => Some(*k),
            _ => None,
        };
        if kind.is_some() {
            self.i += 2;
        }
        let mut items = Vec::new();
        if !self.eat(']') {
            loop {
                items.push(self.value(depth + 1)?);
                if self.eat(',') {
                    if self.eat(']') {
                        break;
                    }
                    continue;
                }
                self.expect(']')?;
                break;
            }
        }
        let num = |t: &Tag| match t {
            Tag::Byte(n) => Some(i64::from(*n)),
            Tag::Short(n) => Some(i64::from(*n)),
            Tag::Int(n) => Some(i64::from(*n)),
            Tag::Long(n) => Some(*n),
            _ => None,
        };
        match kind {
            None => {
                if let Some(first) = items.first()
                    && items.iter().any(|t| !same_kind(t, first))
                {
                    return Err("all items of an NBT list must have the same type".into());
                }
                Ok(Tag::List(items))
            }
            Some(k) => {
                let nums: Option<Vec<i64>> = items.iter().map(num).collect();
                let nums = nums.ok_or("array items must be whole numbers")?;
                Ok(match k {
                    'B' => Tag::ByteArray(nums.iter().map(|n| *n as i8).collect()),
                    'I' => Tag::IntArray(nums.iter().map(|n| *n as i32).collect()),
                    _ => Tag::LongArray(nums),
                })
            }
        }
    }
}

/// An unquoted word: a number with an optional type suffix, `true`/`false`, or text.
fn classify(w: &str) -> Tag {
    match w {
        "true" => return Tag::Byte(1),
        "false" => return Tag::Byte(0),
        _ => {}
    }
    let last = w.chars().last().unwrap_or(' ');
    let body = &w[..w.len() - last.len_utf8()];
    let plain_int = |s: &str| {
        let d = s.strip_prefix(['-', '+']).unwrap_or(s);
        !d.is_empty() && d.chars().all(|c| c.is_ascii_digit())
    };
    let plain_float = |s: &str| {
        let d = s.strip_prefix(['-', '+']).unwrap_or(s);
        d.chars().next().is_some_and(|c| c.is_ascii_digit() || c == '.')
            && d.chars()
                .all(|c| c.is_ascii_digit() || matches!(c, '.' | 'e' | 'E' | '-' | '+'))
            && d.parse::<f64>().is_ok()
    };
    let suffixed = match last.to_ascii_lowercase() {
        'b' if plain_int(body) => body.parse().ok().map(Tag::Byte),
        's' if plain_int(body) => body.parse().ok().map(Tag::Short),
        'l' if plain_int(body) => body.parse().ok().map(Tag::Long),
        'f' if plain_float(body) => body.parse().ok().map(Tag::Float),
        'd' if plain_float(body) => body.parse().ok().map(Tag::Double),
        _ => None,
    };
    if let Some(t) = suffixed {
        return t;
    }
    if plain_int(w)
        && let Ok(n) = w.parse::<i32>()
    {
        return Tag::Int(n);
    }
    if plain_float(w)
        && w.contains(['.', 'e', 'E'])
        && let Ok(n) = w.parse::<f64>()
    {
        return Tag::Double(n);
    }
    Tag::Str(w.to_string())
}

fn push_quoted(out: &mut String, s: &str) {
    out.push('"');
    for ch in s.chars() {
        match ch {
            '"' | '\\' => {
                out.push('\\');
                out.push(ch);
            }
            '\n' => out.push_str("\\n"),
            _ => out.push(ch),
        }
    }
    out.push('"');
}

impl Tag {
    pub fn push_snbt(&self, out: &mut String) {
        use std::fmt::Write;
        match self {
            Tag::Byte(n) => {
                let _ = write!(out, "{n}b");
            }
            Tag::Short(n) => {
                let _ = write!(out, "{n}s");
            }
            Tag::Int(n) => {
                let _ = write!(out, "{n}");
            }
            Tag::Long(n) => {
                let _ = write!(out, "{n}L");
            }
            Tag::Float(n) => {
                let _ = write!(out, "{n}f");
            }
            Tag::Double(n) => {
                let _ = write!(out, "{n}d");
            }
            Tag::Str(s) => push_quoted(out, s),
            Tag::ByteArray(a) => push_array(out, "B", a, "b"),
            Tag::IntArray(a) => push_array(out, "I", a, ""),
            Tag::LongArray(a) => push_array(out, "L", a, "L"),
            Tag::List(l) => {
                out.push('[');
                for (i, t) in l.iter().enumerate() {
                    if i > 0 {
                        out.push(',');
                    }
                    t.push_snbt(out);
                }
                out.push(']');
            }
            Tag::Compound(c) => c.push_snbt(out),
        }
    }

    pub fn to_snbt(&self) -> String {
        let mut s = String::new();
        self.push_snbt(&mut s);
        s
    }
}

fn push_array<T: std::fmt::Display>(out: &mut String, kind: &str, a: &[T], suffix: &str) {
    use std::fmt::Write;
    out.push('[');
    out.push_str(kind);
    out.push(';');
    for (i, n) in a.iter().enumerate() {
        if i > 0 {
            out.push(',');
        }
        let _ = write!(out, "{n}{suffix}");
    }
    out.push(']');
}

impl Compound {
    pub fn push_snbt(&self, out: &mut String) {
        out.push('{');
        for (i, (k, t)) in self.0.iter().enumerate() {
            if i > 0 {
                out.push(',');
            }
            let plain = !k.is_empty()
                && k.chars()
                    .all(|c| c.is_ascii_alphanumeric() || matches!(c, '_' | '-' | '.' | '+'));
            if plain {
                out.push_str(k);
            } else {
                push_quoted(out, k);
            }
            out.push(':');
            t.push_snbt(out);
        }
        out.push('}');
    }

    pub fn to_snbt(&self) -> String {
        let mut s = String::new();
        self.push_snbt(&mut s);
        s
    }
}

// ----------------------------------------------------------- binary format

impl Compound {
    /// Network NBT: type byte 10, then the compound body (no root name).
    pub fn to_bytes(&self) -> Result<Vec<u8>, String> {
        let mut out = vec![10];
        write_body(self, &mut out)?;
        Ok(out)
    }

    /// Reads what [`Compound::to_bytes`] writes. A lone `0` byte is an empty compound.
    pub fn from_bytes(b: &[u8]) -> Result<Compound, String> {
        match b.first() {
            None | Some(0) => Ok(Compound::default()),
            Some(10) => {
                let mut r = Reader { b, i: 1 };
                read_body(&mut r, 0)
            }
            Some(_) => Err("item data is not an NBT compound".into()),
        }
    }
}

fn tag_id(t: &Tag) -> u8 {
    match t {
        Tag::Byte(_) => 1,
        Tag::Short(_) => 2,
        Tag::Int(_) => 3,
        Tag::Long(_) => 4,
        Tag::Float(_) => 5,
        Tag::Double(_) => 6,
        Tag::ByteArray(_) => 7,
        Tag::Str(_) => 8,
        Tag::List(_) => 9,
        Tag::Compound(_) => 10,
        Tag::IntArray(_) => 11,
        Tag::LongArray(_) => 12,
    }
}

fn write_str(s: &str, out: &mut Vec<u8>) -> Result<(), String> {
    let n = u16::try_from(s.len()).map_err(|_| "NBT text is too long".to_string())?;
    out.extend_from_slice(&n.to_be_bytes());
    out.extend_from_slice(s.as_bytes());
    Ok(())
}

fn write_len(n: usize, out: &mut Vec<u8>) -> Result<(), String> {
    let n = i32::try_from(n).map_err(|_| "NBT list is too long".to_string())?;
    out.extend_from_slice(&n.to_be_bytes());
    Ok(())
}

fn write_body(c: &Compound, out: &mut Vec<u8>) -> Result<(), String> {
    for (k, t) in &c.0 {
        out.push(tag_id(t));
        write_str(k, out)?;
        write_payload(t, out)?;
    }
    out.push(0);
    Ok(())
}

fn write_payload(t: &Tag, out: &mut Vec<u8>) -> Result<(), String> {
    match t {
        Tag::Byte(n) => out.push(*n as u8),
        Tag::Short(n) => out.extend_from_slice(&n.to_be_bytes()),
        Tag::Int(n) => out.extend_from_slice(&n.to_be_bytes()),
        Tag::Long(n) => out.extend_from_slice(&n.to_be_bytes()),
        Tag::Float(n) => out.extend_from_slice(&n.to_be_bytes()),
        Tag::Double(n) => out.extend_from_slice(&n.to_be_bytes()),
        Tag::ByteArray(a) => {
            write_len(a.len(), out)?;
            out.extend(a.iter().map(|b| *b as u8));
        }
        Tag::Str(s) => write_str(s, out)?,
        Tag::List(l) => {
            let id = l.first().map_or(0, tag_id);
            if l.iter().any(|t| tag_id(t) != id) {
                return Err("all items of an NBT list must have the same type".into());
            }
            out.push(id);
            write_len(l.len(), out)?;
            for t in l {
                write_payload(t, out)?;
            }
        }
        Tag::Compound(c) => write_body(c, out)?,
        Tag::IntArray(a) => {
            write_len(a.len(), out)?;
            for n in a {
                out.extend_from_slice(&n.to_be_bytes());
            }
        }
        Tag::LongArray(a) => {
            write_len(a.len(), out)?;
            for n in a {
                out.extend_from_slice(&n.to_be_bytes());
            }
        }
    }
    Ok(())
}

struct Reader<'a> {
    b: &'a [u8],
    i: usize,
}

impl Reader<'_> {
    fn take(&mut self, n: usize) -> Result<&[u8], String> {
        let end = self.i.checked_add(n).filter(|e| *e <= self.b.len());
        let end = end.ok_or("the item NBT data is cut off")?;
        let s = &self.b[self.i..end];
        self.i = end;
        Ok(s)
    }

    fn arr<const N: usize>(&mut self) -> Result<[u8; N], String> {
        let mut a = [0u8; N];
        a.copy_from_slice(self.take(N)?);
        Ok(a)
    }

    fn len(&mut self) -> Result<usize, String> {
        let n = i32::from_be_bytes(self.arr()?);
        usize::try_from(n).map_err(|_| "negative length in item NBT data".to_string())
    }

    fn string(&mut self) -> Result<String, String> {
        let n = usize::from(u16::from_be_bytes(self.arr()?));
        Ok(String::from_utf8_lossy(self.take(n)?).into_owned())
    }
}

fn read_body(r: &mut Reader, depth: usize) -> Result<Compound, String> {
    if depth > MAX_DEPTH {
        return Err("item NBT data is nested too deeply".into());
    }
    let mut c = Compound::default();
    loop {
        let id = r.take(1)?[0];
        if id == 0 {
            return Ok(c);
        }
        let k = r.string()?;
        let t = read_payload(id, r, depth + 1)?;
        c.insert(k, t);
    }
}

fn read_payload(id: u8, r: &mut Reader, depth: usize) -> Result<Tag, String> {
    Ok(match id {
        1 => Tag::Byte(r.take(1)?[0] as i8),
        2 => Tag::Short(i16::from_be_bytes(r.arr()?)),
        3 => Tag::Int(i32::from_be_bytes(r.arr()?)),
        4 => Tag::Long(i64::from_be_bytes(r.arr()?)),
        5 => Tag::Float(f32::from_be_bytes(r.arr()?)),
        6 => Tag::Double(f64::from_be_bytes(r.arr()?)),
        7 => {
            let n = r.len()?;
            Tag::ByteArray(r.take(n)?.iter().map(|b| *b as i8).collect())
        }
        8 => Tag::Str(r.string()?),
        9 => {
            if depth > MAX_DEPTH {
                return Err("item NBT data is nested too deeply".into());
            }
            let item = r.take(1)?[0];
            let n = r.len()?;
            let mut l = Vec::new();
            for _ in 0..n {
                l.push(read_payload(item, r, depth + 1)?);
            }
            Tag::List(l)
        }
        10 => Tag::Compound(read_body(r, depth)?),
        11 => {
            let n = r.len()?;
            let mut a = Vec::new();
            for _ in 0..n {
                a.push(i32::from_be_bytes(r.arr()?));
            }
            Tag::IntArray(a)
        }
        12 => {
            let n = r.len()?;
            let mut a = Vec::new();
            for _ in 0..n {
                a.push(i64::from_be_bytes(r.arr()?));
            }
            Tag::LongArray(a)
        }
        _ => return Err(format!("unknown NBT tag type {id}")),
    })
}

// -------------------------------------------------------- tags and values

/// The type word in front of `tag`: `int tag "a" of ...`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NbtTy {
    Byte,
    Short,
    Int,
    Long,
    Float,
    Double,
    Str,
    Bool,
    Compound,
    List,
    ByteArray,
    IntArray,
    LongArray,
}

/// Script value from a tag, when it fits the asked type (`None` otherwise).
pub fn tag_to_value(ty: Option<NbtTy>, t: &Tag) -> Value {
    let num = match t {
        Tag::Byte(n) => Some(f64::from(*n)),
        Tag::Short(n) => Some(f64::from(*n)),
        Tag::Int(n) => Some(f64::from(*n)),
        Tag::Long(n) => Some(*n as f64),
        Tag::Float(n) => Some(f64::from(*n)),
        Tag::Double(n) => Some(*n),
        _ => None,
    };
    let fits = match ty {
        None => true,
        Some(
            NbtTy::Byte
            | NbtTy::Short
            | NbtTy::Int
            | NbtTy::Long
            | NbtTy::Float
            | NbtTy::Double
            | NbtTy::Bool,
        ) => num.is_some(),
        Some(NbtTy::Str) => matches!(t, Tag::Str(_)),
        Some(NbtTy::Compound) => matches!(t, Tag::Compound(_)),
        Some(NbtTy::List) => matches!(t, Tag::List(_)),
        Some(NbtTy::ByteArray) => matches!(t, Tag::ByteArray(_)),
        Some(NbtTy::IntArray) => matches!(t, Tag::IntArray(_)),
        Some(NbtTy::LongArray) => matches!(t, Tag::LongArray(_)),
    };
    if !fits {
        return Value::None;
    }
    if ty == Some(NbtTy::Bool) {
        return Value::Bool(num.is_some_and(|n| n != 0.0));
    }
    if let Some(n) = num {
        return Value::Num(n);
    }
    match t {
        Tag::Str(s) => Value::Text(s.clone()),
        Tag::Compound(c) => Value::Nbt(Box::new(c.clone())),
        Tag::List(l) => Value::List(l.iter().map(|x| tag_to_value(None, x)).collect()),
        Tag::ByteArray(a) => Value::List(a.iter().map(|n| Value::Num(f64::from(*n))).collect()),
        Tag::IntArray(a) => Value::List(a.iter().map(|n| Value::Num(f64::from(*n))).collect()),
        Tag::LongArray(a) => Value::List(a.iter().map(|n| Value::Num(*n as f64)).collect()),
        _ => Value::None,
    }
}

fn plain_tag(v: &Value) -> Option<Tag> {
    Some(match v {
        Value::Num(n) => {
            if n.fract() == 0.0 && n.abs() <= f64::from(i32::MAX) {
                Tag::Int(*n as i32)
            } else {
                Tag::Double(*n)
            }
        }
        Value::Bool(b) => Tag::Byte(i8::from(*b)),
        Value::Nbt(c) => Tag::Compound((**c).clone()),
        Value::List(items) => {
            let tags: Option<Vec<Tag>> = items.iter().map(plain_tag).collect();
            let tags = tags?;
            if let Some(f) = tags.first()
                && tags.iter().any(|t| !same_kind(t, f))
            {
                return None;
            }
            Tag::List(tags)
        }
        Value::None => return None,
        other => Tag::Str(other.to_string()),
    })
}

/// A tag from a script value, using the type word if there is one.
pub fn value_to_tag(ty: Option<NbtTy>, v: &Value) -> Option<Tag> {
    let n = || match v {
        Value::Num(n) => Some(*n),
        Value::Bool(b) => Some(f64::from(u8::from(*b))),
        Value::Text(t) => t.trim().parse::<f64>().ok(),
        _ => None,
    };
    let whole = |lo: f64, hi: f64| n().map(|x| x.round().clamp(lo, hi));
    Some(match ty {
        None => match v {
            Value::Text(_) => Tag::Str(v.to_string()),
            _ => plain_tag(v)?,
        },
        Some(NbtTy::Byte) => Tag::Byte(whole(-128.0, 127.0)? as i8),
        Some(NbtTy::Short) => Tag::Short(whole(-32768.0, 32767.0)? as i16),
        Some(NbtTy::Int) => Tag::Int(whole(f64::from(i32::MIN), f64::from(i32::MAX))? as i32),
        Some(NbtTy::Long) => Tag::Long(n()?.round() as i64),
        Some(NbtTy::Float) => Tag::Float(n()? as f32),
        Some(NbtTy::Double) => Tag::Double(n()?),
        Some(NbtTy::Bool) => Tag::Byte(match v {
            Value::Bool(b) => i8::from(*b),
            Value::Text(t) => i8::from(t.trim().eq_ignore_ascii_case("true")),
            _ => i8::from(n()? != 0.0),
        }),
        Some(NbtTy::Str) => Tag::Str(v.to_string()),
        Some(NbtTy::Compound) => match v {
            Value::Nbt(c) => Tag::Compound((**c).clone()),
            Value::Text(t) => Tag::Compound(Compound::parse(t).ok()?),
            _ => return None,
        },
        Some(NbtTy::List) => match v {
            Value::Text(t) => match parse_snbt(t).ok()? {
                l @ Tag::List(_) => l,
                _ => return None,
            },
            _ => match plain_tag(v)? {
                l @ Tag::List(_) => l,
                _ => return None,
            },
        },
        Some(a @ (NbtTy::ByteArray | NbtTy::IntArray | NbtTy::LongArray)) => {
            let items: Vec<f64> = match v {
                Value::List(l) => l.iter().map(Value::as_num).collect::<Option<_>>()?,
                Value::Text(t) => {
                    let body = t.trim().trim_start_matches('[').trim_end_matches(']');
                    let body = body.split_once(';').map_or(body, |(_, b)| b);
                    body.split(',')
                        .filter(|s| !s.trim().is_empty())
                        .map(|s| {
                            s.trim()
                                .trim_end_matches(['b', 'B', 'l', 'L'])
                                .parse::<f64>()
                                .ok()
                        })
                        .collect::<Option<_>>()?
                }
                _ => return None,
            };
            match a {
                NbtTy::ByteArray => Tag::ByteArray(items.iter().map(|x| *x as i8).collect()),
                NbtTy::IntArray => Tag::IntArray(items.iter().map(|x| *x as i32).collect()),
                _ => Tag::LongArray(items.iter().map(|x| *x as i64).collect()),
            }
        }
    })
}

// -------------------------------------------------------- item components

/// Item data components scripts can read and write on the held item.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ItemComp {
    CustomData,
    CustomModelData,
    Unbreakable,
    MaxStackSize,
    Rarity,
    GlintOverride,
    ItemModel,
    /// Enchantments kept in an enchanted book (`w5_chunk6`).
    StoredEnchantments,
}

const RARITIES: [&str; 4] = ["common", "uncommon", "rare", "epic"];

fn put_varint(mut n: u32, out: &mut Vec<u8>) {
    loop {
        let b = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(b);
            return;
        }
        out.push(b | 0x80);
    }
}

fn get_varint(b: &[u8], at: &mut usize) -> Option<u32> {
    let mut n = 0u32;
    for shift in 0..5 {
        let byte = *b.get(*at)?;
        *at += 1;
        n |= u32::from(byte & 0x7f) << (7 * shift);
        if byte & 0x80 == 0 {
            return Some(n);
        }
    }
    None
}

impl ItemComp {
    pub fn name(self) -> &'static str {
        match self {
            ItemComp::CustomData => "custom data",
            ItemComp::CustomModelData => "custom model data",
            ItemComp::Unbreakable => "unbreakable",
            ItemComp::MaxStackSize => "max stack size",
            ItemComp::Rarity => "rarity",
            ItemComp::GlintOverride => "glint override",
            ItemComp::ItemModel => "item model",
            ItemComp::StoredEnchantments => "stored enchantments",
        }
    }

    /// Script value for stored component bytes.
    pub fn decode(self, b: &[u8]) -> Value {
        let mut at = 0;
        match self {
            ItemComp::CustomData => Compound::from_bytes(b).map_or(Value::None, |c| Value::Nbt(Box::new(c))),
            ItemComp::Unbreakable => Value::Bool(true),
            ItemComp::MaxStackSize => {
                get_varint(b, &mut at).map_or(Value::None, |n| Value::Num(f64::from(n)))
            }
            ItemComp::Rarity => get_varint(b, &mut at)
                .and_then(|n| RARITIES.get(n as usize))
                .map_or(Value::None, |r| Value::Text((*r).into())),
            ItemComp::GlintOverride => b.first().map_or(Value::None, |x| Value::Bool(*x != 0)),
            ItemComp::StoredEnchantments => crate::w5_chunk6::stored_decode(b),
            ItemComp::ItemModel => {
                let Some(n) = get_varint(b, &mut at) else {
                    return Value::None;
                };
                b.get(at..at + n as usize).map_or(Value::None, |s| {
                    Value::Text(String::from_utf8_lossy(s).into_owned())
                })
            }
            ItemComp::CustomModelData => {
                let Some(n) = get_varint(b, &mut at) else {
                    return Value::None;
                };
                if n == 0 {
                    return Value::None;
                }
                match b.get(at..at + 4) {
                    Some(x) => Value::Num(f64::from(f32::from_be_bytes([x[0], x[1], x[2], x[3]]))),
                    None => Value::None,
                }
            }
        }
    }

    /// Component bytes for a script value. `None`: the value does not fit,
    /// or (for `Unbreakable`, `false`) the component should be removed.
    pub fn encode(self, v: &Value) -> Result<Option<Vec<u8>>, String> {
        let mut out = Vec::new();
        let bad = || format!("can't use {v} as the {} of an item", self.name());
        match self {
            ItemComp::CustomData => return Err("use the nbt effects to change custom data".into()),
            ItemComp::Unbreakable => {
                return match v {
                    Value::Bool(true) => Ok(Some(out)),
                    Value::Bool(false) => Ok(None),
                    _ => Err(bad()),
                };
            }
            ItemComp::MaxStackSize => {
                let n = v.as_num().ok_or_else(bad)?;
                put_varint(n.round().clamp(1.0, 99.0) as u32, &mut out);
            }
            ItemComp::Rarity => {
                let t = v.to_string().to_ascii_lowercase();
                let i = RARITIES
                    .iter()
                    .position(|r| *r == t.trim())
                    .ok_or_else(|| format!("rarity must be common, uncommon, rare or epic, not '{t}'"))?;
                put_varint(i as u32, &mut out);
            }
            ItemComp::StoredEnchantments => return crate::w5_chunk6::stored_encode(v).map(Some),
            ItemComp::GlintOverride => match v {
                Value::Bool(b) => out.push(u8::from(*b)),
                _ => return Err(bad()),
            },
            ItemComp::ItemModel => {
                let t = v.to_string();
                let t = if t.contains(':') {
                    t
                } else {
                    format!("minecraft:{t}")
                };
                put_varint(t.len() as u32, &mut out);
                out.extend_from_slice(t.as_bytes());
            }
            ItemComp::CustomModelData => {
                let n = v.as_num().ok_or_else(bad)?;
                put_varint(1, &mut out);
                out.extend_from_slice(&(n as f32).to_be_bytes());
                for _ in 0..3 {
                    put_varint(0, &mut out);
                }
            }
        }
        Ok(Some(out))
    }

    fn from_words(w: &[&str]) -> Option<ItemComp> {
        Some(match w {
            ["custom", "model", "data"] => ItemComp::CustomModelData,
            ["max", "stack", "size"] => ItemComp::MaxStackSize,
            ["rarity"] => ItemComp::Rarity,
            ["item", "model"] => ItemComp::ItemModel,
            ["glint", "override"] | ["enchantment", "glint", "override"] | ["glint"] => {
                ItemComp::GlintOverride
            }
            ["unbreakable"] => ItemComp::Unbreakable,
            ["stored", "enchantments" | "enchants" | "enchantment"] => ItemComp::StoredEnchantments,
            _ => return None,
        })
    }
}

// ------------------------------------------------------------- script AST

#[derive(Debug, Clone, PartialEq)]
pub enum NbtExpr {
    /// `nbt from "{a:1}"`
    FromText(Expr),
    /// `nbt of player's tool`: a player.
    OfTool(Expr),
    /// `nbt of {_x}`: a variable or text holding NBT.
    Of(Expr),
    /// `int tag "a;b" of <source>`
    Tag {
        ty: Option<NbtTy>,
        path: Expr,
        src: Expr,
    },
    /// `tag names of <source>`
    Keys(Expr),
    /// `snbt of <source>`
    Snbt(Expr),
    /// `custom model data of player's tool`: component, player.
    Comp(ItemComp, Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NbtTarget {
    Var(VarRef),
    /// The main-hand item of a player.
    Tool(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub enum NbtEffect {
    SetTag {
        ty: Option<NbtTy>,
        path: Expr,
        target: NbtTarget,
        value: Expr,
    },
    DelTag {
        path: Expr,
        target: NbtTarget,
    },
    Merge {
        value: Expr,
        target: NbtTarget,
    },
    SetAll {
        value: Expr,
        target: NbtTarget,
    },
    Clear(NbtTarget),
    /// Set (`Some`) or remove (`None`) a component of a player's main-hand item.
    SetComp {
        comp: ItemComp,
        player: Expr,
        value: Option<Expr>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub struct NbtCond {
    pub src: Expr,
    pub ty: Option<NbtTy>,
    pub path: Expr,
    pub negate: bool,
}

// ---------------------------------------------------------------- parsing

#[derive(Debug, Clone)]
pub enum HeadKind {
    FromText,
    Of,
    Tag(Option<NbtTy>, Tok),
    Keys,
    Snbt,
    Comp(ItemComp),
}

#[derive(Debug, Clone)]
pub struct Head {
    pub kind: HeadKind,
    /// Index of the first token after the head (where the operand starts).
    pub after: usize,
}

fn lw(toks: &[Tok], i: usize) -> Option<String> {
    match toks.get(i) {
        Some(Tok::Word(w)) => Some(w.to_ascii_lowercase()),
        _ => None,
    }
}

fn ty_at(toks: &[Tok], i: usize) -> Option<(NbtTy, usize)> {
    let a = lw(toks, i)?;
    if lw(toks, i + 1).as_deref() == Some("array") {
        let t = match a.as_str() {
            "byte" => NbtTy::ByteArray,
            "int" | "integer" => NbtTy::IntArray,
            "long" => NbtTy::LongArray,
            _ => return None,
        };
        return Some((t, 2));
    }
    let t = match a.as_str() {
        "byte" => NbtTy::Byte,
        "short" => NbtTy::Short,
        "int" | "integer" => NbtTy::Int,
        "long" => NbtTy::Long,
        "float" => NbtTy::Float,
        "double" => NbtTy::Double,
        "string" | "text" => NbtTy::Str,
        "boolean" | "bool" => NbtTy::Bool,
        "compound" => NbtTy::Compound,
        "list" => NbtTy::List,
        _ => return None,
    };
    Some((t, 1))
}

/// Cheap test, no allocation: can a head start at `toks[i]`? Looks at the
/// word length first because the parser asks this for every word.
fn may_head(toks: &[Tok], i: usize) -> bool {
    let Some(Tok::Word(w)) = toks.get(i) else {
        return false;
    };
    let any = |list: &[&str]| list.iter().any(|h| w.eq_ignore_ascii_case(h));
    match w.len() {
        3 => any(&["nbt", "int", "tag", "max"]),
        4 => any(&["byte", "long", "text", "bool", "list", "item", "snbt"]),
        5 => any(&["short", "float", "glint"]),
        6 => any(&["double", "string", "custom", "rarity", "stored"]),
        7 => any(&["integer", "boolean"]),
        8 => any(&["compound"]),
        11 => any(&["enchantment", "unbreakable"]),
        _ => false,
    }
}

/// Does an NBT or component phrase start at `toks[i]`?
pub fn head(toks: &[Tok], i: usize) -> Option<Head> {
    if !may_head(toks, i) {
        return None;
    }
    let f = lw(toks, i)?;
    if f == "snbt" {
        return (lw(toks, i + 1).as_deref() == Some("of")).then_some(Head {
            kind: HeadKind::Snbt,
            after: i + 2,
        });
    }
    let mut j = i;
    if f == "nbt" {
        j = i + 1;
        if lw(toks, j).as_deref() == Some("compound") {
            j += 1;
        }
        match lw(toks, j).as_deref() {
            Some("from") => {
                return Some(Head {
                    kind: HeadKind::FromText,
                    after: j + 1,
                });
            }
            Some("of") => {
                return Some(Head {
                    kind: HeadKind::Of,
                    after: j + 1,
                });
            }
            _ => j = i + 1,
        }
    }
    // `[nbt] [type] tag "path" of`, `[nbt] tag names of`
    let (ty, n) = match ty_at(toks, j) {
        Some((t, n)) if lw(toks, j + n).as_deref() == Some("tag") => (Some(t), n),
        _ => (None, 0),
    };
    j += n;
    if lw(toks, j).as_deref() == Some("tag") || (f == "nbt" && lw(toks, j).as_deref() == Some("keys")) {
        if matches!(lw(toks, j + 1).as_deref(), Some("names" | "keys"))
            && lw(toks, j + 2).as_deref() == Some("of")
        {
            return Some(Head {
                kind: HeadKind::Keys,
                after: j + 3,
            });
        }
        if lw(toks, j).as_deref() == Some("keys") && lw(toks, j + 1).as_deref() == Some("of") {
            return Some(Head {
                kind: HeadKind::Keys,
                after: j + 2,
            });
        }
        let p = toks.get(j + 1)?;
        if matches!(p, Tok::Str(_) | Tok::Var(_)) && lw(toks, j + 2).as_deref() == Some("of") {
            return Some(Head {
                kind: HeadKind::Tag(ty, p.clone()),
                after: j + 3,
            });
        }
        return None;
    }
    // components: up to three words, then `of`
    for n in 1..=3 {
        let words: Option<Vec<String>> = (0..n).map(|k| lw(toks, i + k)).collect();
        let words = words?;
        let refs: Vec<&str> = words.iter().map(String::as_str).collect();
        if let Some(c) = ItemComp::from_words(&refs)
            && lw(toks, i + n).as_deref() == Some("of")
        {
            return Some(Head {
                kind: HeadKind::Comp(c),
                after: i + n + 1,
            });
        }
    }
    None
}

fn tool_player(e: &Expr) -> Option<&Expr> {
    match e {
        Expr::Prop(Prop::Tool, p) => Some(p),
        _ => None,
    }
}

/// The source of a tag lookup: `player's tool` means the item's NBT.
fn src_of(e: Expr) -> Expr {
    match e {
        Expr::Prop(Prop::Tool, p) => Expr::Nbt(Box::new(NbtExpr::OfTool(*p))),
        e => e,
    }
}

fn path_expr(t: &Tok, env: &Env) -> Result<Expr, String> {
    match t {
        Tok::Str(s) => Ok(Expr::Text(parse_template(s, env)?)),
        Tok::Var(v) => Ok(Expr::Var(parse_varref(v, env)?)),
        _ => Err("expected the tag path in quotes".into()),
    }
}

/// Build the expression once the parser has read the operand after the head.
pub fn build(h: &Head, operand: Expr, env: &Env) -> Result<Expr, String> {
    let n = match &h.kind {
        HeadKind::FromText => NbtExpr::FromText(operand),
        HeadKind::Of => match operand {
            Expr::Prop(Prop::Tool, p) => NbtExpr::OfTool(*p),
            other => NbtExpr::Of(other),
        },
        HeadKind::Tag(ty, p) => NbtExpr::Tag {
            ty: *ty,
            path: path_expr(p, env)?,
            src: src_of(operand),
        },
        HeadKind::Keys => NbtExpr::Keys(src_of(operand)),
        HeadKind::Snbt => NbtExpr::Snbt(src_of(operand)),
        HeadKind::Comp(c) => match tool_player(&operand) {
            Some(p) => NbtExpr::Comp(*c, p.clone()),
            // `max stack size of diamond`: the item type's own limit.
            None if *c == ItemComp::MaxStackSize => {
                return Ok(crate::w2_expressions_5::boxed(
                    crate::w2_expressions_5::W5::MaxStack(Box::new(operand)),
                ));
            }
            None => return Err(format!("use '{} of player's tool'", c.name())),
        },
    };
    Ok(Expr::Nbt(Box::new(n)))
}

pub fn expr_ty(n: &NbtExpr) -> Ty {
    match n {
        NbtExpr::Snbt(_) => Ty::Text,
        NbtExpr::Keys(_) => Ty::List,
        NbtExpr::Comp(ItemComp::StoredEnchantments, _) => Ty::List,
        _ => Ty::Object,
    }
}

fn target_of(e: &Expr) -> Option<NbtTarget> {
    match e {
        Expr::Var(v) if !v.is_list() => Some(NbtTarget::Var(v.clone())),
        Expr::Prop(Prop::Tool, p) => Some(NbtTarget::Tool((**p).clone())),
        Expr::Nbt(n) => match &**n {
            NbtExpr::OfTool(p) => Some(NbtTarget::Tool(p.clone())),
            NbtExpr::Of(inner) => target_of(inner),
            _ => None,
        },
        _ => None,
    }
}

const TARGET_HELP: &str = "can't change that; use a variable like {_nbt} or the player's tool";

fn is_word(t: &Tok, w: &str) -> bool {
    matches!(t, Tok::Word(x) if x.eq_ignore_ascii_case(w))
}

fn value_expr(toks: &[Tok], env: &Env) -> Result<Expr, String> {
    match parse_expr(toks, env) {
        Ok(e) => Ok(e),
        Err(msg) => match toks {
            [Tok::Word(w)] => Ok(Expr::Text(vec![Part::Lit(w.to_ascii_lowercase())])),
            _ => Err(msg),
        },
    }
}

/// Parse `<head> <target>` where the head came from [`head`] at index 0.
fn spec(toks: &[Tok], env: &Env) -> Result<Option<(Head, Expr)>, String> {
    let Some(h) = head(toks, 0) else {
        return Ok(None);
    };
    let rest = toks.get(h.after..).unwrap_or_default();
    if rest.is_empty() {
        return Err("missing what to change after 'of'".into());
    }
    Ok(Some((h, parse_expr(rest, env)?)))
}

/// `Ok(None)`: not an NBT line. `Err`: it is one, but wrong.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    let is = |w: &str| first.eq_ignore_ascii_case(w);
    let eff = if is("set") {
        set_effect(toks, env)?
    } else if is("delete") || is("remove") || is("clear") || is("reset") {
        del_effect(toks, &first.to_ascii_lowercase(), env)?
    } else if is("add") || is("merge") {
        merge_effect(toks, &first.to_ascii_lowercase(), env)?
    } else {
        None
    };
    Ok(eff.map(|e| Effect::Nbt(Box::new(e))))
}

fn set_effect(toks: &[Tok], env: &Env) -> Result<Option<NbtEffect>, String> {
    if !may_head(toks, 1) {
        return Ok(None);
    }
    let Some(to) = toks.iter().position(|t| is_word(t, "to")) else {
        return Ok(None);
    };
    let subject = toks.get(1..to).unwrap_or_default();
    let Some(h) = head(subject, 0) else {
        return Ok(None);
    };
    if matches!(h.kind, HeadKind::FromText | HeadKind::Keys | HeadKind::Snbt) {
        return Ok(None);
    }
    let rest = subject.get(h.after..).unwrap_or_default();
    if rest.is_empty() {
        return Err("missing what to change after 'of'".into());
    }
    let target_expr = parse_expr(rest, env)?;
    let value = value_expr(toks.get(to + 1..).unwrap_or_default(), env)?;
    Ok(Some(match h.kind {
        HeadKind::Comp(c) => {
            let p = tool_player(&target_expr)
                .ok_or_else(|| format!("use 'set {} of player's tool to ...'", c.name()))?;
            if c == ItemComp::CustomData {
                return Ok(None);
            }
            NbtEffect::SetComp {
                comp: c,
                player: p.clone(),
                value: Some(value),
            }
        }
        HeadKind::Of => NbtEffect::SetAll {
            value,
            target: target_of(&target_expr).ok_or(TARGET_HELP)?,
        },
        HeadKind::Tag(ty, p) => NbtEffect::SetTag {
            ty,
            path: path_expr(&p, env)?,
            target: target_of(&target_expr).ok_or(TARGET_HELP)?,
            value,
        },
        _ => return Ok(None),
    }))
}

fn del_effect(toks: &[Tok], verb: &str, env: &Env) -> Result<Option<NbtEffect>, String> {
    let body = toks.get(1..).unwrap_or_default();
    if !may_head(body, 0) {
        return Ok(None);
    }
    // `remove tag "x" from {_nbt}` becomes `tag "x" of {_nbt}`
    let swapped;
    let body = match body.iter().position(|t| is_word(t, "from")) {
        Some(f) if verb == "remove" => {
            swapped = [
                body.get(..f).unwrap_or_default(),
                &[Tok::Word("of".into())],
                body.get(f + 1..).unwrap_or_default(),
            ]
            .concat();
            &swapped[..]
        }
        _ => body,
    };
    let Some((h, target_expr)) = spec(body, env)? else {
        return Ok(None);
    };
    Ok(Some(match h.kind {
        HeadKind::Comp(c) => {
            if c == ItemComp::CustomData {
                return Ok(None);
            }
            NbtEffect::SetComp {
                comp: c,
                player: tool_player(&target_expr)
                    .ok_or_else(|| format!("use '{verb} {} of player's tool'", c.name()))?
                    .clone(),
                value: None,
            }
        }
        HeadKind::Of => NbtEffect::Clear(target_of(&target_expr).ok_or(TARGET_HELP)?),
        HeadKind::Tag(_, p) => NbtEffect::DelTag {
            path: path_expr(&p, env)?,
            target: target_of(&target_expr).ok_or(TARGET_HELP)?,
        },
        _ => return Ok(None),
    }))
}

/// `add <nbt> to nbt of <target>` and `merge <nbt> into <target>`.
fn merge_effect(toks: &[Tok], verb: &str, env: &Env) -> Result<Option<NbtEffect>, String> {
    let join = if verb == "add" { "to" } else { "into" };
    if verb == "add" && !toks.iter().any(|t| is_word(t, "nbt")) {
        return Ok(None);
    }
    let Some(j) = toks.iter().position(|t| is_word(t, join)) else {
        return Ok(None);
    };
    let tail = toks.get(j + 1..).unwrap_or_default();
    if verb == "add" && !tail.first().is_some_and(|t| is_word(t, "nbt")) {
        return Ok(None);
    }
    let value = parse_expr(toks.get(1..j).unwrap_or_default(), env)?;
    let target = parse_expr(tail, env)?;
    Ok(Some(NbtEffect::Merge {
        value,
        target: target_of(&target).ok_or(TARGET_HELP)?,
    }))
}

/// `<source> has [type] tag "path"` and the negations.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    if !toks.iter().any(|t| is_word(t, "tag")) {
        return None;
    }
    let h = toks
        .iter()
        .position(|t| is_word(t, "has") || is_word(t, "have"))?;
    let before = toks.get(..h)?;
    let (subject, negate) = match before.split_last() {
        Some((n, s)) if is_word(n, "not") => match s.split_last() {
            Some((d, s)) if is_word(d, "does") || is_word(d, "do") => (s, true),
            _ => return None,
        },
        Some((n, s)) if is_word(n, "doesn't") || is_word(n, "don't") => (s, true),
        _ => (before, false),
    };
    if subject.is_empty() {
        return None;
    }
    let mut spec_toks: Vec<Tok> = toks.get(h + 1..)?.to_vec();
    if spec_toks.first().is_some_and(|t| is_word(t, "no")) && !negate {
        spec_toks.remove(0);
        return cond_from(subject, spec_toks, true, env);
    }
    cond_from(subject, spec_toks, negate, env)
}

fn cond_from(subject: &[Tok], mut spec_toks: Vec<Tok>, negate: bool, env: &Env) -> Option<Cond> {
    spec_toks.push(Tok::Word("of".into()));
    let h = head(&spec_toks, 0)?;
    let HeadKind::Tag(ty, p) = h.kind else {
        return None;
    };
    if h.after != spec_toks.len() {
        return None;
    }
    let src = src_of(parse_expr(subject, env).ok()?);
    Some(Cond::Nbt(Box::new(NbtCond {
        src,
        ty,
        path: path_expr(&p, env).ok()?,
        negate,
    })))
}

// ----------------------------------------------------------------- engine

impl Engine {
    /// Read a script value as a compound: NBT as is, text as SNBT.
    fn as_compound(&mut self, v: Value) -> Result<Option<Compound>, String> {
        match v {
            Value::Nbt(c) => Ok(Some(*c)),
            Value::Text(t) => Compound::parse(&t).map(Some),
            _ => Ok(None),
        }
    }

    fn nbt_src(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<Option<Compound>, String> {
        let v = self.eval(e, ex, host)?;
        self.as_compound(v)
    }

    fn held_data(&mut self, p: &PlayerRef, host: &mut dyn Host) -> Option<Compound> {
        host.held_item(p)?;
        Some(
            host.held_component(p, ItemComp::CustomData)
                .and_then(|b| Compound::from_bytes(&b).ok())
                .unwrap_or_default(),
        )
    }

    fn load_target(
        &mut self,
        t: &NbtTarget,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Option<Compound>, String> {
        match t {
            NbtTarget::Var(v) => {
                let val = self.eval(&Expr::Var(v.clone()), ex, host)?;
                Ok(Some(self.as_compound(val)?.unwrap_or_default()))
            }
            NbtTarget::Tool(p) => match self.player(p, ex, host)? {
                Some(p) => Ok(self.held_data(&p, host)),
                None => Ok(None),
            },
        }
    }

    fn save_target(
        &mut self,
        t: &NbtTarget,
        c: Compound,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        match t {
            NbtTarget::Var(v) => {
                let k = self.key(v, ex, host)?.into_owned();
                self.store(v.local, &k, Value::Nbt(Box::new(c)), ex, host);
                Ok(())
            }
            NbtTarget::Tool(p) => {
                let Some(p) = self.player(p, ex, host)? else {
                    return Ok(());
                };
                let bytes = if c.is_empty() { None } else { Some(c.to_bytes()?) };
                if !host.set_held_component(&p, ItemComp::CustomData, bytes) {
                    return Err("this server can't change item data".into());
                }
                Ok(())
            }
        }
    }

    fn path_text(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<String, String> {
        self.text(e, ex, host)
    }

    #[inline(never)]
    pub(crate) fn nbt_expr(
        &mut self,
        n: &NbtExpr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        Ok(match n {
            NbtExpr::FromText(e) => {
                let t = self.text(e, ex, host)?;
                Value::Nbt(Box::new(Compound::parse(&t)?))
            }
            NbtExpr::OfTool(p) => match self.player(p, ex, host)? {
                Some(p) => self
                    .held_data(&p, host)
                    .map_or(Value::None, |c| Value::Nbt(Box::new(c))),
                None => Value::None,
            },
            NbtExpr::Of(e) => self
                .nbt_src(e, ex, host)?
                .map_or(Value::None, |c| Value::Nbt(Box::new(c))),
            NbtExpr::Tag { ty, path, src } => {
                let path = self.path_text(path, ex, host)?;
                match self.nbt_src(src, ex, host)? {
                    Some(c) => c.get_path(&path).map_or(Value::None, |t| tag_to_value(*ty, t)),
                    None => Value::None,
                }
            }
            NbtExpr::Keys(e) => match self.nbt_src(e, ex, host)? {
                Some(c) => Value::List(c.keys().map(|k| Value::Text(k.into())).collect()),
                None => Value::None,
            },
            NbtExpr::Snbt(e) => match self.nbt_src(e, ex, host)? {
                Some(c) => Value::Text(c.to_snbt()),
                None => Value::None,
            },
            NbtExpr::Comp(c, p) => match self.player(p, ex, host)? {
                Some(p) if host.held_item(&p).is_some() => match host.held_component(&p, *c) {
                    Some(b) => c.decode(&b),
                    None if *c == ItemComp::Unbreakable => Value::Bool(false),
                    None => Value::None,
                },
                _ => Value::None,
            },
        })
    }

    #[inline(never)]
    pub(crate) fn nbt_effect(
        &mut self,
        e: &NbtEffect,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        match e {
            NbtEffect::SetTag {
                ty,
                path,
                target,
                value,
            } => {
                let path = self.path_text(path, ex, host)?;
                let v = self.eval(value, ex, host)?;
                let tag = value_to_tag(*ty, &v)
                    .ok_or_else(|| format!("can't store {v} as that kind of NBT tag"))?;
                if let Some(mut c) = self.load_target(target, ex, host)? {
                    if !c.set_path(&path, tag) {
                        return Err(format!(
                            "can't set the NBT tag '{path}' (a part of the path is not a compound or the list type does not match)"
                        ));
                    }
                    self.save_target(target, c, ex, host)?;
                }
            }
            NbtEffect::DelTag { path, target } => {
                let path = self.path_text(path, ex, host)?;
                if let Some(mut c) = self.load_target(target, ex, host)?
                    && c.remove_path(&path)
                {
                    self.save_target(target, c, ex, host)?;
                }
            }
            NbtEffect::Merge { value, target } => {
                let v = self.eval(value, ex, host)?;
                let add = self.as_compound(v)?.ok_or("can only add NBT or NBT text")?;
                if let Some(mut c) = self.load_target(target, ex, host)? {
                    c.merge(&add);
                    self.save_target(target, c, ex, host)?;
                }
            }
            NbtEffect::SetAll { value, target } => {
                let v = self.eval(value, ex, host)?;
                let new = self
                    .as_compound(v)?
                    .ok_or("can only set NBT to NBT or NBT text")?;
                self.save_target(target, new, ex, host)?;
            }
            NbtEffect::Clear(target) => {
                if let NbtTarget::Var(v) = target {
                    let k = self.key(v, ex, host)?.into_owned();
                    self.store(v.local, &k, Value::None, ex, host);
                } else {
                    self.save_target(target, Compound::default(), ex, host)?;
                }
            }
            NbtEffect::SetComp { comp, player, value } => {
                let Some(p) = self.player(player, ex, host)? else {
                    return Ok(());
                };
                if host.held_item(&p).is_none() {
                    return Ok(());
                }
                let bytes = match value {
                    Some(v) => {
                        let v = self.eval(v, ex, host)?;
                        comp.encode(&v)?
                    }
                    None => None,
                };
                if !host.set_held_component(&p, *comp, bytes) {
                    return Err(format!("this server can't change the {} of an item", comp.name()));
                }
            }
        }
        Ok(())
    }

    #[inline(never)]
    pub(crate) fn nbt_cond(
        &mut self,
        c: &NbtCond,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<bool, String> {
        let path = self.path_text(&c.path, ex, host)?;
        let found = match self.nbt_src(&c.src, ex, host)? {
            Some(n) => n
                .get_path(&path)
                .is_some_and(|t| !matches!(tag_to_value(c.ty, t), Value::None)),
            None => false,
        };
        Ok(found != c.negate)
    }
}

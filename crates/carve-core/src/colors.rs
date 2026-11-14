//! Chat colors: `&a` and `§a` codes, `&#rrggbb`, `<#rrggbb>` and named tags
//! like `<red>` or `<bold>`. Pure text work, no game types, so the plugin
//! crate can reuse [`parse`] to build real text components.

/// Formatting that applies to one run of text.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Style {
    pub color: Option<(u8, u8, u8)>,
    pub bold: bool,
    pub italic: bool,
    pub underlined: bool,
    pub strikethrough: bool,
    pub obfuscated: bool,
}

/// A run of text with one style.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Span {
    pub text: String,
    pub style: Style,
}

const LEGACY: [(char, (u8, u8, u8)); 16] = [
    ('0', (0x00, 0x00, 0x00)),
    ('1', (0x00, 0x00, 0xAA)),
    ('2', (0x00, 0xAA, 0x00)),
    ('3', (0x00, 0xAA, 0xAA)),
    ('4', (0xAA, 0x00, 0x00)),
    ('5', (0xAA, 0x00, 0xAA)),
    ('6', (0xFF, 0xAA, 0x00)),
    ('7', (0xAA, 0xAA, 0xAA)),
    ('8', (0x55, 0x55, 0x55)),
    ('9', (0x55, 0x55, 0xFF)),
    ('a', (0x55, 0xFF, 0x55)),
    ('b', (0x55, 0xFF, 0xFF)),
    ('c', (0xFF, 0x55, 0x55)),
    ('d', (0xFF, 0x55, 0xFF)),
    ('e', (0xFF, 0xFF, 0x55)),
    ('f', (0xFF, 0xFF, 0xFF)),
];

const NAMES: [(&str, char); 18] = [
    ("black", '0'),
    ("dark_blue", '1'),
    ("dark_green", '2'),
    ("dark_aqua", '3'),
    ("dark_red", '4'),
    ("dark_purple", '5'),
    ("gold", '6'),
    ("gray", '7'),
    ("grey", '7'),
    ("dark_gray", '8'),
    ("blue", '9'),
    ("green", 'a'),
    ("aqua", 'b'),
    ("red", 'c'),
    ("light_purple", 'd'),
    ("yellow", 'e'),
    ("white", 'f'),
    ("dark_grey", '8'),
];

fn legacy_rgb(c: char) -> Option<(u8, u8, u8)> {
    let c = c.to_ascii_lowercase();
    LEGACY.iter().find(|(k, _)| *k == c).map(|(_, v)| *v)
}

#[derive(Clone, Copy)]
enum Flag {
    Bold,
    Italic,
    Underlined,
    Strike,
    Obfuscated,
}

impl Style {
    fn set(&mut self, f: Flag, on: bool) {
        match f {
            Flag::Bold => self.bold = on,
            Flag::Italic => self.italic = on,
            Flag::Underlined => self.underlined = on,
            Flag::Strike => self.strikethrough = on,
            Flag::Obfuscated => self.obfuscated = on,
        }
    }
}

fn flag_of_code(c: char) -> Option<Flag> {
    Some(match c.to_ascii_lowercase() {
        'l' => Flag::Bold,
        'o' => Flag::Italic,
        'n' => Flag::Underlined,
        'm' => Flag::Strike,
        'k' => Flag::Obfuscated,
        _ => return None,
    })
}

fn flag_of_name(n: &str) -> Option<Flag> {
    Some(match n {
        "bold" | "b" => Flag::Bold,
        "italic" | "i" | "em" => Flag::Italic,
        "underlined" | "underline" | "u" => Flag::Underlined,
        "strikethrough" | "st" => Flag::Strike,
        "obfuscated" | "obf" | "magic" => Flag::Obfuscated,
        _ => return None,
    })
}

fn hex6(s: &[char]) -> Option<(u8, u8, u8)> {
    if s.len() != 6 {
        return None;
    }
    let mut v = [0u8; 3];
    for (i, pair) in s.chunks(2).enumerate() {
        let hi = pair.first()?.to_digit(16)?;
        let lo = pair.get(1)?.to_digit(16)?;
        *v.get_mut(i)? = u8::try_from(hi * 16 + lo).ok()?;
    }
    Some((v[0], v[1], v[2]))
}

/// What a code or tag at some position does to the style.
enum Change {
    Reset,
    Color((u8, u8, u8)),
    Flag(Flag, bool),
    ClearColor,
}

/// Try to read a `<...>` tag at `i` (which points at `<`). Returns the change
/// and the index after the `>`.
fn tag_at(c: &[char], i: usize) -> Option<(Change, usize)> {
    let mut j = i + 1;
    let closing = c.get(j) == Some(&'/');
    if closing {
        j += 1;
    }
    let start = j;
    while j < c.len() && j - start <= 14 && c[j] != '>' && c[j] != '<' {
        j += 1;
    }
    if c.get(j) != Some(&'>') {
        return None;
    }
    let body: String = c[start..j].iter().collect::<String>().to_ascii_lowercase();
    let end = j + 1;
    if let Some(hex) = body.strip_prefix('#') {
        let hex: Vec<char> = hex.chars().collect();
        let rgb = hex6(&hex)?;
        return Some((if closing { Change::ClearColor } else { Change::Color(rgb) }, end));
    }
    if body == "reset" || body == "r" {
        return Some((Change::Reset, end));
    }
    if let Some(f) = flag_of_name(&body) {
        return Some((Change::Flag(f, !closing), end));
    }
    let (_, code) = NAMES.iter().find(|(n, _)| *n == body)?;
    let rgb = legacy_rgb(*code)?;
    Some((if closing { Change::ClearColor } else { Change::Color(rgb) }, end))
}

/// Try to read a `&x` / `§x` code at `i` (points at the symbol).
fn code_at(c: &[char], i: usize) -> Option<(Change, usize)> {
    let k = *c.get(i + 1)?;
    if k == '#'
        && let Some(rgb) = c.get(i + 2..i + 8).and_then(hex6)
    {
        return Some((Change::Color(rgb), i + 8));
    }
    if k.eq_ignore_ascii_case(&'x') {
        // `&x&r&r&g&g&b&b`: the long hex form.
        let mut digits = Vec::with_capacity(6);
        for n in 0..6 {
            let at = i + 2 + n * 2;
            if !matches!(c.get(at), Some('&' | '§')) {
                return None;
            }
            digits.push(*c.get(at + 1)?);
        }
        return Some((Change::Color(hex6(&digits)?), i + 14));
    }
    if let Some(rgb) = legacy_rgb(k) {
        return Some((Change::Color(rgb), i + 2));
    }
    if k.eq_ignore_ascii_case(&'r') {
        return Some((Change::Reset, i + 2));
    }
    flag_of_code(k).map(|f| (Change::Flag(f, true), i + 2))
}

/// Split text into styled runs. Anything that is not a valid code or tag
/// stays as plain text.
pub fn parse(s: &str) -> Vec<Span> {
    let c: Vec<char> = s.chars().collect();
    let mut out: Vec<Span> = Vec::new();
    let mut style = Style::default();
    let mut buf = String::new();
    let mut i = 0;
    while i < c.len() {
        let hit = match c[i] {
            '&' | '§' => code_at(&c, i).map(|(ch, n)| (ch, n, true)),
            '<' => tag_at(&c, i).map(|(ch, n)| (ch, n, false)),
            _ => None,
        };
        let Some((change, next, legacy)) = hit else {
            buf.push(c[i]);
            i += 1;
            continue;
        };
        if !buf.is_empty() {
            out.push(Span { text: std::mem::take(&mut buf), style });
        }
        match change {
            Change::Reset => style = Style::default(),
            // A legacy color code also clears formatting, like vanilla.
            Change::Color(rgb) => {
                if legacy {
                    style = Style::default();
                }
                style.color = Some(rgb);
            }
            Change::ClearColor => style.color = None,
            Change::Flag(f, on) => style.set(f, on),
        }
        i = next;
    }
    if !buf.is_empty() {
        out.push(Span { text: buf, style });
    }
    out
}

/// `uncolored`: the text with every code and tag removed.
pub fn strip(s: &str) -> String {
    if !s.contains(['&', '§', '<']) {
        return s.to_string();
    }
    parse(s).into_iter().map(|sp| sp.text).collect()
}

/// `colored`: codes and tags turned into `§` codes (hex as `§x§r§r§g§g§b§b`).
pub fn colorize(s: &str) -> String {
    if !s.contains(['&', '§', '<']) {
        return s.to_string();
    }
    let mut out = String::new();
    for (n, sp) in parse(s).iter().enumerate() {
        let st = sp.style;
        if st == Style::default() {
            if n > 0 {
                out.push_str("§r");
            }
        } else {
            if n > 0 {
                out.push_str("§r");
            }
            if let Some((r, g, b)) = st.color {
                match LEGACY.iter().find(|(_, v)| *v == (r, g, b)) {
                    Some((k, _)) => {
                        out.push('§');
                        out.push(*k);
                    }
                    None => {
                        out.push_str("§x");
                        for ch in format!("{r:02x}{g:02x}{b:02x}").chars() {
                            out.push('§');
                            out.push(ch);
                        }
                    }
                }
            }
            for (on, k) in [
                (st.bold, 'l'),
                (st.italic, 'o'),
                (st.underlined, 'n'),
                (st.strikethrough, 'm'),
                (st.obfuscated, 'k'),
            ] {
                if on {
                    out.push('§');
                    out.push(k);
                }
            }
        }
        out.push_str(&sp.text);
    }
    out
}

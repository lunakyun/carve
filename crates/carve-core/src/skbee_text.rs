//! SkBee-style text components.
//!
//! A component is a text value written in MiniMessage markup, so it can be
//! stored in variables, joined, and sent like any other text. The plugin turns
//! the markup into real text components with [`parse`]. Supported tags:
//! `<hover:show_text:'..'>`, `<hover:show_item:id[:count]>`,
//! `<hover:show_entity:type:uuid[:name]>`, `<click:ACTION:value>`,
//! `<insertion:text>`, `<key:key.jump>`, `<lang:key:'arg'..>`,
//! `<gradient:red:blue>`, `<rainbow>`, `<newline>`, plus every color and
//! style tag and `&` code from `colors`. Use `\<` for a literal `<`.

use crate::colors::{self, Change, Style};
use crate::compile::{EffKind, Effect};
use crate::expr::{Env, Expr, VarRef, parse_expr, parse_varref};
use crate::lexer::Tok;
use crate::value::Value;

/// Registered ahead of the plain `send` patterns.
pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    (
        "send action bar [text] (component|components) %text% to %player%",
        EffKind::ActionBar,
    ),
    ("send [text] (component|components) %text% [to %player%]", EffKind::Send),
    ("broadcast [text] (component|components) %text%", EffKind::Broadcast),
];

#[derive(Debug, Clone, PartialEq)]
pub enum Click {
    Url(String),
    Run(String),
    Suggest(String),
    Copy(String),
    Page(u32),
    File(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Hover {
    Text(Vec<Part>),
    /// Item data as an SNBT string.
    Item(String),
    Entity {
        kind: String,
        id: String,
        name: Option<Vec<Part>>,
    },
}

#[derive(Debug, Clone, PartialEq)]
pub enum Kind {
    Text(String),
    Key(String),
    Lang(String, Vec<Vec<Part>>),
}

/// One piece of a component: content plus everything that applies to it.
#[derive(Debug, Clone, PartialEq)]
pub struct Part {
    pub kind: Kind,
    pub style: Style,
    pub click: Option<Click>,
    pub hover: Option<Box<Hover>>,
    pub insertion: Option<String>,
}

/// Does this text need the rich parser (anything beyond colors)?
pub fn is_rich(s: &str) -> bool {
    if !s.contains(['<', '\\']) {
        return false;
    }
    if s.contains("\\<") {
        return true;
    }
    let l = s.to_ascii_lowercase();
    ["<hover", "<click", "<insertion", "<key:", "<lang:", "<tr:", "<translate:", "<gradient", "<rainbow", "<newline", "<br>"]
        .iter()
        .any(|t| l.contains(t))
}

#[derive(Clone)]
enum Fx {
    Gradient { stops: Vec<(u8, u8, u8)>, phase: f64 },
    Rainbow { reverse: bool, phase: f64 },
}

impl Fx {
    fn is_rainbow(&self) -> bool {
        matches!(self, Fx::Rainbow { .. })
    }

    fn at(&self, i: usize, n: usize) -> (u8, u8, u8) {
        let t = if n <= 1 { 0.0 } else { i as f64 / (n - 1) as f64 };
        match self {
            Fx::Gradient { stops, phase } => {
                let mut x = t + phase;
                if x < 0.0 {
                    x += 1.0;
                } else if x > 1.0 {
                    x -= 1.0;
                }
                gradient_at(stops, x.clamp(0.0, 1.0))
            }
            Fx::Rainbow { reverse, phase } => {
                let base = if n == 0 { 0.0 } else { i as f64 / n as f64 };
                let base = if *reverse { 1.0 - base } else { base };
                hsv((base + phase).rem_euclid(1.0))
            }
        }
    }
}

fn gradient_at(stops: &[(u8, u8, u8)], t: f64) -> (u8, u8, u8) {
    let Some(&first) = stops.first() else {
        return (255, 255, 255);
    };
    if stops.len() == 1 {
        return first;
    }
    let pos = t * (stops.len() - 1) as f64;
    let seg = (pos.floor() as usize).min(stops.len() - 2);
    let f = pos - seg as f64;
    let (a, b) = (stops[seg], stops[seg + 1]);
    let mix = |x: u8, y: u8| (f64::from(x) + (f64::from(y) - f64::from(x)) * f).round() as u8;
    (mix(a.0, b.0), mix(a.1, b.1), mix(a.2, b.2))
}

fn hsv(h: f64) -> (u8, u8, u8) {
    let h6 = h * 6.0;
    let f = h6 - h6.floor();
    let q = (255.0 * (1.0 - f)).round() as u8;
    let t = (255.0 * f).round() as u8;
    match (h6.floor() as i32).rem_euclid(6) {
        0 => (255, t, 0),
        1 => (q, 255, 0),
        2 => (0, 255, t),
        3 => (0, q, 255),
        4 => (t, 0, 255),
        _ => (255, 0, q),
    }
}

/// Unquote `'a \' b'` or `"x"`; anything else is returned as is.
fn unquote(raw: &str) -> String {
    let raw = raw.trim();
    let first = raw.chars().next();
    if raw.len() < 2 || !matches!(first, Some('\'' | '"')) || !raw.ends_with(first.unwrap_or('\'')) {
        return raw.to_string();
    }
    let inner = &raw[1..raw.len() - 1];
    let mut out = String::new();
    let mut it = inner.chars();
    while let Some(c) = it.next() {
        if c != '\\' {
            out.push(c);
            continue;
        }
        match it.next() {
            Some(n @ ('\\' | '\'' | '"')) => out.push(n),
            Some(n) => {
                out.push('\\');
                out.push(n);
            }
            None => out.push('\\'),
        }
    }
    out
}

/// Split at `:` outside quotes.
fn split_args(body: &str) -> Vec<String> {
    let mut out = Vec::new();
    let mut cur = String::new();
    let mut quote: Option<char> = None;
    let mut esc = false;
    for c in body.chars() {
        if let Some(q) = quote {
            cur.push(c);
            if esc {
                esc = false;
            } else if c == '\\' {
                esc = true;
            } else if c == q {
                quote = None;
            }
        } else if (c == '\'' || c == '"') && cur.is_empty() {
            quote = Some(c);
            cur.push(c);
        } else if c == ':' {
            out.push(std::mem::take(&mut cur));
        } else {
            cur.push(c);
        }
    }
    out.push(cur);
    out
}

/// The `<...>` at `i`: body text and index after `>`. Quotes may hold `<` and `>`.
fn raw_tag(c: &[char], i: usize) -> Option<(String, usize)> {
    let mut j = i + 1;
    let mut quote: Option<char> = None;
    let mut esc = false;
    while j < c.len() {
        let ch = c[j];
        if let Some(q) = quote {
            if esc {
                esc = false;
            } else if ch == '\\' {
                esc = true;
            } else if ch == q {
                quote = None;
            }
        } else if (ch == '\'' || ch == '"') && c[j - 1] == ':' {
            quote = Some(ch);
        } else if ch == '>' {
            return Some((c[i + 1..j].iter().collect(), j + 1));
        } else if ch == '<' {
            return None;
        }
        j += 1;
    }
    None
}

#[derive(Default)]
struct State {
    style: Style,
    clicks: Vec<Click>,
    hovers: Vec<Hover>,
    insertions: Vec<String>,
    fx: Vec<usize>,
    specs: Vec<Fx>,
    /// (part index, fx id) for parts that wait for per-letter colors.
    out: Vec<(Part, Option<usize>)>,
}

impl State {
    fn push(&mut self, kind: Kind) {
        let part = Part {
            kind,
            style: self.style,
            click: self.clicks.last().cloned(),
            hover: self.hovers.last().cloned().map(Box::new),
            insertion: self.insertions.last().cloned(),
        };
        self.out.push((part, self.fx.last().copied()));
    }

    fn close_fx(&mut self, rainbow: bool) {
        if let Some(p) = self.fx.iter().rposition(|id| self.specs[*id].is_rainbow() == rainbow) {
            self.fx.remove(p);
        }
    }

    fn reset(&mut self) {
        self.style = Style::default();
        self.clicks.clear();
        self.hovers.clear();
        self.insertions.clear();
        self.fx.clear();
    }
}

fn click_of(action: &str, value: &str) -> Option<Click> {
    let v = value.to_string();
    Some(match action {
        "open_url" => Click::Url(v),
        "run_command" => Click::Run(v),
        "suggest_command" => Click::Suggest(v),
        "copy_to_clipboard" => Click::Copy(v),
        "open_file" => Click::File(v),
        "change_page" => Click::Page(value.trim().parse().ok()?),
        _ => return None,
    })
}

fn item_snbt(raw: &str) -> Option<String> {
    let args = split_args(raw);
    let id = unquote(args.first()?);
    if id.is_empty() {
        return None;
    }
    let id = if id.contains(':') { id } else { format!("minecraft:{id}") };
    let count: u32 = args.get(1).and_then(|c| c.trim().parse().ok()).unwrap_or(1);
    Some(format!("{{id:\"{id}\",count:{count}}}"))
}

fn hover_of(action: &str, raw: &str) -> Option<Hover> {
    match action {
        "show_text" => Some(Hover::Text(parse(&unquote(raw)))),
        "show_item" => item_snbt(raw).map(Hover::Item),
        "show_entity" => {
            let args = split_args(raw);
            Some(Hover::Entity {
                kind: unquote(args.first()?),
                id: unquote(args.get(1)?),
                name: args.get(2).map(|n| parse(&unquote(n))),
            })
        }
        _ => None,
    }
}

/// Handle one `<...>` tag that this module owns. `false`: not ours.
fn rich_tag(st: &mut State, body: &str) -> bool {
    let closing = body.starts_with('/');
    let body = body.strip_prefix('/').unwrap_or(body);
    let (name, rest) = body.split_once(':').unwrap_or((body, ""));
    let name = name.trim().to_ascii_lowercase();
    if closing {
        match name.as_str() {
            "click" => {
                st.clicks.pop();
            }
            "hover" => {
                st.hovers.pop();
            }
            "insertion" => {
                st.insertions.pop();
            }
            "gradient" => st.close_fx(false),
            "rainbow" => st.close_fx(true),
            "key" | "lang" | "tr" | "translate" | "newline" | "br" => {}
            _ => return false,
        }
        return true;
    }
    match name.as_str() {
        "click" => {
            let (action, raw) = rest.split_once(':').unwrap_or((rest, ""));
            match click_of(&action.trim().to_ascii_lowercase(), &unquote(raw)) {
                Some(c) => st.clicks.push(c),
                None => return false,
            }
        }
        "hover" => {
            let (action, raw) = rest.split_once(':').unwrap_or((rest, ""));
            match hover_of(&action.trim().to_ascii_lowercase(), raw) {
                Some(h) => st.hovers.push(h),
                None => return false,
            }
        }
        "insertion" => st.insertions.push(unquote(rest)),
        "key" if !rest.is_empty() => st.push(Kind::Key(unquote(rest))),
        "lang" | "tr" | "translate" if !rest.is_empty() => {
            let mut args = split_args(rest).into_iter();
            let key = unquote(&args.next().unwrap_or_default());
            st.push(Kind::Lang(key, args.map(|a| parse(&unquote(&a))).collect()));
        }
        "gradient" => {
            let mut stops = Vec::new();
            let mut phase = 0.0;
            if !rest.is_empty() {
                for a in split_args(rest) {
                    if let Some(c) = colors::color_named(&unquote(&a)) {
                        stops.push(c);
                    } else if let Ok(p) = a.trim().parse::<f64>() {
                        phase = p.clamp(-1.0, 1.0);
                    } else {
                        return false;
                    }
                }
            }
            if stops.is_empty() {
                stops = vec![(255, 255, 255), (0, 0, 0)];
            }
            st.specs.push(Fx::Gradient { stops, phase });
            st.fx.push(st.specs.len() - 1);
        }
        "rainbow" => {
            let mut reverse = false;
            let mut phase = 0.0;
            for a in split_args(rest).iter().filter(|a| !a.is_empty()) {
                if a.trim() == "!" {
                    reverse = true;
                } else if let Ok(p) = a.trim().parse::<f64>() {
                    phase = p / 10.0;
                } else {
                    return false;
                }
            }
            st.specs.push(Fx::Rainbow { reverse, phase });
            st.fx.push(st.specs.len() - 1);
        }
        "newline" | "br" if rest.is_empty() => st.push(Kind::Text("\n".into())),
        _ => return false,
    }
    true
}

/// Turn component markup into parts.
pub fn parse(s: &str) -> Vec<Part> {
    let c: Vec<char> = s.chars().collect();
    let mut st = State::default();
    let mut buf = String::new();
    let mut i = 0;
    macro_rules! flush {
        () => {
            if !buf.is_empty() {
                st.push(Kind::Text(std::mem::take(&mut buf)));
            }
        };
    }
    while i < c.len() {
        let ch = c[i];
        if ch == '\\' && matches!(c.get(i + 1), Some('<' | '\\' | '&' | '§')) {
            buf.push(c[i + 1]);
            i += 2;
            continue;
        }
        if ch == '<'
            && let Some((body, next)) = raw_tag(&c, i)
        {
            flush!();
            if rich_tag(&mut st, &body) {
                i = next;
                continue;
            }
        }
        if let Some((change, next, legacy)) = colors::next_change(&c, i) {
            flush!();
            if matches!(change, Change::Reset) {
                st.reset();
            } else {
                colors::apply_change(&mut st.style, change, legacy);
            }
            i = next;
            continue;
        }
        buf.push(ch);
        i += 1;
    }
    flush!();
    finish(st)
}

/// Give gradient and rainbow text its per-letter colors.
fn finish(st: State) -> Vec<Part> {
    let mut totals = vec![0usize; st.specs.len()];
    for (p, fx) in &st.out {
        if let (Some(id), Kind::Text(t)) = (fx, &p.kind) {
            totals[*id] += t.chars().count();
        }
    }
    let mut seen = vec![0usize; st.specs.len()];
    let mut out = Vec::with_capacity(st.out.len());
    for (p, fx) in st.out {
        match (fx, &p.kind) {
            (Some(id), Kind::Text(t)) => {
                for ch in t.chars() {
                    let mut q = p.clone();
                    q.kind = Kind::Text(ch.to_string());
                    q.style.color = Some(st.specs[id].at(seen[id], totals[id]));
                    seen[id] += 1;
                    out.push(q);
                }
            }
            _ => out.push(p),
        }
    }
    out
}

fn part_plain(p: &Part, out: &mut String) {
    match &p.kind {
        Kind::Text(t) => out.push_str(t),
        Kind::Key(k) | Kind::Lang(k, _) => out.push_str(k),
    }
}

/// The text with every tag and code removed.
pub fn plain(s: &str) -> String {
    let mut out = String::new();
    for p in parse(s) {
        part_plain(&p, &mut out);
    }
    out
}

/// Escape plain text so it is shown as typed (`<` and `\` lose their meaning).
/// `&` color codes stay active, like SkBee's `text component from`.
pub fn escape(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for c in s.chars() {
        if matches!(c, '<' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out
}

fn quote(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('\'');
    for c in s.chars() {
        if matches!(c, '\'' | '\\') {
            out.push('\\');
        }
        out.push(c);
    }
    out.push('\'');
    out
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ClickKind {
    OpenUrl,
    RunCommand,
    SuggestCommand,
    CopyToClipboard,
    ChangePage,
}

impl ClickKind {
    fn action(self) -> &'static str {
        match self {
            ClickKind::OpenUrl => "open_url",
            ClickKind::RunCommand => "run_command",
            ClickKind::SuggestCommand => "suggest_command",
            ClickKind::CopyToClipboard => "copy_to_clipboard",
            ClickKind::ChangePage => "change_page",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompFn {
    /// `text component from x`: [text]
    Plain,
    /// `mini message from x`: [text]
    Mini,
    /// `hover event showing x`: [text or item]
    HoverEvent,
    /// `click event to run command x`: [text]
    ClickEvent(ClickKind),
    /// Wrap a component in an event: [component, event]
    Wrap,
    /// `set insertion of {c} to x`: [component, text]
    Insertion,
    /// `translate component k with a, b`: [key, args...]
    Translate,
    /// `keybind component k`: [key]
    Keybind,
    /// `merged components of ...`: [parts...]
    Merge,
    /// `plain text of x`: [component]
    PlainText,
}

/// Words a component expression can start with (cheap filter for the parser).
pub fn starts_comp(w: &str) -> bool {
    matches!(
        w,
        "a" | "an"
            | "new"
            | "text"
            | "mini"
            | "minimessage"
            | "mm"
            | "hover"
            | "click"
            | "translate"
            | "translatable"
            | "keybind"
            | "merged"
            | "merge"
            | "plain"
    )
}

fn is_event_tag(s: &str) -> Option<&'static str> {
    if s.starts_with("<hover:") && s.ends_with('>') {
        Some("hover")
    } else if s.starts_with("<click:") && s.ends_with('>') {
        Some("click")
    } else {
        None
    }
}

pub fn apply(f: CompFn, args: &[Value]) -> Value {
    if args.iter().any(|a| matches!(a, Value::None)) {
        return Value::None;
    }
    let txt = |i: usize| args.get(i).map(ToString::to_string).unwrap_or_default();
    Value::Text(match f {
        CompFn::Plain => escape(&txt(0)),
        CompFn::Mini => txt(0),
        CompFn::HoverEvent => match args.first() {
            Some(Value::Item { id, count }) => format!("<hover:show_item:{id}:{count}>"),
            _ => format!("<hover:show_text:{}>", quote(&txt(0))),
        },
        CompFn::ClickEvent(k) => format!("<click:{}:{}>", k.action(), quote(&txt(0))),
        CompFn::Wrap => {
            let ev = txt(1);
            match is_event_tag(&ev) {
                Some(name) => format!("{ev}{}</{name}>", txt(0)),
                None => return Value::None,
            }
        }
        CompFn::Insertion => format!("<insertion:{}>{}</insertion>", quote(&txt(1)), txt(0)),
        CompFn::Translate => {
            let mut s = format!("<lang:{}", quote(&txt(0)));
            for a in args.iter().skip(1) {
                match a {
                    Value::List(items) => {
                        for i in items {
                            s.push(':');
                            s.push_str(&quote(&i.to_string()));
                        }
                    }
                    other => {
                        s.push(':');
                        s.push_str(&quote(&other.to_string()));
                    }
                }
            }
            s.push('>');
            s
        }
        CompFn::Keybind => format!("<key:{}>", txt(0).trim()),
        CompFn::Merge => {
            let mut s = String::new();
            for a in args {
                match a {
                    Value::List(items) => items.iter().for_each(|i| s.push_str(&i.to_string())),
                    other => s.push_str(&other.to_string()),
                }
            }
            s
        }
        CompFn::PlainText => plain(&txt(0)),
    })
}

// ---- parsing the `set ... of {c} to ...` effects ----

fn word(t: Option<&Tok>, w: &str) -> bool {
    t.is_some_and(|t| t.is_lit(w))
}

/// `set hover event of {c} to E`, `set click event of {c} to E`,
/// `set insertion of {c} to "text"`. They rewrite the component variable.
/// `Ok(None)`: not one of these lines.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    if !word(toks.first(), "set") {
        return Ok(None);
    }
    let (f, skip) = if (word(toks.get(1), "hover") || word(toks.get(1), "click")) && word(toks.get(2), "event") {
        (CompFn::Wrap, 3)
    } else if word(toks.get(1), "insertion") {
        (CompFn::Insertion, 2)
    } else {
        return Ok(None);
    };
    let usage = "use 'set hover event of {comp} to hover event showing \"text\"'";
    if !word(toks.get(skip), "of") {
        return Err(usage.into());
    }
    let Some(Tok::Var(raw)) = toks.get(skip + 1) else {
        return Err("the component must be a variable like {_comp}".into());
    };
    let var: VarRef = parse_varref(raw, env)?;
    if var.is_list() {
        return Err("the component must be a single variable, not a list".into());
    }
    if !word(toks.get(skip + 2), "to") {
        return Err(usage.into());
    }
    let value = parse_expr(toks.get(skip + 3..).unwrap_or_default(), env)?;
    Ok(Some(Effect::Set(var.clone(), Expr::Comp(f, vec![Expr::Var(var), value]))))
}

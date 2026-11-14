//! SkBee-style particle options and display entities.
//!
//! Particles extend the `play N <particle> particles at <place>` line from
//! `world_effects` with more `with ...` options. Display entities are the
//! `spawn text display at <place> with ...` family. Both are parsed by hand
//! (free-form words), like the rest of the world effects, and run through
//! `#[inline(never)]` engine methods so the hot paths stay small.

use crate::colors;
use crate::engine::{Engine, Exec};
use crate::expr::{Env, Expr, Part, Ty, parse_expr};
use crate::host::{Host, LogLevel};
use crate::lexer::Tok;
use crate::world_effects::FxPos;

// ---------- particles ----------

/// Particles that need extra data on the wire (a color, a block, an item).
pub fn needs_data(name: &str) -> bool {
    matches!(
        name,
        "dust"
            | "dust_color_transition"
            | "block"
            | "block_marker"
            | "falling_dust"
            | "item"
            | "entity_effect"
            | "tinted_leaves"
            | "sculk_charge"
            | "shriek"
            | "vibration"
            | "trail"
    )
}

/// Extra data a particle can carry.
#[derive(Debug, Clone, PartialEq, Default)]
pub enum ParticleData {
    #[default]
    None,
    /// `dust`: color and size.
    Dust { color: [u8; 3], size: f32 },
    /// `dust_color_transition`: from, to and size.
    DustTransition {
        from: [u8; 3],
        to: [u8; 3],
        size: f32,
    },
    /// `block`, `block_marker`, `falling_dust`: a block id like `stone`.
    Block(String),
    /// `item`: an item id like `diamond`.
    Item(String),
    /// `entity_effect`, `tinted_leaves`: a color.
    Color([u8; 3]),
}

/// A color: text (`"red"`, `"#ff8000"`) or `rgb r, g, b`.
#[derive(Debug, Clone, PartialEq)]
pub enum ColorSrc {
    Text(Expr),
    Rgb(Box<[Expr; 3]>),
}

/// Options after `with` on a particle line. Compile-time form.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct ParticleOpts {
    /// `with offset 1, 0.5, 1`: a different spread on each axis.
    pub offset: Option<Box<[Expr; 3]>>,
    pub force: bool,
    pub color: Option<ColorSrc>,
    pub to_color: Option<ColorSrc>,
    pub size: Option<Expr>,
    pub block: Option<Expr>,
    pub item: Option<Expr>,
}

// ---------- parsing helpers ----------

fn is(t: &Tok, lit: &str) -> bool {
    t.is_lit(lit)
}

fn word(t: &Tok) -> Option<String> {
    match t {
        Tok::Word(w) => Some(w.to_ascii_lowercase()),
        _ => None,
    }
}

fn expr_of(toks: &[Tok], env: &Env, want: &[Ty]) -> Option<Expr> {
    let e = parse_expr(toks, env).ok()?;
    let got = e.ty(env);
    (got == Ty::Object || want.contains(&got)).then_some(e)
}

fn number(toks: &[Tok], env: &Env) -> Option<Expr> {
    expr_of(toks, env, &[Ty::Number])
}

fn lit_text(s: &str) -> Expr {
    Expr::Text(vec![Part::Lit(s.to_string())])
}

/// Split on top-level commas.
fn commas(toks: &[Tok]) -> Vec<&[Tok]> {
    toks.split(|t| is(t, ",")).collect()
}

/// `a, b, c` as three numbers.
fn triple(toks: &[Tok], env: &Env) -> Option<Box<[Expr; 3]>> {
    let parts = commas(toks);
    let [a, b, c] = parts.as_slice() else {
        return None;
    };
    Some(Box::new([number(a, env)?, number(b, env)?, number(c, env)?]))
}

/// `a` (used for all three) or `a, b, c`.
fn one_or_three(toks: &[Tok], env: &Env) -> Option<Box<[Expr; 3]>> {
    if toks.iter().any(|t| is(t, ",")) {
        return triple(toks, env);
    }
    let n = number(toks, env)?;
    Some(Box::new([n.clone(), n.clone(), n]))
}

fn color_src(toks: &[Tok], env: &Env) -> Option<ColorSrc> {
    let (first, rest) = toks.split_first()?;
    if is(first, "rgb") {
        return Some(ColorSrc::Rgb(triple(rest, env)?));
    }
    // A bare word like `red` or `dark_red`.
    if let [Tok::Word(w)] = toks {
        return Some(ColorSrc::Text(lit_text(&w.to_ascii_lowercase())));
    }
    expr_of(toks, env, &[Ty::Text]).map(ColorSrc::Text)
}

/// Words joined with `_` (`dark red` to `dark_red`), lowercase.
fn joined(toks: &[Tok]) -> Option<String> {
    let mut out = String::new();
    for t in toks {
        if !out.is_empty() {
            out.push('_');
        }
        out.push_str(&word(t)?);
    }
    (!out.is_empty()).then_some(out)
}

/// An id given as plain words (`diamond sword`) or as a text expression.
fn id_src(toks: &[Tok], env: &Env) -> Option<Expr> {
    if let Some(name) = joined(toks) {
        return Some(lit_text(&name));
    }
    expr_of(toks, env, &[Ty::Text, Ty::Item])
}

/// One `with ...` segment of a particle line. `Ok(false)`: not one of ours.
pub fn particle_opt(seg: &[Tok], env: &Env, o: &mut ParticleOpts) -> Result<bool, String> {
    let Some((kind, val)) = seg.split_first() else {
        return Ok(false);
    };
    let Some(kind) = word(kind) else {
        return Ok(false);
    };
    match kind.as_str() {
        "force" | "forced" if val.is_empty() => o.force = true,
        "offset" | "spread" => match triple(val, env) {
            Some(t) => o.offset = Some(t),
            None => return Ok(false),
        },
        "color" | "colour" => {
            o.color = Some(color_src(val, env).ok_or("particle color must be a color name, #hex or rgb r, g, b")?);
        }
        "to" => {
            let Some((c, rest)) = val.split_first() else {
                return Ok(false);
            };
            if !(is(c, "color") || is(c, "colour")) {
                return Ok(false);
            }
            o.to_color = Some(color_src(rest, env).ok_or("particle 'to color' must be a color name, #hex or rgb r, g, b")?);
        }
        "size" | "scale" => match number(val, env) {
            Some(n) => o.size = Some(n),
            None => return Ok(false),
        },
        "block" => o.block = Some(id_src(val, env).ok_or("particle block must be a block id")?),
        "item" => o.item = Some(id_src(val, env).ok_or("particle item must be an item id")?),
        _ => return Ok(false),
    }
    Ok(true)
}

// ---------- particles at run time ----------

const DEFAULT_DUST: [u8; 3] = [255, 0, 0];

fn rgb_of_text(s: &str) -> Result<[u8; 3], String> {
    let mut t = s.trim().to_ascii_lowercase().replace(' ', "_");
    if t.len() == 6 && t.chars().all(|c| c.is_ascii_hexdigit()) {
        t.insert(0, '#');
    }
    colors::parse(&format!("<{t}>x"))
        .first()
        .and_then(|sp| sp.style.color)
        .map(|(r, g, b)| [r, g, b])
        .ok_or_else(|| format!("unknown color '{s}'"))
}

fn clamp_byte(n: f64) -> u8 {
    if n.is_finite() { n.clamp(0.0, 255.0) as u8 } else { 0 }
}

impl Engine {
    fn rgb(&mut self, c: &ColorSrc, ex: &mut Exec, host: &mut dyn Host) -> Result<[u8; 3], String> {
        match c {
            ColorSrc::Text(e) => {
                let s = self.text(e, ex, host)?;
                rgb_of_text(&s)
            }
            ColorSrc::Rgb(parts) => {
                let r = self.num(&parts[0], ex, host)?;
                let g = self.num(&parts[1], ex, host)?;
                let b = self.num(&parts[2], ex, host)?;
                Ok([clamp_byte(r), clamp_byte(g), clamp_byte(b)])
            }
        }
    }

    fn id_text(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<String, String> {
        let s = self.text(e, ex, host)?;
        let id = s.trim().to_ascii_lowercase().replace(' ', "_");
        let id = id.strip_prefix("minecraft:").unwrap_or(&id).to_string();
        if id.is_empty() || !id.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_') {
            return Err(format!("'{s}' is not a valid id"));
        }
        Ok(id)
    }

    /// Turn the options into the data this particle carries.
    #[inline(never)]
    pub(crate) fn particle_data(
        &mut self,
        name: &str,
        o: &ParticleOpts,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<ParticleData, String> {
        let size = match &o.size {
            Some(e) => (self.num(e, ex, host)?.clamp(0.01, 4.0)) as f32,
            None => 1.0,
        };
        let color = match &o.color {
            Some(c) => Some(self.rgb(c, ex, host)?),
            None => None,
        };
        Ok(match name {
            "dust" => ParticleData::Dust {
                color: color.unwrap_or(DEFAULT_DUST),
                size,
            },
            "dust_color_transition" => {
                let from = color.unwrap_or(DEFAULT_DUST);
                let to = match &o.to_color {
                    Some(c) => self.rgb(c, ex, host)?,
                    None => from,
                };
                ParticleData::DustTransition { from, to, size }
            }
            "block" | "block_marker" | "falling_dust" => match &o.block {
                Some(b) => ParticleData::Block(self.id_text(b, ex, host)?),
                None => ParticleData::Block("stone".into()),
            },
            "item" => match &o.item {
                Some(i) => ParticleData::Item(self.id_text(i, ex, host)?),
                None => ParticleData::Item("stone".into()),
            },
            "entity_effect" | "tinted_leaves" => ParticleData::Color(color.unwrap_or([255, 255, 255])),
            _ => ParticleData::None,
        })
    }

    /// Per-axis offset, clamped like `spread`.
    #[inline(never)]
    pub(crate) fn particle_offset(
        &mut self,
        o: &ParticleOpts,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Option<[f64; 3]>, String> {
        let Some(parts) = &o.offset else {
            return Ok(None);
        };
        let mut out = [0.0; 3];
        for (slot, e) in out.iter_mut().zip(parts.iter()) {
            let n = self.num(e, ex, host)?;
            *slot = if n.is_finite() { n.clamp(0.0, 64.0) } else { 0.0 };
        }
        Ok(Some(out))
    }
}

/// The default host cannot send particle data. Say so, once per call.
pub fn warn_no_data(host: &mut dyn Host, name: &str) {
    host.log(
        LogLevel::Warn,
        &format!("the '{name}' particle needs color or block data, which this server cannot send; skipped"),
    );
}

// ---------- display entities ----------

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DisplayKind {
    Text,
    Block,
    Item,
}

impl DisplayKind {
    pub fn entity_id(self) -> &'static str {
        match self {
            DisplayKind::Text => "text_display",
            DisplayKind::Block => "block_display",
            DisplayKind::Item => "item_display",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum DisplayOpt {
    Text(Expr),
    Billboard(&'static str),
    Scale(Box<[Expr; 3]>),
    Translation(Box<[Expr; 3]>),
    ViewRange(Expr),
    /// `None`: fully transparent.
    Background(Option<ColorSrc>),
    Opacity(Expr),
    LineWidth(Expr),
    Alignment(&'static str),
    Shadow,
    SeeThrough,
    DefaultBackground,
    Glow(ColorSrc),
    Brightness(Expr),
    Mode(&'static str),
    ShadowRadius(Expr),
    ShadowStrength(Expr),
}

/// `spawn text display at <place> with text "hi" with billboard center`.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplayFx {
    pub kind: DisplayKind,
    /// Text for a text display, a block id, or an item id.
    pub subject: Option<Expr>,
    pub at: Expr,
    pub opts: Vec<DisplayOpt>,
}

const BILLBOARDS: [&str; 4] = ["fixed", "vertical", "horizontal", "center"];
const ALIGNMENTS: [&str; 3] = ["center", "left", "right"];
const ITEM_MODES: [&str; 9] = [
    "none",
    "thirdperson_lefthand",
    "thirdperson_righthand",
    "firstperson_lefthand",
    "firstperson_righthand",
    "head",
    "gui",
    "ground",
    "fixed",
];

fn pick(list: &'static [&'static str], toks: &[Tok]) -> Option<&'static str> {
    let w = joined(toks)?.replace('_', "");
    list.iter().copied().find(|c| c.replace('_', "") == w)
}

/// `Ok(None)`: not a display line. `Err`: it is one, but wrong.
pub fn parse_display(toks: &[Tok], env: &Env) -> Result<Option<DisplayFx>, String> {
    let mut i = 1;
    while toks.get(i).is_some_and(|t| is(t, "a") || is(t, "an") || is(t, "the") || is(t, "new")) {
        i += 1;
    }
    let kind = match toks.get(i).and_then(word).as_deref() {
        Some("text") => DisplayKind::Text,
        Some("block") => DisplayKind::Block,
        Some("item") => DisplayKind::Item,
        _ => return Ok(None),
    };
    if !toks.get(i + 1).is_some_and(|t| is(t, "display")) {
        return Ok(None);
    }
    i += 2;
    if toks.get(i).is_some_and(|t| is(t, "entity")) {
        i += 1;
    }
    let Some(at_i) = toks.iter().skip(i).position(|t| is(t, "at") || is(t, "on")).map(|p| p + i) else {
        return Err("display needs a place: spawn text display at <place>".into());
    };
    let mut subject_toks = toks.get(i..at_i).unwrap_or_default();
    if let Some(f) = subject_toks.first()
        && (is(f, "of") || is(f, "showing") || is(f, "with") || is(f, "from"))
    {
        subject_toks = subject_toks.get(1..).unwrap_or_default();
    }
    let subject = if subject_toks.is_empty() {
        None
    } else {
        Some(match kind {
            DisplayKind::Text => expr_of(subject_toks, env, &[Ty::Text]).ok_or("display text must be text")?,
            DisplayKind::Block => id_src(subject_toks, env).ok_or("display block must be a block id")?,
            DisplayKind::Item => {
                let e = id_src(subject_toks, env).ok_or("display item must be an item id")?;
                if let Some(name) = joined(subject_toks)
                    && !crate::items::is_known(&name)
                {
                    return Err(format!("unknown item '{}'", name.replace('_', " ")));
                }
                e
            }
        })
    };
    let tail = toks.get(at_i + 1..).unwrap_or_default();
    let mut parts = tail.split(|t| is(t, "with"));
    let at = parts
        .next()
        .and_then(|t| expr_of(t, env, &[Ty::Location, Ty::Player]))
        .ok_or("display place must be a location or a player")?;
    let mut fx = DisplayFx {
        kind,
        subject,
        at,
        opts: Vec::new(),
    };
    for seg in parts {
        let seg = match seg.split_last() {
            Some((a, v)) if is(a, "and") => v,
            _ => seg,
        };
        let opt = display_opt(seg, env, kind)?;
        fx.opts.push(opt);
    }
    if kind != DisplayKind::Text && fx.opts.iter().any(|o| matches!(o, DisplayOpt::Text(_))) {
        return Err("only text displays have text".into());
    }
    if kind != DisplayKind::Item && fx.opts.iter().any(|o| matches!(o, DisplayOpt::Mode(_))) {
        return Err("only item displays have a display mode".into());
    }
    if fx.subject.is_none() && !fx.opts.iter().any(|o| matches!(o, DisplayOpt::Text(_))) && kind != DisplayKind::Text {
        return Err("block and item displays need an id: spawn block display of stone at <place>".into());
    }
    Ok(Some(fx))
}

fn display_opt(seg: &[Tok], env: &Env, kind: DisplayKind) -> Result<DisplayOpt, String> {
    let words: Vec<String> = seg.iter().map_while(word).collect();
    let w = |n: usize| words.get(n).map(String::as_str);
    let rest = |n: usize| seg.get(n..).unwrap_or_default();
    let num = |t: &[Tok], what: &str| number(t, env).ok_or_else(|| format!("{what} must be a number"));
    Ok(match (w(0), w(1)) {
        (Some("text"), Some("opacity")) => DisplayOpt::Opacity(num(rest(2), "opacity")?),
        (Some("text"), _) => DisplayOpt::Text(expr_of(rest(1), env, &[Ty::Text]).ok_or("display text must be text")?),
        (Some("billboard"), _) => {
            DisplayOpt::Billboard(pick(&BILLBOARDS, rest(1)).ok_or("billboard must be fixed, vertical, horizontal or center")?)
        }
        (Some("scale" | "size"), _) => DisplayOpt::Scale(one_or_three(rest(1), env).ok_or("scale must be N or x, y, z")?),
        (Some("translation" | "offset"), _) => {
            DisplayOpt::Translation(triple(rest(1), env).ok_or("translation must be x, y, z")?)
        }
        (Some("view"), Some("range" | "distance")) => DisplayOpt::ViewRange(num(rest(2), "view range")?),
        (Some("viewrange"), _) => DisplayOpt::ViewRange(num(rest(1), "view range")?),
        (Some("opacity"), _) => DisplayOpt::Opacity(num(rest(1), "opacity")?),
        (Some("line"), Some("width")) => DisplayOpt::LineWidth(num(rest(2), "line width")?),
        (Some("alignment" | "align"), _) => {
            DisplayOpt::Alignment(pick(&ALIGNMENTS, rest(1)).ok_or("alignment must be center, left or right")?)
        }
        (Some("shadow"), Some("radius")) => DisplayOpt::ShadowRadius(num(rest(2), "shadow radius")?),
        (Some("shadow"), Some("strength")) => DisplayOpt::ShadowStrength(num(rest(2), "shadow strength")?),
        (Some("shadow"), None) => DisplayOpt::Shadow,
        (Some("see"), Some("through")) if seg.len() == 2 => DisplayOpt::SeeThrough,
        (Some("seethrough"), None) => DisplayOpt::SeeThrough,
        (Some("default"), Some("background")) if seg.len() == 2 => DisplayOpt::DefaultBackground,
        (Some("background"), _) => {
            let val = if w(1) == Some("color") || w(1) == Some("colour") { rest(2) } else { rest(1) };
            if joined(val).as_deref() == Some("transparent") || joined(val).as_deref() == Some("none") {
                DisplayOpt::Background(None)
            } else {
                DisplayOpt::Background(Some(color_src(val, env).ok_or("background must be a color")?))
            }
        }
        (Some("glow"), _) => {
            let val = if w(1) == Some("color") || w(1) == Some("colour") { rest(2) } else { rest(1) };
            DisplayOpt::Glow(color_src(val, env).ok_or("glow must be a color")?)
        }
        (Some("brightness" | "light"), _) => DisplayOpt::Brightness(num(rest(1), "brightness")?),
        (Some("mode"), _) => DisplayOpt::Mode(pick(&ITEM_MODES, rest(1)).ok_or("unknown item display mode")?),
        (Some("item"), Some("mode")) => DisplayOpt::Mode(pick(&ITEM_MODES, rest(2)).ok_or("unknown item display mode")?),
        _ => {
            let _ = kind;
            return Err(format!(
                "unknown display option '{}'",
                words.join(" ")
            ));
        }
    })
}

/// A display ready to spawn. All numbers are checked and clamped.
#[derive(Debug, Clone, PartialEq)]
pub struct DisplaySpec {
    pub kind: DisplayKind,
    pub at: FxPos,
    /// Text with `&` and `<tag>` colors still in it.
    pub text: Option<String>,
    /// Block id for a block display, item id for an item display.
    pub id: Option<String>,
    pub billboard: Option<&'static str>,
    pub scale: Option<[f32; 3]>,
    pub translation: Option<[f32; 3]>,
    pub view_range: Option<f32>,
    /// ARGB.
    pub background: Option<i32>,
    pub opacity: Option<u8>,
    pub line_width: Option<i32>,
    pub alignment: Option<&'static str>,
    pub shadow: bool,
    pub see_through: bool,
    pub default_background: bool,
    /// RGB.
    pub glow: Option<i32>,
    /// 0 to 15.
    pub brightness: Option<u8>,
    pub item_mode: Option<&'static str>,
    pub shadow_radius: Option<f32>,
    pub shadow_strength: Option<f32>,
}

const MAX_TEXT: usize = 4000;

fn argb(c: [u8; 3]) -> i32 {
    // Vanilla default background alpha is 0x40; an explicit color is opaque.
    i32::from_be_bytes([0xFF, c[0], c[1], c[2]])
}

fn rgb_int(c: [u8; 3]) -> i32 {
    i32::from_be_bytes([0, c[0], c[1], c[2]])
}

impl Engine {
    fn clamped(&mut self, e: &Expr, lo: f64, hi: f64, ex: &mut Exec, host: &mut dyn Host) -> Result<f64, String> {
        let n = self.num(e, ex, host)?;
        Ok(if n.is_finite() { n.clamp(lo, hi) } else { lo })
    }

    fn triple_f32(
        &mut self,
        p: &[Expr; 3],
        lo: f64,
        hi: f64,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<[f32; 3], String> {
        let mut out = [0.0f32; 3];
        for (slot, e) in out.iter_mut().zip(p.iter()) {
            *slot = self.clamped(e, lo, hi, ex, host)? as f32;
        }
        Ok(out)
    }

    #[inline(never)]
    pub(crate) fn display_effect(&mut self, d: &DisplayFx, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        let Some(at) = self.fx_pos(&d.at, ex, host)? else {
            return Ok(());
        };
        let mut s = DisplaySpec {
            kind: d.kind,
            at,
            text: None,
            id: None,
            billboard: None,
            scale: None,
            translation: None,
            view_range: None,
            background: None,
            opacity: None,
            line_width: None,
            alignment: None,
            shadow: false,
            see_through: false,
            default_background: false,
            glow: None,
            brightness: None,
            item_mode: None,
            shadow_radius: None,
            shadow_strength: None,
        };
        if let Some(sub) = &d.subject {
            match d.kind {
                DisplayKind::Text => s.text = Some(self.text(sub, ex, host)?),
                _ => s.id = Some(self.id_text(sub, ex, host)?),
            }
        }
        for o in &d.opts {
            match o {
                DisplayOpt::Text(e) => s.text = Some(self.text(e, ex, host)?),
                DisplayOpt::Billboard(b) => s.billboard = Some(b),
                DisplayOpt::Scale(p) => s.scale = Some(self.triple_f32(p, 0.0, 100.0, ex, host)?),
                DisplayOpt::Translation(p) => s.translation = Some(self.triple_f32(p, -1000.0, 1000.0, ex, host)?),
                DisplayOpt::ViewRange(e) => s.view_range = Some(self.clamped(e, 0.0, 100.0, ex, host)? as f32),
                DisplayOpt::Background(None) => s.background = Some(0),
                DisplayOpt::Background(Some(c)) => s.background = Some(argb(self.rgb(c, ex, host)?)),
                DisplayOpt::Opacity(e) => s.opacity = Some(self.clamped(e, 0.0, 255.0, ex, host)? as u8),
                DisplayOpt::LineWidth(e) => s.line_width = Some(self.clamped(e, 1.0, 10_000.0, ex, host)? as i32),
                DisplayOpt::Alignment(a) => s.alignment = Some(a),
                DisplayOpt::Shadow => s.shadow = true,
                DisplayOpt::SeeThrough => s.see_through = true,
                DisplayOpt::DefaultBackground => s.default_background = true,
                DisplayOpt::Glow(c) => s.glow = Some(rgb_int(self.rgb(c, ex, host)?)),
                DisplayOpt::Brightness(e) => s.brightness = Some(self.clamped(e, 0.0, 15.0, ex, host)? as u8),
                DisplayOpt::Mode(m) => s.item_mode = Some(m),
                DisplayOpt::ShadowRadius(e) => s.shadow_radius = Some(self.clamped(e, 0.0, 64.0, ex, host)? as f32),
                DisplayOpt::ShadowStrength(e) => s.shadow_strength = Some(self.clamped(e, 0.0, 100.0, ex, host)? as f32),
            }
        }
        if let Some(t) = &mut s.text
            && t.chars().count() > MAX_TEXT
        {
            *t = t.chars().take(MAX_TEXT).collect();
        }
        host.spawn_display(&s);
        Ok(())
    }
}

// ---------- default host: the /summon command ----------

fn snbt_str(s: &str) -> String {
    format!("'{}'", s.replace('\\', "\\\\").replace('\'', "\\'"))
}

/// The JSON text component for `&`-colored text.
pub fn text_json(text: &str) -> String {
    let spans = colors::parse(text);
    let mut parts = Vec::with_capacity(spans.len() + 1);
    parts.push(serde_json::json!({ "text": "" }));
    for sp in spans {
        let mut o = serde_json::Map::new();
        o.insert("text".into(), sp.text.into());
        if let Some((r, g, b)) = sp.style.color {
            o.insert("color".into(), format!("#{r:02x}{g:02x}{b:02x}").into());
        }
        for (key, on) in [
            ("bold", sp.style.bold),
            ("italic", sp.style.italic),
            ("underlined", sp.style.underlined),
            ("strikethrough", sp.style.strikethrough),
            ("obfuscated", sp.style.obfuscated),
        ] {
            if on {
                o.insert(key.into(), true.into());
            }
        }
        parts.push(o.into());
    }
    serde_json::Value::Array(parts).to_string()
}

fn float_list(v: [f32; 3]) -> String {
    format!("[{}f,{}f,{}f]", v[0], v[1], v[2])
}

/// `summon` line for a text display. `None` for kinds the command cannot do.
pub fn summon_command(d: &DisplaySpec) -> Option<String> {
    if d.kind != DisplayKind::Text {
        return None;
    }
    let mut f: Vec<String> = Vec::new();
    f.push(format!("text:{}", snbt_str(&text_json(d.text.as_deref().unwrap_or("")))));
    if let Some(b) = d.billboard {
        f.push(format!("billboard:\"{b}\""));
    }
    if d.scale.is_some() || d.translation.is_some() {
        let sc = d.scale.unwrap_or([1.0; 3]);
        let tr = d.translation.unwrap_or([0.0; 3]);
        f.push(format!(
            "transformation:{{translation:{},scale:{},left_rotation:[0f,0f,0f,1f],right_rotation:[0f,0f,0f,1f]}}",
            float_list(tr),
            float_list(sc)
        ));
    }
    if let Some(v) = d.view_range {
        f.push(format!("view_range:{v}f"));
    }
    if let Some(v) = d.shadow_radius {
        f.push(format!("shadow_radius:{v}f"));
    }
    if let Some(v) = d.shadow_strength {
        f.push(format!("shadow_strength:{v}f"));
    }
    if let Some(b) = d.background {
        f.push(format!("background:{b}"));
    }
    if let Some(o) = d.opacity {
        f.push(format!("text_opacity:{}b", o as i8));
    }
    if let Some(w) = d.line_width {
        f.push(format!("line_width:{w}"));
    }
    if let Some(a) = d.alignment {
        f.push(format!("alignment:\"{a}\""));
    }
    if let Some(g) = d.glow {
        f.push(format!("glow_color_override:{g}"));
    }
    if let Some(b) = d.brightness {
        f.push(format!("brightness:{{block:{b},sky:{b}}}"));
    }
    for (on, key) in [(d.shadow, "shadow"), (d.see_through, "see_through"), (d.default_background, "default_background")] {
        if on {
            f.push(format!("{key}:1b"));
        }
    }
    Some(format!(
        "summon minecraft:text_display {} {} {} {{{}}}",
        d.at.x,
        d.at.y,
        d.at.z,
        f.join(",")
    ))
}

/// `/particle` line for a burst.
pub fn particle_command(p: &crate::world_effects::ParticleFx) -> String {
    let [dx, dy, dz] = p.offsets();
    format!(
        "particle minecraft:{} {} {} {} {dx} {dy} {dz} {} {}{}",
        p.name,
        p.at.x,
        p.at.y,
        p.at.z,
        p.speed,
        p.count,
        if p.force { " force" } else { "" },
    )
}

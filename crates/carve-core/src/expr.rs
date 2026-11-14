//! Expressions: literals, variables, text templates, event values, arithmetic.

use std::cell::Cell;

use crate::lexer::{Tok, lex};
use crate::list_literals::ElemSel;
use crate::list_ops::{Agg, ListSrc, parse_src_at};
use crate::value::format_num;
use crate::w2_expressions_2::W2Op;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Ty {
    Number,
    Text,
    Player,
    Boolean,
    Item,
    Location,
    World,
    Timespan,
    Date,
    Vector,
    /// A type from `types_1`: a named-value type, color, quaternion, ...
    T1(u8),
    /// Unknown until runtime (variables).
    Object,
    /// A list (function parameters and results like `numbers`).
    List,
}

impl Ty {
    pub fn name(self) -> &'static str {
        match self {
            Ty::Number => "number",
            Ty::Text => "text",
            Ty::Player => "player",
            Ty::Boolean => "boolean",
            Ty::Item => "item",
            Ty::Location => "location",
            Ty::World => "world",
            Ty::Timespan => "timespan",
            Ty::Date => "date",
            Ty::Vector => "vector",
            Ty::T1(id) => crate::types_1::ty_name(id),
            Ty::Object => "object",
            Ty::List => "list",
        }
    }

    /// Can a value of type `got` fill a slot of type `self`?
    pub fn accepts(self, got: Ty) -> bool {
        self == Ty::Object
            || got == Ty::Object
            || self == Ty::Text
            || self == got
            || (self == Ty::Number && got == Ty::Date)
            || (matches!(self, Ty::T1(_)) && got == Ty::Text)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Part {
    Lit(String),
    Expr(Expr),
}

/// A variable name, lowercased. Names made only of plain text are lowercased
/// once at compile time; names with `%...%` parts are built on every use.
#[derive(Debug, Clone, PartialEq)]
pub enum Key {
    Static(String),
    Dynamic(Vec<Part>),
}

#[derive(Debug, Clone, PartialEq)]
pub struct VarRef {
    pub key: Key,
    pub local: bool,
    /// `{x::*}`: the whole list.
    pub list: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Bool(bool),
    Text(Vec<Part>),
    Var(VarRef),
    Player,
    /// `sender`, `executor`: whoever started the trigger (a player or the console).
    Sender,
    /// `console`
    Console,
    Message,
    /// 1-based command argument.
    Arg(usize),
    NameOf(Box<Expr>),
    Neg(Box<Expr>),
    Bin(Box<Expr>, BinOp, Box<Expr>),
    /// Current value of the loop at this depth (0 = outermost loop).
    LoopValue(usize),
    /// List index (`loop-index`) of the list loop at this depth.
    LoopIndex(usize),
    /// 1-based round counter (`loop-iteration`) of the loop at this depth.
    LoopIter(usize),
    /// Number of items in a list variable (`size of {x::*}`).
    Size(VarRef),
    /// Call of a script function, by id, with its arguments.
    Call(usize, Vec<Expr>),
    /// Item literal like `diamond sword` or `2 diamonds`.
    Item {
        id: String,
        count: u32,
    },
    /// An item with a name, lore or enchantments added.
    ItemMeta(Box<crate::item_meta::MetaExpr>),
    /// `name of <item>`, `lore of <item>`, `amount of <item>` and so on.
    ItemInfo(crate::item_meta::ItemInfo, Box<Expr>),
    /// An enchantment written in a script: `sharpness 5`. Reads as text.
    Enchant(Box<(String, u32)>),
    /// `location of player`, `player's world`, `x of {_loc}` and so on.
    Prop(Prop, Box<Expr>),
    /// `location(x, y, z[, world])`.
    Loc(Box<Expr>, Box<Expr>, Box<Expr>, Option<Box<Expr>>),
    /// `world "name"`.
    WorldNamed(Box<Expr>),
    /// `block at loc`, `block below player`, `targeted block of player`, `type of block ...`.
    Blk(Box<crate::block_ops::BlockExpr>),
    /// `distance between a and b`.
    Distance(Box<Expr>, Box<Expr>),
    /// `random integer between a and b` (true) / `random number between a and b`.
    Random(bool, Box<Expr>, Box<Expr>),
    /// `event-<name>` of a `w2_events_4` event: index into the event values.
    Ev4Val(u8),
    /// `damage` in `on damage`.
    Damage,
    /// A value of one of the extra events (`w2_events_2`).
    Ev2Val(crate::w2_events_2::Slot),
    /// `attacker` in `on damage` / `on death` (a player, or `<none>`).
    Attacker,
    /// `clicked block` in click events (an item value of the block type).
    ClickedBlock,
    /// `event-item`, `new level`, ... in the item and level events.
    EventVal(crate::events_items::EventVal),
    /// Inventory values: `slot 3 of player's inventory`, `clicked slot`.
    Inv(Box<crate::inventories::InvExpr>),
    /// `event-block`, `event-location`, ...: the nth value of `EventCtx::ev`.
    ExtVal(u8),
    /// `a ? b`: a, or b when a is not set.
    Default(Box<Expr>, Box<Expr>),
    /// `floor(x)`, `max(a, b)`, ...
    Math(MathFn, Vec<Expr>),
    /// `x parsed as number`: the number, or `<none>`.
    ParseNum(Box<Expr>),
    /// `random element of {list::*}`.
    RandomOf(VarRef),
    /// `join {list::*} with ", "`.
    Join(VarRef, Box<Expr>),
    /// `split "a,b" at ","`: a list, only usable in `set {x::*} to` and `loop`.
    Split(Box<Expr>, Box<Expr>),
    /// `length of x` / `amount of characters in x`.
    Length(Box<Expr>),
    /// `x with all "a" replaced with "b"`: text, from, to, replace all.
    Replaced(Box<Expr>, Box<Expr>, Box<Expr>, bool),
    /// Text functions: case, substring, characters, index of, trimmed, reversed.
    TextFn(crate::text_case_substring::TextFn, Vec<Expr>),
    /// SkBee utilities: number formatting, roman, hex, json, random uuid, client info.
    Util(crate::skbee_utilities::UtilFn, Vec<Expr>),
    /// Rare expressions of `w2_expressions_2`, boxed so `Expr` stays small.
    X2(Box<crate::w2_expressions_2::W2>),
    /// `indices of {x::*}`: the index of every item, as text.
    Indices(VarRef),
    /// `first element of {x::*}`, `element 2 of {x::*}`.
    Elem(ElemSel, VarRef),
    /// `now`, `unix timestamp of d`, `date from unix timestamp n`, `d formatted as f`.
    Date(DateOp, Vec<Expr>),
    /// SkBee-style text components (markup text): see `skbee_text`.
    Comp(crate::skbee_text::CompFn, Vec<Expr>),
    /// `colored x` (true) / `uncolored x` (false).
    Color(bool, Box<Expr>),
    /// A time span literal like `1 hour and 30 seconds`, in milliseconds.
    Span(i64),
    /// `time since <date>` (true) / `time until <date>` (false).
    Since(Box<Expr>, bool),
    /// Math where a side is a time span or date.
    TBin(Box<Expr>, BinOp, Box<Expr>),
    /// `minutes of <span>` / `total minutes of <span>`: ms per unit, total?
    SpanPart(i64, bool, Box<Expr>),
    /// `sorted {x::*}`, `{x::*} where [...]`, ...: a list value.
    Items(Box<ListSrc>),
    /// `sum of {x::*}`, `max of ...`, `size of {x::*} where [...]`.
    ListAgg(Agg, Box<ListSrc>),
    /// Bounds: `bound at <loc>`, `owners of bound X`, ...
    Bound(Box<crate::skbee_bound::BoundExpr>),
    /// `time in world "w"`, `weather`, `difficulty`, `gamerule keepInventory`.
    Wt(Box<crate::weather_time::WtExpr>),
    /// Scoreboard scores, teams and objectives (SkBee style).
    Sb(Box<crate::skbee_scoreboard_teams::SbExpr>),
    /// SkBee-style NBT and item component expressions.
    Nbt(Box<crate::skbee_nbt::NbtExpr>),
    /// `last spawned entity`, `nearest zombie to player`, `all pigs in radius 5 of player`, ...
    Ent(Box<crate::entities::EntExpr>),
    /// Statistics and the event id of advancement/recipe events.
    Skb(Box<crate::skbee_recipes_advancements::SkbExpr>),
    /// Player count, MOTD, version, worlds, spawn, world border size.
    Server(crate::server_misc::ServerQ, Option<Box<Expr>>),
    /// Rare expressions of `w2_expressions_1` (eternity, biome, armor, block entities, ...).
    W2(Box<crate::w2_expressions_1::W2>),
    /// Player stats, hashes, memory, uuid lookups and friends (see `w2_expressions_3`).
    Ext(crate::w2_expressions_3::ExtFn, Vec<Expr>),
    /// A rare expression from `w2_expressions_4`.
    W4(Box<crate::w2_expressions_4::W4>),
    /// Rare wave 2 expressions (numbers, midpoint, metadata, ...), boxed to keep this enum small.
    W5(Box<crate::w2_expressions_5::W5>),
    /// `placeholder "x" for player` and `parse placeholders in ...`.
    Ph(Box<crate::placeholders::Ph>),
    /// Rare wave-2 expressions (see `w2_expressions_6.rs`).
    W6(Box<crate::w2_expressions_6::W6>),
    /// Vectors, server facts and other rare expressions (see `w2_expressions_7`).
    W7(Box<crate::w2_expressions_7::Node>),
    /// Wave-4 functions: trigonometry, `clamp`, `concat`, `formatNumber`, ... (see `w4_functions`).
    W4F(Box<crate::w4_functions::Node>),
    /// Colors, named values and other types (see `types_1`).
    T1(Box<crate::types_1::Node>),
    /// Ray tracing, chunks and tags (see `w4_skbee_raytrace_world`).
    Rtw(Box<crate::w4_skbee_raytrace_world::Node>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DateOp {
    Now,
    /// Seconds since 1970 of a date.
    ToUnix,
    /// Date from seconds since 1970.
    FromUnix,
    /// Date, optional format text.
    Format,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MathFn {
    Floor,
    Ceil,
    Round,
    Abs,
    Sqrt,
    Min,
    Max,
    Mod,
}

impl MathFn {
    fn from_name(n: &str) -> Option<(MathFn, usize)> {
        Some(match n {
            "floor" => (MathFn::Floor, 1),
            "ceil" | "ceiling" => (MathFn::Ceil, 1),
            "round" => (MathFn::Round, 1),
            "abs" => (MathFn::Abs, 1),
            "sqrt" => (MathFn::Sqrt, 1),
            "min" => (MathFn::Min, 2),
            "max" => (MathFn::Max, 2),
            "mod" => (MathFn::Mod, 2),
            _ => return None,
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Prop {
    Location,
    World,
    Uuid,
    Name,
    Tool,
    Health,
    Gamemode,
    Level,
    Xp,
    LevelProgress,
    MaxHealth,
    Food,
    Saturation,
    X,
    Y,
    Z,
    DisplayName,
    TabName,
    Ping,
    Ip,
}

impl Prop {
    fn from_word(w: &str) -> Option<Prop> {
        Some(match w {
            "location" | "position" => Prop::Location,
            "world" => Prop::World,
            "uuid" => Prop::Uuid,
            "name" | "display" => Prop::Name,
            "tool" | "held" => Prop::Tool,
            "health" => Prop::Health,
            "gamemode" => Prop::Gamemode,
            "level" => Prop::Level,
            "xp" | "exp" | "experience" => Prop::Xp,
            "nickname" => Prop::DisplayName,
            "ping" => Prop::Ping,
            "ip" => Prop::Ip,
            "x" => Prop::X,
            "y" => Prop::Y,
            "z" => Prop::Z,
            _ => return None,
        })
    }

    pub fn ty(self) -> Ty {
        match self {
            Prop::Location => Ty::Location,
            Prop::World => Ty::World,
            Prop::Uuid | Prop::Name | Prop::Gamemode | Prop::DisplayName | Prop::TabName | Prop::Ip => Ty::Text,
            Prop::Tool => Ty::Item,
            Prop::Health
            | Prop::MaxHealth
            | Prop::Food
            | Prop::Saturation
            | Prop::Level
            | Prop::Xp
            | Prop::LevelProgress
            | Prop::X
            | Prop::Y
            | Prop::Z
            | Prop::Ping => Ty::Number,
        }
    }

    /// Types this property can be read from.
    fn takes(self, t: Ty) -> bool {
        t == Ty::Object
            || match self {
                Prop::X | Prop::Y | Prop::Z | Prop::World => {
                    t == Ty::Player || t == Ty::Location || (t == Ty::Vector && !matches!(self, Prop::World))
                }
                Prop::Location => t == Ty::Player || t == Ty::Vector,
                _ => t == Ty::Player,
            }
    }
}

/// A function's parameters and return type, known to every script at load.
#[derive(Debug, Clone, PartialEq)]
pub struct Sig {
    pub name: String,
    /// (local variable key like `_name`, type, has a default value)
    pub params: Vec<(String, Ty, bool)>,
    pub ret: Option<Ty>,
    /// Its header failed to load: calls to it are errors where they are.
    pub failed: bool,
}

/// What a loop goes over. Decides the type of `loop-value`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LoopKind {
    Times,
    List,
    Players,
    Values,
}

/// What the current trigger offers to expressions.
#[derive(Debug, Clone, Default)]
pub struct Env {
    pub has_message: bool,
    pub arg_types: Vec<Ty>,
    /// `Some("chat")` etc, used in error messages.
    pub trigger_name: String,
    /// Loops around the current line, outermost first (`while` not included).
    pub loops: Vec<LoopKind>,
    /// Every function of every script, by id.
    pub functions: std::sync::Arc<Vec<Sig>>,
    /// Inside a function body: no event values (`player`, `message`, `arg`).
    pub in_function: bool,
    /// The trigger has no player (`on load`, `every`).
    pub no_player: bool,
    pub has_damage: bool,
    pub has_attacker: bool,
    pub has_block: bool,
    /// Bit set of the event values of the trigger (`events_items::mask`).
    pub event_vals: u16,
    /// `on inventory click`: `clicked slot` and `clicked item` exist.
    pub has_inv: bool,
    /// `on bound enter` / `on bound exit`: `event-bound` exists.
    pub has_bound: bool,
    /// `event-...` values of the current trigger (see `w2_events_1`).
    pub ev_fields: &'static [(&'static str, Ty)],
    /// Set inside the extra events of `w2_events_2`.
    pub ext_event: Option<crate::w2_events_2::Ev>,
    /// The trigger is an event of `w2_events_4` (gives `event-<name>`).
    pub ext4: Option<crate::w2_events_4::ExtEvent>,
}

impl Expr {
    pub fn ty(&self, env: &Env) -> Ty {
        match self {
            Expr::Num(_) | Expr::Neg(_) | Expr::Bin(..) | Expr::LoopIter(_) | Expr::Size(_) => Ty::Number,
            Expr::Span(_) | Expr::Since(..) => Ty::Timespan,
            Expr::SpanPart(..) => Ty::Number,
            Expr::TBin(a, op, b) => crate::timespans::result_ty(a.ty(env), *op, b.ty(env)),
            Expr::LoopIndex(_) => Ty::Text,
            Expr::Bool(_) => Ty::Boolean,
            Expr::Call(id, _) => env.functions.get(*id).and_then(|s| s.ret).unwrap_or(Ty::Object),
            Expr::Item { .. } | Expr::ItemMeta(_) => Ty::Item,
            Expr::ItemInfo(info, _) => {
                use crate::item_meta::ItemInfo;
                match info {
                    ItemInfo::Name => Ty::Text,
                    ItemInfo::Lore | ItemInfo::Enchants => Ty::List,
                    ItemInfo::Type => Ty::Item,
                    ItemInfo::Amount => Ty::Number,
                }
            }
            Expr::Enchant(_) => Ty::Text,
            Expr::Prop(p, _) => p.ty(),
            Expr::Ent(e) => crate::entities::ty(e),
            Expr::Loc(..) => Ty::Location,
            Expr::WorldNamed(_) => Ty::World,
            Expr::Blk(b) => b.ty(),
            Expr::Wt(w) => match w.kind {
                crate::weather_time::WtKind::Time => Ty::Number,
                crate::weather_time::WtKind::Weather | crate::weather_time::WtKind::Difficulty => Ty::Text,
                crate::weather_time::WtKind::Gamerule(_) => Ty::Object,
            },
            Expr::Distance(..) | Expr::Random(..) | Expr::Damage => Ty::Number,
            Expr::Attacker => Ty::Player,
            Expr::Ev2Val(s) => s.ty(),
            Expr::Ev4Val(_) => Ty::Object,
            Expr::Math(..) | Expr::ParseNum(_) => Ty::Number,
            Expr::Color(..) | Expr::Comp(..) => Ty::Text,
            Expr::RandomOf(_) | Expr::Split(..) | Expr::Elem(..) => Ty::Object,
            Expr::Join(..) | Expr::Replaced(..) => Ty::Text,
            Expr::Length(_) => Ty::Number,
            Expr::TextFn(f, _) if f.returns_number() => Ty::Number,
            Expr::TextFn(..) => Ty::Text,
            Expr::Util(f, _) if f.returns_number() => Ty::Number,
            Expr::Util(..) => Ty::Text,
            Expr::X2(w) => w.op.ty(&w.args, env),
            Expr::Indices(_) => Ty::Text,
            Expr::Date(op, _) => match op {
                DateOp::Now | DateOp::FromUnix => Ty::Date,
                DateOp::ToUnix => Ty::Number,
                DateOp::Format => Ty::Text,
            },
            Expr::Skb(b) => match **b {
                crate::skbee_recipes_advancements::SkbExpr::Stat(..) => Ty::Number,
                crate::skbee_recipes_advancements::SkbExpr::EventId => Ty::Text,
            },
            Expr::W7(n) => n.ty,
            Expr::W4F(n) => n.f.ty(),
            Expr::T1(n) => n.ty,
            Expr::Rtw(n) => n.ty,
            Expr::Items(_) => Ty::List,
            Expr::Bound(b) => b.ty(),
            Expr::Sb(s) => s.ty(),
            Expr::Nbt(n) => crate::skbee_nbt::expr_ty(n),
            Expr::Server(q, _) => q.ty(),
            Expr::W2(w) => w.ty(),
            Expr::Ext(f, _) => crate::w2_expressions_3::ty(*f),
            Expr::W4(w) => w.ty(),
            Expr::W5(w) => w.ty(),
            Expr::Ph(p) => p.ty(),
            Expr::W6(w) => w.ty(env),
            Expr::ListAgg(Agg::Size | Agg::Sum, _) => Ty::Number,
            Expr::ListAgg(..) => Ty::Object,
            Expr::Default(a, b) => {
                let (x, y) = (a.ty(env), b.ty(env));
                if x == y { x } else { Ty::Object }
            }
            Expr::ClickedBlock => Ty::Item,
            Expr::EventVal(v) => v.ty(),
            Expr::Inv(i) => i.ty(),
            Expr::ExtVal(i) => env.ev_fields.get(usize::from(*i)).map_or(Ty::Object, |f| f.1),
            Expr::LoopValue(d) => match env.loops.get(*d) {
                Some(LoopKind::Times) => Ty::Number,
                Some(LoopKind::Players) => Ty::Player,
                _ => Ty::Object,
            },
            Expr::Text(_) | Expr::Message | Expr::NameOf(_) => Ty::Text,
            Expr::Var(_) => Ty::Object,
            Expr::Player => Ty::Player,
            Expr::Sender | Expr::Console => Ty::Object,
            Expr::Arg(i) => env
                .arg_types
                .get(i.wrapping_sub(1))
                .copied()
                .unwrap_or(Ty::Object),
        }
    }
}

/// Parse one arithmetic expression starting at `pos`; returns it and where it ended.
pub(crate) fn parse_sum_at(toks: &[Tok], pos: usize, env: &Env) -> Result<(Expr, usize), String> {
    let mut p = P {
        toks,
        pos,
        env,
        hit_end: Cell::new(false),
    };
    let e = p.sum()?;
    Ok((e, p.pos))
}

pub fn parse_expr(toks: &[Tok], env: &Env) -> Result<Expr, String> {
    parse_expr_span(toks, env).map_err(|(msg, _)| msg)
}

/// Like [`parse_expr`]. On failure, also says whether the parser looked past
/// the last token. If it did not, adding more tokens can't make the parse
/// succeed, which lets the pattern matcher stop trying longer spans.
pub fn parse_expr_span(toks: &[Tok], env: &Env) -> Result<Expr, (String, bool)> {
    if toks.is_empty() {
        return Err(("missing expression".into(), true));
    }
    let mut p = P {
        toks,
        pos: 0,
        env,
        hit_end: Cell::new(false),
    };
    let e = p.top().map_err(|m| (m, p.hit_end.get()))?;
    if let Some(t) = toks.get(p.pos) {
        return Err((format!("unexpected {}", describe(t)), p.hit_end.get()));
    }
    Ok(e)
}

pub fn parse_expr_str(s: &str, env: &Env) -> Result<Expr, String> {
    parse_expr(&lex(s)?, env)
}

fn describe(t: &Tok) -> String {
    match t {
        Tok::Word(w) => format!("'{w}'"),
        Tok::Num(n) => format!("number {n}"),
        Tok::Str(_) => "text".into(),
        Tok::Var(v) => format!("{{{v}}}"),
        Tok::Sym(s) => format!("'{s}'"),
    }
}

#[path = "w2_expressions_4_parse.rs"]
mod w4_parse;

pub(crate) struct P<'a> {
    pub(crate) toks: &'a [Tok],
    pub(crate) pos: usize,
    pub(crate) env: &'a Env,
    /// Set when the parser asked for a token past the end.
    pub(crate) hit_end: Cell<bool>,
}

impl crate::skbee_scoreboard_teams::Operands for P<'_> {
    fn peek_tok(&self, n: usize) -> Option<&Tok> {
        self.peek_at(n)
    }
    fn advance(&mut self, n: usize) {
        self.pos += n;
    }
    fn operand(&mut self) -> Result<Expr, String> {
        self.primary()
    }
}

impl P<'_> {
    pub(crate) fn peek(&self) -> Option<&Tok> {
        let t = self.toks.get(self.pos);
        if t.is_none() {
            self.hit_end.set(true);
        }
        t
    }

    /// Like `peek`, `n` tokens further on; running off the end counts as "need more tokens".
    fn peek_at(&self, n: usize) -> Option<&Tok> {
        let t = self.toks.get(self.pos + n);
        if t.is_none() {
            self.hit_end.set(true);
        }
        t
    }

    pub(crate) fn eat_sym(&mut self, s: &str) -> bool {
        if matches!(self.peek(), Some(Tok::Sym(x)) if *x == s) {
            self.pos += 1;
            true
        } else {
            false
        }
    }

    /// `a ? b ? c`: the lowest priority, below math.
    pub(crate) fn top(&mut self) -> Result<Expr, String> {
        let mut e = self.sum()?;
        e = self.w6_ternary(e)?;
        while self.eat_sym("?") {
            let b = self.sum()?;
            e = Expr::Default(Box::new(e), Box::new(b));
        }
        Ok(e)
    }

    pub(crate) fn sum(&mut self) -> Result<Expr, String> {
        let mut left = self.product()?;
        loop {
            let op = if self.eat_sym("+") {
                BinOp::Add
            } else if self.eat_sym("-") {
                BinOp::Sub
            } else {
                return Ok(left);
            };
            let right = self.product()?;
            left = self.bin(left, op, right)?;
        }
    }

    fn product(&mut self) -> Result<Expr, String> {
        let mut left = self.unary()?;
        loop {
            let op = if self.eat_sym("*") {
                BinOp::Mul
            } else if self.eat_sym("/") {
                BinOp::Div
            } else {
                return Ok(left);
            };
            let right = self.unary()?;
            left = self.bin(left, op, right)?;
        }
    }

    fn bin(&self, l: Expr, op: BinOp, r: Expr) -> Result<Expr, String> {
        let (lt, rt) = (l.ty(self.env), r.ty(self.env));
        if lt == Ty::Vector || rt == Ty::Vector {
            return crate::w2_expressions_7::vec_bin(l, op, r, lt, rt);
        }
        if matches!(lt, Ty::Timespan | Ty::Date) || matches!(rt, Ty::Timespan | Ty::Date) {
            if let (Expr::Span(x), Expr::Span(y)) = (&l, &r) {
                match op {
                    BinOp::Add => return Ok(Expr::Span(x.saturating_add(*y))),
                    BinOp::Sub => return Ok(Expr::Span(x.saturating_sub(*y))),
                    _ => {}
                }
            }
            if !crate::timespans::ok_types(lt, op, rt) {
                return Err(format!(
                    "can't do {} and {} with that operator",
                    lt.name(),
                    rt.name()
                ));
            }
            return Ok(Expr::TBin(Box::new(l), op, Box::new(r)));
        }
        for side in [&l, &r] {
            let t = side.ty(self.env);
            if !Ty::Number.accepts(t) {
                return Err(format!("math needs numbers, got {}", t.name()));
            }
        }
        // Fold constant math. Division by zero stays a runtime error so it is
        // reported with its line, like any other runtime error.
        if let (Expr::Num(x), Expr::Num(y)) = (&l, &r) {
            let folded = match op {
                BinOp::Add => Some(x + y),
                BinOp::Sub => Some(x - y),
                BinOp::Mul => Some(x * y),
                BinOp::Div => (*y != 0.0).then(|| x / y),
            };
            if let Some(n) = folded {
                return Ok(Expr::Num(n));
            }
        }
        Ok(Expr::Bin(Box::new(l), op, Box::new(r)))
    }

    pub(crate) fn unary(&mut self) -> Result<Expr, String> {
        if self.eat_sym("-") {
            let inner = self.unary()?;
            if inner.ty(self.env) == Ty::Vector {
                return Ok(crate::w2_expressions_7::neg(inner));
            }
            if !Ty::Number.accepts(inner.ty(self.env)) {
                return Err("'-' needs a number".into());
            }
            return Ok(match inner {
                Expr::Num(n) => Expr::Num(-n),
                e => Expr::Neg(Box::new(e)),
            });
        }
        self.primary()
    }

    pub(crate) fn primary(&mut self) -> Result<Expr, String> {
        let mut e = self.primary_base()?;
        e = self.w2e7_post(e)?;
        // `5 minutes ago`, `2 hours later`
        if let Some(Tok::Word(w)) = self.toks.get(self.pos)
            && (w.eq_ignore_ascii_case("ago") || w.eq_ignore_ascii_case("later"))
            && matches!(e.ty(self.env), Ty::Timespan | Ty::Object)
        {
            let op = if w.eq_ignore_ascii_case("ago") {
                BinOp::Sub
            } else {
                BinOp::Add
            };
            self.pos += 1;
            e = Expr::TBin(Box::new(Expr::Date(DateOp::Now, Vec::new())), op, Box::new(e));
        }
        // `10% of x`
        if matches!(self.toks.get(self.pos), Some(t) if t.is_lit("%") || t.is_lit("percent"))
            && self.pos + 2 >= self.toks.len()
        {
            self.hit_end.set(true);
        }
        if crate::w2_expressions_5::percent_follows(self.toks, self.pos) && Ty::Number.accepts(e.ty(self.env)) {
            self.pos += 2;
            let of = self.primary_base()?;
            e = crate::w2_expressions_5::percent(e, of);
        }
        // `arg-1 parsed as number`
        if matches!(self.toks.get(self.pos), Some(t) if t.is_lit("parsed")) && self.pos + 1 >= self.toks.len()
        {
            self.hit_end.set(true);
        }
        if matches!(self.toks.get(self.pos), Some(t) if t.is_lit("parsed"))
            && matches!(self.toks.get(self.pos + 1), Some(t) if t.is_lit("as"))
        {
            match self.toks.get(self.pos + 2) {
                Some(t) if t.is_lit("number") || t.is_lit("integer") || t.is_lit("num") => {
                    self.pos += 3;
                    e = Expr::ParseNum(Box::new(e));
                }
                Some(_) => match crate::types_1::type_at(self.toks, self.pos + 2) {
                    Some((Ty::T1(id), n)) => {
                        self.pos += 2 + n;
                        e = crate::types_1::parse_as(e, id);
                    }
                    _ => {
                        if self.toks.get(self.pos + 2..).is_some_and(|r| r.iter().all(|t| matches!(t, Tok::Word(_)))) {
                            self.hit_end.set(true);
                        }
                        return Err("only 'parsed as number' or a type like 'parsed as click type' is supported".into());
                    }
                },
                None => {
                    self.hit_end.set(true);
                    return Err("expected 'parsed as number'".into());
                }
            }
        }
        e = self.replaced_postfix(e);
        e = self.in_case(e);
        e = self.util_postfix(e)?;
        e = self.w2e3_post(e)?;
        e = self.w6_postfix(e)?;
        // `now formatted as "yyyy-MM-dd"`
        while matches!(self.toks.get(self.pos), Some(t) if t.is_lit("formatted"))
            && matches!(e.ty(self.env), Ty::Date | Ty::Object)
        {
            self.pos += 1;
            e = self.date_format(e)?;
        }
        // `player's location`, `player's world`, ...
        while matches!(self.toks.get(self.pos), Some(Tok::Sym("'s"))) {
            let Some(Tok::Word(w)) = self.toks.get(self.pos + 1) else {
                if self.pos + 1 >= self.toks.len() {
                    self.hit_end.set(true);
                }
                return Err("expected a property after 's, like player's location".into());
            };
            let lw = w.to_ascii_lowercase();
            if (lw == "targeted" || lw == "target")
                && matches!(self.toks.get(self.pos + 2), Some(t) if t.is_lit("block"))
            {
                self.pos += 3;
                e = Expr::Blk(Box::new(crate::block_ops::BlockExpr::Targeted(e)));
                continue;
            }
            if let Some(info) = crate::item_meta::ItemInfo::from_word(&lw)
                && e.ty(self.env) == Ty::Item
            {
                self.pos += 2;
                e = Expr::ItemInfo(info, Box::new(e));
                continue;
            }
            if let Some((info, n)) = self.info_at(self.pos + 1) {
                self.pos += 1 + n;
                e = self.util_info(info, e)?;
                continue;
            }
            if let Some((prop, np)) = self.admin_prop_at(&lw, self.pos + 2) {
                self.pos = np;
                e = self.prop(prop, e)?;
                continue;
            }
            if self.pos + 2 >= self.toks.len() {
                self.hit_end.set(true);
            }
            if lw == "compass" && matches!(self.toks.get(self.pos + 2), Some(t) if t.is_lit("target")) {
                self.pos += 3;
                e = Self::x2(W2Op::CompassTarget, vec![e]);
                continue;
            }
            if let Some((f, n)) = crate::w2_expressions_3::of_head(self.toks, self.pos + 1)
                && matches!(e.ty(self.env), Ty::Player | Ty::Object)
            {
                self.pos += 1 + n;
                e = Expr::Ext(f, vec![e]);
                continue;
            }
            if let Some((ne, np)) = crate::w2_expressions_5::possessive(self.toks, self.pos + 1, &e) {
                e = ne;
                self.pos = np;
                continue;
            }
            if let Some((read, n)) = crate::w2_expressions_6::possessive(self.toks, self.pos + 1) {
                self.pos += 1 + n;
                e = Expr::W6(Box::new(crate::w2_expressions_6::W6::Read(read, Some(e))));
                continue;
            }
            let (prop, extra) = match crate::health_food::words_from(self.toks, self.pos + 1) {
                Some((p, n)) => (p, n - 1),
                None => match lw.as_str() {
                    "inventory" => {
                        return Err("use 'player's inventory contains <item>' as a condition".into());
                    }
                    _ => match self.toks.get(self.pos + 2) {
                        Some(Tok::Word(w2)) if crate::xp_level::two_word(&lw, &w2.to_ascii_lowercase()).is_some() => (
                            crate::xp_level::two_word(&lw, &w2.to_ascii_lowercase()).unwrap_or(Prop::Level),
                            1,
                        ),
                        _ => (
                            Prop::from_word(&lw).ok_or_else(|| format!("unknown property '{w}'"))?,
                            0,
                        ),
                    },
                },
            };
            self.pos += 2 + extra;
            if prop == Prop::Tool && matches!(self.peek(), Some(t) if t.is_lit("item")) {
                self.pos += 1;
            }
            e = self.prop(prop, e)?;
        }
        let meta_next = match self.toks.get(self.pos) {
            Some(Tok::Word(w)) => {
                w.eq_ignore_ascii_case("named")
                    || w.eq_ignore_ascii_case("with")
                    || w.eq_ignore_ascii_case("enchanted")
            }
            Some(_) => false,
            None => true,
        };
        if meta_next && e.ty(self.env) == Ty::Item {
            e = self.item_meta_postfix(e)?;
        }
        Ok(e)
    }

    /// `named "x"`, `with lore "a" and "b"`, `enchanted with sharpness 5 and unbreaking 3`,
    /// in any order, after an item.
    #[inline(never)]
    fn item_meta_postfix(&mut self, e: Expr) -> Result<Expr, String> {
        let mut meta = crate::item_meta::MetaExpr {
            base: e,
            name: None,
            lore: Vec::new(),
            enchants: Vec::new(),
        };
        let mut any = false;
        loop {
            let Some(Tok::Word(w)) = self.toks.get(self.pos) else {
                if self.pos >= self.toks.len() {
                    self.hit_end.set(true);
                }
                break;
            };
            let w = w.to_ascii_lowercase();
            if self.pos + 1 >= self.toks.len() {
                self.hit_end.set(true);
            }
            let next_is = |p: &Self, n: usize, lit: &str| matches!(p.toks.get(p.pos + n), Some(t) if t.is_lit(lit));
            if w == "named" || (w == "with" && next_is(self, 1, "name")) {
                self.pos += if w == "named" { 1 } else { 2 };
                meta.name = Some(self.primary()?);
            } else if w == "with" && next_is(self, 1, "lore") {
                self.pos += 2;
                meta.lore.push(self.primary()?);
                while matches!(self.toks.get(self.pos), Some(t) if t.is_lit("and") || t.is_lit(","))
                    && {
                        if self.pos + 1 >= self.toks.len() {
                            self.hit_end.set(true);
                        }
                        matches!(self.toks.get(self.pos + 1), Some(Tok::Str(_)))
                    }
                {
                    self.pos += 1;
                    meta.lore.push(self.primary()?);
                }
            } else if w == "enchanted" && next_is(self, 1, "with") {
                self.pos += 2;
                let mut first = true;
                loop {
                    let Some((id, level, used)) = crate::item_meta::enchant_at(self.toks, self.pos) else {
                        if first || self.pos >= self.toks.len() {
                            self.hit_end.set(true);
                        }
                        if first {
                            return Err("expected an enchantment after 'enchanted with', like sharpness 5".into());
                        }
                        break;
                    };
                    first = false;
                    self.pos += used;
                    meta.enchants.push(Expr::Enchant(Box::new((id.to_string(), level))));
                    if matches!(self.toks.get(self.pos), Some(t) if t.is_lit("and") || t.is_lit(","))
                        && crate::item_meta::enchant_at(self.toks, self.pos + 1).is_some()
                    {
                        self.pos += 1;
                    } else {
                        break;
                    }
                }
            } else {
                break;
            }
            any = true;
        }
        if !any {
            return Ok(meta.base);
        }
        Ok(Expr::ItemMeta(Box::new(meta)))
    }

    /// Heads of the expressions in `w2_expressions_3`. `None` when the words
    /// do not match one of them, with the position untouched.
    fn w2e3_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        use crate::w2_expressions_3::{ExtFn, IndexSel, MemKind};
        let start = self.pos - 1;
        let ext = |f: ExtFn, args: Vec<Expr>| Ok(Some(Expr::Ext(f, args)));
        if let Some((_, n)) = crate::w2_expressions_3::of_head(self.toks, start) {
            // More tokens could still complete `<words> of <player>`.
            if start + n >= self.toks.len() {
                self.hit_end.set(true);
            }
        }
        if let Some((f, n)) = crate::w2_expressions_3::of_head(self.toks, start)
            && matches!(self.toks.get(start + n), Some(t) if t.is_lit("of"))
        {
            self.pos = start + n + 1;
            let inner = self.primary()?;
            if !matches!(inner.ty(self.env), Ty::Player | Ty::Object) {
                return Err(format!("'{lw}' needs a player, got {}", inner.ty(self.env).name()));
            }
            return ext(f, vec![inner]);
        }
        let (toks, hit_end) = (self.toks, &self.hit_end);
        let next = |k: usize| {
            if start + k >= toks.len() {
                hit_end.set(true);
            }
            match toks.get(start + k) {
                Some(Tok::Word(w)) => w.to_ascii_lowercase(),
                _ => String::new(),
            }
        };
        match lw {
            "infinity" => {
                if next(1) == "value" {
                    self.pos += 1;
                }
                ext(ExtFn::Infinity(false), Vec::new())
            }
            "negative" if next(1) == "infinity" => {
                self.pos += 1;
                if next(2) == "value" {
                    self.pos += 1;
                }
                ext(ExtFn::Infinity(true), Vec::new())
            }
            "final" if next(1) == "damage" => {
                self.pos += 1;
                Ok(Some(Expr::Damage))
            }
            "free" | "max" | "maximum" | "total" if matches!(next(1).as_str(), "memory" | "ram") => {
                self.pos += 1;
                let k = match lw {
                    "free" => MemKind::Free,
                    "total" => MemKind::Total,
                    _ => MemKind::Max,
                };
                ext(ExtFn::Memory(k), Vec::new())
            }
            "player" | "entity" | "world" if next(1) == "from" && next(2) == "uuid" => {
                if lw == "world" {
                    return Err("worlds can't be looked up by UUID".into());
                }
                self.pos += 2;
                let u = self.primary_base()?;
                if !matches!(u.ty(self.env), Ty::Text | Ty::Object) {
                    return Err("'player from uuid' needs the UUID as text".into());
                }
                ext(ExtFn::PlayerFromUuid, vec![u])
            }
            "highest" | "top"
                if (lw == "top" && next(1) == "block" && next(2) == "at")
                    || (next(1) == "solid" && next(2) == "block" && next(3) == "at") =>
            {
                self.pos = start + if next(1) == "solid" { 4 } else { 3 };
                let at = self.primary()?;
                if !matches!(at.ty(self.env), Ty::Location | Ty::Player | Ty::Object) {
                    return Err("'highest solid block at' needs a location".into());
                }
                ext(ExtFn::Highest, vec![at])
            }
            "first" if next(1) == "empty" && next(2) == "slot" && matches!(next(3).as_str(), "of" | "in") => {
                self.pos = start + 4;
                if self.is_word(0, "inventory") && self.is_word(1, "of") {
                    self.pos += 2;
                }
                let who = self.primary_base()?;
                if matches!(self.toks.get(self.pos), Some(Tok::Sym("'s"))) && self.is_word(1, "inventory") {
                    self.pos += 2;
                }
                if !matches!(who.ty(self.env), Ty::Player | Ty::Object) {
                    return Err("'first empty slot of' needs a player".into());
                }
                ext(ExtFn::FirstEmptySlot, vec![who])
            }
            "first" | "last" | "indices" | "indexes" | "all" => {
                let (sel, kw) = match (lw, next(1).as_str()) {
                    ("first", "index") => (IndexSel::First, 2),
                    ("last", "index") => (IndexSel::Last, 2),
                    ("indices" | "indexes", _) => (IndexSel::All, 1),
                    ("all", "indices" | "indexes") => (IndexSel::All, 2),
                    _ => return Ok(None),
                };
                let mut at = kw + 1;
                if next(kw) != "of" {
                    return Ok(None);
                }
                if next(at) == "the" {
                    at += 1;
                }
                if next(at) != "value" {
                    return Ok(None);
                }
                self.pos = start + at + 1;
                let v = self.primary()?;
                if !self.is_word(0, "in") {
                    return Err("expected 'index of value <x> in {list::*}'".into());
                }
                self.pos += 1;
                let list = match self.peek().cloned() {
                    Some(Tok::Var(raw)) => {
                        self.pos += 1;
                        parse_varref(&raw, self.env)?
                    }
                    _ => return Err("'index of value' needs a list variable like {x::*}".into()),
                };
                if !list.is_list() {
                    return Err("'index of value' needs a list variable like {x::*}".into());
                }
                ext(ExtFn::IndexOf(sel), vec![v, Expr::Var(list)])
            }
            "exact" if next(1) == "item" && next(2) == "of" => {
                self.pos = start + 3;
                let it = self.primary()?;
                if !matches!(it.ty(self.env), Ty::Item | Ty::Object) {
                    return Err("'exact item of' needs an item".into());
                }
                ext(ExtFn::ExactItem, vec![it])
            }
            "id" if next(1) == "of" => {
                self.pos = start + 2;
                let it = self.primary()?;
                if !matches!(it.ty(self.env), Ty::Item | Ty::Object) {
                    return Err("'id of' needs an item".into());
                }
                ext(ExtFn::ItemId, vec![it])
            }
            "hex" if next(1) == "code" && next(2) == "of" => {
                self.pos = start + 3;
                let c = self.primary()?;
                if !matches!(c.ty(self.env), Ty::Text | Ty::Object) {
                    return Err("'hex code of' needs a color name or hex text".into());
                }
                ext(ExtFn::HexCode, vec![c])
            }
            _ => Ok(None),
        }
    }

    /// `<x> hashed with MD5`, `<timespan> formatted`.
    fn w2e3_post(&mut self, e: Expr) -> Result<Expr, String> {
        use crate::w2_expressions_3::ExtFn;
        if matches!(self.toks.get(self.pos), Some(t) if t.is_lit("hashed") || t.is_lit("hash"))
            && self.pos + 1 >= self.toks.len()
        {
            self.hit_end.set(true);
        }
        if matches!(self.toks.get(self.pos), Some(t) if t.is_lit("hashed") || t.is_lit("hash"))
            && matches!(self.toks.get(self.pos + 1), Some(t) if t.is_lit("with"))
        {
            return match crate::w2_expressions_3::alg_at(self.toks, self.pos + 2) {
                Some((alg, n)) => {
                    self.pos += 2 + n;
                    Ok(Expr::Ext(ExtFn::Hash(alg), vec![e]))
                }
                None => {
                    // `SHA - 256` is three tokens, so a short span may still complete.
                    if self.pos + 5 > self.toks.len() {
                        self.hit_end.set(true);
                    }
                    Err("unknown hash: use MD5, SHA-1, SHA-256 or SHA-512".into())
                }
            };
        }
        if matches!(self.toks.get(self.pos), Some(t) if t.is_lit("formatted")) && e.ty(self.env) == Ty::Timespan {
            self.pos += 1;
            return Ok(Expr::Ext(ExtFn::Clock, vec![e]));
        }
        Ok(e)
    }

    /// `<text> with [all|first] "a" replaced (with|by) "b"`. Leaves the
    /// position alone and returns `e` when the words after it do not fit.
    fn replaced_postfix(&mut self, e: Expr) -> Expr {
        if !matches!(self.toks.get(self.pos), Some(t) if t.is_lit("with")) {
            return e;
        }
        let save = self.pos;
        self.pos += 1;
        let mut all = true;
        if matches!(self.peek(), Some(t) if t.is_lit("all")) {
            self.pos += 1;
        } else if matches!(self.peek(), Some(t) if t.is_lit("first")) {
            self.pos += 1;
            all = false;
        }
        if let Ok(from) = self.primary_base()
            && matches!(self.peek(), Some(t) if t.is_lit("replaced"))
        {
            self.pos += 1;
            if matches!(self.peek(), Some(t) if t.is_lit("with") || t.is_lit("by")) {
                self.pos += 1;
                if let Ok(to) = self.primary_base() {
                    return Expr::Replaced(Box::new(e), Box::new(from), Box::new(to), all);
                }
            }
        }
        self.pos = save;
        e
    }

    /// After a number: `5 minutes`, `1 hour and 30 seconds`, `1 hour, 5 minutes`.
    fn span_literal(&mut self, first: f64) -> Option<Expr> {
        let unit_at = |p: usize| match self.toks.get(p) {
            Some(Tok::Word(w)) => crate::timespans::unit_ms(w),
            _ => None,
        };
        let mut total = crate::timespans::span_ms(first, unit_at(self.pos)?)?;
        let mut pos = self.pos + 1;
        loop {
            let mut p = pos;
            if matches!(self.toks.get(p), Some(t) if t.is_lit("and") || t.is_lit(",")) {
                p += 1;
            }
            // Running out of tokens mid-literal means a longer span may parse.
            if p + 1 >= self.toks.len() && p > pos {
                self.hit_end.set(true);
            }
            let (Some(Tok::Num(n)), Some(u)) = (self.toks.get(p), unit_at(p + 1)) else {
                break;
            };
            total = total.checked_add(crate::timespans::span_ms(*n, u)?)?;
            pos = p + 2;
        }
        self.pos = pos;
        Some(Expr::Span(total))
    }

    /// `minutes of`, `total seconds of`.
    fn span_part_ahead(&self, first: &str) -> bool {
        let of_at = if first == "total" {
            if !matches!(self.toks.get(self.pos), Some(Tok::Word(w)) if crate::timespans::unit_ms(w).is_some())
            {
                return false;
            }
            self.pos + 1
        } else {
            self.pos
        };
        matches!(self.toks.get(of_at), Some(t) if t.is_lit("of"))
    }

    fn span_part(&mut self, first: &str) -> Result<Expr, String> {
        let (total, unit) = if first == "total" {
            let Some(Tok::Word(w)) = self.toks.get(self.pos).cloned() else {
                return Err("expected a unit after 'total'".into());
            };
            self.pos += 1;
            (true, w)
        } else {
            (false, first.to_string())
        };
        let ms = crate::timespans::unit_ms(&unit).ok_or("unknown time unit")?;
        self.pos += 1; // `of`
        let inner = self.primary()?;
        if !matches!(inner.ty(self.env), Ty::Timespan | Ty::Object) {
            return Err(format!(
                "'{unit} of' needs a timespan, got {}",
                inner.ty(self.env).name()
            ));
        }
        Ok(Expr::SpanPart(ms, total, Box::new(inner)))
    }

    /// `slot 3 of ...inventory`: a number or variable, then an inventory somewhere after.
    fn slot_head(&self) -> bool {
        matches!(self.toks.get(self.pos), Some(Tok::Num(_) | Tok::Var(_) | Tok::Sym("(")))
    }

    /// `clicked slot`, `clicked item`, `click type` in `on inventory click`, and
    /// how many tokens follow the first word.
    fn inv_event_head(&self, first: &str) -> Option<(crate::inventories::InvExpr, usize)> {
        use crate::inventories::InvExpr;
        let next = match self.toks.get(self.pos) {
            Some(Tok::Word(w)) => w.to_ascii_lowercase(),
            _ => String::new(),
        };
        match (first, next.as_str()) {
            ("clicked", "slot") => Some((InvExpr::ClickedSlot, 1)),
            ("clicked", "item") => Some((InvExpr::ClickedItem, 1)),
            ("click", "type") => Some((InvExpr::ClickType, 1)),
            ("event-slot", _) => Some((InvExpr::ClickedSlot, 0)),
            ("event-item", _) => Some((InvExpr::ClickedItem, 0)),
            _ => None,
        }
    }

    /// `slot N of <player>'s [current] inventory` (the `slot` word is read).
    fn slot_expr(&mut self) -> Result<Expr, String> {
        use crate::inventories::{InvExpr, InvTarget, inv_target, inv_target_len};
        let slot = self.primary()?;
        if !Ty::Number.accepts(slot.ty(self.env)) {
            return Err("the slot must be a number".into());
        }
        if !matches!(self.peek(), Some(t) if t.is_lit("of")) {
            return Err("expected 'slot <number> of <player>'s inventory'".into());
        }
        self.pos += 1;
        let toks = self.toks;
        let rest = toks.get(self.pos..).unwrap_or_default();
        let inv = if let Some(len) = inv_target_len(rest) {
            let inv = inv_target(rest.get(..len).unwrap_or_default(), self.env)
                .ok_or("expected a player before 's inventory")?;
            self.pos += len;
            inv
        } else {
            let mut current = false;
            if !rest.iter().any(|t| t.is_lit("inventory")) {
                // A shorter token list may still grow into one.
                self.hit_end.set(true);
            }
            if matches!(self.peek(), Some(t) if t.is_lit("the")) {
                self.pos += 1;
            }
            if matches!(self.peek(), Some(t) if t.is_lit("current")) {
                current = true;
                self.pos += 1;
            }
            if !(matches!(self.peek(), Some(t) if t.is_lit("inventory"))
                && matches!(self.peek_at(1), Some(t) if t.is_lit("of")))
            {
                return Err("expected 'slot <number> of <player>'s inventory'".into());
            }
            self.pos += 2;
            let who = self.primary()?;
            if !matches!(who.ty(self.env), Ty::Player | Ty::Object) {
                return Err("the inventory belongs to a player".into());
            }
            InvTarget { who, current }
        };
        Ok(Expr::Inv(Box::new(InvExpr::Slot { slot, inv })))
    }

    /// The admin properties (`display name`, `nickname`, `tab list name`,
    /// `ping`, `ip [address]`) when their words start at `at`, with the
    /// position after them.
    fn admin_prop_at(&self, first: &str, at: usize) -> Option<(Prop, usize)> {
        // Running out of words may just mean the span is cut short: say so,
        // so the matcher tries a longer one.
        let lit = |i: usize, w: &str| {
            if i >= self.toks.len() {
                self.hit_end.set(true);
            }
            matches!(self.toks.get(i), Some(t) if t.is_lit(w))
        };
        match first {
            "display" if lit(at, "name") => Some((Prop::DisplayName, at + 1)),
            "tab" if lit(at, "list") && lit(at + 1, "name") => Some((Prop::TabName, at + 2)),
            "nickname" => Some((Prop::DisplayName, at)),
            "ping" => Some((Prop::Ping, at)),
            "ip" if lit(at, "address") => Some((Prop::Ip, at + 1)),
            "ip" => Some((Prop::Ip, at)),
            _ => None,
        }
    }

    /// `lore of`, `enchantments of`, `item amount of`, `item type of` after the first word.
    /// Returns what it reads and the number of extra words (`item type` is one extra).
    fn item_info_head(&self, first: &str) -> Option<(crate::item_meta::ItemInfo, usize)> {
        use crate::item_meta::ItemInfo;
        let is_of = |n: usize| {
            if self.pos + n >= self.toks.len() {
                self.hit_end.set(true);
            }
            matches!(self.toks.get(self.pos + n), Some(t) if t.is_lit("of"))
        };
        if first == "item" {
            let Some(Tok::Word(w)) = self.toks.get(self.pos) else {
                self.hit_end.set(true);
                return None;
            };
            let info = match w.to_ascii_lowercase().as_str() {
                "type" => ItemInfo::Type,
                "amount" => ItemInfo::Amount,
                _ => return None,
            };
            return is_of(1).then_some((info, 1));
        }
        let info = ItemInfo::from_word(first)?;
        is_of(0).then_some((info, 0))
    }

    pub(crate) fn prop(&self, prop: Prop, of: Expr) -> Result<Expr, String> {
        let t = of.ty(self.env);
        if !prop.takes(t) {
            return Err(format!("can't take the {prop:?} of a {}", t.name()).to_lowercase());
        }
        if prop == Prop::Name && t == Ty::Player {
            return Ok(Expr::NameOf(Box::new(of)));
        }
        Ok(Expr::Prop(prop, Box::new(of)))
    }

    /// `max health of`, `food level of`, ... starting at the word just read.
    /// Marks the end as hit when more tokens could complete it.
    fn hf_head(&self) -> Option<(Prop, usize)> {
        let start = self.pos - 1;
        if start + 2 >= self.toks.len() {
            self.hit_end.set(true);
        }
        let (p, n) = crate::health_food::words_from(self.toks, start)?;
        matches!(self.toks.get(start + n), Some(t) if t.is_lit("of")).then_some((p, n))
    }

    /// `<word> of <value>` for the properties above.
    /// `6:30`, `6:30 pm`, `6 am`: a time of day as ticks (tick 0 is 6:00).
    fn clock_literal(&mut self, hour: f64) -> Option<Expr> {
        let (minute, mut end) = match (self.toks.get(self.pos), self.toks.get(self.pos + 1)) {
            (Some(Tok::Sym(":")), Some(Tok::Num(m))) => (*m, self.pos + 2),
            (Some(Tok::Sym(":")), None) => {
                self.hit_end.set(true);
                return None;
            }
            _ => (0.0, self.pos),
        };
        let ampm = match self.toks.get(end) {
            Some(Tok::Word(w)) if w.eq_ignore_ascii_case("am") => Some(false),
            Some(Tok::Word(w)) if w.eq_ignore_ascii_case("pm") => Some(true),
            _ => None,
        };
        if end == self.pos && ampm.is_none() {
            return None;
        }
        let ticks = crate::weather_time::clock_ticks(hour, minute, ampm)?;
        if ampm.is_some() {
            end += 1;
        }
        self.pos = end;
        Some(Expr::Num(ticks))
    }

    /// `time [in world W]`, `weather [in world W]`, `difficulty ...`,
    /// `gamerule <name> [in world W]`. `kw` is already consumed.
    fn wt_expr(&mut self, kw: &str) -> Result<Expr, String> {
        use crate::weather_time::{WtExpr, WtKind};
        let kind = match kw {
            "time" => {
                // `time of day`
                if matches!(self.toks.get(self.pos), Some(t) if t.is_lit("of"))
                    && matches!(self.toks.get(self.pos + 1), Some(t) if t.is_lit("day"))
                {
                    self.pos += 2;
                }
                WtKind::Time
            }
            "weather" => WtKind::Weather,
            "difficulty" => WtKind::Difficulty,
            _ => {
                let raw = match self.peek() {
                    Some(Tok::Word(w) | Tok::Str(w)) => w.clone(),
                    _ => return Err("expected a game rule name, like 'gamerule keepInventory'".into()),
                };
                let Some(rule) = crate::weather_time::rule_name(&raw) else {
                    return Err(format!("unknown game rule '{raw}'"));
                };
                self.pos += 1;
                WtKind::Gamerule(rule.to_string())
            }
        };
        let mut world = None;
        if matches!(self.peek(), Some(t) if t.is_lit("in") || t.is_lit("of")) {
            self.pos += 1;
            let w = self.primary()?;
            if !matches!(w.ty(self.env), Ty::World | Ty::Text | Ty::Player | Ty::Object) {
                return Err(format!("expected a world, got {}", w.ty(self.env).name()));
            }
            world = Some(w);
        }
        Ok(Expr::Wt(Box::new(WtExpr { kind, world })))
    }

    fn prop_of(&mut self, prop: Prop) -> Result<Expr, String> {
        // already past the property word
        if prop == Prop::Tool && matches!(self.peek(), Some(t) if t.is_lit("item")) {
            self.pos += 1;
        }
        if !matches!(self.peek(), Some(t) if t.is_lit("of")) {
            return Err("expected 'of'".into());
        }
        self.pos += 1;
        let inner = self.primary()?;
        self.prop(prop, inner)
    }

    /// Longest run of words that names an item, like `diamond sword`. Plural
    /// `diamonds` and `torches` work too.
    fn item(&mut self, first: &str) -> Option<Expr> {
        let lower = first.to_ascii_lowercase();
        if !crate::items::could_start(&lower) {
            return None;
        }
        let start = self.pos - 1;
        let mut words = vec![lower];
        let mut i = self.pos;
        while let Some(Tok::Word(w)) = self.toks.get(i) {
            if words.len() >= 5 {
                break;
            }
            words.push(w.to_ascii_lowercase());
            i += 1;
        }
        // More words after the end could make a longer item name.
        if i >= self.toks.len() && words.len() < 5 {
            self.hit_end.set(true);
        }
        for n in (1..=words.len()).rev() {
            let joined = words.get(..n).unwrap_or_default().join("_");
            let mut candidates = vec![joined.clone()];
            if let Some(s) = joined.strip_suffix("es") {
                candidates.push(s.to_string());
            }
            if let Some(s) = joined.strip_suffix('s') {
                candidates.push(s.to_string());
            }
            if let Some(id) = candidates.into_iter().find(|c| crate::items::is_known(c)) {
                self.pos = start + n;
                return Some(Expr::Item { id, count: 1 });
            }
        }
        None
    }

    /// Does a bound expression start here (`bound ...`, `owners of bound ...`)?
    fn bound_head(&self, lw: &str) -> bool {
        let at = |n: usize, lit: &str| matches!(self.toks.get(self.pos + n), Some(t) if t.is_lit(lit));
        match lw {
            "bound" | "event-bound" => true,
            "bounds" => at(0, "at"),
            "all" => at(0, "bounds"),
            "owners" | "members" | "world" | "volume" => at(0, "of") && at(1, "bound"),
            "lesser" | "greater" => at(0, "corner") && at(1, "of"),
            "flag" => self.toks.iter().skip(self.pos).any(|t| t.is_lit("bound")),
            _ => false,
        }
    }

    fn bound_operand(&mut self) -> Result<Expr, String> {
        let e = self.primary()?;
        match e.ty(self.env) {
            Ty::Text | Ty::Object => Ok(e),
            t => Err(format!("expected a bound, got a {}", t.name())),
        }
    }

    fn bound_expr(&mut self, lw: &str) -> Result<Expr, String> {
        use crate::skbee_bound::BoundExpr as B;
        let b = match lw {
            "event-bound" => {
                if !self.env.has_bound || self.env.in_function {
                    return Err(format!(
                        "'event-bound' only exists in 'on bound enter' and 'on bound exit', not in {}",
                        self.env.trigger_name
                    ));
                }
                B::Event
            }
            "all" => {
                self.pos += 1;
                B::All
            }
            "bounds" | "bound" if matches!(self.peek(), Some(t) if t.is_lit("at")) => {
                self.pos += 1;
                let loc = self.primary()?;
                if !matches!(loc.ty(self.env), Ty::Location | Ty::Player | Ty::Object) {
                    return Err("'bound at' needs a location or a player".into());
                }
                if lw == "bound" { B::At(loc) } else { B::BoundsAt(loc) }
            }
            "bound" => {
                self.pos = crate::skbee_bound::skip_id_words(self.toks, self.pos);
                B::Named(self.bound_operand()?)
            }
            "owners" | "members" => {
                self.pos += 1;
                B::People(lw == "owners", self.bound_operand()?)
            }
            "lesser" | "greater" => {
                self.pos += 2;
                B::Corner(lw == "greater", self.bound_operand()?)
            }
            "world" => {
                self.pos += 1;
                B::WorldOf(self.bound_operand()?)
            }
            "volume" => {
                self.pos += 1;
                B::Volume(self.bound_operand()?)
            }
            _ => {
                let flag = self.primary()?;
                if !matches!(self.peek(), Some(t) if t.is_lit("of")) {
                    return Err("expected 'flag \"name\" of bound <id>'".into());
                }
                self.pos += 1;
                B::Flag(flag, self.bound_operand()?)
            }
        };
        Ok(Expr::Bound(Box::new(b)))
    }

    pub(crate) fn primary_base(&mut self) -> Result<Expr, String> {
        let Some(tok) = self.peek().cloned() else {
            return Err("missing expression".into());
        };
        self.pos += 1;
        match tok {
            Tok::Num(n) => {
                if let Some(t) = self.clock_literal(n) {
                    return Ok(t);
                }
                if let Some(e) = self.w2e7_num_of(n) {
                    return Ok(e);
                }
                if let Some(span) = self.span_literal(n) {
                    return Ok(span);
                }
                if let Some((e, np)) = crate::w2_expressions_1::angle_suffix(self.toks, self.pos, n) {
                    self.pos = np;
                    return Ok(e);
                }
                if let Some(off) = self.x2_offset(n)? {
                    return Ok(off);
                }
                // `2 diamonds`: an amount of an item.
                if self.pos >= self.toks.len() {
                    self.hit_end.set(true);
                }
                if let Some(Tok::Word(w)) = self.toks.get(self.pos).cloned()
                    && n >= 1.0
                    && n.fract() == 0.0
                    && n <= f64::from(u32::MAX)
                {
                    let save = self.pos;
                    self.pos += 1;
                    if let Some(Expr::Item { id, .. }) = self.item(&w) {
                        return Ok(Expr::Item { id, count: n as u32 });
                    }
                    self.pos = save;
                }
                Ok(Expr::Num(n))
            }
            Tok::Str(s) => Ok(Expr::Text(parse_template(&s, self.env)?)),
            Tok::Var(v) => {
                let var = parse_varref(&v, self.env)?;
                if var.is_list() && matches!(self.peek(), Some(t) if t.is_lit("where")) {
                    return self.list_expr(self.pos - 1);
                }
                Ok(Expr::Var(var))
            }
            Tok::Sym("(") => {
                let e = self.sum()?;
                if !self.eat_sym(")") {
                    return Err("missing ')'".into());
                }
                Ok(e)
            }
            Tok::Word(w) => {
                let lw = w.to_ascii_lowercase();
                if let Some(e) = self.rtw_head(&lw)? {
                    return Ok(e);
                }
                if let Some(e) = self.t1_head(&lw)? {
                    return Ok(e);
                }
                if let Some((e, end)) = crate::entities::parse_expr_at(self.toks, self.pos - 1, self.env, &self.hit_end)? {
                    self.pos = end;
                    return Ok(e);
                }
                if let Some(ev) = self.env.ext_event
                    && !self.env.in_function
                {
                    let mut more = false;
                    let rest = self.toks.get(self.pos..).unwrap_or(&[]);
                    if let Some((e, used)) = crate::w2_events_2::parse_value(ev, &lw, rest, &mut more) {
                        self.pos += used;
                        return Ok(e);
                    }
                    if more {
                        self.hit_end.set(true);
                    }
                }
                if crate::w2_expressions_1::starts(&lw) {
                    // The sub-parsers can't tell us whether they ran off the end,
                    // so say "maybe" and let the pattern matcher try longer spans.
                    let before = self.hit_end.replace(true);
                    if let Some((w2, np)) = crate::w2_expressions_1::parse(self.toks, self.pos, &lw, self.env)? {
                        self.pos = np;
                        self.hit_end.set(before);
                        return Ok(Expr::W2(Box::new(w2)));
                    }
                }
                if let Some(e) = self.x2_fn(&lw)? {
                    return Ok(e);
                }
                if let Some(e) = self.w2e3_head(&lw)? {
                    return Ok(e);
                }
                if let Some(e) = self.w4(&lw)? {
                    return Ok(e);
                }
                if let Some(e) = self.w2e7_head(&lw)? {
                    return Ok(e);
                }
                if let Some(e) = self.w6_word(&lw)? {
                    return Ok(e);
                }
                if let Some(e) = self.text_fn(&lw)? {
                    return Ok(e);
                }
                if let Some(e) = self.server_misc(&lw)? {
                    return Ok(e);
                }
                if let Some(e) = self.block_expr(&lw)? {
                    return Ok(e);
                }
                if self.env.event_vals != 0
                    && !self.env.in_function
                    && let Some((v, n)) =
                        crate::events_items::parse_val(self.env.event_vals, &lw, self.toks.get(self.pos..).unwrap_or(&[]))
                {
                    self.pos += n;
                    return Ok(Expr::EventVal(v));
                }
                if let Some(e) = self.comp_expr(&lw)? {
                    return Ok(e);
                }
                if crate::skbee_scoreboard_teams::is_head(&lw) {
                    let save = self.pos;
                    match crate::skbee_scoreboard_teams::parse_expr_at(self, &lw) {
                        Some(r) => return r,
                        None => self.pos = save,
                    }
                }
                if let Some(h) = crate::skbee_nbt::head(self.toks, self.pos - 1) {
                    self.pos = h.after;
                    if self.pos >= self.toks.len() {
                        self.hit_end.set(true);
                    }
                    let operand = self.primary()?;
                    return crate::skbee_nbt::build(&h, operand, self.env);
                }
                if let Some(e) = self.util_prefix(&lw)? {
                    return Ok(e);
                }
                match crate::placeholders::parse_word(self.toks, self.pos, self.env, &lw) {
                    Ok(Some((e, end))) => {
                        self.pos = end;
                        return Ok(e);
                    }
                    Ok(None) => {}
                    Err(m) => {
                        self.hit_end.set(true);
                        return Err(m);
                    }
                }
                match crate::w2_expressions_5::parse_word(self.toks, self.pos, self.env, &lw) {
                    Ok(Some((e, end))) => {
                        self.pos = end;
                        return Ok(e);
                    }
                    Ok(None) => {
                        if crate::w2_expressions_5::may_continue(&lw) && self.pos + 4 > self.toks.len() {
                            self.hit_end.set(true);
                        }
                    }
                    Err(m) => {
                        self.hit_end.set(true);
                        return Err(m);
                    }
                }
                match lw.as_str() {
                    _ if matches!(self.peek(), Some(t) if t.is_lit("("))
                        && !self.has_function(&lw)
                        && crate::w4_functions::lookup(&lw).is_some() =>
                    {
                        self.w4_function(&lw)
                    }
                    "the" => self.primary(),
                    _ if self.bound_head(&lw) => self.bound_expr(&lw),
                    _ if crate::gamemode::is_mode_word(&lw) => Ok(Expr::Text(vec![Part::Lit(lw)])),
                    "game"
                        if matches!(self.peek(), Some(t) if t.is_lit("mode"))
                            && matches!(self.toks.get(self.pos + 1), Some(t) if t.is_lit("of")) =>
                    {
                        self.pos += 1;
                        self.prop_of(Prop::Gamemode)
                    }
                    "colored" | "coloured" | "uncolored" | "uncoloured" | "unformatted" => {
                        if self.pos >= self.toks.len() {
                            self.hit_end.set(true);
                        }
                        let inner = self.primary()?;
                        Ok(Expr::Color(!lw.starts_with('u'), Box::new(inner)))
                    }
                    "time" if matches!(self.peek(), Some(t) if t.is_lit("since") || t.is_lit("until")) => {
                        let since = matches!(self.peek(), Some(t) if t.is_lit("since"));
                        self.pos += 1;
                        let d = self.primary()?;
                        if !matches!(d.ty(self.env), Ty::Date | Ty::Object) {
                            return Err(format!(
                                "'time since' needs a date, got {}",
                                d.ty(self.env).name()
                            ));
                        }
                        Ok(Expr::Since(Box::new(d), since))
                    }
                    "time" | "weather" | "difficulty" | "gamerule" => self.wt_expr(&lw),
                    "total" | "tick" | "ticks" | "millisecond" | "milliseconds" | "second" | "seconds"
                    | "minute" | "minutes" | "hour" | "hours" | "day" | "days" | "week" | "weeks"
                    | "month" | "months" | "year" | "years"
                        if self.span_part_ahead(&lw) =>
                    {
                        self.span_part(&lw)
                    }
                    "true" | "yes" => Ok(Expr::Bool(true)),
                    "false" | "no" => Ok(Expr::Bool(false)),
                    "player" if self.env.in_function => {
                        Err("'player' doesn't exist inside a function; pass the player as a parameter".into())
                    }
                    "player" | "victim" if self.env.no_player => Err(format!(
                        "'{lw}' doesn't exist in {}: nobody started it",
                        self.env.trigger_name
                    )),
                    "sender" | "executor" | "console" if self.env.in_function => Err(format!(
                        "'{lw}' doesn't exist inside a function; pass it as a parameter"
                    )),
                    "sender" | "executor" => Ok(Expr::Sender),
                    "console" => Ok(Expr::Console),
                    "command"
                        if !self.env.in_function
                            && matches!(self.peek(), Some(t) if t.is_lit("sender") || t.is_lit("executor")) =>
                    {
                        self.pos += 1;
                        Ok(Expr::Sender)
                    }
                    "me" | "myself" if self.env.in_function => {
                        Err("'me' doesn't exist inside a function; pass the player as a parameter".into())
                    }
                    "me" | "myself" if self.env.no_player => Err(format!(
                        "'{lw}' doesn't exist in {}: nobody started it",
                        self.env.trigger_name
                    )),
                    "me" | "myself" => Ok(Expr::Player),
                    "player" => Ok(Expr::Player),
                    "victim" if self.env.has_damage || self.env.has_attacker => Ok(Expr::Player),
                    "damage" | "event-number" if self.env.has_damage && !self.env.in_function => Ok(Expr::Damage),
                    "event-text" if self.env.has_message && !self.env.in_function => Ok(Expr::Message),
                    "attacker" | "damager" if self.env.has_attacker && !self.env.in_function => {
                        Ok(Expr::Attacker)
                    }
                    "slot" if self.slot_head() => self.slot_expr(),
                    "clicked" | "click" | "event-slot" | "event-item"
                        if self.env.has_inv && !self.env.in_function && self.inv_event_head(&lw).is_some() =>
                    {
                        let (e, n) = self.inv_event_head(&lw).ok_or("unknown inventory value")?;
                        self.pos += n;
                        Ok(Expr::Inv(Box::new(e)))
                    }
                    w if w.starts_with("event-")
                        && !self.env.in_function
                        && crate::w2_events_1::field_index(self.env.ev_fields, w).is_some() =>
                    {
                        let i = crate::w2_events_1::field_index(self.env.ev_fields, w).map_or(0, |f| f.0);
                        Ok(Expr::ExtVal(i))
                    }
                    w if w.starts_with("event-") && !self.env.in_function && self.env.ext4.is_some() => {
                        let ev = self.env.ext4.unwrap_or(crate::w2_events_4::ExtEvent::WorldInit);
                        match ev.slot_index(&w["event-".len()..]) {
                            Some(i) => Ok(Expr::Ev4Val(i as u8)),
                            None => Err(format!(
                                "'on {}' has no value '{w}' (it has: {})",
                                ev.name(),
                                if ev.slots().is_empty() {
                                    "none".to_string()
                                } else {
                                    ev.slots().iter().map(|(n, _)| format!("event-{n}")).collect::<Vec<_>>().join(", ")
                                }
                            )),
                        }
                    }
                    "clicked" | "event-block"
                        if self.env.has_block
                            && !self.env.in_function
                            && (lw == "event-block"
                                || matches!(self.peek(), Some(t) if t.is_lit("block"))) =>
                    {
                        if lw == "clicked" {
                            self.pos += 1;
                        }
                        Ok(Expr::ClickedBlock)
                    }
                    "message" => {
                        if self.env.has_message && !self.env.in_function {
                            Ok(Expr::Message)
                        } else {
                            Err(format!(
                                "'message' only exists in 'on chat', not in {}",
                                self.env.trigger_name
                            ))
                        }
                    }
                    "last" | "caught" | "runtime"
                        if crate::structures::error_phrase(&lw, self.toks.get(self.pos..).unwrap_or_default())
                            .is_some()
                            || crate::structures::error_phrase_start(
                                &lw,
                                self.toks.get(self.pos..).unwrap_or_default(),
                            ) =>
                    {
                        let rest = self.toks.get(self.pos..).unwrap_or_default();
                        match crate::structures::error_phrase(&lw, rest) {
                            Some(n) => {
                                self.pos += n;
                                Ok(crate::structures::caught_error_expr())
                            }
                            None => {
                                self.hit_end.set(true);
                                Err("expected 'last runtime error'".into())
                            }
                        }
                    }
                    "statistic" if !matches!(self.peek(), Some(t) if t.is_lit("of")) => {
                        let name = self.primary()?;
                        if !matches!(self.peek(), Some(t) if t.is_lit("of")) {
                            return Err("expected 'statistic \"custom:jump\" of <player>'".into());
                        }
                        self.pos += 1;
                        let who = self.primary()?;
                        Ok(Expr::Skb(Box::new(crate::skbee_recipes_advancements::SkbExpr::Stat(name, who))))
                    }
                    "advancement"
                        if self.env.trigger_name.contains("advancement done") && !self.env.in_function =>
                    {
                        if matches!(self.peek(), Some(t) if t.is_lit("id")) {
                            self.pos += 1;
                        }
                        Ok(Expr::Skb(Box::new(crate::skbee_recipes_advancements::SkbExpr::EventId)))
                    }
                    "recipe"
                        if self.env.trigger_name.contains("recipe discover") && !self.env.in_function =>
                    {
                        if matches!(self.peek(), Some(t) if t.is_lit("id")) {
                            self.pos += 1;
                        }
                        Ok(Expr::Skb(Box::new(crate::skbee_recipes_advancements::SkbExpr::EventId)))
                    }
                    "now" => Ok(Expr::Date(DateOp::Now, Vec::new())),
                    "unix"
                        if matches!(self.peek(), Some(t) if t.is_lit("timestamp"))
                            && matches!(self.peek_at(1), Some(t) if t.is_lit("of")) =>
                    {
                        self.pos += 2;
                        let d = self.primary()?;
                        if !matches!(d.ty(self.env), Ty::Date | Ty::Object) {
                            return Err("'unix timestamp of' needs a date, like 'unix timestamp of now'".into());
                        }
                        Ok(Expr::Date(DateOp::ToUnix, vec![d]))
                    }
                    "unix"
                        if matches!(self.peek(), Some(t) if t.is_lit("date"))
                            && matches!(self.peek_at(1), Some(t) if t.is_lit("of")) =>
                    {
                        self.pos += 2;
                        self.date_from_unix()
                    }
                    "date"
                        if matches!(self.peek(), Some(t) if t.is_lit("from"))
                            && matches!(self.peek_at(1), Some(t) if t.is_lit("unix"))
                            && matches!(self.peek_at(2), Some(t) if t.is_lit("timestamp")) =>
                    {
                        self.pos += 3;
                        self.date_from_unix()
                    }
                    "formatted" => {
                        // `formatted now as "..."` / `formatted {date}`
                        let d = self.primary_base()?;
                        let as_next = matches!(self.peek(), Some(t) if t.is_lit("as"));
                        match d.ty(self.env) {
                            Ty::Date => self.date_format(d),
                            Ty::Object | Ty::Text if as_next => self.date_format(d),
                            Ty::Object | Ty::Text => Ok(Expr::Color(true, Box::new(d))),
                            _ => Err("'formatted' needs a date or text, like 'formatted now'".into()),
                        }
                    }
                    "arg" | "argument" => self.arg(1),
                    "join" if matches!(self.peek(), Some(Tok::Var(_))) => {
                        let Some(Tok::Var(v)) = self.peek().cloned() else {
                            return Err("expected 'join {list::*} with \", \"'".into());
                        };
                        self.pos += 1;
                        let var = parse_varref(&v, self.env)?;
                        if !var.is_list() {
                            return Err("'join' needs a list variable like {x::*}".into());
                        }
                        if !matches!(self.peek(), Some(t) if t.is_lit("with") || t.is_lit("by")) {
                            return Err("expected 'join {list::*} with \", \"'".into());
                        }
                        self.pos += 1;
                        let sep = self.primary_base()?;
                        Ok(Expr::Join(var, Box::new(sep)))
                    }
                    "split" if self.peek().is_some() && !matches!(self.peek(), Some(t) if t.is_lit("(")) => {
                        let text = self.primary_base()?;
                        if !matches!(self.peek(), Some(t) if t.is_lit("at") || t.is_lit("by")) {
                            return Err("expected 'split <text> at <separator>'".into());
                        }
                        self.pos += 1;
                        let sep = self.primary_base()?;
                        Ok(Expr::Split(Box::new(text), Box::new(sep)))
                    }
                    "length" if matches!(self.peek(), Some(t) if t.is_lit("of")) => {
                        self.pos += 1;
                        let inner = self.primary_base()?;
                        Ok(Expr::Length(Box::new(inner)))
                    }
                    "size" | "amount" | "count" | "number"
                        if matches!(self.peek(), Some(t) if t.is_lit("of"))
                            && matches!(self.toks.get(self.pos + 1), Some(t) if t.is_lit("characters") || t.is_lit("letters")) =>
                    {
                        self.pos += 2;
                        if !matches!(self.peek(), Some(t) if t.is_lit("in") || t.is_lit("of")) {
                            return Err(format!("expected '{lw} of characters in <text>'"));
                        }
                        self.pos += 1;
                        let inner = self.primary_base()?;
                        Ok(Expr::Length(Box::new(inner)))
                    }
                    "number" if matches!(self.peek(), Some(t) if t.is_lit("of")) => self.size_of(&lw),
                    "indices" | "indexes" if matches!(self.peek(), Some(t) if t.is_lit("of")) => {
                        Ok(Expr::Indices(self.list_after_of(&lw)?))
                    }
                    "first" | "last" if matches!(self.peek(), Some(t) if t.is_lit("element")) => {
                        self.pos += 1;
                        let sel = if lw == "first" { ElemSel::First } else { ElemSel::Last };
                        Ok(Expr::Elem(sel, self.list_after_of(&format!("{lw} element"))?))
                    }
                    "element" if !matches!(self.peek(), Some(t) if t.is_lit("of")) => {
                        let n = self.unary()?;
                        if !Ty::Number.accepts(n.ty(self.env)) {
                            return Err("'element N of' needs a number".into());
                        }
                        Ok(Expr::Elem(ElemSel::Nth(Box::new(n)), self.list_after_of("element N")?))
                    }
                    "size" | "amount" | "count" => {
                        if lw == "amount" && matches!(self.peek(), Some(t) if t.is_lit("of")) {
                            let save = self.pos;
                            self.pos += 1;
                            if let Ok(inner) = self.primary()
                                && inner.ty(self.env) == Ty::Item
                            {
                                return Ok(Expr::ItemInfo(crate::item_meta::ItemInfo::Amount, Box::new(inner)));
                            }
                            // `amount of player` may still become `amount of player's tool`.
                            if self.pos >= self.toks.len() {
                                self.hit_end.set(true);
                            }
                            self.pos = save;
                        }
                        self.size_of(&lw)
                    }
                    "sorted" | "reversed" | "shuffled" if self.peek().is_some() => {
                        self.list_expr(self.pos - 1)
                    }
                    "random" if matches!(self.peek(), Some(Tok::Num(_) | Tok::Var(_))) => {
                        self.list_expr(self.pos - 1)
                    }
                    "sum" | "average" | "avg" | "min" | "max" | "minimum" | "maximum"
                        if matches!(self.peek(), Some(t) if t.is_lit("of")) =>
                    {
                        self.pos += 1;
                        let agg = match lw.as_str() {
                            "sum" => Agg::Sum,
                            "average" | "avg" => Agg::Avg,
                            "min" | "minimum" => Agg::Min,
                            _ => Agg::Max,
                        };
                        self.try_agg(agg).unwrap_or_else(|| Err("missing list".into()))
                    }
                    "location"
                        if matches!(self.peek(), Some(t) if t.is_lit("("))
                            && !self.has_function("location") =>
                    {
                        self.pos += 1;
                        let x = self.sum()?;
                        let mut parts = vec![x];
                        while self.eat_sym(",") {
                            parts.push(self.sum()?);
                        }
                        if !self.eat_sym(")") {
                            return Err("missing ')' after location(x, y, z)".into());
                        }
                        if !(3..=4).contains(&parts.len()) {
                            return Err("location(x, y, z) or location(x, y, z, world)".into());
                        }
                        for p in parts.iter().take(3) {
                            if !Ty::Number.accepts(p.ty(self.env)) {
                                return Err("location() needs numbers for x, y and z".into());
                            }
                        }
                        let mut it = parts.into_iter();
                        let (Some(x), Some(y), Some(z)) = (it.next(), it.next(), it.next()) else {
                            return Err("location(x, y, z)".into());
                        };
                        let w = it.next().map(Box::new);
                        Ok(Expr::Loc(Box::new(x), Box::new(y), Box::new(z), w))
                    }
                    "world" if matches!(self.peek(), Some(Tok::Str(_))) => {
                        let name = self.primary()?;
                        Ok(Expr::WorldNamed(Box::new(name)))
                    }
                    "random" if matches!(self.peek(), Some(t) if t.is_lit("element")) => {
                        self.pos += 1;
                        if !matches!(self.peek(), Some(t) if t.is_lit("of")) {
                            return Err("expected 'random element of {list::*}'".into());
                        }
                        self.pos += 1;
                        match self.peek().cloned() {
                            Some(Tok::Var(v)) => {
                                self.pos += 1;
                                let var = parse_varref(&v, self.env)?;
                                if !var.is_list() {
                                    return Err(
                                        "'random element of' needs a list variable like {x::*}".into()
                                    );
                                }
                                Ok(Expr::RandomOf(var))
                            }
                            _ => Err("'random element of' needs a list variable like {x::*}".into()),
                        }
                    }
                    "x" | "y" | "z"
                        if matches!(self.peek(), Some(t) if t.is_lit("-") || t.is_lit("coordinate"))
                            && matches!(self.toks.get(self.pos + 1), Some(t) if t.is_lit("coordinate") || t.is_lit("of")) =>
                    {
                        // `x-coordinate of` / `x coordinate of`
                        self.eat_sym("-");
                        if matches!(self.peek(), Some(t) if t.is_lit("coordinate")) {
                            self.pos += 1;
                        }
                        let prop = Prop::from_word(&lw).ok_or("unknown property")?;
                        self.prop_of(prop)
                    }
                    "random" if matches!(self.peek(), Some(Tok::Word(w)) if matches!(w.to_ascii_lowercase().as_str(), "integer" | "number" | "int")) =>
                    {
                        let int = !matches!(self.peek(), Some(t) if t.is_lit("number"));
                        self.pos += 1;
                        if !matches!(self.peek(), Some(t) if t.is_lit("between")) {
                            return Err("expected 'random integer between <number> and <number>'".into());
                        }
                        self.pos += 1;
                        let a = self.unary()?;
                        if !matches!(self.peek(), Some(t) if t.is_lit("and")) {
                            return Err("expected 'random integer between <number> and <number>'".into());
                        }
                        self.pos += 1;
                        let b = self.unary()?;
                        for side in [&a, &b] {
                            if !Ty::Number.accepts(side.ty(self.env)) {
                                return Err("random needs numbers".into());
                            }
                        }
                        Ok(Expr::Random(int, Box::new(a), Box::new(b)))
                    }
                    "distance" if matches!(self.peek(), Some(t) if t.is_lit("between")) => {
                        self.pos += 1;
                        let a = self.primary()?;
                        if !matches!(self.peek(), Some(t) if t.is_lit("and")) {
                            return Err("expected 'distance between <location> and <location>'".into());
                        }
                        self.pos += 1;
                        let b = self.primary()?;
                        for side in [&a, &b] {
                            let t = side.ty(self.env);
                            if !matches!(t, Ty::Location | Ty::Player | Ty::Object) {
                                return Err(format!("distance needs locations, got a {}", t.name()));
                            }
                        }
                        Ok(Expr::Distance(Box::new(a), Box::new(b)))
                    }
                    "level" | "total"
                        if matches!(self.peek(), Some(Tok::Word(w2)) if crate::xp_level::two_word(&lw, &w2.to_ascii_lowercase()).is_some()) =>
                    {
                        let prop = match self.peek() {
                            Some(Tok::Word(w2)) => crate::xp_level::two_word(&lw, &w2.to_ascii_lowercase()),
                            _ => None,
                        }
                        .ok_or("unknown property")?;
                        self.pos += 1;
                        self.prop_of(prop)
                    }
                    "location" | "position" | "world" | "uuid" | "tool" | "health" | "gamemode" | "x" | "y" | "z"
                    | "held" | "level" | "xp" | "exp" | "experience"
                        if matches!(self.peek(), Some(t) if t.is_lit("of") || t.is_lit("item")) =>
                    {
                        let prop = Prop::from_word(&lw).ok_or("unknown property")?;
                        self.prop_of(prop)
                    }
                    "max" | "food" | "hunger" | "saturation" if self.hf_head().is_some() => {
                        let (prop, n) = self.hf_head().ok_or("unknown property")?;
                        self.pos += n - 1;
                        self.prop_of(prop)
                    }
                    "display" | "tab" | "nickname" | "ping" | "ip"
                        if self.admin_prop_at(&lw, self.pos).is_some_and(|(_, np)| {
                            if np >= self.toks.len() {
                                self.hit_end.set(true);
                            }
                            matches!(self.toks.get(np), Some(t) if t.is_lit("of"))
                        }) =>
                    {
                        let (prop, np) = self.admin_prop_at(&lw, self.pos).ok_or("unknown property")?;
                        self.pos = np;
                        self.prop_of(prop)
                    }
                    "name" => {
                        if !matches!(self.peek(), Some(t) if t.is_lit("of")) {
                            return Err("expected 'name of <player>'".into());
                        }
                        self.pos += 1;
                        let inner = self.primary()?;
                        let t = inner.ty(self.env);
                        if t == Ty::Item {
                            return Ok(Expr::ItemInfo(crate::item_meta::ItemInfo::Name, Box::new(inner)));
                        }
                        if !(t == Ty::Player || t == Ty::Object) {
                            return Err(format!("'name of' needs a player or an item, got {}", t.name()));
                        }
                        Ok(Expr::NameOf(Box::new(inner)))
                    }
                    "lore" | "enchantments" | "enchants" | "type" | "item"
                        if self.item_info_head(&lw).is_some() =>
                    {
                        let (info, extra) = self.item_info_head(&lw).ok_or("unknown item property")?;
                        self.pos += extra + 1;
                        let inner = self.primary()?;
                        let t = inner.ty(self.env);
                        if !(t == Ty::Item || t == Ty::Object) {
                            // `lore of player` may still become `lore of player's tool`.
                            if self.pos >= self.toks.len() {
                                self.hit_end.set(true);
                            }
                            return Err(format!("this needs an item, got {}", t.name()));
                        }
                        Ok(Expr::ItemInfo(info, Box::new(inner)))
                    }
                    _ if lw.starts_with("loop-") => self.loop_expr(&lw),
                    _ if matches!(self.peek(), Some(t) if t.is_lit("("))
                        && !self.has_function(&lw)
                        && MathFn::from_name(&lw).is_some() =>
                    {
                        let (f, n) = MathFn::from_name(&lw).ok_or("unknown math function")?;
                        self.pos += 1;
                        let mut args = vec![self.top()?];
                        while self.eat_sym(",") {
                            args.push(self.top()?);
                        }
                        if !self.eat_sym(")") {
                            return Err(format!("missing ')' after {lw}("));
                        }
                        if args.len() != n {
                            return Err(format!("{lw}() takes {n} number(s), got {}", args.len()));
                        }
                        if args.iter().any(|a| !Ty::Number.accepts(a.ty(self.env))) {
                            return Err(format!("{lw}() needs numbers"));
                        }
                        Ok(Expr::Math(f, args))
                    }
                    _ if matches!(self.peek(), Some(t) if t.is_lit("(")) => self.call(&w),
                    _ => {
                        if let Some(d) = lw.strip_prefix("arg-") {
                            let i: usize = d.parse().map_err(|_| format!("bad argument '{w}'"))?;
                            return self.arg(i);
                        }
                        if let Some((id, level, used)) = crate::item_meta::enchant_at(self.toks, self.pos - 1) {
                            self.pos += used - 1;
                            if self.pos >= self.toks.len() {
                                self.hit_end.set(true);
                            }
                            return Ok(Expr::Enchant(Box::new((id.to_string(), level))));
                        }
                        if let Some(item) = self.item(&w) {
                            return Ok(item);
                        }
                        if let Some(e) = self.t1_literal(&w) {
                            return Ok(e);
                        }
                        Err(format!("unknown expression '{w}'"))
                    }
                }
            }
            other => Err(format!("unexpected {}", describe(&other))),
        }
    }

    /// Wave-4 functions like `clamp(x, 0, 10)` (see `w4_functions`).
    fn w4_function(&mut self, lw: &str) -> Result<Expr, String> {
        let (f, min, max) = crate::w4_functions::lookup(lw).ok_or("unknown function")?;
        self.pos += 1; // (
        let mut args = Vec::new();
        loop {
            let rest = self.toks.get(self.pos..).unwrap_or_default();
            let is_list = matches!(rest.first(), Some(Tok::Var(v)) if v.contains("::"))
                || crate::list_ops::looks_like_list_op(rest, self.env);
            if f.takes_lists() && is_list {
                let (src, end) = crate::list_ops::parse_src_at(self.toks, self.pos, self.env, false)?;
                self.pos = end;
                args.push(Expr::Items(Box::new(src)));
            } else {
                args.push(self.top()?);
            }
            if self.eat_sym(",") {
                continue;
            }
            break;
        }
        if !self.eat_sym(")") {
            return Err(format!("missing ')' after {lw}("));
        }
        if args.len() < min || args.len() > max {
            let want = if min == max {
                min.to_string()
            } else if max == usize::MAX {
                format!("at least {min}")
            } else {
                format!("{min} to {max}")
            };
            return Err(format!("{lw}() takes {want} argument(s), got {}", args.len()));
        }
        for (i, a) in args.iter().enumerate() {
            if f.number_arg(i) && !Ty::Number.accepts(a.ty(self.env)) {
                return Err(format!("argument {} of {lw}() must be a number, got a {}", i + 1, a.ty(self.env).name()));
            }
        }
        Ok(Expr::W4F(Box::new(crate::w4_functions::Node { f, args })))
    }

    /// Server expressions: `online players count`, `motd`, `spawn of <world>`, ...
    fn server_misc(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        use crate::server_misc::WorldArg;
        let start = self.pos - 1;
        let Some(h) = crate::server_misc::expr_head(self.toks, start, lw, &self.hit_end) else {
            return Ok(None);
        };
        let after = start + h.len;
        let has_of = matches!(self.toks.get(after), Some(t) if t.is_lit("of"));
        if h.arg != WorldArg::No && after >= self.toks.len() {
            // `of <world>` could still follow in a longer span.
            self.hit_end.set(true);
        }
        let world = match h.arg {
            WorldArg::No => None,
            WorldArg::Required if !has_of => return Ok(None),
            WorldArg::Optional if !has_of => None,
            _ => {
                self.pos = after + 1;
                let w = self.primary()?;
                if !crate::server_misc::world_ty_ok(w.ty(self.env)) {
                    return Err(format!("expected a world after 'of', got {}", w.ty(self.env).name()));
                }
                Some(Box::new(w))
            }
        };
        if world.is_none() {
            self.pos = after;
        }
        Ok(Some(Expr::Server(h.q, world)))
    }

    fn is_word(&self, at: usize, w: &str) -> bool {
        if self.pos + at >= self.toks.len() {
            self.hit_end.set(true);
        }
        matches!(self.toks.get(self.pos + at), Some(t) if t.is_lit(w))
    }

    /// `x in upper case`, `x in lower case`, `x in proper case`.
    fn in_case(&mut self, e: Expr) -> Expr {
        use crate::text_case_substring::TextFn;
        if !matches!(self.toks.get(self.pos), Some(t) if t.is_lit("in")) {
            return e;
        }
        if !self.is_word(2, "case") {
            return e;
        }
        let f = match self.toks.get(self.pos + 1) {
            Some(t) if t.is_lit("upper") => TextFn::Upper,
            Some(t) if t.is_lit("lower") => TextFn::Lower,
            Some(t) if t.is_lit("proper") => TextFn::Proper,
            _ => return e,
        };
        self.pos += 3;
        Expr::TextFn(f, vec![e])
    }

    fn text_arg(&mut self, what: &str) -> Result<Expr, String> {
        let e = self.primary()?;
        if !Ty::Text.accepts(e.ty(self.env)) {
            return Err(format!("{what} needs text"));
        }
        Ok(e)
    }

    fn num_arg(&mut self, what: &str) -> Result<Expr, String> {
        let e = self.sum()?;
        if !Ty::Number.accepts(e.ty(self.env)) {
            return Err(format!("{what} needs a number"));
        }
        Ok(e)
    }

    fn expected(&self, usage: &str) -> String {
        if self.pos >= self.toks.len() {
            self.hit_end.set(true);
        }
        format!("expected '{usage}'")
    }

    /// A place for a block expression: a location, a player or a variable.
    fn block_place(&mut self, what: &str) -> Result<Expr, String> {
        let e = self.primary()?;
        match e.ty(self.env) {
            Ty::Location | Ty::Player | Ty::Object => Ok(e),
            t => Err(format!("'{what}' needs a location or player, got a {}", t.name())),
        }
    }

    /// Block words: `block at <loc>`, `block below <player>`, `targeted block of <player>`,
    /// `type of block ...`, `location at x, y, z [in world "w"]`. `Ok(None)` when the words
    /// are not one of these forms.
    fn block_expr(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        use crate::block_ops::{BlockExpr, direction};
        let blk = |b: BlockExpr| Ok(Some(Expr::Blk(Box::new(b))));
        match lw {
            "block" => {
                if self.is_word(0, "at") || self.is_word(0, "of") {
                    self.pos += 1;
                    let at = self.block_place("block at")?;
                    return blk(BlockExpr::At(at));
                }
                if let Some(Tok::Word(d)) = self.toks.get(self.pos)
                    && let Some((dx, dy, dz)) = direction(&d.to_ascii_lowercase())
                {
                    self.pos += 1;
                    if self.is_word(0, "of") {
                        self.pos += 1;
                    }
                    let at = self.block_place("block above/below")?;
                    return blk(BlockExpr::Rel(at, dx, dy, dz));
                }
                if matches!(self.peek(), Some(Tok::Var(_))) {
                    let at = self.block_place("block")?;
                    return blk(BlockExpr::At(at));
                }
                Ok(None)
            }
            "targeted" | "target" if self.is_word(0, "block") => {
                self.pos += 1;
                if self.is_word(0, "of") || self.is_word(0, "for") {
                    self.pos += 1;
                    let who = self.primary()?;
                    if !matches!(who.ty(self.env), Ty::Player | Ty::Object) {
                        return Err("'targeted block of' needs a player".into());
                    }
                    return blk(BlockExpr::Targeted(who));
                }
                blk(BlockExpr::Targeted(Expr::Player))
            }
            "type" if self.is_word(0, "of") => {
                let save = self.pos;
                self.pos += 1;
                let block_word = self.is_word(0, "block") || self.is_word(0, "targeted");
                match self.primary() {
                    Ok(inner)
                        if (block_word || matches!(inner, Expr::Var(_) | Expr::Blk(_)))
                            && matches!(inner.ty(self.env), Ty::Location | Ty::Object | Ty::Item) =>
                    {
                        blk(BlockExpr::TypeOf(inner))
                    }
                    Err(e) if block_word => Err(e),
                    _ => {
                        self.pos = save;
                        Ok(None)
                    }
                }
            }
            _ => Ok(None),
        }
    }

    /// SkBee text components. The first word `lw` is already consumed. Gives
    /// `Ok(None)` (position restored) when the words are not one of these forms.
    fn comp_expr(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        if !crate::skbee_text::starts_comp(lw) {
            return Ok(None);
        }
        let save = self.pos;
        let mut first = lw.to_string();
        while matches!(first.as_str(), "a" | "an" | "new") {
            match self.toks.get(self.pos) {
                Some(Tok::Word(w)) => {
                    first = w.to_ascii_lowercase();
                    self.pos += 1;
                }
                _ => {
                    self.pos = save;
                    return Ok(None);
                }
            }
        }
        match self.comp_phrase(&first) {
            Ok(Some(e)) => Ok(Some(e)),
            Ok(None) => {
                self.pos = save;
                Ok(None)
            }
            Err(e) => Err(e),
        }
    }

    fn comp_phrase(&mut self, first: &str) -> Result<Option<Expr>, String> {
        use crate::skbee_text::{ClickKind, CompFn};
        let one = |f: CompFn, a: Expr| Ok(Some(Expr::Comp(f, vec![a])));
        match first {
            "text" if self.is_word(0, "component") || self.is_word(0, "components") => {
                self.pos += 1;
                if !(self.is_word(0, "from") || self.is_word(0, "of")) {
                    return Ok(None);
                }
                self.pos += 1;
                let a = self.primary()?;
                one(CompFn::Plain, a)
            }
            "mini" | "minimessage" | "mm" => {
                if first == "mini" {
                    if !self.is_word(0, "message") {
                        return Ok(None);
                    }
                    self.pos += 1;
                }
                if !(self.is_word(0, "from") || self.is_word(0, "of")) {
                    return Ok(None);
                }
                self.pos += 1;
                let a = self.primary()?;
                one(CompFn::Mini, a)
            }
            "hover" if self.is_word(0, "event") && self.is_word(1, "showing") => {
                self.pos += 2;
                let a = self.primary()?;
                one(CompFn::HoverEvent, a)
            }
            "click" if self.is_word(0, "event") && self.is_word(1, "to") => {
                self.pos += 2;
                let kind = if self.is_word(0, "open") && self.is_word(1, "url") {
                    self.pos += 2;
                    ClickKind::OpenUrl
                } else if self.is_word(0, "run") && self.is_word(1, "command") {
                    self.pos += 2;
                    ClickKind::RunCommand
                } else if self.is_word(0, "suggest") && self.is_word(1, "command") {
                    self.pos += 2;
                    ClickKind::SuggestCommand
                } else if self.is_word(0, "copy") && self.is_word(1, "to") && self.is_word(2, "clipboard") {
                    self.pos += 3;
                    ClickKind::CopyToClipboard
                } else if self.is_word(0, "change") && self.is_word(1, "page") {
                    self.pos += 2;
                    ClickKind::ChangePage
                } else {
                    return Err(
                        "click event needs 'open url', 'run command', 'suggest command', 'copy to clipboard' or 'change page'"
                            .into(),
                    );
                };
                let a = self.primary()?;
                one(CompFn::ClickEvent(kind), a)
            }
            "translate" | "translatable" if self.is_word(0, "component") => {
                self.pos += 1;
                let key = self.primary()?;
                let mut args = vec![key];
                if self.is_word(0, "with") {
                    self.pos += 1;
                    args.push(self.primary()?);
                    while self.eat_sym(",") || self.is_word(0, "and") {
                        if self.is_word(0, "and") {
                            self.pos += 1;
                        }
                        args.push(self.primary()?);
                    }
                }
                Ok(Some(Expr::Comp(CompFn::Translate, args)))
            }
            "keybind" if self.is_word(0, "component") => {
                self.pos += 1;
                let a = self.primary()?;
                one(CompFn::Keybind, a)
            }
            "merged" | "merge" => {
                if self.is_word(0, "text") {
                    self.pos += 1;
                }
                if !(self.is_word(0, "component") || self.is_word(0, "components")) {
                    return Ok(None);
                }
                self.pos += 1;
                if self.is_word(0, "of") {
                    self.pos += 1;
                }
                let mut args = vec![self.primary()?];
                while self.eat_sym(",") || self.is_word(0, "and") {
                    if self.is_word(0, "and") {
                        self.pos += 1;
                    }
                    args.push(self.primary()?);
                }
                if let [Expr::Var(v)] = args.as_slice()
                    && v.is_list()
                {
                    let v = v.clone();
                    return Ok(Some(Expr::Join(v, Box::new(Expr::Text(vec![Part::Lit(String::new())])))));
                }
                Ok(Some(Expr::Comp(CompFn::Merge, args)))
            }
            "plain" if self.is_word(0, "text") && self.is_word(1, "of") => {
                self.pos += 2;
                let a = self.primary()?;
                one(CompFn::PlainText, a)
            }
            _ => Ok(None),
        }
    }

    fn x2(op: W2Op, args: Vec<Expr>) -> Expr {
        Expr::X2(Box::new(crate::w2_expressions_2::W2 { op, args }))
    }

    /// `3 blocks above loc`, `2 meters north of player's location`. The cursor is
    /// after the number. `None` when the words after it are not this form.
    fn x2_offset(&mut self, n: f64) -> Result<Option<Expr>, String> {
        let word = |k: usize| match self.toks.get(self.pos + k) {
            Some(Tok::Word(w)) => Some(w.to_ascii_lowercase()),
            _ => None,
        };
        if self.pos + 1 >= self.toks.len() {
            self.hit_end.set(true);
        }
        if !matches!(word(0).as_deref(), Some("block" | "blocks" | "meter" | "meters" | "metre" | "metres")) {
            return Ok(None);
        }
        let Some((dx, dy, dz)) = word(1).as_deref().and_then(crate::w2_expressions_2::direction) else {
            return Ok(None);
        };
        self.pos += 2;
        if self.is_word(0, "of") {
            self.pos += 1;
        }
        let loc = self.primary()?;
        if !matches!(loc.ty(self.env), Ty::Location | Ty::Player | Ty::Object) {
            return Err("expected a location after 'blocks above/below/north/south/east/west'".into());
        }
        Ok(Some(Self::x2(W2Op::Offset(dx, dy, dz), vec![Expr::Num(n), loc])))
    }

    /// `command`, `full command` and `past event-location` of the
    /// movement events (`w4_events_movement`).
    fn mv_value(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let (field, used) = match lw {
            "command" if !self.is_word(0, "sender") => ("event-command", 0),
            "full" if self.is_word(0, "command") => ("event-full-command", 1),
            "past" if self.is_word(0, "event-location") => ("event-past-location", 1),
            "past" if self.is_word(0, "event-world") => ("event-past-world", 1),
            _ => return Ok(None),
        };
        match crate::w2_events_1::field_index(self.env.ev_fields, field) {
            Some((i, _)) => {
                self.pos += used;
                Ok(Some(Expr::ExtVal(i)))
            }
            None => Ok(None),
        }
    }

    /// `codepoint of`, `character from codepoint`, `characters between`,
    /// `difference between`, `chunk x of`, `compass target of`, `death message`,
    /// `damage cause` and `chat recipients`. `None` when the words do not fit.
    fn x2_fn(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        match lw {
            "codepoint" | "unicode" if self.is_word(0, "of") => {
                self.pos += 1;
                let a = self.text_arg("codepoint of")?;
                Ok(Some(Self::x2(W2Op::Codepoint, vec![a])))
            }
            "code" if self.is_word(0, "point") && self.is_word(1, "of") => {
                self.pos += 2;
                let a = self.text_arg("code point of")?;
                Ok(Some(Self::x2(W2Op::Codepoint, vec![a])))
            }
            "character" | "char"
                if self.is_word(0, "from")
                    && (self.is_word(1, "codepoint")
                        || self.is_word(1, "unicode")
                        || (self.is_word(1, "code") && self.is_word(2, "point"))) =>
            {
                self.pos += if self.is_word(1, "code") { 3 } else { 2 };
                let a = self.sum()?;
                if !Ty::Number.accepts(a.ty(self.env)) {
                    return Err("'character from codepoint' needs a number".into());
                }
                Ok(Some(Self::x2(W2Op::FromCodepoint, vec![a])))
            }
            "characters" | "alphanumeric" | "all" => {
                let mut words = vec![lw.to_string()];
                for k in 0..3 {
                    match self.toks.get(self.pos + k) {
                        Some(Tok::Word(w)) => words.push(w.to_ascii_lowercase()),
                        _ => break,
                    }
                }
                let mut i = usize::from(words[0] == "all");
                let alnum = words.get(i).map(String::as_str) == Some("alphanumeric");
                i += usize::from(alnum);
                if words.get(i).map(String::as_str) != Some("characters")
                    || words.get(i + 1).map(String::as_str) != Some("between")
                {
                    return Ok(None);
                }
                self.pos += i + 1;
                let a = self.text_arg("characters between")?;
                if !(self.is_word(0, "and") || self.is_word(0, "to")) {
                    return Err(self.expected("characters between <text> and <text>"));
                }
                self.pos += 1;
                let b = self.text_arg("characters between")?;
                Ok(Some(Self::x2(W2Op::CharsBetween { alnum }, vec![a, b])))
            }
            "difference" if self.is_word(0, "between") => {
                self.pos += 1;
                let a = self.sum()?;
                if !self.is_word(0, "and") {
                    return Err(self.expected("difference between <a> and <b>"));
                }
                self.pos += 1;
                let b = self.sum()?;
                Ok(Some(Self::x2(W2Op::Difference, vec![a, b])))
            }
            "chunk" if (self.is_word(0, "x") || self.is_word(0, "z")) && self.is_word(1, "of") => {
                let op = if self.is_word(0, "x") { W2Op::ChunkX } else { W2Op::ChunkZ };
                self.pos += 2;
                let l = self.primary()?;
                if !matches!(l.ty(self.env), Ty::Location | Ty::Player | Ty::Object) {
                    return Err("'chunk x of' needs a location".into());
                }
                Ok(Some(Self::x2(op, vec![l])))
            }
            "compass" if self.is_word(0, "target") && self.is_word(1, "of") => {
                self.pos += 2;
                let p = self.primary()?;
                Ok(Some(Self::x2(W2Op::CompassTarget, vec![p])))
            }
            "death" if self.is_word(0, "message") && self.env.has_attacker && !self.env.has_damage => {
                if self.env.in_function {
                    return Err("'death message' is not available inside a function".into());
                }
                self.pos += 1;
                Ok(Some(Self::x2(W2Op::DeathMessage, Vec::new())))
            }
            "damage" if self.is_word(0, "cause") && self.env.has_damage && !self.env.in_function => {
                self.pos += 1;
                Ok(Some(Self::x2(W2Op::DamageCause, Vec::new())))
            }
            "command" | "full" | "past" if !self.env.in_function && !self.env.ev_fields.is_empty() => {
                self.mv_value(lw)
            }
            "chat" if self.is_word(0, "recipients") && self.env.has_message && !self.env.in_function => {
                self.pos += 1;
                Ok(Some(Self::x2(W2Op::ChatRecipients, Vec::new())))
            }
            _ => Ok(None),
        }
    }

    /// Text functions that start with a word (`uppercase of x`, `substring of x from 1 to 3`,
    /// `first 3 characters of x`, `index of "a" in x`, `nl`, `trimmed x`, `reversed x`).
    /// Returns `None` when the words are not one of these forms.
    fn text_fn(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        use crate::text_case_substring::TextFn;
        if !matches!(
            lw,
            "nl" | "newline"
                | "new"
                | "uppercase"
                | "lowercase"
                | "upper"
                | "lower"
                | "capitalized"
                | "capitalised"
                | "trimmed"
                | "trim"
                | "reversed"
                | "substring"
                | "subtext"
                | "first"
                | "last"
                | "character"
                | "char"
                | "index"
        ) || matches!(self.toks.get(self.pos), Some(t) if t.is_lit("("))
        {
            return Ok(None);
        }
        let f = |f: TextFn, args: Vec<Expr>| Ok(Some(Expr::TextFn(f, args)));
        match lw {
            "nl" | "newline" => Ok(Some(Expr::Text(vec![Part::Lit("\n".into())]))),
            "new" if self.is_word(0, "line") => {
                self.pos += 1;
                Ok(Some(Expr::Text(vec![Part::Lit("\n".into())])))
            }
            "uppercase" | "lowercase" if self.is_word(0, "of") => {
                self.pos += 1;
                let a = self.text_arg(&format!("{lw} of"))?;
                f(if lw == "uppercase" { TextFn::Upper } else { TextFn::Lower }, vec![a])
            }
            "upper" | "lower" if self.is_word(0, "case") && self.is_word(1, "of") => {
                self.pos += 2;
                let a = self.text_arg(&format!("{lw} case of"))?;
                f(if lw == "upper" { TextFn::Upper } else { TextFn::Lower }, vec![a])
            }
            "capitalized" | "capitalised" | "trimmed" | "trim"
                if matches!(self.toks.get(self.pos), Some(Tok::Str(_) | Tok::Var(_) | Tok::Word(_))) =>
            {
                if self.is_word(0, "of") {
                    self.pos += 1;
                }
                let a = self.text_arg(lw)?;
                let k = if lw.starts_with("cap") { TextFn::Capitalized } else { TextFn::Trim };
                f(k, vec![a])
            }
            "reversed" => {
                let save = self.pos;
                if self.is_word(0, "text") {
                    self.pos += 1;
                    if self.is_word(0, "of") {
                        self.pos += 1;
                    }
                } else if !matches!(self.toks.get(self.pos), Some(Tok::Str(_))) {
                    // `reversed {x::*}` and friends belong to lists.
                    return Ok(None);
                }
                let a = self.text_arg("reversed")?;
                if matches!(&a, Expr::Var(v) if v.is_list()) {
                    self.pos = save;
                    return Ok(None);
                }
                f(TextFn::Reverse, vec![a])
            }
            "substring" | "subtext" if self.is_word(0, "of") => {
                self.pos += 1;
                let t = self.text_arg(lw)?;
                if !self.is_word(0, "from") {
                    return Err(self.expected(&format!("{lw} of <text> from <number> to <number>")));
                }
                self.pos += 1;
                if self.is_word(0, "characters") || self.is_word(0, "character") {
                    self.pos += 1;
                }
                let a = self.num_arg(lw)?;
                let mut args = vec![t, a];
                if self.is_word(0, "to") {
                    self.pos += 1;
                    args.push(self.num_arg(lw)?);
                }
                f(TextFn::Sub, args)
            }
            "first" | "last"
                if matches!(self.toks.get(self.pos), Some(Tok::Num(_) | Tok::Var(_)))
                    || (self.is_word(0, "index") && self.is_word(1, "of")) =>
            {
                if self.is_word(0, "index") {
                    self.pos += 2;
                    return self.index_args(lw == "last");
                }
                let n = self.unary()?;
                if !Ty::Number.accepts(n.ty(self.env)) {
                    return Err(format!("'{lw}' needs a number of characters"));
                }
                let usage = format!("{lw} <number> characters of <text>");
                if !(self.is_word(0, "characters") || self.is_word(0, "character")) {
                    return Err(self.expected(&usage));
                }
                self.pos += 1;
                if !self.is_word(0, "of") {
                    return Err(self.expected(&usage));
                }
                self.pos += 1;
                let t = self.text_arg(lw)?;
                f(if lw == "first" { TextFn::First } else { TextFn::Last }, vec![n, t])
            }
            "character" | "char"
                if self.is_word(0, "at")
                    && (self.is_word(1, "index") || matches!(self.toks.get(self.pos + 1), Some(Tok::Num(_)))) =>
            {
                self.pos += 1;
                if self.is_word(0, "index") {
                    self.pos += 1;
                }
                let n = self.unary()?;
                if !Ty::Number.accepts(n.ty(self.env)) {
                    return Err("'character at index' needs a number".into());
                }
                if !self.is_word(0, "of") {
                    return Err(self.expected("character at index <number> of <text>"));
                }
                self.pos += 1;
                let t = self.text_arg("character at index")?;
                f(TextFn::CharAt, vec![n, t])
            }
            "index" if self.is_word(0, "of") => {
                self.pos += 1;
                self.index_args(false)
            }
            _ => Ok(None),
        }
    }

    /// The client properties (`client brand`, `view distance`, ...) whose words
    /// start at `at`, with how many words they use.
    fn info_at(&self, at: usize) -> Option<(crate::skbee_utilities::Info, usize)> {
        let word = |i: usize| match self.toks.get(at + i) {
            Some(Tok::Word(w)) => Some(w.to_ascii_lowercase()),
            Some(_) => None,
            None => {
                self.hit_end.set(true);
                None
            }
        };
        let first = word(0)?;
        crate::skbee_utilities::info_words(&first, word(1).as_deref(), word(2).as_deref())
    }

    fn util_info(&self, info: crate::skbee_utilities::Info, of: Expr) -> Result<Expr, String> {
        let t = of.ty(self.env);
        if !(t == Ty::Player || t == Ty::Object) {
            return Err(format!("client properties need a player, got {}", t.name()));
        }
        Ok(Expr::Util(crate::skbee_utilities::UtilFn::Info(info), vec![of]))
    }

    /// SkBee utilities that start with a word: `random uuid`, `hex of n`,
    /// `roman numeral of n`, `json of x`, `abbreviated n`, `number from hex "ff"`,
    /// `client brand of player`.
    fn util_prefix(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        use crate::skbee_utilities::UtilFn;
        if !matches!(
            lw,
            "random"
                | "number"
                | "hex"
                | "roman"
                | "json"
                | "abbreviated"
                | "client"
                | "locale"
                | "language"
                | "view"
                | "pose"
                | "main"
                | "brand"
        ) || matches!(self.toks.get(self.pos), Some(t) if t.is_lit("("))
        {
            return Ok(None);
        }
        let util = |f: UtilFn, a: Vec<Expr>| Ok(Some(Expr::Util(f, a)));
        match lw {
            "random" if self.is_word(0, "uuid") => {
                self.pos += 1;
                util(UtilFn::RandomUuid, Vec::new())
            }
            "number" if self.is_word(0, "from") && (self.is_word(1, "roman") || self.is_word(1, "hex")) => {
                let roman = self.is_word(1, "roman");
                self.pos += 2;
                if roman && self.is_word(0, "numeral") {
                    self.pos += 1;
                }
                let a = self.text_arg("number from")?;
                util(if roman { UtilFn::FromRoman } else { UtilFn::FromHex }, vec![a])
            }
            "hex" if self.is_word(0, "of") => {
                self.pos += 1;
                let a = self.num_arg("hex of")?;
                util(UtilFn::Hex, vec![a])
            }
            "roman" if self.is_word(0, "of") || (self.is_word(0, "numeral") && self.is_word(1, "of")) => {
                self.pos += if self.is_word(0, "of") { 1 } else { 2 };
                let a = self.num_arg("roman numeral of")?;
                util(UtilFn::Roman, vec![a])
            }
            "json" if self.is_word(0, "of") => {
                self.pos += 1;
                let a = self.primary()?;
                util(UtilFn::Json, vec![a])
            }
            "abbreviated" if self.toks.get(self.pos).is_some() => {
                let a = self.primary()?;
                if !Ty::Number.accepts(a.ty(self.env)) {
                    return Err("'abbreviated' needs a number".into());
                }
                util(UtilFn::Abbrev, vec![a])
            }
            _ => {
                let Some((info, n)) = self.info_at(self.pos - 1) else {
                    return Ok(None);
                };
                if !self.is_word(n - 1, "of") {
                    return Ok(None);
                }
                self.pos += n;
                let inner = self.primary()?;
                Ok(Some(self.util_info(info, inner)?))
            }
        }
    }

    /// `n with commas [and 2 decimal places]`, `n rounded to 2 decimal places`.
    fn util_postfix(&mut self, e: Expr) -> Result<Expr, String> {
        use crate::skbee_utilities::UtilFn;
        let Some(Tok::Word(w)) = self.toks.get(self.pos) else {
            return Ok(e);
        };
        let numeric = matches!(e.ty(self.env), Ty::Number | Ty::Object);

        let is_with = w.eq_ignore_ascii_case("with");
        let is_rounded = w.eq_ignore_ascii_case("rounded");
        if is_with && numeric && self.is_word(1, "commas") {
            self.pos += 2;
            let mut args = vec![e];
            let save = self.pos;
            if self.is_word(0, "and") && matches!(self.peek_at(1), Some(Tok::Num(_) | Tok::Var(_))) {
                self.pos += 1;
                let n = self.unary()?;
                if self.is_word(0, "decimal") || self.is_word(0, "decimals") {
                    self.pos += 1;
                    if self.is_word(0, "place") || self.is_word(0, "places") {
                        self.pos += 1;
                    }
                    args.push(n);
                } else {
                    self.pos = save;
                }
            }
            return Ok(Expr::Util(UtilFn::Commas, args));
        }
        if is_rounded && numeric && self.is_word(1, "to") {
            let save = self.pos;
            self.pos += 2;
            let n = self.unary()?;
            if self.is_word(0, "decimal") || self.is_word(0, "decimals") {
                self.pos += 1;
                if self.is_word(0, "place") || self.is_word(0, "places") {
                    self.pos += 1;
                }
                return Ok(Expr::Util(UtilFn::Round, vec![e, n]));
            }
            self.pos = save;
        }
        Ok(e)
    }

    /// After `index of`: `<text> in <text>`.
    fn index_args(&mut self, last: bool) -> Result<Option<Expr>, String> {
        use crate::text_case_substring::TextFn;
        let a = self.text_arg("index of")?;
        if !self.is_word(0, "in") {
            return Err(self.expected("index of <text> in <text>"));
        }
        self.pos += 1;
        let b = self.text_arg("index of")?;
        let k = if last { TextFn::IndexLast } else { TextFn::IndexFirst };
        Ok(Some(Expr::TextFn(k, vec![a, b])))
    }

    fn date_from_unix(&mut self) -> Result<Expr, String> {
        let n = self.primary()?;
        if !Ty::Number.accepts(n.ty(self.env)) {
            return Err("'date from unix timestamp' needs a number of seconds".into());
        }
        Ok(Expr::Date(DateOp::FromUnix, vec![n]))
    }

    /// After a date and the word `formatted`: optional `as "pattern"`.
    fn date_format(&mut self, d: Expr) -> Result<Expr, String> {
        let mut args = vec![d];
        if matches!(self.peek(), Some(t) if t.is_lit("as")) {
            self.pos += 1;
            let f = self.primary()?;
            if !Ty::Text.accepts(f.ty(self.env)) {
                return Err("'formatted as' needs a format text like \"yyyy-MM-dd\"".into());
            }
            args.push(f);
        }
        Ok(Expr::Date(DateOp::Format, args))
    }

    /// `loop-value`, `loop-value-2`, `loop-player`, `loop-index`, `loop-number`, `loop-iteration`.
    /// `of {x::*}` after a word like `indices`; the cursor is on `of`.
    fn list_after_of(&mut self, what: &str) -> Result<VarRef, String> {
        if !matches!(self.peek(), Some(t) if t.is_lit("of")) {
            return Err(format!("expected '{what} of {{list::*}}'"));
        }
        self.pos += 1;
        if let Some(Tok::Var(v)) = self.peek().cloned() {
            self.pos += 1;
            let var = parse_varref(&v, self.env)?;
            if var.is_list() {
                return Ok(var);
            }
        }
        Err(format!("'{what} of' needs a list variable like {{x::*}}"))
    }

    /// `size of`, `amount of`, `number of`.
    fn size_of(&mut self, word: &str) -> Result<Expr, String> {
        if matches!(self.peek(), Some(t) if t.is_lit("of")) {
            let plain_var = matches!(self.toks.get(self.pos + 1), Some(Tok::Var(_)))
                && !matches!(self.toks.get(self.pos + 2), Some(t) if t.is_lit("where"));
            if !plain_var {
                let save = self.pos;
                self.pos += 1;
                if let Some(e) = self.try_agg(Agg::Size) {
                    return e;
                }
                self.pos = save;
            }
        }
        Ok(Expr::Size(self.list_after_of(word)?))
    }

    fn loop_expr(&self, word: &str) -> Result<Expr, String> {
        let loops = &self.env.loops;
        if loops.is_empty() {
            return Err(format!("'{word}' only works inside a loop"));
        }
        let (name, nth) = match word.rsplit_once('-') {
            Some((head, d)) if d.chars().all(|c| c.is_ascii_digit()) && head.contains('-') => {
                (head, d.parse::<usize>().ok())
            }
            _ => (word, None),
        };
        let want: Option<LoopKind> = match name {
            "loop-value" | "loop-iteration" | "loop-counter" | "loop-entity" => None,
            "loop-player" => Some(LoopKind::Players),
            "loop-index" => Some(LoopKind::List),
            "loop-number" => Some(LoopKind::Times),
            _ => {
                return Err(format!(
                    "unknown loop value '{word}' (use loop-value, loop-player, loop-index, loop-number or loop-iteration)"
                ));
            }
        };
        let depth = match nth {
            Some(n) => {
                if n == 0 || n > loops.len() {
                    return Err(format!("'{word}': there are only {} loop(s) here", loops.len()));
                }
                n - 1
            }
            None => match want {
                None => loops.len() - 1,
                Some(k) => match loops.iter().rposition(|l| *l == k) {
                    Some(d) => d,
                    None => {
                        return Err(format!(
                            "'{word}' needs a surrounding {} loop",
                            match k {
                                LoopKind::Players => "'loop all players'",
                                LoopKind::List => "list ('loop {x::*}')",
                                LoopKind::Times => "'loop N times'",
                                LoopKind::Values => "value",
                            }
                        ));
                    }
                },
            },
        };
        if let (Some(k), Some(got)) = (want, loops.get(depth))
            && *got != k
        {
            return Err(format!("'{word}': loop {} is not that kind of loop", depth + 1));
        }
        Ok(match name {
            "loop-index" => Expr::LoopIndex(depth),
            "loop-iteration" | "loop-counter" => Expr::LoopIter(depth),
            _ => Expr::LoopValue(depth),
        })
    }

    /// A list expression starting at `start` (a word or list variable).
    fn list_expr(&mut self, start: usize) -> Result<Expr, String> {
        match parse_src_at(self.toks, start, self.env, false) {
            Ok((s, end)) => {
                self.pos = end;
                Ok(Expr::Items(Box::new(s)))
            }
            Err(e) => {
                self.hit_end.set(true);
                Err(e)
            }
        }
    }

    /// The list after `sum of` / `size of`: a number computed from it.
    /// `None` when it is a single plain value (not a list at all).
    fn try_agg(&mut self, agg: Agg) -> Option<Result<Expr, String>> {
        match parse_src_at(self.toks, self.pos, self.env, false) {
            Ok((ListSrc::One(e), _)) if agg == Agg::Size && !matches!(e, Expr::X2(_)) => None,
            Ok((s, end)) => {
                self.pos = end;
                Some(Ok(Expr::ListAgg(agg, Box::new(s))))
            }
            Err(e) => {
                self.hit_end.set(true);
                Some(Err(e))
            }
        }
    }

    pub(crate) fn has_function(&self, name: &str) -> bool {
        self.env
            .functions
            .iter()
            .any(|s| s.name.eq_ignore_ascii_case(name))
    }

    /// `name(a, b)`: a call of a script function. The `(` is next.
    fn call(&mut self, name: &str) -> Result<Expr, String> {
        let env = self.env;
        let Some(id) = env
            .functions
            .iter()
            .position(|s| s.name.eq_ignore_ascii_case(name))
        else {
            return Err(format!("unknown function '{name}'"));
        };
        if env.functions.get(id).is_some_and(|s| s.failed) {
            return Err(format!("the function '{name}' failed to load (see its errors)"));
        }
        self.pos += 1; // (
        let mut args = Vec::new();
        if !self.eat_sym(")") {
            loop {
                let want_list = env.functions.get(id).and_then(|s| s.params.get(args.len())).map(|p| p.1) == Some(Ty::List);
                if want_list {
                    let (src, end) = parse_src_at(self.toks, self.pos, env, false)?;
                    self.pos = end;
                    args.push(Expr::Items(Box::new(src)));
                } else {
                    args.push(self.top()?);
                }
                if self.eat_sym(")") {
                    break;
                }
                if !self.eat_sym(",") {
                    return Err(format!("expected ',' or ')' in the call of {name}"));
                }
            }
        }
        let sig = env.functions.get(id).ok_or("unknown function")?;
        let required = sig.params.iter().filter(|p| !p.2).count();
        if args.len() < required || args.len() > sig.params.len() {
            return Err(format!(
                "{}() takes {} argument(s), got {}",
                sig.name,
                if required == sig.params.len() {
                    required.to_string()
                } else {
                    format!("{required} to {}", sig.params.len())
                },
                args.len()
            ));
        }
        for (i, (a, (pname, pty, _))) in args.iter().zip(&sig.params).enumerate() {
            let got = a.ty(env);
            if !pty.accepts(got) {
                return Err(format!(
                    "argument {} ({}) of {}() must be a {}, got a {}",
                    i + 1,
                    pname.trim_start_matches('_'),
                    sig.name,
                    pty.name(),
                    got.name()
                ));
            }
        }
        Ok(Expr::Call(id, args))
    }

    fn arg(&self, i: usize) -> Result<Expr, String> {
        let n = if self.env.in_function {
            0
        } else {
            self.env.arg_types.len()
        };
        if n == 0 {
            return Err(format!(
                "'arg' only exists in commands with arguments, not in {}",
                self.env.trigger_name
            ));
        }
        if i == 0 || i > n {
            return Err(format!(
                "arg-{i} does not exist, this command has {n} argument(s)"
            ));
        }
        Ok(Expr::Arg(i))
    }
}

/// Parse a `"hello %player%!"` template body. `%%` is a literal percent sign.
pub fn parse_template(s: &str, env: &Env) -> Result<Vec<Part>, String> {
    let mut parts: Vec<Part> = Vec::new();
    let mut lit = String::new();
    let chars: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        if c != '%' {
            lit.push(c);
            i += 1;
            continue;
        }
        if chars.get(i + 1) == Some(&'%') {
            lit.push('%');
            i += 2;
            continue;
        }
        // find closing %, skipping braces
        let mut j = i + 1;
        let mut depth = 0;
        while j < chars.len() {
            match chars[j] {
                '{' => depth += 1,
                '}' => depth -= 1,
                '%' if depth <= 0 => break,
                _ => {}
            }
            j += 1;
        }
        if j >= chars.len() {
            return Err("unclosed '%' in text (use %% for a percent sign)".into());
        }
        let inner: String = chars.get(i + 1..j).unwrap_or_default().iter().collect();
        let e = parse_expr_str(&inner, env).map_err(|e| format!("in %{inner}%: {e}"))?;
        // Constant parts are folded into the surrounding text.
        match e {
            Expr::Num(n) => lit.push_str(&format_num(n)),
            Expr::Text(inner_parts) => {
                for p in inner_parts {
                    match p {
                        Part::Lit(l) => lit.push_str(&l),
                        e @ Part::Expr(_) => {
                            if !lit.is_empty() {
                                parts.push(Part::Lit(std::mem::take(&mut lit)));
                            }
                            parts.push(e);
                        }
                    }
                }
            }
            e => {
                if !lit.is_empty() {
                    parts.push(Part::Lit(std::mem::take(&mut lit)));
                }
                parts.push(Part::Expr(e));
            }
        }
        i = j + 1;
    }
    if !lit.is_empty() {
        parts.push(Part::Lit(lit));
    }
    Ok(parts)
}

impl VarRef {
    /// `{x::*}`: the whole list rather than one variable.
    pub fn is_list(&self) -> bool {
        self.list
    }
}

pub fn parse_varref(raw: &str, env: &Env) -> Result<VarRef, String> {
    let raw = raw.trim();
    let mut parts = parse_template(raw, env)?;
    let local = raw.starts_with('_');
    let list = raw.ends_with("::*");
    let key = match parts.as_mut_slice() {
        [] => Key::Static(String::new()),
        [Part::Lit(l)] => Key::Static(std::mem::take(l).to_ascii_lowercase()),
        _ => Key::Dynamic(parts),
    };
    Ok(VarRef { key, local, list })
}

//! Block spheres and lines, written books,
//! world border settings, item damage, ender chests,
//! entity AI and attributes, enchanting offers, inventory actions and a few
//! event values (`broadcast message`, `consumed item`, `the zombie`).
//!
//! Every expression is one boxed [`Node`] (`Expr::BlockShapes`) and every effect one
//! boxed [`ShapeEffect`] (`Effect::BlockShapes`), so the hot enums stay small. The engine
//! side is in the `impl Engine` block at the bottom; both entry points are
//! `#[cold]`. Game access goes through one host call, [`Host::block_shape_query`].

use crate::block_ops::BlockPos;
use crate::compile::Effect;
use crate::engine::{Engine, Exec, online};
use crate::expr::{Env, Expr, P, Ty, parse_expr};
use crate::host::Host;
use crate::lexer::Tok;
use crate::skbee_nbt::{Compound, Tag};
use crate::value::{EntityRef, PlayerRef, Value};

/// Largest radius of `blocks in radius`.
pub const MAX_RADIUS: f64 = 32.0;
/// Most blocks `blocks between a and b` gives.
pub const MAX_CUBOID: i64 = 262_144;
/// Most blocks of `blocks from a to b` and `blocks above x`.
pub const MAX_LINE: usize = 1000;
/// How far `blocks above x` reaches, like Skript's default.
pub const RAY_LEN: usize = 100;
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Book {
    Author,
    Title,
    Pages,
    /// `page N of x`: the page number is the second argument.
    Page,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BorderField {
    Center,
    DamageAmount,
    DamageBuffer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Op {
    /// `blocks in radius r of loc`: radius, center.
    Sphere,
    /// `blocks between a and b`: the box.
    Cuboid,
    /// `blocks from a to b`: a straight line.
    Line,
    /// `blocks above x`: 0 up, 1 down, 2 north, 3 south, 4 east, 5 west.
    Ray(u8),
    Book(Book),
    BarById,
    /// Brewing stand slot 0-2 bottles, 3 ingredient, 4 fuel.
    BrewSlot(u8),
    Border(BorderField),
    Cursor,
    InvAction,
    Damage,
    Durability,
    /// `diamond sword with damage 10`.
    WithDamage,
    EnderChest,
    EnderSlot,
    Ai,
    /// Attribute index in `data::named_types::ATTRIBUTES`, and whether it is the
    /// total value (base plus modifiers) instead of the base.
    Attr(u8, bool),
    CarriedBlock,
    EnchLevel,
    Offers,
    Offer,
    OfferCost,
    /// `the zombie` / `the creature` in damage and death events. The kind is
    /// the first argument (none for any entity).
    EventEntity,
}

/// One expression of this module.
#[derive(Debug, Clone, PartialEq)]
pub struct Node {
    pub op: Op,
    pub args: Vec<Expr>,
    pub ty: Ty,
}

impl Node {
    /// Can `set` change it?
    pub fn settable(&self) -> bool {
        matches!(
            self.op,
            Op::Book(_)
                | Op::BrewSlot(_)
                | Op::Border(_)
                | Op::Damage
                | Op::Durability
                | Op::EnderSlot
                | Op::Ai
                | Op::Attr(..)
                | Op::CarriedBlock
        )
    }
}

fn node(op: Op, args: Vec<Expr>, ty: Ty) -> Expr {
    Expr::BlockShapes(Box::new(Node { op, args, ty }))
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Set,
    Add,
    Remove,
    Reset,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ShapeEffect {
    Change {
        target: Node,
        mode: Mode,
        value: Option<Expr>,
    },
    OpenEnder {
        owner: Expr,
        to: Option<Expr>,
    },
    ClearEnder(Expr),
}

/// A request to the game. Setters answer `Some(Value::Bool(true))` when they worked.
#[derive(Debug, Clone, Copy)]
pub enum Q<'a> {
    /// Block id with its properties: `minecraft:oak_stairs[facing=east,half=bottom]`.
    /// Brewing stand slot 0-4: an item, or none for an empty slot.
    Brew(&'a BlockPos, u8),
    SetBrew(&'a BlockPos, u8, &'a Value),
    /// World name and field. The center is a location with y 0.
    Border(&'a str, BorderField),
    SetBorder(&'a str, BorderField, &'a Value),
    /// The `written_book_content` component bytes of the held item.
    HeldBook(&'a PlayerRef),
    SetHeldBook(&'a PlayerRef, &'a [u8]),
    BarExists(&'a str),
    /// One ender chest slot, or every slot as a list.
    Ender(&'a PlayerRef, Option<u8>),
    SetEnder(&'a PlayerRef, u8, &'a Value),
    ClearEnder(&'a PlayerRef),
    OpenEnder(&'a PlayerRef),
    /// A player or entity value.
    Ai(&'a Value),
    SetAi(&'a Value, bool),
    /// Attribute id like `movement_speed`; true for the total value.
    Attr(&'a Value, &'a str, bool),
    SetAttr(&'a Value, &'a str, f64),
    ResetAttr(&'a Value, &'a str),
    /// Block data an enderman carries.
    CarriedBlock(&'a EntityRef),
    SetCarriedBlock(&'a EntityRef, Option<&'a str>),
}

// ---------------------------------------------------------------- parsing

fn is(t: Option<&Tok>, w: &str) -> bool {
    t.is_some_and(|t| t.is_lit(w))
}

fn dir_index(w: &str) -> Option<u8> {
    Some(match w {
        "above" | "upwards" | "up" => 0,
        "below" | "downwards" | "down" | "under" => 1,
        "north" => 2,
        "south" => 3,
        "east" => 4,
        "west" => 5,
        _ => return None,
    })
}

const DIRS: [(i32, i32, i32); 6] = [
    (0, 1, 0),
    (0, -1, 0),
    (0, 0, -1),
    (0, 0, 1),
    (1, 0, 0),
    (-1, 0, 0),
];

/// Attribute index of `movement speed`, `generic_max_health`, ...
pub fn attr_index(name: &str) -> Option<u8> {
    let n = name.trim().to_ascii_lowercase().replace([' ', '-'], "_");
    let n = n.strip_prefix("minecraft:").unwrap_or(&n);
    let n = n
        .strip_prefix("generic_")
        .or_else(|| n.strip_prefix("player_"))
        .or_else(|| n.strip_prefix("zombie_"))
        .unwrap_or(n);
    crate::data::named_types::ATTRIBUTES
        .iter()
        .position(|a| *a == n)
        .and_then(|i| u8::try_from(i).ok())
}

fn ordinal(w: &str) -> Option<u8> {
    Some(match w {
        "first" | "1st" => 0,
        "second" | "2nd" => 1,
        "third" | "3rd" => 2,
        _ => return None,
    })
}

const GENERIC_ENTITY: [&str; 6] = ["entity", "creature", "mob", "monster", "animal", "livingentity"];

impl P<'_> {
    fn c4(&self, n: usize, w: &str) -> bool {
        if self.pos + n >= self.toks.len() {
            self.hit_end.set(true);
        }
        is(self.toks.get(self.pos + n), w)
    }

    fn shape_word(&self, n: usize) -> Option<String> {
        if self.pos + n >= self.toks.len() {
            self.hit_end.set(true);
        }
        match self.toks.get(self.pos + n) {
            Some(Tok::Word(w)) => Some(w.to_ascii_lowercase()),
            _ => None,
        }
    }

    /// Skips `the` at the current position.
    fn shape_the(&mut self) {
        if self.c4(0, "the") {
            self.pos += 1;
        }
    }

    fn shape_of(&mut self, what: &str) -> Result<Expr, String> {
        if !(self.c4(0, "of") || self.c4(0, "in")) {
            return Err(format!("expected 'of' after '{what}'"));
        }
        self.pos += 1;
        self.primary()
    }

    fn shape_num(&mut self, what: &str) -> Result<Expr, String> {
        let e = self.primary()?;
        if !Ty::Number.accepts(e.ty(self.env)) {
            return Err(format!("{what} needs a number"));
        }
        Ok(e)
    }

    /// Expression heads of this module. `self.pos` is after the first word `lw`.
    pub(crate) fn shape_head(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let start = self.pos;
        let r = self.shape_head_inner(lw);
        if matches!(r, Ok(None)) {
            self.pos = start;
        }
        r
    }

    fn shape_head_inner(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        Ok(Some(match lw {
            "all" if self.c4(0, "blocks") || (self.c4(0, "the") && self.c4(1, "blocks")) => {
                self.shape_the();
                self.pos += 1;
                match self.shape_blocks()? {
                    Some(e) => e,
                    None => return Ok(None),
                }
            }
            "blocks" => match self.shape_blocks()? {
                Some(e) => e,
                None => return Ok(None),
            },
            "book" => {
                let part = match self.shape_word(0).as_deref() {
                    Some("author" | "writer" | "publisher") => Book::Author,
                    Some("title" | "name") => Book::Title,
                    Some("pages" | "content") => Book::Pages,
                    Some("page") => Book::Page,
                    _ => return Ok(None),
                };
                self.pos += 1;
                self.shape_book(part)?
            }
            "author" | "writer" | "publisher" if self.c4(0, "of") => self.shape_book(Book::Author)?,
            "pages" if self.c4(0, "of") => self.shape_book(Book::Pages)?,
            "page" if matches!(self.toks.get(self.pos), Some(Tok::Num(_) | Tok::Var(_))) => {
                // One token: `2 of player's tool` would read as `N of item`.
                let n = parse_expr(&self.toks[self.pos..=self.pos], self.env)?;
                self.pos += 1;
                if !self.c4(0, "of") {
                    return Ok(None);
                }
                let mut e = self.shape_book(Book::Page)?;
                if let Expr::BlockShapes(b) = &mut e {
                    b.args.push(n);
                }
                e
            }
            "bossbar" | "boss" => {
                if lw == "boss" {
                    if !self.c4(0, "bar") {
                        return Ok(None);
                    }
                    self.pos += 1;
                }
                let n = if self.c4(0, "with") || self.c4(0, "from") {
                    1
                } else {
                    0
                };
                // `boss bar id of <bar>` is the bar's id (`potions_particles_queues`), not a lookup.
                if !(self.c4(n, "id") || self.c4(n, "key")) || (n == 0 && self.c4(1, "of")) {
                    return Ok(None);
                }
                self.pos += n + 1;
                let id = self.primary()?;
                node(Op::BarById, vec![id], Ty::Text)
            }
            "brewing" if self.c4(0, "stand") || self.c4(0, "stand's") => {
                self.pos += 1;
                if matches!(self.toks.get(self.pos), Some(Tok::Sym("'s"))) {
                    self.pos += 1;
                }
                let Some(w) = self.shape_word(0) else {
                    return Ok(None);
                };
                self.pos += 1;
                match self.shape_brew_slot(&w)? {
                    Some(e) => e,
                    None => return Ok(None),
                }
            }
            "first" | "second" | "third" | "ingredient" | "fuel" | "1st" | "2nd" | "3rd" => {
                if self.c4(0, "bottle") || (lw == "ingredient" || lw == "fuel") && self.c4(0, "slot") {
                    match self.shape_brew_slot(lw)? {
                        Some(e) => e,
                        None => return Ok(None),
                    }
                } else if (self.c4(0, "enchantment") || self.c4(0, "enchant")) && self.c4(1, "offer") {
                    self.pos += 2;
                    let i = ordinal(lw).unwrap_or(0);
                    node(Op::Offer, vec![Expr::Num(f64::from(i) + 1.0)], Ty::Object)
                } else {
                    return Ok(None);
                }
            }
            "world" | "worldborder" if self.shape_border_ahead(lw) => {
                self.pos += if lw == "world" { 1 } else { 0 };
                let field = match self.shape_word(0).as_deref() {
                    Some("center" | "centre" | "middle") => {
                        self.pos += 1;
                        BorderField::Center
                    }
                    Some("damage") if self.c4(1, "amount") => {
                        self.pos += 2;
                        BorderField::DamageAmount
                    }
                    Some("damage") if self.c4(1, "buffer") => {
                        self.pos += 2;
                        BorderField::DamageBuffer
                    }
                    _ => return Ok(None),
                };
                let w = self.shape_of("world border")?;
                let ty = if field == BorderField::Center {
                    Ty::Location
                } else {
                    Ty::Number
                };
                node(Op::Border(field), vec![w], ty)
            }
            "cursor" if self.c4(0, "slot") || self.c4(0, "item") => {
                self.pos += 1;
                if self.c4(0, "of") {
                    self.pos += 1;
                    let _ = self.primary()?;
                }
                self.shape_click("cursor slot")?;
                node(Op::Cursor, vec![], Ty::Item)
            }
            "inventory" | "click" if self.c4(0, "action") => {
                self.pos += 1;
                self.shape_click("inventory action")?;
                let ty = crate::named_types::ty_from_name("inventory action").unwrap_or(Ty::Object);
                node(Op::InvAction, vec![], ty)
            }
            "damage" | "durability" | "durabilities" | "damages"
                if self.c4(0, "of") || (self.c4(0, "value") || self.c4(0, "values")) && self.c4(1, "of") =>
            {
                if !self.c4(0, "of") {
                    self.pos += 1;
                }
                let it = self.shape_of(lw)?;
                if !matches!(it.ty(self.env), Ty::Item | Ty::Object) {
                    return Err(format!("'{lw} of' needs an item"));
                }
                let op = if lw.starts_with("dur") {
                    Op::Durability
                } else {
                    Op::Damage
                };
                node(op, vec![it], Ty::Number)
            }
            "ender" | "enderchest" => {
                if lw == "ender" {
                    if !self.c4(0, "chest") {
                        return Ok(None);
                    }
                    self.pos += 1;
                }
                if self.c4(0, "inventory") {
                    self.pos += 1;
                }
                let p = self.shape_of("ender chest")?;
                if !matches!(p.ty(self.env), Ty::Player | Ty::Object) {
                    return Err("'ender chest of' needs a player".into());
                }
                node(Op::EnderChest, vec![p], Ty::List)
            }
            "slot" => match self.shape_ender_slot()? {
                Some(e) => e,
                None => return Ok(None),
            },
            "ai" if self.c4(0, "of") => {
                let e = self.shape_of("ai")?;
                node(Op::Ai, vec![e], Ty::Boolean)
            }
            "artificial" if self.c4(0, "intelligence") && self.c4(1, "of") => {
                self.pos += 1;
                let e = self.shape_of("artificial intelligence")?;
                node(Op::Ai, vec![e], Ty::Boolean)
            }
            "carried" if self.c4(0, "block") || self.c4(0, "blockdata") => {
                self.pos += 1;
                if self.c4(0, "data") {
                    self.pos += 1;
                }
                let e = self.shape_of("carried block")?;
                node(Op::CarriedBlock, vec![e], Ty::Text)
            }
            "enchantment" | "enchant" if self.c4(0, "level") || self.c4(0, "levels") => {
                self.pos += 1;
                let e = self.shape_of("enchantment level")?;
                node(Op::EnchLevel, vec![e], Ty::Number)
            }
            "enchantment" | "enchant" if self.c4(0, "offers") => {
                self.pos += 1;
                self.shape_prepare("enchantment offers")?;
                node(Op::Offers, vec![], Ty::List)
            }
            "enchantment" | "enchant"
                if self.c4(0, "offer")
                    && matches!(self.toks.get(self.pos + 1), Some(Tok::Num(_) | Tok::Var(_))) =>
            {
                self.pos += 1;
                self.shape_prepare("enchantment offer")?;
                let n = self.shape_num("'enchantment offer'")?;
                node(Op::Offer, vec![n], Ty::Object)
            }
            "enchantment" | "enchant" if self.c4(0, "cost") && self.c4(1, "of") => {
                self.pos += 1;
                let e = self.shape_of("enchantment cost")?;
                node(Op::OfferCost, vec![e], Ty::Number)
            }
            "cost" if self.c4(0, "of") => {
                let e = self.shape_of("cost")?;
                node(Op::OfferCost, vec![e], Ty::Number)
            }
            "the" | "event-entity" | "event-creature" | "event-mob" => match self.shape_event_entity(lw) {
                Some(e) => e,
                None => return Ok(None),
            },
            _ if lw.starts_with("event-") && self.env.has_attacker && !self.env.in_function => {
                let kind = &lw["event-".len()..];
                match crate::entities::kind_of(kind) {
                    Some(k) if k != "player" => node(Op::EventEntity, vec![text(k)], Ty::Object),
                    _ => return Ok(None),
                }
            }
            "broadcast" | "broadcasted"
                if self.c4(0, "message")
                    && !self.env.in_function
                    && self.env.ev_fields == crate::events_blocks_players::ExtEvent::Broadcast.fields() =>
            {
                self.pos += 1;
                Expr::ExtVal(0)
            }
            "consumed"
                if self.c4(0, "item")
                    && !self.env.in_function
                    && self.env.trigger_name.contains("consume") =>
            {
                self.pos += 1;
                Expr::EventVal(crate::events_items::EventVal::Item)
            }
            _ => match self.shape_attribute(lw)? {
                Some(e) => e,
                None => return Ok(None),
            },
        }))
    }

    fn shape_click(&self, what: &str) -> Result<(), String> {
        if self.env.has_inv && !self.env.in_function {
            Ok(())
        } else {
            Err(format!("'{what}' only works in 'on inventory click'"))
        }
    }

    fn shape_prepare(&self, what: &str) -> Result<(), String> {
        if self.env.ext_event == Some(crate::events_game::Ev::EnchantPrepare) && !self.env.in_function {
            Ok(())
        } else {
            Err(format!("'{what}' only works in 'on enchant prepare'"))
        }
    }

    /// After `blocks` (and `all [the]`).
    fn shape_blocks(&mut self) -> Result<Option<Expr>, String> {
        let Some(w) = self.shape_word(0) else {
            return Ok(None);
        };
        Ok(Some(match w.as_str() {
            // `blocks in radius 5 [of|around] player`
            "in" | "within" if self.c4(1, "radius") => {
                self.pos += 2;
                let r = self.shape_num("'blocks in radius'")?;
                if !(self.c4(0, "of") || self.c4(0, "around")) {
                    return Err("expected 'of' or 'around' after the radius".into());
                }
                self.pos += 1;
                let c = self.primary()?;
                node(Op::Sphere, vec![r, c], Ty::List)
            }
            // `blocks around player in radius 5`
            "around" => {
                self.pos += 1;
                let c = self.primary()?;
                if !(self.c4(0, "in") || self.c4(0, "within")) || !self.c4(1, "radius") {
                    return Err("expected 'in radius <number>' after 'blocks around <location>'".into());
                }
                self.pos += 2;
                let r = self.shape_num("'blocks around ... in radius'")?;
                node(Op::Sphere, vec![r, c], Ty::List)
            }
            "between" | "within" => {
                self.pos += 1;
                let a = self.primary()?;
                if !self.c4(0, "and") {
                    return Err("expected 'and' in 'blocks between <location> and <location>'".into());
                }
                self.pos += 1;
                let b = self.primary()?;
                node(Op::Cuboid, vec![a, b], Ty::List)
            }
            "from" => {
                self.pos += 1;
                let a = self.primary()?;
                if !self.c4(0, "to") {
                    return Err("expected 'to' in 'blocks from <location> to <location>'".into());
                }
                self.pos += 1;
                let b = self.primary()?;
                node(Op::Line, vec![a, b], Ty::List)
            }
            d => {
                let Some(i) = dir_index(d) else {
                    return Ok(None);
                };
                self.pos += 1;
                if self.c4(0, "of") {
                    self.pos += 1;
                }
                let from = self.primary()?;
                node(Op::Ray(i), vec![from], Ty::List)
            }
        }))
    }

    fn shape_book(&mut self, part: Book) -> Result<Expr, String> {
        let it = self.shape_of("book")?;
        if !matches!(it.ty(self.env), Ty::Item | Ty::Object) {
            return Err("book values need an item, like player's tool".into());
        }
        let ty = if part == Book::Pages { Ty::List } else { Ty::Text };
        Ok(node(Op::Book(part), vec![it], ty))
    }

    /// `first bottle [slot] of`, `ingredient slot of`, `fuel slot of`.
    fn shape_brew_slot(&mut self, w: &str) -> Result<Option<Expr>, String> {
        let slot = match w {
            "ingredient" => 3,
            "fuel" => 4,
            o => match ordinal(o) {
                Some(i) if self.c4(0, "bottle") => {
                    self.pos += 1;
                    i
                }
                _ => return Ok(None),
            },
        };
        if self.c4(0, "slot") {
            self.pos += 1;
        } else if slot > 2 {
            return Ok(None);
        }
        if !self.c4(0, "of") {
            return Ok(None);
        }
        let b = self.shape_of("brewing stand slot")?;
        Ok(Some(node(Op::BrewSlot(slot), vec![b], Ty::Item)))
    }

    fn shape_border_ahead(&self, lw: &str) -> bool {
        let i = if lw == "world" {
            if !self.c4(0, "border") {
                return false;
            }
            1
        } else {
            0
        };
        self.c4(i, "center")
            || self.c4(i, "centre")
            || self.c4(i, "middle")
            || (self.c4(i, "damage") && (self.c4(i + 1, "amount") || self.c4(i + 1, "buffer")))
    }

    /// `slot N of ender chest of P` / `slot N of P's ender chest`.
    fn shape_ender_slot(&mut self) -> Result<Option<Expr>, String> {
        let save = self.pos;
        let Ok(n) = self.primary() else {
            self.pos = save;
            return Ok(None);
        };
        if !self.c4(0, "of") {
            self.pos = save;
            return Ok(None);
        }
        self.pos += 1;
        self.shape_the();
        let owner = if self.c4(0, "ender") && self.c4(1, "chest") || self.c4(0, "enderchest") {
            self.pos += if self.c4(0, "ender") { 2 } else { 1 };
            if self.c4(0, "inventory") {
                self.pos += 1;
            }
            if !self.c4(0, "of") {
                self.pos = save;
                return Ok(None);
            }
            self.pos += 1;
            self.primary()?
        } else {
            // `P's ender chest`: only a player-ish token then `'s ender chest`.
            let i = self.pos;
            let one = matches!(self.toks.get(i), Some(Tok::Var(_)))
                || matches!(self.toks.get(i), Some(Tok::Word(w)) if w.eq_ignore_ascii_case("player"));
            if !(one
                && matches!(self.toks.get(i + 1), Some(Tok::Sym("'s")))
                && (is(self.toks.get(i + 2), "ender") && is(self.toks.get(i + 3), "chest")
                    || is(self.toks.get(i + 2), "enderchest")))
            {
                self.pos = save;
                return Ok(None);
            }
            let owner = if is(self.toks.get(i), "player") {
                Expr::Player
            } else {
                parse_expr(&self.toks[i..=i], self.env)?
            };
            self.pos = i + if is(self.toks.get(i + 2), "ender") { 4 } else { 3 };
            owner
        };
        Ok(Some(node(Op::EnderSlot, vec![n, owner], Ty::Item)))
    }

    /// `movement speed attribute of player`, `max health total attribute of e`.
    fn shape_attribute(&mut self, lw: &str) -> Result<Option<Expr>, String> {
        let first = |a: &&str| a.split('_').next() == Some(lw);
        if !(lw == "generic" || lw == "player" || crate::data::named_types::ATTRIBUTES.iter().any(first)) {
            return Ok(None);
        }
        let mut words = vec![lw.to_string()];
        let mut i = 0;
        let mut total = false;
        loop {
            let Some(w) = self.shape_word(i) else {
                return Ok(None);
            };
            if w == "attribute" {
                break;
            }
            if matches!(w.as_str(), "total" | "final" | "modified") {
                total = true;
            } else {
                words.push(w);
            }
            i += 1;
            if i > 5 {
                return Ok(None);
            }
        }
        let Some(idx) = attr_index(&words.join("_")) else {
            return Ok(None);
        };
        self.pos += i + 1;
        if self.c4(0, "value") {
            self.pos += 1;
        }
        let e = self.shape_of("attribute")?;
        Ok(Some(node(Op::Attr(idx, total), vec![e], Ty::Number)))
    }

    fn shape_event_entity(&mut self, lw: &str) -> Option<Expr> {
        if !self.env.has_attacker || self.env.in_function {
            return None;
        }
        let word = if lw == "the" {
            let w = self.shape_word(0)?;
            if w == "living" && self.c4(1, "entity") {
                self.pos += 2;
                return Some(node(Op::EventEntity, vec![], Ty::Object));
            }
            self.pos += 1;
            w
        } else {
            lw["event-".len()..].to_string()
        };
        if GENERIC_ENTITY.contains(&word.as_str()) {
            return Some(node(Op::EventEntity, vec![], Ty::Object));
        }
        match crate::entities::kind_of(&word) {
            Some(k) if k != "player" => Some(node(Op::EventEntity, vec![text(k)], Ty::Object)),
            _ => None,
        }
    }

    /// `<loc>'s block data`, `<player>'s ender chest`, `<entity>'s ai`. `self.pos` is at the word after `'s`.
    pub(crate) fn shape_poss(&mut self, lw: &str, e: &Expr) -> Option<Expr> {
        let (op, n, ty) = match lw {
            "ender" if self.c4(1, "chest") => (Op::EnderChest, 2, Ty::List),
            "enderchest" => (Op::EnderChest, 1, Ty::List),
            "ai" => (Op::Ai, 1, Ty::Boolean),
            "artificial" if self.c4(1, "intelligence") => (Op::Ai, 2, Ty::Boolean),
            "carried" if self.c4(1, "block") => (Op::CarriedBlock, 2, Ty::Text),
            _ => return None,
        };
        self.pos += n;
        if op == Op::CarriedBlock && self.c4(0, "data") {
            self.pos += 1;
        }
        Some(node(op, vec![e.clone()], ty))
    }

    /// `<item> with damage 10`, `<item> damaged by 10`. `self.pos` is after the item.
    pub(crate) fn shape_post(&mut self, e: Expr) -> Result<Expr, String> {
        if !matches!(e.ty(self.env), Ty::Item) {
            return Ok(e);
        }
        let n = if self.c4(0, "with") && (self.c4(1, "damage") || self.c4(1, "data")) {
            if self.c4(2, "value") { 3 } else { 2 }
        } else if self.c4(0, "damaged") && self.c4(1, "by") {
            2
        } else {
            return Ok(e);
        };
        self.pos += n;
        let d = self.shape_num("'with damage'")?;
        Ok(node(Op::WithDamage, vec![e, d], Ty::Item))
    }
}

fn text(s: &str) -> Expr {
    Expr::Text(vec![crate::expr::Part::Lit(s.to_string())])
}

// ---------------------------------------------------------------- effects

fn rpos(toks: &[Tok], w: &str) -> Option<usize> {
    toks.iter().rposition(|t| t.is_lit(w))
}

fn shape_node(toks: &[Tok], env: &Env) -> Option<Node> {
    match parse_expr(toks, env) {
        Ok(Expr::BlockShapes(n)) if n.settable() => Some(*n),
        _ => None,
    }
}

/// Does the line hold one of this module's list expressions (`blocks in radius ...`)?
/// Cheap first-word check before parsing.
pub fn is_list_expr(toks: &[Tok], env: &Env) -> bool {
    let Some(Tok::Word(w)) = toks.first() else {
        return false;
    };
    let lw = w.to_ascii_lowercase();
    let maybe = match lw.as_str() {
        "blocks" | "ender" | "enderchest" | "pages" | "book" => true,
        "all" => toks.get(1).is_some_and(|t| t.is_lit("blocks") || t.is_lit("the")),
        "enchantment" | "enchant" => toks.get(1).is_some_and(|t| t.is_lit("offers")),
        _ => toks.iter().any(|t| matches!(t, Tok::Sym("'s"))),
    };
    maybe && matches!(parse_expr(toks, env), Ok(e @ Expr::BlockShapes(_)) if e.ty(env) == Ty::List)
}

fn eff(e: ShapeEffect) -> Effect {
    Effect::BlockShapes(Box::new(e))
}

/// The effects of this module: `set/reset/add/remove` on a settable
/// expression, `apply {_effect} to player`, `open/clear ender chest of player`.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    let lw = first.to_ascii_lowercase();
    let rest = toks.get(1..).unwrap_or_default();
    Ok(match lw.as_str() {
        "set" => {
            let Some(to) = rpos(toks, "to") else {
                return Ok(None);
            };
            // `set {_b} to boss bar with id "x"`: the bar words would send it to the boss bar effects.
            if let [Tok::Var(raw)] = toks.get(1..to).unwrap_or_default()
                && let Ok(var) = crate::expr::parse_varref(raw, env)
                && !var.is_list()
                && let Ok(v @ Expr::BlockShapes(_)) = parse_expr(toks.get(to + 1..).unwrap_or_default(), env)
            {
                return Ok(Some(Effect::Set(var, v)));
            }
            let Some(target) = shape_node(toks.get(1..to).unwrap_or_default(), env) else {
                return Ok(None);
            };
            let v = parse_expr(toks.get(to + 1..).unwrap_or_default(), env)?;
            Some(eff(ShapeEffect::Change {
                target,
                mode: Mode::Set,
                value: Some(v),
            }))
        }
        "reset" | "delete" | "clear" => {
            if let Some(owner) = ender_owner(rest, env) {
                return Ok(Some(eff(ShapeEffect::ClearEnder(owner))));
            }
            let Some(target) = shape_node(rest, env) else {
                return Ok(None);
            };
            Some(eff(ShapeEffect::Change {
                target,
                mode: Mode::Reset,
                value: None,
            }))
        }
        "add" | "remove" | "subtract" => {
            let sep = if lw == "add" { "to" } else { "from" };
            let Some(at) = rpos(toks, sep) else {
                return Ok(None);
            };
            let Some(target) = shape_node(toks.get(at + 1..).unwrap_or_default(), env) else {
                return Ok(None);
            };
            if !matches!(target.ty, Ty::Number) {
                return Err("only number values can be added to or removed from".into());
            }
            let v = parse_expr(toks.get(1..at).unwrap_or_default(), env)?;
            let mode = if lw == "add" { Mode::Add } else { Mode::Remove };
            Some(eff(ShapeEffect::Change {
                target,
                mode,
                value: Some(v),
            }))
        }
        "open" => {
            let (owner_toks, to) = match rpos(toks, "to").or_else(|| rpos(toks, "for")) {
                Some(t) => (
                    toks.get(1..t).unwrap_or_default(),
                    Some(toks.get(t + 1..).unwrap_or_default()),
                ),
                None => (rest, None),
            };
            let Some(owner) = ender_owner(owner_toks, env) else {
                return Ok(None);
            };
            let to = match to {
                Some(t) => Some(parse_expr(t, env)?),
                None => None,
            };
            Some(eff(ShapeEffect::OpenEnder { owner, to }))
        }
        _ => None,
    })
}

/// `ender chest of P`, `P's ender chest`, `the ender chest of P`.
fn ender_owner(toks: &[Tok], env: &Env) -> Option<Expr> {
    match parse_expr(toks, env) {
        Ok(Expr::BlockShapes(n)) if n.op == Op::EnderChest => n.args.into_iter().next(),
        _ => None,
    }
}

// ---------------------------------------------------------------- pure helpers

/// Whole-block position of a location value.
pub fn block_pos(v: &Value) -> Option<BlockPos> {
    let Value::Location { x, y, z, world } = v else {
        return None;
    };
    if !(x.is_finite() && y.is_finite() && z.is_finite()) {
        return None;
    }
    Some(BlockPos {
        x: x.floor() as i32,
        y: y.floor() as i32,
        z: z.floor() as i32,
        world: world.clone(),
    })
}

fn loc(x: i32, y: i32, z: i32, world: &Option<String>) -> Value {
    Value::Location {
        x: f64::from(x),
        y: f64::from(y),
        z: f64::from(z),
        world: world.clone(),
    }
}

/// Every block whose center is within `r` of the point, nearest layers first by y, x, z.
pub fn sphere(cx: f64, cy: f64, cz: f64, r: f64, world: &Option<String>) -> Result<Vec<Value>, String> {
    if !(r.is_finite() && r >= 0.0) {
        return Err("the radius must be a positive number".into());
    }
    if r > MAX_RADIUS {
        return Err(format!("'blocks in radius' goes up to radius {MAX_RADIUS}"));
    }
    let (bx, by, bz) = (cx.floor() as i32, cy.floor() as i32, cz.floor() as i32);
    let ri = r.ceil() as i32;
    let r2 = r * r;
    let mut out = Vec::new();
    for dy in -ri..=ri {
        for dx in -ri..=ri {
            for dz in -ri..=ri {
                let (x, y, z) = (bx + dx, by + dy, bz + dz);
                let d = (f64::from(x) + 0.5 - cx).powi(2)
                    + (f64::from(y) + 0.5 - cy).powi(2)
                    + (f64::from(z) + 0.5 - cz).powi(2);
                if d <= r2 {
                    out.push(loc(x, y, z, world));
                }
            }
        }
    }
    Ok(out)
}

/// Every block of the box between two block positions (both included).
pub fn cuboid(a: &BlockPos, b: &BlockPos) -> Result<Vec<Value>, String> {
    let (x0, x1) = (a.x.min(b.x), a.x.max(b.x));
    let (y0, y1) = (a.y.min(b.y), a.y.max(b.y));
    let (z0, z1) = (a.z.min(b.z), a.z.max(b.z));
    let n = (i64::from(x1) - i64::from(x0) + 1)
        * (i64::from(y1) - i64::from(y0) + 1)
        * (i64::from(z1) - i64::from(z0) + 1);
    if n > MAX_CUBOID {
        return Err(format!(
            "'blocks between' gives at most {MAX_CUBOID} blocks, this box has {n}"
        ));
    }
    let w = a.world.clone().or_else(|| b.world.clone());
    let mut out = Vec::with_capacity(n as usize);
    for y in y0..=y1 {
        for x in x0..=x1 {
            for z in z0..=z1 {
                out.push(loc(x, y, z, &w));
            }
        }
    }
    Ok(out)
}

/// The blocks a straight line from `a` to `b` passes through, in order (3D DDA).
pub fn line(a: (f64, f64, f64), b: (f64, f64, f64), world: &Option<String>) -> Vec<Value> {
    let mut cur = (a.0.floor() as i32, a.1.floor() as i32, a.2.floor() as i32);
    let end = (b.0.floor() as i32, b.1.floor() as i32, b.2.floor() as i32);
    let d = (b.0 - a.0, b.1 - a.1, b.2 - a.2);
    let axis = |p: f64, dp: f64, c: i32| -> (i32, f64, f64) {
        if dp > 0.0 {
            (1, (f64::from(c) + 1.0 - p) / dp, 1.0 / dp)
        } else if dp < 0.0 {
            (-1, (p - f64::from(c)) / -dp, -1.0 / dp)
        } else {
            (0, f64::INFINITY, f64::INFINITY)
        }
    };
    let (sx, mut tx, dx) = axis(a.0, d.0, cur.0);
    let (sy, mut ty, dy) = axis(a.1, d.1, cur.1);
    let (sz, mut tz, dz) = axis(a.2, d.2, cur.2);
    let mut out = vec![loc(cur.0, cur.1, cur.2, world)];
    while cur != end && out.len() < MAX_LINE {
        if tx <= ty && tx <= tz {
            cur.0 += sx;
            tx += dx;
        } else if ty <= tz {
            cur.1 += sy;
            ty += dy;
        } else {
            cur.2 += sz;
            tz += dz;
        }
        out.push(loc(cur.0, cur.1, cur.2, world));
    }
    out
}

/// `RAY_LEN` blocks next to `from` in one direction.
pub fn ray(from: &BlockPos, dir: u8) -> Vec<Value> {
    let (dx, dy, dz) = DIRS[usize::from(dir.min(5))];
    (1..=RAY_LEN as i32)
        .map(|i| loc(from.x + dx * i, from.y + dy * i, from.z + dz * i, &from.world))
        .collect()
}

/// Level of an enchantment written like `sharpness 5`, `sharpness V` or `sharpness`.
pub fn ench_level(v: &Value) -> Option<f64> {
    let t = match v {
        Value::Text(t) => t.clone(),
        Value::Enum(_) => return Some(1.0),
        Value::Nbt(c) => {
            return match c.get("level") {
                Some(Tag::Int(n)) => Some(f64::from(*n)),
                _ => None,
            };
        }
        _ => return None,
    };
    let last = t.split_whitespace().last()?;
    if let Ok(n) = last.parse::<f64>() {
        return Some(n);
    }
    let roman = last.to_ascii_uppercase();
    let val = |c: char| match c {
        'I' => Some(1),
        'V' => Some(5),
        'X' => Some(10),
        'L' => Some(50),
        'C' => Some(100),
        _ => None,
    };
    if roman.chars().all(|c| val(c).is_some()) && t.split_whitespace().count() > 1 {
        let ds: Vec<i32> = roman.chars().filter_map(val).collect();
        let mut n = 0;
        for (i, d) in ds.iter().enumerate() {
            if ds.get(i + 1).is_some_and(|nx| nx > d) {
                n -= d;
            } else {
                n += d;
            }
        }
        return Some(f64::from(n));
    }
    Some(1.0)
}

/// Bukkit's inventory action for a click: from the click type, the clicked
/// slot's item and the cursor item.
/// A real stack, or `None` for the air (count 0) the host sends for an empty
/// slot or cursor.
fn stack(s: Option<&(String, u32)>) -> Option<&(String, u32)> {
    s.filter(|(id, n)| *n > 0 && id != "air")
}

pub fn inventory_action(
    click: &str,
    slot: i32,
    item: Option<&(String, u32)>,
    cursor: Option<&(String, u32)>,
) -> &'static str {
    let outside = slot < 0;
    match click {
        "left" | "right" if outside => match (cursor, click) {
            (None, _) => "nothing",
            (Some(_), "left") => "drop_all_cursor",
            _ => "drop_one_cursor",
        },
        "left" | "right" => {
            let right = click == "right";
            match (item, cursor) {
                (None, None) => "nothing",
                (Some((_, n)), None) => {
                    if right && *n > 1 {
                        "pickup_half"
                    } else {
                        "pickup_all"
                    }
                }
                (None, Some((_, n))) => {
                    if right && *n > 1 {
                        "place_one"
                    } else {
                        "place_all"
                    }
                }
                (Some((a, _)), Some((b, _))) if a != b => "swap_with_cursor",
                (Some((_, have)), Some((_, n))) => {
                    if right {
                        "place_one"
                    } else if have + n <= 64 {
                        "place_all"
                    } else {
                        "place_some"
                    }
                }
            }
        }
        "shift left" | "shift right" => {
            if item.is_some() {
                "move_to_other_inventory"
            } else {
                "nothing"
            }
        }
        "drop" | "control drop" => match (item, click) {
            (None, _) => "nothing",
            (Some(_), "drop") => "drop_one_slot",
            _ => "drop_all_slot",
        },
        "number key" => "hotbar_swap",
        "double click" => "collect_to_cursor",
        "middle" => {
            if item.is_some() {
                "clone_stack"
            } else {
                "nothing"
            }
        }
        _ => "unknown",
    }
}

// ---- written books (the `written_book_content` component, network form)

/// Title, author and plain-text pages of a book.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct BookData {
    pub title: String,
    pub author: String,
    pub pages: Vec<String>,
}

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
    for i in 0..5 {
        let x = *b.get(*at)?;
        *at += 1;
        n |= u32::from(x & 0x7f) << (7 * i);
        if x & 0x80 == 0 {
            return Some(n);
        }
    }
    None
}

fn put_str(s: &str, out: &mut Vec<u8>) {
    put_varint(s.len() as u32, out);
    out.extend_from_slice(s.as_bytes());
}

fn get_str(b: &[u8], at: &mut usize) -> Option<String> {
    let n = get_varint(b, at)? as usize;
    let s = b.get(*at..at.checked_add(n)?)?;
    *at += n;
    Some(String::from_utf8_lossy(s).into_owned())
}

fn get_bool(b: &[u8], at: &mut usize) -> Option<bool> {
    let x = *b.get(*at)?;
    *at += 1;
    Some(x != 0)
}

/// Plain text of a text component tag: a string, or a compound with
/// `text` and `extra`.
pub fn tag_text(t: &Tag) -> String {
    match t {
        Tag::Str(s) => s.clone(),
        Tag::Compound(c) => {
            let mut s = match c.get("text").or_else(|| c.get("")) {
                Some(Tag::Str(x)) => x.clone(),
                _ => String::new(),
            };
            if let Some(Tag::List(extra)) = c.get("extra") {
                for e in extra {
                    s.push_str(&tag_text(e));
                }
            }
            s
        }
        Tag::List(l) => l.iter().map(tag_text).collect(),
        _ => String::new(),
    }
}

impl BookData {
    pub fn encode(&self) -> Result<Vec<u8>, String> {
        let mut out = Vec::new();
        put_str(&self.title, &mut out);
        out.push(0);
        put_str(&self.author, &mut out);
        put_varint(0, &mut out);
        put_varint(self.pages.len() as u32, &mut out);
        for p in &self.pages {
            let mut c = Compound::default();
            c.insert("text".into(), Tag::Str(p.clone()));
            out.extend_from_slice(&c.to_bytes()?);
            out.push(0);
        }
        out.push(1);
        Ok(out)
    }

    pub fn decode(b: &[u8]) -> Option<BookData> {
        let mut at = 0;
        let title = get_str(b, &mut at)?;
        if get_bool(b, &mut at)? {
            get_str(b, &mut at)?;
        }
        let author = get_str(b, &mut at)?;
        get_varint(b, &mut at)?;
        let n = get_varint(b, &mut at)? as usize;
        let mut pages = Vec::with_capacity(n.min(100));
        for _ in 0..n {
            let t = crate::skbee_nbt::read_net_tag(b, &mut at).ok()?;
            pages.push(tag_text(&t));
            if get_bool(b, &mut at)? {
                crate::skbee_nbt::read_net_tag(b, &mut at).ok()?;
            }
        }
        Some(BookData { title, author, pages })
    }
}

/// An enchantment offer value: enchantment, level and cost.
pub fn offer_value(cost: f64, id: f64, level: f64) -> Value {
    let Some(name) = crate::item_meta::ENCHANTS.get(id as usize).filter(|_| id >= 0.0) else {
        return Value::None;
    };
    let mut c = Compound::default();
    c.insert("enchantment".into(), Tag::Str((*name).to_string()));
    c.insert("level".into(), Tag::Int(level as i32));
    c.insert("cost".into(), Tag::Int(cost as i32));
    Value::Nbt(Box::new(c))
}

// ---------------------------------------------------------------- running

fn border_world(v: &Value) -> Option<&str> {
    match v {
        Value::Border(w) | Value::World(w) | Value::Text(w) => Some(w),
        _ => None,
    }
}

impl Engine {
    fn shape_held(
        &mut self,
        e: &Expr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Option<PlayerRef>, String> {
        match e {
            Expr::Prop(crate::expr::Prop::Tool, p) => self.player(p, ex, host),
            _ => Err("this only works on a held item, like player's tool".into()),
        }
    }

    fn shape_book_data(
        &mut self,
        e: &Expr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Option<(PlayerRef, BookData)>, String> {
        let Some(p) = self.shape_held(e, ex, host)? else {
            return Ok(None);
        };
        let data = match host.block_shape_query(Q::HeldBook(&p)) {
            Some(Value::Text(_)) | None => BookData::default(),
            Some(Value::List(bytes)) => {
                let b: Vec<u8> = bytes.iter().filter_map(|v| v.as_num().map(|n| n as u8)).collect();
                BookData::decode(&b).unwrap_or_default()
            }
            Some(_) => BookData::default(),
        };
        Ok(Some((p, data)))
    }

    fn shape_entity_val(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        let v = self.eval(e, ex, host)?;
        Ok(match v {
            Value::Entity(_) => v,
            other => online(&other, &mut ex.seen, host).map_or(Value::None, Value::Player),
        })
    }

    /// The block position of a brewing stand value, with its world filled in.
    fn shape_brew_pos(
        &mut self,
        e: &Expr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Option<BlockPos>, String> {
        let v = self.eval(e, ex, host)?;
        let Some(mut pos) = block_pos(&v) else {
            return Ok(None);
        };
        if pos.world.is_none() {
            pos.world = host.default_world();
        }
        Ok(Some(pos))
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn shape_expr(
        &mut self,
        n: &Node,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        let arg = |i: usize| n.args.get(i).ok_or_else(|| "missing value".to_string());
        Ok(match n.op {
            Op::Sphere => {
                let r = self.num(arg(0)?, ex, host)?;
                let c = self.eval(arg(1)?, ex, host)?;
                match self.as_location(&c, ex, host) {
                    Some((x, y, z, w)) => Value::List(sphere(x, y, z, r, &w)?),
                    None => Value::None,
                }
            }
            Op::Cuboid | Op::Line => {
                let a = self.eval(arg(0)?, ex, host)?;
                let b = self.eval(arg(1)?, ex, host)?;
                let (Some(a), Some(b)) = (self.as_location(&a, ex, host), self.as_location(&b, ex, host))
                else {
                    return Ok(Value::None);
                };
                if n.op == Op::Cuboid {
                    let pa = block_pos(&Value::Location {
                        x: a.0,
                        y: a.1,
                        z: a.2,
                        world: a.3.clone(),
                    });
                    let pb = block_pos(&Value::Location {
                        x: b.0,
                        y: b.1,
                        z: b.2,
                        world: b.3,
                    });
                    match (pa, pb) {
                        (Some(pa), Some(pb)) => Value::List(cuboid(&pa, &pb)?),
                        _ => Value::None,
                    }
                } else {
                    // Through the block centers, like Skript's block line.
                    let c = |p: (f64, f64, f64, Option<String>)| {
                        (p.0.floor() + 0.5, p.1.floor() + 0.5, p.2.floor() + 0.5)
                    };
                    let w = a.3.clone();
                    Value::List(line(c(a), c(b), &w))
                }
            }
            Op::Ray(d) => {
                let v = self.eval(arg(0)?, ex, host)?;
                match self.as_location(&v, ex, host) {
                    Some((x, y, z, w)) => match block_pos(&Value::Location { x, y, z, world: w }) {
                        Some(p) => Value::List(ray(&p, d)),
                        None => Value::None,
                    },
                    None => Value::None,
                }
            }
            Op::Book(part) => {
                let Some((_, b)) = self.shape_book_data(arg(0)?, ex, host)? else {
                    return Ok(Value::None);
                };
                let opt = |s: String| if s.is_empty() { Value::None } else { Value::Text(s) };
                match part {
                    Book::Author => opt(b.author),
                    Book::Title => opt(b.title),
                    Book::Pages => Value::List(b.pages.into_iter().map(Value::Text).collect()),
                    Book::Page => {
                        let i = self.num(arg(1)?, ex, host)?;
                        if i < 1.0 {
                            Value::None
                        } else {
                            b.pages
                                .get(i as usize - 1)
                                .cloned()
                                .map_or(Value::None, Value::Text)
                        }
                    }
                }
            }
            Op::BarById => {
                let id = self.text(arg(0)?, ex, host)?;
                match host.block_shape_query(Q::BarExists(&id)) {
                    Some(Value::Bool(true)) => Value::Text(id),
                    _ => Value::None,
                }
            }
            Op::BrewSlot(s) => match self.shape_brew_pos(arg(0)?, ex, host)? {
                Some(pos) => host.block_shape_query(Q::Brew(&pos, s)).unwrap_or(Value::None),
                None => Value::None,
            },
            Op::Border(f) => {
                let v = self.eval(arg(0)?, ex, host)?;
                match border_world(&v) {
                    Some(w) => host.block_shape_query(Q::Border(w, f)).unwrap_or(Value::None),
                    None => Value::None,
                }
            }
            Op::Cursor => ex
                .ctx
                .inv
                .as_ref()
                .and_then(|c| stack(c.cursor.as_ref()).cloned())
                .map_or(Value::None, |(id, count)| Value::Item {
                    id,
                    count,
                    meta: None,
                }),
            Op::InvAction => {
                let Some(c) = ex.ctx.inv.as_ref() else {
                    return Ok(Value::None);
                };
                let name =
                    inventory_action(&c.click, c.slot, stack(c.item.as_ref()), stack(c.cursor.as_ref()));
                match n.ty {
                    Ty::T1(t) => crate::named_types::parse_text(t, name),
                    _ => Value::Text(name.replace('_', " ")),
                }
            }
            Op::Damage | Op::Durability => {
                let v = self.eval(arg(0)?, ex, host)?;
                let Value::Item { id, meta, .. } = &v else {
                    return Ok(Value::None);
                };
                let dmg = f64::from(meta.as_ref().and_then(|m| m.damage).unwrap_or(0));
                if n.op == Op::Damage {
                    Value::Num(dmg)
                } else {
                    match crate::expr_item_damage_facts::max_durability(id) {
                        Some(max) => Value::Num((f64::from(max) - dmg).max(0.0)),
                        None => Value::None,
                    }
                }
            }
            Op::WithDamage => {
                let v = self.eval(arg(0)?, ex, host)?;
                let d = self.num(arg(1)?, ex, host)?;
                let Value::Item { id, count, meta } = v else {
                    return Ok(Value::None);
                };
                let mut m = meta.unwrap_or_default();
                m.damage = Some(d.clamp(0.0, f64::from(i32::MAX)) as u32);
                Value::Item {
                    id,
                    count,
                    meta: Some(m),
                }
            }
            Op::EnderChest => match self.player(arg(0)?, ex, host)? {
                Some(p) => host.block_shape_query(Q::Ender(&p, None)).unwrap_or(Value::None),
                None => Value::None,
            },
            Op::EnderSlot => {
                let s = self.num(arg(0)?, ex, host)?;
                if !(0.0..27.0).contains(&s) {
                    return Ok(Value::None);
                }
                match self.player(arg(1)?, ex, host)? {
                    Some(p) => host
                        .block_shape_query(Q::Ender(&p, Some(s as u8)))
                        .unwrap_or(Value::None),
                    None => Value::None,
                }
            }
            Op::Ai => {
                let v = self.shape_entity_val(arg(0)?, ex, host)?;
                if v == Value::None {
                    return Ok(Value::None);
                }
                host.block_shape_query(Q::Ai(&v)).unwrap_or(Value::None)
            }
            Op::Attr(i, total) => {
                let v = self.shape_entity_val(arg(0)?, ex, host)?;
                let name = crate::data::named_types::ATTRIBUTES
                    .get(usize::from(i))
                    .copied()
                    .unwrap_or("");
                if v == Value::None {
                    return Ok(Value::None);
                }
                host.block_shape_query(Q::Attr(&v, name, total))
                    .unwrap_or(Value::None)
            }
            Op::CarriedBlock => match self.eval(arg(0)?, ex, host)? {
                Value::Entity(e) => host.block_shape_query(Q::CarriedBlock(&e)).unwrap_or(Value::None),
                _ => Value::None,
            },
            Op::EnchLevel => {
                let v = self.eval(arg(0)?, ex, host)?;
                ench_level(&v).map_or(Value::None, Value::Num)
            }
            Op::Offers => {
                let mut out = Vec::new();
                for i in 0..3 {
                    let o = shape_offer(&ex.ctx, i);
                    if o != Value::None {
                        out.push(o);
                    }
                }
                Value::List(out)
            }
            Op::Offer => {
                let i = self.num(arg(0)?, ex, host)?;
                if !(1.0..4.0).contains(&i) {
                    return Ok(Value::None);
                }
                shape_offer(&ex.ctx, i as u8 - 1)
            }
            Op::OfferCost => match self.eval(arg(0)?, ex, host)? {
                Value::Nbt(c) => match c.get("cost") {
                    Some(Tag::Int(n)) => Value::Num(f64::from(*n)),
                    _ => Value::None,
                },
                _ => Value::None,
            },
            Op::EventEntity => {
                let v = crate::combat::victim(ex, host);
                match (n.args.first(), &v) {
                    (_, Value::Player(_)) | (_, Value::None) => Value::None,
                    (None, _) => v,
                    (Some(k), Value::Entity(e)) => {
                        let k = self.text(k, ex, host)?;
                        if e.kind == k { v } else { Value::None }
                    }
                    _ => Value::None,
                }
            }
        })
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn shape_effect(
        &mut self,
        e: &ShapeEffect,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        match e {
            ShapeEffect::Change { target, mode, value } => {
                self.shape_change(target, *mode, value.as_ref(), ex, host)
            }
            ShapeEffect::OpenEnder { owner, to } => {
                let Some(owner) = self.player(owner, ex, host)? else {
                    return Ok(());
                };
                if let Some(to) = to {
                    let viewer = self.player(to, ex, host)?;
                    if viewer.as_ref().is_some_and(|v| v.uuid != owner.uuid) {
                        return Err("Pumpkin can only open a player's own ender chest".into());
                    }
                }
                host.block_shape_query(Q::OpenEnder(&owner));
                Ok(())
            }
            ShapeEffect::ClearEnder(owner) => {
                if let Some(p) = self.player(owner, ex, host)? {
                    host.block_shape_query(Q::ClearEnder(&p));
                }
                Ok(())
            }
        }
    }

    fn shape_change(
        &mut self,
        t: &Node,
        mode: Mode,
        value: Option<&Expr>,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        let val = match value {
            Some(v) => self.eval(v, ex, host)?,
            None => Value::None,
        };
        let arg = |i: usize| t.args.get(i).ok_or_else(|| "missing value".to_string());
        // Number targets: add/remove work on the current value.
        let number = |engine: &mut Self, ex: &mut Exec, host: &mut dyn Host| -> Result<Option<f64>, String> {
            let delta = match (&val, mode) {
                (_, Mode::Reset) => return Ok(None),
                (Value::Num(n), _) => *n,
                _ => return Err("expected a number".into()),
            };
            Ok(Some(match mode {
                Mode::Set => delta,
                _ => {
                    let cur = engine.shape_expr(t, ex, host)?.as_num().unwrap_or(0.0);
                    if mode == Mode::Add {
                        cur + delta
                    } else {
                        cur - delta
                    }
                }
            }))
        };
        match t.op {
            Op::Book(part) => {
                let Some((p, mut b)) = self.shape_book_data(arg(0)?, ex, host)? else {
                    return Ok(());
                };
                let s = match &val {
                    Value::None => String::new(),
                    v => v.to_string(),
                };
                match part {
                    Book::Author => b.author = s,
                    Book::Title => b.title = s,
                    Book::Pages => {
                        b.pages = match val {
                            Value::List(l) => l.iter().map(Value::to_string).collect(),
                            Value::None => Vec::new(),
                            v => vec![v.to_string()],
                        };
                    }
                    Book::Page => {
                        let i = self.num(arg(1)?, ex, host)?;
                        if !(1.0..=100.0).contains(&i) {
                            return Err("a book has pages 1 to 100".into());
                        }
                        let i = i as usize - 1;
                        while b.pages.len() <= i {
                            b.pages.push(String::new());
                        }
                        b.pages[i] = s;
                    }
                }
                let bytes = b.encode()?;
                host.block_shape_query(Q::SetHeldBook(&p, &bytes));
            }
            Op::BrewSlot(s) => {
                if let Some(pos) = self.shape_brew_pos(arg(0)?, ex, host)? {
                    host.block_shape_query(Q::SetBrew(&pos, s, &val));
                }
            }
            Op::Border(f) => {
                let w = self.eval(arg(0)?, ex, host)?;
                let Some(w) = border_world(&w).map(str::to_string) else {
                    return Ok(());
                };
                let v = if f == BorderField::Center {
                    match (&val, mode) {
                        (_, Mode::Reset) => Value::Location {
                            x: 0.0,
                            y: 0.0,
                            z: 0.0,
                            world: None,
                        },
                        (v @ Value::Location { .. }, Mode::Set) => v.clone(),
                        _ => return Err("the world border center must be set to a location".into()),
                    }
                } else {
                    match number(self, ex, host)? {
                        Some(n) => Value::Num(n.max(0.0)),
                        None => Value::Num(if f == BorderField::DamageAmount { 0.2 } else { 5.0 }),
                    }
                };
                host.block_shape_query(Q::SetBorder(&w, f, &v));
            }
            Op::Damage | Op::Durability => {
                let Some(p) = self.shape_held(arg(0)?, ex, host)? else {
                    return Ok(());
                };
                let Some((id, _)) = host.held_item(&p) else {
                    return Ok(());
                };
                let max = crate::expr_item_damage_facts::max_durability(&id);
                let n = number(self, ex, host)?.unwrap_or(if t.op == Op::Damage {
                    0.0
                } else {
                    f64::from(max.unwrap_or(0))
                });
                let dmg = if t.op == Op::Damage {
                    n
                } else {
                    match max {
                        Some(m) => f64::from(m) - n,
                        None => return Err(format!("{id} has no durability")),
                    }
                };
                let mut meta = host.held_item_meta(&p).unwrap_or_default();
                let dmg = dmg.clamp(0.0, f64::from(max.unwrap_or(i32::MAX as u32)));
                meta.damage = if dmg <= 0.0 { None } else { Some(dmg as u32) };
                host.set_held_item_meta(&p, &meta);
            }
            Op::EnderSlot => {
                let s = self.num(arg(0)?, ex, host)?;
                if !(0.0..27.0).contains(&s) {
                    return Err("an ender chest has slots 0 to 26".into());
                }
                if let Some(p) = self.player(arg(1)?, ex, host)? {
                    host.block_shape_query(Q::SetEnder(&p, s as u8, &val));
                }
            }
            Op::Ai => {
                let v = self.shape_entity_val(arg(0)?, ex, host)?;
                let on = match (&val, mode) {
                    (_, Mode::Reset) => true,
                    (Value::Bool(b), Mode::Set) => *b,
                    _ => return Err("ai must be set to true or false".into()),
                };
                if v != Value::None {
                    host.block_shape_query(Q::SetAi(&v, on));
                }
            }
            Op::Attr(i, _) => {
                let v = self.shape_entity_val(arg(0)?, ex, host)?;
                let name = crate::data::named_types::ATTRIBUTES
                    .get(usize::from(i))
                    .copied()
                    .unwrap_or("");
                if v == Value::None {
                    return Ok(());
                }
                // add/remove change the base value.
                let delta = match (&val, mode) {
                    (_, Mode::Reset) => {
                        host.block_shape_query(Q::ResetAttr(&v, name));
                        return Ok(());
                    }
                    (Value::Num(n), _) => *n,
                    _ => return Err("an attribute must be set to a number".into()),
                };
                let n = match mode {
                    Mode::Set => delta,
                    _ => {
                        let cur = host
                            .block_shape_query(Q::Attr(&v, name, false))
                            .and_then(|x| x.as_num())
                            .unwrap_or(0.0);
                        if mode == Mode::Add {
                            cur + delta
                        } else {
                            cur - delta
                        }
                    }
                };
                host.block_shape_query(Q::SetAttr(&v, name, n));
            }
            Op::CarriedBlock => {
                if let Value::Entity(e) = self.eval(arg(0)?, ex, host)? {
                    let data = match (&val, mode) {
                        (_, Mode::Reset) | (Value::None, _) => None,
                        (Value::Text(s), _) => Some(s.clone()),
                        (Value::Item { id, .. }, _) => Some(id.clone()),
                        _ => return Err("carried block must be a block type or block data text".into()),
                    };
                    host.block_shape_query(Q::SetCarriedBlock(&e, data.as_deref()));
                }
            }
            _ => return Err("that value can't be changed".into()),
        }
        Ok(())
    }
}

fn shape_offer(ctx: &crate::engine::EventCtx, i: u8) -> Value {
    use crate::events_game::{Slot, read};
    let num = |k: u8| read(ctx, Slot::Num(2 + i * 3 + k)).as_num();
    match (num(0), num(1), num(2)) {
        (Some(c), Some(id), Some(l)) => offer_value(c, id, l),
        _ => Value::None,
    }
}

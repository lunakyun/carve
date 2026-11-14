//! Blocks: `block at <location>`, `type of block`, `set block ... to stone`,
//! `break block ...`, `targeted block of player`, `block above/below player`
//! and the `block ... is <type>` condition.
//!
//! A block value is a [`Value::Location`] with whole-number coordinates. The
//! expression parser lives in `expr.rs` (`block_expr`); effects and conditions
//! are parsed here and run by the `Engine` methods at the bottom.

use crate::compile::{Cond, Effect};
use crate::engine::{Engine, Exec, online};
use crate::expr::{Env, Expr, Ty, parse_expr};
use crate::host::{Host, LogLevel};
use crate::lexer::Tok;
use crate::value::Value;

/// A block position: whole coordinates. `world: None` means the default world.
#[derive(Debug, Clone, PartialEq)]
pub struct BlockPos {
    pub x: i32,
    pub y: i32,
    pub z: i32,
    pub world: Option<String>,
}

/// How far `targeted block` looks.
pub const TARGET_RANGE: f64 = 100.0;
const MAX_COORD: f64 = 30_000_000.0;
const MAX_Y: f64 = 4096.0;

/// The rare block expressions. Boxed inside `Expr::Blk`.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockExpr {
    /// `block at <location|player>`: the block that holds the point.
    At(Expr),
    /// `block above/below/north of ...`: a neighbour of the block that holds the point.
    Rel(Expr, i32, i32, i32),
    /// `targeted block of <player>`.
    Targeted(Expr),
    /// `type of block ...`: the block type as an item.
    TypeOf(Expr),
}

impl BlockExpr {
    pub fn ty(&self) -> Ty {
        match self {
            BlockExpr::TypeOf(_) => Ty::Item,
            _ => Ty::Location,
        }
    }
}

/// `above`, `north of`, ... to a step. North is -z, east is +x.
pub fn direction(word: &str) -> Option<(i32, i32, i32)> {
    Some(match word {
        "above" | "over" => (0, 1, 0),
        "below" | "under" | "beneath" => (0, -1, 0),
        "north" => (0, 0, -1),
        "south" => (0, 0, 1),
        "east" => (1, 0, 0),
        "west" => (-1, 0, 0),
        _ => return None,
    })
}

#[derive(Debug, Clone, PartialEq)]
pub enum BlockFx {
    Set { at: Expr, to: Expr },
    Break(Expr),
}

#[derive(Debug, Clone, PartialEq)]
pub struct BlockCond {
    pub block: Expr,
    /// The block must be one of these types.
    pub types: Vec<Expr>,
    pub negate: bool,
}

// ---------- parsing ----------

fn is_block_expr(e: &Expr) -> bool {
    matches!(e, Expr::Blk(b) if b.ty() == Ty::Location)
}

/// `set <block> to <type>`, `break <block>`. `Ok(None)`: not a block line.
pub fn parse_effect(toks: &[Tok], env: &Env) -> Result<Option<Effect>, String> {
    let Some(Tok::Word(first)) = toks.first() else {
        return Ok(None);
    };
    let first = first.to_ascii_lowercase();
    match first.as_str() {
        "set" => {
            let Some(to) = toks.iter().position(|t| t.is_lit("to")) else {
                return Ok(None);
            };
            let (Some(left), Some(right)) = (toks.get(1..to), toks.get(to + 1..)) else {
                return Ok(None);
            };
            let looks_like_block = matches!(left.first(), Some(t) if t.is_lit("block") || t.is_lit("targeted") || t.is_lit("target"));
            if !looks_like_block {
                return Ok(None);
            }
            let at = parse_expr(left, env)?;
            if !is_block_expr(&at) {
                return Ok(None);
            }
            let to = parse_expr(right, env)?;
            if !matches!(to.ty(env), Ty::Item | Ty::Text | Ty::Object) {
                return Err("a block needs a block type after 'to', like stone".into());
            }
            Ok(Some(Effect::Block(Box::new(BlockFx::Set { at, to }))))
        }
        "break" | "destroy" => {
            let Some(rest) = toks.get(1..) else {
                return Ok(None);
            };
            if !matches!(rest.first(), Some(t) if t.is_lit("block") || t.is_lit("targeted") || t.is_lit("target")) {
                return Ok(None);
            }
            let at = parse_expr(rest, env)?;
            if !is_block_expr(&at) {
                return Ok(None);
            }
            Ok(Some(Effect::Block(Box::new(BlockFx::Break(at)))))
        }
        _ => Ok(None),
    }
}

/// `block at <loc> is stone [or dirt]`, `... is not air`.
pub fn parse_cond(toks: &[Tok], env: &Env) -> Option<Cond> {
    let first = toks.first()?;
    if !(first.is_lit("block") || first.is_lit("targeted") || first.is_lit("target")) {
        return None;
    }
    // The first `is` splits subject and types.
    let i = toks.iter().position(|t| t.is_lit("is"))?;
    let mut negate = false;
    let mut from = i + 1;
    if toks.get(from).is_some_and(|t| t.is_lit("not")) {
        negate = !negate;
        from += 1;
    }
    if toks.get(from).is_some_and(|t| t.is_lit("a") || t.is_lit("an")) {
        from += 1;
    }
    let block = parse_expr(toks.get(..i)?, env).ok()?;
    if !is_block_expr(&block) {
        return None;
    }
    let mut types = Vec::new();
    for part in toks.get(from..)?.split(|t| t.is_lit("or")) {
        let e = parse_expr(part, env).ok()?;
        if !matches!(e.ty(env), Ty::Item | Ty::Text | Ty::Object) {
            return None;
        }
        types.push(e);
    }
    if types.is_empty() {
        return None;
    }
    Some(Cond::Block(Box::new(BlockCond { block, types, negate })))
}

// ---------- running ----------

/// A block id from a value: `stone`, `minecraft:stone`, `Oak Planks`.
/// `None` when it is not a plain block name (nothing odd reaches a command).
pub fn block_id(v: &Value) -> Option<String> {
    let raw = match v {
        Value::Item { id, .. } => id.clone(),
        Value::Text(t) => t.trim().to_ascii_lowercase().replace(' ', "_"),
        _ => return None,
    };
    let id = raw.strip_prefix("minecraft:").unwrap_or(&raw);
    let ok = !id.is_empty()
        && id.len() <= 64
        && id.bytes().all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'_');
    ok.then(|| id.to_string())
}

/// The block that holds a point. `None` when a coordinate is not usable.
pub fn pos_of(x: f64, y: f64, z: f64, world: Option<String>) -> Option<BlockPos> {
    let ok = x.is_finite()
        && y.is_finite()
        && z.is_finite()
        && x.abs() < MAX_COORD
        && z.abs() < MAX_COORD
        && y.abs() < MAX_Y;
    ok.then(|| BlockPos {
        x: x.floor() as i32,
        y: y.floor() as i32,
        z: z.floor() as i32,
        world,
    })
}

fn block_value(p: &BlockPos) -> Value {
    Value::Location {
        x: f64::from(p.x),
        y: f64::from(p.y),
        z: f64::from(p.z),
        world: p.world.clone(),
    }
}

impl Engine {
    /// The block a location or player value stands in.
    fn block_pos(&mut self, v: &Value, ex: &mut Exec, host: &mut dyn Host) -> Option<BlockPos> {
        let (x, y, z, world) = self.as_location(v, ex, host)?;
        pos_of(x, y, z, world)
    }

    #[inline(never)]
    pub(crate) fn block_eval(
        &mut self,
        b: &BlockExpr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        match b {
            BlockExpr::At(e) => {
                let v = self.eval(e, ex, host)?;
                Ok(self.block_pos(&v, ex, host).map_or(Value::None, |p| block_value(&p)))
            }
            BlockExpr::Rel(e, dx, dy, dz) => {
                let v = self.eval(e, ex, host)?;
                let Some(mut p) = self.block_pos(&v, ex, host) else {
                    return Ok(Value::None);
                };
                p.x = p.x.saturating_add(*dx);
                p.y = p.y.saturating_add(*dy);
                p.z = p.z.saturating_add(*dz);
                Ok(block_value(&p))
            }
            BlockExpr::Targeted(e) => {
                let v = self.eval(e, ex, host)?;
                let Some(p) = online(&v, &mut ex.seen, host) else {
                    return Ok(Value::None);
                };
                Ok(host.targeted_block(&p, TARGET_RANGE).map_or(Value::None, |b| block_value(&b)))
            }
            BlockExpr::TypeOf(e) => {
                let v = self.eval(e, ex, host)?;
                if let Value::Item { id, .. } = v {
                    return Ok(Value::Item { id, count: 1 });
                }
                let Some(p) = self.block_pos(&v, ex, host) else {
                    return Ok(Value::None);
                };
                Ok(host
                    .block_type(&p)
                    .map_or(Value::None, |id| Value::Item { id, count: 1 }))
            }
        }
    }

    #[inline(never)]
    pub(crate) fn block_effect(
        &mut self,
        fx: &BlockFx,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        match fx {
            BlockFx::Set { at, to } => {
                let target = self.eval(at, ex, host)?;
                let ty = self.eval(to, ex, host)?;
                let Some(id) = block_id(&ty) else {
                    return Err(format!("'{ty}' is not a block type"));
                };
                match self.block_pos(&target, ex, host) {
                    Some(p) => host.set_block(&p, &id),
                    None => host.log(LogLevel::Warn, "set block: no block at that place"),
                }
            }
            BlockFx::Break(at) => {
                let target = self.eval(at, ex, host)?;
                match self.block_pos(&target, ex, host) {
                    Some(p) => host.break_block(&p),
                    None => host.log(LogLevel::Warn, "break block: no block at that place"),
                }
            }
        }
        Ok(())
    }

    #[inline(never)]
    pub(crate) fn block_cond(
        &mut self,
        c: &BlockCond,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<bool, String> {
        let v = self.eval(&c.block, ex, host)?;
        let Some(p) = self.block_pos(&v, ex, host) else {
            return Ok(false);
        };
        let Some(actual) = host.block_type(&p) else {
            return Ok(false);
        };
        let mut hit = false;
        for t in &c.types {
            let tv = self.eval(t, ex, host)?;
            if block_id(&tv).is_some_and(|id| id == actual) {
                hit = true;
                break;
            }
        }
        Ok(hit != c.negate)
    }
}

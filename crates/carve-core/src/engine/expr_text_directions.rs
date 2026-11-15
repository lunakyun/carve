//! Running the `expr_text_directions` expressions and effects.

use super::{Engine, Exec, online, to_num_opt};
use crate::expr::Expr;
use crate::expr_text_directions::{self as text, TextEffOp, TextExpr, TextOp};
use crate::host::Host;
use crate::value::Value;

impl Engine {
    pub(super) fn eval_text_directions(
        &mut self,
        w: &TextExpr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        let arg = |i: usize| w.args.get(i);
        Ok(match w.op {
            TextOp::Codepoint => {
                let Some(a) = arg(0) else { return Ok(Value::None) };
                let t = self.text(a, ex, host)?;
                text::codepoint(&t).map_or(Value::None, Value::Num)
            }
            TextOp::FromCodepoint => {
                let Some(a) = arg(0) else { return Ok(Value::None) };
                let n = self.num(a, ex, host)?;
                text::from_codepoint(n).map_or(Value::None, Value::Text)
            }
            TextOp::CharsBetween { alnum } => {
                let (Some(a), Some(b)) = (arg(0), arg(1)) else {
                    return Ok(Value::None);
                };
                let a = self.text(a, ex, host)?;
                let b = self.text(b, ex, host)?;
                Value::List(
                    text::chars_between(&a, &b, alnum)
                        .into_iter()
                        .map(Value::Text)
                        .collect(),
                )
            }
            TextOp::CommandArgs => {
                let (Some(full), Some(name)) = (arg(0), arg(1)) else {
                    return Ok(Value::None);
                };
                let full = self.text(full, ex, host)?;
                let name = self.text(name, ex, host)?;
                let full = full.trim_start().trim_start_matches('/');
                let rest = full.get(..name.len()).filter(|h| h.eq_ignore_ascii_case(&name));
                Value::Text(
                    rest.map_or("", |_| full.get(name.len()..).unwrap_or(""))
                        .trim()
                        .to_string(),
                )
            }
            TextOp::MenuTitle => match ex.ctx.inv.as_deref().and_then(|c| c.menu.clone()) {
                Some((_, title, _)) => Value::Text(title),
                None => Value::None,
            },
            TextOp::Difference => {
                let (Some(a), Some(b)) = (arg(0), arg(1)) else {
                    return Ok(Value::None);
                };
                let x = self.eval(a, ex, host)?;
                let y = self.eval(b, ex, host)?;
                match (&x, &y) {
                    (Value::Date(p), Value::Date(q)) => Value::Timespan {
                        millis: p.checked_sub(*q).map_or(i64::MAX, i64::abs),
                    },
                    (Value::Timespan { millis: p }, Value::Timespan { millis: q }) => Value::Timespan {
                        millis: p.checked_sub(*q).map_or(i64::MAX, i64::abs),
                    },
                    _ => match (to_num_opt(&x), to_num_opt(&y)) {
                        (Some(p), Some(q)) => Value::Num((p - q).abs()),
                        _ => Value::None,
                    },
                }
            }
            TextOp::Offset(dx, dy, dz) => {
                let (Some(n), Some(l)) = (arg(0), arg(1)) else {
                    return Ok(Value::None);
                };
                let n = self.num(n, ex, host)?;
                let l = self.eval(l, ex, host)?;
                match self.as_location(&l, ex, host) {
                    Some((x, y, z, world)) => Value::Location {
                        x: x + n * f64::from(dx),
                        y: y + n * f64::from(dy),
                        z: z + n * f64::from(dz),
                        world,
                    },
                    None => Value::None,
                }
            }
            TextOp::Facing(fwd, side) => {
                let (Some(n), Some(l)) = (arg(0), arg(1)) else {
                    return Ok(Value::None);
                };
                let n = self.num(n, ex, host)?;
                let l = self.eval(l, ex, host)?;
                let Some((x, y, z, world)) = self.as_location(&l, ex, host) else {
                    return Ok(Value::None);
                };
                // Players face where they look; other places face south (yaw 0).
                let yaw = match online(&l, &mut ex.seen, host) {
                    Some(p) => host
                        .pstat(&p, crate::expr_player_state::PStat::Yaw)
                        .unwrap_or(0.0)
                        .to_radians(),
                    None => 0.0,
                };
                let (fx, fz) = (-yaw.sin(), yaw.cos());
                let (rx, rz) = (-yaw.cos(), -yaw.sin());
                let (f, s) = (f64::from(fwd), f64::from(side));
                Value::Location {
                    x: x + n * (f * fx + s * rx),
                    y,
                    z: z + n * (f * fz + s * rz),
                    world,
                }
            }
            TextOp::ChunkX | TextOp::ChunkZ => {
                let Some(l) = arg(0) else { return Ok(Value::None) };
                let l = self.eval(l, ex, host)?;
                if let Value::Chunk { x, z, .. } = l {
                    return Ok(Value::Num(f64::from(if w.op == TextOp::ChunkX { x } else { z })));
                }
                match self.as_location(&l, ex, host) {
                    Some((x, _, z, _)) => {
                        Value::Num(text::chunk_of(if w.op == TextOp::ChunkX { x } else { z }))
                    }
                    None => Value::None,
                }
            }
            TextOp::CompassTarget => {
                let Some(a) = arg(0) else { return Ok(Value::None) };
                let v = self.eval(a, ex, host)?;
                match online(&v, &mut ex.seen, host).and_then(|p| host.compass_target(&p)) {
                    Some((x, y, z, world)) => Value::Location { x, y, z, world },
                    None => Value::None,
                }
            }
            TextOp::DeathMessage => ex.ctx.death_message.clone().map_or(Value::None, Value::Text),
            TextOp::Victim => crate::combat::victim(ex, host),
            TextOp::DamageCause => ex.ctx.damage_cause.clone().map_or(Value::None, Value::Text),
            TextOp::ChatRecipients => match &ex.ctx.recipients {
                Some(r) => Value::List(r.iter().cloned().map(Value::Player).collect()),
                None => Value::None,
            },
        })
    }

    pub(super) fn run_text_directions(
        &mut self,
        kind: TextEffOp,
        args: &(Expr, Expr),
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        let Some(p) = self.player(&args.0, ex, host)? else {
            return Ok(());
        };
        match kind {
            TextEffOp::SetCompass => {
                let l = self.eval(&args.1, ex, host)?;
                match self.as_location(&l, ex, host) {
                    Some((x, y, z, world)) => host.set_compass_target(&p, x, y, z, world.as_deref()),
                    None => return Err("'set compass target' needs a location".into()),
                }
            }
            TextEffOp::ResetCompass => host.set_compass_target(&p, 0.0, 0.0, 0.0, None),
        }
        Ok(())
    }
}

//! Running the `w2_expressions_2` expressions and effects.

use super::{Engine, Exec, online, to_num_opt};
use crate::expr::Expr;
use crate::host::Host;
use crate::value::Value;
use crate::w2_expressions_2::{self as w2, W2, W2Eff, W2Op};

impl Engine {
    pub(super) fn eval_x2(&mut self, w: &W2, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        let arg = |i: usize| w.args.get(i);
        Ok(match w.op {
            W2Op::Codepoint => {
                let Some(a) = arg(0) else { return Ok(Value::None) };
                let t = self.text(a, ex, host)?;
                w2::codepoint(&t).map_or(Value::None, Value::Num)
            }
            W2Op::FromCodepoint => {
                let Some(a) = arg(0) else { return Ok(Value::None) };
                let n = self.num(a, ex, host)?;
                w2::from_codepoint(n).map_or(Value::None, Value::Text)
            }
            W2Op::CharsBetween { alnum } => {
                let (Some(a), Some(b)) = (arg(0), arg(1)) else { return Ok(Value::None) };
                let a = self.text(a, ex, host)?;
                let b = self.text(b, ex, host)?;
                Value::List(w2::chars_between(&a, &b, alnum).into_iter().map(Value::Text).collect())
            }
            W2Op::Difference => {
                let (Some(a), Some(b)) = (arg(0), arg(1)) else { return Ok(Value::None) };
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
            W2Op::Offset(dx, dy, dz) => {
                let (Some(n), Some(l)) = (arg(0), arg(1)) else { return Ok(Value::None) };
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
            W2Op::ChunkX | W2Op::ChunkZ => {
                let Some(l) = arg(0) else { return Ok(Value::None) };
                let l = self.eval(l, ex, host)?;
                match self.as_location(&l, ex, host) {
                    Some((x, _, z, _)) => Value::Num(w2::chunk_of(if w.op == W2Op::ChunkX { x } else { z })),
                    None => Value::None,
                }
            }
            W2Op::CompassTarget => {
                let Some(a) = arg(0) else { return Ok(Value::None) };
                let v = self.eval(a, ex, host)?;
                match online(&v, &mut ex.seen, host).and_then(|p| host.compass_target(&p)) {
                    Some((x, y, z, world)) => Value::Location { x, y, z, world },
                    None => Value::None,
                }
            }
            W2Op::DeathMessage => ex.ctx.death_message.clone().map_or(Value::None, Value::Text),
            W2Op::DamageCause => ex.ctx.damage_cause.clone().map_or(Value::None, Value::Text),
            W2Op::ChatRecipients => match &ex.ctx.recipients {
                Some(r) => Value::List(r.iter().cloned().map(Value::Player).collect()),
                None => Value::None,
            },
        })
    }

    pub(super) fn run_x2(
        &mut self,
        kind: W2Eff,
        args: &(Expr, Expr),
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        let Some(p) = self.player(&args.0, ex, host)? else {
            return Ok(());
        };
        match kind {
            W2Eff::SetCompass => {
                let l = self.eval(&args.1, ex, host)?;
                match self.as_location(&l, ex, host) {
                    Some((x, y, z, world)) => host.set_compass_target(&p, x, y, z, world.as_deref()),
                    None => return Err("'set compass target' needs a location".into()),
                }
            }
            W2Eff::ResetCompass => host.set_compass_target(&p, 0.0, 0.0, 0.0, None),
        }
        Ok(())
    }
}

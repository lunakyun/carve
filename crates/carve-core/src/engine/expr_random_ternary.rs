//! Running the wave-2 expressions-6 variants (syntax in `expr_random_ternary.rs`).

use super::{Engine, Exec, online};
use crate::expr::VarRef;
use crate::expr_random_ternary::{REDUCED_KEY, Read, W6, W6Effect, W6Op};
use crate::host::Host;
use crate::list_ops::INPUT_KEY;
use crate::value::Value;

/// Milliseconds in one server tick.
const TICK_MS: i64 = 50;

impl Engine {
    pub(super) fn eval_w6(&mut self, w: &W6, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        Ok(match w {
            W6::Ternary(c, a, b) => {
                if self.cond(c, ex, host)? {
                    self.eval(a, ex, host)?
                } else {
                    self.eval(b, ex, host)?
                }
            }
            W6::RandomChar { alnum, from, to } => {
                let range = match (from, to) {
                    (Some(a), Some(b)) => {
                        let first = |s: String| s.chars().next();
                        let a = self.text(a, ex, host)?;
                        let b = self.text(b, ex, host)?;
                        match (first(a), first(b)) {
                            (Some(a), Some(b)) => Some((a, b)),
                            _ => return Ok(Value::None),
                        }
                    }
                    _ => None,
                };
                let r = self.next_random(host);
                Value::Text(crate::expr_random_ternary::pick_char(*alnum, range, r).to_string())
            }
            W6::RandomUuid => {
                let mut bytes = [0u8; 16];
                for b in &mut bytes {
                    *b = (self.next_random(host) * 256.0) as u8;
                }
                Value::Text(crate::expr_random_ternary::uuid_from(bytes))
            }
            W6::Raw(e) => {
                let t = self.text(e, ex, host)?;
                Value::Text(crate::expr_random_ternary::raw(&t))
            }
            W6::Repeat(e, n) => {
                let t = self.text(e, ex, host)?;
                let n = self.num(n, ex, host)?;
                match crate::expr_random_ternary::repeat(&t, n) {
                    Some(s) => {
                        self.charge(s.len() / 64)?;
                        Value::Text(s)
                    }
                    None => {
                        return Err(format!(
                            "'repeated' would make a text longer than {} characters",
                            crate::expr_random_ternary::MAX_REPEAT_CHARS
                        ));
                    }
                }
            }
            W6::Reduce(src, body) => {
                let items = self.eval_list(src, ex, host)?;
                let saved_in = ex.locals.get(INPUT_KEY).cloned();
                let saved_acc = ex.locals.get(REDUCED_KEY).cloned();
                let mut it = items.into_iter();
                let mut acc = it.next().unwrap_or(Value::None);
                let mut failed = None;
                for item in it {
                    ex.locals.set(INPUT_KEY, item);
                    ex.locals.set(REDUCED_KEY, acc.clone());
                    match self.eval(body, ex, host) {
                        Ok(v) => acc = v,
                        Err(e) => {
                            failed = Some(e);
                            break;
                        }
                    }
                }
                ex.locals.set(INPUT_KEY, saved_in.unwrap_or(Value::None));
                ex.locals.set(REDUCED_KEY, saved_acc.unwrap_or(Value::None));
                if let Some(e) = failed {
                    return Err(e);
                }
                acc
            }
            W6::RecursiveSize(v) => Value::Num(self.recursive_items(v, ex, host)?.len() as f64),
            W6::Read(r, of) => self.read_w6(*r, of.as_ref(), ex, host)?,
        })
    }

    /// Every value under the list `v`, at any depth.
    pub(super) fn recursive_items(
        &mut self,
        v: &VarRef,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Vec<Value>, String> {
        let key = self.key(v, ex, host)?;
        let prefix = key.strip_suffix('*').unwrap_or(&key);
        let all = if v.local {
            ex.locals.all_under(prefix)
        } else {
            self.vars.all_under(prefix)
        };
        Ok(all.into_iter().map(|(_, x)| x).collect())
    }

    fn read_w6(
        &mut self,
        r: Read,
        of: Option<&crate::expr::Expr>,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        let operand = match of {
            Some(e) => Some(self.eval(e, ex, host)?),
            None => None,
        };
        let player = |v: &Option<Value>, ex: &mut Exec, host: &mut dyn Host| {
            v.as_ref().and_then(|v| online(v, &mut ex.seen, host))
        };
        Ok(match r {
            Read::RemainingAir => match player(&operand, ex, host).and_then(|p| host.remaining_air(&p)) {
                Some(t) => Value::Timespan { millis: t * TICK_MS },
                None => Value::None,
            },
            Read::RespawnLocation => match player(&operand, ex, host).and_then(|p| host.respawn_location(&p))
            {
                Some((x, y, z)) => Value::Location { x, y, z, world: None },
                None => Value::None,
            },
            Read::SeaLevel => {
                let world = match &operand {
                    Some(Value::World(w) | Value::Text(w)) => Some(w.clone()),
                    Some(Value::Location { world, .. }) => world.clone(),
                    Some(v @ Value::Player(_)) => {
                        online(v, &mut ex.seen, host).and_then(|p| host.player_location(&p).map(|l| l.3))
                    }
                    _ => None,
                };
                match world.and_then(|w| host.sea_level(&w)) {
                    Some(n) => Value::Num(n as f64),
                    None => Value::None,
                }
            }
            Read::SimulationDistance => host
                .simulation_distance()
                .map_or(Value::None, |n| Value::Num(n as f64)),
            Read::Tablist => Value::List(host.online_players().into_iter().map(Value::Player).collect()),
        })
    }

    pub(super) fn exec_w6(&mut self, e: &W6Effect, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        let value = self.eval(&e.value, ex, host)?;
        let Some(p) = self.player(&e.player, ex, host)? else {
            return Ok(());
        };
        match e.op {
            W6Op::SetAir | W6Op::AddAir | W6Op::RemoveAir => {
                let Value::Timespan { millis } = value else {
                    return Err(format!(
                        "expected a timespan, got {} ({})",
                        value.type_name(),
                        value
                    ));
                };
                let ticks = millis / TICK_MS;
                let new = match e.op {
                    W6Op::SetAir => ticks,
                    _ => {
                        let cur = host.remaining_air(&p).unwrap_or(0);
                        if e.op == W6Op::AddAir {
                            cur + ticks
                        } else {
                            cur - ticks
                        }
                    }
                };
                host.set_remaining_air(&p, new);
            }
            W6Op::SetRespawn => match value {
                Value::Location { x, y, z, .. } => host.set_respawn_location(&p, x, y, z),
                v => return Err(format!("expected a location, got {} ({})", v.type_name(), v)),
            },
        }
        Ok(())
    }
}

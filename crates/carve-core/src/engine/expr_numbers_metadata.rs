//! Running the wave 2 expressions-5 expressions and effects
//! (see `expr_numbers_metadata.rs` for the syntax side).

use super::{Engine, Exec};
use crate::expr::Expr;
use crate::expr_numbers_metadata::{self as numbers, DataKind, NumberEffect, NumberFact, NumberOp};
use crate::host::Host;
use crate::value::Value;

impl Engine {
    /// The world name a value points to: a world, a location, a player, or a name.
    fn number_world_name(&mut self, v: &Value, ex: &mut Exec, host: &mut dyn Host) -> Option<String> {
        match v {
            Value::World(w) => Some(w.clone()),
            other => self.as_location(other, ex, host).and_then(|(_, _, _, w)| w),
        }
    }

    pub(super) fn eval_numbers_metadata(
        &mut self,
        e: &NumberFact,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        Ok(match e {
            NumberFact::Numbers(a, b, ints) => {
                let (a, b) = (self.num(a, ex, host)?, self.num(b, ex, host)?);
                if !a.is_finite() || !b.is_finite() {
                    return Err("numbers from <a> to <b> needs finite numbers".into());
                }
                let (start, end) = if *ints {
                    if a <= b {
                        (a.ceil(), b.floor())
                    } else {
                        (a.floor(), b.ceil())
                    }
                } else {
                    (a, b)
                };
                let step = if end >= start { 1.0 } else { -1.0 };
                let count = ((end - start).abs() + 1e-9).floor() + 1.0;
                if count > numbers::MAX_NUMBERS.min(2.0 * self.budget as f64) {
                    return Err(format!(
                        "numbers from <a> to <b> would make too many numbers (at most {} in one run)",
                        (2 * self.max_instructions).min(numbers::MAX_NUMBERS as usize)
                    ));
                }
                let mut out = Vec::with_capacity(count as usize);
                let mut i = 0.0;
                while i < count {
                    out.push(Value::Num(start + step * i));
                    i += 1.0;
                }
                Value::List(out)
            }
            NumberFact::Percent(p, of) => {
                let (p, of) = (self.num(p, ex, host)?, self.num(of, ex, host)?);
                Value::Num(p / 100.0 * of)
            }
            NumberFact::CountChars(kind, t) => {
                let t = self.text(t, ex, host)?;
                Value::Num(numbers::count_chars(*kind, &t) as f64)
            }
            NumberFact::Midpoint(a, b) => {
                let (a, b) = (self.eval(a, ex, host)?, self.eval(b, ex, host)?);
                match (self.as_location(&a, ex, host), self.as_location(&b, ex, host)) {
                    (Some((ax, ay, az, aw)), Some((bx, by, bz, bw))) => {
                        if aw.is_some() && bw.is_some() && aw != bw {
                            return Err("midpoint of locations in different worlds".into());
                        }
                        Value::Location {
                            x: (ax + bx) / 2.0,
                            y: (ay + by) / 2.0,
                            z: (az + bz) / 2.0,
                            world: aw.or(bw),
                        }
                    }
                    _ => Value::None,
                }
            }
            NumberFact::MiddleOf(a) => {
                let a = self.eval(a, ex, host)?;
                match self.as_location(&a, ex, host) {
                    Some((x, y, z, world)) => Value::Location {
                        x: x.floor() + 0.5,
                        y: y.floor() + 0.5,
                        z: z.floor() + 0.5,
                        world,
                    },
                    None => Value::None,
                }
            }
            NumberFact::MaxStack(i) => match self.eval(i, ex, host)? {
                Value::Item { id, .. } => {
                    Value::Num(f64::from(crate::parsing_loot_components::max_stack(&id, host)))
                }
                _ => Value::None,
            },
            NumberFact::MoonPhase(w) => {
                let v = self.eval(w, ex, host)?;
                match self
                    .number_world_name(&v, ex, host)
                    .and_then(|n| host.world_time(Some(&n)))
                    .and_then(|t| u64::try_from(t).ok())
                {
                    Some(t) => Value::Text(numbers::moon_phase(t).to_string()),
                    None => Value::None,
                }
            }
            NumberFact::ListName(p) => match self.player(p, ex, host)? {
                Some(p) => match host.tab_list_name(&p) {
                    Some(n) => Value::Text(n),
                    None => Value::Text(p.name),
                },
                None => Value::None,
            },
            NumberFact::Data(kind, key, target) => {
                let key = self.text(key, ex, host)?;
                let t = self.eval(target, ex, host)?;
                let Some(id) = numbers::target_id(&t) else {
                    return Ok(Value::None);
                };
                let k = numbers::data_key(*kind, &id, &key);
                match kind {
                    DataKind::Metadata => self.numbers.metadata.get(&k).cloned().unwrap_or(Value::None),
                    DataKind::Persistent => self.vars.get(&k),
                }
            }
            NumberFact::ParseError => match &self.numbers.last_parse_error {
                Some(m) => Value::Text(m.clone()),
                None => Value::None,
            },
            NumberFact::Plain(i) => match self.eval(i, ex, host)? {
                v @ Value::Item { .. } => v,
                _ => Value::None,
            },
        })
    }

    pub(super) fn run_number_effect(
        &mut self,
        e: &NumberEffect,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        match e.op {
            NumberOp::SetListName | NumberOp::ResetListName | NumberOp::SetListPriority => {
                let Some(p) = self.player(&e.target, ex, host)? else {
                    return Ok(());
                };
                match e.op {
                    NumberOp::SetListName => {
                        let t = self.number_arg_text(&e.a, ex, host)?;
                        host.set_tab_list_name(&p, Some(&t));
                    }
                    NumberOp::ResetListName => host.set_tab_list_name(&p, None),
                    _ => {
                        let Some(a) = &e.a else { return Ok(()) };
                        let n = self.num(a, ex, host)?;
                        host.set_list_order(&p, n as i32);
                    }
                }
            }
            NumberOp::SetData(kind) | NumberOp::DeleteData(kind) => {
                let key = self.number_arg_text(&e.a, ex, host)?;
                let t = self.eval(&e.target, ex, host)?;
                let Some(id) = numbers::target_id(&t) else {
                    return Ok(());
                };
                let k = numbers::data_key(kind, &id, &key);
                let value = if matches!(e.op, NumberOp::SetData(_)) {
                    let Some(b) = &e.b else { return Ok(()) };
                    match self.eval(b, ex, host)? {
                        Value::List(_) => return Err("a data value can't be a list".into()),
                        v => Some(v),
                    }
                } else {
                    None
                };
                match (kind, value) {
                    (DataKind::Metadata, Some(v)) => {
                        self.numbers.metadata.insert(k, v);
                    }
                    (DataKind::Metadata, None) => {
                        self.numbers.metadata.remove(&k);
                    }
                    (DataKind::Persistent, Some(v)) => {
                        self.vars.set(k, v);
                        self.mark_dirty(host);
                    }
                    (DataKind::Persistent, None) => {
                        self.vars.delete(&k);
                        self.mark_dirty(host);
                    }
                }
            }
        }
        Ok(())
    }

    fn number_arg_text(
        &mut self,
        e: &Option<Expr>,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<String, String> {
        match e {
            Some(e) => self.text(e, ex, host),
            None => Ok(String::new()),
        }
    }
}

//! Running the `effects_player_state` effects (see that module for the syntax side).

use super::{Engine, Exec};
use crate::effects_player_state::{Arg, Args, Transform, W2Eff, W2Op, run};
use crate::host::Host;
use crate::list_ops::INPUT_KEY;
use crate::value::Value;

impl Engine {
    #[inline(never)]
    pub(super) fn exec_w2(&mut self, w: &W2Eff, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        if w.op == W2Op::Toggle {
            return self.exec_toggle(w, ex, host);
        }
        let mut a = Args::default();
        let (mut pi, mut ti) = (0, 0);
        for (kind, expr) in w.op.layout().iter().zip(&w.args) {
            let Some(expr) = expr else {
                // An absent optional part still takes its slot.
                match kind {
                    Arg::Player => pi += 1,
                    Arg::Text => ti += 1,
                    _ => {}
                }
                continue;
            };
            match kind {
                Arg::Player => {
                    if let Some(slot) = a.players.get_mut(pi) {
                        *slot = self.player(expr, ex, host)?;
                    }
                    pi += 1;
                }
                Arg::Text => {
                    let t = self.text(expr, ex, host)?;
                    if let Some(slot) = a.texts.get_mut(ti) {
                        *slot = Some(t);
                    }
                    ti += 1;
                }
                Arg::Span => {
                    a.span_ms = match self.eval(expr, ex, host)? {
                        Value::Timespan { millis } => Some(millis),
                        Value::Num(n) if n.is_finite() => Some((n * 1000.0) as i64),
                        v => return Err(format!("expected a timespan, got {} ({})", v.type_name(), v)),
                    };
                }
                Arg::Var => {}
            }
        }
        run(w.op, &a, host);
        Ok(())
    }

    /// `toggle {x}`: flip a boolean. An unset variable becomes true.
    fn exec_toggle(&mut self, w: &W2Eff, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        let Some(v) = &w.var else { return Ok(()) };
        let k = self.key(v, ex, host)?.into_owned();
        let cur = if v.local {
            ex.locals.get(&k)
        } else {
            self.vars.get_ref(&k)
        };
        let next = match cur {
            None | Some(Value::None) => true,
            Some(Value::Bool(b)) => !*b,
            Some(other) => {
                return Err(format!(
                    "can't toggle a variable holding {} ({}); it must be true or false",
                    other.type_name(),
                    other
                ));
            }
        };
        self.store(v.local, &k, Value::Bool(next), ex, host);
        Ok(())
    }

    /// `transform {list::*} with (input * 2)`.
    #[inline(never)]
    pub(super) fn exec_transform(
        &mut self,
        t: &Transform,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        let items: Vec<Value> = self
            .list_items(&t.list, ex, host)?
            .into_iter()
            .map(|(_, x)| x)
            .collect();
        let saved = ex.locals.get(INPUT_KEY).cloned();
        let mut out = Vec::with_capacity(items.len());
        let mut failed = None;
        for it in items {
            ex.locals.set(INPUT_KEY, it);
            match self.eval(&t.with, ex, host) {
                Ok(v) => out.push(v),
                Err(e) => {
                    failed = Some(e);
                    break;
                }
            }
        }
        ex.locals.set(INPUT_KEY, saved.unwrap_or(Value::None));
        if let Some(e) = failed {
            return Err(e);
        }
        self.write_list(&t.list, out, ex, host)
    }
}

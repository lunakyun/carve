//! Running `Expr::Ext` (see `w2_expressions_3.rs`). Kept out of the hot
//! `eval` match.

use super::{Engine, Exec, online};
use crate::expr::Expr;
use crate::host::Host;
use crate::value::Value;
use crate::w2_expressions_3::{ExtFn, IndexSel, apply, key_value, takes_player};

impl Engine {
    #[inline(never)]
    pub(super) fn eval_ext(
        &mut self,
        f: ExtFn,
        args: &[Expr],
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        if let ExtFn::IndexOf(sel) = f {
            return self.eval_index_of(sel, args, ex, host);
        }
        let mut vals = Vec::with_capacity(args.len());
        for a in args {
            vals.push(self.eval(a, ex, host)?);
        }
        if takes_player(f) {
            match vals.first().and_then(|v| online(v, &mut ex.seen, host)) {
                Some(p) => vals[0] = Value::Player(p),
                None => return Ok(Value::None),
            }
        }
        Ok(apply(f, &vals, host))
    }

    fn eval_index_of(
        &mut self,
        sel: IndexSel,
        args: &[Expr],
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        let (Some(want), Some(Expr::Var(list))) = (args.first(), args.get(1)) else {
            return Ok(Value::None);
        };
        let want = self.eval(want, ex, host)?;
        let items = self.list_items(list, ex, host)?;
        let mut hits = items
            .iter()
            .filter(|(_, v)| v.loose_eq(&want))
            .map(|(k, _)| key_value(k));
        Ok(match sel {
            IndexSel::First => hits.next().unwrap_or(Value::None),
            IndexSel::Last => hits.next_back().unwrap_or(Value::None),
            IndexSel::All => Value::List(hits.collect()),
        })
    }
}

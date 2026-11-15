//! Running the `conditions_flags_metadata` conditions. Kept out of line: none of them is on
//! a hot path.

use super::{Engine, Exec};
use crate::conditions_flags_metadata::{self as c2, C2Cond, Kind};
use crate::expr_numbers_metadata::{self as w5, DataKind};
use crate::host::Host;
use crate::value::Value;

impl Engine {
    #[cold]
    #[inline(never)]
    pub(super) fn w5c2_cond(
        &mut self,
        c: &C2Cond,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<bool, String> {
        let a = match &c.a {
            Some(e) => self.eval(e, ex, host)?,
            None => Value::None,
        };
        let b = match &c.b {
            Some(e) => Some(self.eval(e, ex, host)?),
            None => None,
        };
        let hit = match c.kind {
            Kind::DamageCause => match (&ex.ctx.damage_cause, &b) {
                (Some(actual), Some(want)) => {
                    let want = want.to_string();
                    Some(c2::cause_matches(actual, &want))
                }
                _ => None,
            },
            Kind::EggHatch => match a {
                Value::Bool(h) => Some(h),
                _ => None,
            },
            Kind::Metadata | Kind::Persistent => {
                let (Some(key), Some(id)) = (b.as_ref(), w5::target_id(&a)) else {
                    return Ok(false);
                };
                let key = key.to_string();
                let kind = if c.kind == Kind::Metadata {
                    DataKind::Metadata
                } else {
                    DataKind::Persistent
                };
                let k = w5::data_key(kind, &id, &key);
                Some(match kind {
                    DataKind::Metadata => self.w5.metadata.contains_key(&k),
                    DataKind::Persistent => self.vars.get(&k) != Value::None,
                })
            }
            kind => c2::check(kind, &a, b.as_ref(), host),
        };
        Ok(hit.is_some_and(|h| h != c.negate))
    }
}

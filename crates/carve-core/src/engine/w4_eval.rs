//! Running the wave-4 functions (syntax in `w4_functions.rs`).

use super::{Engine, Exec};
use crate::host::Host;
use crate::value::Value;
use crate::w4_functions::{Node, apply};

impl Engine {
    #[inline(never)]
    pub(super) fn eval_w4f(&mut self, n: &Node, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        let mut vals = Vec::with_capacity(n.args.len());
        for a in &n.args {
            vals.push(self.eval(a, ex, host)?);
        }
        apply(n.f, &vals, host)
    }
}

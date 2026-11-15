//! Running the wave-4 functions (syntax in `functions_builtin.rs`).

use super::{Engine, Exec};
use crate::functions_builtin::{Node, apply};
use crate::host::Host;
use crate::value::Value;

impl Engine {
    #[inline(never)]
    pub(super) fn eval_builtin(
        &mut self,
        n: &Node,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        let mut vals = Vec::with_capacity(n.args.len());
        for a in &n.args {
            vals.push(self.eval(a, ex, host)?);
        }
        apply(n.f, &vals, host)
    }
}

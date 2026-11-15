//! Game-side answers for `w6_chunk5`.

use carve_core::Value;
use carve_core::w6_chunk5::Q;
use pumpkin_plugin_api::ItemStack;

use super::WasmHost;

impl WasmHost<'_> {
    pub(crate) fn w6c5_impl(&mut self, q: Q<'_>) -> Option<Value> {
        match q {
            // The server's own stack limit (reads the max stack size component).
            Q::MaxStack(id) => {
                let s = ItemStack::new(&format!("minecraft:{id}"), 1);
                Some(Value::Num(f64::from(s.get_max_count())))
            }
        }
    }
}

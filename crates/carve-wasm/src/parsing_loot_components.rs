//! Game-side answers for `parsing_loot_components`.

use carve_core::Value;
use carve_core::parsing_loot_components::Q;
use pumpkin_plugin_api::ItemStack;

use super::WasmHost;

impl WasmHost<'_> {
    pub(crate) fn game_fact_impl(&mut self, q: Q<'_>) -> Option<Value> {
        match q {
            // The server's own stack limit (reads the max stack size component).
            Q::MaxStack(id) => {
                let s = ItemStack::new(&format!("minecraft:{id}"), 1);
                Some(Value::Num(f64::from(s.get_max_count())))
            }
        }
    }
}

//! Game-side answers for the `w5_chunk2` conditions: bees in a hive, boss bar
//! flags, whether a block needs the right tool, interaction entities and
//! `has played before`.

use carve_core::Value;
use carve_core::w5_chunk2::Query;
use pumpkin_plugin_api::block_entity::BlockEntityType;
use pumpkin_plugin_api::common::BlockPos as WitPos;
use pumpkin_plugin_api::display::InteractionEntity;
use pumpkin_plugin_api::world::Block;

use super::WasmHost;

impl WasmHost<'_> {
    pub(super) fn w5c2_query_impl(&mut self, q: Query<'_>) -> Option<Value> {
        match q {
            Query::PlayedBefore(uuid) => super::w4_events_movement::played_before(uuid).map(Value::Bool),
            Query::BeeCount { world, x, y, z } => {
                let w = match world {
                    Some(n) => self.server.get_world_by_name(n)?,
                    None => self.server.get_all_worlds().into_iter().next()?,
                };
                match w.get_block_entity(WitPos { x, y, z })? {
                    BlockEntityType::BeehiveBlockEntity(b) => Some(Value::Num(f64::from(b.get_bee_count()))),
                    _ => None,
                }
            }
            Query::BarFlag(id, flag) => super::sb::bar_flag(id, flag).map(Value::Bool),
            Query::ToolRequired(id) => {
                let block = Block::from_name(id)?;
                Block::get_default_state_for_id(block.id).map(|s| Value::Bool(s.tool_required))
            }
            Query::Responsive(uuid) => {
                let e = self.find_entity(uuid)?;
                Some(Value::Bool(
                    InteractionEntity::from_entity(&e).is_some_and(|i| i.get_response()),
                ))
            }
        }
    }
}

//! Game-side answers for the `w4_conditions_1` checks: mob data, chat
//! settings, player weather, bell blocks.

use carve_core::w4_conditions_1::Check;
use carve_core::Value;
use pumpkin_plugin_api::block_entity::BlockEntityType;
use pumpkin_plugin_api::common::BlockPos as WitPos;
use pumpkin_plugin_api::player::ChatMode;
use pumpkin_plugin_api::world::{Entity, MobData};

use super::WasmHost;

impl WasmHost<'_> {
    fn w4_entity(&mut self, v: &Value) -> Option<Entity> {
        match v {
            Value::Player(p) => self.get(p).map(|pl| pl.as_entity()),
            Value::Entity(e) => self.find_entity(&e.uuid),
            _ => None,
        }
    }

    pub(super) fn w4c1_check_impl(&mut self, c: Check, v: &Value) -> Option<bool> {
        match c {
            Check::ChatColors | Check::ChatMessages | Check::ChatFiltering => {
                let Value::Player(p) = v else { return None };
                let s = self.get(p)?.as_java()?.get_settings();
                Some(match c {
                    Check::ChatColors => s.chat_colors,
                    Check::ChatMessages => matches!(s.chat_mode, ChatMode::Enabled),
                    _ => s.text_filtering,
                })
            }
            Check::ClientWeather => {
                let Value::Player(p) = v else { return None };
                Some(self.get(p)?.get_player_weather().is_some())
            }
            Check::BellRinging => {
                let Value::Location { x, y, z, world } = v else { return None };
                let w = match world {
                    Some(n) => self.server.get_world_by_name(n)?,
                    None => self.server.get_all_worlds().into_iter().next()?,
                };
                let pos = WitPos { x: x.floor() as i32, y: y.floor() as i32, z: z.floor() as i32 };
                match w.get_block_entity(pos)? {
                    BlockEntityType::BellBlockEntity(b) => Some(b.is_ringing()),
                    _ => Some(false),
                }
            }
            Check::InLiquid | Check::Wet => {
                let e = self.w4_entity(v)?;
                let water = e.is_in_water();
                Some(if c == Check::Wet { water } else { water || e.is_in_lava() })
            }
            Check::HasAi => {
                let mob = self.w4_entity(v)?.as_mob()?;
                Some(!mob.is_ai_disabled())
            }
            Check::CanBreed | Check::Sheared | Check::Stared => {
                let mob = self.w4_entity(v)?.as_mob()?;
                match (c, mob.get_mob_data()) {
                    (Check::CanBreed, MobData::Ageable(d)) => Some(!d.is_baby && d.age == 0),
                    (Check::Sheared, MobData::Sheep(d)) => Some(d.is_sheared),
                    (Check::Stared, MobData::Enderman(d)) => Some(d.is_staring),
                    _ => Some(false),
                }
            }
        }
    }
}

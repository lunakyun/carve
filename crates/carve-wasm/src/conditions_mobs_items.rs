//! Game-side answers for `conditions_mobs_items`: mob data, equipment, poses, display
//! flags, potion effect flags, chest and barrel lids, beacons, and the
//! operator and ban lists.

use carve_core::conditions_mobs_items::{Check, EFFECT_IDS, EffFlag, Q};
use carve_core::{PlayerRef, Value};
use pumpkin_plugin_api::block_entity::BlockEntityType;
use pumpkin_plugin_api::common::{BlockPos as WitPos, EntityPose};
use pumpkin_plugin_api::uuid;
use pumpkin_plugin_api::world::{Entity, EquipmentSlot, MobData};
use pumpkin_plugin_api::{Block, BlockTypeExt, EntityDisplayExt};

use super::{WasmHost, fx};

fn stack_id(s: &pumpkin_plugin_api::ItemStack) -> Option<String> {
    let key = s.get_registry_key();
    let id = key.strip_prefix("minecraft:").unwrap_or(&key);
    (s.get_count() > 0 && id != "air").then(|| id.to_string())
}

impl WasmHost<'_> {
    fn c3_entity(&mut self, v: &Value) -> Option<Entity> {
        match v {
            Value::Player(p) => self.get(p).map(|pl| pl.as_entity()),
            Value::Entity(e) => self.find_entity(&e.uuid),
            _ => None,
        }
    }

    fn c3_check(&mut self, c: Check, v: &Value) -> Option<bool> {
        let e = self.c3_entity(v)?;
        match c {
            Check::Riptiding => Some(matches!(e.get_pose(), EntityPose::SpinAttack)),
            Check::Saddled => {
                let l = e.as_living()?;
                Some(
                    l.get_equipment(EquipmentSlot::Saddle)
                        .as_ref()
                        .and_then(stack_id)
                        .is_some(),
                )
            }
            Check::Screaming | Check::Tamed => {
                let Some(mob) = e.as_mob() else {
                    return Some(false);
                };
                match (c, mob.get_mob_data()) {
                    (Check::Screaming, MobData::Enderman(d)) => Some(d.is_screaming),
                    (Check::Tamed, MobData::Wolf(d)) => Some(d.is_tamed),
                    (Check::Tamed, MobData::Cat(d)) => Some(d.is_tamed),
                    // Horses, parrots, ...: the API has no data for them.
                    (Check::Tamed, MobData::Generic) => None,
                    _ => Some(false),
                }
            }
            Check::TextShadow | Check::SeeThrough => {
                let t = e.as_text_display()?;
                Some(if c == Check::TextShadow {
                    t.get_shadow()
                } else {
                    t.get_see_through()
                })
            }
        }
    }

    fn c3_worn(&mut self, v: &Value) -> Option<Value> {
        let l = self.c3_entity(v)?.as_living()?;
        let mut out = Vec::new();
        for slot in [
            EquipmentSlot::Head,
            EquipmentSlot::Chest,
            EquipmentSlot::Legs,
            EquipmentSlot::Feet,
            EquipmentSlot::Body,
            EquipmentSlot::Saddle,
        ] {
            if let Some(id) = l.get_equipment(slot).as_ref().and_then(stack_id) {
                out.push(Value::Text(id));
            }
        }
        Some(Value::List(out))
    }

    fn c3_effect(&mut self, p: &PlayerRef, effect: &str, f: EffFlag) -> Option<Value> {
        let kind = fx::effect(effect)?;
        let e = self.get(p)?.get_effect(kind)?;
        Some(Value::Bool(match f {
            EffFlag::Icon => e.show_icon,
            EffFlag::Particles => e.show_particles,
            EffFlag::Ambient => e.ambient,
        }))
    }

    fn c3_lid(&mut self, world: &str, x: i32, y: i32, z: i32) -> Option<Value> {
        let w = self.server.get_world_by_name(world)?;
        let pos = WitPos { x, y, z };
        let open = match w.get_block_entity(pos)? {
            BlockEntityType::ChestBlockEntity(c) => c.viewer_count() > 0,
            BlockEntityType::TrappedChestBlockEntity(c) => c.viewer_count() > 0,
            BlockEntityType::BarrelBlockEntity(_) => {
                // The API has no block properties. Barrel states go facing x
                // open with `open=true` first, and the default state is
                // `facing=north, open=false`: an odd distance from it is open.
                let default = i32::from(Block::of("minecraft:barrel")?.default_state_id);
                (i32::from(w.get_block_state_id(pos)) - default).rem_euclid(2) == 1
            }
            // Pumpkin reports no viewers for ender chests and shulker boxes.
            _ => return None,
        };
        Some(Value::Bool(open))
    }

    fn c3_beacon(&mut self, world: &str, x: i32, y: i32, z: i32, primary: bool) -> Option<Value> {
        let w = self.server.get_world_by_name(world)?;
        let BlockEntityType::BeaconBlockEntity(b) = w.get_block_entity(WitPos { x, y, z })? else {
            return None;
        };
        let id = if primary {
            b.get_primary_effect()
        } else {
            b.get_secondary_effect()
        };
        let name = EFFECT_IDS.get(usize::try_from(id).ok()?)?;
        Some(Value::Text((*name).to_string()))
    }

    pub(super) fn w5c3_impl(&mut self, q: Q<'_>) -> Option<Value> {
        let player = |id: pumpkin_plugin_api::uuid::Uuid, name: String| {
            Value::Player(PlayerRef {
                uuid: uuid::to_string(id),
                name,
            })
        };
        match q {
            Q::Check(c, v) => self.c3_check(c, v).map(Value::Bool),
            Q::Worn(v) => self.c3_worn(v),
            Q::Effect(p, e, f) => self.c3_effect(p, e, f),
            Q::Lid(w, x, y, z) => self.c3_lid(w, x, y, z),
            Q::Beacon(w, x, y, z, primary) => self.c3_beacon(w, x, y, z, primary),
            Q::Ops => Some(Value::List(
                self.server
                    .get_op_manager()
                    .list_ops()
                    .into_iter()
                    .map(|o| player(o.uuid, o.name))
                    .collect(),
            )),
            Q::BannedPlayers => Some(Value::List(
                self.server
                    .get_ban_manager()
                    .list_player_bans()
                    .into_iter()
                    .map(|b| player(b.uuid, b.name))
                    .collect(),
            )),
            Q::BannedIps => Some(Value::List(
                self.server
                    .get_ban_manager()
                    .list_ip_bans()
                    .into_iter()
                    .map(|b| Value::Text(b.ip))
                    .collect(),
            )),
        }
    }
}

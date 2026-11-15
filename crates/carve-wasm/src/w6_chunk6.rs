//! Host side of `carve_core::w6_chunk6`: portal cooldown, protocol version,
//! vehicle and passengers, villager data, bell ringing time, the whitelist
//! switch and list, and entity rotation.

use carve_core::w6_chunk6::{Get, Set, Vf, protocol_of};
use carve_core::{EntityRef, PlayerRef, Value};
use pumpkin_plugin_api::block_entity::BlockEntityType;
use pumpkin_plugin_api::common::BlockPos as WitPos;
use pumpkin_plugin_api::uuid;
use pumpkin_plugin_api::world::{Entity, MobData};

use super::{WasmHost, player_ref};

/// The Debug name of a villager profession in lower case (`librarian`).
fn profession_name(d: &impl std::fmt::Debug) -> String {
    let s = format!("{d:?}").to_ascii_lowercase();
    s.rsplit("::").next().unwrap_or(&s).to_string()
}

impl WasmHost<'_> {
    fn w6_value(&mut self, e: &Entity) -> Value {
        let id = uuid::to_string(e.get_uuid());
        if let Some(p) = self.get_by_uuid(&id) {
            return Value::Player(player_ref(p));
        }
        Value::Entity(EntityRef {
            uuid: id,
            kind: crate::ents::entity_name(e.get_type()).to_string(),
        })
    }

    #[cold]
    pub(super) fn w6c6_get_impl(&mut self, q: Get<'_>) -> Option<Value> {
        match q {
            Get::Portal(r) => {
                let e = self.find_entity(&r.uuid)?;
                Some(Value::Num(f64::from(e.get_portal_cooldown())))
            }
            Get::Protocol(p) => {
                let v = self.get(p)?.as_java()?.get_version();
                let name = format!("{v:?}");
                protocol_of(&name)
                    .map(|n| Value::Num(f64::from(n)))
                    .or(Some(Value::Text(name)))
            }
            Get::Vehicle(r) => {
                let e = self.find_entity(&r.uuid)?;
                let v = e.get_vehicle()?;
                Some(self.w6_value(&v))
            }
            Get::Passengers(r) => {
                let e = self.find_entity(&r.uuid)?;
                let list = e.get_passengers();
                Some(Value::List(list.iter().map(|p| self.w6_value(p)).collect()))
            }
            Get::Villager(r, f) => {
                let e = self.find_entity(&r.uuid)?;
                let MobData::Villager(d) = e.as_mob()?.get_mob_data() else {
                    return None;
                };
                Some(match f {
                    Vf::Level => Value::Num(f64::from(d.level)),
                    Vf::Experience => Value::Num(f64::from(d.experience)),
                    Vf::Profession => Value::Text(profession_name(&d.profession)),
                })
            }
            Get::Ringing(world, x, y, z) => {
                let w = self.server.get_world_by_name(world)?;
                match w.get_block_entity(WitPos { x, y, z })? {
                    BlockEntityType::BellBlockEntity(b) => Some(Value::Num(f64::from(b.get_ring_ticks()))),
                    _ => None,
                }
            }
            Get::Whitelist => Some(Value::Bool(self.server.get_whitelist_manager().is_enabled())),
            Get::WhitelistPlayers => {
                let entries = self.server.get_whitelist_manager().list_entries();
                Some(Value::List(
                    entries
                        .into_iter()
                        .map(|e| {
                            Value::Player(PlayerRef {
                                uuid: uuid::to_string(e.uuid),
                                name: e.name,
                            })
                        })
                        .collect(),
                ))
            }
        }
    }

    #[cold]
    pub(super) fn w6c6_set_impl(&mut self, s: Set<'_>) -> bool {
        match s {
            Set::Portal(r, ticks) => match self.find_entity(&r.uuid) {
                Some(e) => {
                    e.set_portal_cooldown(ticks);
                    true
                }
                None => false,
            },
            Set::Whitelist(on) => {
                self.server.get_whitelist_manager().set_enabled(on);
                true
            }
            Set::Rotation(r, yaw, pitch) => match self.find_entity(&r.uuid) {
                Some(e) => {
                    let y = yaw.unwrap_or_else(|| e.get_yaw());
                    let p = pitch.unwrap_or_else(|| e.get_pitch());
                    e.set_rotation(y, p);
                    true
                }
                None => false,
            },
        }
    }
}

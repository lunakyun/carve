//! Game side of `carve_core::effects_entity_commands`: flight, fake blocks, interaction and
//! text display flags, sound categories, shearing, taming and timed bans.

use carve_core::Value;
use carve_core::effects_entity_commands::{DisplayFlag, HostOp};
use pumpkin_plugin_api::common::BlockPos as WitPos;
use pumpkin_plugin_api::display::InteractionEntity;
use pumpkin_plugin_api::wit::pumpkin::plugin::sounds::SoundCategory;
use pumpkin_plugin_api::world::{Entity, MobData};
use pumpkin_plugin_api::{EntityDisplayExt, uuid};

use super::WasmHost;

fn category(name: &str) -> Option<SoundCategory> {
    Some(match name {
        "master" => SoundCategory::Master,
        "music" => SoundCategory::Music,
        "record" => SoundCategory::Records,
        "weather" => SoundCategory::Weather,
        "block" => SoundCategory::Blocks,
        "hostile" => SoundCategory::Hostile,
        "neutral" => SoundCategory::Neutral,
        "player" => SoundCategory::Players,
        "ambient" => SoundCategory::Ambient,
        "voice" => SoundCategory::Voice,
        _ => return None,
    })
}

impl WasmHost<'_> {
    fn w6_entity(&mut self, v: &Value) -> Option<Entity> {
        match v {
            Value::Player(p) => self.get(p).map(|pl| pl.as_entity()),
            Value::Entity(e) => self.find_entity(&e.uuid),
            _ => None,
        }
    }

    pub(super) fn w6c2_impl(&mut self, op: HostOp<'_>) -> Option<bool> {
        match op {
            HostOp::AllowFlight(p, on) => {
                let pl = self.get(p)?;
                pl.set_allow_flight(on);
                if !on {
                    pl.set_flying(false);
                }
                Some(true)
            }
            HostOp::FakeBlock(p, world, x, y, z, block) => {
                // Only the world the player is in shows it.
                let _ = world;
                let pl = self.get(p)?;
                let pos = WitPos { x, y, z };
                match block {
                    Some(b) => {
                        let id = super::block_shapes_ai::resolve_block(b)?;
                        pl.send_block_change(pos, id);
                    }
                    None => pl.reset_block_change(pos),
                }
                Some(true)
            }
            HostOp::Responsive(v, on) => {
                let e = self.w6_entity(v)?;
                let i = InteractionEntity::from_entity(&e)?;
                i.set_response(on);
                Some(true)
            }
            HostOp::Display(v, flag, on) => {
                let e = self.w6_entity(v)?;
                let t = e.as_text_display()?;
                match flag {
                    DisplayFlag::Shadow => t.set_shadow(on),
                    DisplayFlag::SeeThrough => t.set_see_through(on),
                }
                Some(true)
            }
            HostOp::StopSound(p, sound, cat) => {
                let pl = self.get(p)?;
                let name = sound.map(|s| {
                    if s.contains(':') {
                        s.to_string()
                    } else {
                        format!("minecraft:{s}")
                    }
                });
                pl.stop_custom_sound(name.as_deref(), category(cat));
                Some(true)
            }
            HostOp::Ride(rider, vehicle) => self.ride_op(rider, vehicle),
            HostOp::Tame(v, on, owner) => {
                let e = self.w6_entity(v)?;
                let mob = e.as_mob()?;
                let owner = if on {
                    owner.and_then(|p| uuid::parse(&p.uuid))
                } else {
                    None
                };
                match mob.get_mob_data() {
                    MobData::Wolf(mut d) => {
                        d.is_tamed = on;
                        d.owner = owner;
                        if !on {
                            d.is_sitting = false;
                        }
                        Some(mob.set_mob_data(MobData::Wolf(d)))
                    }
                    MobData::Cat(mut d) => {
                        d.is_tamed = on;
                        d.owner = owner;
                        if !on {
                            d.is_sitting = false;
                        }
                        Some(mob.set_mob_data(MobData::Cat(d)))
                    }
                    _ => Some(false),
                }
            }
        }
    }
}

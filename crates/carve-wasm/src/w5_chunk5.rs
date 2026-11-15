//! Game-side answers for `w5_chunk5`: potion effects of players, boss bar
//! progress, client weather, block states and menu viewers.

use carve_core::w5_chunk5::{INFINITE, Obj, PotionVal, Q};
use carve_core::{PlayerRef, Value};
use pumpkin_plugin_api::common::BlockPos as WitPos;
use pumpkin_plugin_api::player::{ClientGameEvent, PlayerWeather};
use pumpkin_plugin_api::wit::pumpkin::plugin::status_effect::StatusEffectInstance;
use pumpkin_plugin_api::world::BlockFlags;

use super::{WasmHost, enum_name, fx, inv, sb};

fn wit_pos(at: &carve_core::block_ops::BlockPos) -> WitPos {
    WitPos {
        x: at.x,
        y: at.y,
        z: at.z,
    }
}

impl WasmHost<'_> {
    pub(super) fn w5c5_impl(&mut self, q: Q<'_>) -> Value {
        self.w5c5_inner(q).unwrap_or(Value::None)
    }

    fn w5c5_inner(&mut self, q: Q<'_>) -> Option<Value> {
        Some(match q {
            Q::Effects(p) => {
                let list = self.get(p)?.get_active_effects();
                Value::List(
                    list.into_iter()
                        .map(|e| {
                            Value::W5(Box::new(Obj::Potion(PotionVal {
                                effect: enum_name(&format!("{:?}", e.effect_type)),
                                amplifier: e.amplifier,
                                ticks: if e.duration as i32 == -1 {
                                    INFINITE
                                } else {
                                    e.duration
                                },
                                ambient: e.ambient,
                                particles: e.show_particles,
                            })))
                        })
                        .collect(),
                )
            }
            Q::AddEffect(p, e) => {
                let effect_type = fx::effect(&e.effect)?;
                self.get(p)?.add_effect(StatusEffectInstance {
                    effect_type,
                    // Pumpkin reads the duration as an i32; u32::MAX is -1, infinite.
                    duration: e.ticks,
                    amplifier: e.amplifier,
                    ambient: e.ambient,
                    show_particles: e.particles,
                    show_icon: true,
                });
                Value::Bool(true)
            }
            Q::BarProgress(id) => Value::Num(f64::from(sb::bar_health(id)?) * 100.0),
            Q::BarExists(id) => Value::Bool(sb::bar_health(id).is_some()),
            Q::Weather(p) => match self.get(p)?.get_player_weather()? {
                PlayerWeather::Clear => Value::Text("clear".into()),
                PlayerWeather::Downfall => Value::Text("rain".into()),
            },
            Q::SetWeather(p, rain) => {
                // Pumpkin only stores the client weather; the packets are ours to send.
                let world = super::world_name(&self.get(p)?.get_world());
                let world_rain = self.named_world(Some(&world)).is_some_and(|w| w.is_raining());
                let pl = self.get(p)?;
                match rain {
                    Some(true) => pl.set_player_weather(PlayerWeather::Downfall),
                    Some(false) => pl.set_player_weather(PlayerWeather::Clear),
                    None => pl.reset_player_weather(),
                }
                if let Some(java) = pl.as_java() {
                    let wet = rain.unwrap_or(world_rain);
                    let ev = if wet {
                        ClientGameEvent::BeginRaining
                    } else {
                        ClientGameEvent::EndRaining
                    };
                    java.send_game_event(ev, 0.0);
                    java.send_game_event(ClientGameEvent::RainLevelChange, if wet { 1.0 } else { 0.0 });
                }
                Value::Bool(true)
            }
            Q::BlockState(at) => {
                let w = self.block_world(at)?;
                let pos = wit_pos(at);
                let b = w.get_block(pos);
                let state = w.get_block_state_id(pos);
                Value::List(vec![
                    Value::Text(b.name.clone()),
                    Value::Num(f64::from(state)),
                    Value::Num(f64::from(b.default_state_id)),
                ])
            }
            Q::SetBlockState(at, state) => {
                let w = self.block_world(at)?;
                let flags = BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::NOTIFY_LISTENERS;
                w.set_block_state(wit_pos(at), state, flags);
                Value::Bool(true)
            }
            Q::Viewers(id, viewer) => {
                let uuids = inv::viewers(id, viewer);
                let mut out = Vec::new();
                for uuid in uuids {
                    if let Some(name) = self.resolve_player(&uuid) {
                        out.push(Value::Player(PlayerRef { uuid, name }));
                    }
                }
                Value::List(out)
            }
        })
    }
}

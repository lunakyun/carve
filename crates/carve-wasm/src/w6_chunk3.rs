//! Game-side answers for `w6_chunk3`: mob age, creeper state, climbing,
//! navigation, handedness, block properties (conditional, powered, persistent,
//! occluding), loot tables, held items, vehicles, plugins, feature flags, the
//! Minecraft version and the player's pressed keys.

use std::sync::Mutex;

use carve_core::w6_chunk3::{Bk, Ck, Q};
use carve_core::{PlayerRef, Value};
use pumpkin_plugin_api::CustomStatistic;
use pumpkin_plugin_api::common::{BlockPos as WitPos, Hand};
use pumpkin_plugin_api::events::{EventData, EventHandler, PlayerInputEvent};
use pumpkin_plugin_api::world::{Entity, EquipmentSlot, MobData};
use pumpkin_plugin_api::{Server, uuid};

use super::WasmHost;

/// The input bits each player holds right now (uuid, bits).
static KEYS: Mutex<Vec<(String, u32)>> = Mutex::new(Vec::new());
/// Entity ids that a spawner block spawned while Carve was running.
static SPAWNED: Mutex<Vec<i32>> = Mutex::new(Vec::new());
/// Per player: last jump count and the tick it last changed (uuid, count, tick).
static JUMPS: Mutex<Vec<(String, i32, i32)>> = Mutex::new(Vec::new());

/// Remember which entities a spawner spawned (capped, oldest dropped first).
pub fn note_spawned(id: i32) {
    if let Ok(mut s) = SPAWNED.lock() {
        if s.len() >= 4096 {
            s.drain(..1024);
        }
        s.push(id);
    }
}

pub struct OnInput;

impl EventHandler<PlayerInputEvent> for OnInput {
    fn handle(&self, _server: Server, data: EventData<PlayerInputEvent>) -> EventData<PlayerInputEvent> {
        let bits = u32::from_str_radix(&data.input, 2).unwrap_or(0);
        let id = uuid::to_string(data.player.get_id());
        if let Ok(mut k) = KEYS.lock() {
            match k.iter_mut().find(|(u, _)| *u == id) {
                Some(e) => e.1 = bits,
                None => k.push((id, bits)),
            }
        }
        data
    }
}

pub struct OnSpawner;

impl EventHandler<pumpkin_plugin_api::events::SpawnerSpawnEvent> for OnSpawner {
    fn handle(
        &self,
        _server: Server,
        data: EventData<pumpkin_plugin_api::events::SpawnerSpawnEvent>,
    ) -> EventData<pumpkin_plugin_api::events::SpawnerSpawnEvent> {
        note_spawned(data.entity_id);
        data
    }
}

pub struct OnTrialSpawner;

impl EventHandler<pumpkin_plugin_api::events::TrialSpawnerSpawnEvent> for OnTrialSpawner {
    fn handle(
        &self,
        _server: Server,
        data: EventData<pumpkin_plugin_api::events::TrialSpawnerSpawnEvent>,
    ) -> EventData<pumpkin_plugin_api::events::TrialSpawnerSpawnEvent> {
        note_spawned(data.entity_id);
        data
    }
}

/// Blocks an entity can climb.
const CLIMBABLE: &[&str] = &[
    "ladder",
    "vine",
    "scaffolding",
    "twisting_vines",
    "twisting_vines_plant",
    "weeping_vines",
    "weeping_vines_plant",
    "cave_vines",
    "cave_vines_plant",
];

fn short(id: &str) -> &str {
    id.strip_prefix("minecraft:").unwrap_or(id)
}

impl WasmHost<'_> {
    fn c6_entity(&mut self, v: &Value) -> Option<Entity> {
        match v {
            Value::Player(p) => self.get(p).map(|pl| pl.as_entity()),
            Value::Entity(e) => self.find_entity(&e.uuid),
            _ => None,
        }
    }

    fn c6_block_id(&mut self, world: &str, x: i32, y: i32, z: i32) -> Option<String> {
        let w = self.server.get_world_by_name(world)?;
        let id = w.get_block_state_id(WitPos { x, y, z });
        let st = pumpkin_plugin_api::BlockState::from_id(id)?;
        Some(short(&st.block_name).to_string())
    }

    fn c6_check(&mut self, c: Ck, v: &Value) -> Option<bool> {
        let e = self.c6_entity(v)?;
        match c {
            Ck::Baby => {
                let mob = e.as_mob()?;
                match mob.get_mob_data() {
                    MobData::Ageable(d) => Some(d.is_baby),
                    MobData::Zombie(d) => Some(d.is_baby),
                    _ => Some(false),
                }
            }
            Ck::Charged => {
                if let Some(mob) = e.as_mob() {
                    return match mob.get_mob_data() {
                        MobData::Creeper(d) => Some(d.is_powered),
                        _ => Some(false),
                    };
                }
                // A player: a crossbow in the main hand holds projectiles.
                let l = e.as_living()?;
                let stack = l.get_equipment(EquipmentSlot::MainHand)?;
                if short(&stack.get_registry_key()) != "crossbow" {
                    return Some(false);
                }
                Some(stack.get_components().into_iter().any(|c| {
                    c.component == pumpkin_plugin_api::item_stack::DataComponent::ChargedProjectiles
                        && c.value.first().is_some_and(|n| *n > 0)
                }))
            }
            Ck::Climbing => {
                if e.is_on_ground() {
                    return Some(false);
                }
                let p = e.get_position();
                let w = e.get_world();
                let id = w.get_block_state_id(WitPos {
                    x: p.0.floor() as i32,
                    y: p.1.floor() as i32,
                    z: p.2.floor() as i32,
                });
                let st = pumpkin_plugin_api::BlockState::from_id(id)?;
                Some(CLIMBABLE.contains(&short(&st.block_name)))
            }
            Ck::Spawner => {
                let id = i32::try_from(e.get_id()).ok()?;
                Some(SPAWNED.lock().is_ok_and(|s| s.contains(&id)))
            }
            Ck::Pathfinding => Some(e.as_mob().is_some_and(|m| m.is_navigating())),
            Ck::LeftHanded => match v {
                Value::Player(p) => {
                    let pl = self.get(p)?;
                    let settings = pl.as_java()?.get_settings();
                    Some(matches!(settings.main_hand, Hand::Left))
                }
                _ => None,
            },
            Ck::Igniting => {
                let mob = e.as_mob()?;
                match mob.get_mob_data() {
                    MobData::Creeper(d) => Some(d.is_ignited),
                    _ => Some(false),
                }
            }
            Ck::Silent => Some(e.is_silent()),
            Ck::Riding => Some(e.get_vehicle().is_some()),
            Ck::Jumping => {
                let Value::Player(p) = v else {
                    return None;
                };
                let pl = self.get(p)?;
                let count = pl.get_custom_statistic(CustomStatistic::Jump);
                let now = e.get_ticks_lived();
                let mut j = JUMPS.lock().ok()?;
                match j.iter_mut().find(|(u, _, _)| *u == p.uuid) {
                    Some(entry) => {
                        if entry.1 != count {
                            entry.1 = count;
                            entry.2 = now;
                        }
                        Some(now - entry.2 <= 10)
                    }
                    None => {
                        j.push((p.uuid.clone(), count, i32::MIN / 2));
                        Some(false)
                    }
                }
            }
            // No loot table or persistence flag of an entity is readable.
            Ck::Lootable | Ck::Persistent => None,
        }
    }

    fn c6_block(&mut self, k: Bk, world: &str, x: i32, y: i32, z: i32) -> Option<bool> {
        let w = self.server.get_world_by_name(world)?;
        let pos = WitPos { x, y, z };
        let st = w.get_block_state(pos);
        let props = || st_props(st.id);
        match k {
            Bk::Occluding => Some(st.is_full_cube && st.opacity == 15),
            Bk::Solid => Some(st.is_solid),
            Bk::Transparent => Some(st.opacity == 0),
            Bk::Passable => Some(st.collision_shapes.is_empty()),
            Bk::Conditional => prop(&props(), "conditional").map(|v| v == "true"),
            Bk::Persistent => prop(&props(), "persistent").map(|v| v == "true"),
            Bk::Powered => {
                let p = props();
                if let Some(v) = prop(&p, "powered") {
                    return Some(v == "true");
                }
                if let Some(v) = prop(&p, "power") {
                    return Some(v.parse::<u32>().is_ok_and(|n| n > 0));
                }
                prop(&p, "lit").map(|v| v == "true")
            }
            Bk::Lootable => {
                let nbt = w.get_block_entity_nbt(pos)?;
                // The loot table is stored as the string key `LootTable`.
                Some(nbt.windows(9).any(|win| win == b"LootTable"))
            }
        }
    }

    fn c6_hands(&mut self, v: &Value) -> Option<Value> {
        if let Value::Player(p) = v {
            let pl = self.get(p)?;
            let id = |hand| {
                pl.get_item_in_hand(hand)
                    .filter(|s| s.get_count() > 0)
                    .map(|s| short(&s.get_registry_key()).to_string())
                    .filter(|s| s != "air")
                    .unwrap_or_default()
            };
            return Some(Value::List(vec![
                Value::Text(id(Hand::Right)),
                Value::Text(id(Hand::Left)),
            ]));
        }
        let e = self.c6_entity(v)?;
        let l = e.as_living()?;
        let id = |slot| {
            l.get_equipment(slot)
                .filter(|s| s.get_count() > 0)
                .map(|s| short(&s.get_registry_key()).to_string())
                .filter(|s| s != "air")
                .unwrap_or_default()
        };
        Some(Value::List(vec![
            Value::Text(id(EquipmentSlot::MainHand)),
            Value::Text(id(EquipmentSlot::OffHand)),
        ]))
    }

    fn c6_feature(&mut self, name: &str) -> Option<Value> {
        let name = name.strip_prefix("minecraft_").unwrap_or(name);
        if name == "vanilla" {
            return Some(Value::Bool(true));
        }
        let packs = self.server.get_datapack_manager().list_enabled_packs();
        Some(Value::Bool(packs.iter().any(|p| {
            let id = p.id.strip_prefix("file/").unwrap_or(&p.id);
            let id = id.strip_prefix("minecraft:").unwrap_or(id);
            id == name
        })))
    }

    pub(super) fn w6c3_impl(&mut self, q: Q<'_>) -> Option<Value> {
        match q {
            Q::Check(c, v) => self.c6_check(c, v).map(Value::Bool),
            Q::Block(k, w, x, y, z) => self.c6_block(k, w, x, y, z).map(Value::Bool),
            Q::BlockId(w, x, y, z) => self.c6_block_id(w, x, y, z).map(Value::Text),
            Q::Hands(v) => self.c6_hands(v),
            Q::Plugin(name) => {
                if name.eq_ignore_ascii_case("carve") {
                    return Some(Value::Bool(true));
                }
                // The IPC call fails with an outer error when no plugin has that name.
                Some(Value::Bool(
                    pumpkin_plugin_api::ipc::send_ipc_message(name, &[]).is_ok(),
                ))
            }
            Q::Feature(name) => self.c6_feature(name),
            Q::Key(p, bit) => {
                let bits = KEYS
                    .lock()
                    .ok()?
                    .iter()
                    .find(|(u, _)| *u == p.uuid)
                    .map_or(0, |e| e.1);
                Some(Value::Bool(bits & u32::from(bit) != 0))
            }
            Q::Version => version(self.server).map(|v| Value::Text(v.into())),
            Q::SpawnKind(p) => {
                let pl = self.get(p)?;
                let at = pl.get_respawn_location()?;
                let w = pl.as_entity().get_world();
                let x = at.0.floor() as i32;
                let z = at.2.floor() as i32;
                let y0 = at.1.floor() as i32;
                let mut kind = "none";
                for y in [y0, y0 - 1] {
                    let id = w.get_block_state_id(WitPos { x, y, z });
                    let Some(st) = pumpkin_plugin_api::BlockState::from_id(id) else {
                        continue;
                    };
                    let name = short(&st.block_name);
                    if name == "respawn_anchor" {
                        kind = "anchor";
                        break;
                    }
                    if name.ends_with("_bed") {
                        kind = "bed";
                        break;
                    }
                }
                Some(Value::Text(kind.into()))
            }
            Q::Vehicle(v) => {
                let e = self.c6_entity(v)?;
                Some(match e.get_vehicle() {
                    Some(veh) => Value::Text(crate::ents::entity_name(veh.get_type()).to_string()),
                    None => Value::None,
                })
            }
            Q::InfiniteEffect(p, name) => self.c6_infinite(p, name),
            Q::Border(world, x, z) => {
                let w = self.server.get_world_by_name(world)?;
                Some(Value::Bool(w.get_world_border().contains(x, z)))
            }
        }
    }

    fn c6_infinite(&mut self, p: &PlayerRef, name: &str) -> Option<Value> {
        let kind = super::fx::effect(name)?;
        let e = self.get(p)?.get_effect(kind)?;
        // The host maps Pumpkin's infinite duration (-1) to 0.
        Some(Value::Bool(e.duration == 0))
    }
}

fn st_props(id: u16) -> Vec<(String, String)> {
    pumpkin_plugin_api::BlockState::from_id(id)
        .map(|s| s.get_properties())
        .unwrap_or_default()
}

fn prop<'a>(props: &'a [(String, String)], key: &str) -> Option<&'a str> {
    props.iter().find(|(k, _)| k == key).map(|(_, v)| v.as_str())
}

static VERSION: std::sync::OnceLock<Option<&'static str>> = std::sync::OnceLock::new();

/// The Minecraft version, read once: the server's system info is slow to build.
pub fn version(server: &Server) -> Option<&'static str> {
    *VERSION.get_or_init(|| minecraft_version(&server.get_sys_info().pumpkin_version))
}

/// The Minecraft version of a Pumpkin release. The v0.1 plugin API has no
/// getter for it, so the releases this build was made for are listed.
fn minecraft_version(pumpkin: &str) -> Option<&'static str> {
    let base = pumpkin.split(['+', '-']).next().unwrap_or(pumpkin);
    match base {
        "0.2.2" => Some("26.3"),
        _ => None,
    }
}

//! Game-side answers for `carve_core::w5_chunk4`: block data, world border
//! settings, written books, ender chests, entity AI and attributes, and the
//! enchanting offers remembered for `enchantment hint`.

use std::sync::Mutex;

use carve_core::block_ops::BlockPos;
use carve_core::w5_chunk4::{BorderField, Q};
use carve_core::{PlayerRef, Value};
use pumpkin_plugin_api::block_entity::{BlockEntityType, ContainerBlockEntity};
use pumpkin_plugin_api::common::BlockPos as WitPos;
use pumpkin_plugin_api::common::Hand;
use pumpkin_plugin_api::item_stack::ItemStack;
use pumpkin_plugin_api::wit::pumpkin::plugin::attributes::Attribute;
use pumpkin_plugin_api::wit::pumpkin::plugin::data_components::DataComponent;
use pumpkin_plugin_api::wit::pumpkin::plugin::status_effect::StatusEffectInstance;
use pumpkin_plugin_api::world::{self as wapi, BlockFlags, Entity, MobData};

use super::{WasmHost, fx};

/// Offers of the last `prepare enchant` per player: uuid, then cost, id and
/// level of the three offers.
static OFFERS: Mutex<Vec<(String, [i32; 9])>> = Mutex::new(Vec::new());

pub fn remember_offers(uuid: &str, nums: [i32; 9]) {
    if let Ok(mut o) = OFFERS.lock() {
        o.retain(|(u, _)| u != uuid);
        o.push((uuid.to_string(), nums));
    }
}

/// Enchantment name and level the table showed for an option (0-2).
pub fn hint(uuid: &str, option: i32) -> Option<(String, i32)> {
    let o = OFFERS.lock().ok()?;
    let (_, n) = o.iter().find(|(u, _)| u == uuid)?;
    let i = usize::try_from(option).ok().filter(|i| *i < 3)? * 3;
    let name = carve_core::item_meta::ENCHANTS.get(usize::try_from(n[i + 1]).ok()?)?;
    Some(((*name).to_string(), n[i + 2]))
}

fn varint(mut n: u32) -> Vec<u8> {
    let mut out = Vec::new();
    loop {
        let b = (n & 0x7f) as u8;
        n >>= 7;
        if n == 0 {
            out.push(b);
            return out;
        }
        out.push(b | 0x80);
    }
}

/// The `damage` component of a stack.
pub fn damage_of(stack: &ItemStack) -> Option<u32> {
    let v = stack
        .get_components()
        .into_iter()
        .find(|c| c.component == DataComponent::Damage)?
        .value;
    let mut n = 0u32;
    for (i, b) in v.iter().take(5).enumerate() {
        n |= u32::from(b & 0x7f) << (7 * i);
        if b & 0x80 == 0 {
            return (n > 0).then_some(n);
        }
    }
    None
}

pub fn set_damage(stack: &ItemStack, damage: Option<u32>) {
    match damage {
        Some(d) if d > 0 => stack.set_component(DataComponent::Damage, &varint(d)),
        _ => stack.remove_component(DataComponent::Damage),
    }
}

fn attribute(name: &str) -> Option<Attribute> {
    use Attribute as A;
    Some(match name {
        "air_drag_modifier" => A::AirDragModifier,
        "armor" => A::Armor,
        "armor_toughness" => A::ArmorToughness,
        "attack_damage" => A::AttackDamage,
        "attack_knockback" => A::AttackKnockback,
        "attack_speed" => A::AttackSpeed,
        "below_name_distance" => A::BelowNameDistance,
        "block_break_speed" => A::BlockBreakSpeed,
        "block_interaction_range" => A::BlockInteractionRange,
        "bounciness" => A::Bounciness,
        "burning_time" => A::BurningTime,
        "camera_distance" => A::CameraDistance,
        "explosion_knockback_resistance" => A::ExplosionKnockbackResistance,
        "entity_interaction_range" => A::EntityInteractionRange,
        "fall_damage_multiplier" => A::FallDamageMultiplier,
        "flying_speed" => A::FlyingSpeed,
        "follow_range" => A::FollowRange,
        "friction_modifier" => A::FrictionModifier,
        "gravity" => A::Gravity,
        "jump_strength" => A::JumpStrength,
        "knockback_resistance" => A::KnockbackResistance,
        "luck" => A::Luck,
        "max_absorption" => A::MaxAbsorption,
        "max_health" => A::MaxHealth,
        "mining_efficiency" => A::MiningEfficiency,
        "movement_efficiency" => A::MovementEfficiency,
        "movement_speed" => A::MovementSpeed,
        "name_tag_distance" => A::NameTagDistance,
        "oxygen_bonus" => A::OxygenBonus,
        "safe_fall_distance" => A::SafeFallDistance,
        "scale" => A::Scale,
        "sneaking_speed" => A::SneakingSpeed,
        "spawn_reinforcements" => A::SpawnReinforcements,
        "step_height" => A::StepHeight,
        "submerged_mining_speed" => A::SubmergedMiningSpeed,
        "sweeping_damage_ratio" => A::SweepingDamageRatio,
        "tempt_range" => A::TemptRange,
        "water_movement_efficiency" => A::WaterMovementEfficiency,
        "waypoint_transmit_range" => A::WaypointTransmitRange,
        "waypoint_receive_range" => A::WaypointReceiveRange,
        _ => return None,
    })
}

/// `oak_stairs[facing=east,half=bottom]` into a name and properties.
fn split_data(s: &str) -> (String, Vec<(String, String)>) {
    let s = s.trim();
    let (name, props) = match s.find('[') {
        Some(i) => (&s[..i], s[i + 1..].trim_end_matches(']')),
        None => (s, ""),
    };
    let name = name.trim().to_ascii_lowercase().replace(' ', "_");
    let name = if name.contains(':') {
        name
    } else {
        format!("minecraft:{name}")
    };
    let props = props
        .split(',')
        .filter_map(|kv| {
            let (k, v) = kv.split_once('=')?;
            Some((k.trim().to_ascii_lowercase(), v.trim().to_ascii_lowercase()))
        })
        .collect();
    (name, props)
}

fn data_text(state: u16) -> Option<String> {
    let info = wapi::block_state_to_info(state)?;
    let mut s = info.name;
    if !s.contains(':') {
        s = format!("minecraft:{s}");
    }
    if !info.properties.is_empty() {
        let p: Vec<String> = info.properties.iter().map(|(k, v)| format!("{k}={v}")).collect();
        s = format!("{s}[{}]", p.join(","));
    }
    Some(s)
}

fn resolve(data: &str) -> Option<u16> {
    let (name, props) = split_data(data);
    wapi::resolve_block_state(&name, &props)
}

fn wpos(at: &BlockPos) -> WitPos {
    WitPos {
        x: at.x,
        y: at.y,
        z: at.z,
    }
}

impl WasmHost<'_> {
    fn c4_entity(&mut self, v: &Value) -> Option<Entity> {
        match v {
            Value::Player(p) => self.get(p).map(|pl| pl.as_entity()),
            Value::Entity(e) => self.find_entity(&e.uuid),
            _ => None,
        }
    }

    fn c4_held(&mut self, p: &PlayerRef) -> Option<ItemStack> {
        self.get(p)?.get_item_in_hand(Hand::Right)
    }

    /// The slots of the brewing stand at `at`. A stand placed by a script has
    /// no block entity yet; with `create` one is written first.
    fn c4_brew(&mut self, at: &BlockPos, create: bool) -> Option<ContainerBlockEntity> {
        let w = self.block_world(at)?;
        let get = |w: &wapi::World| match w.get_block_entity(wpos(at)) {
            Some(BlockEntityType::BrewingStandBlockEntity(b)) => Some(b.get_container()),
            _ => None,
        };
        if let Some(c) = get(&w) {
            return Some(c);
        }
        if !create || !w.get_block(wpos(at)).name.ends_with(":brewing_stand") {
            return None;
        }
        // An empty compound with the id: tag 10, "id" string, end.
        let mut nbt = vec![10, 8, 0, 2, b'i', b'd'];
        let id = b"minecraft:brewing_stand";
        nbt.extend_from_slice(&(id.len() as u16).to_be_bytes());
        nbt.extend_from_slice(id);
        nbt.push(0);
        w.set_block_entity_nbt(wpos(at), &nbt).ok()?;
        get(&w)
    }

    pub(super) fn w5c4_impl(&mut self, q: Q<'_>) -> Option<Value> {
        let ok = |b: bool| Some(Value::Bool(b));
        match q {
            // Ambient and particle flags kept: the plain `add_effect` sets neither.
            Q::Effect(p, id, ticks, amplifier, ambient, show_particles) => {
                let effect_type = fx::effect(id)?;
                self.get(p)?.add_effect(StatusEffectInstance {
                    effect_type,
                    duration: ticks,
                    amplifier,
                    ambient,
                    show_particles,
                    show_icon: true,
                });
                ok(true)
            }
            Q::BlockData(at) => {
                let w = self.block_world(at)?;
                data_text(w.get_block_state_id(wpos(at))).map(Value::Text)
            }
            Q::SetBlockData(at, data) => {
                let w = self.block_world(at)?;
                let Some(state) = resolve(data) else {
                    return ok(false);
                };
                w.set_block_state(
                    wpos(at),
                    state,
                    BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::NOTIFY_LISTENERS,
                );
                ok(true)
            }
            Q::Brew(at, slot) => {
                let c = self.c4_brew(at, false)?;
                Some(
                    c.get_stack(u32::from(slot))
                        .map_or(Value::None, |st| super::gaps::value_of(&st)),
                )
            }
            Q::SetBrew(at, slot, v) => {
                let stack = match v {
                    Value::Item { id, .. } if id == "air" => None,
                    Value::None => None,
                    other => Some(super::gaps::stack_of(other)?),
                };
                self.c4_brew(at, true)?.set_stack(u32::from(slot), stack);
                ok(true)
            }
            Q::Border(world, f) => {
                let b = self.server.get_world_by_name(world)?.get_world_border();
                Some(match f {
                    BorderField::Center => Value::Location {
                        x: b.get_center_x(),
                        y: 0.0,
                        z: b.get_center_z(),
                        world: Some(world.to_string()),
                    },
                    BorderField::DamageAmount => Value::Num(b.get_damage_amount()),
                    BorderField::DamageBuffer => Value::Num(b.get_damage_buffer()),
                })
            }
            Q::SetBorder(world, f, v) => {
                let b = self.server.get_world_by_name(world)?.get_world_border();
                match (f, v) {
                    (BorderField::Center, Value::Location { x, z, .. }) => b.set_center(*x, *z),
                    (BorderField::DamageAmount, Value::Num(n)) => b.set_damage_amount(*n),
                    (BorderField::DamageBuffer, Value::Num(n)) => b.set_damage_buffer(*n),
                    _ => return ok(false),
                }
                ok(true)
            }
            Q::HeldBook(p) => {
                let stack = self.c4_held(p)?;
                let bytes = stack
                    .get_components()
                    .into_iter()
                    .find(|c| c.component == DataComponent::WrittenBookContent)?
                    .value;
                Some(Value::List(
                    bytes.into_iter().map(|b| Value::Num(f64::from(b))).collect(),
                ))
            }
            Q::SetHeldBook(p, bytes) => {
                let pl = self.get(p)?;
                let stack = pl.get_item_in_hand(Hand::Right)?;
                stack.set_component(DataComponent::WrittenBookContent, bytes);
                pl.set_item_in_hand(Hand::Right, Some(stack));
                ok(true)
            }
            Q::BarExists(id) => ok(Self::bar_exists(id)),
            Q::Ender(p, slot) => {
                let pl = self.get(p)?;
                Some(match slot {
                    Some(s) => pl
                        .get_ender_chest_item(s)
                        .map_or(Value::None, |st| super::gaps::value_of(&st)),
                    None => {
                        let inv = pl.get_ender_chest();
                        Value::List(
                            inv.get_all_items()
                                .into_iter()
                                .flatten()
                                .map(|st| super::gaps::value_of(&st))
                                .collect(),
                        )
                    }
                })
            }
            Q::SetEnder(p, slot, v) => {
                let stack = match v {
                    Value::Item { id, .. } if id == "air" => None,
                    Value::None => None,
                    other => Some(super::gaps::stack_of(other)?),
                };
                self.get(p)?.set_ender_chest_item(slot, stack);
                ok(true)
            }
            Q::ClearEnder(p) => {
                self.get(p)?.clear_ender_chest();
                ok(true)
            }
            Q::OpenEnder(p) => {
                self.get(p)?.open_ender_chest();
                ok(true)
            }
            Q::Ai(v) => {
                let mob = self.c4_entity(v)?.as_mob()?;
                ok(!mob.is_ai_disabled())
            }
            Q::SetAi(v, on) => {
                let mob = self.c4_entity(v)?.as_mob()?;
                mob.set_ai_disabled(!on);
                ok(true)
            }
            Q::Attr(v, name, total) => {
                let a = attribute(name)?;
                let l = self.c4_entity(v)?.as_living()?;
                Some(Value::Num(if total {
                    l.get_attribute_value(a)
                } else {
                    l.get_attribute_base(a)
                }))
            }
            Q::SetAttr(v, name, n) => {
                let a = attribute(name)?;
                self.c4_entity(v)?.as_living()?.set_attribute_base(a, n);
                ok(true)
            }
            Q::ResetAttr(v, name) => {
                let a = attribute(name)?;
                self.c4_entity(v)?.as_living()?.reset_attribute(a);
                ok(true)
            }
            Q::CarriedBlock(e) => {
                let mob = self.find_entity(&e.uuid)?.as_mob()?;
                match mob.get_mob_data() {
                    MobData::Enderman(d) => Some(
                        d.carried_block_state
                            .and_then(data_text)
                            .map_or(Value::None, Value::Text),
                    ),
                    _ => None,
                }
            }
            Q::SetCarriedBlock(e, data) => {
                let mob = self.find_entity(&e.uuid)?.as_mob()?;
                let MobData::Enderman(mut d) = mob.get_mob_data() else {
                    return ok(false);
                };
                d.carried_block_state = match data {
                    Some(s) => Some(resolve(s)?),
                    None => None,
                };
                ok(mob.set_mob_data(MobData::Enderman(d)))
            }
        }
    }
}

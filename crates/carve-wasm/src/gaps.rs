//! Host side of `carve_core::gaps`: inventory values opened as menus, names,
//! lore and enchantments written to real item stacks, entity rotation and the
//! mob death event.

use carve_core::combat::EntSlot;
use carve_core::gaps::InvVal;
use carve_core::item_meta::{ItemMeta, plain_from_json};
use carve_core::{EntityRef, EventCtx, EventKind, PlayerRef, Value};
use pumpkin_plugin_api::Enchantment;
use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::common::Hand;
use pumpkin_plugin_api::events::{EntityDeathEvent, EventData, EventHandler};
use pumpkin_plugin_api::item_stack::ItemStack;

use super::{TickHost, WasmHost, enchant_id, inv, text, with_state};

/// The vanilla enchantment with this Carve id (`sweeping_edge`).
pub fn enchantment(id: &str) -> Option<Enchantment> {
    use Enchantment as E;
    Some(match id {
        "aqua_affinity" => E::AquaAffinity,
        "bane_of_arthropods" => E::BaneOfArthropods,
        "binding_curse" => E::BindingCurse,
        "blast_protection" => E::BlastProtection,
        "breach" => E::Breach,
        "channeling" => E::Channeling,
        "density" => E::Density,
        "depth_strider" => E::DepthStrider,
        "efficiency" => E::Efficiency,
        "feather_falling" => E::FeatherFalling,
        "fire_aspect" => E::FireAspect,
        "fire_protection" => E::FireProtection,
        "flame" => E::Flame,
        "fortune" => E::Fortune,
        "frost_walker" => E::FrostWalker,
        "impaling" => E::Impaling,
        "infinity" => E::Infinity,
        "knockback" => E::Knockback,
        "looting" => E::Looting,
        "loyalty" => E::Loyalty,
        "luck_of_the_sea" => E::LuckOfTheSea,
        "lunge" => E::Lunge,
        "lure" => E::Lure,
        "mending" => E::Mending,
        "multishot" => E::Multishot,
        "piercing" => E::Piercing,
        "power" => E::Power,
        "projectile_protection" => E::ProjectileProtection,
        "protection" => E::Protection,
        "punch" => E::Punch,
        "quick_charge" => E::QuickCharge,
        "respiration" => E::Respiration,
        "riptide" => E::Riptide,
        "sharpness" => E::Sharpness,
        "silk_touch" => E::SilkTouch,
        "smite" => E::Smite,
        "soul_speed" => E::SoulSpeed,
        "sweeping_edge" => E::SweepingEdge,
        "swift_sneak" => E::SwiftSneak,
        "thorns" => E::Thorns,
        "unbreaking" => E::Unbreaking,
        "vanishing_curse" => E::VanishingCurse,
        "wind_burst" => E::WindBurst,
        _ => return None,
    })
}

/// Write name, lore and enchantments onto a stack, replacing what it had.
pub fn apply_meta(stack: &ItemStack, m: &ItemMeta) {
    stack.set_custom_name(m.name.as_deref().map(text));
    stack.set_lore(m.lore.iter().map(|l| text(l)).collect());
    for e in stack.get_enchantments() {
        let id = enchant_id(&format!("{:?}", e.enchantment));
        if m.level_of(&id) == 0 {
            stack.remove_enchantment(e.enchantment);
        }
    }
    for (id, level) in &m.enchants {
        if let Some(e) = enchantment(id) {
            stack.add_enchantment(e, *level);
        }
    }
    super::block_shapes_ai::set_damage(stack, m.damage);
}

/// A stack for an item value, with its meta. `None` for anything else.
pub fn stack_of(v: &Value) -> Option<ItemStack> {
    let Value::Item { id, count, meta } = v else {
        return None;
    };
    let s = ItemStack::new(&format!("minecraft:{id}"), (*count).clamp(1, 99) as u8);
    if let Some(m) = meta {
        apply_meta(&s, m);
    }
    Some(s)
}

/// An item value for a stack, with its name, lore and enchantments.
pub fn value_of(stack: &ItemStack) -> Value {
    let (id, count) = inv::pair(stack);
    let meta = ItemMeta {
        name: stack.get_custom_name().map(|n| plain_from_json(&n.to_json())),
        lore: stack
            .get_lore()
            .iter()
            .map(|l| plain_from_json(&l.to_json()))
            .collect(),
        enchants: stack
            .get_enchantments()
            .into_iter()
            .map(|e| (enchant_id(&format!("{:?}", e.enchantment)), e.level))
            .collect(),
        damage: super::block_shapes_ai::damage_of(stack),
        ..ItemMeta::default()
    };
    Value::Item {
        id,
        count,
        meta: (!meta.is_empty()).then(|| Box::new(meta)),
    }
}

impl WasmHost<'_> {
    pub(super) fn open_inventory(&mut self, p: &PlayerRef, v: &InvVal) {
        if let Some(pl) = self.get(p) {
            inv::open_value(pl, &p.uuid, v);
        }
    }

    pub(super) fn menu_update(&mut self, id: u64, slot: u32, item: &Value) {
        inv::set_by_id(id, slot, item);
    }

    pub(super) fn menu_set(&mut self, p: &PlayerRef, slot: u32, item: &Value) {
        inv::set_in_menu(&p.uuid, slot, item);
    }

    pub(super) fn set_held_item_meta(&mut self, p: &PlayerRef, m: &ItemMeta) -> bool {
        let Some(pl) = self.get(p) else {
            return false;
        };
        let Some(stack) = pl.get_item_in_hand(Hand::Right) else {
            return false;
        };
        apply_meta(&stack, m);
        pl.set_item_in_hand(Hand::Right, Some(stack));
        true
    }

    pub(super) fn inv_items(&mut self, p: &PlayerRef) -> Vec<Value> {
        let Some(pl) = self.get(p) else {
            return Vec::new();
        };
        let pi = pl.get_inventory();
        let mut out: Vec<Value> = pi
            .as_inventory()
            .get_all_items()
            .iter()
            .flatten()
            .map(value_of)
            .collect();
        for s in [
            pi.get_boots(),
            pi.get_leggings(),
            pi.get_chestplate(),
            pi.get_helmet(),
            pi.get_off_hand(),
        ]
        .into_iter()
        .flatten()
        {
            out.push(value_of(&s));
        }
        out.retain(|v| !matches!(v, Value::Item { id, .. } if id == "air"));
        out
    }

    pub(super) fn entity_rotation(&mut self, e: &EntityRef) -> Option<(f64, f64)> {
        let ent = self.find_entity(&e.uuid)?;
        Some((f64::from(ent.get_yaw()), f64::from(ent.get_pitch())))
    }
}

/// `on death` for mobs. Players have their own event (`combat::OnDeath`).
pub struct OnMobDeath;

impl EventHandler<EntityDeathEvent> for OnMobDeath {
    fn handle(&self, server: Server, data: EventData<EntityDeathEvent>) -> EventData<EntityDeathEvent> {
        let id = data.entity_id;
        with_state(&server, None, |st, host| {
            if !st.engine.table().has_event(EventKind::Death) || st.player_by_entity(host, id).is_some() {
                return;
            }
            let mut ctx = EventCtx::entity_victim(EntSlot::Id(id));
            if let Some((a, tick)) = st.hs.w4.hit_by.remove(&format!("#{id}"))
                && st.hs.tick.saturating_sub(tick) <= 100
            {
                match st.player_by_entity(host, a) {
                    Some(p) => ctx.attacker = Some(p),
                    None => ctx.set_entity_attacker(EntSlot::Id(a)),
                }
            }
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine.fire(EventKind::Death, ctx, &mut th);
        });
        data
    }
}

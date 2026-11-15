//! Wires the extra events of `carve_core::w2_events_2` to Pumpkin events.
//! Each handler builds a context with the event values and fires `on <event>`.

use super::*;
use carve_core::w2_events_2::{Ev, make_ctx};
use pumpkin_plugin_api::events::*;

type Build<E> = fn(&mut State, &mut WasmHost<'_>, &EventData<E>) -> Option<EventCtx>;
type Cancel<E> = fn(&mut EventData<E>);

struct Ext<E: FromIntoEvent> {
    ev: Ev,
    build: Build<E>,
    cancel: Option<Cancel<E>>,
}

impl<E: FromIntoEvent> EventHandler<E> for Ext<E> {
    fn handle(&self, server: Server, mut data: EventData<E>) -> EventData<E> {
        let kind = EventKind::Ext2(self.ev);
        let cancelled = with_state(&server, None, |st, host| {
            if !st.engine.table().has_event(kind) {
                return false;
            }
            let Some(ctx) = (self.build)(st, host, &data) else {
                return false;
            };
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine.fire(kind, ctx, &mut th).cancelled
        });
        if cancelled == Some(true)
            && let Some(c) = self.cancel
        {
            c(&mut data);
        }
        data
    }
}

fn register<E: FromIntoEvent + Send + Sync + 'static>(
    ctx: &Context,
    ev: Ev,
    build: Build<E>,
    cancel: Option<Cancel<E>>,
) {
    if let Err(e) = ctx.register_event_handler::<E, _>(Ext { ev, build, cancel }, EventPriority::Normal, true)
    {
        plog(
            ApiLevel::Error,
            &format!("could not listen to {ev:?} events: {e}"),
        );
    }
}

fn id(s: &str) -> &str {
    s.strip_prefix("minecraft:").unwrap_or(s)
}

fn f(n: i32) -> f64 {
    f64::from(n)
}

/// `register::<E>(ctx, Ev, |st, host, d| ..., cancel)` with the closure typed.
macro_rules! reg {
    ($c:expr, $E:ty, $ev:expr, cancel, |$st:ident, $h:ident, $d:ident| $b:expr) => {
        register::<$E>(
            $c,
            $ev,
            |$st: &mut State, $h: &mut WasmHost<'_>, $d: &EventData<$E>| $b,
            Some(|d: &mut EventData<$E>| d.cancelled = true),
        )
    };
    ($c:expr, $E:ty, $ev:expr, plain, |$st:ident, $h:ident, $d:ident| $b:expr) => {
        register::<$E>(
            $c,
            $ev,
            |$st: &mut State, $h: &mut WasmHost<'_>, $d: &EventData<$E>| $b,
            None,
        )
    };
}

pub fn register_all(c: &Context) {
    // ---- player events ----
    reg!(c, EnchantItemEvent, Ev::Enchant, cancel, |_s, _h, d| {
        let mut ctx = make_ctx(
            Ev::Enchant,
            Some(player_ref(&d.player)),
            &[f(d.option), f(d.cost), d.enchantments_to_add.len() as f64],
            &[id(&d.item.get_registry_key())],
            &[],
        );
        // `applied enchantments` (w5_chunk3) reads the texts after the item.
        if let Some(x) = ctx.ext.as_mut() {
            for e in &d.enchantments_to_add {
                let name = crate::enchant_id(&format!("{:?}", e.enchantment));
                x.texts.push(format!("{name} {}", e.level));
            }
        }
        Some(ctx)
    });
    reg!(
        c,
        PrepareItemEnchantEvent,
        Ev::EnchantPrepare,
        cancel,
        |_s, _h, d| Some(make_ctx(
            Ev::EnchantPrepare,
            Some(player_ref(&d.player)),
            &[f(d.bookshelf_count), d.offers.len() as f64],
            &[id(&d.item.get_registry_key())],
            &[],
        ))
    );
    reg!(c, PlayerExpChangeEvent, Ev::XpChange, plain, |_s, _h, d| Some(
        make_ctx(
            Ev::XpChange,
            Some(player_ref(&d.player)),
            &[f(d.amount)],
            &[],
            &[],
        )
    ));
    reg!(
        c,
        PlayerExpCooldownChangeEvent,
        Ev::XpCooldown,
        cancel,
        |_s, _h, d| Some(make_ctx(
            Ev::XpCooldown,
            Some(player_ref(&d.player)),
            &[f(d.new_cooldown)],
            &[],
            &[],
        ))
    );
    reg!(c, PlayerFishEvent, Ev::Fishing, cancel, |_s, _h, d| Some(
        make_ctx(
            Ev::Fishing,
            Some(player_ref(&d.player)),
            &[f(d.exp_to_drop)],
            &[&format!("{:?}", d.state).to_ascii_lowercase(), id(&d.caught_type)],
            &[],
        )
    ));
    reg!(c, PlayerHarvestBlockEvent, Ev::Harvest, cancel, |_s, _h, d| Some(
        make_ctx(
            Ev::Harvest,
            Some(player_ref(&d.player)),
            &[
                f(d.block_pos.x),
                f(d.block_pos.y),
                f(d.block_pos.z),
                d.harvested_items.len() as f64
            ],
            &[],
            &[],
        )
    ));
    reg!(c, FurnaceExtractEvent, Ev::FurnaceExtract, plain, |_s, _h, d| {
        Some(make_ctx(
            Ev::FurnaceExtract,
            Some(player_ref(&d.player)),
            &[
                f(d.block_pos.x),
                f(d.block_pos.y),
                f(d.block_pos.z),
                f64::from(d.item_amount),
                f64::from(d.exp_gained),
            ],
            &[id(&d.item_id)],
            &[],
        ))
    });
    reg!(c, InventoryOpenEvent, Ev::InvOpen, cancel, |_s, _h, d| Some(
        make_ctx(Ev::InvOpen, Some(player_ref(&d.player)), &[], &[], &[],)
    ));
    reg!(c, InventoryDragEvent, Ev::InvDrag, cancel, |_s, _h, d| Some(
        make_ctx(Ev::InvDrag, Some(player_ref(&d.player)), &[], &[], &[],)
    ));
    reg!(c, PlayerItemBreakEvent, Ev::ItemBreak, plain, |_s, _h, d| Some(
        make_ctx(
            Ev::ItemBreak,
            Some(player_ref(&d.player)),
            &[],
            &[id(&d.item_name)],
            &[],
        )
    ));
    reg!(
        c,
        PlayerItemDamageEvent,
        Ev::ItemDamage,
        cancel,
        |_s, _h, d| Some(make_ctx(
            Ev::ItemDamage,
            Some(player_ref(&d.player)),
            &[f(d.damage)],
            &[id(&d.item_name)],
            &[],
        ))
    );
    reg!(c, PlayerItemMendEvent, Ev::ItemMend, cancel, |_s, _h, d| Some(
        make_ctx(
            Ev::ItemMend,
            Some(player_ref(&d.player)),
            &[f(d.repair_amount), f(d.exp_consumed)],
            &[id(&d.item_name)],
            &[],
        )
    ));

    // ---- entity events (a player is set when the entity is one) ----
    reg!(c, EntityRegainHealthEvent, Ev::Heal, cancel, |s, h, d| {
        let p = s.player_by_entity(h, d.entity_id);
        Some(make_ctx(
            Ev::Heal,
            p,
            &[f64::from(d.amount), f(d.entity_id)],
            &[],
            &[],
        ))
    });
    reg!(c, FoodLevelChangeEvent, Ev::Hunger, cancel, |s, h, d| {
        let p = s.player_by_entity(h, d.entity_id);
        Some(make_ctx(
            Ev::Hunger,
            p,
            &[f64::from(d.food_level), f(d.entity_id)],
            &[],
            &[],
        ))
    });
    reg!(c, EntityMountEvent, Ev::Mount, cancel, |s, h, d| {
        let p = s.player_by_entity(h, d.entity_id);
        Some(make_ctx(
            Ev::Mount,
            p,
            &[f(d.entity_id), f(d.mounted_id)],
            &[],
            &[],
        ))
    });
    reg!(c, EntityDismountEvent, Ev::Dismount, cancel, |s, h, d| {
        let p = s.player_by_entity(h, d.entity_id);
        Some(make_ctx(
            Ev::Dismount,
            p,
            &[f(d.entity_id), f(d.dismounted_id)],
            &[],
            &[],
        ))
    });
    reg!(c, EntityPotionEffectEvent, Ev::PotionEffect, cancel, |s, h, d| {
        let p = s.player_by_entity(h, d.entity_id);
        Some(make_ctx(
            Ev::PotionEffect,
            p,
            &[f(d.duration), f64::from(d.amplifier), f(d.entity_id)],
            &[id(&d.effect_name)],
            &[],
        ))
    });
    reg!(c, EntityShootBowEvent, Ev::ShootBow, cancel, |s, h, d| {
        let p = s.player_by_entity(h, d.entity_id);
        Some(make_ctx(
            Ev::ShootBow,
            p,
            &[f64::from(d.force), f(d.entity_id)],
            &[id(&d.weapon_name)],
            &[],
        ))
    });
    reg!(c, EntityTransformEvent, Ev::Transform, cancel, |s, h, d| {
        let p = s.player_by_entity(h, d.entity_id);
        Some(make_ctx(
            Ev::Transform,
            p,
            &[f(d.entity_id), f(d.new_entity_id)],
            &[&d.transform_reason.to_ascii_lowercase()],
            &[],
        ))
    });
    reg!(c, EntityToggleGlideEvent, Ev::Glide, cancel, |s, h, d| {
        let p = s.player_by_entity(h, d.entity_id);
        Some(make_ctx(Ev::Glide, p, &[f(d.entity_id)], &[], &[d.is_gliding]))
    });

    // ---- events without a player ----
    reg!(c, EntityChangeBlockEvent, Ev::ChangeBlock, cancel, |_s, _h, d| {
        Some(make_ctx(
            Ev::ChangeBlock,
            None,
            &[
                f(d.entity_id),
                f(d.block_pos.x),
                f(d.block_pos.y),
                f(d.block_pos.z),
            ],
            &[id(&d.new_block)],
            &[],
        ))
    });
    reg!(c, EntityBreedEvent, Ev::Breed, cancel, |_s, _h, d| Some(
        make_ctx(
            Ev::Breed,
            None,
            &[f(d.father_id), f(d.mother_id), f(d.child_id)],
            &[],
            &[],
        )
    ));
    reg!(c, EntityExplodeEvent, Ev::Explode, cancel, |_s, _h, d| Some(
        make_ctx(
            Ev::Explode,
            None,
            &[
                f64::from(d.yield_rate),
                d.position.0,
                d.position.1,
                d.position.2,
                f(d.entity_id)
            ],
            &[],
            &[],
        )
    ));
    reg!(c, BlockExplodeEvent, Ev::Explode, cancel, |_s, _h, d| Some(
        make_ctx(
            Ev::Explode,
            None,
            &[
                f64::from(d.yield_rate),
                f(d.block_pos.x),
                f(d.block_pos.y),
                f(d.block_pos.z),
                -1.0
            ],
            &[],
            &[],
        )
    ));
    reg!(c, ExplosionPrimeEvent, Ev::ExplosionPrime, cancel, |_s, _h, d| {
        Some(make_ctx(
            Ev::ExplosionPrime,
            None,
            &[f64::from(d.radius), f(d.entity_id)],
            &[],
            &[d.fire],
        ))
    });
    reg!(c, BlockFadeEvent, Ev::Fade, cancel, |_s, _h, d| Some(make_ctx(
        Ev::Fade,
        None,
        &[f(d.block_pos.x), f(d.block_pos.y), f(d.block_pos.z)],
        &[],
        &[],
    )));
    reg!(
        c,
        FireworkExplodeEvent,
        Ev::FireworkExplode,
        cancel,
        |_s, _h, d| Some(make_ctx(Ev::FireworkExplode, None, &[f(d.entity_id)], &[], &[],))
    );
    reg!(c, BlockFromToEvent, Ev::Flow, cancel, |_s, _h, d| Some(make_ctx(
        Ev::Flow,
        None,
        &[
            f(d.from_pos.x),
            f(d.from_pos.y),
            f(d.from_pos.z),
            f(d.to_pos.x),
            f(d.to_pos.y),
            f(d.to_pos.z),
        ],
        &[],
        &[],
    )));
    reg!(c, BlockFormEvent, Ev::Form, cancel, |_s, _h, d| Some(make_ctx(
        Ev::Form,
        None,
        &[f(d.block_pos.x), f(d.block_pos.y), f(d.block_pos.z)],
        &[],
        &[],
    )));
    reg!(c, FurnaceBurnEvent, Ev::FuelBurn, cancel, |_s, _h, d| Some(
        make_ctx(
            Ev::FuelBurn,
            None,
            &[
                f(d.block_pos.x),
                f(d.block_pos.y),
                f(d.block_pos.z),
                f64::from(d.burn_time)
            ],
            &[id(&d.fuel_item)],
            &[],
        )
    ));
    reg!(c, BlockGrowEvent, Ev::Grow, cancel, |_s, _h, d| Some(make_ctx(
        Ev::Grow,
        None,
        &[f(d.block_pos.x), f(d.block_pos.y), f(d.block_pos.z)],
        &[id(&d.old_block), id(&d.new_block)],
        &[],
    )));
    reg!(c, HorseJumpEvent, Ev::HorseJump, cancel, |_s, _h, d| Some(
        make_ctx(
            Ev::HorseJump,
            None,
            &[f64::from(d.power), f(d.entity_id)],
            &[],
            &[],
        )
    ));
    reg!(c, BlockIgniteEvent, Ev::Ignition, cancel, |_s, _h, d| Some(
        make_ctx(
            Ev::Ignition,
            None,
            &[f(d.block_pos.x), f(d.block_pos.y), f(d.block_pos.z)],
            &[],
            &[],
        )
    ));
    reg!(c, InventoryMoveItemEvent, Ev::InvMove, cancel, |_s, _h, d| Some(
        make_ctx(
            Ev::InvMove,
            None,
            &[
                f(d.source_pos.x),
                f(d.source_pos.y),
                f(d.source_pos.z),
                f(d.target_pos.x),
                f(d.target_pos.y),
                f(d.target_pos.z),
                f64::from(d.item_amount),
            ],
            &[id(&d.item_id)],
            &[],
        )
    ));
    reg!(c, InventoryPickupItemEvent, Ev::InvPickup, cancel, |_s, _h, d| {
        Some(make_ctx(
            Ev::InvPickup,
            None,
            &[
                f(d.block_pos.x),
                f(d.block_pos.y),
                f(d.block_pos.z),
                f(d.item_entity_id),
            ],
            &[id(&d.item_id)],
            &[],
        ))
    });
    reg!(c, ItemDespawnEvent, Ev::ItemDespawn, cancel, |_s, _h, d| Some(
        make_ctx(Ev::ItemDespawn, None, &[f(d.entity_id)], &[], &[],)
    ));
    reg!(c, ItemMergeEvent, Ev::ItemMerge, cancel, |_s, _h, d| Some(
        make_ctx(Ev::ItemMerge, None, &[f(d.entity_id), f(d.target_id)], &[], &[],)
    ));
    reg!(c, ItemSpawnEvent, Ev::ItemSpawn, cancel, |_s, _h, d| Some(
        make_ctx(
            Ev::ItemSpawn,
            None,
            &[f(d.entity_id), d.position.0, d.position.1, d.position.2],
            &[id(&d.item_name)],
            &[],
        )
    ));
}

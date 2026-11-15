//! Item, mob and level events: drop, pick up, consume, projectile hit,
//! mob spawn, gamemode change, craft and level change.

use super::*;
use carve_core::{EvPos, EventExtra};
use pumpkin_plugin_api::events::{
    CraftItemEvent, CreatureSpawnEvent, EntityPickupItemEvent, PlayerDropItemEvent,
    PlayerGamemodeChangeEvent, PlayerItemConsumeEvent, PlayerLevelChangeEvent, ProjectileHitEvent,
};

/// `minecraft:stone` -> `stone`.
fn short(s: &str) -> String {
    s.strip_prefix("minecraft:").unwrap_or(s).to_string()
}

fn mode_name(m: GameMode) -> String {
    match m {
        GameMode::Survival => "survival",
        GameMode::Creative => "creative",
        GameMode::Adventure => "adventure",
        GameMode::Spectator => "spectator",
    }
    .to_string()
}

impl EventAccess for PlayerDropItemEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        let extra = EventExtra {
            item: Some((short(&d.item_name), u32::from(d.count))),
            ..Default::default()
        };
        EventCtx::with_extra(player_ref(&d.player), extra)
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerItemConsumeEvent {
    /// Pumpkin fires this for every right click with any item (even an empty
    /// hand), so only food and drinks reach `on consume`.
    fn skip(d: &EventData<Self>) -> bool {
        !carve_core::conditions_values::is_edible(&short(&d.item_name))
    }
    fn ctx(d: &EventData<Self>) -> EventCtx {
        let extra = EventExtra {
            item: Some((short(&d.item_name), 1)),
            ..Default::default()
        };
        EventCtx::with_extra(player_ref(&d.player), extra)
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for CraftItemEvent {
    /// The server gives the recipe id only; recipes named like their result
    /// (most of them) also fill `event-item`.
    fn ctx(d: &EventData<Self>) -> EventCtx {
        let recipe = short(&d.recipe_id);
        let extra = EventExtra {
            item: Some((recipe.clone(), 1)),
            recipe: Some(recipe),
            ..Default::default()
        };
        EventCtx::with_extra(player_ref(&d.player), extra)
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerGamemodeChangeEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        let extra = EventExtra {
            old_mode: Some(mode_name(d.previous_gamemode)),
            new_mode: Some(mode_name(d.new_gamemode)),
            ..Default::default()
        };
        EventCtx::with_extra(player_ref(&d.player), extra)
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerLevelChangeEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        let extra = EventExtra {
            old_level: Some(i64::from(d.old_level)),
            new_level: Some(i64::from(d.new_level)),
            ..Default::default()
        };
        EventCtx::with_extra(player_ref(&d.player), extra)
    }
    fn set_cancelled(_d: &mut EventData<Self>) {}
}

impl EventAccess for CreatureSpawnEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        let (x, y, z) = d.position;
        let extra = EventExtra {
            entity: Some(short(&d.entity_type)),
            location: Some(EvPos {
                x,
                y,
                z,
                world: Some(world_name(&d.target_world)),
            }),
            reason: Some(d.spawn_reason.to_ascii_lowercase()),
            ..Default::default()
        };
        EventCtx::console_with_extra(extra)
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

/// An item picked up by an entity. Only players are reported.
struct OnPickUp;

impl EventHandler<EntityPickupItemEvent> for OnPickUp {
    fn handle(
        &self,
        server: Server,
        mut data: EventData<EntityPickupItemEvent>,
    ) -> EventData<EntityPickupItemEvent> {
        let cancelled = with_state(&server, None, |st, host| {
            if !st.engine.table().has_event(EventKind::PickUp) {
                return false;
            }
            let Some(p) = st.player_by_entity(host, data.entity_id) else {
                return false;
            };
            // Pumpkin 0.2.2 sends the item entity's type id ("72") here, not
            // the item; leave `event-item` unset rather than a wrong item.
            let item = (!data.item_name.bytes().all(|b| b.is_ascii_digit()))
                .then(|| (short(&data.item_name), u32::from(data.count)));
            let extra = EventExtra {
                item,
                ..Default::default()
            };
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine
                .fire(EventKind::PickUp, EventCtx::with_extra(p, extra), &mut th)
                .cancelled
        });
        if cancelled == Some(true) {
            data.cancelled = true;
        }
        data
    }
}

/// A projectile landed. `hit player` is set when it hit a player.
struct OnProjectileHit;

impl EventHandler<ProjectileHitEvent> for OnProjectileHit {
    fn handle(
        &self,
        server: Server,
        mut data: EventData<ProjectileHitEvent>,
    ) -> EventData<ProjectileHitEvent> {
        let cancelled = with_state(&server, None, |st, host| {
            if !st.engine.table().has_event(EventKind::ProjectileHit) {
                return false;
            }
            let hit_player = match data.hit_entity_id {
                Some(id) => st.player_by_entity(host, id),
                None => None,
            };
            let (x, y, z) = data.hit_position;
            let extra = EventExtra {
                location: Some(EvPos { x, y, z, world: None }),
                hit_player,
                ..Default::default()
            };
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine
                .fire(
                    EventKind::ProjectileHit,
                    EventCtx::console_with_extra(extra),
                    &mut th,
                )
                .cancelled
        });
        if cancelled == Some(true) {
            data.cancelled = true;
        }
        data
    }
}

pub fn register(context: &Context, table: &carve_core::engine::ScriptTable) {
    register_event::<PlayerDropItemEvent>(context, EventKind::Drop, table);
    register_event::<PlayerItemConsumeEvent>(context, EventKind::Consume, table);
    register_event::<CraftItemEvent>(context, EventKind::Craft, table);
    register_event::<PlayerGamemodeChangeEvent>(context, EventKind::GamemodeChange, table);
    register_event::<PlayerLevelChangeEvent>(context, EventKind::LevelChange, table);
    register_event::<CreatureSpawnEvent>(context, EventKind::MobSpawn, table);
    if let Err(e) =
        context.register_event_handler::<EntityPickupItemEvent, _>(OnPickUp, EventPriority::Normal, true)
    {
        plog(
            ApiLevel::Error,
            &format!("could not listen to pick up events: {e}"),
        );
    }
    if let Err(e) =
        context.register_event_handler::<ProjectileHitEvent, _>(OnProjectileHit, EventPriority::Normal, true)
    {
        plog(
            ApiLevel::Error,
            &format!("could not listen to projectile hit events: {e}"),
        );
    }
}

//! Extra event triggers (kick, locale change, piston, portal, redstone, ...).
//! Each game event is turned into a small [`Extract`] and fired as
//! `EventKind::PlayerServer(..)`.

use std::marker::PhantomData;

use carve_core::events_player_server::PlayerServerEvent;
use carve_core::{EventCtx, EventKind, FireOutcome, PlayerRef, Value};
use pumpkin_plugin_api::Context;
use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::events::{
    BlockPistonExtendEvent, BlockPistonRetractEvent, BlockRedstoneEvent, EntityEnterLoveModeEvent,
    EntityPortalEnterEvent, EntityPortalEvent, EntityResurrectEvent, EntityUnleashEvent, EventData,
    EventHandler, EventPriority, FromIntoEvent, LeavesDecayEvent, LightningStrikeEvent, LootGenerateEvent,
    PiglinBarterEvent, PlayerCommandSendEvent, PlayerInputEvent, PlayerKickEvent, PlayerLeashEntityEvent,
    PlayerLocaleChangeEvent, PlayerResourcePackStatusEvent, PortalCreateEvent,
};
use pumpkin_plugin_api::logging::{LogLevel as ApiLevel, log};

use super::{TickHost, player_ref, with_state};

/// What the script sees of one game event.
#[derive(Default)]
struct Extract {
    player: Option<PlayerRef>,
    /// An entity id to look up among online players (when `player` is unset).
    entity: Option<i32>,
    text: Option<String>,
    number: Option<f64>,
    /// `event-...` values in `EventCtx::ev` (the barter input and drops).
    ev: Vec<Value>,
    /// An entity id looked up when the event fires and pushed after `ev`
    /// as `event-entity`.
    event_entity: Option<i32>,
}

/// A block position as an `event-location`.
fn loc(world: Option<String>, p: &pumpkin_plugin_api::common::BlockPos) -> Value {
    Value::Location {
        x: f64::from(p.x),
        y: f64::from(p.y),
        z: f64::from(p.z),
        world,
    }
}

trait ExtEvent: FromIntoEvent {
    fn extract(d: &EventData<Self>) -> Extract;
    fn cancel(d: &mut EventData<Self>);
    /// Write values the script changed back into the event.
    fn apply(_d: &mut EventData<Self>, _o: &FireOutcome) {}
}

macro_rules! ext {
    ($ty:ty, |$d:ident| $body:expr) => {
        impl ExtEvent for $ty {
            fn extract($d: &EventData<Self>) -> Extract {
                $body
            }
            fn cancel(d: &mut EventData<Self>) {
                d.cancelled = true;
            }
        }
    };
}

ext!(PlayerKickEvent, |d| Extract {
    player: Some(player_ref(&d.player)),
    text: Some(d.reason.clone()),
    ..Extract::default()
});
ext!(PlayerLocaleChangeEvent, |d| Extract {
    player: Some(player_ref(&d.player)),
    text: Some(d.new_locale.clone()),
    ..Extract::default()
});
ext!(PlayerLeashEntityEvent, |d| Extract {
    player: Some(player_ref(&d.player)),
    event_entity: Some(d.entity_id),
    ..Extract::default()
});
ext!(EntityUnleashEvent, |d| Extract {
    text: Some(d.reason.clone()),
    event_entity: Some(d.entity_id),
    ..Extract::default()
});
ext!(LeavesDecayEvent, |d| Extract {
    ev: vec![loc(Some(super::world_name(&d.target_world)), &d.block_pos)],
    ..Extract::default()
});
ext!(LightningStrikeEvent, |d| Extract {
    ev: vec![Value::Location {
        x: d.position.0,
        y: d.position.1,
        z: d.position.2,
        world: None,
    }],
    ..Extract::default()
});
ext!(EntityEnterLoveModeEvent, |d| Extract {
    entity: d.human_entity_id,
    number: Some(f64::from(d.ticks_in_love)),
    event_entity: Some(d.entity_id),
    ..Extract::default()
});
impl ExtEvent for PiglinBarterEvent {
    fn extract(d: &EventData<Self>) -> Extract {
        Extract {
            ev: vec![
                crate::gaps::value_of(&d.input_item),
                Value::List(d.outcome.iter().map(crate::gaps::value_of).collect()),
            ],
            ..Extract::default()
        }
    }
    fn cancel(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
    fn apply(d: &mut EventData<Self>, o: &FireOutcome) {
        let slot = carve_core::conditions_mobs_items::DROPS_SLOT;
        if let Some((_, Value::List(l))) = o.set.iter().rev().find(|(s, _)| *s == slot) {
            d.outcome = l.iter().filter_map(crate::gaps::stack_of).collect();
        }
    }
}
ext!(BlockPistonExtendEvent, |d| Extract {
    text: Some(d.direction.clone()),
    ev: vec![loc(None, &d.block_pos)],
    ..Extract::default()
});
ext!(BlockPistonRetractEvent, |d| Extract {
    text: Some(d.direction.clone()),
    ev: vec![loc(None, &d.block_pos)],
    ..Extract::default()
});
ext!(EntityPortalEvent, |d| Extract {
    entity: Some(d.entity_id),
    ..Extract::default()
});
ext!(EntityPortalEnterEvent, |d| Extract {
    entity: Some(d.entity_id),
    ..Extract::default()
});
ext!(PortalCreateEvent, |d| Extract {
    text: Some(d.portal_type.clone()),
    ev: vec![loc(None, &d.pos)],
    ..Extract::default()
});
ext!(BlockRedstoneEvent, |d| Extract {
    number: Some(f64::from(d.new_current)),
    ev: vec![loc(Some(super::world_name(&d.target_world)), &d.block_pos)],
    ..Extract::default()
});
ext!(PlayerResourcePackStatusEvent, |d| Extract {
    player: Some(player_ref(&d.player)),
    text: Some(d.status.clone()),
    ..Extract::default()
});
ext!(EntityResurrectEvent, |d| Extract {
    entity: Some(d.entity_id),
    ..Extract::default()
});
ext!(PlayerCommandSendEvent, |d| Extract {
    player: Some(player_ref(&d.player)),
    text: Some(d.command.clone()),
    ..Extract::default()
});
ext!(LootGenerateEvent, |d| Extract {
    text: Some(d.loot_table.clone()),
    ..Extract::default()
});
ext!(PlayerInputEvent, |d| Extract {
    player: Some(player_ref(&d.player)),
    number: u32::from_str_radix(&d.input, 2).ok().map(f64::from),
    text: Some(d.input.clone()),
    ..Extract::default()
});

struct OnExt<E>(PlayerServerEvent, PhantomData<fn() -> E>);

impl<E: ExtEvent> EventHandler<E> for OnExt<E> {
    fn handle(&self, server: Server, mut data: EventData<E>) -> EventData<E> {
        let kind = EventKind::PlayerServer(self.0);
        let cancelled = with_state(&server, None, |st, host| {
            if !st.engine.table().has_event(kind) {
                return None;
            }
            let x = E::extract(&data);
            let player = match (x.player, x.entity) {
                (Some(p), _) => Some(p),
                (None, Some(id)) => st.player_by_entity(host, id),
                (None, None) => None,
            };
            let mut ctx = match player {
                Some(p) => EventCtx::player(p),
                None => EventCtx::console(),
            };
            ctx.message = x.text;
            ctx.damage = x.number;
            ctx.ev = x.ev;
            if let Some(id) = x.event_entity {
                ctx.ev
                    .push(host.entity_by_id(id).map_or(Value::None, Value::Entity));
            }
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            Some(st.engine.fire(kind, ctx, &mut th))
        });
        if let Some(Some(o)) = cancelled {
            E::apply(&mut data, &o);
            if o.cancelled {
                E::cancel(&mut data);
            }
        }
        data
    }
}

/// High-volume events: the server only calls into the plugin for these when
/// a script uses them at startup. Adding the first script for one needs a
/// server restart.
fn is_hot(ev: PlayerServerEvent) -> bool {
    matches!(
        ev,
        PlayerServerEvent::Redstone
            | PlayerServerEvent::PistonExtend
            | PlayerServerEvent::PistonRetract
            | PlayerServerEvent::LeavesDecay
            | PlayerServerEvent::PlayerInput
    )
}

fn reg<E: ExtEvent + Send + Sync + 'static>(
    ctx: &Context,
    used: &dyn Fn(PlayerServerEvent) -> bool,
    ev: PlayerServerEvent,
) {
    if is_hot(ev) && !used(ev) {
        return;
    }
    let h = OnExt::<E>(ev, PhantomData);
    if let Err(e) = ctx.register_event_handler::<E, _>(h, EventPriority::Normal, true) {
        log(
            ApiLevel::Error,
            &format!("could not listen to {} events: {e}", ev.name()),
        );
    }
}

/// Listen to the extra events. Handlers return at once when no script uses
/// the event.
pub fn register_all(ctx: &Context, used: &dyn Fn(PlayerServerEvent) -> bool) {
    reg::<PlayerKickEvent>(ctx, used, PlayerServerEvent::Kick);
    reg::<PlayerLocaleChangeEvent>(ctx, used, PlayerServerEvent::LanguageChange);
    reg::<PlayerLeashEntityEvent>(ctx, used, PlayerServerEvent::Leash);
    reg::<EntityUnleashEvent>(ctx, used, PlayerServerEvent::Unleash);
    reg::<LeavesDecayEvent>(ctx, used, PlayerServerEvent::LeavesDecay);
    reg::<LightningStrikeEvent>(ctx, used, PlayerServerEvent::LightningStrike);
    reg::<EntityEnterLoveModeEvent>(ctx, used, PlayerServerEvent::LoveModeEnter);
    reg::<PiglinBarterEvent>(ctx, used, PlayerServerEvent::PiglinBarter);
    reg::<BlockPistonExtendEvent>(ctx, used, PlayerServerEvent::PistonExtend);
    reg::<BlockPistonRetractEvent>(ctx, used, PlayerServerEvent::PistonRetract);
    reg::<EntityPortalEvent>(ctx, used, PlayerServerEvent::Portal);
    reg::<EntityPortalEnterEvent>(ctx, used, PlayerServerEvent::PortalEnter);
    reg::<PortalCreateEvent>(ctx, used, PlayerServerEvent::PortalCreate);
    reg::<BlockRedstoneEvent>(ctx, used, PlayerServerEvent::Redstone);
    reg::<PlayerResourcePackStatusEvent>(ctx, used, PlayerServerEvent::ResourcePackResponse);
    reg::<EntityResurrectEvent>(ctx, used, PlayerServerEvent::ResurrectAttempt);
    reg::<PlayerCommandSendEvent>(ctx, used, PlayerServerEvent::SendCommandList);
    reg::<LootGenerateEvent>(ctx, used, PlayerServerEvent::LootGenerate);
    reg::<PlayerInputEvent>(ctx, used, PlayerServerEvent::PlayerInput);
}

//! Extra event triggers (kick, locale change, piston, portal, redstone, ...).
//! Each game event is turned into a small [`Extract`] and fired as
//! `EventKind::Ext(..)`.

use std::marker::PhantomData;

use carve_core::w2_events_3::Ev3;
use carve_core::{EventCtx, EventKind, PlayerRef};
use pumpkin_plugin_api::Context;
use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::events::{
    BlockPistonExtendEvent, BlockPistonRetractEvent, BlockRedstoneEvent, EntityEnterLoveModeEvent,
    EntityPortalEnterEvent, EntityPortalEvent, EntityResurrectEvent, EntityUnleashEvent, EventData, EventHandler,
    EventPriority, FromIntoEvent, ItemMergeEvent, ItemSpawnEvent, LeavesDecayEvent, LightningStrikeEvent,
    LootGenerateEvent, PiglinBarterEvent, PlayerCommandSendEvent, PlayerInputEvent, PlayerItemMendEvent, PlayerKickEvent, PlayerLeashEntityEvent,
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
}

trait ExtEvent: FromIntoEvent {
    fn extract(d: &EventData<Self>) -> Extract;
    fn cancel(d: &mut EventData<Self>);
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
ext!(PlayerItemMendEvent, |d| Extract {
    player: Some(player_ref(&d.player)),
    text: Some(d.item_name.strip_prefix("minecraft:").unwrap_or(&d.item_name).to_string()),
    number: Some(f64::from(d.repair_amount)),
    ..Extract::default()
});
ext!(PlayerLeashEntityEvent, |d| Extract {
    player: Some(player_ref(&d.player)),
    ..Extract::default()
});
ext!(EntityUnleashEvent, |d| Extract {
    text: Some(d.reason.clone()),
    ..Extract::default()
});
ext!(LeavesDecayEvent, |_d| Extract::default());
ext!(LightningStrikeEvent, |_d| Extract::default());
ext!(EntityEnterLoveModeEvent, |d| Extract {
    entity: d.human_entity_id,
    number: Some(f64::from(d.ticks_in_love)),
    ..Extract::default()
});
ext!(PiglinBarterEvent, |_d| Extract::default());
ext!(BlockPistonExtendEvent, |d| Extract {
    text: Some(d.direction.clone()),
    ..Extract::default()
});
ext!(BlockPistonRetractEvent, |d| Extract {
    text: Some(d.direction.clone()),
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
    ..Extract::default()
});
ext!(BlockRedstoneEvent, |d| Extract {
    number: Some(f64::from(d.new_current)),
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
ext!(ItemSpawnEvent, |d| Extract {
    text: Some(d.item_name.strip_prefix("minecraft:").unwrap_or(&d.item_name).to_string()),
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
ext!(ItemMergeEvent, |_d| Extract::default());

struct OnExt<E>(Ev3, PhantomData<fn() -> E>);

impl<E: ExtEvent> EventHandler<E> for OnExt<E> {
    fn handle(&self, server: Server, mut data: EventData<E>) -> EventData<E> {
        let kind = EventKind::Ext(self.0);
        let cancelled = with_state(&server, None, |st, host| {
            if !st.engine.table().has_event(kind) {
                return false;
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
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine.fire(kind, ctx, &mut th).cancelled
        });
        if cancelled == Some(true) {
            E::cancel(&mut data);
        }
        data
    }
}

/// High-volume events: the server only calls into the plugin for these when
/// a script uses them at startup. Adding the first script for one needs a
/// server restart.
fn is_hot(ev: Ev3) -> bool {
    matches!(
        ev,
        Ev3::Redstone | Ev3::PistonExtend | Ev3::PistonRetract | Ev3::ItemSpawn | Ev3::ItemMerge | Ev3::LeavesDecay | Ev3::PlayerInput
    )
}

fn reg<E: ExtEvent + Send + Sync + 'static>(ctx: &Context, used: &dyn Fn(Ev3) -> bool, ev: Ev3) {
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
pub fn register_all(ctx: &Context, used: &dyn Fn(Ev3) -> bool) {
    reg::<PlayerKickEvent>(ctx, used, Ev3::Kick);
    reg::<PlayerLocaleChangeEvent>(ctx, used, Ev3::LanguageChange);
    reg::<PlayerItemMendEvent>(ctx, used, Ev3::ItemMend);
    reg::<PlayerLeashEntityEvent>(ctx, used, Ev3::Leash);
    reg::<EntityUnleashEvent>(ctx, used, Ev3::Unleash);
    reg::<LeavesDecayEvent>(ctx, used, Ev3::LeavesDecay);
    reg::<LightningStrikeEvent>(ctx, used, Ev3::LightningStrike);
    reg::<EntityEnterLoveModeEvent>(ctx, used, Ev3::LoveModeEnter);
    reg::<PiglinBarterEvent>(ctx, used, Ev3::PiglinBarter);
    reg::<BlockPistonExtendEvent>(ctx, used, Ev3::PistonExtend);
    reg::<BlockPistonRetractEvent>(ctx, used, Ev3::PistonRetract);
    reg::<EntityPortalEvent>(ctx, used, Ev3::Portal);
    reg::<EntityPortalEnterEvent>(ctx, used, Ev3::PortalEnter);
    reg::<PortalCreateEvent>(ctx, used, Ev3::PortalCreate);
    reg::<BlockRedstoneEvent>(ctx, used, Ev3::Redstone);
    reg::<PlayerResourcePackStatusEvent>(ctx, used, Ev3::ResourcePackResponse);
    reg::<EntityResurrectEvent>(ctx, used, Ev3::ResurrectAttempt);
    reg::<PlayerCommandSendEvent>(ctx, used, Ev3::SendCommandList);
    reg::<ItemSpawnEvent>(ctx, used, Ev3::ItemSpawn);
    reg::<ItemMergeEvent>(ctx, used, Ev3::ItemMerge);
    reg::<LootGenerateEvent>(ctx, used, Ev3::LootGenerate);
    reg::<PlayerInputEvent>(ctx, used, Ev3::PlayerInput);
}

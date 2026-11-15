//! `on command`, `on first join`, `on move`, `on swap hand items`,
//! `on teleport` and `on world change`. Each game event becomes an
//! `EventKind::Mv(..)` with its values in `EventCtx::ev`.

use std::collections::HashSet;
use std::marker::PhantomData;
use std::sync::Mutex;

use carve_core::w4_events_movement::{self as mv, MvEvent};
use carve_core::{EventCtx, EventKind};
use pumpkin_plugin_api::Context;
use pumpkin_plugin_api::Server;
use pumpkin_plugin_api::events::{
    EventData, EventHandler, EventPriority, FromIntoEvent, PlayerChangedWorldEvent, PlayerCommandSendEvent,
    PlayerJoinEvent, PlayerMoveEvent, PlayerSwapHandsEvent, PlayerTeleportEvent,
};
use pumpkin_plugin_api::logging::{LogLevel as ApiLevel, log};

use super::{TickHost, player_ref, with_state, world_name};

/// Players that have joined before, as uuids (table `seen_players` of the store).
static SEEN: Mutex<Option<HashSet<String>>> = Mutex::new(None);
/// Players whose first join is in this server run: they have not played before.
static FRESH: Mutex<Vec<String>> = Mutex::new(Vec::new());

/// The players the store knows, read once at load.
pub(crate) fn init_seen(uuids: Vec<String>) {
    if let Ok(mut g) = SEEN.lock() {
        *g = Some(uuids.into_iter().collect());
    }
}

/// A player left: from their next join on, they have played before.
pub(crate) fn left(uuid: &str) {
    if let Ok(mut fresh) = FRESH.lock() {
        fresh.retain(|u| u != uuid);
    }
}

/// `player has played before`: joined in an earlier server run, or left and
/// came back in this one. `None` when the list can't be read.
pub(crate) fn played_before(uuid: &str) -> Option<bool> {
    if FRESH.lock().ok()?.iter().any(|u| u == uuid) {
        return Some(false);
    }
    let g = SEEN.lock().ok()?;
    Some(g.as_ref()?.contains(uuid))
}

trait MvAccess: FromIntoEvent {
    const KIND: MvEvent;
    /// `None` skips the event (throttled).
    fn ctx(d: &EventData<Self>) -> Option<EventCtx>;
    fn cancel(_d: &mut EventData<Self>) {}
}

impl MvAccess for PlayerCommandSendEvent {
    const KIND: MvEvent = MvEvent::Command;
    fn ctx(d: &EventData<Self>) -> Option<EventCtx> {
        let mut c = EventCtx::player(player_ref(&d.player));
        c.ev = mv::command_vals(&d.command);
        Some(c)
    }
    fn cancel(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl MvAccess for PlayerMoveEvent {
    const KIND: MvEvent = MvEvent::Move;
    fn ctx(d: &EventData<Self>) -> Option<EventCtx> {
        if !mv::moved_enough(d.from_position, d.to_position) {
            return None;
        }
        let mut c = EventCtx::player(player_ref(&d.player));
        c.ev = mv::loc_vals(
            d.to_position,
            d.from_position,
            Some(world_name(&d.player.get_world())),
        );
        Some(c)
    }
    fn cancel(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl MvAccess for PlayerTeleportEvent {
    const KIND: MvEvent = MvEvent::Teleport;
    fn ctx(d: &EventData<Self>) -> Option<EventCtx> {
        let mut c = EventCtx::player(player_ref(&d.player));
        c.ev = mv::loc_vals(
            d.to_position,
            d.from_position,
            Some(world_name(&d.player.get_world())),
        );
        Some(c)
    }
    fn cancel(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl MvAccess for PlayerSwapHandsEvent {
    const KIND: MvEvent = MvEvent::SwapHands;
    fn ctx(d: &EventData<Self>) -> Option<EventCtx> {
        Some(EventCtx::player(player_ref(&d.player)))
    }
    fn cancel(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl MvAccess for PlayerChangedWorldEvent {
    const KIND: MvEvent = MvEvent::WorldChange;
    fn ctx(d: &EventData<Self>) -> Option<EventCtx> {
        let mut c = EventCtx::player(player_ref(&d.player));
        c.ev = mv::world_vals(world_name(&d.to_world), world_name(&d.from_world));
        Some(c)
    }
}

struct OnMv<E>(PhantomData<fn() -> E>);

impl<E: MvAccess> EventHandler<E> for OnMv<E> {
    fn handle(&self, server: Server, mut data: EventData<E>) -> EventData<E> {
        let kind = EventKind::Mv(E::KIND);
        let cancelled = with_state(&server, None, |st, host| {
            if !st.engine.table().has_event(kind) {
                return false;
            }
            let Some(ctx) = E::ctx(&data) else { return false };
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

/// Fires `on first join` the first time a uuid joins, then remembers it.
struct OnFirstJoin;

impl EventHandler<PlayerJoinEvent> for OnFirstJoin {
    fn handle(&self, server: Server, data: EventData<PlayerJoinEvent>) -> EventData<PlayerJoinEvent> {
        let who = player_ref(&data.player);
        with_state(&server, None, |st, host| {
            let first = {
                let Ok(mut g) = SEEN.lock() else { return };
                let seen = g.get_or_insert_with(HashSet::new);
                let first = seen.insert(who.uuid.clone());
                if let Ok(mut fresh) = FRESH.lock() {
                    fresh.retain(|u| *u != who.uuid);
                    if first {
                        fresh.push(who.uuid.clone());
                    }
                }
                if first {
                    // Written with the next transaction (the next tick).
                    let now_ms = std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .map_or(0, |d| d.as_millis() as u64);
                    crate::store::with(|s| s.queue_seen(&who.uuid, now_ms));
                }
                first
            };
            let kind = EventKind::Mv(MvEvent::FirstJoin);
            if first && st.engine.table().has_event(kind) {
                let mut th = TickHost {
                    inner: host,
                    hs: &mut st.hs,
                };
                st.engine.fire(kind, EventCtx::player(who), &mut th);
            }
        });
        data
    }
}

fn reg<E: MvAccess + Send + Sync + 'static>(ctx: &Context) {
    if let Err(e) = ctx.register_event_handler::<E, _>(OnMv::<E>(PhantomData), EventPriority::Normal, true) {
        log(
            ApiLevel::Error,
            &format!("could not listen to {} events: {e}", E::KIND.name()),
        );
    }
}

/// Listen to the events. `on move` fires for every movement packet, so the
/// server only calls into the plugin when a script uses it at startup
/// (adding the first `on move` script needs a restart).
pub fn register_all(ctx: &Context, move_used: bool) {
    reg::<PlayerCommandSendEvent>(ctx);
    reg::<PlayerTeleportEvent>(ctx);
    reg::<PlayerSwapHandsEvent>(ctx);
    reg::<PlayerChangedWorldEvent>(ctx);
    if move_used {
        reg::<PlayerMoveEvent>(ctx);
    }
    if let Err(e) = ctx.register_event_handler::<PlayerJoinEvent, _>(OnFirstJoin, EventPriority::Normal, true)
    {
        log(
            ApiLevel::Error,
            &format!("could not listen to first join events: {e}"),
        );
    }
}

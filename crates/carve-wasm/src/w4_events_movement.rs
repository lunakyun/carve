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

use super::{TickHost, player_ref, with_state};

/// Players that have joined before, as uuids (file `first_join.txt`).
static SEEN: Mutex<Option<HashSet<String>>> = Mutex::new(None);
const SEEN_FILE: &str = "first_join.txt";

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
            Some(d.player.get_world().get_name()),
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
            Some(d.player.get_world().get_name()),
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
        c.ev = mv::world_vals(d.to_world.get_name(), d.from_world.get_name());
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
                let seen = g.get_or_insert_with(|| {
                    std::fs::read_to_string(format!("{}/{SEEN_FILE}", st.data_dir))
                        .map(|s| s.lines().map(str::to_string).collect())
                        .unwrap_or_default()
                });
                let first = seen.insert(who.uuid.clone());
                if first {
                    let mut all: Vec<&String> = seen.iter().collect();
                    all.sort();
                    let text = all.iter().map(|u| u.as_str()).collect::<Vec<_>>().join("\n");
                    st.save_to(SEEN_FILE, &text);
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

//! Glue for the events of `carve_core::w2_events_4`: server list ping, sign
//! change, smelting, vehicles, worlds, weather, targeting, taming and so on.
//! Each event builds the list of values the script can read (`event-...`),
//! and may take changed values back (`set event-motd to ...`).

use std::marker::PhantomData;

use carve_core::w2_events_4::{self as core_ev, ExtEvent};
use carve_core::{EventKind, PlayerRef, Value};
use pumpkin_plugin_api::events::*;
use pumpkin_plugin_api::logging::LogLevel as ApiLevel;
use pumpkin_plugin_api::{Context, Server};

use super::{State, TickHost, WasmHost, plog, player_ref, text, with_state};

/// What an event hands to scripts: the player (if any) and the values.
type Built = (Option<PlayerRef>, Vec<Value>);

trait ExtAccess: FromIntoEvent {
    /// The event, and a second one that also runs (weather: `... to rain`).
    fn kinds(d: &EventData<Self>) -> (ExtEvent, Option<ExtEvent>);
    fn build(st: &mut State, host: &mut WasmHost<'_>, d: &EventData<Self>) -> Built;
    /// Take values changed by `set event-...` back into the event.
    fn apply(_d: &mut EventData<Self>, _out: &[(u8, Value)]) {}
    fn cancel(_d: &mut EventData<Self>) {}
}

struct OnExt<E>(PhantomData<fn() -> E>);

impl<E: ExtAccess> EventHandler<E> for OnExt<E> {
    fn handle(&self, server: Server, mut data: EventData<E>) -> EventData<E> {
        let (k1, k2) = E::kinds(&data);
        let res = with_state(&server, None, |st, host| {
            let wanted = |st: &State, k: ExtEvent| st.engine.table().has_event(EventKind::Ext4(k));
            let a = wanted(st, k1);
            let b = k2.is_some_and(|k| wanted(st, k));
            if !a && !b {
                return None;
            }
            let (player, vals) = E::build(st, host, &data);
            let mut cancelled = false;
            let mut outs = Vec::new();
            for (k, on) in [(Some(k1), a), (k2, b)] {
                let (Some(k), true) = (k, on) else { continue };
                let c = core_ev::ctx(player.clone(), vals.clone());
                let mut th = TickHost {
                    inner: host,
                    hs: &mut st.hs,
                };
                let o = st.engine.fire(EventKind::Ext4(k), c, &mut th);
                cancelled |= o.cancelled;
                outs.extend(o.set);
            }
            Some((cancelled, outs))
        });
        if let Some(Some((cancelled, outs))) = res {
            if cancelled {
                E::cancel(&mut data);
            }
            if !outs.is_empty() {
                E::apply(&mut data, &outs);
            }
        }
        data
    }
}

// ---------- value helpers ----------

fn num(n: impl Into<f64>) -> Value {
    Value::Num(n.into())
}

fn block(x: i32, y: i32, z: i32, world: Option<String>) -> Value {
    Value::Location {
        x: f64::from(x),
        y: f64::from(y),
        z: f64::from(z),
        world,
    }
}

fn pos(p: (f64, f64, f64), world: Option<String>) -> Value {
    Value::Location {
        x: p.0,
        y: p.1,
        z: p.2,
        world,
    }
}

fn item(id: &str) -> Value {
    Value::Item {
        id: id.strip_prefix("minecraft:").unwrap_or(id).to_string(),
        count: 1,
    }
}

/// An entity: the player when it is one, else its numeric entity id.
fn ent(st: &mut State, host: &mut WasmHost<'_>, id: i32) -> Value {
    match st.player_by_entity(host, id) {
        Some(p) => Value::Player(p),
        None => Value::Num(f64::from(id)),
    }
}

fn ent_opt(st: &mut State, host: &mut WasmHost<'_>, id: Option<i32>) -> Value {
    id.map_or(Value::None, |i| ent(st, host, i))
}

fn out_text(out: &[(u8, Value)], i: u8) -> Option<String> {
    out.iter().rev().find(|(k, _)| *k == i).map(|(_, v)| match v {
        Value::Text(t) => t.clone(),
        Value::Num(n) if n.fract() == 0.0 => format!("{}", *n as i64),
        Value::Num(n) => n.to_string(),
        Value::Bool(b) => b.to_string(),
        other => other.type_name().to_string(),
    })
}

fn out_num(out: &[(u8, Value)], i: u8) -> Option<f64> {
    out.iter().rev().find(|(k, _)| *k == i).and_then(|(_, v)| match v {
        Value::Num(n) => Some(*n),
        _ => None,
    })
}

fn out_bool(out: &[(u8, Value)], i: u8) -> Option<bool> {
    out.iter().rev().find(|(k, _)| *k == i).and_then(|(_, v)| match v {
        Value::Bool(b) => Some(*b),
        _ => None,
    })
}

// ---------- the events ----------

macro_rules! ext {
    (
        $ty:ty, $kind:expr,
        cancel = $cancel:expr,
        build = |$st:ident, $host:ident, $d:ident| $build:expr
        $(, apply = |$ad:ident, $out:ident| $apply:block)?
    ) => {
        impl ExtAccess for $ty {
            fn kinds(_d: &EventData<Self>) -> (ExtEvent, Option<ExtEvent>) {
                ($kind, None)
            }
            fn build($st: &mut State, $host: &mut WasmHost<'_>, $d: &EventData<Self>) -> Built {
                let _ = (&$st, &$host);
                $build
            }
            fn cancel(d: &mut EventData<Self>) {
                let f: fn(&mut EventData<Self>) = $cancel;
                f(d);
            }
            $(fn apply($ad: &mut EventData<Self>, $out: &[(u8, Value)]) $apply)?
        }
    };
}

const NO_CANCEL: bool = false;
macro_rules! can {
    () => {
        |d| d.cancelled = true
    };
    (no) => {
        |_d| {
            let _ = NO_CANCEL;
        }
    };
}

ext!(ServerListPingEvent, ExtEvent::ServerListPing, cancel = can!(no),
    build = |_st, _host, d| (
        None,
        vec![
            Value::Text(d.motd.get_text()),
            num(d.max_players),
            num(d.num_players),
            Value::Text(d.hostname.clone()),
            Value::Text(format!("{}:{}", d.address.host, d.address.port)),
        ]
    ),
    apply = |d, out| {
        if let Some(t) = out_text(out, 0) {
            d.motd = text(&t);
        }
        if let Some(n) = out_num(out, 1) {
            d.max_players = n.max(0.0) as u32;
        }
        if let Some(n) = out_num(out, 2) {
            d.num_players = n.max(0.0) as u32;
        }
    }
);

ext!(SheepRegrowWoolEvent, ExtEvent::SheepRegrowWool, cancel = can!(),
    build = |st, host, d| (None, vec![ent(st, host, d.entity_id)])
);

ext!(EntityShootBowEvent, ExtEvent::Shoot, cancel = can!(),
    build = |st, host, d| {
        let e = ent(st, host, d.entity_id);
        let p = match &e {
            Value::Player(p) => Some(p.clone()),
            _ => None,
        };
        (p, vec![e, Value::Text(d.weapon_name.clone()), num(d.force)])
    }
);

ext!(SignChangeEvent, ExtEvent::SignChange, cancel = can!(),
    build = |_st, _host, d| {
        let line = |i: usize| Value::Text(d.lines.get(i).cloned().unwrap_or_default());
        (
            Some(player_ref(&d.player)),
            vec![
                block(d.block_pos.x, d.block_pos.y, d.block_pos.z, Some(d.player.get_world().get_name())),
                line(0),
                line(1),
                line(2),
                line(3),
            ],
        )
    },
    apply = |d, out| {
        while d.lines.len() < 4 {
            d.lines.push(String::new());
        }
        for i in 0..4u8 {
            if let Some(t) = out_text(out, i + 1) {
                d.lines[usize::from(i)] = t;
            }
        }
    }
);

ext!(SlimeSplitEvent, ExtEvent::SlimeSplit, cancel = can!(),
    build = |st, host, d| (None, vec![ent(st, host, d.entity_id), num(d.count)]),
    apply = |d, out| {
        if let Some(n) = out_num(out, 1) {
            d.count = n as i32;
        }
    }
);

ext!(FurnaceSmeltEvent, ExtEvent::Smelt, cancel = can!(),
    build = |_st, _host, d| (
        None,
        vec![
            block(d.block_pos.x, d.block_pos.y, d.block_pos.z, None),
            item(&d.source_item),
            item(&d.result_item),
        ]
    )
);

ext!(FurnaceStartSmeltEvent, ExtEvent::StartSmelt, cancel = can!(),
    build = |_st, _host, d| (
        None,
        vec![
            block(d.block_pos.x, d.block_pos.y, d.block_pos.z, None),
            item(&d.source_item),
            num(d.cooking_time),
        ]
    ),
    apply = |d, out| {
        if let Some(n) = out_num(out, 2) {
            d.cooking_time = n.max(0.0) as u32;
        }
    }
);

ext!(SpawnChangeEvent, ExtEvent::SpawnChange, cancel = can!(no),
    build = |_st, _host, d| {
        let w = d.target_world.get_name();
        (
            None,
            vec![
                Value::World(w.clone()),
                block(d.previous_position.x, d.previous_position.y, d.previous_position.z, Some(w.clone())),
                block(d.new_position.x, d.new_position.y, d.new_position.z, Some(w)),
            ],
        )
    }
);

ext!(SpongeAbsorbEvent, ExtEvent::SpongeAbsorb, cancel = can!(),
    build = |_st, _host, d| (None, vec![block(d.block_pos.x, d.block_pos.y, d.block_pos.z, None)])
);

ext!(BlockSpreadEvent, ExtEvent::Spread, cancel = can!(),
    build = |_st, _host, d| {
        let w = d.target_world.get_name();
        (
            None,
            vec![
                block(d.target_pos.x, d.target_pos.y, d.target_pos.z, Some(w.clone())),
                block(d.source_pos.x, d.source_pos.y, d.source_pos.z, Some(w.clone())),
                Value::World(w),
            ],
        )
    }
);

ext!(EntityToggleSwimEvent, ExtEvent::SwimToggle, cancel = can!(),
    build = |st, host, d| {
        let e = ent(st, host, d.entity_id);
        let p = match &e {
            Value::Player(p) => Some(p.clone()),
            _ => None,
        };
        (p, vec![e, Value::Bool(d.is_swimming)])
    }
);

ext!(EntityTameEvent, ExtEvent::Tame, cancel = can!(),
    build = |st, host, d| {
        let o = player_ref(&d.owner);
        (
            Some(o.clone()),
            vec![ent(st, host, d.entity_id), Value::Player(o)],
        )
    }
);

ext!(EntityTargetEvent, ExtEvent::Target, cancel = can!(),
    build = |st, host, d| (
        None,
        vec![ent(st, host, d.entity_id), ent_opt(st, host, d.target_id)]
    )
);

ext!(PlayerEggThrowEvent, ExtEvent::EggThrow, cancel = can!(),
    build = |_st, _host, d| (
        Some(player_ref(&d.player)),
        vec![
            Value::Bool(d.hatching),
            num(d.num_hatches),
            Value::Text(d.hatching_type.clone()),
        ]
    ),
    apply = |d, out| {
        if let Some(b) = out_bool(out, 0) {
            d.hatching = b;
        }
        if let Some(n) = out_num(out, 1) {
            d.num_hatches = n.clamp(0.0, 255.0) as u8;
        }
    }
);

ext!(PlayerItemHeldEvent, ExtEvent::ToolChange, cancel = can!(),
    build = |_st, _host, d| (
        Some(player_ref(&d.player)),
        vec![num(d.previous_slot), num(d.new_slot)]
    )
);

ext!(VaultDisplayItemEvent, ExtEvent::VaultDisplayItem, cancel = can!(),
    build = |_st, _host, d| {
        let w = d.target_world.get_name();
        (
            None,
            vec![
                block(d.block_pos.x, d.block_pos.y, d.block_pos.z, Some(w.clone())),
                Value::World(w),
                item(&d.item.get_registry_key()),
            ],
        )
    }
);

ext!(VehicleEntityCollisionEvent, ExtEvent::VehicleCollision, cancel = can!(),
    build = |st, host, d| (
        None,
        vec![
            ent(st, host, d.vehicle_id),
            ent(st, host, d.collided_entity_id),
            Value::None,
        ]
    )
);

ext!(VehicleBlockCollisionEvent, ExtEvent::VehicleCollision, cancel = can!(),
    build = |st, host, d| (
        None,
        vec![
            ent(st, host, d.vehicle_id),
            Value::None,
            block(d.block_pos.x, d.block_pos.y, d.block_pos.z, None),
        ]
    )
);

ext!(VehicleCreateEvent, ExtEvent::VehicleCreate, cancel = can!(),
    build = |st, host, d| (None, vec![ent(st, host, d.vehicle_id)])
);

ext!(VehicleDamageEvent, ExtEvent::VehicleDamage, cancel = can!(),
    build = |st, host, d| {
        let a = ent_opt(st, host, d.attacker_id);
        let p = match &a {
            Value::Player(p) => Some(p.clone()),
            _ => None,
        };
        (p, vec![ent(st, host, d.vehicle_id), num(d.damage), a])
    }
);

ext!(VehicleDestroyEvent, ExtEvent::VehicleDestroy, cancel = can!(),
    build = |st, host, d| {
        let a = ent_opt(st, host, d.attacker_id);
        let p = match &a {
            Value::Player(p) => Some(p.clone()),
            _ => None,
        };
        (p, vec![ent(st, host, d.vehicle_id), a])
    }
);

ext!(VehicleEnterEvent, ExtEvent::VehicleEnter, cancel = can!(),
    build = |st, host, d| {
        let e = ent(st, host, d.entered_id);
        let p = match &e {
            Value::Player(p) => Some(p.clone()),
            _ => None,
        };
        (p, vec![ent(st, host, d.vehicle_id), e])
    }
);

ext!(VehicleExitEvent, ExtEvent::VehicleExit, cancel = can!(),
    build = |st, host, d| {
        let e = ent(st, host, d.exited_id);
        let p = match &e {
            Value::Player(p) => Some(p.clone()),
            _ => None,
        };
        (p, vec![ent(st, host, d.vehicle_id), e])
    }
);

ext!(VehicleMoveEvent, ExtEvent::VehicleMove, cancel = can!(),
    build = |st, host, d| (
        None,
        vec![
            ent(st, host, d.vehicle_id),
            pos(d.from_position, None),
            pos(d.to_position, None),
        ]
    )
);

ext!(VillagerCareerChangeEvent, ExtEvent::VillagerCareerChange, cancel = can!(),
    build = |st, host, d| (
        None,
        vec![
            ent(st, host, d.entity_id),
            Value::Text(d.profession.clone()),
            Value::Text(d.reason.clone()),
        ]
    )
);

impl ExtAccess for WeatherChangeEvent {
    fn kinds(d: &EventData<Self>) -> (ExtEvent, Option<ExtEvent>) {
        (
            ExtEvent::WeatherChange,
            Some(if d.to_weather_state {
                ExtEvent::WeatherToRain
            } else {
                ExtEvent::WeatherToClear
            }),
        )
    }
    fn build(_st: &mut State, _host: &mut WasmHost<'_>, d: &EventData<Self>) -> Built {
        let w = if d.to_weather_state { "rain" } else { "clear" };
        (
            None,
            vec![Value::World(d.target_world.get_name()), Value::Text(w.into())],
        )
    }
    fn cancel(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

ext!(WorldInitEvent, ExtEvent::WorldInit, cancel = can!(no),
    build = |_st, _host, d| (None, vec![Value::World(d.target_world.get_name())])
);

ext!(WorldLoadEvent, ExtEvent::WorldLoad, cancel = can!(no),
    build = |_st, _host, d| (None, vec![Value::World(d.target_world.get_name())])
);

ext!(WorldUnloadEvent, ExtEvent::WorldUnload, cancel = can!(),
    build = |_st, _host, d| (None, vec![Value::World(d.target_world.get_name())])
);

ext!(WorldSaveEvent, ExtEvent::WorldSave, cancel = can!(),
    build = |_st, _host, d| (None, vec![Value::World(d.world_name.clone())])
);

ext!(EntityBreakDoorEvent, ExtEvent::ZombieBreakDoor, cancel = can!(),
    build = |st, host, d| (
        None,
        vec![
            ent(st, host, d.entity_id),
            block(d.block_pos.x, d.block_pos.y, d.block_pos.z, None),
        ]
    )
);

// ---------- registration and the clock ----------

fn listen<E: ExtAccess + Send + Sync + 'static>(ctx: &Context) {
    let h = OnExt::<E>(PhantomData);
    if let Err(e) = ctx.register_event_handler::<E, _>(h, EventPriority::Normal, true) {
        plog(ApiLevel::Error, &format!("could not listen to an event: {e}"));
    }
}

pub(crate) fn register_all(ctx: &Context) {
    listen::<ServerListPingEvent>(ctx);
    listen::<SheepRegrowWoolEvent>(ctx);
    listen::<EntityShootBowEvent>(ctx);
    listen::<SignChangeEvent>(ctx);
    listen::<SlimeSplitEvent>(ctx);
    listen::<FurnaceSmeltEvent>(ctx);
    listen::<FurnaceStartSmeltEvent>(ctx);
    listen::<SpawnChangeEvent>(ctx);
    listen::<SpongeAbsorbEvent>(ctx);
    listen::<BlockSpreadEvent>(ctx);
    listen::<EntityToggleSwimEvent>(ctx);
    listen::<EntityTameEvent>(ctx);
    listen::<EntityTargetEvent>(ctx);
    listen::<PlayerEggThrowEvent>(ctx);
    listen::<PlayerItemHeldEvent>(ctx);
    listen::<VaultDisplayItemEvent>(ctx);
    listen::<VehicleEntityCollisionEvent>(ctx);
    listen::<VehicleBlockCollisionEvent>(ctx);
    listen::<VehicleCreateEvent>(ctx);
    listen::<VehicleDamageEvent>(ctx);
    listen::<VehicleDestroyEvent>(ctx);
    listen::<VehicleEnterEvent>(ctx);
    listen::<VehicleExitEvent>(ctx);
    listen::<VehicleMoveEvent>(ctx);
    listen::<VillagerCareerChangeEvent>(ctx);
    listen::<WeatherChangeEvent>(ctx);
    listen::<WorldInitEvent>(ctx);
    listen::<WorldLoadEvent>(ctx);
    listen::<WorldUnloadEvent>(ctx);
    listen::<WorldSaveEvent>(ctx);
    listen::<EntityBreakDoorEvent>(ctx);
}

/// Called every tick: runs `at HH:MM in real time` triggers on minute change.
pub(crate) fn poll_clock(st: &mut State, host: &mut WasmHost<'_>) {
    let mut th = TickHost {
        inner: host,
        hs: &mut st.hs,
    };
    core_ev::poll_system_time(&mut st.engine, &mut st.last_minute, &mut th);
}

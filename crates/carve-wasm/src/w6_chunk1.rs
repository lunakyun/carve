//! Host side of `carve_core::w6_chunk1`: events built from other server
//! events (jump from player input, chunk enter and step on from movement,
//! pressure plates from redstone, shots from projectile launches) and events
//! found by polling (world border, armor, spectating).
//!
//! All of them are `ExtEvent`s of `carve_core::w2_events_4`. The listeners
//! are only attached when a script uses the event at startup, because most of
//! these server events fire all the time.

use std::cell::RefCell;
use std::collections::HashMap;
use std::marker::PhantomData;

use carve_core::w2_events_4::{self as core_ev, ExtEvent};
use carve_core::w6_chunk1 as logic;
use carve_core::{EventKind, PlayerRef, Value};
use pumpkin_plugin_api::events::*;
use pumpkin_plugin_api::logging::LogLevel as ApiLevel;
use pumpkin_plugin_api::{Context, Server};

use super::{State, TickHost, WasmHost, player_ref, plog, with_state, world_name};

type Vals = Vec<Value>;

fn used(st: &State, k: ExtEvent) -> bool {
    st.engine.table().has_event(EventKind::Ext4(k))
}

/// Run the triggers of one event. Returns whether a script cancelled it.
fn fire(st: &mut State, host: &mut WasmHost<'_>, k: ExtEvent, player: Option<PlayerRef>, vals: Vals) -> bool {
    let kind = EventKind::Ext4(k);
    if !st.engine.table().has_event(kind) {
        return false;
    }
    let c = core_ev::ctx(player, vals);
    let mut th = TickHost {
        inner: host,
        hs: &mut st.hs,
    };
    st.engine.fire(kind, c, &mut th).cancelled
}

/// An entity as a script value: the player, the entity, or its id.
fn ent(st: &mut State, host: &mut WasmHost<'_>, id: i32) -> Value {
    if let Some(p) = st.player_by_entity(host, id) {
        return Value::Player(p);
    }
    host.entity_by_id(id)
        .map_or(Value::Num(f64::from(id)), Value::Entity)
}

fn at(p: (f64, f64, f64), world: Option<String>) -> Value {
    Value::Location {
        x: p.0,
        y: p.1,
        z: p.2,
        world,
    }
}

fn item_id(id: &str) -> Value {
    Value::Item {
        id: id.strip_prefix("minecraft:").unwrap_or(id).to_string(),
        count: 1,
        meta: None,
    }
}

// ---------- events that are one server event each ----------

trait W6: FromIntoEvent {
    const KIND: ExtEvent;
    /// The player and values, or `None` to skip this event.
    fn build(
        st: &mut State,
        host: &mut WasmHost<'_>,
        d: &EventData<Self>,
    ) -> Option<(Option<PlayerRef>, Vals)>;
    fn cancel(_d: &mut EventData<Self>) {}
}

struct On<E>(PhantomData<fn() -> E>);

impl<E: W6> EventHandler<E> for On<E> {
    fn handle(&self, server: Server, mut data: EventData<E>) -> EventData<E> {
        let cancelled = with_state(&server, None, |st, host| {
            if !used(st, E::KIND) {
                return false;
            }
            let Some((p, v)) = E::build(st, host, &data) else {
                return false;
            };
            fire(st, host, E::KIND, p, v)
        });
        if cancelled == Some(true) {
            E::cancel(&mut data);
        }
        data
    }
}

impl W6 for PlayerPickupArrowEvent {
    const KIND: ExtEvent = ExtEvent::PickupArrow;
    fn build(
        st: &mut State,
        host: &mut WasmHost<'_>,
        d: &EventData<Self>,
    ) -> Option<(Option<PlayerRef>, Vals)> {
        Some((Some(player_ref(&d.player)), vec![ent(st, host, d.arrow_id)]))
    }
    fn cancel(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl W6 for PlayerRiptideEvent {
    const KIND: ExtEvent = ExtEvent::Riptide;
    fn build(
        _st: &mut State,
        _host: &mut WasmHost<'_>,
        d: &EventData<Self>,
    ) -> Option<(Option<PlayerRef>, Vals)> {
        Some((Some(player_ref(&d.player)), vec![item_id(&d.item_name)]))
    }
    fn cancel(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

/// Pumpkin 0.2.2 sends the numeric registry id of the entity type in this
/// event (`entities.json`: `experience_orb` is 50), not its name.
const EXPERIENCE_ORB_ID: &str = "50";

impl W6 for EntitySpawnEvent {
    const KIND: ExtEvent = ExtEvent::ExperienceSpawn;
    fn build(
        st: &mut State,
        host: &mut WasmHost<'_>,
        d: &EventData<Self>,
    ) -> Option<(Option<PlayerRef>, Vals)> {
        if d.entity_type != EXPERIENCE_ORB_ID && !d.entity_type.ends_with("experience_orb") {
            return None;
        }
        let w = world_name(&d.target_world);
        Some((None, vec![ent(st, host, d.entity_id), at(d.position, Some(w))]))
    }
    fn cancel(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl W6 for ProjectileHitEvent {
    const KIND: ExtEvent = ExtEvent::ProjectileCollide;
    fn build(
        st: &mut State,
        host: &mut WasmHost<'_>,
        d: &EventData<Self>,
    ) -> Option<(Option<PlayerRef>, Vals)> {
        let hit = d.hit_entity_id?;
        let proj = ent(st, host, d.entity_id);
        let target = ent(st, host, hit);
        Some((None, vec![proj, target, at(d.hit_position, None)]))
    }
    fn cancel(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl W6 for BlockRedstoneEvent {
    const KIND: ExtEvent = ExtEvent::PressurePlate;
    fn build(
        _st: &mut State,
        host: &mut WasmHost<'_>,
        d: &EventData<Self>,
    ) -> Option<(Option<PlayerRef>, Vals)> {
        if !logic::plate_pressed(d.old_current, d.new_current) {
            return None;
        }
        let w = host.server.get_world_by_name(&world_name(&d.target_world))?;
        if !logic::is_pressure_plate(&w.get_block(d.block_pos).name) {
            return None;
        }
        let loc = Value::Location {
            x: f64::from(d.block_pos.x),
            y: f64::from(d.block_pos.y),
            z: f64::from(d.block_pos.z),
            world: Some(world_name(&d.target_world)),
        };
        Some((None, vec![loc, Value::Num(f64::from(d.new_current))]))
    }
    fn cancel(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

/// Players shooting arrows: Pumpkin fires `entity-shoot-bow` for mobs only,
/// but every arrow fires a launch event with its shooter.
impl W6 for ProjectileLaunchEvent {
    const KIND: ExtEvent = ExtEvent::Shoot;
    fn build(
        st: &mut State,
        host: &mut WasmHost<'_>,
        d: &EventData<Self>,
    ) -> Option<(Option<PlayerRef>, Vals)> {
        let shooter = d.shooter_id?;
        let p = st.player_by_entity(host, shooter)?;
        let weapon = host
            .get(&p)
            .and_then(|pl| pl.get_item_in_hand(pumpkin_plugin_api::common::Hand::Right))
            .map_or(String::new(), |s| s.get_registry_key());
        Some((
            Some(p.clone()),
            vec![
                Value::Player(p),
                Value::Text(weapon),
                Value::None,
                ent(st, host, d.entity_id),
            ],
        ))
    }
    fn cancel(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl W6 for PlayerSpawnChangeEvent {
    const KIND: ExtEvent = ExtEvent::SpawnChange;
    fn build(
        _st: &mut State,
        _host: &mut WasmHost<'_>,
        d: &EventData<Self>,
    ) -> Option<(Option<PlayerRef>, Vals)> {
        let w = world_name(&d.player.get_world());
        let new = d.new_spawn.as_ref().map_or(Value::None, |p| Value::Location {
            x: f64::from(p.x),
            y: f64::from(p.y),
            z: f64::from(p.z),
            world: Some(w.clone()),
        });
        Some((
            Some(player_ref(&d.player)),
            vec![Value::World(w), Value::None, new],
        ))
    }
}

// ---------- player movement: chunk enter and step on ----------

struct OnMove;

impl EventHandler<PlayerMoveEvent> for OnMove {
    fn handle(&self, server: Server, mut data: EventData<PlayerMoveEvent>) -> EventData<PlayerMoveEvent> {
        let cancelled = with_state(&server, None, |st, host| {
            let chunk = used(st, ExtEvent::ChunkEnter);
            let step = used(st, ExtEvent::StepOn);
            if !chunk && !step {
                return false;
            }
            let (from, to) = (data.from_position, data.to_position);
            let who = player_ref(&data.player);
            let mut stop = false;
            if chunk && logic::chunk_changed((from.0, from.2), (to.0, to.2)) {
                let vals = vec![
                    Value::Num(f64::from(logic::chunk_of(to.0))),
                    Value::Num(f64::from(logic::chunk_of(to.2))),
                    Value::Num(f64::from(logic::chunk_of(from.0))),
                    Value::Num(f64::from(logic::chunk_of(from.2))),
                ];
                stop |= fire(st, host, ExtEvent::ChunkEnter, Some(who.clone()), vals);
            }
            if step {
                let below = |p: (f64, f64, f64)| {
                    (
                        p.0.floor() as i32,
                        (p.1 - 0.001).floor() as i32,
                        p.2.floor() as i32,
                    )
                };
                let (a, b) = (below(from), below(to));
                if a != b {
                    let w = data.player.get_world();
                    let name = w
                        .get_block(pumpkin_plugin_api::common::BlockPos {
                            x: b.0,
                            y: b.1,
                            z: b.2,
                        })
                        .name;
                    let id = name.strip_prefix("minecraft:").unwrap_or(&name).to_string();
                    let loc = Value::Location {
                        x: f64::from(b.0),
                        y: f64::from(b.1),
                        z: f64::from(b.2),
                        world: Some(world_name(&w)),
                    };
                    stop |= fire(st, host, ExtEvent::StepOn, Some(who), vec![Value::Text(id), loc]);
                }
            }
            stop
        });
        if cancelled == Some(true) {
            data.cancelled = true;
        }
        data
    }
}

// ---------- jump: rising edge of the jump bit in player input ----------

thread_local! {
    static LAST_INPUT: RefCell<HashMap<String, u32>> = RefCell::new(HashMap::new());
}

struct OnInput;

impl EventHandler<PlayerInputEvent> for OnInput {
    fn handle(&self, server: Server, data: EventData<PlayerInputEvent>) -> EventData<PlayerInputEvent> {
        let Ok(now) = u32::from_str_radix(&data.input, 2) else {
            return data;
        };
        let who = player_ref(&data.player);
        let before = LAST_INPUT.with(|m| m.borrow_mut().insert(who.uuid.clone(), now).unwrap_or(0));
        if logic::jump_started(before, now) {
            with_state(&server, None, |st, host| {
                fire(st, host, ExtEvent::Jump, Some(who), Vec::new());
            });
        }
        data
    }
}

// ---------- polling: world border, armor and spectating ----------

#[derive(Default)]
struct Poll {
    /// World name -> (target diameter, center x, center z).
    border: HashMap<String, (f64, f64, f64)>,
    armor: HashMap<String, [Option<(String, u32)>; 4]>,
    camera: HashMap<String, u32>,
}

thread_local! {
    static POLL: RefCell<Poll> = RefCell::new(Poll::default());
}

const ARMOR_SLOTS: [(&str, u32); 4] = [
    ("boots", 36),
    ("leggings", 37),
    ("chestplate", 38),
    ("helmet", 39),
];

/// Called every tick. Cheap when no script uses these events.
pub(crate) fn tick(server: &Server, st: &mut State, host: &mut WasmHost<'_>, now: u64) {
    let border = used(st, ExtEvent::BorderBounds) || used(st, ExtEvent::BorderCenter);
    if border {
        poll_border(server, st, host);
    }
    if now % 4 == 0 && used(st, ExtEvent::ArmorChange) {
        poll_armor(server, st, host);
    }
    if now % 2 == 0 && used(st, ExtEvent::Spectate) {
        poll_spectate(server, st, host);
    }
}

/// Pumpkin sets no end for a border transition (the client does the lerping
/// and the speed is not stored), so only target and center changes are seen.
fn poll_border(server: &Server, st: &mut State, host: &mut WasmHost<'_>) {
    for w in server.get_all_worlds() {
        let name = world_name(&w);
        let b = w.get_world_border();
        let cur = (b.get_target_diameter(), b.get_center_x(), b.get_center_z());
        let Some(prev) = POLL.with(|p| p.borrow_mut().border.insert(name.clone(), cur)) else {
            continue;
        };
        if (prev.0 - cur.0).abs() > f64::EPSILON {
            let v = vec![Value::World(name.clone()), Value::Num(prev.0), Value::Num(cur.0)];
            fire(st, host, ExtEvent::BorderBounds, None, v);
        }
        if (prev.1 - cur.1).abs() > f64::EPSILON || (prev.2 - cur.2).abs() > f64::EPSILON {
            let v = vec![
                Value::World(name),
                Value::Num(prev.1),
                Value::Num(prev.2),
                Value::Num(cur.1),
                Value::Num(cur.2),
            ];
            fire(st, host, ExtEvent::BorderCenter, None, v);
        }
    }
}

fn poll_armor(server: &Server, st: &mut State, host: &mut WasmHost<'_>) {
    for pl in server.get_all_players() {
        let who = player_ref(&pl);
        let pi = pl.get_inventory();
        let stacks = [
            pi.get_boots(),
            pi.get_leggings(),
            pi.get_chestplate(),
            pi.get_helmet(),
        ];
        let snap: [Option<(String, u32)>; 4] =
            std::array::from_fn(|i| stacks[i].as_ref().map(super::inv::pair));
        let Some(prev) = POLL.with(|p| p.borrow_mut().armor.insert(who.uuid.clone(), snap.clone())) else {
            continue;
        };
        for i in 0..4 {
            if prev[i] == snap[i] {
                continue;
            }
            let as_item = |s: &Option<(String, u32)>| {
                s.as_ref().map_or(Value::None, |(id, n)| Value::Item {
                    id: id.clone(),
                    count: *n,
                    meta: None,
                })
            };
            let v = vec![
                Value::Text(ARMOR_SLOTS[i].0.to_string()),
                as_item(&snap[i]),
                as_item(&prev[i]),
            ];
            fire(st, host, ExtEvent::ArmorChange, Some(who.clone()), v);
        }
    }
}

fn poll_spectate(server: &Server, st: &mut State, host: &mut WasmHost<'_>) {
    for pl in server.get_all_players() {
        let who = player_ref(&pl);
        let own = pl.as_entity().get_id();
        let cam = pl.get_camera_entity_id();
        let prev = POLL.with(|p| p.borrow_mut().camera.insert(who.uuid.clone(), cam));
        if cam != own && prev.is_some_and(|p| p != cam) {
            let target = ent(st, host, cam as i32);
            fire(st, host, ExtEvent::Spectate, Some(who), vec![target]);
        }
    }
}

// ---------- unloading ----------

/// Run `on unload` and `on server stop` triggers (the plugin is unloading).
pub(crate) fn fire_unload(server: &Server, stopping: bool) {
    with_state(server, None, |st, host| fire_unload_here(st, host, stopping));
}

/// Same, inside a state that is already open (`/carve reload`).
pub(crate) fn fire_unload_here(st: &mut State, host: &mut WasmHost<'_>, stopping: bool) {
    fire(st, host, ExtEvent::ScriptUnload, None, Vec::new());
    if stopping {
        fire(st, host, ExtEvent::ServerStop, None, Vec::new());
    }
}

// ---------- registration ----------

fn listen<E: W6 + Send + Sync + 'static>(ctx: &Context) {
    if let Err(e) = ctx.register_event_handler::<E, _>(On::<E>(PhantomData), EventPriority::Normal, true) {
        plog(
            ApiLevel::Error,
            &format!("could not listen to {} events: {e}", E::KIND.name()),
        );
    }
}

/// Attach the listeners for the events that loaded scripts use.
pub(crate) fn register_all(ctx: &Context, used: &dyn Fn(ExtEvent) -> bool) {
    if used(ExtEvent::PickupArrow) {
        listen::<PlayerPickupArrowEvent>(ctx);
    }
    if used(ExtEvent::Riptide) {
        listen::<PlayerRiptideEvent>(ctx);
    }
    if used(ExtEvent::ExperienceSpawn) {
        listen::<EntitySpawnEvent>(ctx);
    }
    if used(ExtEvent::ProjectileCollide) {
        listen::<ProjectileHitEvent>(ctx);
    }
    if used(ExtEvent::PressurePlate) {
        listen::<BlockRedstoneEvent>(ctx);
    }
    if used(ExtEvent::Shoot) {
        listen::<ProjectileLaunchEvent>(ctx);
    }
    if used(ExtEvent::SpawnChange) {
        listen::<PlayerSpawnChangeEvent>(ctx);
    }
    if used(ExtEvent::ChunkEnter) || used(ExtEvent::StepOn) {
        let _ = ctx.register_event_handler::<PlayerMoveEvent, _>(OnMove, EventPriority::Normal, true);
    }
    if used(ExtEvent::Jump) {
        let _ = ctx.register_event_handler::<PlayerInputEvent, _>(OnInput, EventPriority::Normal, true);
    }
}

// ---------- values of the older extra events that need a lookup ----------

/// Fill in values that need the server: the player who rang a bell and the
/// entity of a bucket catch or a combustion. Rare events only.
#[inline]
pub(crate) fn ext_fixup(
    st: &mut State,
    host: &mut WasmHost<'_>,
    k: carve_core::w2_events_1::ExtEvent,
    ctx: &mut carve_core::EventCtx,
) {
    use carve_core::w2_events_1::ExtEvent as E1;
    if matches!(k, E1::BellRing | E1::BucketCatch | E1::Combust) {
        fixup_lookup(st, host, k, ctx);
    }
}

#[cold]
fn fixup_lookup(
    st: &mut State,
    host: &mut WasmHost<'_>,
    k: carve_core::w2_events_1::ExtEvent,
    ctx: &mut carve_core::EventCtx,
) {
    use carve_core::w2_events_1::ExtEvent as E1;
    let id_at = |ctx: &carve_core::EventCtx, i: usize| match ctx.ev.get(i) {
        Some(Value::Num(n)) => Some(*n as i32),
        _ => None,
    };
    match k {
        E1::BellRing => {
            if let Some(id) = id_at(ctx, 2)
                && let Some(p) = st.player_by_entity(host, id)
            {
                let ev = std::mem::take(&mut ctx.ev);
                *ctx = carve_core::EventCtx::player(p);
                ctx.ev = ev;
            }
        }
        E1::BucketCatch | E1::Combust => {
            let at = if k == E1::BucketCatch { 1 } else { 0 };
            if let Some(id) = id_at(ctx, at) {
                let e = host.entity_by_id(id).map_or(Value::None, Value::Entity);
                ctx.ev.resize(2, Value::None);
                ctx.ev.push(e);
            }
        }
        _ => {}
    }
}

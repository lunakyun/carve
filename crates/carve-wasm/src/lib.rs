//! Carve as a Pumpkin WASM plugin. All game access goes through
//! [`WasmHost`], which implements the core's `Host` trait.

use std::collections::BTreeSet;
use std::sync::Mutex;

use carve_core::{
    CommandOutcome, Engine, EventCtx, EventKind, Host, LogLevel, PlayerRef, Recipient, ScriptError, Variables,
};
use pumpkin_plugin_api::DamageType;
use pumpkin_plugin_api::command::{
    ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType,
};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::common::Hand;
use pumpkin_plugin_api::events::{
    BlockBreakEvent, BlockPlaceEvent, EntityDamageByEntityEvent, EntityDamageEvent, EventData, EventHandler,
    EventPriority, FromIntoEvent, InteractAction, PlayerChatEvent, PlayerDeathEvent, PlayerInteractEvent,
    PlayerJoinEvent, PlayerLeaveEvent, PlayerRespawnEvent,
};
use pumpkin_plugin_api::logging::{LogLevel as ApiLevel, log};
use pumpkin_plugin_api::permission::{Permission, PermissionDefault, PermissionLevel};
use pumpkin_plugin_api::scheduler::schedule_repeating_task;
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::{
    Context, Player, Plugin, PluginMetadata, Server, permissions, register_plugin, uuid,
};

const SCRIPTS_DIR: &str = "scripts";
const VARS_FILE: &str = "variables.json";
const ADMIN_PERM: &str = "carve:admin";
const COMMAND_PERM: &str = "carve:command";

struct HostState {
    tick: u64,
    /// (due tick, resume token)
    pending: Vec<(u64, u64)>,
}

struct State {
    engine: Engine,
    hs: HostState,
    data_dir: String,
    /// Script commands registered with the server (only possible at load).
    registered: BTreeSet<String>,
    /// entity id -> player uuid, rebuilt at most once per tick on a miss.
    entity_players: Vec<(i32, String)>,
    entity_map_tick: u64,
    /// Victim of the damage event being handled, and whether scripts already ran
    /// for it (the by-entity event that may follow must not run them twice).
    damage_pending: Option<(i32, bool, u64)>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn plog(level: ApiLevel, msg: &str) {
    log(level, &format!("[Carve] {msg}"));
}

fn text(s: &str) -> TextComponent {
    TextComponent::from_legacy_string_with_code(s, '&')
}

/// Runs `f` with the plugin state. Returns None (and logs) when the state is
/// busy, which happens if the server calls back into the plugin while a script
/// is running (re-entrant event). Console commands queued by scripts run after
/// the state is released.
fn with_state<R>(
    server: &Server,
    reply: Option<&CommandSender>,
    f: impl FnOnce(&mut State, &mut WasmHost) -> R,
) -> Option<R> {
    let mut host = WasmHost {
        server,
        reply,
        deferred: Vec::new(),
        players: Vec::new(),
    };
    let result = {
        let Ok(mut guard) = STATE.try_lock() else {
            plog(
                ApiLevel::Warn,
                "skipped a nested event while a script was running",
            );
            return None;
        };
        let state = guard.as_mut()?;
        f(state, &mut host)
    };
    for cmd in std::mem::take(&mut host.deferred) {
        server.execute_command(&cmd, pumpkin_plugin_api::server::CommandSender::Console);
    }
    Some(result)
}

struct WasmHost<'a> {
    server: &'a Server,
    /// The sender of the command being run, for console replies.
    reply: Option<&'a CommandSender>,
    deferred: Vec<String>,
    /// Player handles looked up during this `with_state` call, by UUID.
    /// Every lookup crosses the component boundary, so each UUID is looked
    /// up at most once per call. `None` means offline.
    players: Vec<(String, Option<Player>)>,
}

impl WasmHost<'_> {
    fn get(&mut self, p: &PlayerRef) -> Option<&Player> {
        self.get_by_uuid(&p.uuid)
    }

    fn get_by_uuid(&mut self, uuid_str: &str) -> Option<&Player> {
        let i = match self.players.iter().position(|(u, _)| u == uuid_str) {
            Some(i) => i,
            None => {
                let player = uuid::parse(uuid_str).and_then(|id| self.server.get_player_by_uuid(id));
                self.players.push((uuid_str.to_string(), player));
                self.players.len() - 1
            }
        };
        self.players.get(i).and_then(|(_, p)| p.as_ref())
    }
}

/// Bare permission names live in the plugin's namespace: `vip` -> `carve:vip`.
fn full_node(node: &str) -> String {
    if node.contains(':') {
        node.to_string()
    } else {
        format!("carve:{node}")
    }
}

fn player_ref(p: &Player) -> PlayerRef {
    PlayerRef {
        uuid: uuid::to_string(p.get_id()),
        name: p.get_name(),
    }
}

// Inherent methods; `TickHost` exposes them through the `Host` trait.
impl WasmHost<'_> {
    fn send_message(&mut self, to: &Recipient, msg: &str) {
        match to {
            Recipient::Player(p) => {
                if let Some(pl) = self.get(p) {
                    pl.send_system_message(text(msg), false);
                }
            }
            Recipient::Console => match self.reply {
                Some(sender) => sender.send_message(text(msg)),
                None => plog(ApiLevel::Info, msg),
            },
        }
    }

    fn broadcast(&mut self, msg: &str) {
        for p in self.server.get_all_players() {
            p.send_system_message(text(msg), false);
        }
        plog(ApiLevel::Info, &format!("[broadcast] {msg}"));
    }

    fn send_title(&mut self, p: &PlayerRef, title: &str, subtitle: Option<&str>) {
        if let Some(pl) = self.get(p) {
            if let Some(s) = subtitle {
                pl.show_subtitle(text(s));
            }
            pl.show_title(text(title));
        }
    }

    fn send_action_bar(&mut self, p: &PlayerRef, msg: &str) {
        if let Some(pl) = self.get(p) {
            pl.show_actionbar(text(msg));
        }
    }

    fn heal(&mut self, p: &PlayerRef) {
        if let Some(pl) = self.get(p) {
            pl.set_health(pl.get_max_health());
        }
    }

    fn set_health(&mut self, p: &PlayerRef, health: f64) {
        if let Some(pl) = self.get(p) {
            let max = pl.get_max_health();
            pl.set_health((health as f32).clamp(0.0, max));
        }
    }

    fn kill(&mut self, p: &PlayerRef) {
        if let Some(pl) = self.get(p) {
            pl.kill();
        }
    }

    fn teleport(&mut self, p: &PlayerRef, x: f64, y: f64, z: f64) {
        if let Some(pl) = self.get(p) {
            let world = pl.get_world();
            pl.teleport((x, y, z), None, None, world);
        }
    }

    fn execute_command(&mut self, command: &str) {
        self.deferred.push(command.to_string());
    }

    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        self.get_by_uuid(uuid).map(|p| p.get_name())
    }

    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        let p = self.server.get_player_by_name(name)?;
        let r = player_ref(&p);
        if !self.players.iter().any(|(u, _)| *u == r.uuid) {
            self.players.push((r.uuid.clone(), Some(p)));
        }
        Some(r)
    }

    fn has_permission(&mut self, p: &PlayerRef, node: &str) -> bool {
        let node = full_node(node);
        self.get(p).is_some_and(|pl| pl.has_permission(&node))
    }

    fn online_players(&mut self) -> Vec<PlayerRef> {
        let mut out = Vec::new();
        for p in self.server.get_all_players() {
            let r = player_ref(&p);
            if !self.players.iter().any(|(u, _)| *u == r.uuid) {
                self.players.push((r.uuid.clone(), Some(p)));
            }
            out.push(r);
        }
        out
    }

    fn player_location(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        let pl = self.get(p)?;
        let (x, y, z) = pl.get_position();
        Some((x, y, z, pl.get_world().get_name()))
    }

    fn teleport_to(&mut self, p: &PlayerRef, x: f64, y: f64, z: f64, world: Option<&str>) {
        let target = world.and_then(|w| self.server.get_world_by_name(w));
        if world.is_some() && target.is_none() {
            plog(
                ApiLevel::Warn,
                &format!("teleport: no world named '{}'", world.unwrap_or_default()),
            );
            return;
        }
        if let Some(pl) = self.get(p) {
            let w = target.unwrap_or_else(|| pl.get_world());
            pl.teleport((x, y, z), None, None, w);
        }
    }

    fn held_item(&mut self, p: &PlayerRef) -> Option<(String, u32)> {
        let stack = self.get(p)?.get_item_in_hand(Hand::Right)?;
        let key = stack.get_registry_key();
        let id = key.strip_prefix("minecraft:").unwrap_or(&key).to_string();
        Some((id, u32::from(stack.get_count())))
    }

    fn count_item(&mut self, p: &PlayerRef, id: &str) -> u32 {
        self.get(p)
            .map_or(0, |pl| pl.get_inventory().as_inventory().count_item(id))
    }

    fn health(&mut self, p: &PlayerRef) -> Option<f64> {
        self.get(p).map(|pl| f64::from(pl.get_health()))
    }

    fn log(&mut self, level: LogLevel, msg: &str) {
        let l = match level {
            LogLevel::Info => ApiLevel::Info,
            LogLevel::Warn => ApiLevel::Warn,
            LogLevel::Error => ApiLevel::Error,
        };
        plog(l, msg);
    }
}

/// The `Host` the engine sees: game calls go to `WasmHost`, waits and the
/// clock use the plugin's own tick counter.
struct TickHost<'a, 'b> {
    inner: &'a mut WasmHost<'b>,
    hs: &'a mut HostState,
}

macro_rules! delegate {
    ($($name:ident($($a:ident: $t:ty),*) $(-> $r:ty)?;)*) => {
        $(fn $name(&mut self, $($a: $t),*) $(-> $r)? { self.inner.$name($($a),*) })*
    };
}

impl Host for TickHost<'_, '_> {
    delegate! {
        send_message(to: &Recipient, msg: &str);
        broadcast(msg: &str);
        send_title(p: &PlayerRef, title: &str, subtitle: Option<&str>);
        send_action_bar(p: &PlayerRef, msg: &str);
        heal(p: &PlayerRef);
        set_health(p: &PlayerRef, health: f64);
        kill(p: &PlayerRef);
        teleport(p: &PlayerRef, x: f64, y: f64, z: f64);
        execute_command(command: &str);
        resolve_player(uuid: &str) -> Option<String>;
        find_player(name: &str) -> Option<PlayerRef>;
        has_permission(p: &PlayerRef, node: &str) -> bool;
        log(level: LogLevel, msg: &str);
        online_players() -> Vec<PlayerRef>;
        player_location(p: &PlayerRef) -> Option<(f64, f64, f64, String)>;
        teleport_to(p: &PlayerRef, x: f64, y: f64, z: f64, world: Option<&str>);
        held_item(p: &PlayerRef) -> Option<(String, u32)>;
        count_item(p: &PlayerRef, id: &str) -> u32;
        health(p: &PlayerRef) -> Option<f64>;
    }

    fn schedule_wait(&mut self, ticks: u64, token: u64) {
        self.hs.pending.push((self.hs.tick + ticks, token));
    }

    fn now(&mut self) -> u64 {
        self.hs.tick * 50
    }
}

impl State {
    fn scripts_dir(&self) -> String {
        format!("{}/{SCRIPTS_DIR}", self.data_dir)
    }

    fn vars_path(&self) -> String {
        format!("{}/{VARS_FILE}", self.data_dir)
    }

    fn save(&self, json: &str) {
        let path = self.vars_path();
        let tmp = format!("{path}.tmp");
        let r = std::fs::write(&tmp, json).and_then(|()| std::fs::rename(&tmp, &path));
        if let Err(e) = r {
            plog(ApiLevel::Error, &format!("could not save {path}: {e}"));
        }
    }
}

fn read_scripts(dir: &str) -> Result<Vec<(String, String)>, String> {
    let entries = std::fs::read_dir(dir).map_err(|e| format!("can't read {dir}: {e}"))?;
    let mut files = Vec::new();
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|x| x == "carve") {
            let name = path
                .file_name()
                .map(|n| n.to_string_lossy().into_owned())
                .unwrap_or_default();
            match std::fs::read_to_string(&path) {
                Ok(src) => files.push((name, src)),
                Err(e) => return Err(format!("can't read {name}: {e}")),
            }
        }
    }
    files.sort();
    Ok(files)
}

fn report(errors: &[ScriptError]) {
    for e in errors {
        plog(ApiLevel::Error, &e.to_string());
    }
}

// ---------- events ----------

struct OnEvent<E>(EventKind, std::marker::PhantomData<fn() -> E>);

trait EventAccess: FromIntoEvent {
    fn ctx(data: &EventData<Self>) -> EventCtx;
    fn set_cancelled(data: &mut EventData<Self>);
}

impl EventAccess for PlayerJoinEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        EventCtx::player(player_ref(&d.player))
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerLeaveEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        EventCtx::player(player_ref(&d.player))
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerChatEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        EventCtx::chat(player_ref(&d.player), d.message.clone())
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for BlockBreakEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        match &d.player {
            Some(p) => EventCtx::player(player_ref(p)),
            None => EventCtx::console(),
        }
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for BlockPlaceEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        EventCtx::player(player_ref(&d.player))
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerDeathEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        EventCtx::player(player_ref(&d.player))
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerRespawnEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        EventCtx::player(player_ref(&d.player))
    }
    fn set_cancelled(_d: &mut EventData<Self>) {}
}

/// Left and right clicks: runs `on click` and `on right/left click`.
struct OnInteract;

impl EventHandler<PlayerInteractEvent> for OnInteract {
    fn handle(
        &self,
        server: Server,
        mut data: EventData<PlayerInteractEvent>,
    ) -> EventData<PlayerInteractEvent> {
        let specific = match data.action {
            InteractAction::RightClickAir | InteractAction::RightClickBlock => EventKind::RightClick,
            InteractAction::LeftClickAir | InteractAction::LeftClickBlock => EventKind::LeftClick,
        };
        let cancelled = with_state(&server, None, |st, host| {
            let t = st.engine.table();
            if !t.has_event(EventKind::Click) && !t.has_event(specific) {
                return false;
            }
            let mut ctx = EventCtx::player(player_ref(&data.player));
            let block = data.block.strip_prefix("minecraft:").unwrap_or(&data.block);
            ctx.block = Some(if block.is_empty() {
                "air".to_string()
            } else {
                block.to_string()
            });
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            let a = st.engine.fire(specific, ctx.clone(), &mut th).cancelled;
            let b = st.engine.fire(EventKind::Click, ctx, &mut th).cancelled;
            a || b
        });
        if cancelled == Some(true) {
            data.cancelled = true;
        }
        data
    }
}

/// Damage types that come with an attacking entity. For these, scripts run
/// from the by-entity event that follows, so `attacker` is known.
fn has_attacker(t: DamageType) -> bool {
    matches!(
        t,
        DamageType::Arrow
            | DamageType::Fireball
            | DamageType::Fireworks
            | DamageType::MaceSmash
            | DamageType::MobAttack
            | DamageType::MobAttackNoAggro
            | DamageType::MobProjectile
            | DamageType::PlayerAttack
            | DamageType::PlayerExplosion
            | DamageType::SonicBoom
            | DamageType::Spear
            | DamageType::Spit
            | DamageType::Sting
            | DamageType::Thorns
            | DamageType::Thrown
            | DamageType::Trident
            | DamageType::WindCharge
            | DamageType::WitherSkull
    )
}

impl State {
    /// The online player with this entity id, if any.
    fn player_by_entity(&mut self, host: &mut WasmHost, id: i32) -> Option<PlayerRef> {
        let find = |list: &[(i32, String)]| list.iter().find(|(e, _)| *e == id).map(|(_, u)| u.clone());
        let uuid = match find(&self.entity_players) {
            Some(u) => Some(u),
            None if self.entity_map_tick != self.hs.tick => {
                self.entity_map_tick = self.hs.tick;
                self.entity_players = host
                    .server
                    .get_all_players()
                    .iter()
                    .map(|p| (p.as_entity().get_id() as i32, uuid::to_string(p.get_id())))
                    .collect();
                find(&self.entity_players)
            }
            None => None,
        }?;
        let name = host.resolve_player(&uuid)?;
        Some(PlayerRef { uuid, name })
    }

    fn fire_damage(
        &mut self,
        host: &mut WasmHost,
        victim: PlayerRef,
        damage: f32,
        attacker: Option<PlayerRef>,
    ) -> bool {
        let mut ctx = EventCtx::player(victim);
        ctx.damage = Some(f64::from(damage));
        ctx.attacker = attacker;
        let mut th = TickHost {
            inner: host,
            hs: &mut self.hs,
        };
        self.engine.fire(EventKind::Damage, ctx, &mut th).cancelled
    }
}

struct OnDamage;

impl EventHandler<EntityDamageEvent> for OnDamage {
    fn handle(&self, server: Server, mut data: EventData<EntityDamageEvent>) -> EventData<EntityDamageEvent> {
        let cancelled = with_state(&server, None, |st, host| {
            if !st.engine.table().has_event(EventKind::Damage) {
                return false;
            }
            let Some(victim) = st.player_by_entity(host, data.entity_id) else {
                return false;
            };
            if has_attacker(data.damage_type) {
                st.damage_pending = Some((data.entity_id, false, st.hs.tick));
                return false;
            }
            st.damage_pending = Some((data.entity_id, true, st.hs.tick));
            st.fire_damage(host, victim, data.damage, None)
        });
        if cancelled == Some(true) {
            data.cancelled = true;
        }
        data
    }
}

struct OnDamageByEntity;

impl EventHandler<EntityDamageByEntityEvent> for OnDamageByEntity {
    fn handle(
        &self,
        server: Server,
        mut data: EventData<EntityDamageByEntityEvent>,
    ) -> EventData<EntityDamageByEntityEvent> {
        let cancelled = with_state(&server, None, |st, host| {
            if !st.engine.table().has_event(EventKind::Damage) {
                return false;
            }
            // Already handled by the plain damage event of this hit.
            if let Some((id, true, tick)) = st.damage_pending
                && id == data.entity_id
                && tick == st.hs.tick
            {
                return false;
            }
            st.damage_pending = None;
            let Some(victim) = st.player_by_entity(host, data.entity_id) else {
                return false;
            };
            let attacker = st.player_by_entity(host, data.damager_id);
            st.fire_damage(host, victim, data.damage, attacker)
        });
        if cancelled == Some(true) {
            data.cancelled = true;
        }
        data
    }
}

impl<E: EventAccess> EventHandler<E> for OnEvent<E> {
    fn handle(&self, server: Server, mut data: EventData<E>) -> EventData<E> {
        let kind = self.0;
        let cancelled = with_state(&server, None, |st, host| {
            if !st.engine.table().has_event(kind) {
                return false;
            }
            let ctx = E::ctx(&data);
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine.fire(kind, ctx, &mut th).cancelled
        });
        if cancelled == Some(true) {
            E::set_cancelled(&mut data);
        }
        data
    }
}

fn register_event<E: EventAccess + Send + Sync + 'static>(ctx: &Context, kind: EventKind) {
    let h = OnEvent::<E>(kind, std::marker::PhantomData);
    if let Err(e) = ctx.register_event_handler::<E, _>(h, EventPriority::Normal, true) {
        plog(
            ApiLevel::Error,
            &format!("could not listen to {} events: {e}", kind.name()),
        );
    }
}

// ---------- commands ----------

fn raw_args(args: &ConsumedArgs) -> String {
    use pumpkin_plugin_api::command::Arg;
    match args.get_value("args") {
        Arg::Simple(s) | Arg::Msg(s) => s,
        _ => String::new(),
    }
}

struct ScriptCommand {
    name: String,
    with_args: bool,
}

impl CommandHandler for ScriptCommand {
    fn handle(&self, sender: CommandSender, server: Server, args: ConsumedArgs) -> Result<i32, CommandError> {
        let raw = if self.with_args {
            raw_args(&args)
        } else {
            String::new()
        };
        let recipient = match sender.as_player() {
            Some(p) => Recipient::Player(player_ref(&p)),
            None => Recipient::Console,
        };
        let outcome = with_state(&server, Some(&sender), |st, host| {
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine.run_command(&self.name, recipient, &raw, &mut th)
        });
        match outcome {
            Some(CommandOutcome::Ran) => Ok(1),
            Some(CommandOutcome::NoPermission) => {
                sender.send_error(text("You don't have permission to use this command."));
                Ok(0)
            }
            Some(CommandOutcome::Usage(msg)) => {
                sender.send_error(text(&msg));
                Ok(0)
            }
            Some(CommandOutcome::NotFound) => {
                sender.send_error(text(&format!(
                    "/{} is no longer in any script. Restart the server to remove it.",
                    self.name
                )));
                Ok(0)
            }
            None => Err(CommandError::CommandFailed(text("Carve is busy, try again."))),
        }
    }
}

struct ReloadCommand;

impl CommandHandler for ReloadCommand {
    fn handle(
        &self,
        sender: CommandSender,
        server: Server,
        _args: ConsumedArgs,
    ) -> Result<i32, CommandError> {
        let result = with_state(&server, None, |st, _host| -> Result<Vec<String>, String> {
            let files = read_scripts(&st.scripts_dir())?;
            match st.engine.reload(&files) {
                Ok(sum) => {
                    let new: Vec<String> = sum
                        .commands
                        .iter()
                        .filter(|c| !st.registered.contains(*c))
                        .map(|c| format!("/{c}"))
                        .collect();
                    let mut msg = format!("&aReloaded {} script(s), {} trigger(s).", sum.files, sum.triggers);
                    if !new.is_empty() {
                        msg.push_str(&format!(
                            " &eNew commands need a server restart: {}",
                            new.join(", ")
                        ));
                    }
                    Ok(vec![msg])
                }
                Err(errors) => {
                    report(&errors);
                    let mut lines = vec![format!("&c{} error(s), old scripts kept:", errors.len())];
                    lines.extend(errors.iter().take(10).map(|e| format!("&c{e}")));
                    Ok(lines)
                }
            }
        });
        match result {
            Some(Ok(lines)) => {
                if lines.first().is_some_and(|l| l.starts_with("&aReloaded")) {
                    fire_load(&server);
                }
                for l in lines {
                    sender.send_message(text(&l));
                }
                Ok(1)
            }
            Some(Err(e)) => {
                sender.send_error(text(&e));
                Ok(0)
            }
            None => Err(CommandError::CommandFailed(text("Carve is busy, try again."))),
        }
    }
}

// ---------- plugin ----------

fn tick(server: Server) {
    with_state(&server, None, |st, host| {
        st.hs.tick += 1;
        let now = st.hs.tick;
        let mut due = Vec::new();
        st.hs.pending.retain(|(at, tok)| {
            if *at <= now {
                due.push((*at, *tok));
                false
            } else {
                true
            }
        });
        due.sort();
        for (_, tok) in due {
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine.resume(tok, &mut th);
        }
        if st.engine.table().has_every() {
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine.tick(now, &mut th);
        }
        if let Some(json) = st.engine.save_due(now * 50) {
            st.save(&json);
        }
    });
}

/// Run `on load` triggers (after the first load and after each good reload).
fn fire_load(server: &Server) {
    with_state(server, None, |st, host| {
        if st.engine.table().has_event(EventKind::Load) {
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine.fire(EventKind::Load, EventCtx::console(), &mut th);
        }
    });
}

struct Carve;

impl Plugin for Carve {
    fn new() -> Self {
        Carve
    }

    fn metadata(&self) -> PluginMetadata {
        PluginMetadata {
            name: "carve".into(),
            version: env!("CARGO_PKG_VERSION").into(),
            authors: vec!["Carve contributors".into()],
            description: "Skript-like scripting for Pumpkin.".into(),
            dependencies: vec![],
            permissions: vec![
                permissions::FS_READ_DATA.into(),
                permissions::FS_WRITE_DATA.into(),
            ],
        }
    }

    fn on_load(&self, context: Context) -> Result<(), String> {
        let data_dir = context.get_data_folder();
        let scripts = format!("{data_dir}/{SCRIPTS_DIR}");
        if let Err(e) = std::fs::create_dir_all(&scripts) {
            plog(ApiLevel::Error, &format!("can't create {scripts}: {e}"));
        }
        let vars_path = format!("{data_dir}/{VARS_FILE}");
        let vars = match std::fs::read_to_string(&vars_path) {
            Ok(s) => Variables::from_json(&s).unwrap_or_else(|e| {
                let backup = format!("{vars_path}.broken");
                let _ = std::fs::rename(&vars_path, &backup);
                plog(
                    ApiLevel::Error,
                    &format!("{e}; starting with no variables, old file moved to {backup}"),
                );
                Variables::default()
            }),
            Err(_) => Variables::default(),
        };
        let mut engine = Engine::new(vars);
        let files = read_scripts(&scripts).unwrap_or_else(|e| {
            plog(ApiLevel::Error, &e);
            Vec::new()
        });
        let (sum, errors) = engine.load(&files);
        report(&errors);
        plog(
            ApiLevel::Info,
            &format!(
                "loaded {} script(s), {} trigger(s), {} error(s)",
                sum.files,
                sum.triggers,
                errors.len()
            ),
        );

        for (node, desc, default) in [
            (
                ADMIN_PERM,
                "Use /carve reload",
                PermissionDefault::Op(PermissionLevel::Two),
            ),
            (
                COMMAND_PERM,
                "Use Carve script commands",
                PermissionDefault::Allow,
            ),
        ] {
            let p = Permission {
                node: node.into(),
                description: desc.into(),
                default,
                children: vec![],
            };
            if let Err(e) = context.register_permission(&p) {
                plog(ApiLevel::Warn, &format!("permission {node}: {e}"));
            }
        }

        let admin = Command::new(&["carve".to_string(), "cv".to_string()], "Carve admin")
            .then(CommandNode::literal("reload").execute(ReloadCommand));
        context.register_command(admin, ADMIN_PERM);

        let mut registered = BTreeSet::new();
        let mut perms_done = BTreeSet::new();
        for name in &sum.commands {
            let Some(def) = engine.table().command(name) else {
                continue;
            };
            // Unregistered nodes are always denied by Pumpkin, so register ours.
            if let Some(node) = def.permission.as_deref().map(full_node)
                && node.starts_with("carve:")
                && perms_done.insert(node.clone())
            {
                let p = Permission {
                    node: node.clone(),
                    description: format!("Use /{name}"),
                    default: PermissionDefault::Op(PermissionLevel::Two),
                    children: vec![],
                };
                if let Err(e) = context.register_permission(&p) {
                    plog(ApiLevel::Warn, &format!("permission {node}: {e}"));
                }
            }
            let desc = def.description.clone().unwrap_or_else(|| def.usage());
            let mut cmd = Command::new(std::slice::from_ref(name), &desc);
            if def.args.iter().all(|a| a.optional) {
                cmd = cmd.execute(ScriptCommand {
                    name: name.clone(),
                    with_args: false,
                });
            }
            if !def.args.is_empty() {
                let node = CommandNode::argument("args", &ArgumentType::String(StringType::Greedy)).execute(
                    ScriptCommand {
                        name: name.clone(),
                        with_args: true,
                    },
                );
                cmd = cmd.then(node);
            }
            context.register_command(cmd, COMMAND_PERM);
            registered.insert(name.clone());
        }

        register_event::<PlayerJoinEvent>(&context, EventKind::Join);
        register_event::<PlayerLeaveEvent>(&context, EventKind::Quit);
        register_event::<PlayerChatEvent>(&context, EventKind::Chat);
        register_event::<BlockBreakEvent>(&context, EventKind::Break);
        register_event::<BlockPlaceEvent>(&context, EventKind::Place);
        register_event::<PlayerDeathEvent>(&context, EventKind::Death);
        register_event::<PlayerRespawnEvent>(&context, EventKind::Respawn);
        for r in [
            context.register_event_handler::<PlayerInteractEvent, _>(OnInteract, EventPriority::Normal, true),
            context.register_event_handler::<EntityDamageEvent, _>(OnDamage, EventPriority::Normal, true),
            context.register_event_handler::<EntityDamageByEntityEvent, _>(
                OnDamageByEntity,
                EventPriority::Normal,
                true,
            ),
        ] {
            if let Err(e) = r {
                plog(ApiLevel::Error, &format!("could not listen to an event: {e}"));
            }
        }

        if let Ok(mut g) = STATE.try_lock() {
            *g = Some(State {
                engine,
                hs: HostState {
                    tick: 0,
                    pending: Vec::new(),
                },
                data_dir,
                registered,
                entity_players: Vec::new(),
                entity_map_tick: u64::MAX,
                damage_pending: None,
            });
        }
        schedule_repeating_task(1, 1, tick);
        fire_load(&context.get_server());
        Ok(())
    }

    fn on_unload(&self, _context: Context) -> Result<(), String> {
        if let Ok(mut g) = STATE.try_lock()
            && let Some(st) = g.as_mut()
            && let Some(json) = st.engine.save_now()
        {
            st.save(&json);
        }
        Ok(())
    }
}

register_plugin!(Carve);

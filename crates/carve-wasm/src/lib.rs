//! Carve as a Pumpkin WASM plugin. All game access goes through
//! [`WasmHost`], which implements the core's `Host` trait.

use std::collections::BTreeSet;
use std::sync::Mutex;

use carve_core::health_food::Vital;
use carve_core::{
    CommandOutcome, Engine, EventCtx, EventKind, Host, LogLevel, PlayerFlag, PlayerRef, Recipient, ScriptError, Variables,
};
use pumpkin_plugin_api::DamageType;
use pumpkin_plugin_api::command::{
    ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType,
};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::common::{EntityPose, GameMode, Hand, RgbColor};
use pumpkin_plugin_api::events::{
    BlockBreakEvent, BlockPlaceEvent, EntityDamageByEntityEvent, EntityDamageEvent, EventData, EventHandler,
    EventPriority, FromIntoEvent, InteractAction, PlayerChatEvent, PlayerDeathEvent, PlayerInteractEvent,
    PlayerJoinEvent, PlayerLeaveEvent, PlayerRespawnEvent, PlayerToggleFlightEvent, PlayerToggleSneakEvent,
    PlayerToggleSprintEvent,
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
    /// Extra events (`w2_events_1`) that have a listener on the server.
    ext_registered: Vec<carve_core::w2_events_1::ExtEvent>,
    /// World time of the previous tick, for `at <time>:` triggers.
    at_prev: Option<i64>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

fn plog(level: ApiLevel, msg: &str) {
    log(level, &format!("[Carve] {msg}"));
}

/// Build a component from `&a` / `§a` codes, `<#rrggbb>` and `<red>` style tags.
fn text(s: &str) -> TextComponent {
    if !s.contains(['&', '§', '<']) {
        return TextComponent::text(s);
    }
    let mut root = TextComponent::text("");
    for span in carve_core::colors::parse(s) {
        let mut part = TextComponent::text(&span.text);
        let st = span.style;
        if let Some((r, g, b)) = st.color {
            part = part.color_rgb(RgbColor { r, g, b });
        }
        if st.bold {
            part = part.bold(true);
        }
        if st.italic {
            part = part.italic(true);
        }
        if st.underlined {
            part = part.underlined(true);
        }
        if st.strikethrough {
            part = part.strikethrough(true);
        }
        if st.obfuscated {
            part = part.obfuscated(true);
        }
        root = root.add_child(part);
    }
    root
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
    for (who, cmd) in std::mem::take(&mut host.deferred) {
        let sender = who
            .and_then(|u| uuid::parse(&u))
            .and_then(|id| server.get_player_by_uuid(id))
            .map_or(
                pumpkin_plugin_api::server::CommandSender::Console,
                pumpkin_plugin_api::server::CommandSender::Player,
            );
        server.execute_command(&cmd, sender);
    }
    Some(result)
}

struct WasmHost<'a> {
    server: &'a Server,
    /// The sender of the command being run, for console replies.
    reply: Option<&'a CommandSender>,
    /// Queued commands: the player uuid to run as (None: console), and the command.
    deferred: Vec<(Option<String>, String)>,
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

/// UUIDs of players who are flying right now. The API reports flight only
/// through the toggle event, so it is tracked here.
static FLYING: Mutex<Vec<String>> = Mutex::new(Vec::new());

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
        self.deferred.push((None, command.to_string()));
    }

    fn execute_command_as(&mut self, p: &PlayerRef, command: &str) {
        self.deferred.push((Some(p.uuid.clone()), command.to_string()));
    }

    fn send_console(&mut self, msg: &str) {
        plog(ApiLevel::Info, msg);
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

    fn gamemode(&mut self, p: &PlayerRef) -> Option<String> {
        self.get(p).map(|pl| {
            match pl.get_gamemode() {
                GameMode::Survival => "survival",
                GameMode::Creative => "creative",
                GameMode::Adventure => "adventure",
                GameMode::Spectator => "spectator",
            }
            .to_string()
        })
    }

    fn set_gamemode(&mut self, p: &PlayerRef, mode: &str) {
        let m = match mode {
            "creative" => GameMode::Creative,
            "adventure" => GameMode::Adventure,
            "spectator" => GameMode::Spectator,
            _ => GameMode::Survival,
        };
        if let Some(pl) = self.get(p) {
            pl.set_gamemode(m);
        }
    }

    fn player_flag(&mut self, p: &PlayerRef, flag: PlayerFlag) -> bool {
        let flying = FLYING.lock().is_ok_and(|f| f.contains(&p.uuid));
        let Some(pl) = self.get(p) else {
            return false;
        };
        let e = pl.as_entity();
        match flag {
            PlayerFlag::Sneaking => e.is_sneaking(),
            PlayerFlag::Sprinting => e.is_sprinting(),
            PlayerFlag::Swimming => e.is_swimming(),
            PlayerFlag::OnGround => e.is_on_ground(),
            PlayerFlag::Sleeping => matches!(e.get_pose(), EntityPose::Sleeping),
            PlayerFlag::Flying => flying,
            // The API has no "allow flight" ability, so this follows the game mode.
            PlayerFlag::CanFly => {
                matches!(pl.get_gamemode(), GameMode::Creative | GameMode::Spectator) || flying
            }
            // No shield-blocking state in the plugin API yet.
            PlayerFlag::Blocking => false,
        }
    }

    fn xp_get(&mut self, p: &PlayerRef, field: carve_core::xp_level::XpField) -> Option<f64> {
        use carve_core::xp_level::XpField;
        self.get(p).map(|pl| match field {
            XpField::Level => f64::from(pl.get_experience_level()),
            XpField::Points => f64::from(pl.get_experience_points()),
            XpField::Progress => f64::from(pl.get_experience_progress()),
        })
    }

    fn xp_set(&mut self, p: &PlayerRef, field: carve_core::xp_level::XpField, value: f64) {
        use carve_core::xp_level::XpField;
        if let Some(pl) = self.get(p) {
            match field {
                XpField::Level => pl.set_experience_level(value as i32),
                XpField::Points => pl.set_experience_points(value as i32),
                XpField::Progress => pl.set_experience_progress(value as f32),
            }
        }
    }

    fn vital(&mut self, p: &PlayerRef, v: Vital) -> Option<f64> {
        self.get(p).map(|pl| match v {
            Vital::Health => f64::from(pl.get_health()),
            Vital::MaxHealth => f64::from(pl.get_max_health()),
            Vital::Food => f64::from(pl.get_food_level()),
            Vital::Saturation => f64::from(pl.get_saturation()),
        })
    }

    fn set_vital(&mut self, p: &PlayerRef, v: Vital, value: f64) {
        if let Some(pl) = self.get(p) {
            match v {
                Vital::Health => pl.set_health(value as f32),
                Vital::MaxHealth => pl.set_max_health(value as f32),
                Vital::Food => pl.set_food_level(value.round() as u8),
                Vital::Saturation => pl.set_saturation(value as f32),
            }
        }
    }

    fn damage(&mut self, p: &PlayerRef, amount: f64) {
        if let Some(pl) = self.get(p) {
            pl.damage(amount as f32, DamageType::Generic);
        }
    }

    fn transfer(&mut self, p: &PlayerRef, host: &str, port: u16) {
        if let Some(pl) = self.get(p) {
            pl.transfer(host, port);
        }
    }

    fn respawn(&mut self, p: &PlayerRef) {
        if let Some(pl) = self.get(p) {
            pl.respawn();
        }
    }

    fn hide_player(&mut self, viewer: &PlayerRef, target: &PlayerRef) {
        // The API takes the other player by value, so ask the server for a fresh handle.
        let Some(t) = uuid::parse(&target.uuid).and_then(|id| self.server.get_player_by_uuid(id)) else {
            return;
        };
        if let Some(v) = self.get(viewer) {
            v.hide_player(t);
        }
    }

    fn show_player(&mut self, viewer: &PlayerRef, target: &PlayerRef) {
        let Some(t) = uuid::parse(&target.uuid).and_then(|id| self.server.get_player_by_uuid(id)) else {
            return;
        };
        if let Some(v) = self.get(viewer) {
            v.show_player(t);
        }
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
        execute_command_as(p: &PlayerRef, command: &str);
        send_console(msg: &str);
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
        gamemode(p: &PlayerRef) -> Option<String>;
        set_gamemode(p: &PlayerRef, mode: &str);
        player_flag(p: &PlayerRef, flag: PlayerFlag) -> bool;
        xp_get(p: &PlayerRef, field: carve_core::xp_level::XpField) -> Option<f64>;
        xp_set(p: &PlayerRef, field: carve_core::xp_level::XpField, value: f64);
        vital(p: &PlayerRef, v: Vital) -> Option<f64>;
        set_vital(p: &PlayerRef, v: Vital, value: f64);
        damage(p: &PlayerRef, amount: f64);
        transfer(p: &PlayerRef, host: &str, port: u16);
        respawn(p: &PlayerRef);
        hide_player(viewer: &PlayerRef, target: &PlayerRef);
        show_player(viewer: &PlayerRef, target: &PlayerRef);
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
            // `-name.carve` is disabled (Skript's convention).
            if name.starts_with('-') {
                continue;
            }
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

impl EventAccess for PlayerToggleSneakEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        EventCtx::player(player_ref(&d.player))
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerToggleSprintEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        EventCtx::player(player_ref(&d.player))
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerToggleFlightEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        let p = player_ref(&d.player);
        if let Ok(mut f) = FLYING.lock() {
            f.retain(|u| *u != p.uuid);
            if d.is_flying {
                f.push(p.uuid.clone());
            }
        }
        EventCtx::player(p)
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
        // The flight change did not happen.
        if let Ok(mut f) = FLYING.lock() {
            let u = uuid::to_string(d.player.get_id());
            f.retain(|x| *x != u);
            if !d.is_flying {
                f.push(u);
            }
        }
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

// ---------- extra events (carve-core `w2_events_1`) ----------

use carve_core::Value;
use carve_core::w2_events_1::ExtEvent;
use pumpkin_plugin_api::events as ev;

/// Like `EventAccess`, but the event kind may depend on the data (a book
/// edit and a book signing are one server event).
trait ExtAccess: FromIntoEvent {
    fn kind(d: &EventData<Self>) -> ExtEvent;
    fn ctx(d: &EventData<Self>) -> EventCtx;
    fn set_cancelled(d: &mut EventData<Self>);
}

struct OnExt<E>(std::marker::PhantomData<fn() -> E>);

impl<E: ExtAccess> EventHandler<E> for OnExt<E> {
    fn handle(&self, server: Server, mut data: EventData<E>) -> EventData<E> {
        let cancelled = with_state(&server, None, |st, host| {
            let kind = EventKind::Ext(E::kind(&data));
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

macro_rules! ext_event {
    ($ty:ty, |$k:ident| $kind:expr, |$d:ident| $ctx:expr, |$c:ident| $cancel:expr) => {
        impl ExtAccess for $ty {
            fn kind($k: &EventData<Self>) -> ExtEvent {
                let _ = $k;
                $kind
            }
            fn ctx($d: &EventData<Self>) -> EventCtx {
                $ctx
            }
            fn set_cancelled($c: &mut EventData<Self>) {
                let _ = &$c;
                $cancel
            }
        }
    };
}

fn ev_ctx(player: Option<&Player>, ev: Vec<Value>) -> EventCtx {
    let mut c = match player {
        Some(p) => EventCtx::player(player_ref(p)),
        None => EventCtx::console(),
    };
    c.ev = ev;
    c
}

fn at_pos(world: Option<String>, p: &pumpkin_plugin_api::common::BlockPos) -> Value {
    Value::Location {
        x: f64::from(p.x),
        y: f64::from(p.y),
        z: f64::from(p.z),
        world,
    }
}

fn ev_item(id: &str) -> Value {
    Value::Item {
        id: id.strip_prefix("minecraft:").unwrap_or(id).to_string(),
        count: 1,
    }
}

ext_event!(ev::PrepareAnvilEvent, |d| ExtEvent::AnvilPrepare,
    |d| ev_ctx(Some(&d.player), vec![Value::Text(d.rename_text.clone()), Value::Num(f64::from(d.repair_cost))]),
    |d| {});
ext_event!(ev::PlayerAnimationEvent, |d| ExtEvent::ArmSwing,
    |d| ev_ctx(Some(&d.player), vec![Value::Text(d.animation_type.clone())]),
    |d| d.cancelled = true);
ext_event!(ev::PlayerBedEnterEvent, |d| ExtEvent::BedEnter,
    |d| ev_ctx(Some(&d.player), vec![at_pos(Some(d.player.get_world().get_name()), &d.bed_pos)]),
    |d| d.cancelled = true);
ext_event!(ev::PlayerBedLeaveEvent, |d| ExtEvent::BedLeave,
    |d| ev_ctx(Some(&d.player), vec![at_pos(Some(d.player.get_world().get_name()), &d.bed_pos)]),
    |d| {});
ext_event!(ev::BellRingEvent, |d| ExtEvent::BellRing,
    |d| ev_ctx(None, vec![
        at_pos(Some(d.target_world.get_name()), &d.block_pos),
        Value::World(d.target_world.get_name()),
        d.entity_id.map_or(Value::None, |i| Value::Num(f64::from(i))),
        d.direction.clone().map_or(Value::None, Value::Text),
    ]),
    |d| d.cancelled = true);
ext_event!(ev::BlockDamageEvent, |d| ExtEvent::BlockDamage,
    |d| ev_ctx(Some(&d.player), vec![
        at_pos(Some(d.player.get_world().get_name()), &d.block_pos),
        Value::Bool(d.insta_break),
    ]),
    |d| d.cancelled = true);
ext_event!(ev::BlockDropItemEvent, |d| ExtEvent::BlockDrop,
    |d| ev_ctx(d.player.as_ref(), vec![
        at_pos(Some(d.target_world.get_name()), &d.block_pos),
        Value::World(d.target_world.get_name()),
        d.items.first().map_or(Value::None, |s| {
            let mut v = ev_item(&s.get_registry_key());
            if let Value::Item { count, .. } = &mut v {
                *count = u32::from(s.get_count());
            }
            v
        }),
        Value::Num(d.items.len() as f64),
    ]),
    |d| d.cancelled = true);
ext_event!(ev::BlockFertilizeEvent, |d| ExtEvent::BlockFertilize,
    |d| ev_ctx(d.player.as_ref(), vec![
        at_pos(Some(d.target_world.get_name()), &d.block_pos),
        Value::World(d.target_world.get_name()),
        Value::Num(d.changed_blocks.len() as f64),
    ]),
    |d| d.cancelled = true);
ext_event!(ev::BlockGrowEvent, |d| ExtEvent::BlockGrowth,
    |d| ev_ctx(None, vec![
        at_pos(Some(d.target_world.get_name()), &d.block_pos),
        Value::World(d.target_world.get_name()),
        ev_item(&d.new_block),
        ev_item(&d.old_block),
    ]),
    |d| d.cancelled = true);
ext_event!(ev::PlayerEditBookEvent, |d| if d.signing { ExtEvent::BookSign } else { ExtEvent::BookEdit },
    |d| ev_ctx(Some(&d.player), vec![
        d.title.clone().map_or(Value::None, Value::Text),
        Value::Num(f64::from(d.slot)),
        Value::Text(d.pages.join("\n")),
    ]),
    |d| d.cancelled = true);
ext_event!(ev::BrewEvent, |d| ExtEvent::BrewingComplete,
    |d| ev_ctx(None, vec![at_pos(None, &d.block_pos), Value::Num(f64::from(d.fuel_level))]),
    |d| d.cancelled = true);
ext_event!(ev::BrewingStandFuelEvent, |d| ExtEvent::BrewingFuel,
    |d| ev_ctx(None, vec![at_pos(None, &d.block_pos), Value::Num(f64::from(d.fuel_power))]),
    |d| d.cancelled = true);
ext_event!(ev::BrewingStartEvent, |d| ExtEvent::BrewingStart,
    |d| ev_ctx(None, vec![
        at_pos(Some(d.target_world.get_name()), &d.block_pos),
        Value::World(d.target_world.get_name()),
        Value::Num(f64::from(d.brewing_time)),
    ]),
    |d| d.cancelled = true);
ext_event!(ev::ServerBroadcastEvent, |d| ExtEvent::Broadcast,
    |d| ev_ctx(None, vec![Value::Text(d.message.get_text()), Value::Text(d.sender.get_text())]),
    |d| d.cancelled = true);
ext_event!(ev::PlayerBucketEntityEvent, |d| ExtEvent::BucketCatch,
    |d| ev_ctx(Some(&d.player), vec![ev_item(&d.bucket_item), Value::Num(f64::from(d.entity_id))]),
    |d| d.cancelled = true);
ext_event!(ev::PlayerBucketEmptyEvent, |d| ExtEvent::BucketEmpty,
    |d| ev_ctx(Some(&d.player), vec![
        at_pos(Some(d.player.get_world().get_name()), &d.block_pos),
        ev_item(&d.bucket),
    ]),
    |d| d.cancelled = true);
ext_event!(ev::PlayerBucketFillEvent, |d| ExtEvent::BucketFill,
    |d| ev_ctx(Some(&d.player), vec![
        at_pos(Some(d.player.get_world().get_name()), &d.block_pos),
        ev_item(&d.bucket),
    ]),
    |d| d.cancelled = true);
ext_event!(ev::BlockBurnEvent, |d| ExtEvent::Burn,
    |d| ev_ctx(None, vec![
        at_pos(Some(d.target_world.get_name()), &d.block_pos),
        Value::World(d.target_world.get_name()),
        ev_item(&d.block),
        ev_item(&d.igniting_block),
    ]),
    |d| d.cancelled = true);
ext_event!(ev::BlockCanBuildEvent, |d| ExtEvent::CanBuild,
    |d| ev_ctx(Some(&d.player), vec![
        ev_item(&d.block_to_build),
        ev_item(&d.block),
        Value::Bool(d.buildable),
    ]),
    |d| {
        d.cancelled = true;
        d.buildable = false;
    });
ext_event!(ev::ChunkPopulateEvent, |d| ExtEvent::ChunkGenerate,
    |d| ev_ctx(None, vec![Value::Num(f64::from(d.chunk_x)), Value::Num(f64::from(d.chunk_z))]),
    |d| d.cancelled = true);
ext_event!(ev::ChunkUnloadEvent, |d| ExtEvent::ChunkUnload,
    |d| ev_ctx(None, vec![Value::Num(f64::from(d.chunk_x)), Value::Num(f64::from(d.chunk_z))]),
    |d| d.cancelled = true);
ext_event!(ev::EntityCombustEvent, |d| ExtEvent::Combust,
    |d| ev_ctx(None, vec![
        Value::Num(f64::from(d.entity_id)),
        Value::Timespan { millis: (f64::from(d.duration_secs) * 1000.0) as i64 },
    ]),
    |d| d.cancelled = true);
ext_event!(ev::PlayerLoginEvent, |d| ExtEvent::Connect,
    |d| ev_ctx(Some(&d.player), vec![]),
    |d| d.cancelled = true);
ext_event!(ev::BlockDispenseEvent, |d| ExtEvent::Dispense,
    |d| ev_ctx(None, vec![at_pos(None, &d.block_pos), ev_item(&d.item_name)]),
    |d| d.cancelled = true);
ext_event!(ev::PlayerEggThrowEvent, |d| ExtEvent::EggThrow,
    |d| ev_ctx(Some(&d.player), vec![
        Value::Bool(d.hatching),
        Value::Num(f64::from(d.num_hatches)),
        Value::Text(d.hatching_type.clone()),
    ]),
    |d| d.cancelled = true);
ext_event!(ev::PlayerElytraBoostEvent, |d| ExtEvent::ElytraBoost,
    |d| ev_ctx(Some(&d.player), vec![Value::Num(f64::from(d.firework_id))]),
    |d| d.cancelled = true);

/// Attach the listeners for the extra events that the loaded scripts use.
/// Listeners can only be added at load, so a trigger added by a later
/// `/carve reload` needs a restart (see [`missing_ext_listeners`]).
fn register_ext_events(ctx: &Context, engine: &Engine) -> Vec<ExtEvent> {
    let table = engine.table();
    let mut done = Vec::new();
    macro_rules! reg {
        ($ty:ty, $($k:expr),+) => {{
            let kinds = [$($k),+];
            if kinds.iter().any(|k| table.has_event(EventKind::Ext(*k))) {
                let h = OnExt::<$ty>(std::marker::PhantomData);
                match ctx.register_event_handler::<$ty, _>(h, EventPriority::Normal, true) {
                    Ok(_) => done.extend(kinds),
                    Err(e) => plog(ApiLevel::Error, &format!("could not listen to {}: {e}", kinds[0].name())),
                }
            }
        }};
    }
    reg!(ev::PrepareAnvilEvent, ExtEvent::AnvilPrepare);
    reg!(ev::PlayerAnimationEvent, ExtEvent::ArmSwing);
    reg!(ev::PlayerBedEnterEvent, ExtEvent::BedEnter);
    reg!(ev::PlayerBedLeaveEvent, ExtEvent::BedLeave);
    reg!(ev::BellRingEvent, ExtEvent::BellRing);
    reg!(ev::BlockDamageEvent, ExtEvent::BlockDamage);
    reg!(ev::BlockDropItemEvent, ExtEvent::BlockDrop);
    reg!(ev::BlockFertilizeEvent, ExtEvent::BlockFertilize);
    reg!(ev::BlockGrowEvent, ExtEvent::BlockGrowth);
    reg!(ev::PlayerEditBookEvent, ExtEvent::BookEdit, ExtEvent::BookSign);
    reg!(ev::BrewEvent, ExtEvent::BrewingComplete);
    reg!(ev::BrewingStandFuelEvent, ExtEvent::BrewingFuel);
    reg!(ev::BrewingStartEvent, ExtEvent::BrewingStart);
    reg!(ev::ServerBroadcastEvent, ExtEvent::Broadcast);
    reg!(ev::PlayerBucketEntityEvent, ExtEvent::BucketCatch);
    reg!(ev::PlayerBucketEmptyEvent, ExtEvent::BucketEmpty);
    reg!(ev::PlayerBucketFillEvent, ExtEvent::BucketFill);
    reg!(ev::BlockBurnEvent, ExtEvent::Burn);
    reg!(ev::BlockCanBuildEvent, ExtEvent::CanBuild);
    reg!(ev::ChunkPopulateEvent, ExtEvent::ChunkGenerate);
    reg!(ev::ChunkUnloadEvent, ExtEvent::ChunkUnload);
    reg!(ev::EntityCombustEvent, ExtEvent::Combust);
    reg!(ev::PlayerLoginEvent, ExtEvent::Connect);
    reg!(ev::BlockDispenseEvent, ExtEvent::Dispense);
    reg!(ev::PlayerEggThrowEvent, ExtEvent::EggThrow);
    reg!(ev::PlayerElytraBoostEvent, ExtEvent::ElytraBoost);
    done
}

/// Names of used extra events that have no listener (added after startup).
fn missing_ext_listeners(engine: &Engine, registered: &[ExtEvent]) -> Vec<&'static str> {
    let table = engine.table();
    ALL_EXT
        .iter()
        .filter(|k| table.has_event(EventKind::Ext(**k)) && !registered.contains(k))
        .map(|k| k.name())
        .collect()
}

const ALL_EXT: [ExtEvent; 27] = [
    ExtEvent::AnvilPrepare,
    ExtEvent::ArmSwing,
    ExtEvent::BedEnter,
    ExtEvent::BedLeave,
    ExtEvent::BellRing,
    ExtEvent::BlockDamage,
    ExtEvent::BlockDrop,
    ExtEvent::BlockFertilize,
    ExtEvent::BlockGrowth,
    ExtEvent::BookEdit,
    ExtEvent::BookSign,
    ExtEvent::BrewingComplete,
    ExtEvent::BrewingFuel,
    ExtEvent::BrewingStart,
    ExtEvent::Broadcast,
    ExtEvent::BucketCatch,
    ExtEvent::BucketEmpty,
    ExtEvent::BucketFill,
    ExtEvent::Burn,
    ExtEvent::CanBuild,
    ExtEvent::ChunkGenerate,
    ExtEvent::ChunkUnload,
    ExtEvent::Combust,
    ExtEvent::Connect,
    ExtEvent::Dispense,
    ExtEvent::EggThrow,
    ExtEvent::ElytraBoost,
];

/// World clock handling for `at <time>:` triggers, once per tick.
fn tick_at_time(server: &Server, st: &mut State, host: &mut WasmHost) {
    if !st.engine.table().has_at() {
        return;
    }
    let Some(w) = server.get_all_worlds().into_iter().next() else {
        return;
    };
    let cur = (w.get_time_of_day() % 24000) as i64;
    if let Some(prev) = st.at_prev.replace(cur) {
        let mut th = TickHost {
            inner: host,
            hs: &mut st.hs,
        };
        st.engine.fire_at_time(prev, cur, &mut th);
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

/// The scripts folder, for `/carve`.
struct DirStore(String);

impl carve_core::admin::ScriptStore for DirStore {
    fn list(&mut self) -> Result<Vec<String>, String> {
        let entries = std::fs::read_dir(&self.0).map_err(|e| format!("can't read {}: {e}", self.0))?;
        Ok(entries
            .flatten()
            .filter_map(|e| e.file_name().into_string().ok())
            .collect())
    }
    fn read(&mut self, name: &str) -> Result<String, String> {
        std::fs::read_to_string(format!("{}/{name}", self.0)).map_err(|e| format!("can't read {name}: {e}"))
    }
    fn rename(&mut self, from: &str, to: &str) -> Result<(), String> {
        std::fs::rename(format!("{}/{from}", self.0), format!("{}/{to}", self.0))
            .map_err(|e| format!("can't rename {from} to {to}: {e}"))
    }
}

fn admin_args(args: &ConsumedArgs) -> String {
    use pumpkin_plugin_api::command::Arg;
    match args.get_value("args") {
        Arg::Simple(s) | Arg::Msg(s) => s,
        _ => String::new(),
    }
}

/// `/carve <sub> [args]` (alias `/cv`).
struct AdminCommand {
    sub: &'static str,
    with_args: bool,
}

impl CommandHandler for AdminCommand {
    fn handle(&self, sender: CommandSender, server: Server, args: ConsumedArgs) -> Result<i32, CommandError> {
        let raw = if self.with_args {
            format!("{} {}", self.sub, admin_args(&args))
        } else {
            self.sub.to_string()
        };
        let recipient = match sender.as_player() {
            Some(p) => Recipient::Player(player_ref(&p)),
            None => Recipient::Console,
        };
        let out = with_state(&server, Some(&sender), |st, host| {
            let allowed = |node: &str| sender.has_permission(&server, node);
            let mut store = DirStore(st.scripts_dir());
            let env = carve_core::admin::AdminEnv {
                sender: recipient,
                allowed: &allowed,
                registered: &st.registered,
                version: env!("CARGO_PKG_VERSION"),
                api: "v0.1",
                data_folder: "plugins/data/carve",
            };
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            let mut out = carve_core::admin::admin(&mut st.engine, &mut store, &mut th, &env, &raw);
            let missing = missing_ext_listeners(&st.engine, &st.ext_registered);
            if out.loaded && !missing.is_empty() {
                out.lines.push(format!(
                    "Restart the server to listen to: {}.",
                    missing.join(", ")
                ));
            }

            out
        });
        let Some(out) = out else {
            return Err(CommandError::CommandFailed(text("Carve is busy, try again.")));
        };
        if out.loaded {
            fire_load(&server);
        }
        for l in &out.lines {
            sender.send_message(text(l));
        }
        Ok(1)
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
        tick_at_time(&server, st, host);
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
                "Use every /carve subcommand",
                PermissionDefault::Op(PermissionLevel::Two),
            ),
            (
                "carve:reload",
                "Use /carve reload",
                PermissionDefault::Op(PermissionLevel::Two),
            ),
            (
                "carve:enable",
                "Use /carve enable and disable",
                PermissionDefault::Op(PermissionLevel::Two),
            ),
            (
                "carve:vars",
                "Use /carve vars",
                PermissionDefault::Op(PermissionLevel::Two),
            ),
            (
                "carve:run",
                "Use /carve run",
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

        // Everyone may run /carve (help); each subcommand checks its own permission.
        // Subcommands are literal nodes, so the client completes them by itself.
        // No plugin suggestion handler: Pumpkin 0.2.2 panics (and the server
        // stops) when it calls one ("there is no reactor running").
        let mut admin = Command::new(
            &["carve".to_string(), "cv".to_string()],
            "Carve scripts: /carve help",
        )
        .execute(AdminCommand {
            sub: "help",
            with_args: false,
        });
        for (name, ..) in carve_core::admin::SUBCOMMANDS {
            admin = admin.then(
                CommandNode::literal(name)
                    .execute(AdminCommand {
                        sub: name,
                        with_args: false,
                    })
                    .then(
                        CommandNode::argument("args", &ArgumentType::String(StringType::Greedy)).execute(
                            AdminCommand {
                                sub: name,
                                with_args: true,
                            },
                        ),
                    ),
            );
        }
        context.register_command(admin, COMMAND_PERM);

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
        register_event::<PlayerToggleSneakEvent>(&context, EventKind::SneakToggle);
        register_event::<PlayerToggleSprintEvent>(&context, EventKind::SprintToggle);
        register_event::<PlayerToggleFlightEvent>(&context, EventKind::FlightToggle);
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

        let ext_registered = register_ext_events(&context, &engine);
        if let Ok(mut g) = STATE.try_lock() {
            *g = Some(State {
                ext_registered,
                at_prev: None,
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

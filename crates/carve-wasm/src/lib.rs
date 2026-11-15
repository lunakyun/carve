//! Carve as a Pumpkin WASM plugin. All game access goes through
//! [`WasmHost`], which implements the core's `Host` trait.
#![allow(clippy::missing_const_for_thread_local)]

mod w2_events_4;

use std::collections::BTreeSet;
use std::sync::Mutex;

use carve_core::block_ops::BlockPos;
use carve_core::health_food::Vital;
use carve_core::inventories::InvClick;
use carve_core::skbee_recipes_advancements::{CookKind, Ingredient as RIngredient, RecipeDef, RecipeKind};
use carve_core::skbee_text::{Click, Hover, Kind, Part};
use carve_core::skbee_utilities::Info;
use carve_core::w2_expressions_7::Query;
use carve_core::weather_time::Weather;
use carve_core::world_effects::{FxPos, ParticleFx};
use carve_core::{
    CommandOutcome, Engine, EventCtx, EventKind, Host, LogLevel, PlayerFlag, PlayerRef, Recipient, Value,
    Variables,
};
mod w2_events_3;

use pumpkin_plugin_api::CustomStatistic;
use pumpkin_plugin_api::DamageType;
use pumpkin_plugin_api::World;
use pumpkin_plugin_api::command::{
    ArgumentType, Command, CommandError, CommandNode, CommandSender, ConsumedArgs, StringType,
};
use pumpkin_plugin_api::commands::CommandHandler;
use pumpkin_plugin_api::common::{BlockPos as WitPos, EntityPose, GameMode, Hand, RgbColor};
use pumpkin_plugin_api::events::{
    BlockBreakEvent, BlockPlaceEvent, EntityDamageByEntityEvent, EntityDamageEvent, EventData, EventHandler,
    EventPriority, FromIntoEvent, InteractAction, InventoryClickEvent, InventoryCloseEvent, PlayerChatEvent,
    PlayerDeathEvent, PlayerInteractEvent, PlayerJoinEvent, PlayerLeaveEvent, PlayerRespawnEvent,
    PlayerToggleFlightEvent, PlayerToggleSneakEvent, PlayerToggleSprintEvent,
};
use pumpkin_plugin_api::events::{PlayerAdvancementDoneEvent, PlayerRecipeDiscoverEvent};
use pumpkin_plugin_api::logging::{LogLevel as ApiLevel, log};
use pumpkin_plugin_api::permission::{Permission, PermissionDefault, PermissionLevel};
use pumpkin_plugin_api::player::JavaResourcePack;
use pumpkin_plugin_api::scheduler::schedule_repeating_task;
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::wit::pumpkin::plugin::status_effect::StatusEffectInstance;
use pumpkin_plugin_api::wit::pumpkin::plugin::status_effect::StatusEffectType;
use pumpkin_plugin_api::world::{BlockFlags, ExplosionInteraction};
use pumpkin_plugin_api::{Block, GameRule, GameRuleValue};
use pumpkin_plugin_api::{
    Context, Player, Plugin, PluginMetadata, Server, permissions, register_plugin, uuid,
};

mod combat;
mod custom_stats;
mod displays;
mod ent_host;
mod ents;
mod events_items;
mod fx;
mod gaps;
mod inv;
mod sb;

mod w2_events_2;
mod w2_expressions_3;
mod w4;
mod w4_conditions_1;
mod w4_events_movement;
mod w5_chunk1;
mod w5_chunk2;
mod w5_chunk3;
mod w5_chunk4;
mod w5_chunk5;
mod w5_chunk6;
mod w6_chunk1;

const SCRIPTS_DIR: &str = "scripts";
const VARS_FILE: &str = "variables.json";
const BOUNDS_FILE: &str = "bounds.json";
const ADMIN_PERM: &str = "carve:admin";
const COMMAND_PERM: &str = "carve:command";

struct HostState {
    tick: u64,
    /// (due tick, resume token)
    pending: Vec<(u64, u64)>,
    w4: w4::W4State,
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
    /// Last wall-clock minute handled by `at HH:MM in real time` triggers.
    last_minute: i64,
    /// Victim of the damage event being handled, and whether scripts already ran
    /// for it (the by-entity event that may follow must not run them twice).
    damage_pending: Option<(i32, bool, u64)>,
    /// A hit whose by-entity event may still follow (see `combat`).
    late_hit: Option<combat::LateHit>,
    /// Extra events (`w2_events_1`) that have a listener on the server.
    ext_registered: Vec<carve_core::w2_events_1::ExtEvent>,
    /// World time of the previous tick, for `at <time>:` triggers.
    at_prev: Option<i64>,
}

static STATE: Mutex<Option<State>> = Mutex::new(None);

thread_local! {
    // The server handle, kept for IPC calls from other plugins (they come without one).
    static PH_SERVER: std::cell::RefCell<Option<Server>> = const { std::cell::RefCell::new(None) };
}

fn plog(level: ApiLevel, msg: &str) {
    log(level, &format!("[Carve] {msg}"));
}

/// `Enchantment::SweepingEdge` (or `SweepingEdge`) as `sweeping_edge`.
fn enchant_id(debug: &str) -> String {
    let name = debug.rsplit("::").next().unwrap_or(debug);
    let mut out = String::with_capacity(name.len() + 4);
    for (i, c) in name.chars().enumerate() {
        if c.is_ascii_uppercase() {
            if i > 0 {
                out.push('_');
            }
            out.push(c.to_ascii_lowercase());
        } else {
            out.push(c);
        }
    }
    out
}

/// Build a component from parsed SkBee-style markup: hover, click,
/// insertion, keybinds, translations and per-letter colors.
fn rich(parts: &[Part]) -> TextComponent {
    let mut root = TextComponent::text("");
    for p in parts {
        root = root.add_child(rich_part(p));
    }
    root
}

fn rich_part(p: &Part) -> TextComponent {
    let mut c = match &p.kind {
        Kind::Text(t) => TextComponent::text(t),
        Kind::Key(k) => TextComponent::keybind(k),
        Kind::Lang(k, args) => TextComponent::translate(k, args.iter().map(|a| rich(a)).collect()),
    };
    let st = p.style;
    if let Some((r, g, b)) = st.color {
        c = c.color_rgb(RgbColor { r, g, b });
    }
    if st.bold {
        c = c.bold(true);
    }
    if st.italic {
        c = c.italic(true);
    }
    if st.underlined {
        c = c.underlined(true);
    }
    if st.strikethrough {
        c = c.strikethrough(true);
    }
    if st.obfuscated {
        c = c.obfuscated(true);
    }
    if let Some(i) = &p.insertion {
        c = c.insertion(i);
    }
    c = match &p.click {
        Some(Click::Url(u)) => c.click_open_url(u),
        Some(Click::Run(u)) => c.click_run_command(u),
        Some(Click::Suggest(u)) => c.click_suggest_command(u),
        Some(Click::Copy(u)) => c.click_copy_to_clipboard(u),
        Some(Click::File(u)) => c.click_open_file(u),
        Some(Click::Page(n)) => c.click_change_page(*n),
        None => c,
    };
    match p.hover.as_deref() {
        Some(Hover::Text(t)) => c.hover_show_text(rich(t)),
        Some(Hover::Item(i)) => c.hover_show_item(i),
        Some(Hover::Entity { kind, id, name }) => c.hover_show_entity(kind, id, name.as_deref().map(rich)),
        None => c,
    }
}

/// Build a component from `&a` / `§a` codes, `<#rrggbb>` and `<red>` style tags.
fn text(s: &str) -> TextComponent {
    if !s.contains(['&', '§', '<']) {
        return TextComponent::text(s);
    }
    if carve_core::skbee_text::is_rich(s) {
        return rich(&carve_core::skbee_text::parse(s));
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
        late: Vec::new(),
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
    let late = std::mem::take(&mut host.late);
    if !late.is_empty() {
        fire_late(server, late);
    }
    Some(result)
}

/// Fire the events a script's own effects started. Pumpkin fires them while
/// the script still holds the state, so the nested handler had to skip them.
/// They can no longer be cancelled. The depth cap stops an event that starts
/// itself again (`on weather change: set weather to rain`) from looping.
fn fire_late(server: &Server, late: Vec<(EventKind, carve_core::EventCtx)>) {
    use std::sync::atomic::{AtomicU8, Ordering};
    static DEPTH: AtomicU8 = AtomicU8::new(0);
    if DEPTH.load(Ordering::Relaxed) >= 4 {
        plog(ApiLevel::Warn, "events started by scripts nest too deep; skipped");
        return;
    }
    DEPTH.fetch_add(1, Ordering::Relaxed);
    with_state(server, None, |st, host| {
        for (kind, ctx) in late {
            if st.engine.table().has_event(kind) {
                let mut th = TickHost {
                    inner: host,
                    hs: &mut st.hs,
                };
                st.engine.fire(kind, ctx, &mut th);
            }
        }
    });
    DEPTH.fetch_sub(1, Ordering::Relaxed);
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
    /// Events started by this call's own effects, fired once the state is free.
    late: Vec<(EventKind, carve_core::EventCtx)>,
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

/// UUIDs of players whose resource pack finished loading. Set from the
/// resource pack status event; cleared when the player leaves.
static PACKS: Mutex<Vec<String>> = Mutex::new(Vec::new());

struct OnPackStatus;

impl EventHandler<pumpkin_plugin_api::events::PlayerResourcePackStatusEvent> for OnPackStatus {
    fn handle(
        &self,
        server: Server,
        data: EventData<pumpkin_plugin_api::events::PlayerResourcePackStatusEvent>,
    ) -> EventData<pumpkin_plugin_api::events::PlayerResourcePackStatusEvent> {
        let uuid = uuid::to_string(data.player.get_id());
        let status = data.status.to_lowercase();
        let _ = with_state(&server, None, |st, _| {
            st.hs.w4.packs.insert(uuid.clone(), status.clone());
        });
        if let Ok(mut p) = PACKS.lock() {
            p.retain(|u| *u != uuid);
            if status.contains("success") || status.contains("loaded") {
                p.push(uuid);
            }
        }
        data
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

/// The name scripts see for a world. Pumpkin 0.2.2 names every dimension
/// after the level folder (`world`), so the other dimensions go by their
/// dimension id (`minecraft:the_nether`), which `get_world_by_name` accepts.
fn world_name(w: &World) -> String {
    let dim = w.get_dimension();
    if dim == "minecraft:overworld" {
        w.get_name()
    } else {
        dim
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
            let alive = pl.get_health() > 0.0;
            pl.kill();
            // Pumpkin fires the death event while this script holds the state.
            if alive {
                self.late
                    .push((EventKind::Death, carve_core::EventCtx::player(p.clone())));
            }
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
        Some((x, y, z, world_name(&pl.get_world())))
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

    fn held_item_meta(&mut self, p: &PlayerRef) -> Option<carve_core::item_meta::ItemMeta> {
        use carve_core::item_meta::{ItemMeta, plain_from_json};
        let stack = self.get(p)?.get_item_in_hand(Hand::Right)?;
        let meta = ItemMeta {
            name: stack.get_custom_name().map(|n| plain_from_json(&n.to_json())),
            lore: stack
                .get_lore()
                .iter()
                .map(|l| plain_from_json(&l.to_json()))
                .collect(),
            enchants: stack
                .get_enchantments()
                .into_iter()
                .map(|e| (enchant_id(&format!("{:?}", e.enchantment)), e.level))
                .collect(),
            damage: w5_chunk4::damage_of(&stack),
            ..ItemMeta::default()
        };
        (!meta.is_empty()).then_some(meta)
    }

    fn held_component(&mut self, p: &PlayerRef, c: carve_core::skbee_nbt::ItemComp) -> Option<Vec<u8>> {
        let stack = self.get(p)?.get_item_in_hand(Hand::Right)?;
        let want = data_component(c);
        stack
            .get_components()
            .into_iter()
            .find(|v| v.component == want)
            .map(|v| v.value)
    }

    fn set_held_component(
        &mut self,
        p: &PlayerRef,
        c: carve_core::skbee_nbt::ItemComp,
        bytes: Option<Vec<u8>>,
    ) -> bool {
        let Some(pl) = self.get(p) else {
            return false;
        };
        let Some(stack) = pl.get_item_in_hand(Hand::Right) else {
            return false;
        };
        match bytes {
            Some(b) => stack.set_component(data_component(c), &b),
            None => stack.remove_component(data_component(c)),
        }
        pl.set_item_in_hand(Hand::Right, Some(stack));
        true
    }

    fn count_item(&mut self, p: &PlayerRef, id: &str) -> u32 {
        self.get(p)
            .map_or(0, |pl| pl.get_inventory().as_inventory().count_item(id))
    }

    fn biome_at(&mut self, world: &str, x: i32, y: i32, z: i32) -> Option<String> {
        let w = self.server.get_world_by_name(world)?;
        let b = w.get_biome(carve_block_pos(x, y, z));
        Some(enum_name(&format!("{b:?}")))
    }

    fn armor_item(
        &mut self,
        p: &PlayerRef,
        slot: carve_core::w2_expressions_1::ArmorSlot,
    ) -> Option<(String, u32)> {
        use carve_core::w2_expressions_1::ArmorSlot;
        use pumpkin_plugin_api::world::EquipmentSlot;
        let slot = match slot {
            ArmorSlot::Helmet => EquipmentSlot::Head,
            ArmorSlot::Chestplate => EquipmentSlot::Chest,
            ArmorSlot::Leggings => EquipmentSlot::Legs,
            ArmorSlot::Boots => EquipmentSlot::Feet,
        };
        let stack = self.get(p)?.as_entity().as_living()?.get_equipment(slot)?;
        let key = stack.get_registry_key();
        let id = key.strip_prefix("minecraft:").unwrap_or(&key).to_string();
        Some((id, u32::from(stack.get_count())))
    }

    fn block_entity_nbt(&mut self, world: &str, x: i32, y: i32, z: i32) -> Option<Vec<u8>> {
        self.server
            .get_world_by_name(world)?
            .get_block_entity_nbt(carve_block_pos(x, y, z))
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
        let pl_uuid = p.uuid.clone();
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
            PlayerFlag::Burning => e.is_on_fire(),
            PlayerFlag::CustomNameVisible => e.is_custom_name_visible(),
            // Vanilla: fully frozen at 140 ticks.
            PlayerFlag::Frozen => pl.get_freeze_ticks() >= 140,
            PlayerFlag::Gliding => e.is_fall_flying(),
            PlayerFlag::Glowing => e.is_glowing(),
            PlayerFlag::Invisible => e.is_invisible(),
            PlayerFlag::Invulnerable => e.is_invulnerable(),
            PlayerFlag::ResourcePack => PACKS.lock().is_ok_and(|p| p.contains(&pl_uuid)),
        }
    }

    fn player_state(&mut self, p: &PlayerRef, state: carve_core::w2_conditions_3::PlayerState) -> bool {
        use carve_core::w2_conditions_3::PlayerState;
        let Some(pl) = self.get(p) else {
            return false;
        };
        match state {
            PlayerState::Poisoned => pl.has_effect(StatusEffectType::Poison),
            PlayerState::Silent => pl.as_entity().is_silent(),
            PlayerState::Riding => pl.as_entity().get_vehicle().is_some(),
        }
    }

    fn pvp_enabled(&mut self, world: Option<&str>) -> Option<bool> {
        let w = match world {
            Some(name) => self.server.get_world_by_name(name)?,
            None => self.server.get_all_worlds().into_iter().next()?,
        };
        match w.get_game_rule(GameRule::Pvp) {
            GameRuleValue::Bool(b) => Some(b),
            GameRuleValue::Int(n) => Some(n != 0),
        }
    }

    fn block_trait(&mut self, id: &str, t: carve_core::w2_conditions_3::BlockTrait) -> Option<bool> {
        use carve_core::w2_conditions_3::BlockTrait;
        let block = Block::from_name(id)?;
        match t {
            BlockTrait::Solid => Some(block.is_solid),
            BlockTrait::Transparent => {
                Block::get_default_state_for_id(block.id).map(|s| s.opacity == 0 && !block.is_solid)
            }
            BlockTrait::Passable => {
                Block::get_default_state_for_id(block.id).map(|s| s.collision_shapes.is_empty())
            }
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

    fn strike_lightning(&mut self, at: &FxPos, effect_only: bool) {
        if let Some(w) = self.fx_world(at) {
            w.strike_lightning((at.x, at.y, at.z), effect_only);
            self.late.push((
                EventKind::Ext3(carve_core::w2_events_3::Ev3::LightningStrike),
                carve_core::EventCtx::console(),
            ));
        }
    }

    fn explode(&mut self, at: &FxPos, power: f64, fire: bool) {
        if let Some(w) = self.fx_world(at) {
            w.create_explosion((at.x, at.y, at.z), power as f32, fire, ExplosionInteraction::Tnt);
            // Same values as Pumpkin's block explode event: yield 1 / power, no entity.
            let yield_rate = if power > 0.0 { 1.0 / power } else { 1.0 };
            let ev = carve_core::w2_events_2::Ev::Explode;
            let ctx = carve_core::w2_events_2::make_ctx(
                ev,
                None,
                &[yield_rate, at.x.floor(), at.y.floor(), at.z.floor(), -1.0],
                &[],
                &[],
            );
            self.late.push((EventKind::Ext2(ev), ctx));
        }
    }

    fn block_world(&mut self, at: &BlockPos) -> Option<World> {
        self.fx_world(&FxPos {
            x: 0.0,
            y: 0.0,
            z: 0.0,
            world: at.world.clone(),
        })
    }

    fn block_type(&mut self, at: &BlockPos) -> Option<String> {
        let w = self.block_world(at)?;
        let b = w.get_block(WitPos {
            x: at.x,
            y: at.y,
            z: at.z,
        });
        Some(b.name.strip_prefix("minecraft:").unwrap_or(&b.name).to_string())
    }

    fn set_block(&mut self, at: &BlockPos, id: &str) {
        if let Some(w) = self.block_world(at) {
            let flags = BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::NOTIFY_LISTENERS;
            let pos = WitPos {
                x: at.x,
                y: at.y,
                z: at.z,
            };
            if !w.set_block_by_name(pos, &format!("minecraft:{id}"), flags) {
                plog(ApiLevel::Warn, &format!("set block: unknown block type '{id}'"));
            }
        }
    }

    fn break_block(&mut self, at: &BlockPos) {
        if let Some(w) = self.block_world(at) {
            let flags = BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::NOTIFY_LISTENERS;
            w.set_block_by_name(
                WitPos {
                    x: at.x,
                    y: at.y,
                    z: at.z,
                },
                "minecraft:air",
                flags,
            );
        }
    }

    fn targeted_block(&mut self, p: &PlayerRef, max: f64) -> Option<BlockPos> {
        let pl = self.get(p)?;
        let hit = pl.ray_trace_block(max, false)?;
        Some(BlockPos {
            x: hit.pos.x,
            y: hit.pos.y,
            z: hit.pos.z,
            world: Some(world_name(&pl.get_world())),
        })
    }

    fn spawn_particle(&mut self, p: &ParticleFx) {
        let Some(kind) = fx::particle(&p.name) else {
            return;
        };
        // Only the command can force-show a particle to far away players.
        if p.force {
            self.execute_command(&carve_core::skbee_particles_display::particle_command(p));
            return;
        }
        if let Some(w) = self.fx_world(&p.at) {
            let [dx, dy, dz] = p.offsets();
            w.spawn_particle(
                kind,
                (p.at.x, p.at.y, p.at.z),
                (dx, dy, dz),
                p.speed as f32,
                p.count as i32,
            );
        }
    }

    fn spawn_display(&mut self, d: &carve_core::skbee_particles_display::DisplaySpec) {
        let Some(w) = self.fx_world(&d.at) else {
            return;
        };
        if let Err(e) = displays::spawn(&w, d, text) {
            plog(ApiLevel::Warn, &format!("display not spawned: {e}"));
        }
    }

    fn add_effect(&mut self, p: &PlayerRef, effect: &str, ticks: u32, amplifier: u8) {
        let Some(effect_type) = fx::effect(effect) else {
            return;
        };
        if let Some(pl) = self.get(p) {
            pl.add_effect(StatusEffectInstance {
                effect_type,
                duration: ticks,
                amplifier,
                ambient: false,
                show_particles: true,
                show_icon: true,
            });
        }
    }

    fn clear_effects(&mut self, p: &PlayerRef, effect: Option<&str>) {
        let kind = effect.map(fx::effect);
        if let Some(pl) = self.get(p) {
            match kind {
                Some(Some(k)) => pl.remove_effect(k),
                Some(None) => {}
                None => pl.clear_effects(),
            }
        }
    }

    fn open_chest(&mut self, p: &PlayerRef, rows: u8, title: &str) {
        if let Some(pl) = self.get(p) {
            inv::open_chest(pl, &p.uuid, rows, title);
        }
    }

    fn inv_get(&mut self, p: &PlayerRef, current: bool, slot: i64) -> Option<(String, u32)> {
        let pl = self.get(p)?;
        inv::get(pl, &p.uuid, current, slot)
    }

    fn inv_set(&mut self, p: &PlayerRef, current: bool, slot: i64, item: Option<(&str, u32)>) {
        if let Some(pl) = self.get(p) {
            inv::set(pl, &p.uuid, current, slot, item);
        }
    }

    /// Through the inventory API, so the item is there for the rest of the
    /// script (the default `item replace` command only runs after it).
    fn equip(&mut self, p: &PlayerRef, slot: &str, id: &str) {
        let Some(pl) = self.get(p) else {
            return;
        };
        let index = match slot {
            "armor.head" => 39,
            "armor.chest" => 38,
            "armor.legs" => 37,
            "armor.feet" => 36,
            "weapon.offhand" => 40,
            _ => i64::from(pl.get_inventory().get_selected_slot()),
        };
        inv::set(pl, &p.uuid, false, index, Some((id, 1)));
    }

    fn inv_clear(&mut self, p: &PlayerRef, current: bool) {
        if let Some(pl) = self.get(p) {
            inv::clear(pl, &p.uuid, current);
        }
    }

    fn inv_remove(&mut self, p: &PlayerRef, id: &str, count: u32) -> u32 {
        self.get(p).map_or(0, |pl| inv::remove(pl, id, count))
    }

    /// Pumpkin's plugin API v0.1 has no call to close a window (v0.2 does).
    fn close_inventory(&mut self, _p: &PlayerRef) {
        plog(
            ApiLevel::Warn,
            "close inventory: this Pumpkin plugin API has no way to close a window yet",
        );
    }

    fn has_effect(&mut self, p: &PlayerRef, effect: &str) -> bool {
        let Some(kind) = fx::effect(effect) else {
            return false;
        };
        self.get(p).is_some_and(|pl| pl.has_effect(kind))
    }

    fn advancement(&mut self, p: &PlayerRef, id: &str, criterion: Option<&str>, grant: bool) -> bool {
        let Some(pl) = self.get(p) else {
            return false;
        };
        match (criterion, grant) {
            (None, true) => pl.award_advancement(id),
            (None, false) => pl.revoke_advancement(id),
            (Some(c), true) => pl.award_advancement_criterion(id, c),
            (Some(c), false) => pl.revoke_advancement_criterion(id, c),
        }
    }

    fn has_advancement(&mut self, p: &PlayerRef, id: &str) -> bool {
        self.get(p).is_some_and(|pl| pl.has_advancement(id))
    }

    fn register_recipe(&mut self, r: &RecipeDef) -> Result<(), String> {
        use pumpkin_plugin_api::recipe::{
            CookingRecipeBuilder, Ingredient as Ing, ShapedRecipeBuilder, ShapelessRecipeBuilder,
        };
        let ing = |i: &RIngredient| match i {
            RIngredient::Item(s) => Ing::item(s),
            RIngredient::Tag(s) => Ing::tag(s),
        };
        let out = pumpkin_plugin_api::ItemStack::new(&r.result, r.count as u8);
        let res = match &r.kind {
            RecipeKind::Shaped { rows, key } => {
                let mut b =
                    ShapedRecipeBuilder::new(r.id.clone(), out).pattern(rows.iter().map(String::as_str));
                for (c, i) in key {
                    b = b.key(*c, ing(i));
                }
                self.server.register_recipe(b)
            }
            RecipeKind::Shapeless(list) => {
                let mut b = ShapelessRecipeBuilder::new(r.id.clone(), out);
                for i in list {
                    b = b.ingredient(ing(i));
                }
                self.server.register_recipe(b)
            }
            RecipeKind::Cooking {
                kind,
                ingredient,
                experience,
                ticks,
            } => {
                let i = ing(ingredient);
                let b = match kind {
                    CookKind::Smelting => CookingRecipeBuilder::smelting(r.id.clone(), i, out),
                    CookKind::Blasting => CookingRecipeBuilder::blasting(r.id.clone(), i, out),
                    CookKind::Smoking => CookingRecipeBuilder::smoking(r.id.clone(), i, out),
                    CookKind::Campfire => CookingRecipeBuilder::campfire(r.id.clone(), i, out),
                };
                self.server
                    .register_recipe(b.cooking_time(*ticks).experience(*experience))
            }
        };
        res.map_err(|e| e.to_string())
    }

    fn statistic(&mut self, p: &PlayerRef, category: &str, name: &str) -> Option<i64> {
        if category != "custom" {
            return None;
        }
        let stat = custom_stats::custom_statistic(name)?;
        self.get(p).map(|pl| i64::from(pl.get_custom_statistic(stat)))
    }

    fn set_statistic(&mut self, p: &PlayerRef, category: &str, name: &str, value: i64) -> bool {
        if category != "custom" {
            self.log(
                LogLevel::Warn,
                "only custom statistics (custom:jump, custom:play_time, ...) can be changed on this server",
            );
            return false;
        }
        let Some(stat) = custom_stats::custom_statistic(name) else {
            self.log(LogLevel::Warn, &format!("unknown custom statistic '{name}'"));
            return false;
        };
        let Some(pl) = self.get(p) else {
            return false;
        };
        pl.set_custom_statistic(stat, value.clamp(0, i64::from(i32::MAX)) as i32);
        pl.send_stats();
        true
    }

    fn set_fire(&mut self, p: &PlayerRef, ticks: i32) {
        if let Some(pl) = self.get(p) {
            let e = pl.as_entity();
            if ticks > 0 {
                e.set_fire_ticks(ticks);
            } else {
                e.set_fire_ticks(0);
                e.set_on_fire(false);
            }
        }
    }

    fn world_time(&mut self, world: Option<&str>) -> Option<i64> {
        let w = self.named_world(world)?;
        i64::try_from(w.get_time_of_day()).ok()
    }

    fn set_world_time(&mut self, world: Option<&str>, ticks: i64) {
        if let Some(w) = self.named_world(world) {
            w.set_time_of_day(ticks.max(0) as u64);
        }
    }

    fn weather(&mut self, world: Option<&str>) -> Option<Weather> {
        let w = self.named_world(world)?;
        Some(Weather::from_flags(w.is_raining(), w.is_thundering()))
    }

    fn set_weather(&mut self, world: Option<&str>, to: Weather, ticks: Option<u32>) {
        let Some(w) = self.named_world(world) else {
            if ticks.is_some() {
                self.execute_command(&to.command(ticks));
            }
            return;
        };
        let was_raining = w.is_raining();
        // The API has no length, so a length goes through the command.
        if ticks.is_some() {
            self.execute_command(&to.command(ticks));
        } else {
            w.set_raining(to != Weather::Clear);
            w.set_thundering(to == Weather::Thunder);
        }
        // Pumpkin's own weather change event comes while this script holds
        // the state, so it is fired again here, after the script.
        let rain = to != Weather::Clear;
        if rain != was_raining {
            use carve_core::w2_events_4::{ExtEvent as E4, ctx};
            let name = if rain { "rain" } else { "clear" };
            let to_kind = if rain {
                E4::WeatherToRain
            } else {
                E4::WeatherToClear
            };
            for k in [E4::WeatherChange, to_kind] {
                let vals = vec![Value::World(w.get_name()), Value::Text(name.into())];
                self.late.push((EventKind::Ext4(k), ctx(None, vals)));
            }
        }
    }

    fn game_rule(&mut self, world: Option<&str>, rule: &str) -> Option<String> {
        let rule = rule_enum(rule)?;
        let w = self.named_world(world)?;
        match w.get_game_rule(rule) {
            GameRuleValue::Bool(b) => Some(b.to_string()),
            GameRuleValue::Int(i) => Some(i.to_string()),
        }
    }

    fn set_game_rule(&mut self, world: Option<&str>, rule: &str, value: &str) {
        let (Some(r), Some(w)) = (rule_enum(rule), self.named_world(world)) else {
            return;
        };
        match value {
            "true" => w.set_game_rule_bool(r, true),
            "false" => w.set_game_rule_bool(r, false),
            n => match n.parse::<i32>() {
                Ok(i) => w.set_game_rule_int(r, i),
                Err(_) => plog(ApiLevel::Warn, &format!("bad game rule value '{n}'")),
            },
        }
    }

    fn set_difficulty(&mut self, _world: Option<&str>, to: &str) {
        // There is no difficulty API yet; the command changes the default world.
        self.execute_command(&format!("difficulty {to}"));
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

    fn set_flight(&mut self, p: &PlayerRef, on: bool) {
        if let Some(pl) = self.get(p) {
            pl.set_allow_flight(on);
            pl.set_flying(on);
        }
        if let Ok(mut f) = FLYING.lock() {
            f.retain(|u| *u != p.uuid);
            if on {
                f.push(p.uuid.clone());
            }
        }
    }

    fn set_invulnerable(&mut self, p: &PlayerRef, on: bool) {
        if let Some(pl) = self.get(p) {
            pl.set_invulnerable(on);
        }
    }

    fn set_invisible(&mut self, p: &PlayerRef, on: bool) {
        if let Some(pl) = self.get(p) {
            pl.as_entity().set_invisible(on);
        }
    }

    fn apply_knockback(&mut self, p: &PlayerRef, strength: f64, x: f64, z: f64) {
        if let Some(pl) = self.get(p) {
            pl.apply_knockback(strength, x, z);
        }
    }

    fn set_rotation(&mut self, p: &PlayerRef, yaw: f32, pitch: f32) {
        if let Some(pl) = self.get(p) {
            let world = pl.get_world();
            pl.teleport(pl.get_position(), Some(yaw), Some(pitch), world);
        }
    }

    fn open_book(&mut self, p: &PlayerRef, off_hand: bool) {
        if let Some(pl) = self.get(p) {
            pl.open_book(if off_hand { Hand::Left } else { Hand::Right });
        }
    }

    fn set_list_order(&mut self, p: &PlayerRef, order: i32) {
        if let Some(pl) = self.get(p) {
            pl.set_tab_list_order(order);
        }
    }

    fn damage(&mut self, p: &PlayerRef, amount: f64) {
        if let Some(pl) = self.get(p) {
            pl.damage(amount as f32, DamageType::Generic);
        }
    }

    fn is_op(&mut self, p: &PlayerRef) -> bool {
        self.get(p)
            .is_some_and(|pl| !matches!(pl.get_permission_level(), PermissionLevel::Zero))
    }

    fn display_name(&mut self, p: &PlayerRef) -> Option<String> {
        self.get(p).map(|pl| pl.get_display_name().get_text())
    }

    fn set_display_name(&mut self, p: &PlayerRef, name: Option<&str>) {
        if let Some(pl) = self.get(p) {
            // Resetting means going back to the account name.
            let t = text(name.unwrap_or(&p.name));
            pl.set_display_name(t);
        }
    }

    fn tab_list_name(&mut self, p: &PlayerRef) -> Option<String> {
        self.get(p)?.get_tab_list_name().map(|t| t.get_text())
    }

    fn set_tab_list_name(&mut self, p: &PlayerRef, name: Option<&str>) {
        if let Some(pl) = self.get(p) {
            let t = name.map(text);
            pl.set_tab_list_name(t);
        }
    }

    fn board_show(&mut self, p: &PlayerRef, title: &str, lines: &[String]) {
        use pumpkin_plugin_api::wit::pumpkin::plugin::scoreboard::{DisplaySlot, NumberFormat, RenderType};
        const BOARD: &str = "carve_board";
        let Some(pl) = self.get(p) else {
            return;
        };
        let Some(java) = pl.as_java() else {
            return;
        };
        let sb = java.get_scoreboard();
        // Redraw from scratch: the sidebar has no "set line" call.
        sb.remove_objective(BOARD);
        sb.add_objective(BOARD, text(title), RenderType::Integer, Some(NumberFormat::Blank));
        sb.set_display_slot(DisplaySlot::Sidebar, BOARD);
        let n = lines.len();
        for (i, line) in lines.iter().enumerate() {
            // Each holder name must be unique, so pad with invisible resets.
            let mut name = carve_core::colors::colorize(line);
            name.push_str(&"\u{a7}r".repeat(i + 1));
            sb.update_score(&name, BOARD, (n - i) as i32, Some(NumberFormat::Blank));
        }
    }

    fn board_clear(&mut self, p: &PlayerRef) {
        if let Some(pl) = self.get(p)
            && let Some(java) = pl.as_java()
        {
            let sb = java.get_scoreboard();
            sb.remove_objective("carve_board");
        }
    }

    fn player_info(&mut self, p: &PlayerRef, info: Info) -> Option<String> {
        let pl = self.get(p)?;
        Some(match info {
            Info::Pose => match pl.as_entity().get_pose() {
                EntityPose::Standing => "standing",
                EntityPose::FallFlying => "fall flying",
                EntityPose::Sleeping => "sleeping",
                EntityPose::Swimming => "swimming",
                EntityPose::SpinAttack => "spin attack",
                EntityPose::Crouching => "crouching",
                EntityPose::LongJumping => "long jumping",
                EntityPose::Dying => "dying",
                _ => "other",
            }
            .to_string(),
            Info::Locale => pl.get_locale(),
            Info::Brand => pl.as_java()?.get_brand(),
            Info::ViewDistance => pl.as_java()?.get_settings().view_distance.to_string(),
            Info::MainHand => match pl.as_java()?.get_settings().main_hand {
                Hand::Left => "left",
                Hand::Right => "right",
            }
            .to_string(),
        })
    }

    fn ping(&mut self, p: &PlayerRef) -> Option<f64> {
        self.get(p).map(|pl| f64::from(pl.get_ping()))
    }

    fn ip(&mut self, p: &PlayerRef) -> Option<String> {
        // Pumpkin gives "ip:port"; Skript's ip has no port.
        self.get(p).map(|pl| {
            let ip = pl.get_ip();
            match ip.rsplit_once(':') {
                Some((host, port)) if port.bytes().all(|b| b.is_ascii_digit()) => {
                    host.trim_start_matches('[').trim_end_matches(']').to_string()
                }
                _ => ip,
            }
        })
    }

    fn player_count(&mut self) -> f64 {
        f64::from(self.server.get_player_count())
    }

    fn max_players(&mut self) -> Option<f64> {
        Some(f64::from(self.server.get_max_players()))
    }

    fn motd(&mut self) -> Option<String> {
        Some(self.server.get_motd())
    }

    fn server_version(&mut self) -> Option<String> {
        Some(self.server.get_sys_info().pumpkin_version)
    }

    fn is_whitelisted(&mut self, name: &str) -> bool {
        self.server
            .get_whitelist_manager()
            .list_entries()
            .iter()
            .any(|e| e.name.eq_ignore_ascii_case(name))
    }

    fn set_whitelisted(&mut self, name: &str, uuid_str: Option<&str>, on: bool) {
        // Online players have a UUID, so the manager can be used. Offline
        // names need the profile lookup that only the command does.
        if let Some(id) = uuid_str.and_then(uuid::parse) {
            let m = self.server.get_whitelist_manager();
            if on {
                m.add_player(name, id);
            } else {
                m.remove_player(id);
            }
            return;
        }
        self.deferred.push((
            None,
            format!("whitelist {} {name}", if on { "add" } else { "remove" }),
        ));
    }

    fn world_names(&mut self) -> Vec<String> {
        self.server.get_all_worlds().iter().map(world_name).collect()
    }

    fn broadcast_world(&mut self, world: &str, msg: &str) {
        let Some(w) = self.server.get_world_by_name(world) else {
            plog(ApiLevel::Warn, &format!("broadcast: no world named '{world}'"));
            return;
        };
        for p in self.server.get_players_in_world(w) {
            p.send_system_message(text(msg), false);
        }
    }

    fn world_spawn(&mut self, world: &str) -> Option<(f64, f64, f64)> {
        let w = self.server.get_world_by_name(world)?;
        let s = w.get_spawn_location().pos;
        Some((f64::from(s.x) + 0.5, f64::from(s.y), f64::from(s.z) + 0.5))
    }

    fn border_size(&mut self, world: &str) -> Option<f64> {
        Some(
            self.server
                .get_world_by_name(world)?
                .get_world_border()
                .get_diameter(),
        )
    }

    fn set_border_size(&mut self, world: &str, size: f64, seconds: f64) {
        let Some(w) = self.server.get_world_by_name(world) else {
            plog(ApiLevel::Warn, &format!("world border: no world named '{world}'"));
            return;
        };
        let b = w.get_world_border();
        if seconds > 0.0 {
            b.set_size_transition(size, seconds as u64);
        } else {
            b.set_size(size);
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

    fn compass_target(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64, Option<String>)> {
        let pl = self.get(p)?;
        let (x, y, z) = pl.get_compass_target();
        Some((x, y, z, Some(world_name(&pl.get_world()))))
    }

    fn set_compass_target(&mut self, p: &PlayerRef, x: f64, y: f64, z: f64, _world: Option<&str>) {
        if let Some(pl) = self.get(p) {
            pl.set_compass_target((x, y, z));
        }
    }

    fn save_world(&mut self) {
        if let Err(e) = self.server.save_all() {
            plog(ApiLevel::Warn, &format!("save world failed: {e}"));
        }
    }

    fn set_sprinting(&mut self, p: &PlayerRef, on: bool) {
        if let Some(pl) = self.get(p) {
            pl.as_entity().set_sprinting(on);
        }
    }

    fn eject_passengers(&mut self, p: &PlayerRef) {
        if let Some(pl) = self.get(p) {
            pl.as_entity().eject_passengers();
        }
    }

    fn set_name_visible(&mut self, p: &PlayerRef, visible: bool) {
        if let Some(pl) = self.get(p) {
            pl.as_entity().set_custom_name_visible(visible);
        }
    }

    fn send_resource_pack(&mut self, p: &PlayerRef, url: &str, hash: &str, id: &str) {
        let Some(id) = uuid::parse(id) else { return };
        if let Some(java) = self.get(p).and_then(Player::as_java) {
            java.send_resource_pack(JavaResourcePack {
                id,
                url: url.to_string(),
                hash: hash.to_string(),
                forced: false,
                prompt_message: None,
            });
        }
    }

    fn remaining_air(&mut self, p: &PlayerRef) -> Option<i64> {
        self.get(p)
            .map(|pl| i64::from(pl.as_entity().get_remaining_air()))
    }

    fn set_remaining_air(&mut self, p: &PlayerRef, ticks: i64) {
        if let Some(pl) = self.get(p) {
            let e = pl.as_entity();
            let max = i64::from(e.get_max_air());
            e.set_remaining_air(ticks.clamp(0, max.max(1)) as i32);
        }
    }

    fn respawn_location(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64)> {
        self.get(p)?.get_respawn_location()
    }

    fn set_respawn_location(&mut self, p: &PlayerRef, x: f64, y: f64, z: f64) {
        if let Some(pl) = self.get(p) {
            pl.set_respawn_location((x, y, z));
        }
    }

    fn sea_level(&mut self, world: &str) -> Option<i64> {
        self.server
            .get_world_by_name(world)
            .map(|w| i64::from(w.get_sea_level()))
    }

    fn simulation_distance(&mut self) -> Option<i64> {
        Some(i64::from(self.server.get_simulation_distance()))
    }

    fn w4c1_check(&mut self, c: carve_core::w4_conditions_1::Check, v: &Value) -> Option<bool> {
        self.w4c1_check_impl(c, v)
    }

    fn w5c2_query(&mut self, q: carve_core::w5_chunk2::Query<'_>) -> Option<Value> {
        self.w5c2_query_impl(q)
    }

    fn default_world(&mut self) -> Option<String> {
        self.server.get_all_worlds().first().map(world_name)
    }

    fn w5c5(&mut self, q: carve_core::w5_chunk5::Q<'_>) -> Value {
        self.w5c5_impl(q)
    }

    fn w5c3(&mut self, q: carve_core::w5_chunk3::Q<'_>) -> Option<Value> {
        self.w5c3_impl(q)
    }

    fn w5c1(&mut self, op: carve_core::w5_chunk1::HostOp<'_>) -> Option<bool> {
        self.w5c1_impl(op)
    }

    fn read_data_file(&mut self, path: &str) -> Option<Vec<u8>> {
        w5_chunk1::read_data_file(path)
    }

    fn w5c4(&mut self, q: carve_core::w5_chunk4::Q<'_>) -> Option<Value> {
        self.w5c4_impl(q)
    }

    fn w2e7_query(&mut self, q: Query<'_>) -> Option<Value> {
        let num = |n: f64| Some(Value::Num(n));
        match q {
            Query::Tps => num(self.server.get_tps()),
            Query::Mspt => num(self.server.get_mspt()),
            // The Minecraft version Pumpkin was built for; the plugin API has no call for it.
            Query::MinecraftVersion => Some(Value::Text("26.3".into())),
            Query::ServerViewDistance => num(f64::from(self.server.get_view_distance())),
            Query::PlayTicks(p) => {
                let pl = self.get(p)?;
                num(f64::from(pl.get_custom_statistic(CustomStatistic::PlayTime)))
            }
            Query::LivedTicks(p) => num(f64::from(self.get(p)?.as_entity().get_ticks_lived())),
            Query::Yaw(p) => num(f64::from(self.get(p)?.get_yaw())),
            Query::Pitch(p) => num(f64::from(self.get(p)?.get_pitch())),
            Query::Environment(name) => {
                let w = self.server.get_world_by_name(name)?;
                Some(Value::Text(w.get_dimension()))
            }
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

impl WasmHost<'_> {
    /// A world by name; with none named, the world of the first player
    /// online, else the one called `world`.
    fn named_world(&mut self, name: Option<&str>) -> Option<World> {
        let found = match name {
            Some(n) => self.server.get_world_by_name(n),
            None => self
                .server
                .get_all_players()
                .first()
                .map(Player::get_world)
                .or_else(|| self.server.get_world_by_name("world")),
        };
        if found.is_none() {
            plog(
                ApiLevel::Warn,
                &format!("no world found: {}", name.unwrap_or("default")),
            );
        }
        found
    }
}

impl WasmHost<'_> {
    /// The world a position is in; with no world named, the world of the first
    /// player online, else the one called `world`.
    fn fx_world(&mut self, at: &FxPos) -> Option<World> {
        let found = match at.world.as_deref() {
            Some(name) => self.server.get_world_by_name(name),
            None => self
                .server
                .get_all_players()
                .first()
                .map(Player::get_world)
                .or_else(|| self.server.get_world_by_name("world")),
        };
        if found.is_none() {
            plog(ApiLevel::Warn, "no world found for this effect");
        }
        found
    }
}

/// The `Host` the engine sees: game calls go to `WasmHost`, waits and the
/// clock use the plugin's own tick counter.
struct TickHost<'a, 'b> {
    inner: &'a mut WasmHost<'b>,
    hs: &'a mut HostState,
}

/// `Biome::DarkForest` (the Debug form of a WIT enum) as `dark_forest`.
fn enum_name(debug: &str) -> String {
    let last = debug.rsplit("::").next().unwrap_or(debug);
    carve_core::w2_expressions_1::snake(last)
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
        held_item_meta(p: &PlayerRef) -> Option<carve_core::item_meta::ItemMeta>;
        count_item(p: &PlayerRef, id: &str) -> u32;
        held_component(p: &PlayerRef, c: carve_core::skbee_nbt::ItemComp) -> Option<Vec<u8>>;
        set_held_component(p: &PlayerRef, c: carve_core::skbee_nbt::ItemComp, bytes: Option<Vec<u8>>) -> bool;
        health(p: &PlayerRef) -> Option<f64>;
        gamemode(p: &PlayerRef) -> Option<String>;
        set_gamemode(p: &PlayerRef, mode: &str);
        player_flag(p: &PlayerRef, flag: PlayerFlag) -> bool;
        player_state(p: &PlayerRef, state: carve_core::w2_conditions_3::PlayerState) -> bool;
        pvp_enabled(world: Option<&str>) -> Option<bool>;
        block_trait(id: &str, t: carve_core::w2_conditions_3::BlockTrait) -> Option<bool>;
        xp_get(p: &PlayerRef, field: carve_core::xp_level::XpField) -> Option<f64>;
        xp_set(p: &PlayerRef, field: carve_core::xp_level::XpField, value: f64);
        vital(p: &PlayerRef, v: Vital) -> Option<f64>;
        set_vital(p: &PlayerRef, v: Vital, value: f64);
        damage(p: &PlayerRef, amount: f64);
        is_op(p: &PlayerRef) -> bool;
        display_name(p: &PlayerRef) -> Option<String>;
        set_display_name(p: &PlayerRef, name: Option<&str>);
        tab_list_name(p: &PlayerRef) -> Option<String>;
        set_tab_list_name(p: &PlayerRef, name: Option<&str>);
        ping(p: &PlayerRef) -> Option<f64>;
        ip(p: &PlayerRef) -> Option<String>;
        player_info(p: &PlayerRef, info: Info) -> Option<String>;
        strike_lightning(at: &FxPos, effect_only: bool);
        explode(at: &FxPos, power: f64, fire: bool);
        spawn_particle(p: &ParticleFx);
        spawn_display(d: &carve_core::skbee_particles_display::DisplaySpec);
        add_effect(p: &PlayerRef, effect: &str, ticks: u32, amplifier: u8);
        clear_effects(p: &PlayerRef, effect: Option<&str>);
        has_effect(p: &PlayerRef, effect: &str) -> bool;
        set_fire(p: &PlayerRef, ticks: i32);
        block_type(at: &BlockPos) -> Option<String>;
        set_block(at: &BlockPos, id: &str);
        break_block(at: &BlockPos);
        targeted_block(p: &PlayerRef, max: f64) -> Option<BlockPos>;
        open_chest(p: &PlayerRef, rows: u8, title: &str);
        inv_get(p: &PlayerRef, current: bool, slot: i64) -> Option<(String, u32)>;
        inv_set(p: &PlayerRef, current: bool, slot: i64, item: Option<(&str, u32)>);
        equip(p: &PlayerRef, slot: &str, id: &str);
        inv_clear(p: &PlayerRef, current: bool);
        inv_remove(p: &PlayerRef, id: &str, count: u32) -> u32;
        close_inventory(p: &PlayerRef);
        world_time(world: Option<&str>) -> Option<i64>;
        set_world_time(world: Option<&str>, ticks: i64);
        weather(world: Option<&str>) -> Option<Weather>;
        set_weather(world: Option<&str>, to: Weather, ticks: Option<u32>);
        game_rule(world: Option<&str>, rule: &str) -> Option<String>;
        set_game_rule(world: Option<&str>, rule: &str, value: &str);
        set_difficulty(world: Option<&str>, to: &str);
        board_show(p: &PlayerRef, title: &str, lines: &[String]);
        board_clear(p: &PlayerRef);
        spawn_entity(kind: &str, at: &FxPos) -> Option<carve_core::EntityRef>;
        entity_kill(e: &carve_core::EntityRef);
        entity_remove(e: &carve_core::EntityRef);
        entity_location(e: &carve_core::EntityRef) -> Option<(f64, f64, f64, String)>;
        entity_name(e: &carve_core::EntityRef) -> Option<String>;
        entity_health(e: &carve_core::EntityRef) -> Option<f64>;
        set_entity_health(e: &carve_core::EntityRef, health: f64);
        set_entity_name(e: &carve_core::EntityRef, name: Option<&str>);
        entities_near(at: &FxPos, radius: f64, kind: Option<&str>) -> Vec<carve_core::EntityRef>;
        entity_velocity(e: &carve_core::EntityRef) -> Option<(f64, f64, f64)>;
        set_entity_velocity(e: &carve_core::EntityRef, v: (f64, f64, f64));
        entity_teleport(e: &carve_core::EntityRef, x: f64, y: f64, z: f64, world: Option<&str>);
        open_inventory(p: &PlayerRef, v: &carve_core::gaps::InvVal);
        menu_update(id: u64, slot: u32, item: &Value);
        menu_set(p: &PlayerRef, slot: u32, item: &Value);
        set_held_item_meta(p: &PlayerRef, m: &carve_core::item_meta::ItemMeta) -> bool;
        inv_items(p: &PlayerRef) -> Vec<Value>;
        entity_rotation(e: &carve_core::EntityRef) -> Option<(f64, f64)>;
        advancement(p: &PlayerRef, id: &str, criterion: Option<&str>, grant: bool) -> bool;
        has_advancement(p: &PlayerRef, id: &str) -> bool;
        register_recipe(r: &RecipeDef) -> Result<(), String>;
        statistic(p: &PlayerRef, category: &str, name: &str) -> Option<i64>;
        set_statistic(p: &PlayerRef, category: &str, name: &str, value: i64) -> bool;
        player_count() -> f64;
        max_players() -> Option<f64>;
        motd() -> Option<String>;
        server_version() -> Option<String>;
        is_whitelisted(name: &str) -> bool;
        set_whitelisted(name: &str, uuid: Option<&str>, on: bool);
        world_names() -> Vec<String>;
        broadcast_world(world: &str, msg: &str);
        world_spawn(world: &str) -> Option<(f64, f64, f64)>;
        border_size(world: &str) -> Option<f64>;
        set_border_size(world: &str, size: f64, seconds: f64);
        transfer(p: &PlayerRef, host: &str, port: u16);
        respawn(p: &PlayerRef);
        hide_player(viewer: &PlayerRef, target: &PlayerRef);
        show_player(viewer: &PlayerRef, target: &PlayerRef);
        set_flight(p: &PlayerRef, on: bool);
        set_invulnerable(p: &PlayerRef, on: bool);
        set_invisible(p: &PlayerRef, on: bool);
        apply_knockback(p: &PlayerRef, strength: f64, x: f64, z: f64);
        set_rotation(p: &PlayerRef, yaw: f32, pitch: f32);
        open_book(p: &PlayerRef, off_hand: bool);
        save_world();
        set_sprinting(p: &PlayerRef, on: bool);
        eject_passengers(p: &PlayerRef);
        set_name_visible(p: &PlayerRef, visible: bool);
        send_resource_pack(p: &PlayerRef, url: &str, hash: &str, id: &str);
        remaining_air(p: &PlayerRef) -> Option<i64>;
        set_remaining_air(p: &PlayerRef, ticks: i64);
        respawn_location(p: &PlayerRef) -> Option<(f64, f64, f64)>;
        set_respawn_location(p: &PlayerRef, x: f64, y: f64, z: f64);
        sea_level(world: &str) -> Option<i64>;
        simulation_distance() -> Option<i64>;
    }
    delegate! {
        biome_at(world: &str, x: i32, y: i32, z: i32) -> Option<String>;
        armor_item(p: &PlayerRef, slot: carve_core::w2_expressions_1::ArmorSlot) -> Option<(String, u32)>;
        block_entity_nbt(world: &str, x: i32, y: i32, z: i32) -> Option<Vec<u8>>;
        compass_target(p: &PlayerRef) -> Option<(f64, f64, f64, Option<String>)>;
        set_compass_target(p: &PlayerRef, x: f64, y: f64, z: f64, world: Option<&str>);
    }

    fn pstat(&mut self, p: &PlayerRef, s: carve_core::w2_expressions_3::PStat) -> Option<f64> {
        self.inner.pstat_impl(p, s)
    }

    fn set_pstat(&mut self, p: &PlayerRef, s: carve_core::w2_expressions_3::PStat, v: f64) {
        self.inner.set_pstat_impl(p, s, v);
    }

    fn eye_location(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        self.inner.eye_location_impl(p)
    }

    fn first_empty_slot(&mut self, p: &PlayerRef) -> Option<u32> {
        self.inner.first_empty_slot_impl(p)
    }

    fn can_see(&mut self, viewer: &PlayerRef, other: &PlayerRef) -> bool {
        self.inner.can_see_impl(viewer, other)
    }

    fn system_memory(&mut self) -> Option<(u64, u64)> {
        self.inner.system_memory_impl()
    }

    /// Cached for the tick: every script of a hit asks for the same entities.
    fn entity_by_id(&mut self, id: i32) -> Option<carve_core::EntityRef> {
        let w4 = &mut self.hs.w4;
        if w4.ent_tick != self.hs.tick {
            w4.ent_tick = self.hs.tick;
            w4.ent_ids.clear();
        }
        if let Some((_, r)) = w4.ent_ids.iter().find(|(i, _)| *i == id) {
            return r.clone();
        }
        let r = self.inner.entity_by_id(id);
        self.hs.w4.ent_ids.push((id, r.clone()));
        r
    }

    fn top_block_y(&mut self, world: Option<&str>, x: i32, z: i32) -> Option<i32> {
        self.inner.top_block_y_impl(world, x, z)
    }

    /// Pumpkin has one difficulty for the whole server.
    fn difficulty(&mut self, _world: Option<&str>) -> Option<String> {
        Some(enum_name(&format!("{:?}", self.inner.server.get_difficulty())))
    }

    /// `get-chunk` only returns chunks that are already loaded.
    fn chunk_loaded(&mut self, world: &str, x: i32, z: i32) -> Option<bool> {
        let w = self.inner.server.get_world_by_name(world)?;
        Some(w.get_chunk(x, z).is_some())
    }

    delegate! {
        item_cooldown(p: &PlayerRef, item: &str) -> Option<i64>;
        set_item_cooldown(p: &PlayerRef, item: &str, ticks: i64);
        solid_block_y(world: &str, x: i32, z: i32, highest: bool) -> Option<i32>;
        light_level(world: &str, x: i32, y: i32, z: i32, kind: u8) -> Option<u8>;
        set_list_order(p: &PlayerRef, order: i32);
    }

    fn last_death_location(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        self.hs.w4.deaths.get(&p.uuid).cloned()
    }

    fn last_damage(&mut self, p: &PlayerRef) -> Option<carve_core::w2_expressions_4::DamageInfo> {
        self.hs.w4.hits.get(&p.uuid).cloned()
    }

    fn join_message(&mut self) -> Option<String> {
        self.hs.w4.join_msg.clone()
    }

    fn set_join_message(&mut self, text: &str) {
        self.hs.w4.join_msg = Some(text.to_string());
        self.hs.w4.join_set = Some(text.to_string());
    }

    fn resource_pack_status(&mut self, p: &PlayerRef) -> Option<String> {
        self.hs.w4.packs.get(&p.uuid).cloned()
    }

    fn w2e7_query(&mut self, q: Query<'_>) -> Option<Value> {
        self.inner.w2e7_query(q)
    }

    fn w4c1_check(&mut self, c: carve_core::w4_conditions_1::Check, v: &Value) -> Option<bool> {
        self.inner.w4c1_check(c, v)
    }

    fn w5c2_query(&mut self, q: carve_core::w5_chunk2::Query<'_>) -> Option<Value> {
        self.inner.w5c2_query(q)
    }

    fn default_world(&mut self) -> Option<String> {
        self.inner.default_world()
    }

    fn w5c5(&mut self, q: carve_core::w5_chunk5::Q<'_>) -> Value {
        self.inner.w5c5(q)
    }

    fn w5c3(&mut self, q: carve_core::w5_chunk3::Q<'_>) -> Option<Value> {
        self.inner.w5c3(q)
    }

    fn c6_get(&mut self, q: carve_core::w5_chunk6::Get<'_>) -> Option<Value> {
        self.inner.c6_get_impl(q)
    }

    fn c6_set(&mut self, s: carve_core::w5_chunk6::Set<'_>) -> bool {
        self.inner.c6_set_impl(s)
    }

    fn c6_block_state(&mut self, at: &BlockPos) -> Option<carve_core::w5_chunk6::BlockState> {
        self.inner.c6_block_state_impl(at)
    }

    fn last_display(&mut self) -> Option<carve_core::EntityRef> {
        displays::take_last()
    }

    fn w5c1(&mut self, op: carve_core::w5_chunk1::HostOp<'_>) -> Option<bool> {
        self.inner.w5c1(op)
    }

    fn read_data_file(&mut self, path: &str) -> Option<Vec<u8>> {
        self.inner.read_data_file(path)
    }

    fn w5c4(&mut self, q: carve_core::w5_chunk4::Q<'_>) -> Option<Value> {
        self.inner.w5c4(q)
    }

    sb::sb_host_methods!();

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

    fn save(&self, json: &str) {
        self.save_to(VARS_FILE, json);
    }

    fn save_to(&self, file: &str, json: &str) {
        let path = format!("{}/{file}", self.data_dir);
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

// ---------- events ----------

struct OnEvent<E>(EventKind, std::marker::PhantomData<fn() -> E>);

trait EventAccess: FromIntoEvent {
    fn ctx(data: &EventData<Self>) -> EventCtx;
    fn set_cancelled(data: &mut EventData<Self>);
    /// The uuid of a player who just left, for events that say so.
    fn left(_data: &EventData<Self>) -> Option<String> {
        None
    }
    /// Called for every event, even when no script listens.
    fn observe(_data: &EventData<Self>, _hs: &mut HostState) {}
    /// A message a script set during the event (`set join message to`).
    fn set_message(_data: &mut EventData<Self>, _msg: &str) {}
    /// True when this server event is not the script event (no script runs).
    fn skip(_data: &EventData<Self>) -> bool {
        false
    }
    /// Values the script changed (`FireOutcome::set`), such as cancelled drops.
    fn apply_out(_data: &mut EventData<Self>, _out: &[(u8, Value)]) {}
}

impl EventAccess for PlayerJoinEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        EventCtx::player(player_ref(&d.player))
    }
    fn observe(d: &EventData<Self>, hs: &mut HostState) {
        hs.w4.join_msg = Some(d.join_message.get_text());
        hs.w4.join_set = None;
    }
    fn set_message(d: &mut EventData<Self>, msg: &str) {
        d.join_message = text(msg);
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerLeaveEvent {
    fn left(d: &EventData<Self>) -> Option<String> {
        Some(uuid::to_string(d.player.get_id()))
    }
    fn ctx(d: &EventData<Self>) -> EventCtx {
        if let Ok(mut p) = PACKS.lock() {
            let u = uuid::to_string(d.player.get_id());
            p.retain(|x| *x != u);
        }
        let mut c = EventCtx::player(player_ref(&d.player));
        c.message = Some(d.leave_message.get_text());
        c
    }
    fn set_message(d: &mut EventData<Self>, msg: &str) {
        d.leave_message = text(msg);
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerChatEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        let mut ctx = EventCtx::chat(player_ref(&d.player), d.message.clone());
        ctx.recipients = Some(d.recipients.iter().map(player_ref).collect());
        ctx
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
    fn apply_out(d: &mut EventData<Self>, out: &[(u8, Value)]) {
        use carve_core::w5_chunk1::{SLOT_NO_ITEM_DROPS, SLOT_NO_XP_DROPS};
        for (slot, _) in out {
            match *slot {
                SLOT_NO_ITEM_DROPS => d.should_drop = false,
                SLOT_NO_XP_DROPS => d.exp = 0,
                _ => {}
            }
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
        let mut ctx = EventCtx::player(player_ref(&d.player));
        ctx.death_message = Some(d.death_message.get_text());
        ctx
    }
    fn observe(d: &EventData<Self>, hs: &mut HostState) {
        let (x, y, z) = d.player.get_position();
        hs.w4.deaths.insert(
            uuid::to_string(d.player.get_id()),
            (x, y, z, world_name(&d.player.get_world())),
        );
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

impl EventAccess for InventoryClickEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        let mut c = EventCtx::player(player_ref(&d.player));
        c.inv = Some(Box::new(InvClick {
            slot: i32::from(d.slot),
            item: d.clicked_item.as_ref().map(inv::pair),
            click: inv::click_name(d.click_type).to_string(),
            menu: inv::menu_info(&uuid::to_string(d.player.get_id())),
            cursor: d.cursor.as_ref().map(inv::pair),
        }));
        c
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerAdvancementDoneEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        EventCtx::chat(player_ref(&d.player), d.advancement_id.clone())
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

/// A window closed: forget the menu, then run `on inventory close`.
struct OnInvClose;

impl EventHandler<InventoryCloseEvent> for OnInvClose {
    fn handle(&self, server: Server, data: EventData<InventoryCloseEvent>) -> EventData<InventoryCloseEvent> {
        let who = player_ref(&data.player);
        inv::forget(&who.uuid);
        with_state(&server, None, |st, host| {
            if !st.engine.table().has_event(EventKind::InventoryClose) {
                return;
            }
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine
                .fire(EventKind::InventoryClose, EventCtx::player(who), &mut th);
        });
        data
    }
}

impl EventAccess for PlayerRecipeDiscoverEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        EventCtx::chat(player_ref(&d.player), d.recipe_id.clone())
    }
    fn set_cancelled(d: &mut EventData<Self>) {
        d.cancelled = true;
    }
}

impl EventAccess for PlayerRespawnEvent {
    fn ctx(d: &EventData<Self>) -> EventCtx {
        let mut ctx = EventCtx::player(player_ref(&d.player));
        // `respawn reason`: Pumpkin fires this after a death, `alive` when leaving the End.
        ctx.message = Some(if d.alive { "end_portal" } else { "death" }.to_string());
        ctx
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
}

impl<E: EventAccess> EventHandler<E> for OnEvent<E> {
    fn handle(&self, server: Server, mut data: EventData<E>) -> EventData<E> {
        let kind = self.0;
        let left = E::left(&data);
        if let Some(u) = &left {
            sb::forget(u);
        }
        let result = with_state(&server, None, |st, host| {
            E::observe(&data, &mut st.hs);
            if !st.engine.table().has_event(kind) || E::skip(&data) {
                return (false, None, Vec::new());
            }
            let ctx = E::ctx(&data);
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            let o = st.engine.fire(kind, ctx, &mut th);
            st.hs.w4.join_msg = None;
            (o.cancelled, st.hs.w4.join_set.take(), o.set)
        });
        if let Some((cancelled, msg, out)) = result {
            if cancelled {
                E::set_cancelled(&mut data);
            }
            if !out.is_empty() {
                E::apply_out(&mut data, &out);
            }
            if let Some(m) = msg {
                E::set_message(&mut data, &m);
            }
        }
        if let Some(u) = &left {
            w4_events_movement::left(u);
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

use carve_core::w2_events_1::ExtEvent;
use pumpkin_plugin_api::events as ev;

/// Like `EventAccess`, but the event kind may depend on the data (a book
/// edit and a book signing are one server event).
trait ExtAccess: FromIntoEvent {
    fn kind(d: &EventData<Self>) -> ExtEvent;
    fn ctx(d: &EventData<Self>) -> EventCtx;
    fn set_cancelled(d: &mut EventData<Self>);
    /// Take values changed by `set event-...` back into the event.
    fn apply(_d: &mut EventData<Self>, _o: &carve_core::FireOutcome) {}
}

struct OnExt<E>(std::marker::PhantomData<fn() -> E>);

impl<E: ExtAccess> EventHandler<E> for OnExt<E> {
    fn handle(&self, server: Server, mut data: EventData<E>) -> EventData<E> {
        let cancelled = with_state(&server, None, |st, host| {
            let kind = EventKind::Ext(E::kind(&data));
            if !st.engine.table().has_event(kind) {
                return false;
            }
            let mut ctx = E::ctx(&data);
            w6_chunk1::ext_fixup(st, host, E::kind(&data), &mut ctx);
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            let o = st.engine.fire(kind, ctx, &mut th);
            if !o.set.is_empty() {
                E::apply(&mut data, &o);
            }
            o.cancelled
        });
        if cancelled == Some(true) {
            E::set_cancelled(&mut data);
        }
        data
    }
}

macro_rules! ext_event {
    ($ty:ty, |$k:ident| $kind:expr, |$d:ident| $ctx:expr, |$c:ident| $cancel:expr $(, apply = |$ad:ident, $out:ident| $apply:block)?) => {
        impl ExtAccess for $ty {
            $(fn apply($ad: &mut EventData<Self>, $out: &carve_core::FireOutcome) $apply)?
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
        meta: None,
    }
}

ext_event!(
    ev::PrepareAnvilEvent,
    |d| ExtEvent::AnvilPrepare,
    |d| ev_ctx(
        Some(&d.player),
        vec![
            Value::Text(d.rename_text.clone()),
            Value::Num(f64::from(d.repair_cost))
        ]
    ),
    |d| {}
);
ext_event!(
    ev::PlayerAnimationEvent,
    |d| ExtEvent::ArmSwing,
    |d| ev_ctx(Some(&d.player), vec![Value::Text(d.animation_type.clone())]),
    |d| d.cancelled = true
);
ext_event!(
    ev::PlayerBedEnterEvent,
    |d| ExtEvent::BedEnter,
    |d| ev_ctx(
        Some(&d.player),
        vec![at_pos(Some(world_name(&d.player.get_world())), &d.bed_pos)]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::PlayerBedLeaveEvent,
    |d| ExtEvent::BedLeave,
    |d| ev_ctx(
        Some(&d.player),
        vec![at_pos(Some(world_name(&d.player.get_world())), &d.bed_pos)]
    ),
    |d| {}
);
ext_event!(
    ev::BellRingEvent,
    |d| ExtEvent::BellRing,
    |d| ev_ctx(
        None,
        vec![
            at_pos(Some(world_name(&d.target_world)), &d.block_pos),
            Value::World(world_name(&d.target_world)),
            d.entity_id.map_or(Value::None, |i| Value::Num(f64::from(i))),
            d.direction.clone().map_or(Value::None, Value::Text),
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::BlockDamageEvent,
    |d| ExtEvent::BlockDamage,
    |d| ev_ctx(
        Some(&d.player),
        vec![
            at_pos(Some(world_name(&d.player.get_world())), &d.block_pos),
            Value::Bool(d.insta_break),
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::BlockDropItemEvent,
    |d| ExtEvent::BlockDrop,
    |d| ev_ctx(
        d.player.as_ref(),
        vec![
            at_pos(Some(world_name(&d.target_world)), &d.block_pos),
            Value::World(world_name(&d.target_world)),
            d.items.first().map_or(Value::None, |s| {
                let mut v = ev_item(&s.get_registry_key());
                if let Value::Item { count, .. } = &mut v {
                    *count = u32::from(s.get_count());
                }
                v
            }),
            Value::Num(d.items.len() as f64),
            Value::List(d.items.iter().map(gaps::value_of).collect()),
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::BlockFertilizeEvent,
    |d| ExtEvent::BlockFertilize,
    |d| ev_ctx(
        d.player.as_ref(),
        vec![
            at_pos(Some(world_name(&d.target_world)), &d.block_pos),
            Value::World(world_name(&d.target_world)),
            Value::Num(d.changed_blocks.len() as f64),
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::BlockGrowEvent,
    |d| ExtEvent::BlockGrowth,
    |d| ev_ctx(
        None,
        vec![
            at_pos(Some(world_name(&d.target_world)), &d.block_pos),
            Value::World(world_name(&d.target_world)),
            ev_item(&d.new_block),
            ev_item(&d.old_block),
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::PlayerEditBookEvent,
    |d| if d.signing {
        ExtEvent::BookSign
    } else {
        ExtEvent::BookEdit
    },
    |d| ev_ctx(
        Some(&d.player),
        vec![
            d.title.clone().map_or(Value::None, Value::Text),
            Value::Num(f64::from(d.slot)),
            Value::Text(d.pages.join("\n")),
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::BrewEvent,
    |d| ExtEvent::BrewingComplete,
    |d| ev_ctx(
        None,
        vec![at_pos(None, &d.block_pos), Value::Num(f64::from(d.fuel_level))]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::BrewingStandFuelEvent,
    |d| ExtEvent::BrewingFuel,
    |d| ev_ctx(
        None,
        vec![at_pos(None, &d.block_pos), Value::Num(f64::from(d.fuel_power))]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::BrewingStartEvent,
    |d| ExtEvent::BrewingStart,
    |d| ev_ctx(
        None,
        vec![
            at_pos(Some(world_name(&d.target_world)), &d.block_pos),
            Value::World(world_name(&d.target_world)),
            Value::Num(f64::from(d.brewing_time)),
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::ServerBroadcastEvent,
    |d| ExtEvent::Broadcast,
    |d| ev_ctx(
        None,
        vec![
            Value::Text(d.message.get_text()),
            Value::Text(d.sender.get_text())
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::PlayerBucketEntityEvent,
    |d| ExtEvent::BucketCatch,
    |d| ev_ctx(
        Some(&d.player),
        vec![ev_item(&d.bucket_item), Value::Num(f64::from(d.entity_id))]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::PlayerBucketEmptyEvent,
    |d| ExtEvent::BucketEmpty,
    |d| ev_ctx(
        Some(&d.player),
        vec![
            at_pos(Some(world_name(&d.player.get_world())), &d.block_pos),
            ev_item(&d.bucket),
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::PlayerBucketFillEvent,
    |d| ExtEvent::BucketFill,
    |d| ev_ctx(
        Some(&d.player),
        vec![
            at_pos(Some(world_name(&d.player.get_world())), &d.block_pos),
            ev_item(&d.bucket),
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::BlockBurnEvent,
    |d| ExtEvent::Burn,
    |d| ev_ctx(
        None,
        vec![
            at_pos(Some(world_name(&d.target_world)), &d.block_pos),
            Value::World(world_name(&d.target_world)),
            ev_item(&d.block),
            ev_item(&d.igniting_block),
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::BlockCanBuildEvent,
    |d| ExtEvent::CanBuild,
    |d| ev_ctx(
        Some(&d.player),
        vec![
            ev_item(&d.block_to_build),
            ev_item(&d.block),
            Value::Bool(d.buildable),
        ]
    ),
    |d| {
        d.cancelled = true;
        d.buildable = false;
    },
    apply = |d, o| {
        if let Some((_, Value::Bool(b))) = o.set.iter().rev().find(|(s, _)| *s == 0) {
            d.buildable = *b;
        }
    }
);
ext_event!(
    ev::ChunkPopulateEvent,
    |d| ExtEvent::ChunkGenerate,
    |d| ev_ctx(
        None,
        vec![Value::Num(f64::from(d.chunk_x)), Value::Num(f64::from(d.chunk_z))]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::ChunkUnloadEvent,
    |d| ExtEvent::ChunkUnload,
    |d| ev_ctx(
        None,
        vec![Value::Num(f64::from(d.chunk_x)), Value::Num(f64::from(d.chunk_z))]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::EntityCombustEvent,
    |d| ExtEvent::Combust,
    |d| ev_ctx(
        None,
        vec![
            Value::Num(f64::from(d.entity_id)),
            Value::Timespan {
                millis: (f64::from(d.duration_secs) * 1000.0) as i64
            },
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::PlayerLoginEvent,
    |d| ExtEvent::Connect,
    |d| ev_ctx(Some(&d.player), vec![]),
    |d| d.cancelled = true
);
ext_event!(
    ev::BlockDispenseEvent,
    |d| ExtEvent::Dispense,
    |d| ev_ctx(None, vec![at_pos(None, &d.block_pos), ev_item(&d.item_name)]),
    |d| d.cancelled = true
);
ext_event!(
    ev::PlayerEggThrowEvent,
    |d| ExtEvent::EggThrow,
    |d| ev_ctx(
        Some(&d.player),
        vec![
            Value::Bool(d.hatching),
            Value::Num(f64::from(d.num_hatches)),
            Value::Text(d.hatching_type.clone()),
        ]
    ),
    |d| d.cancelled = true
);
ext_event!(
    ev::PlayerElytraBoostEvent,
    |d| ExtEvent::ElytraBoost,
    |d| ev_ctx(Some(&d.player), vec![Value::Num(f64::from(d.firework_id))]),
    |d| d.cancelled = true
);

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
        st.flush_late(host, None);
        host.w5_tick();
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
        w2_events_4::poll_clock(st, host);
        due.sort();
        for (_, tok) in due {
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine.resume(tok, &mut th);
        }
        if st.engine.table().has_every() || st.engine.bounds_wanted() {
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            st.engine.tick(now, &mut th);
        }
        tick_at_time(&server, st, host);
        w6_chunk1::tick(&server, st, host, now);
        if let Some(json) = st.engine.save_due(now * 50) {
            st.save(&json);
        }
        if let Some(json) = st.engine.bounds.save_due(now * 50) {
            st.save_to(BOUNDS_FILE, &json);
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
        w5_chunk1::set_data_dir(&data_dir);
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
        let bounds_path = format!("{data_dir}/{BOUNDS_FILE}");
        if let Ok(s) = std::fs::read_to_string(&bounds_path) {
            match carve_core::skbee_bound::Bounds::from_json(&s) {
                Ok(b) => engine.bounds = b,
                Err(e) => {
                    let backup = format!("{bounds_path}.broken");
                    let _ = std::fs::rename(&bounds_path, &backup);
                    plog(
                        ApiLevel::Error,
                        &format!("{e}; starting with no bounds, old file moved to {backup}"),
                    );
                }
            }
        }
        let files = read_scripts(&scripts).unwrap_or_else(|e| {
            plog(ApiLevel::Error, &e);
            Vec::new()
        });
        let t0 = std::time::Instant::now();
        let (sum, errors) = engine.load(&files);
        // Skript-style: errors and the summary go to the console only.
        for (is_err, line) in carve_core::admin::load_report(&sum, &errors, t0.elapsed().as_secs_f64()) {
            plog(if is_err { ApiLevel::Error } else { ApiLevel::Info }, &line);
        }

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
        register_event::<PlayerRespawnEvent>(&context, EventKind::Respawn);
        register_event::<PlayerToggleSneakEvent>(&context, EventKind::SneakToggle);
        register_event::<PlayerToggleSprintEvent>(&context, EventKind::SprintToggle);
        register_event::<PlayerToggleFlightEvent>(&context, EventKind::FlightToggle);
        events_items::register(&context);
        register_event::<InventoryClickEvent>(&context, EventKind::InventoryClick);
        register_event::<PlayerAdvancementDoneEvent>(&context, EventKind::AdvancementDone);
        register_event::<PlayerRecipeDiscoverEvent>(&context, EventKind::RecipeDiscover);
        w2_events_2::register_all(&context);
        w2_events_3::register_all(&context, &|ev| engine.table().has_event(EventKind::Ext3(ev)));
        w2_events_4::register_all(&context);
        w6_chunk1::register_all(&context, &|k| engine.table().has_event(EventKind::Ext4(k)));
        w4_events_movement::register_all(
            &context,
            engine
                .table()
                .has_event(EventKind::Mv(carve_core::w4_events_movement::MvEvent::Move)),
        );
        for r in [
            context.register_event_handler::<PlayerInteractEvent, _>(OnInteract, EventPriority::Normal, true),
            context.register_event_handler::<InventoryCloseEvent, _>(OnInvClose, EventPriority::Normal, true),
            context.register_event_handler::<EntityDamageEvent, _>(
                combat::OnDamage,
                EventPriority::Normal,
                true,
            ),
            context.register_event_handler::<PlayerDeathEvent, _>(
                combat::OnDeath,
                EventPriority::Normal,
                true,
            ),
            context.register_event_handler::<pumpkin_plugin_api::events::EntityDeathEvent, _>(
                gaps::OnMobDeath,
                EventPriority::Normal,
                true,
            ),
            context.register_event_handler::<pumpkin_plugin_api::events::PlayerResourcePackStatusEvent, _>(
                OnPackStatus,
                EventPriority::Lowest,
                false,
            ),
            context.register_event_handler::<EntityDamageByEntityEvent, _>(
                combat::OnDamageByEntity,
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
                    w4: w4::W4State::default(),
                },
                data_dir,
                registered,
                entity_players: Vec::new(),
                entity_map_tick: u64::MAX,
                last_minute: 0,
                damage_pending: None,
                late_hit: None,
            });
        }
        schedule_repeating_task(1, 1, tick);
        PH_SERVER.with(|s| *s.borrow_mut() = Some(context.get_server()));
        fire_load(&context.get_server());
        Ok(())
    }

    /// Other plugins ask for placeholders here (protocol: docs/placeholders.md).
    fn handle_ipc_message(&self, _sender: String, message: Vec<u8>) -> Result<Vec<u8>, String> {
        PH_SERVER.with(|s| {
            let guard = s.borrow();
            let server = guard.as_ref().ok_or("Carve is not loaded yet")?;
            with_state(server, None, |st, host| {
                let mut th = TickHost {
                    inner: host,
                    hs: &mut st.hs,
                };
                carve_core::placeholders::handle_ipc(&mut st.engine, &message, &mut th)
            })
            .unwrap_or_else(|| Err("Carve is busy, try again".to_string()))
        })
    }

    fn on_unload(&self, _context: Context) -> Result<(), String> {
        PH_SERVER.with(|s| {
            if let Some(server) = s.borrow().as_ref() {
                w6_chunk1::fire_unload(server, true);
            }
        });
        if let Ok(mut g) = STATE.try_lock()
            && let Some(st) = g.as_mut()
            && let Some(json) = st.engine.save_now()
        {
            st.save(&json);
        }
        if let Ok(mut g) = STATE.try_lock()
            && let Some(st) = g.as_mut()
            && let Some(json) = st.engine.bounds.save_now()
        {
            st.save_to(BOUNDS_FILE, &json);
        }
        Ok(())
    }
}

register_plugin!(Carve);

/// A vanilla game rule name to the API enum.
fn rule_enum(name: &str) -> Option<GameRule> {
    Some(match name {
        "advance_time" => GameRule::AdvanceTime,
        "advance_weather" => GameRule::AdvanceWeather,
        "allow_entering_nether_using_portals" => GameRule::AllowEnteringNetherUsingPortals,
        "block_drops" => GameRule::BlockDrops,
        "block_explosion_drop_decay" => GameRule::BlockExplosionDropDecay,
        "command_block_output" => GameRule::CommandBlockOutput,
        "command_blocks_work" => GameRule::CommandBlocksWork,
        "drowning_damage" => GameRule::DrowningDamage,
        "elytra_movement_check" => GameRule::ElytraMovementCheck,
        "ender_pearls_vanish_on_death" => GameRule::EnderPearlsVanishOnDeath,
        "entity_drops" => GameRule::EntityDrops,
        "fall_damage" => GameRule::FallDamage,
        "fire_damage" => GameRule::FireDamage,
        "fire_spread_radius_around_player" => GameRule::FireSpreadRadiusAroundPlayer,
        "forgive_dead_players" => GameRule::ForgiveDeadPlayers,
        "freeze_damage" => GameRule::FreezeDamage,
        "global_sound_events" => GameRule::GlobalSoundEvents,
        "immediate_respawn" => GameRule::ImmediateRespawn,
        "keep_inventory" => GameRule::KeepInventory,
        "lava_source_conversion" => GameRule::LavaSourceConversion,
        "limited_crafting" => GameRule::LimitedCrafting,
        "locator_bar" => GameRule::LocatorBar,
        "log_admin_commands" => GameRule::LogAdminCommands,
        "max_block_modifications" => GameRule::MaxBlockModifications,
        "max_command_forks" => GameRule::MaxCommandForks,
        "max_command_sequence_length" => GameRule::MaxCommandSequenceLength,
        "max_entity_cramming" => GameRule::MaxEntityCramming,
        "max_minecart_speed" => GameRule::MaxMinecartSpeed,
        "max_snow_accumulation_height" => GameRule::MaxSnowAccumulationHeight,
        "mob_drops" => GameRule::MobDrops,
        "mob_explosion_drop_decay" => GameRule::MobExplosionDropDecay,
        "mob_griefing" => GameRule::MobGriefing,
        "natural_health_regeneration" => GameRule::NaturalHealthRegeneration,
        "player_movement_check" => GameRule::PlayerMovementCheck,
        "players_nether_portal_creative_delay" => GameRule::PlayersNetherPortalCreativeDelay,
        "players_nether_portal_default_delay" => GameRule::PlayersNetherPortalDefaultDelay,
        "players_sleeping_percentage" => GameRule::PlayersSleepingPercentage,
        "projectiles_can_break_blocks" => GameRule::ProjectilesCanBreakBlocks,
        "pvp" => GameRule::Pvp,
        "raids" => GameRule::Raids,
        "random_tick_speed" => GameRule::RandomTickSpeed,
        "reduced_debug_info" => GameRule::ReducedDebugInfo,
        "respawn_radius" => GameRule::RespawnRadius,
        "send_command_feedback" => GameRule::SendCommandFeedback,
        "show_advancement_messages" => GameRule::ShowAdvancementMessages,
        "show_death_messages" => GameRule::ShowDeathMessages,
        "spawn_mobs" => GameRule::SpawnMobs,
        "spawn_monsters" => GameRule::SpawnMonsters,
        "spawn_patrols" => GameRule::SpawnPatrols,
        "spawn_phantoms" => GameRule::SpawnPhantoms,
        "spawn_wandering_traders" => GameRule::SpawnWanderingTraders,
        "spawn_wardens" => GameRule::SpawnWardens,
        "spawner_blocks_work" => GameRule::SpawnerBlocksWork,
        "spectators_generate_chunks" => GameRule::SpectatorsGenerateChunks,
        "spread_vines" => GameRule::SpreadVines,
        "tnt_explodes" => GameRule::TntExplodes,
        "tnt_explosion_drop_decay" => GameRule::TntExplosionDropDecay,
        "universal_anger" => GameRule::UniversalAnger,
        "water_source_conversion" => GameRule::WaterSourceConversion,
        _ => return None,
    })
}

/// Carve's item components as Pumpkin's data component ids.
fn data_component(
    c: carve_core::skbee_nbt::ItemComp,
) -> pumpkin_plugin_api::wit::pumpkin::plugin::data_components::DataComponent {
    use carve_core::skbee_nbt::ItemComp;
    use pumpkin_plugin_api::wit::pumpkin::plugin::data_components::DataComponent as D;
    match c {
        ItemComp::CustomData => D::CustomData,
        ItemComp::CustomModelData => D::CustomModelData,
        ItemComp::Unbreakable => D::Unbreakable,
        ItemComp::MaxStackSize => D::MaxStackSize,
        ItemComp::Rarity => D::Rarity,
        ItemComp::GlintOverride => D::EnchantmentGlintOverride,
        ItemComp::ItemModel => D::ItemModel,
        ItemComp::StoredEnchantments => D::StoredEnchantments,
        ItemComp::DyedColor => D::DyedColor,
    }
}

fn carve_block_pos(x: i32, y: i32, z: i32) -> pumpkin_plugin_api::common::BlockPos {
    pumpkin_plugin_api::common::BlockPos { x, y, z }
}

use crate::block_ops::BlockPos;
use crate::player_flags::PlayerFlag;
use crate::value::{EntityRef, PlayerRef, Recipient};
use crate::world_effects::{FxPos, ParticleFx};
use crate::xp_level::XpField;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogLevel {
    Info,
    Warn,
    Error,
}

/// Everything the interpreter needs from the game. The core never talks to the
/// server any other way, so it can be tested with a mock.
pub trait Host {
    fn send_message(&mut self, to: &Recipient, text: &str);
    fn broadcast(&mut self, text: &str);
    fn send_title(&mut self, player: &PlayerRef, title: &str, subtitle: Option<&str>);
    fn send_action_bar(&mut self, player: &PlayerRef, text: &str);
    /// Heal to full health.
    fn heal(&mut self, player: &PlayerRef);
    fn set_health(&mut self, player: &PlayerRef, health: f64);
    fn kill(&mut self, player: &PlayerRef);
    fn teleport(&mut self, player: &PlayerRef, x: f64, y: f64, z: f64);
    /// Run a command as the console (no leading `/`).
    fn execute_command(&mut self, command: &str);
    /// Current name of an online player, `None` if offline.
    fn resolve_player(&mut self, uuid: &str) -> Option<String>;
    /// Find an online player by name.
    fn find_player(&mut self, name: &str) -> Option<PlayerRef>;
    fn has_permission(&mut self, player: &PlayerRef, node: &str) -> bool;
    /// The script line the next effect comes from. Called before every effect
    /// except `set`, so a host can name the line if the effect goes wrong later.
    fn effect_at(&mut self, _file: &std::sync::Arc<str>, _line: usize) {}
    /// Call `Engine::resume(token)` after `ticks` server ticks.
    fn schedule_wait(&mut self, ticks: u64, token: u64);
    /// Milliseconds on any monotonic clock (used for the save debounce).
    fn now(&mut self) -> u64;
    fn log(&mut self, level: LogLevel, msg: &str);

    /// Report that a call could not do what the script asked (unknown sound,
    /// player not online, ...). A host that knows the script line collects these and hands
    /// them back through [`Host::take_failures`], so the warning can name the line. The
    /// default logs the message as it is.
    fn fail(&mut self, msg: &str) {
        self.log(LogLevel::Warn, msg);
    }

    /// The messages given to [`Host::fail`] since the last call.
    fn take_failures(&mut self) -> Vec<String> {
        Vec::new()
    }

    /// A scoreboard or team change. The default runs the vanilla `/scoreboard`
    /// or `/team` command as the console; a host with a direct API overrides it.
    fn sb_edit(&mut self, op: &crate::skbee_scoreboard_teams::SbEdit) {
        self.execute_command(&op.command());
    }

    /// Wall-clock time in milliseconds since 1970-01-01 UTC (for `now`).
    fn wall_clock_millis(&mut self) -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_millis() as i64)
            .unwrap_or(0)
    }

    /// Raw bytes of a data component on the player's main-hand item (the
    /// network format). `None`: no item, or the item does not have it.
    fn held_component(&mut self, _player: &PlayerRef, _comp: crate::skbee_nbt::ItemComp) -> Option<Vec<u8>> {
        None
    }

    /// Set (`Some`) or remove (`None`) a data component on the main-hand item.
    /// Returns false when the host cannot do it.
    fn set_held_component(
        &mut self,
        _player: &PlayerRef,
        _comp: crate::skbee_nbt::ItemComp,
        _bytes: Option<Vec<u8>>,
    ) -> bool {
        false
    }

    /// Everyone online, for `loop all players`.
    fn online_players(&mut self) -> Vec<PlayerRef> {
        Vec::new()
    }

    /// Position and world name of an online player.
    fn player_location(&mut self, _player: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        None
    }

    /// Teleport, optionally into another world (by name).
    fn teleport_to(&mut self, player: &PlayerRef, x: f64, y: f64, z: f64, world: Option<&str>) {
        let _ = world;
        self.teleport(player, x, y, z);
    }

    /// Item id and amount in the player's main hand.
    fn held_item(&mut self, _player: &PlayerRef) -> Option<(String, u32)> {
        None
    }

    /// Name, lore and enchantments of the item in the player's main hand.
    fn held_item_meta(&mut self, _player: &PlayerRef) -> Option<crate::item_meta::ItemMeta> {
        None
    }

    /// Give an item that has a name, lore or enchantments. The default runs the vanilla
    /// `give` command with item components as the console.
    fn give_item_meta(
        &mut self,
        player: &PlayerRef,
        id: &str,
        count: u32,
        meta: &crate::item_meta::ItemMeta,
    ) {
        self.execute_command(&crate::item_meta::give_command(&player.name, id, count, meta));
    }

    /// How many of an item the player has in their inventory.
    fn count_item(&mut self, _player: &PlayerRef, _id: &str) -> u32 {
        0
    }

    /// Give items. The default runs the vanilla `give` command as the console.
    fn give_item(&mut self, player: &PlayerRef, id: &str, count: u32) {
        self.execute_command(&format!("give {} minecraft:{id} {count}", player.name));
    }

    /// Run a command as this player (no leading `/`). The default asks the
    /// console to run it with vanilla `execute as`.
    fn execute_command_as(&mut self, player: &PlayerRef, command: &str) {
        self.execute_command(&format!("execute as {} run {command}", player.name));
    }

    /// Text for the console itself (`send "x" to console` from a player's trigger).
    fn send_console(&mut self, text: &str) {
        self.send_message(&Recipient::Console, text);
    }

    fn health(&mut self, _player: &PlayerRef) -> Option<f64> {
        None
    }

    /// Play a sound to one player. `at` is the sound position; `None` means at the player.
    /// The default runs the vanilla `playsound` command as the console.
    fn play_sound(
        &mut self,
        player: &PlayerRef,
        sound: &str,
        volume: f64,
        pitch: f64,
        at: Option<(f64, f64, f64)>,
    ) {
        let pos = at.or_else(|| self.player_location(player).map(|(x, y, z, _)| (x, y, z)));
        if let Some((x, y, z)) = pos {
            let cmd = crate::sounds::play_command(&player.name, sound, x, y, z, volume, pitch);
            self.execute_command(&cmd);
        }
    }

    /// Stop one sound (or all sounds with `None`) for a player.
    fn stop_sound(&mut self, player: &PlayerRef, sound: Option<&str>) {
        let cmd = crate::sounds::stop_command(&player.name, sound);
        self.execute_command(&cmd);
    }

    /// Current game mode name (`survival`, `creative`, ...).
    fn gamemode(&mut self, _player: &PlayerRef) -> Option<String> {
        None
    }

    /// Set the game mode. `mode` is already a canonical name. The default runs
    /// the vanilla `gamemode` command as the console.
    fn set_gamemode(&mut self, player: &PlayerRef, mode: &str) {
        self.execute_command(&format!("gamemode {mode} {}", player.name));
    }

    /// A yes/no state of an online player (sneaking, flying, ...).
    /// Hosts that cannot tell answer `false`.
    fn player_flag(&mut self, _player: &PlayerRef, _flag: PlayerFlag) -> bool {
        false
    }

    /// Current level, total experience points, or level progress (0 to 1).
    fn xp_get(&mut self, _player: &PlayerRef, _field: XpField) -> Option<f64> {
        None
    }

    /// Set level, total points, or level progress. The default uses the
    /// `experience` command, which cannot set progress.
    fn xp_set(&mut self, player: &PlayerRef, field: XpField, value: f64) {
        let unit = match field {
            XpField::Level => "levels",
            XpField::Points => "points",
            XpField::Progress => return,
        };
        self.execute_command(&format!("experience set {} {} {unit}", player.name, value as i64));
    }

    /// Add (or with a negative amount, remove) levels or points. Only used
    /// when `xp_get` is unavailable.
    fn xp_add(&mut self, player: &PlayerRef, field: XpField, amount: f64) {
        let unit = match field {
            XpField::Level => "levels",
            XpField::Points => "points",
            XpField::Progress => return,
        };
        self.execute_command(&format!(
            "experience add {} {} {unit}",
            player.name, amount as i64
        ));
    }

    /// Read health, max health, food level or saturation.
    fn vital(&mut self, player: &PlayerRef, v: crate::health_food::Vital) -> Option<f64> {
        match v {
            crate::health_food::Vital::Health => self.health(player),
            _ => None,
        }
    }

    /// Write one of those values. The core has already clamped it.
    fn set_vital(&mut self, player: &PlayerRef, v: crate::health_food::Vital, value: f64) {
        if v == crate::health_food::Vital::Health {
            self.set_health(player, value);
        }
    }

    /// Biome name (`plains`, `birch_forest`) at a block position.
    fn biome_at(&mut self, _world: &str, _x: i32, _y: i32, _z: i32) -> Option<String> {
        None
    }

    /// Item id and amount worn in an armor slot of a player.
    fn armor_item(
        &mut self,
        _player: &PlayerRef,
        _slot: crate::expr_item_world_facts::ArmorSlot,
    ) -> Option<(String, u32)> {
        None
    }

    /// Raw NBT bytes of the block entity (beacon, furnace, brewing stand, ...) at a position.
    fn block_entity_nbt(&mut self, _world: &str, _x: i32, _y: i32, _z: i32) -> Option<Vec<u8>> {
        None
    }

    /// Where the player's compass points: x, y, z and world name.
    fn compass_target(&mut self, _player: &PlayerRef) -> Option<(f64, f64, f64, Option<String>)> {
        None
    }

    /// Point the player's compass at a place. Hosts that cannot do it ignore it.
    fn set_compass_target(&mut self, _player: &PlayerRef, _x: f64, _y: f64, _z: f64, _world: Option<&str>) {}

    /// Read a number or yes/no (as 0 or 1) state of a player: exhaustion,
    /// fall distance, fire/freeze ticks, hotbar slot, yaw, pitch, ...
    fn pstat(&mut self, _player: &PlayerRef, _stat: crate::expr_player_state::PStat) -> Option<f64> {
        None
    }

    /// Write one of those states. The core has already clamped it.
    fn set_pstat(&mut self, _player: &PlayerRef, _stat: crate::expr_player_state::PStat, _value: f64) {}

    /// Eye position and world name. The default is the feet position plus 1.62.
    fn eye_location(&mut self, player: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        self.player_location(player)
            .map(|(x, y, z, w)| (x, y + 1.62, z, w))
    }

    /// Slot number of the first empty slot in the player's inventory.
    fn first_empty_slot(&mut self, _player: &PlayerRef) -> Option<u32> {
        None
    }

    /// Can `viewer` see `other` (not hidden from them)?
    fn can_see(&mut self, _viewer: &PlayerRef, _other: &PlayerRef) -> bool {
        true
    }

    /// Machine memory in bytes: total and used.
    fn system_memory(&mut self) -> Option<(u64, u64)> {
        None
    }

    /// Y of the highest non-air block in a column. `world` `None` means the default world.
    fn top_block_y(&mut self, _world: Option<&str>, _x: i32, _z: i32) -> Option<i32> {
        None
    }

    /// Sort order in the player list.
    fn set_list_order(&mut self, _player: &PlayerRef, _order: i32) {}

    /// Air left, in ticks (300 is a full breath).
    fn remaining_air(&mut self, _player: &PlayerRef) -> Option<i64> {
        None
    }

    fn set_remaining_air(&mut self, _player: &PlayerRef, _ticks: i64) {}

    /// Where the player respawns (bed or anchor), if known.
    fn respawn_location(&mut self, _player: &PlayerRef) -> Option<(f64, f64, f64)> {
        None
    }

    fn set_respawn_location(&mut self, _player: &PlayerRef, _x: f64, _y: f64, _z: f64) {}

    /// Sea level (block height) of a world by name.
    fn sea_level(&mut self, _world: &str) -> Option<i64> {
        None
    }

    /// Simulation distance of the server, in chunks.
    fn simulation_distance(&mut self) -> Option<i64> {
        None
    }

    /// Server facts and per-player numbers for vectors, `tps`, versions and
    /// play time (see `expr_vectors_server::Query`). `None` means unknown.
    fn w2e7_query(&mut self, _q: crate::expr_vectors_server::Query<'_>) -> Option<crate::value::Value> {
        None
    }

    /// Game facts for `parsing_loot_components` (see `parsing_loot_components::Q`). `None` means unknown.
    fn w6c5(&mut self, _q: crate::parsing_loot_components::Q<'_>) -> Option<crate::value::Value> {
        None
    }

    /// Potion effects, boss bar progress, client weather, block states and
    /// menu viewers (see `potions_particles_queues::Q`). `Value::None` means unknown.
    fn w5c5(&mut self, _q: crate::potions_particles_queues::Q<'_>) -> crate::value::Value {
        crate::value::Value::None
    }

    /// Hurt the player. The default lowers health and kills at zero.
    fn damage(&mut self, player: &PlayerRef, amount: f64) {
        if let Some(h) = self.health(player) {
            if h - amount <= 0.0 {
                self.kill(player);
            } else {
                self.set_health(player, h - amount);
            }
        }
    }

    /// A client property of an online player as text (view distance as digits).
    fn player_info(&mut self, _player: &PlayerRef, _info: crate::skbee_utilities::Info) -> Option<String> {
        None
    }

    // ---- admin: op, kick, ban, names, ping ----

    /// Is the player an operator (permission level above zero)?
    fn is_op(&mut self, _player: &PlayerRef) -> bool {
        false
    }

    /// Make a player (online or not) an operator, or remove it. Default: the
    /// `op` / `deop` console command.
    fn set_op(&mut self, name: &str, op: bool) {
        self.execute_command(&format!("{} {name}", if op { "op" } else { "deop" }));
    }

    /// Kick an online player. Default: the `kick` console command.
    fn kick(&mut self, name: &str, reason: Option<&str>) {
        match reason {
            Some(r) => self.execute_command(&format!("kick {name} {r}")),
            None => self.execute_command(&format!("kick {name}")),
        }
    }

    /// Ban a player by name (or their IP). Default: `ban` / `ban-ip`.
    fn ban(&mut self, name: &str, reason: Option<&str>, ip: bool) {
        let cmd = if ip { "ban-ip" } else { "ban" };
        match reason {
            Some(r) => self.execute_command(&format!("{cmd} {name} {r}")),
            None => self.execute_command(&format!("{cmd} {name}")),
        }
    }

    /// Ban that ends after `seconds` (`ban x for 1 day`). `false`: not possible
    /// here (the target has to be online on Pumpkin).
    fn ban_for(&mut self, _name: &str, _reason: Option<&str>, _ip: bool, _seconds: u64) -> bool {
        false
    }

    /// Lift a ban. Default: `pardon` / `pardon-ip`.
    fn unban(&mut self, name: &str, ip: bool) {
        let cmd = if ip { "pardon-ip" } else { "pardon" };
        self.execute_command(&format!("{cmd} {name}"));
    }

    /// Is this name on the ban list? Hosts with no way to read it say no.
    fn is_banned(&mut self, _name: &str) -> bool {
        false
    }

    /// Display name as plain text, `None` when unknown.
    fn display_name(&mut self, _player: &PlayerRef) -> Option<String> {
        None
    }

    /// Set the display name (`&` colors allowed); `None` resets it.
    fn set_display_name(&mut self, _player: &PlayerRef, _name: Option<&str>) {}

    /// The name shown in the player list (tab list), `None` when not set.
    fn tab_list_name(&mut self, _player: &PlayerRef) -> Option<String> {
        None
    }

    /// Set the player list name, or `None` to go back to the player's name.
    fn set_tab_list_name(&mut self, _player: &PlayerRef, _name: Option<&str>) {}

    /// Latency in milliseconds.
    fn ping(&mut self, _player: &PlayerRef) -> Option<f64> {
        None
    }

    /// The player's IP address.
    fn ip(&mut self, _player: &PlayerRef) -> Option<String> {
        None
    }

    /// Lightning at a point. `effect_only`: no fire or damage.
    /// The default runs `/summon`.
    fn strike_lightning(&mut self, at: &FxPos, _effect_only: bool) {
        self.execute_command(&format!(
            "summon minecraft:lightning_bolt {} {} {}",
            at.x, at.y, at.z
        ));
    }

    /// An explosion. Pumpkin has no command for it, so the default only warns.
    fn explode(&mut self, _at: &FxPos, _power: f64, _fire: bool) {
        self.log(LogLevel::Warn, "explosions are not supported on this server");
    }

    /// The default runs `/particle`.
    fn spawn_particle(&mut self, p: &ParticleFx) {
        self.execute_command(&crate::skbee_particles_display::particle_command(p));
    }

    /// Can this host send particle data (dust color, block, item)?
    /// When not, those particles are skipped with a warning.
    fn supports_particle_data(&mut self) -> bool {
        false
    }

    /// Spawn a display entity. The default runs `/summon` for text displays
    /// and warns for block and item displays, which the command cannot fill.
    fn spawn_display(&mut self, d: &crate::skbee_particles_display::DisplaySpec) {
        match crate::skbee_particles_display::summon_command(d) {
            Some(cmd) => self.execute_command(&cmd),
            None => self.log(
                LogLevel::Warn,
                &format!("{} entities are not supported on this server", d.kind.entity_id()),
            ),
        }
    }

    /// Give a potion effect (`effect` is a vanilla id like `speed`).
    /// The default runs `/effect give`.
    fn add_effect(&mut self, player: &PlayerRef, effect: &str, ticks: u32, amplifier: u8) {
        self.execute_command(&format!(
            "effect give {} minecraft:{effect} {} {amplifier} false",
            player.name,
            ticks.div_ceil(20).max(1)
        ));
    }

    /// Poison for `seconds` with hidden particles, or cure the poison (`None`).
    /// The default runs `/effect`.
    fn poison(&mut self, player: &PlayerRef, seconds: Option<u64>) {
        match seconds {
            Some(secs) => self.execute_command(&format!(
                "effect give {} minecraft:poison {secs} 0 true",
                player.name
            )),
            None => self.execute_command(&format!("effect clear {} minecraft:poison", player.name)),
        }
    }

    /// Remove one potion effect, or all of them. The default runs `/effect clear`.
    fn clear_effects(&mut self, player: &PlayerRef, effect: Option<&str>) {
        match effect {
            Some(e) => self.execute_command(&format!("effect clear {} minecraft:{e}", player.name)),
            None => self.execute_command(&format!("effect clear {}", player.name)),
        }
    }

    fn has_effect(&mut self, _player: &PlayerRef, _effect: &str) -> bool {
        false
    }

    // ----- scoreboards and boss bars -----
    // The defaults do nothing, except the title ones, which use `/title`.

    /// A title with its own times, all in ticks. The default runs `/title ... times`.
    fn send_title_timed(
        &mut self,
        player: &PlayerRef,
        title: &str,
        subtitle: Option<&str>,
        fade_in: u32,
        stay: u32,
        fade_out: u32,
    ) {
        self.execute_command(&format!(
            "title {} times {fade_in} {stay} {fade_out}",
            player.name
        ));
        self.send_title(player, title, subtitle);
    }

    /// Remove the title on screen (`reset`: also forget its times). The default runs `/title`.
    fn clear_title(&mut self, player: &PlayerRef, reset: bool) {
        let what = if reset { "reset" } else { "clear" };
        self.execute_command(&format!("title {} {what}", player.name));
    }

    /// Set the tab list header and footer. `None` keeps what is there now;
    /// `Some("")` empties it.
    fn set_tab_list(&mut self, _player: &PlayerRef, _header: Option<&str>, _footer: Option<&str>) {}

    /// Show this player's own sidebar (made if needed) with this title.
    fn sidebar_set_title(&mut self, _player: &PlayerRef, _title: &str) {}
    /// Set a sidebar line. `line` is 1 (top) to 15.
    fn sidebar_set_line(&mut self, _player: &PlayerRef, _line: u32, _text: &str) {}
    fn sidebar_clear_line(&mut self, _player: &PlayerRef, _line: u32) {}
    /// Take the sidebar away and go back to the server scoreboard.
    fn sidebar_remove(&mut self, _player: &PlayerRef) {}

    /// Make a boss bar under a script-chosen id. `progress` is 0.0 to 1.0.
    /// An existing bar with the same id is replaced. Nobody sees it until `bossbar_show`.
    fn bossbar_create(
        &mut self,
        _id: &str,
        _title: &str,
        _color: crate::scoreboards_bossbars::BarColor,
        _style: crate::scoreboards_bossbars::BarStyle,
        _progress: f64,
    ) {
    }
    fn bossbar_set_title(&mut self, _id: &str, _title: &str) {}
    /// 0.0 to 1.0.
    fn bossbar_set_progress(&mut self, _id: &str, _fraction: f64) {}
    fn bossbar_set_color(&mut self, _id: &str, _color: crate::scoreboards_bossbars::BarColor) {}
    fn bossbar_set_style(&mut self, _id: &str, _style: crate::scoreboards_bossbars::BarStyle) {}
    fn bossbar_set_flag(&mut self, _id: &str, _flag: crate::scoreboards_bossbars::BarFlag, _on: bool) {}
    fn bossbar_show(&mut self, _id: &str, _player: &PlayerRef) {}
    fn bossbar_hide(&mut self, _id: &str, _player: &PlayerRef) {}
    /// Hide the bar from everyone and forget it.
    fn bossbar_remove(&mut self, _id: &str) {}
    /// Draw a per-player sidebar. `lines` is top to bottom (empty text is a
    /// blank line). Called again with the full content after every change.
    fn board_show(&mut self, _player: &PlayerRef, _title: &str, _lines: &[String]) {
        self.log(LogLevel::Warn, "sidebar boards are not supported on this server");
    }

    /// Remove the player's sidebar board.
    fn board_clear(&mut self, _player: &PlayerRef) {}
    // ---- recipes, advancements, statistics ----

    /// Grant or revoke an advancement (or one criterion). The default runs the
    /// `advancement` console command.
    fn advancement(&mut self, player: &PlayerRef, id: &str, criterion: Option<&str>, grant: bool) -> bool {
        let verb = if grant { "grant" } else { "revoke" };
        match criterion {
            Some(c) => self.execute_command(&format!("advancement {verb} {} only {id} {c}", player.name)),
            None => self.execute_command(&format!("advancement {verb} {} only {id}", player.name)),
        }
        true
    }

    fn has_advancement(&mut self, _player: &PlayerRef, _id: &str) -> bool {
        false
    }

    /// Register a custom recipe.
    fn register_recipe(
        &mut self,
        _recipe: &crate::skbee_recipes_advancements::RecipeDef,
    ) -> Result<(), String> {
        Err("custom recipes are not supported on this server".into())
    }

    /// Unlock (`true`) or lock a recipe in the recipe book. The default runs
    /// the `recipe give` / `recipe take` console command. The plugin API v0.1
    /// has no call to unlock a recipe for a player, so Carve keeps the command.
    fn discover_recipe(&mut self, player: &PlayerRef, id: &str, discover: bool) {
        let verb = if discover { "give" } else { "take" };
        self.execute_command(&format!("recipe {verb} {} {id}", player.name));
    }

    /// A statistic value. `category` is one of `STAT_CATEGORIES`; `name` has no namespace.
    fn statistic(&mut self, _player: &PlayerRef, _category: &str, _name: &str) -> Option<i64> {
        None
    }

    fn set_statistic(&mut self, _player: &PlayerRef, _category: &str, _name: &str, _value: i64) -> bool {
        self.log(LogLevel::Warn, "statistics are not supported on this server");
        false
    }

    /// Set the player on fire for this many ticks; 0 puts the fire out.
    fn set_fire(&mut self, _player: &PlayerRef, _ticks: i32) {
        self.log(LogLevel::Warn, "setting fire is not supported on this server");
    }

    // ---- blocks ----

    /// Block type at a position (`stone`, no `minecraft:`), `None` when unknown.
    fn block_type(&mut self, _at: &BlockPos) -> Option<String> {
        None
    }

    /// Put a block of this type there. `id` is a plain id like `stone`.
    /// The default runs `/setblock` (it ignores the world).
    fn set_block(&mut self, at: &BlockPos, id: &str) {
        self.execute_command(&format!("setblock {} {} {} minecraft:{id}", at.x, at.y, at.z));
    }

    /// Break the block like a player would. The default runs `/setblock ... destroy`.
    fn break_block(&mut self, at: &BlockPos) {
        self.execute_command(&format!(
            "setblock {} {} {} minecraft:air destroy",
            at.x, at.y, at.z
        ));
    }

    /// The block a player looks at, within `max` blocks.
    fn targeted_block(&mut self, _player: &PlayerRef, _max: f64) -> Option<BlockPos> {
        None
    }

    /// Open a chest menu with this many rows (1 to 6) and a title.
    fn open_chest(&mut self, _player: &PlayerRef, _rows: u8, _title: &str) {
        self.log(LogLevel::Warn, "chest menus are not supported on this server");
    }

    /// The item in a slot. `current`: the open menu if there is one, else
    /// the player's own inventory. Item id has no `minecraft:`.
    fn inv_get(&mut self, _player: &PlayerRef, _current: bool, _slot: i64) -> Option<(String, u32)> {
        None
    }

    /// Put an item in a slot; `None` empties it.
    fn inv_set(&mut self, _player: &PlayerRef, _current: bool, _slot: i64, _item: Option<(&str, u32)>) {
        self.log(
            LogLevel::Warn,
            "setting inventory slots is not supported on this server",
        );
    }

    /// Put an item value in a slot, with its name, lore and enchantments;
    /// `<none>` or air empties it. The default drops the meta.
    fn inv_set_item(&mut self, player: &PlayerRef, current: bool, slot: i64, item: &crate::value::Value) {
        match item {
            crate::value::Value::Item { id, count, .. } if id != "air" => {
                self.inv_set(player, current, slot, Some((id, (*count).max(1))));
            }
            _ => self.inv_set(player, current, slot, None),
        }
    }

    /// Empty the player's own inventory, or the open menu when `current`.
    fn inv_clear(&mut self, _player: &PlayerRef, _current: bool) {
        self.log(
            LogLevel::Warn,
            "clearing inventories is not supported on this server",
        );
    }

    /// Take up to `count` of an item from the player's inventory. Returns how many were taken.
    fn inv_remove(&mut self, _player: &PlayerRef, _id: &str, _count: u32) -> u32 {
        self.log(LogLevel::Warn, "removing items is not supported on this server");
        0
    }

    /// Close the inventory the player has open.
    fn close_inventory(&mut self, _player: &PlayerRef) {
        self.log(
            LogLevel::Warn,
            "closing inventories is not supported on this server",
        );
    }

    // ---- world: time, weather, game rules, difficulty ----
    // `world: None` means the default world. The default setters run the
    // vanilla commands, which always act on the default world.

    /// Time of day in ticks. May be the whole day counter; the core wraps it.
    fn world_time(&mut self, _world: Option<&str>) -> Option<i64> {
        None
    }

    /// Set the time of day. Default: `/time set`.
    fn set_world_time(&mut self, _world: Option<&str>, ticks: i64) {
        self.execute_command(&format!("time set {ticks}"));
    }

    fn weather(&mut self, _world: Option<&str>) -> Option<crate::weather_time::Weather> {
        None
    }

    /// Change the weather, optionally for a number of ticks. Default: `/weather`.
    fn set_weather(&mut self, _world: Option<&str>, to: crate::weather_time::Weather, ticks: Option<u32>) {
        self.execute_command(&to.command(ticks));
    }

    /// A game rule by vanilla name (`keep_inventory`) as `true`, `false` or a number.
    fn game_rule(&mut self, _world: Option<&str>, _rule: &str) -> Option<String> {
        None
    }

    /// Set a game rule. `value` is `true`, `false` or a whole number. Default: `/gamerule`.
    fn set_game_rule(&mut self, _world: Option<&str>, rule: &str, value: &str) {
        self.execute_command(&format!("gamerule {rule} {value}"));
    }

    /// `peaceful`, `easy`, `normal` or `hard`.
    fn difficulty(&mut self, _world: Option<&str>) -> Option<String> {
        None
    }

    /// Set the difficulty. Default: `/difficulty`.
    fn set_difficulty(&mut self, _world: Option<&str>, to: &str) {
        self.execute_command(&format!("difficulty {to}"));
    }

    // ---- entities ----

    /// Spawn an entity (`kind` is a vanilla id like `zombie`). Hosts that can
    /// return the new entity should; the default runs `/summon`, which cannot,
    /// so `last spawned entity` stays unset.
    fn spawn_entity(&mut self, kind: &str, at: &FxPos) -> Option<EntityRef> {
        self.execute_command(&format!("summon minecraft:{kind} {} {} {}", at.x, at.y, at.z));
        None
    }

    /// Launch a firework rocket. The default runs `/summon`.
    fn launch_firework(&mut self, at: &FxPos) {
        self.execute_command(&format!(
            "summon minecraft:firework_rocket {:.3} {:.3} {:.3}",
            at.x, at.y, at.z
        ));
    }

    /// Kill an entity. The default runs `/kill <uuid>`.
    fn entity_kill(&mut self, e: &EntityRef) {
        self.execute_command(&format!("kill {}", e.uuid));
    }

    /// Remove an entity with no death. The default kills it.
    fn entity_remove(&mut self, e: &EntityRef) {
        self.entity_kill(e);
    }

    /// Position and world name of an entity, `None` when it is gone.
    fn entity_location(&mut self, _e: &EntityRef) -> Option<(f64, f64, f64, String)> {
        None
    }

    /// Custom name of an entity as plain text.
    fn entity_name(&mut self, _e: &EntityRef) -> Option<String> {
        None
    }

    fn entity_health(&mut self, _e: &EntityRef) -> Option<f64> {
        None
    }

    /// Set the health of a living entity (clamped to its max health).
    fn set_entity_health(&mut self, _e: &EntityRef, _health: f64) {
        self.log(
            LogLevel::Warn,
            "setting entity health is not supported on this server",
        );
    }

    /// Set (or with `None`, clear) the custom name of an entity.
    fn set_entity_name(&mut self, _e: &EntityRef, _name: Option<&str>) {
        self.log(LogLevel::Warn, "naming entities is not supported on this server");
    }

    /// Entities (never players) within `radius` blocks of a point, optionally of one type.
    fn entities_near(&mut self, _at: &FxPos, _radius: f64, _kind: Option<&str>) -> Vec<EntityRef> {
        Vec::new()
    }

    /// The live entity with this uuid, in any world.
    fn entity_by_uuid(&mut self, _uuid: &str) -> Option<EntityRef> {
        None
    }

    /// The entity with this server entity id (the id events carry).
    fn entity_by_id(&mut self, _id: i32) -> Option<EntityRef> {
        None
    }

    /// Velocity of an entity or player (`uuid` of either).
    fn entity_velocity(&mut self, _e: &EntityRef) -> Option<(f64, f64, f64)> {
        None
    }

    /// Set the velocity of an entity or player.
    fn set_entity_velocity(&mut self, _e: &EntityRef, _v: (f64, f64, f64)) {
        self.log(LogLevel::Warn, "setting velocity is not supported on this server");
    }

    /// Teleport an entity. The default runs `/teleport <uuid> x y z`.
    fn entity_teleport(&mut self, e: &EntityRef, x: f64, y: f64, z: f64, _world: Option<&str>) {
        self.execute_command(&format!("teleport {} {x} {y} {z}", e.uuid));
    }

    // ---- server info, whitelist, worlds ----

    /// Players online now. The default counts `online_players`.
    fn player_count(&mut self) -> f64 {
        self.online_players().len() as f64
    }

    fn max_players(&mut self) -> Option<f64> {
        None
    }

    fn motd(&mut self) -> Option<String> {
        None
    }

    fn server_version(&mut self) -> Option<String> {
        None
    }

    /// Is this name on the whitelist? Hosts with no way to read it say no.
    fn is_whitelisted(&mut self, _name: &str) -> bool {
        false
    }

    /// Add or remove a player. `uuid` is set when the player is online. The
    /// default runs the `whitelist add` / `whitelist remove` console command.
    fn set_whitelisted(&mut self, name: &str, _uuid: Option<&str>, on: bool) {
        self.execute_command(&format!("whitelist {} {name}", if on { "add" } else { "remove" }));
    }

    /// Names of all worlds, the main world first.
    fn world_names(&mut self) -> Vec<String> {
        Vec::new()
    }

    /// The world used when a script names none: the main world.
    fn default_world(&mut self) -> Option<String> {
        self.world_names().into_iter().next()
    }

    /// Text for everyone in one world. The default filters `online_players`
    /// by `player_location`.
    fn broadcast_world(&mut self, world: &str, text: &str) {
        for p in self.online_players() {
            if self
                .player_location(&p)
                .is_some_and(|(_, _, _, w)| w.eq_ignore_ascii_case(world))
            {
                self.send_message(&Recipient::Player(p), text);
            }
        }
    }

    /// Spawn point of a world, centered on its block.
    fn world_spawn(&mut self, _world: &str) -> Option<(f64, f64, f64)> {
        None
    }

    /// Current diameter of a world's border.
    fn border_size(&mut self, _world: &str) -> Option<f64> {
        None
    }

    /// Set the border diameter, over `seconds` (0: at once). The default runs
    /// `worldborder set` as the console, which only reaches the main world.
    fn set_border_size(&mut self, world: &str, size: f64, seconds: f64) {
        if self
            .default_world()
            .is_some_and(|d| !d.eq_ignore_ascii_case(world))
        {
            self.log(
                LogLevel::Warn,
                "setting the world border of another world is not supported on this server",
            );
            return;
        }
        if seconds > 0.0 {
            self.execute_command(&format!("worldborder set {size} {}s", seconds.round() as i64));
        } else {
            self.execute_command(&format!("worldborder set {size}"));
        }
    }

    /// Send a player to another server. The default runs the vanilla
    /// `transfer` command as the console.
    fn transfer(&mut self, player: &PlayerRef, host: &str, port: u16) {
        self.execute_command(&format!("transfer {host} {port} {}", player.name));
    }

    /// Force a dead player to respawn. Hosts that cannot do it ignore this.
    fn respawn(&mut self, _player: &PlayerRef) {}

    /// Hide `target` from `viewer`'s view.
    fn hide_player(&mut self, _viewer: &PlayerRef, _target: &PlayerRef) {}

    /// Undo `hide_player`.
    fn show_player(&mut self, _viewer: &PlayerRef, _target: &PlayerRef) {}

    /// Put an item in a vanilla slot such as `armor.head`. The default runs
    /// the `item replace` command as the console.
    fn equip(&mut self, player: &PlayerRef, slot: &str, id: &str) {
        self.execute_command(&format!(
            "item replace entity {} {slot} with minecraft:{id}",
            player.name
        ));
    }

    /// `make player fly` / `stop flying`: allow flight and set flying.
    fn set_flight(&mut self, _player: &PlayerRef, _on: bool) {}

    fn set_invulnerable(&mut self, _player: &PlayerRef, _on: bool) {}

    /// Invisible flag. The default gives or clears the invisibility effect.
    fn set_invisible(&mut self, player: &PlayerRef, on: bool) {
        let cmd = if on {
            format!(
                "effect give {} minecraft:invisibility infinite 0 true",
                player.name
            )
        } else {
            format!("effect clear {} minecraft:invisibility", player.name)
        };
        self.execute_command(&cmd);
    }

    /// Push the player. `(x, z)` points from the player toward the pusher.
    fn apply_knockback(&mut self, _player: &PlayerRef, _strength: f64, _x: f64, _z: f64) {}

    /// Turn the player's view. The default uses the vanilla `rotate` command.
    fn set_rotation(&mut self, player: &PlayerRef, yaw: f32, pitch: f32) {
        self.execute_command(&format!("rotate {} {yaw} {pitch}", player.name));
    }

    /// Open the book held in a hand.
    fn open_book(&mut self, _player: &PlayerRef, _off_hand: bool) {}

    /// Make the player say something in chat. The default runs `say` as them.
    fn say_as(&mut self, player: &PlayerRef, text: &str) {
        self.execute_command_as(player, &format!("say {text}"));
    }

    /// Append a line to `logs/<file>`. `file` is already a clean plain name.
    fn log_to_file(&mut self, file: &str, text: &str) {
        use std::io::Write;
        let _ = std::fs::create_dir_all("logs");
        let ms = self.wall_clock_millis();
        if let Ok(mut f) = std::fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(std::path::Path::new("logs").join(file))
        {
            let _ = writeln!(f, "[{ms}] {text}");
        }
    }

    /// Poisoned, silent or riding (players only). Hosts that cannot tell answer `false`.
    fn player_state(
        &mut self,
        _player: &PlayerRef,
        _state: crate::conditions_entity_block::PlayerState,
    ) -> bool {
        false
    }

    /// Whether PvP is on in a world (by name), or in the default world with `None`.
    /// `None` means the host cannot tell.
    fn pvp_enabled(&mut self, _world: Option<&str>) -> Option<bool> {
        None
    }

    /// A property of a block type (`stone`, no namespace). `None` if unknown.
    fn block_trait(&mut self, _id: &str, _t: crate::conditions_entity_block::BlockTrait) -> Option<bool> {
        None
    }
    // ---- expr_item_damage_facts ----

    /// Answer one of the `conditions_mob_block_state` questions (see [`crate::conditions_mob_block_state::Q`]).
    fn w6c3(&mut self, _q: crate::conditions_mob_block_state::Q<'_>) -> Option<crate::value::Value> {
        None
    }

    /// Answer one of the `conditions_mobs_items` questions (see [`crate::conditions_mobs_items::Q`]).
    /// `None` when the game cannot tell.
    fn w5c3(&mut self, _q: crate::conditions_mobs_items::Q<'_>) -> Option<crate::value::Value> {
        None
    }

    /// Answer a yes/no question about a player, entity or block (see
    /// `conditions_entity_checks::Check`). `None` means the game cannot tell.
    fn w4c1_check(
        &mut self,
        _c: crate::conditions_entity_checks::Check,
        _subject: &crate::value::Value,
    ) -> Option<bool> {
        None
    }

    /// Answer a question for the `conditions_flags_metadata` conditions. `None` means the game
    /// cannot tell.
    fn w5c2_query(&mut self, _q: crate::conditions_flags_metadata::Query<'_>) -> Option<crate::value::Value> {
        None
    }

    /// Ticks left on an item cooldown, `None` when there is none.
    fn item_cooldown(&mut self, _player: &PlayerRef, _item: &str) -> Option<i64> {
        None
    }

    /// Start (or with 0 ticks, clear) an item cooldown.
    fn set_item_cooldown(&mut self, _player: &PlayerRef, _item: &str, _ticks: i64) {}

    /// Where the player last died: x, y, z and world name.
    fn last_death_location(&mut self, _player: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        None
    }

    /// The last hit the player took.
    fn last_damage(&mut self, _player: &PlayerRef) -> Option<crate::expr_item_damage_facts::DamageInfo> {
        None
    }

    /// The join message of the `on join` event that is running.
    fn join_message(&mut self) -> Option<String> {
        None
    }

    /// Change the join message of the `on join` event that is running.
    fn set_join_message(&mut self, _text: &str) {}

    /// Last resource pack status the player reported (`loaded`, `declined`, ...).
    fn resource_pack_status(&mut self, _player: &PlayerRef) -> Option<String> {
        None
    }

    /// Y of the highest (or lowest) solid block in a column.
    fn solid_block_y(&mut self, _world: &str, _x: i32, _z: i32, _highest: bool) -> Option<i32> {
        None
    }

    /// Light at a block: kind 0 block light, 1 sky light, 2 the larger of both.
    fn light_level(&mut self, _world: &str, _x: i32, _y: i32, _z: i32, _kind: u8) -> Option<u8> {
        None
    }

    /// Show or hide `target` to one `viewer`. Hosts without the API do nothing.
    fn set_visible_to(&mut self, viewer: &PlayerRef, target: &PlayerRef, visible: bool) {
        if visible {
            self.show_player(viewer, target);
        } else {
            self.hide_player(viewer, target);
        }
    }

    /// Save every world. The default runs the vanilla `save-all` command.
    fn save_world(&mut self) {
        self.execute_command("save-all");
    }

    /// Stop the server. The default runs the vanilla `stop` command. The plugin API v0.1
    /// has no shutdown call, so Carve keeps the command.
    fn stop_server(&mut self) {
        self.execute_command("stop");
    }

    fn set_sprinting(&mut self, _player: &PlayerRef, _on: bool) {}

    /// Play the arm swing. The default runs the vanilla `swing` command. The plugin API v0.1
    /// can send entity animations but not the swing packet 26.3 clients read, so Carve keeps the command.
    fn swing_hand(&mut self, player: &PlayerRef, main: bool) {
        let cmd = crate::effects_player_state::swing_command(&player.name, main);
        self.execute_command(&cmd);
    }

    /// Get off whatever the player is riding. The default runs `ride dismount`.
    fn dismount(&mut self, player: &PlayerRef) {
        self.execute_command(&format!("ride {} dismount", player.name));
    }

    /// Throw off everything riding the player.
    fn eject_passengers(&mut self, _player: &PlayerRef) {}

    /// Put `rider` on `vehicle`. The default runs `ride mount`.
    fn ride(&mut self, rider: &PlayerRef, vehicle: &PlayerRef) {
        self.execute_command(&format!("ride {} mount {}", rider.name, vehicle.name));
    }

    fn set_name_visible(&mut self, _player: &PlayerRef, _visible: bool) {}

    /// Offer a resource pack. `id` is a UUID string that is the same for the same URL.
    fn send_resource_pack(&mut self, _player: &PlayerRef, _url: &str, _hash: &str, _id: &str) {}
    // ---- ray tracing, worlds and chunks (`skbee_raytrace_world`) ----

    /// Cast a ray and report the first block and the first entity on it.
    /// The default walks the blocks with `block_type` (every solid block counts
    /// as a full cube) and finds no entities.
    fn ray_trace(
        &mut self,
        req: &crate::skbee_raytrace_world::RayReq,
    ) -> crate::skbee_raytrace_world::RayRaw {
        crate::skbee_raytrace_world::walk_blocks(req, self)
    }

    /// Create a world, or load it when it exists.
    fn create_world(&mut self, _name: &str, _env: crate::skbee_raytrace_world::Env) -> Result<(), String> {
        Err("this server cannot create worlds".into())
    }

    /// Unload and save a world.
    fn unload_world(&mut self, _name: &str) -> Result<(), String> {
        Err("this server cannot unload worlds".into())
    }

    /// Is the world loaded? By default: is it in `world_names`.
    fn world_loaded(&mut self, name: &str) -> bool {
        self.world_names().iter().any(|w| w.eq_ignore_ascii_case(name))
    }

    /// Is the chunk loaded? `None`: unknown.
    fn chunk_loaded(&mut self, _world: &str, _x: i32, _z: i32) -> Option<bool> {
        None
    }

    // ---- inventory values and held items (see `gaps`) ----

    /// Open a chest menu built from an inventory value. The default opens a
    /// plain chest and fills it slot by slot, without names or lore.
    fn open_inventory(&mut self, player: &PlayerRef, inv: &crate::gaps::InvVal) {
        self.open_chest(player, inv.rows, &inv.title);
        for (i, v) in inv.slots.iter().enumerate() {
            if let crate::value::Value::Item { id, count, .. } = v {
                self.inv_set(player, true, i as i64, Some((id, *count)));
            }
        }
    }

    /// A slot of the inventory value `id` changed: update the open menus
    /// that show it.
    fn menu_update(&mut self, _id: u64, _slot: u32, _item: &crate::value::Value) {}

    /// Set a slot of the menu the player has open. `item` is an item value or
    /// `<none>`. The default drops names and lore.
    fn menu_set(&mut self, player: &PlayerRef, slot: u32, item: &crate::value::Value) {
        let it = match item {
            crate::value::Value::Item { id, count, .. } => Some((id.as_str(), *count)),
            _ => None,
        };
        self.inv_set(player, true, i64::from(slot), it);
    }

    /// Replace the name, lore and enchantments of the item in the player's
    /// main hand. False when this server can't.
    fn set_held_item_meta(&mut self, _player: &PlayerRef, _meta: &crate::item_meta::ItemMeta) -> bool {
        false
    }

    /// The items in a player's inventory (slots 0 to 40), with names and lore
    /// when the server gives them.
    fn inv_items(&mut self, player: &PlayerRef) -> Vec<crate::value::Value> {
        (0..=40)
            .filter_map(|s| self.inv_get(player, false, s))
            .map(|(id, count)| crate::value::Value::Item {
                id,
                count,
                meta: None,
            })
            .collect()
    }

    /// Yaw and pitch of an entity or player, in degrees.
    fn entity_rotation(&mut self, _e: &EntityRef) -> Option<(f64, f64)> {
        None
    }

    /// `set quit message to` in `on quit`.
    /// Block data, world border settings, books, ender chests, AI and
    /// attributes (see `block_shapes_ai::Q`). `None` means unknown or not done.
    fn w5c4(&mut self, _q: crate::block_shapes_ai::Q<'_>) -> Option<crate::value::Value> {
        None
    }

    fn set_quit_message(&mut self, text: &str) {
        self.set_join_message(text);
    }

    /// A question from `quaternions_protocol` (portal cooldown, protocol, vehicle, whitelist, ...).
    fn w6c6_get(&mut self, _q: crate::quaternions_protocol::Get<'_>) -> Option<crate::value::Value> {
        None
    }

    /// A change from `quaternions_protocol`. Returns whether it was done.
    fn w6c6_set(&mut self, _s: crate::quaternions_protocol::Set<'_>) -> bool {
        false
    }

    /// A game-side change of `effects_entity_commands` (flight, fake blocks, display flags,
    /// sound categories, shearing, taming). `None`: this host can't.
    fn w6c2(&mut self, _op: crate::effects_entity_commands::HostOp<'_>) -> Option<bool> {
        None
    }

    /// A question from `entity_block_properties` (scale, targets, text displays, borders, boss bars).
    fn c6_get(&mut self, _q: crate::entity_block_properties::Get<'_>) -> Option<crate::value::Value> {
        None
    }

    /// A change from `entity_block_properties`. Returns whether it was done.
    fn c6_set(&mut self, _s: crate::entity_block_properties::Set<'_>) -> bool {
        false
    }

    /// The display entity the last `spawn_display` made, taken once.
    fn last_display(&mut self) -> Option<EntityRef> {
        None
    }

    /// Block id, state id and the block's first state id at a position.
    fn c6_block_state(&mut self, _at: &BlockPos) -> Option<crate::entity_block_properties::BlockState> {
        None
    }

    /// Mob, sign and potion changes of `drops_border_leashes` (creepers, babies,
    /// pathfinding, glowing sign text, potion flags). `None`: not supported
    /// or no such target.
    fn w5c1(&mut self, _op: crate::drops_border_leashes::HostOp<'_>) -> Option<bool> {
        None
    }

    /// The bytes of a file in the plugin's data folder (`load server icon`).
    fn read_data_file(&mut self, _path: &str) -> Option<Vec<u8>> {
        None
    }
}

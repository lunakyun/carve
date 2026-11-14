use crate::block_ops::BlockPos;
use crate::player_flags::PlayerFlag;
use crate::value::{EntityRef, PlayerRef, Recipient};
use crate::xp_level::XpField;
use crate::world_effects::{FxPos, ParticleFx};

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
    /// Call `Engine::resume(token)` after `ticks` server ticks.
    fn schedule_wait(&mut self, ticks: u64, token: u64);
    /// Milliseconds on any monotonic clock (used for the save debounce).
    fn now(&mut self) -> u64;
    fn log(&mut self, level: LogLevel, msg: &str);

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
    fn give_item_meta(&mut self, player: &PlayerRef, id: &str, count: u32, meta: &crate::item_meta::ItemMeta) {
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
        self.execute_command(&format!("experience add {} {} {unit}", player.name, amount as i64));
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

    fn tab_list_name(&mut self, _player: &PlayerRef) -> Option<String> {
        None
    }

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
        self.execute_command(&format!("title {} times {fade_in} {stay} {fade_out}", player.name));
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
    fn register_recipe(&mut self, _recipe: &crate::skbee_recipes_advancements::RecipeDef) -> Result<(), String> {
        Err("custom recipes are not supported on this server".into())
    }

    /// Unlock (`true`) or lock a recipe in the recipe book. The default runs
    /// the `recipe give` / `recipe take` console command.
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
        self.execute_command(&format!("setblock {} {} {} minecraft:air destroy", at.x, at.y, at.z));
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
        self.log(LogLevel::Warn, "setting inventory slots is not supported on this server");
    }

    /// Empty the player's own inventory, or the open menu when `current`.
    fn inv_clear(&mut self, _player: &PlayerRef, _current: bool) {
        self.log(LogLevel::Warn, "clearing inventories is not supported on this server");
    }

    /// Take up to `count` of an item from the player's inventory. Returns how many were taken.
    fn inv_remove(&mut self, _player: &PlayerRef, _id: &str, _count: u32) -> u32 {
        self.log(LogLevel::Warn, "removing items is not supported on this server");
        0
    }

    /// Close the inventory the player has open.
    fn close_inventory(&mut self, _player: &PlayerRef) {
        self.log(LogLevel::Warn, "closing inventories is not supported on this server");
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

    /// Set (or with `None`, clear) the custom name of an entity.
    fn set_entity_name(&mut self, _e: &EntityRef, _name: Option<&str>) {
        self.log(LogLevel::Warn, "naming entities is not supported on this server");
    }

    /// Entities (never players) within `radius` blocks of a point, optionally of one type.
    fn entities_near(&mut self, _at: &FxPos, _radius: f64, _kind: Option<&str>) -> Vec<EntityRef> {
        Vec::new()
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
}

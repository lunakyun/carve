use crate::player_flags::PlayerFlag;
use crate::value::{PlayerRef, Recipient};
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
        self.execute_command(&format!(
            "particle minecraft:{} {} {} {} {sp} {sp} {sp} {} {}",
            p.name,
            p.at.x,
            p.at.y,
            p.at.z,
            p.speed,
            p.count,
            sp = p.spread,
        ));
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

    /// Set the player on fire for this many ticks; 0 puts the fire out.
    fn set_fire(&mut self, _player: &PlayerRef, _ticks: i32) {
        self.log(LogLevel::Warn, "setting fire is not supported on this server");
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
        if self.default_world().is_some_and(|d| !d.eq_ignore_ascii_case(world)) {
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
}

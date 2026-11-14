use crate::player_flags::PlayerFlag;
use crate::value::{PlayerRef, Recipient};
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
}

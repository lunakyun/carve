use crate::value::{PlayerRef, Recipient};
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

    fn health(&mut self, _player: &PlayerRef) -> Option<f64> {
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
}

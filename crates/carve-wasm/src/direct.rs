//! Actions that used to go through console commands, done with the plugin API
//! instead: sounds, kick, ban, op, give, experience, titles, riding, poison,
//! fireworks and scoreboard teams.
//!
//! Anything that can't be done (unknown sound, player not online, bad item)
//! goes to `WasmHost::fail`, which the engine logs with the script line.
//! What the plugin API v0.1 has no call for stays a console command in the
//! core (swing, recipe book, stop, difficulty, weather length, entity NBT); each
//! of those has a comment naming the missing call.

use carve_core::item_meta::ItemMeta;
use carve_core::skbee_scoreboard_teams::{SbEdit, TeamProp};
use carve_core::sounds::{self, SoundKind};
use carve_core::world_effects::FxPos;
use carve_core::xp_level::XpField;
use carve_core::{PlayerRef, Value};
use pumpkin_plugin_api::common::NamedColor;
use pumpkin_plugin_api::item_stack::ItemStack;
use pumpkin_plugin_api::permission::PermissionLevel;
use pumpkin_plugin_api::player::{
    BanIpOptions, BanPlayerOptions, BedrockDisconnectReason, BedrockKickOptions, JavaKickOptions,
};
use pumpkin_plugin_api::scoreboard::{CollisionRule, NametagVisibility, RenderType, TeamSettings};
use pumpkin_plugin_api::text::TextComponent;
use pumpkin_plugin_api::wit::pumpkin::plugin::java_packets::{CClearTitle, ClientboundPacket as JavaPacket};
use pumpkin_plugin_api::wit::pumpkin::plugin::sounds::SoundCategory;
use pumpkin_plugin_api::wit::pumpkin::plugin::status_effect::StatusEffectInstance;
use pumpkin_plugin_api::world::Entity;
use pumpkin_plugin_api::{Player, TeamSettingsBuilder, uuid};

use super::{WasmHost, fx, gaps, inv, text};

/// The source shown in the ban list.
const SOURCE: &str = "Carve";

/// How a ban request went.
enum Ban {
    Done,
    /// The target is not online and no UUID is known for the name. The caller decides what to do.
    NeedsLookup,
    Failed(String),
}

impl WasmHost<'_> {
    pub(super) fn fail(&mut self, msg: &str) {
        self.failures.push(msg.to_string());
    }

    pub(super) fn take_failures(&mut self) -> Vec<String> {
        std::mem::take(&mut self.failures)
    }

    // ---- sounds ----

    pub(super) fn play_sound(
        &mut self,
        p: &PlayerRef,
        sound: &str,
        volume: f64,
        pitch: f64,
        at: Option<(f64, f64, f64)>,
    ) {
        if sounds::classify(sound) == SoundKind::Unknown {
            self.fail(&format!(
                "play sound: '{sound}' is not a vanilla sound (try \"entity.experience_orb.pickup\"), nothing was played"
            ));
            return;
        }
        let Some(pl) = self.get(p) else {
            self.fail(&format!("play sound: {} is not online", p.name));
            return;
        };
        let (volume, pitch) = sounds::clamp_levels(volume, pitch);
        let name = sounds::namespaced(sound);
        match at {
            Some(pos) => pl.play_custom_sound_at(pos, &name, SoundCategory::Master, volume, pitch),
            None => pl.play_custom_sound(&name, SoundCategory::Master, volume, pitch),
        }
    }

    pub(super) fn stop_sound(&mut self, p: &PlayerRef, sound: Option<&str>) {
        let Some(pl) = self.get(p) else {
            self.fail(&format!("stop sound: {} is not online", p.name));
            return;
        };
        let name = sound.map(sounds::namespaced);
        pl.stop_custom_sound(name.as_deref(), None);
    }

    // ---- kick, ban, op ----

    pub(super) fn kick(&mut self, name: &str, reason: Option<&str>) {
        let Some(pl) = self.server.get_player_by_name(name) else {
            self.fail(&format!("kick: {name} is not online"));
            return;
        };
        let msg = reason.unwrap_or("Kicked by an operator");
        if let Some(java) = pl.as_java() {
            java.kick(JavaKickOptions::new(text(msg)));
        } else if let Some(bedrock) = pl.as_bedrock() {
            bedrock.kick(&BedrockKickOptions::new(BedrockDisconnectReason::Kicked, msg));
        }
    }

    /// A UUID for a name the server already knows: online, banned, an operator or whitelisted.
    fn known_uuid(&mut self, name: &str) -> Option<uuid::Uuid> {
        if let Some(p) = self.server.get_player_by_name(name) {
            return Some(p.get_id());
        }
        if let Some(b) = self
            .server
            .get_ban_manager()
            .list_player_bans()
            .into_iter()
            .find(|b| b.name.eq_ignore_ascii_case(name))
        {
            return Some(b.uuid);
        }
        if let Some(o) = self
            .server
            .get_op_manager()
            .list_ops()
            .into_iter()
            .find(|o| o.name.eq_ignore_ascii_case(name))
        {
            return Some(o.uuid);
        }
        self.server
            .get_whitelist_manager()
            .list_entries()
            .into_iter()
            .find(|e| e.name.eq_ignore_ascii_case(name))
            .map(|e| e.uuid)
    }

    /// The IP an `ip` ban means: the text itself when it is an address, else the address of the online player.
    fn ban_address(&mut self, name: &str) -> Result<String, String> {
        if name.parse::<std::net::IpAddr>().is_ok() {
            return Ok(name.to_string());
        }
        let Some(pl) = self.server.get_player_by_name(name) else {
            return Err(format!("ban ip: {name} is not an IP address and not online"));
        };
        let ip = pl.get_ip();
        Ok(match ip.rsplit_once(':') {
            Some((host, port)) if port.bytes().all(|b| b.is_ascii_digit()) => {
                host.trim_start_matches('[').trim_end_matches(']').to_string()
            }
            _ => ip,
        })
    }

    fn ban_core(&mut self, name: &str, reason: Option<&str>, ip: bool, seconds: Option<u64>) -> Ban {
        let reason = reason.map(text);
        if ip {
            let addr = match self.ban_address(name) {
                Ok(a) => a,
                Err(e) => return Ban::Failed(e),
            };
            self.server.get_ban_manager().ban_ip(
                &addr,
                BanIpOptions {
                    reason,
                    source: Some(SOURCE.to_string()),
                    expires_at_utc: None,
                    duration_seconds: seconds,
                    kick_matching_players: true,
                    log_to_console: true,
                },
            );
            return Ban::Done;
        }
        let options = BanPlayerOptions {
            reason,
            source: Some(SOURCE.to_string()),
            expires_at_utc: None,
            duration_seconds: seconds,
            kick_if_online: true,
            log_to_console: true,
        };
        let Some(id) = self.known_uuid(name) else {
            return Ban::NeedsLookup;
        };
        self.server.get_ban_manager().ban_player(name, id, options);
        Ban::Done
    }

    pub(super) fn ban(&mut self, name: &str, reason: Option<&str>, ip: bool) {
        match self.ban_core(name, reason, ip, None) {
            Ban::Done => {}
            Ban::Failed(e) => self.fail(&e),
            // An offline name nobody knows needs the profile lookup that only the `ban` command does.
            Ban::NeedsLookup => match reason {
                Some(r) => self.execute_command(&format!("ban {name} {r}")),
                None => self.execute_command(&format!("ban {name}")),
            },
        }
    }

    /// A ban that ends by itself. `false` when the name can't be turned into a UUID.
    pub(super) fn ban_for_impl(&mut self, name: &str, reason: Option<&str>, ip: bool, seconds: u64) -> bool {
        match self.ban_core(name, reason, ip, Some(seconds)) {
            Ban::Done => true,
            Ban::Failed(e) => {
                self.fail(&e);
                true
            }
            Ban::NeedsLookup => false,
        }
    }

    pub(super) fn unban(&mut self, name: &str, ip: bool) {
        let bans = self.server.get_ban_manager();
        if ip {
            let addr = match self.ban_address(name) {
                Ok(a) => a,
                Err(e) => {
                    self.fail(&e.replace("ban ip", "unban ip"));
                    return;
                }
            };
            if !bans.unban_ip(&addr) {
                self.fail(&format!("unban ip: {addr} is not banned"));
            }
            return;
        }
        let found = bans
            .list_player_bans()
            .into_iter()
            .find(|b| b.name.eq_ignore_ascii_case(name));
        match found {
            Some(b) => {
                bans.unban_player(b.uuid);
            }
            None => self.fail(&format!("unban: {name} is not banned")),
        }
    }

    pub(super) fn set_op(&mut self, name: &str, op: bool) {
        let ops = self.server.get_op_manager();
        let Some(id) = self.known_uuid(name) else {
            if op {
                // An offline name nobody knows needs the profile lookup that only the `op` command does.
                self.execute_command(&format!("op {name}"));
            } else {
                self.fail(&format!("deop: {name} is not an operator"));
            }
            return;
        };
        if op {
            // `op` gives the level the server config says; Pumpkin's default is the top one.
            let level = ops.get_op(id).map_or(PermissionLevel::Four, |e| e.level);
            ops.op_player(name, id, level, false);
        } else if !ops.deop_player(id) {
            self.fail(&format!("deop: {name} is not an operator"));
        }
    }

    // ---- items and experience ----

    /// Put `count` of an item in the player's inventory (stacks first, then empty
    /// slots). What doesn't fit comes back.
    fn insert_item(pl: &Player, id: &str, count: u32, meta: Option<&ItemMeta>) -> u32 {
        let inv = pl.get_inventory().as_inventory();
        let key = format!("minecraft:{id}");
        let max = u32::from(ItemStack::new(&key, 1).get_max_count()).max(1);
        let wanted = meta.filter(|m| !m.is_empty());
        let mut left = count;
        for slot in 0..inv.get_size() {
            if left == 0 {
                break;
            }
            let Some(s) = inv.get_item(slot) else { continue };
            let have = u32::from(s.get_count());
            let (sid, _) = inv::pair(&s);
            if sid != id || have >= max {
                continue;
            }
            let same = match gaps::value_of(&s) {
                Value::Item { meta: m, .. } => m.as_deref() == wanted,
                _ => false,
            };
            if !same {
                continue;
            }
            let take = (max - have).min(left);
            s.set_count((have + take) as u8);
            inv.set_item(slot, Some(s));
            left -= take;
        }
        for slot in 0..inv.get_size() {
            if left == 0 {
                break;
            }
            if inv.get_item(slot).is_some() {
                continue;
            }
            let take = max.min(left);
            let s = ItemStack::new(&key, take as u8);
            if let Some(m) = wanted {
                apply_given_meta(&s, m);
            }
            inv.set_item(slot, Some(s));
            left -= take;
        }
        left
    }

    fn give(&mut self, p: &PlayerRef, id: &str, count: u32, meta: Option<&ItemMeta>) {
        if !carve_core::items::is_known(id) {
            self.fail(&format!("give: '{id}' is not an item"));
            return;
        }
        if let Some(m) = meta {
            for (e, _) in &m.enchants {
                if gaps::enchantment(e).is_none() {
                    self.fail(&format!(
                        "give: unknown enchantment '{e}', it was left off the item"
                    ));
                }
            }
        }
        // Raw item components are only known to the `give` command.
        if meta.is_some_and(|m| !m.comps.is_empty()) {
            if self.get(p).is_none() {
                self.fail(&format!("give: {} is not online", p.name));
                return;
            }
            let m = meta.map(|m| carve_core::item_meta::give_command(&p.name, id, count, m));
            if let Some(cmd) = m {
                self.execute_command(&cmd);
            }
            return;
        }
        let Some(pl) = self.get(p) else {
            self.fail(&format!("give: {} is not online", p.name));
            return;
        };
        let left = Self::insert_item(pl, id, count, meta);
        if left > 0 {
            // A full inventory drops the rest at the player's feet, which only the command does (no call
            // in the plugin API v0.1 drops an item stack).
            match meta {
                Some(m) => self.execute_command(&carve_core::item_meta::give_command(&p.name, id, left, m)),
                None => self.execute_command(&format!("give {} minecraft:{id} {left}", p.name)),
            }
        }
    }

    pub(super) fn give_item(&mut self, p: &PlayerRef, id: &str, count: u32) {
        self.give(p, id, count, None);
    }

    pub(super) fn give_item_meta(&mut self, p: &PlayerRef, id: &str, count: u32, meta: &ItemMeta) {
        self.give(p, id, count, Some(meta));
    }

    pub(super) fn xp_add(&mut self, p: &PlayerRef, field: XpField, amount: f64) {
        let Some(pl) = self.get(p) else {
            self.fail(&format!("experience: {} is not online", p.name));
            return;
        };
        match field {
            XpField::Level => pl.add_experience_levels(amount as i32),
            XpField::Points => pl.add_experience_points(amount as i32),
            XpField::Progress => {}
        }
    }

    pub(super) fn poison(&mut self, p: &PlayerRef, seconds: Option<u64>) {
        let Some(kind) = fx::effect("poison") else { return };
        let Some(pl) = self.get(p) else {
            self.fail(&format!("poison: {} is not online", p.name));
            return;
        };
        match seconds {
            Some(s) => pl.add_effect(StatusEffectInstance {
                effect_type: kind,
                duration: u32::try_from(s.saturating_mul(20)).unwrap_or(u32::MAX),
                amplifier: 0,
                ambient: false,
                show_particles: false,
                show_icon: true,
            }),
            None => pl.remove_effect(kind),
        }
    }

    // ---- titles ----

    /// A Java player gets the clear-title packet. Anyone else (there is no clear-title call in the
    /// plugin API v0.1) gets the title and subtitle replaced with empty text.
    pub(super) fn clear_title(&mut self, p: &PlayerRef, reset: bool) {
        let Some(pl) = self.get(p) else {
            self.fail(&format!("clear title: {} is not online", p.name));
            return;
        };
        if let Some(java) = pl.as_java() {
            java.send_packet(&JavaPacket::CClearTitle(CClearTitle { reset }));
            return;
        }
        if reset {
            pl.send_title_animation(10, 70, 20);
        }
        pl.show_subtitle(text(""));
        pl.show_title(text(""));
    }

    // ---- riding ----

    /// `ride`: `rider` gets on `vehicle`, or off whatever it rides. `Some(false)`: one of them is not found.
    pub(super) fn ride_op(&mut self, rider: &Value, vehicle: Option<&Value>) -> Option<bool> {
        let rider = self.entcmd_entity_of(rider);
        match vehicle {
            Some(v) => {
                let vehicle = self.entcmd_entity_of(v);
                let (Some(rider), Some(vehicle)) = (rider, vehicle) else {
                    return Some(false);
                };
                vehicle.add_passenger(rider);
            }
            None => {
                let Some(rider) = rider else { return Some(false) };
                let current = rider.get_vehicle();
                if let Some(v) = current {
                    v.remove_passenger(rider);
                }
            }
        }
        Some(true)
    }

    fn entcmd_entity_of(&mut self, v: &Value) -> Option<Entity> {
        match v {
            Value::Player(p) => self.get(p).map(Player::as_entity),
            Value::Entity(e) => self.find_entity(&e.uuid),
            _ => None,
        }
    }

    pub(super) fn ride(&mut self, rider: &PlayerRef, vehicle: &PlayerRef) {
        let (r, v) = (Value::Player(rider.clone()), Value::Player(vehicle.clone()));
        if self.ride_op(&r, Some(&v)) != Some(true) {
            self.fail(&format!("ride: {} or {} is not online", rider.name, vehicle.name));
        }
    }

    pub(super) fn dismount(&mut self, p: &PlayerRef) {
        if self.ride_op(&Value::Player(p.clone()), None) != Some(true) {
            self.fail(&format!("dismount: {} is not online", p.name));
        }
    }

    pub(super) fn launch_firework(&mut self, at: &FxPos) {
        if self.spawn_entity("firework_rocket", at).is_none() {
            self.fail("launch firework: the server could not spawn a firework rocket there");
        }
    }

    // ---- scoreboard and teams ----

    pub(super) fn sb_edit(&mut self, op: &SbEdit) {
        let Some(world) = self.named_world(None) else {
            self.fail("scoreboard: there is no world to hold the scoreboard");
            return;
        };
        let sb = world.get_scoreboard();
        match op {
            SbEdit::AddObjective {
                name,
                criteria,
                display,
            } => {
                if criteria == "dummy" {
                    sb.add_objective(
                        name,
                        text(display.as_deref().unwrap_or(name)),
                        RenderType::Integer,
                        None,
                    );
                } else {
                    // Objective criteria (`health`, `playerKillCount`) have no call in the plugin API v0.1.
                    self.execute_command(&op.command());
                }
            }
            SbEdit::RemoveObjective(name) => sb.remove_objective(name),
            SbEdit::SetScore {
                holder,
                objective,
                value,
            } => sb.update_score(holder, objective, *value, None),
            SbEdit::ResetScore {
                holder,
                objective: Some(o),
            } => sb.remove_score(holder, o),
            SbEdit::ResetScore {
                holder,
                objective: None,
            } => sb.reset_entity_scores(holder),
            SbEdit::AddTeam(t) => sb.create_team(t, TeamSettingsBuilder::new().display_name(text(t)).build()),
            SbEdit::RemoveTeam(t) => sb.remove_team(t),
            SbEdit::EmptyTeam(t) => sb.clear_team_players(t),
            SbEdit::Join { team, holder } => sb.add_player_to_team(team, holder),
            SbEdit::Leave { holder } => {
                if let Some(team) = sb.get_player_team(holder) {
                    sb.remove_player_from_team(&team, holder);
                }
            }
            SbEdit::TeamModify { team, prop, arg } => {
                let Some(mut s) = sb.get_team(team) else {
                    self.fail(&format!("team '{team}' does not exist on the server"));
                    return;
                };
                if !apply_team_prop(&mut s, *prop, arg) {
                    // The plugin API v0.1 team settings have no death message visibility.
                    self.execute_command(&op.command());
                    return;
                }
                sb.update_team(team, s);
            }
        }
    }
}

/// Name, lore, enchantments and damage for an item that is given. Pumpkin keeps
/// only the plain text of a custom name and of lore lines, so the colours go in
/// as `§` codes, which the client draws (what the `give` command did, without
/// its chat message).
fn apply_given_meta(stack: &ItemStack, m: &ItemMeta) {
    let coded = |s: &str| TextComponent::text(&carve_core::colors::colorize(s));
    stack.set_custom_name(m.name.as_deref().map(coded));
    stack.set_lore(m.lore.iter().map(|l| coded(l)).collect());
    for (id, level) in &m.enchants {
        if let Some(e) = gaps::enchantment(id) {
            stack.add_enchantment(e, *level);
        }
    }
    super::block_shapes_ai::set_damage(stack, m.damage);
}

/// Change one setting of a team. `false`: the API has no field for it.
fn apply_team_prop(s: &mut TeamSettings, prop: TeamProp, arg: &str) -> bool {
    match prop {
        TeamProp::Prefix => s.prefix = text(arg),
        TeamProp::Suffix => s.suffix = text(arg),
        TeamProp::DisplayName => s.display_name = text(arg),
        TeamProp::Color => s.color = named_color(arg),
        TeamProp::FriendlyFire => s.friendly_fire = arg == "true",
        TeamProp::SeeInvisibles => s.see_friendly_invisibles = arg == "true",
        TeamProp::NametagVisibility => {
            s.nametag_visibility = match arg {
                "never" => NametagVisibility::Never,
                "hideForOtherTeams" => NametagVisibility::HideForOtherTeams,
                "hideForOwnTeam" => NametagVisibility::HideForOwnTeam,
                _ => NametagVisibility::Always,
            };
        }
        TeamProp::CollisionRule => {
            s.collision_rule = match arg {
                "never" => CollisionRule::Never,
                "pushOtherTeams" => CollisionRule::PushOtherTeams,
                "pushOwnTeam" => CollisionRule::PushOwnTeam,
                _ => CollisionRule::Always,
            };
        }
        TeamProp::DeathMessageVisibility => return false,
    }
    true
}

/// A vanilla team colour name; `reset` (no colour) is white, which is how names look without one.
fn named_color(name: &str) -> NamedColor {
    match name {
        "black" => NamedColor::Black,
        "dark_blue" => NamedColor::DarkBlue,
        "dark_green" => NamedColor::DarkGreen,
        "dark_aqua" => NamedColor::DarkAqua,
        "dark_red" => NamedColor::DarkRed,
        "dark_purple" => NamedColor::DarkPurple,
        "gold" => NamedColor::Gold,
        "gray" => NamedColor::Gray,
        "dark_gray" => NamedColor::DarkGray,
        "blue" => NamedColor::Blue,
        "green" => NamedColor::Green,
        "aqua" => NamedColor::Aqua,
        "red" => NamedColor::Red,
        "light_purple" => NamedColor::LightPurple,
        "yellow" => NamedColor::Yellow,
        _ => NamedColor::White,
    }
}

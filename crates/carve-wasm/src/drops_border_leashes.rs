//! Game side of `carve_core::drops_border_leashes`: creepers, babies, pathfinding,
//! glowing sign text, potion effect flags, the whitelist switch, the data
//! folder for `load server icon`, and kept inventories after a death.

use std::sync::Mutex;

use carve_core::drops_border_leashes::{self as core5, HostOp, Ignite, PathTarget, PotionFlag};
use carve_core::xp_level::XpField;
use carve_core::{PlayerRef, Value};
use pumpkin_plugin_api::block_entity::BlockEntityType;
use pumpkin_plugin_api::common::BlockPos as WitPos;
use pumpkin_plugin_api::world::{Entity, MobData};
use pumpkin_plugin_api::{GameRule, GameRuleValue};

use super::WasmHost;

/// The plugin data folder, set on load.
static DATA_DIR: Mutex<String> = Mutex::new(String::new());

/// A death that kept (or dropped) things: undone once the player is back.
struct Pending {
    uuid: String,
    world: String,
    /// keepInventory before the death, when Carve switched it on.
    rule_was: Option<bool>,
    /// Clear the kept inventory after the respawn (`cancel drops`).
    clear_items: bool,
    /// Experience to put back after the respawn (level, progress).
    xp: Option<(f64, f64)>,
    /// Remove the experience the rule kept.
    reset_xp: bool,
}

static PENDING: Mutex<Vec<Pending>> = Mutex::new(Vec::new());

/// A walk in progress. Pumpkin's own goals (wandering, looking around) take
/// the navigator back, so the walk is given again every few ticks until the
/// mob arrives, the time runs out or `stop pathfinding` ends it.
struct Walk {
    mob: String,
    to: WalkTo,
    speed: f64,
    ticks: u32,
}

enum WalkTo {
    Pos(f64, f64, f64),
    Entity(String),
}

/// Ticks between re-issued walks, and the longest walk.
const WALK_EVERY: u32 = 5;
const WALK_MAX: u32 = 1200;

static WALKS: Mutex<Vec<Walk>> = Mutex::new(Vec::new());

fn end_walk(uuid: &str) {
    if let Ok(mut w) = WALKS.lock() {
        w.retain(|x| x.mob != uuid);
    }
}

pub(super) fn set_data_dir(dir: &str) {
    if let Ok(mut d) = DATA_DIR.lock() {
        *d = dir.to_string();
    }
}

pub(super) fn read_data_file(path: &str) -> Option<Vec<u8>> {
    let dir = DATA_DIR.lock().ok()?.clone();
    if dir.is_empty() || path.contains("..") {
        return None;
    }
    std::fs::read(format!("{dir}/{path}")).ok()
}

/// What the death event's script asked for, from the outcome slots.
pub(super) fn death_flags(out: &[(u8, Value)]) -> (bool, bool, bool) {
    let has = |s: u8| out.iter().any(|(k, v)| *k == s && *v == Value::Bool(true));
    (
        has(core5::SLOT_KEEP_INVENTORY),
        has(core5::SLOT_KEEP_XP),
        has(core5::SLOT_NO_ITEM_DROPS),
    )
}

impl WasmHost<'_> {
    fn w5_entity(&mut self, v: &Value) -> Option<Entity> {
        match v {
            Value::Player(p) => self.get(p).map(|pl| pl.as_entity()),
            Value::Entity(e) => self.find_entity(&e.uuid),
            _ => None,
        }
    }

    /// Called from the death event, before Pumpkin drops the inventory.
    pub(super) fn w5_death(&mut self, p: &PlayerRef, keep_inv: bool, keep_xp: bool, no_drops: bool) {
        if !keep_inv && !keep_xp && !no_drops {
            return;
        }
        let Some(pl) = self.get(p) else { return };
        let world = pl.get_world();
        let name = super::world_name(&world);
        let needs_rule = keep_inv || no_drops;
        let mut rule_was = None;
        if needs_rule {
            let was = matches!(
                world.get_game_rule(GameRule::KeepInventory),
                GameRuleValue::Bool(true)
            );
            if !was {
                world.set_game_rule_bool(GameRule::KeepInventory, true);
                rule_was = Some(false);
            }
        }
        let xp = (keep_xp && !needs_rule).then(|| {
            (
                self.xp_get(p, XpField::Level).unwrap_or(0.0),
                self.xp_get(p, XpField::Progress).unwrap_or(0.0),
            )
        });
        if let Ok(mut q) = PENDING.lock() {
            q.retain(|x| x.uuid != p.uuid);
            q.push(Pending {
                uuid: p.uuid.clone(),
                world: name,
                rule_was,
                clear_items: no_drops && !keep_inv,
                xp,
                reset_xp: needs_rule && !keep_xp,
            });
        }
    }

    /// Every tick: keep walks going, and finish the deaths whose player is
    /// alive again (or gone).
    pub(super) fn w5_tick(&mut self) {
        self.w5_walks();
        let todo: Vec<Pending> = match PENDING.lock() {
            Ok(mut q) if !q.is_empty() => std::mem::take(&mut *q),
            _ => return,
        };
        let mut keep = Vec::new();
        for d in todo {
            let online = self.resolve_player(&d.uuid);
            let p = online.as_ref().map(|name| PlayerRef {
                uuid: d.uuid.clone(),
                name: name.clone(),
            });
            let alive = p
                .as_ref()
                .is_some_and(|p| self.health(p).is_some_and(|h| h > 0.0));
            if p.is_some() && !alive {
                keep.push(d);
                continue;
            }
            if let Some(false) = d.rule_was
                && let Some(w) = self.server.get_world_by_name(&d.world)
            {
                w.set_game_rule_bool(GameRule::KeepInventory, false);
            }
            let Some(p) = p else { continue };
            if d.clear_items {
                self.inv_clear(&p, false);
            }
            if d.reset_xp {
                self.xp_set(&p, XpField::Level, 0.0);
                self.xp_set(&p, XpField::Progress, 0.0);
            }
            if let Some((lvl, prog)) = d.xp {
                self.xp_set(&p, XpField::Level, lvl);
                self.xp_set(&p, XpField::Progress, prog);
            }
        }
        if !keep.is_empty()
            && let Ok(mut q) = PENDING.lock()
        {
            q.extend(keep);
        }
    }

    fn w5_walks(&mut self) {
        let walks = match WALKS.lock() {
            Ok(mut q) if !q.is_empty() => std::mem::take(&mut *q),
            _ => return,
        };
        self.w5_walks_cold(walks);
    }

    #[cold]
    fn w5_walks_cold(&mut self, walks: Vec<Walk>) {
        let mut keep = Vec::new();
        for mut w in walks {
            w.ticks += 1;
            if w.ticks % WALK_EVERY != 0 {
                keep.push(w);
                continue;
            }
            let Some(e) = self.find_entity(&w.mob) else {
                continue;
            };
            let Some(mob) = e.as_mob() else { continue };
            let (mx, my, mz) = e.get_position();
            let dest = match &w.to {
                WalkTo::Pos(x, y, z) => Some((*x, *y, *z)),
                WalkTo::Entity(t) => self.find_entity(t).map(|t| t.get_position()),
            };
            let Some((x, y, z)) = dest else { continue };
            let near = (mx - x).powi(2) + (my - y).powi(2) + (mz - z).powi(2) < 2.25;
            if near || w.ticks > WALK_MAX {
                continue;
            }
            mob.navigate_to_pos((x, y, z), w.speed);
            keep.push(w);
        }
        if let Ok(mut q) = WALKS.lock() {
            q.extend(keep);
        }
    }

    pub(super) fn w5c1_impl(&mut self, op: HostOp<'_>) -> Option<bool> {
        match op {
            HostOp::Creeper(v, mode) => {
                let mob = self.w5_entity(v)?.as_mob()?;
                let MobData::Creeper(mut d) = mob.get_mob_data() else {
                    return Some(false);
                };
                match mode {
                    Ignite::Instant => {
                        d.is_ignited = true;
                        d.fuse = 1;
                    }
                    Ignite::Start => d.is_ignited = true,
                    Ignite::Stop => d.is_ignited = false,
                }
                Some(mob.set_mob_data(MobData::Creeper(d)))
            }
            HostOp::Ageable(v) => {
                let mob = self.w5_entity(v)?.as_mob()?;
                Some(matches!(
                    mob.get_mob_data(),
                    MobData::Ageable(_) | MobData::Zombie(_)
                ))
            }
            HostOp::Pathfind(v, to, speed) => {
                let e = self.w5_entity(v)?;
                let mob = e.as_mob()?;
                let uuid = super::ent_host::uuid_of(&e);
                let (ok, to) = match to {
                    PathTarget::Pos(x, y, z) => (mob.navigate_to_pos((x, y, z), speed), WalkTo::Pos(x, y, z)),
                    PathTarget::Entity(t) => {
                        let t = self.w5_entity(t)?;
                        let id = super::ent_host::uuid_of(&t);
                        (mob.navigate_to_entity(t, speed), WalkTo::Entity(id))
                    }
                };
                if ok {
                    end_walk(&uuid);
                    if let Ok(mut q) = WALKS.lock() {
                        q.push(Walk {
                            mob: uuid,
                            to,
                            speed,
                            ticks: 0,
                        });
                    }
                }
                Some(ok)
            }
            HostOp::StopPath(v) => {
                let e = self.w5_entity(v)?;
                end_walk(&super::ent_host::uuid_of(&e));
                let mob = e.as_mob()?;
                mob.stop_navigation();
                Some(true)
            }
            HostOp::SignGlow(world, x, y, z, on) => {
                let w = self.server.get_world_by_name(world)?;
                match w.get_block_entity(WitPos { x, y, z })? {
                    BlockEntityType::SignBlockEntity(s) => {
                        let mut t = s.get_front_text();
                        t.has_glowing_text = on;
                        s.set_front_text(&t);
                        Some(true)
                    }
                    BlockEntityType::HangingSignBlockEntity(s) => {
                        let mut t = s.get_front_text();
                        t.has_glowing_text = on;
                        s.set_front_text(&t);
                        Some(true)
                    }
                    _ => Some(false),
                }
            }
            HostOp::Potion(p, flag, on) => {
                let pl = self.get(p)?;
                for mut e in pl.get_active_effects() {
                    match flag {
                        PotionFlag::Ambient => e.ambient = on,
                        PotionFlag::Particles => e.show_particles = on,
                        PotionFlag::Icon => e.show_icon = on,
                        PotionFlag::Infinite => {
                            if on {
                                e.duration = u32::MAX;
                            } else if e.duration == u32::MAX {
                                // Skript gives an effect that stops being infinite 30 seconds.
                                e.duration = 600;
                            }
                        }
                    }
                    pl.remove_effect(e.effect_type);
                    pl.add_effect(e);
                }
                Some(true)
            }
            HostOp::WhitelistOn => Some(self.server.get_whitelist_manager().is_enabled()),
        }
    }
}

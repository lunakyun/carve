//! Game side of the `expr_player_state` expressions: player stats, eye
//! position, free inventory slot, hidden players, memory, top block.

use carve_core::PlayerRef;
use carve_core::expr_player_state::PStat;

use crate::{WasmHost, world_name};

/// When each player last did something, in milliseconds since the epoch.
static ACTIVE: std::sync::Mutex<Vec<(String, u64)>> = std::sync::Mutex::new(Vec::new());

fn now_ms() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |d| d.as_millis() as u64)
}

/// The player did something (moved, clicked, chatted).
pub(crate) fn touch(uuid: &str) {
    let now = now_ms();
    if let Ok(mut a) = ACTIVE.lock() {
        match a.iter_mut().find(|(u, _)| u == uuid) {
            Some(e) => e.1 = now,
            None => a.push((uuid.to_string(), now)),
        }
    }
}

/// Ticks since the player last did something (since the plugin saw them first).
fn idle_ticks(uuid: &str) -> f64 {
    let now = now_ms();
    let Ok(mut a) = ACTIVE.lock() else { return 0.0 };
    let since = match a.iter().find(|(u, _)| u == uuid) {
        Some((_, t)) => *t,
        None => {
            a.push((uuid.to_string(), now));
            now
        }
    };
    (now.saturating_sub(since) / 50) as f64
}

impl WasmHost<'_> {
    pub(crate) fn pstat_impl(&mut self, p: &PlayerRef, s: PStat) -> Option<f64> {
        let pl = self.get(p)?;
        let e = pl.as_entity();
        let b = |v: bool| f64::from(u8::from(v));
        Some(match s {
            PStat::Exhaustion => f64::from(pl.get_exhaustion()),
            PStat::FallDistance => f64::from(e.get_fall_distance()),
            PStat::FireTicks => f64::from(e.get_fire_ticks().max(0)),
            PStat::FreezeTicks => f64::from(pl.get_freeze_ticks().max(0)),
            PStat::Gliding => b(e.is_fall_flying()),
            PStat::Glowing => b(e.is_glowing()),
            PStat::Gravity => b(e.has_gravity()),
            PStat::HotbarSlot => f64::from(pl.get_selected_slot()),
            PStat::Yaw => f64::from(e.get_yaw()),
            PStat::Pitch => f64::from(e.get_pitch()),
            PStat::IdleTicks => idle_ticks(&p.uuid),
        })
    }

    pub(crate) fn set_pstat_impl(&mut self, p: &PlayerRef, s: PStat, v: f64) {
        let Some(pl) = self.get(p) else { return };
        let e = pl.as_entity();
        match s {
            PStat::Exhaustion => pl.set_exhaustion(v as f32),
            PStat::FallDistance => e.set_fall_distance(v as f32),
            PStat::FireTicks => e.set_fire_ticks(v as i32),
            PStat::FreezeTicks => pl.set_freeze_ticks(v as i32),
            PStat::Gliding => e.set_fall_flying(v != 0.0),
            PStat::Glowing => e.set_glowing(v != 0.0),
            PStat::Gravity => e.set_has_gravity(v != 0.0),
            PStat::HotbarSlot | PStat::Yaw | PStat::Pitch | PStat::IdleTicks => {}
        }
    }

    pub(crate) fn eye_location_impl(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        let pl = self.get(p)?;
        let (x, y, z) = pl.as_entity().get_eye_position();
        Some((x, y, z, world_name(&pl.get_world())))
    }

    pub(crate) fn first_empty_slot_impl(&mut self, p: &PlayerRef) -> Option<u32> {
        let pl = self.get(p)?;
        (0u8..36)
            .find(|s| pl.get_inventory_item(*s).is_none())
            .map(u32::from)
    }

    pub(crate) fn can_see_impl(&mut self, viewer: &PlayerRef, other: &PlayerRef) -> bool {
        let Some(other_pl) =
            pumpkin_plugin_api::uuid::parse(&other.uuid).and_then(|id| self.server.get_player_by_uuid(id))
        else {
            return true;
        };
        match self.get(viewer) {
            Some(v) => v.can_see_player(other_pl),
            None => true,
        }
    }

    pub(crate) fn system_memory_impl(&mut self) -> Option<(u64, u64)> {
        let info = self.server.get_sys_info();
        Some((info.total_memory?, info.used_memory.unwrap_or(0)))
    }

    pub(crate) fn top_block_y_impl(&mut self, world: Option<&str>, x: i32, z: i32) -> Option<i32> {
        let w = match world {
            Some(name) => self.server.get_world_by_name(name)?,
            None => self.server.get_all_worlds().into_iter().next()?,
        };
        Some(w.get_top_block_y(x, z))
    }
}

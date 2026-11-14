//! Game-side support for the `w2_expressions_4` expressions: what the plugin
//! remembers between events (last death, last hit, join message, resource
//! pack status), plus world lookups (light, solid blocks).

use std::collections::HashMap;

use carve_core::PlayerRef;
use carve_core::w2_expressions_4::DamageInfo;
use pumpkin_plugin_api::DamageType;
use pumpkin_plugin_api::common::BlockPos;

use super::{State, WasmHost};

/// Things scripts can ask about after the event that produced them.
#[derive(Default)]
pub struct W4State {
    /// player uuid -> x, y, z, world
    pub deaths: HashMap<String, (f64, f64, f64, String)>,
    /// victim uuid -> last hit
    pub hits: HashMap<String, DamageInfo>,
    /// player uuid -> last resource pack status
    pub packs: HashMap<String, String>,
    /// Join message while an `on join` trigger runs.
    pub join_msg: Option<String>,
    /// A join message a script set, waiting to be applied to the event.
    pub join_set: Option<String>,
}

/// `PlayerAttack` -> `player_attack`.
pub fn damage_name(t: DamageType) -> String {
    let dbg = format!("{t:?}");
    let mut out = String::with_capacity(dbg.len() + 4);
    for (i, c) in dbg.chars().enumerate() {
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

impl State {
    /// Remember a hit on a player. `damager` is the attacking entity id when
    /// the by-entity event gives one.
    pub fn track_hit(
        &mut self,
        host: &mut WasmHost,
        victim_id: i32,
        amount: f32,
        cause: Option<DamageType>,
        damager: Option<i32>,
    ) {
        let Some(victim) = self.player_by_entity(host, victim_id) else {
            return;
        };
        let attacker: Option<PlayerRef> = damager.and_then(|id| self.player_by_entity(host, id));
        let entry = self.hs.w4.hits.entry(victim.uuid).or_insert_with(|| DamageInfo {
            amount: 0.0,
            cause: String::new(),
            attacker: None,
        });
        entry.amount = f64::from(amount);
        match cause {
            Some(c) => {
                entry.cause = damage_name(c);
                entry.attacker = attacker;
            }
            // The by-entity event follows the plain one and only adds the attacker.
            None => entry.attacker = attacker,
        }
    }
}

const NON_SOLID: [&str; 12] = [
    "air",
    "cave_air",
    "void_air",
    "water",
    "lava",
    "short_grass",
    "tall_grass",
    "fern",
    "large_fern",
    "snow",
    "light",
    "vine",
];

impl WasmHost<'_> {
    pub fn item_cooldown(&mut self, p: &PlayerRef, item: &str) -> Option<i64> {
        self.get(p)
            .and_then(|pl| pl.get_item_cooldown(&format!("minecraft:{item}")))
            .map(i64::from)
    }

    pub fn set_item_cooldown(&mut self, p: &PlayerRef, item: &str, ticks: i64) {
        let ticks = i32::try_from(ticks).unwrap_or(i32::MAX);
        if let Some(pl) = self.get(p) {
            pl.set_item_cooldown(&format!("minecraft:{item}"), ticks);
        }
    }

    /// Highest or lowest block in a column that is not air, a liquid or a plant.
    pub fn solid_block_y(&mut self, world: &str, x: i32, z: i32, highest: bool) -> Option<i32> {
        let w = self.server.get_world_by_name(world)?;
        let solid = |y: i32| {
            let name = w.get_block(BlockPos { x, y, z }).name;
            let short = name.strip_prefix("minecraft:").unwrap_or(&name);
            !NON_SOLID.contains(&short)
        };
        if highest {
            let top = w.get_top_block_y(x, z);
            (-64..=top.max(-64)).rev().find(|y| solid(*y))
        } else {
            (-64..=320).find(|y| solid(*y))
        }
    }

    /// `0` block light, `1` sky light, `2` the larger of both.
    pub fn light_level(&mut self, world: &str, x: i32, y: i32, z: i32, kind: u8) -> Option<u8> {
        let w = self.server.get_world_by_name(world)?;
        let pos = BlockPos { x, y, z };
        Some(match kind {
            0 => w.get_block_light(pos),
            1 => w.get_sky_light(pos),
            _ => w.get_block_light(pos).max(w.get_sky_light(pos)),
        })
    }
}

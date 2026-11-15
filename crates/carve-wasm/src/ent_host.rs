//! Entity calls of the host: spawn, find, kill, move and name entities
//! through the Pumpkin WIT entity API.

use carve_core::EntityRef;
use carve_core::world_effects::FxPos;
use pumpkin_plugin_api::world::{Entity, World};
use pumpkin_plugin_api::{DamageType, uuid};

use super::{WasmHost, text, world_name};
use crate::ents;

fn uuid_of(e: &Entity) -> String {
    uuid::to_string(e.get_uuid())
}

fn handle(e: &Entity) -> EntityRef {
    EntityRef {
        uuid: uuid_of(e),
        kind: ents::entity_name(e.get_type()).to_string(),
    }
}

impl WasmHost<'_> {
    /// The live entity (or player) with this uuid.
    pub(super) fn find_entity(&mut self, id: &str) -> Option<Entity> {
        if let Some(p) = self.get_by_uuid(id) {
            return Some(p.as_entity());
        }
        for w in self.server.get_all_worlds() {
            if let Some(e) = w.get_entities().into_iter().find(|e| uuid_of(e) == id) {
                return Some(e);
            }
        }
        None
    }

    /// The entity with this server entity id, searched in every world.
    #[cold]
    pub(super) fn entity_by_id(&mut self, id: i32) -> Option<EntityRef> {
        let id = u32::try_from(id).ok()?;
        for w in self.server.get_all_worlds() {
            if let Some(e) = w.get_entities().into_iter().find(|e| e.get_id() == id) {
                return Some(handle(&e));
            }
        }
        None
    }

    /// The worlds an area search covers: the named one, or all of them.
    fn search_worlds(&mut self, at: &FxPos) -> Vec<World> {
        match at.world.as_deref() {
            Some(name) => self.server.get_world_by_name(name).into_iter().collect(),
            None => self.server.get_all_worlds(),
        }
    }

    pub(super) fn spawn_entity(&mut self, kind: &str, at: &FxPos) -> Option<EntityRef> {
        let kind_enum = ents::entity_type(kind)?;
        let w = self.fx_world(at)?;
        let e = w.spawn_entity(kind_enum, (at.x, at.y, at.z));
        Some(handle(&e))
    }

    pub(super) fn entity_kill(&mut self, r: &EntityRef) {
        if let Some(e) = self.find_entity(&r.uuid) {
            match e.as_living() {
                Some(l) => l.damage(1.0e9, DamageType::GenericKill),
                None => e.remove(),
            }
        }
    }

    pub(super) fn entity_remove(&mut self, r: &EntityRef) {
        if let Some(e) = self.find_entity(&r.uuid) {
            e.remove();
        }
    }

    pub(super) fn entity_location(&mut self, r: &EntityRef) -> Option<(f64, f64, f64, String)> {
        let e = self.find_entity(&r.uuid)?;
        let (x, y, z) = e.get_position();
        Some((x, y, z, world_name(&e.get_world())))
    }

    pub(super) fn entity_name(&mut self, r: &EntityRef) -> Option<String> {
        self.find_entity(&r.uuid)?.get_custom_name().map(|t| t.get_text())
    }

    pub(super) fn entity_health(&mut self, r: &EntityRef) -> Option<f64> {
        let l = self.find_entity(&r.uuid)?.as_living()?;
        Some(f64::from(l.get_health()))
    }

    pub(super) fn set_entity_health(&mut self, r: &EntityRef, health: f64) {
        if let Some(l) = self.find_entity(&r.uuid).and_then(|e| e.as_living()) {
            let max = f64::from(l.get_max_health());
            l.set_health(health.min(max) as f32);
        }
    }

    pub(super) fn set_entity_name(&mut self, r: &EntityRef, name: Option<&str>) {
        if let Some(e) = self.find_entity(&r.uuid) {
            match name {
                Some(n) => {
                    e.set_custom_name(text(n));
                    e.set_custom_name_visible(true);
                }
                None => e.set_custom_name_visible(false),
            }
        }
    }

    pub(super) fn entities_near(&mut self, at: &FxPos, radius: f64, kind: Option<&str>) -> Vec<EntityRef> {
        let mut out = Vec::new();
        for w in self.search_worlds(at) {
            for e in w.get_entities() {
                let h = handle(&e);
                if h.kind == "player" || kind.is_some_and(|k| k != h.kind) {
                    continue;
                }
                let (x, y, z) = e.get_position();
                let (dx, dy, dz) = (x - at.x, y - at.y, z - at.z);
                if (dx * dx + dy * dy + dz * dz).sqrt() <= radius {
                    out.push(h);
                }
            }
        }
        out
    }

    pub(super) fn entity_velocity(&mut self, r: &EntityRef) -> Option<(f64, f64, f64)> {
        Some(self.find_entity(&r.uuid)?.get_velocity())
    }

    pub(super) fn set_entity_velocity(&mut self, r: &EntityRef, v: (f64, f64, f64)) {
        // The entity call only stores the velocity, and the client moves its own
        // player, so a player needs the player call that sends the packet.
        if let Some(p) = self.get_by_uuid(&r.uuid) {
            p.set_velocity(v);
            return;
        }
        if let Some(e) = self.find_entity(&r.uuid) {
            e.set_velocity(v);
        }
    }

    pub(super) fn entity_teleport(&mut self, r: &EntityRef, x: f64, y: f64, z: f64, world: Option<&str>) {
        let target = world.and_then(|w| self.server.get_world_by_name(w));
        if let Some(e) = self.find_entity(&r.uuid) {
            let w = target.unwrap_or_else(|| e.get_world());
            e.teleport((x, y, z), w);
        }
    }
}

//! Wave 6, chunk 7: host calls.

use carve_core::EntityRef;

use super::WasmHost;
use crate::ent_host::uuid_of;
use crate::ents;

impl WasmHost<'_> {
    /// The live entity (or player) with this uuid, in any world.
    #[cold]
    pub(super) fn entity_by_uuid_impl(&mut self, id: &str) -> Option<EntityRef> {
        let e = self.find_entity(id)?;
        Some(EntityRef {
            uuid: uuid_of(&e),
            kind: ents::entity_name(e.get_type()).to_string(),
        })
    }
}

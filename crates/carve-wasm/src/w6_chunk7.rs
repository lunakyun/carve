//! Wave 6, chunk 7: host calls (`entity from uuid`) and `auto reload:`.

use carve_core::EntityRef;
use pumpkin_plugin_api::logging::LogLevel as ApiLevel;

use super::{DirStore, State, TickHost, WasmHost, plog};
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

// ---- auto reload ----

/// Size and modification time of each script, as last seen.
type Stamps = Vec<(String, u64, u128)>;

static STAMPS: std::sync::Mutex<Option<Stamps>> = std::sync::Mutex::new(None);

/// Ticks between looks at the scripts folder (2 seconds).
pub(crate) const AUTO_RELOAD_TICKS: u64 = 40;

fn stamps_of(dir: &str) -> Stamps {
    let mut out = Vec::new();
    let Ok(entries) = std::fs::read_dir(dir) else {
        return out;
    };
    for e in entries.flatten() {
        let name = e.file_name().to_string_lossy().into_owned();
        if !name.ends_with(".carve") || name.starts_with('-') {
            continue;
        }
        let Ok(meta) = e.metadata() else { continue };
        let ns = meta
            .modified()
            .ok()
            .and_then(|m| m.duration_since(std::time::UNIX_EPOCH).ok())
            .map_or(0, |d| d.as_nanos());
        out.push((name, meta.len(), ns));
    }
    out.sort();
    out
}

/// A top-level `auto reload` line (Skript's structure).
fn wants_auto_reload(src: &str) -> bool {
    src.lines().any(|l| {
        let t = l.trim_end();
        t.eq_ignore_ascii_case("auto reload") || t.eq_ignore_ascii_case("auto reload:")
    })
}

/// Reload the scripts that have an `auto reload` line and changed on disk
/// since the last look. Returns true when `on load` triggers should run.
#[cold]
pub(crate) fn auto_reload(st: &mut State, host: &mut WasmHost<'_>) -> bool {
    let dir = st.scripts_dir();
    let now = stamps_of(&dir);
    let old = {
        let Ok(mut g) = STAMPS.lock() else { return false };
        match g.replace(now.clone()) {
            Some(o) => o,
            None => return false,
        }
    };
    let mut loaded = false;
    for (name, len, ns) in &now {
        if old.iter().any(|(n, l, t)| n == name && l == len && t == ns) {
            continue;
        }
        if !old.iter().any(|(n, _, _)| n == name) {
            continue;
        }
        let Ok(src) = std::fs::read_to_string(format!("{dir}/{name}")) else {
            continue;
        };
        if !wants_auto_reload(&src) {
            continue;
        }
        let mut store = DirStore(dir.clone());
        let allowed = |_: &str| true;
        let env = carve_core::admin::AdminEnv {
            sender: carve_core::Recipient::Console,
            allowed: &allowed,
            registered: &st.registered,
            version: env!("CARGO_PKG_VERSION"),
            api: "v0.1",
            data_folder: "plugins/data/carve",
        };
        let mut th = TickHost {
            inner: host,
            hs: &mut st.hs,
        };
        let out = carve_core::admin::admin(
            &mut st.engine,
            &mut store,
            &mut th,
            &env,
            &format!("reload {name}"),
        );
        plog(ApiLevel::Info, &format!("auto reload: {name} changed"));
        for l in &out.lines {
            plog(ApiLevel::Info, l);
        }
        loaded |= out.loaded;
    }
    loaded
}

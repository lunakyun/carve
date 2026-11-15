//! Carve's one data file: `carve.redb`, a [redb](https://docs.rs/redb) database
//! in the plugin's data folder.
//!
//! Everything the plugin keeps between runs goes through here: variables,
//! bounds, the players that have joined before, and a generic keyed table for
//! features to come. Callers queue changes (cheap, in memory) and the plugin
//! writes them in one transaction per tick or flush. redb commits are
//! crash-safe: after a crash the file holds the last committed state.
//!
//! Older versions wrote `variables.json`, `bounds.json` and `first_join.txt`.
//! [`import_legacy`] moves them in on first start.

use std::collections::BTreeMap;
use std::path::Path;
use std::sync::Mutex;

use carve_core::persist::{Changes, Legacy, read_legacy};
use carve_core::vars::Entry;
use redb::{Database, ReadableDatabase, ReadableTable, TableDefinition};

/// File name inside the data folder.
pub const FILE: &str = "carve.redb";
/// The files older versions wrote, and what they are renamed to after the import.
pub const LEGACY_FILES: [&str; 3] = ["variables.json", "bounds.json", "first_join.txt"];

const VARIABLES: TableDefinition<&str, &[u8]> = TableDefinition::new("variables");
const BOUNDS: TableDefinition<&str, &[u8]> = TableDefinition::new("bounds");
const SEEN: TableDefinition<&str, &[u8]> = TableDefinition::new("seen_players");
/// Keys are `namespace/key`.
const KV: TableDefinition<&str, &[u8]> = TableDefinition::new("kv");

type Rows = BTreeMap<String, Option<Vec<u8>>>;

/// Changes waiting for the next transaction. A key queued twice keeps the later value.
#[derive(Default)]
struct Pending {
    vars: Rows,
    bounds: Rows,
    seen: Rows,
    kv: Rows,
}

impl Pending {
    fn is_empty(&self) -> bool {
        self.vars.is_empty() && self.bounds.is_empty() && self.seen.is_empty() && self.kv.is_empty()
    }

    fn len(&self) -> usize {
        self.vars.len() + self.bounds.len() + self.seen.len() + self.kv.len()
    }
}

/// What the store held when it was opened.
#[derive(Default)]
pub struct Loaded {
    pub vars: Vec<Entry>,
    pub bounds: Vec<Entry>,
    pub seen: Vec<String>,
}

pub struct Store {
    db: Database,
    pending: Pending,
}

impl Store {
    /// Open (or create) the database file.
    pub fn open(path: &Path) -> Result<Store, String> {
        let db = Database::create(path).map_err(|e| format!("can't open {}: {e}", path.display()))?;
        // Make every table exist, so reads never fail on a fresh file.
        let tx = db.begin_write().map_err(|e| e.to_string())?;
        for def in [VARIABLES, BOUNDS, SEEN, KV] {
            tx.open_table(def).map_err(|e| e.to_string())?;
        }
        tx.commit().map_err(|e| e.to_string())?;
        Ok(Store {
            db,
            pending: Pending::default(),
        })
    }

    fn read_table(&self, def: TableDefinition<&str, &[u8]>) -> Result<Vec<Entry>, String> {
        let rx = self.db.begin_read().map_err(|e| e.to_string())?;
        let t = rx.open_table(def).map_err(|e| e.to_string())?;
        let mut out = Vec::new();
        for row in t.iter().map_err(|e| e.to_string())? {
            let (k, v) = row.map_err(|e| e.to_string())?;
            out.push((k.value().to_string(), v.value().to_vec()));
        }
        Ok(out)
    }

    /// Everything saved, without anything still queued.
    pub fn load(&self) -> Result<Loaded, String> {
        Ok(Loaded {
            vars: self.read_table(VARIABLES)?,
            bounds: self.read_table(BOUNDS)?,
            seen: self.read_table(SEEN)?.into_iter().map(|(k, _)| k).collect(),
        })
    }

    /// No variables, bounds or players saved (the generic table does not count).
    pub fn is_empty(&self) -> Result<bool, String> {
        let l = self.load()?;
        Ok(l.vars.is_empty() && l.bounds.is_empty() && l.seen.is_empty())
    }

    pub fn queue(&mut self, c: Changes) {
        self.pending.vars.extend(c.vars);
        self.pending.bounds.extend(c.bounds);
    }

    /// A player joined for the first time (`first_seen_ms`: wall clock).
    pub fn queue_seen(&mut self, uuid: &str, first_seen_ms: u64) {
        self.pending
            .seen
            .insert(uuid.to_string(), Some(first_seen_ms.to_string().into_bytes()));
    }

    /// Write (`Some`) or delete (`None`) a row of the generic table, for features that need to keep something.
    #[allow(dead_code)]
    pub fn queue_kv(&mut self, namespace: &str, key: &str, value: Option<Vec<u8>>) {
        self.pending.kv.insert(format!("{namespace}/{key}"), value);
    }

    /// A row of the generic table: the queued value if there is one, else the saved one.
    #[allow(dead_code)]
    pub fn kv_get(&self, namespace: &str, key: &str) -> Result<Option<Vec<u8>>, String> {
        let k = format!("{namespace}/{key}");
        if let Some(v) = self.pending.kv.get(&k) {
            return Ok(v.clone());
        }
        let rx = self.db.begin_read().map_err(|e| e.to_string())?;
        let t = rx.open_table(KV).map_err(|e| e.to_string())?;
        let got = t.get(k.as_str()).map_err(|e| e.to_string())?;
        Ok(got.map(|v| v.value().to_vec()))
    }

    #[cfg(test)]
    pub fn has_pending(&self) -> bool {
        !self.pending.is_empty()
    }

    /// Write everything queued in one transaction. On failure the queue is kept for the next try.
    /// Returns how many rows were written.
    pub fn flush(&mut self) -> Result<usize, String> {
        if self.pending.is_empty() {
            return Ok(0);
        }
        let n = self.pending.len();
        let tx = self.db.begin_write().map_err(|e| e.to_string())?;
        for (def, rows) in [
            (VARIABLES, &self.pending.vars),
            (BOUNDS, &self.pending.bounds),
            (SEEN, &self.pending.seen),
            (KV, &self.pending.kv),
        ] {
            if rows.is_empty() {
                continue;
            }
            let mut t = tx.open_table(def).map_err(|e| e.to_string())?;
            for (k, v) in rows {
                match v {
                    Some(bytes) => {
                        t.insert(k.as_str(), bytes.as_slice())
                            .map_err(|e| e.to_string())?;
                    }
                    None => {
                        t.remove(k.as_str()).map_err(|e| e.to_string())?;
                    }
                }
            }
        }
        tx.commit().map_err(|e| e.to_string())?;
        self.pending = Pending::default();
        Ok(n)
    }
}

/// What an import found and did, for the log.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct ImportReport {
    pub lines: Vec<String>,
    /// Files that could not be read at all: `(file name, why)`. Moved to `<name>.broken`.
    pub broken: Vec<(String, String)>,
}

fn read_old(dir: &Path, name: &str) -> Option<String> {
    std::fs::read_to_string(dir.join(name)).ok()
}

/// First start with the store: if it is empty and the old files exist, import
/// them, then rename each one to `<name>.old`. Files that don't parse are
/// renamed to `<name>.broken` and reported. The store stays untouched (and
/// the old files stay where they are) when the import can't be written.
pub fn import_legacy(store: &mut Store, dir: &Path, now_ms: u64) -> Result<ImportReport, String> {
    let mut report = ImportReport::default();
    let present: Vec<&str> = LEGACY_FILES
        .iter()
        .copied()
        .filter(|f| dir.join(f).exists())
        .collect();
    if present.is_empty() {
        return Ok(report);
    }
    if !store.is_empty()? {
        report.lines.push(format!(
            "found {} next to {FILE}, but the store already has data; the old file(s) were left alone",
            present.join(", ")
        ));
        return Ok(report);
    }
    let legacy: Legacy = read_legacy(
        read_old(dir, "variables.json").as_deref(),
        read_old(dir, "bounds.json").as_deref(),
        read_old(dir, "first_join.txt").as_deref(),
    );
    for (file, why) in &legacy.broken {
        let from = dir.join(file);
        let to = dir.join(format!("{file}.broken"));
        let _ = std::fs::rename(&from, &to);
        report.broken.push(((*file).to_string(), why.clone()));
    }
    store.queue(Changes {
        vars: legacy
            .vars
            .iter()
            .map(|(k, v)| (k.clone(), Some(v.clone())))
            .collect(),
        bounds: legacy
            .bounds
            .iter()
            .map(|(k, v)| (k.clone(), Some(v.clone())))
            .collect(),
    });
    for uuid in &legacy.seen {
        store.queue_seen(uuid, now_ms);
    }
    store.flush()?;
    for f in present {
        if report.broken.iter().any(|(b, _)| b == f) {
            continue;
        }
        let _ = std::fs::rename(dir.join(f), dir.join(format!("{f}.old")));
    }
    report.lines.push(format!(
        "imported {} variables, {} bounds and {} first-join players from the old files into {FILE} (old files renamed to .old)",
        legacy.vars.len(),
        legacy.bounds.len(),
        legacy.seen.len()
    ));
    Ok(report)
}

// ---------- the plugin's one open store ----------

static STORE: Mutex<Option<Store>> = Mutex::new(None);

/// Run `f` on the open store. `None` when it isn't open (it could not be, or the lock is taken).
pub fn with<R>(f: impl FnOnce(&mut Store) -> R) -> Option<R> {
    let mut g = STORE.lock().ok()?;
    g.as_mut().map(f)
}

/// Keep `store` as the plugin's store (replaces the old one, which is dropped and closed).
pub fn install(store: Store) {
    if let Ok(mut g) = STORE.lock() {
        *g = Some(store);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use carve_core::Variables;

    fn temp_dir(name: &str) -> std::path::PathBuf {
        let d = std::env::temp_dir().join(format!("carve-store-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&d);
        std::fs::create_dir_all(&d).unwrap();
        d
    }

    #[test]
    fn rows_survive_a_reopen() {
        let dir = temp_dir("reopen");
        let path = dir.join(FILE);
        {
            let mut s = Store::open(&path).unwrap();
            assert!(s.is_empty().unwrap());
            s.queue(Changes {
                vars: vec![
                    ("a".into(), Some(b"1".to_vec())),
                    ("b".into(), Some(b"2".to_vec())),
                ],
                bounds: vec![("spawn".into(), Some(b"{}".to_vec()))],
            });
            s.queue_seen("uuid-1", 5);
            assert!(s.has_pending());
            assert_eq!(s.flush().unwrap(), 4);
            assert!(!s.has_pending());
            assert_eq!(s.flush().unwrap(), 0, "nothing queued, nothing written");
        }
        let s = Store::open(&path).unwrap();
        let l = s.load().unwrap();
        assert_eq!(l.vars.len(), 2);
        assert_eq!(l.bounds.len(), 1);
        assert_eq!(l.seen, vec!["uuid-1".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_key_queued_twice_keeps_the_later_value_and_deletes_work() {
        let dir = temp_dir("later");
        let mut s = Store::open(&dir.join(FILE)).unwrap();
        s.queue(Changes {
            vars: vec![
                ("k".into(), Some(b"old".to_vec())),
                ("gone".into(), Some(b"x".to_vec())),
            ],
            bounds: vec![],
        });
        s.queue(Changes {
            vars: vec![("k".into(), Some(b"new".to_vec()))],
            bounds: vec![],
        });
        s.flush().unwrap();
        s.queue(Changes {
            vars: vec![("gone".into(), None), ("never-there".into(), None)],
            bounds: vec![],
        });
        s.flush().unwrap();
        let l = s.load().unwrap();
        assert_eq!(l.vars, vec![("k".to_string(), b"new".to_vec())]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn generic_table_reads_queued_then_saved() {
        let dir = temp_dir("kv");
        let mut s = Store::open(&dir.join(FILE)).unwrap();
        assert_eq!(s.kv_get("feature", "x").unwrap(), None);
        s.queue_kv("feature", "x", Some(b"1".to_vec()));
        assert_eq!(
            s.kv_get("feature", "x").unwrap(),
            Some(b"1".to_vec()),
            "queued value is visible"
        );
        s.flush().unwrap();
        assert_eq!(s.kv_get("feature", "x").unwrap(), Some(b"1".to_vec()));
        s.queue_kv("feature", "x", None);
        s.flush().unwrap();
        assert_eq!(s.kv_get("feature", "x").unwrap(), None);
        assert!(s.is_empty().unwrap(), "the generic table does not count as data");
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn imports_the_three_old_files_once() {
        let dir = temp_dir("import");
        let mut v = Variables::default();
        v.set("score".into(), carve_core::Value::Num(7.0));
        v.set("name".into(), carve_core::Value::Text("Void".into()));
        std::fs::write(dir.join("variables.json"), v.to_json()).unwrap();
        std::fs::write(
            dir.join("bounds.json"),
            r#"{"bounds":[{"id":"Spawn","world":"world","min":[0,0,0],"max":[9,9,9],"owners":[],"members":[],"flags":{"pvp":false}}]}"#,
        )
        .unwrap();
        std::fs::write(dir.join("first_join.txt"), "uuid-a\nuuid-b\n").unwrap();

        let mut s = Store::open(&dir.join(FILE)).unwrap();
        let r = import_legacy(&mut s, &dir, 1000).unwrap();
        assert_eq!(r.lines.len(), 1);
        assert!(
            r.lines[0].contains("2 variables, 1 bounds and 2 first-join"),
            "{:?}",
            r.lines
        );
        for f in LEGACY_FILES {
            assert!(!dir.join(f).exists(), "{f} is renamed");
            assert!(dir.join(format!("{f}.old")).exists(), "{f}.old is there");
        }
        let l = s.load().unwrap();
        let (vars, bad) = Variables::from_entries(l.vars);
        assert!(bad.is_empty());
        assert_eq!(vars.get("score"), carve_core::Value::Num(7.0));
        assert_eq!(vars.get("name"), carve_core::Value::Text("Void".into()));
        let (bounds, bad) = carve_core::skbee_bound::Bounds::from_entries(l.bounds);
        assert!(bad.is_empty());
        assert_eq!(bounds.len(), 1);
        assert_eq!(l.seen, vec!["uuid-a".to_string(), "uuid-b".to_string()]);

        // A second run finds nothing to import.
        let r = import_legacy(&mut s, &dir, 2000).unwrap();
        assert!(r.lines.is_empty());
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn old_files_are_left_alone_when_the_store_has_data() {
        let dir = temp_dir("leave");
        std::fs::write(dir.join("first_join.txt"), "uuid-old\n").unwrap();
        let mut s = Store::open(&dir.join(FILE)).unwrap();
        s.queue_seen("uuid-new", 1);
        s.flush().unwrap();
        let r = import_legacy(&mut s, &dir, 1).unwrap();
        assert!(r.lines[0].contains("already has data"));
        assert!(dir.join("first_join.txt").exists());
        assert_eq!(s.load().unwrap().seen, vec!["uuid-new".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn a_broken_file_is_moved_aside_and_the_rest_is_imported() {
        let dir = temp_dir("broken");
        std::fs::write(dir.join("variables.json"), "{ not json").unwrap();
        std::fs::write(dir.join("first_join.txt"), "uuid-a\n").unwrap();
        let mut s = Store::open(&dir.join(FILE)).unwrap();
        let r = import_legacy(&mut s, &dir, 1).unwrap();
        assert_eq!(r.broken.len(), 1);
        assert_eq!(r.broken[0].0, "variables.json");
        assert!(dir.join("variables.json.broken").exists());
        assert!(dir.join("first_join.txt.old").exists());
        assert_eq!(s.load().unwrap().seen, vec!["uuid-a".to_string()]);
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn no_old_files_means_no_import() {
        let dir = temp_dir("none");
        let mut s = Store::open(&dir.join(FILE)).unwrap();
        let r = import_legacy(&mut s, &dir, 1).unwrap();
        assert_eq!(r, ImportReport::default());
        let _ = std::fs::remove_dir_all(&dir);
    }
}

//! What the plugin keeps between runs, and how it was kept before.
//!
//! The plugin saves everything in one store (`carve.redb`, see the plugin's
//! `store` module). The core only decides what changed ([`Changes`]) and reads
//! the three files older versions wrote (`variables.json`, `bounds.json`,
//! `first_join.txt`) so they can be moved into the store once ([`read_legacy`]).

use crate::skbee_bound::Bounds;
use crate::vars::{Change, Entry, Variables};

/// Keys that changed since the last write, ready for one transaction.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Changes {
    /// Variable name and its new value (`None`: deleted).
    pub vars: Vec<Change>,
    /// Lower-case bound id and the bound (`None`: deleted).
    pub bounds: Vec<Change>,
}

impl Changes {
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty() && self.bounds.is_empty()
    }
}

/// The old files' content, turned into store rows.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Legacy {
    pub vars: Vec<Entry>,
    pub bounds: Vec<Entry>,
    /// Uuids from `first_join.txt`.
    pub seen: Vec<String>,
    /// Files that were there but could not be read, as `(file name, why)`.
    pub broken: Vec<(&'static str, String)>,
}

impl Legacy {
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty() && self.bounds.is_empty() && self.seen.is_empty()
    }
}

/// Read the text of the old files (`None`: the file is not there).
pub fn read_legacy(vars: Option<&str>, bounds: Option<&str>, seen: Option<&str>) -> Legacy {
    let mut out = Legacy::default();
    if let Some(text) = vars {
        match Variables::from_json(text) {
            Ok(v) => out.vars = v.entries(),
            Err(e) => out.broken.push(("variables.json", e)),
        }
    }
    if let Some(text) = bounds {
        match Bounds::from_json(text) {
            Ok(b) => out.bounds = b.entries(),
            Err(e) => out.broken.push(("bounds.json", e)),
        }
    }
    if let Some(text) = seen {
        out.seen = text
            .lines()
            .map(str::trim)
            .filter(|l| !l.is_empty())
            .map(str::to_string)
            .collect();
    }
    out
}

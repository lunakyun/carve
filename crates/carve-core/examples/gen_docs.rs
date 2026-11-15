//! Generates the syntax reference: `docs/reference/*.md` and
//! `docs/public/syntax.json`.
//!
//! Run with `cargo run -p carve-core --example gen_docs` (or `npm run docs:gen`).
//! Output is sorted and only rewritten when it changes, so running it twice
//! gives the same files.
//!
//! Sources:
//! - effect and condition patterns from the registries in `carve_core::docs_meta::registries()`,
//!   plus any `pub const *_PATTERNS` found in `src/` that the list misses;
//! - event names from `EventKind::name` and type names from `Ty::name` (read from the sources);
//! - names, descriptions, examples and hand-parsed syntax from `carve_core::docs_meta::DOCS`.

use std::collections::{BTreeMap, BTreeSet, HashSet};
use std::fs;
use std::path::{Path, PathBuf};

use carve_core::docs_meta::{self, Cat, DOCS, Registry};
use serde_json::{Value, json};

struct Entry {
    id: String,
    name: String,
    cat: Cat,
    patterns: Vec<String>,
    desc: String,
    example: String,
    since: String,
    page: String,
    /// "registry", "hand" or "registry+hand".
    source: &'static str,
}

fn main() {
    let core = Path::new(env!("CARGO_MANIFEST_DIR"));
    let root = core.join("../..");
    let src = core.join("src");
    let docs = root.join("docs");

    let mut warnings = Vec::new();
    let mut regs = docs_meta::registries();
    let known: HashSet<String> = regs.iter().map(|r| r.name.to_string()).collect();
    for r in scan_registries(&src) {
        if !known.contains(r.0.as_str()) {
            warnings.push(format!(
                "{} is not listed in docs_meta::registries(); read from the source",
                r.0
            ));
            regs.push(Registry {
                name: Box::leak(r.0.into_boxed_str()),
                cat: r.1,
                patterns: r.2.into_iter().map(|s| &*Box::leak(s.into_boxed_str())).collect(),
            });
        }
    }

    let mut entries = Vec::new();
    for cat in [Cat::Effect, Cat::Condition] {
        entries.extend(registry_entries(cat, &regs));
    }
    for cat in [Cat::Event, Cat::Expression, Cat::Type, Cat::Function] {
        entries.extend(DOCS.iter().filter(|d| d.cat == cat).map(|d| Entry {
            id: d.id.to_string(),
            name: d.name.to_string(),
            cat,
            patterns: d.patterns.iter().map(|s| s.to_string()).collect(),
            desc: d.desc.to_string(),
            example: d.example.to_string(),
            since: d.since.to_string(),
            page: d.page.to_string(),
            source: "hand",
        }));
    }

    // Events and types the compiler knows but the table does not describe.
    let mut events: BTreeSet<String> = scan_names(&src.join("compile.rs"), "EventKind::")
        .into_iter()
        .collect();
    // The extra event tables (`EventKind::Ext*`) and the item events.
    events.extend(scan_names(&src.join("events_blocks_players.rs"), "ExtEvent::"));
    events.extend(
        carve_core::events_game::SPECS
            .iter()
            .map(|s| s.names[0].to_string()),
    );
    events.extend(carve_core::events_player_server::PlayerServerEvent::all().map(|e| e.name().to_string()));
    events.extend(carve_core::events_movement::MvEvent::all().map(|e| e.name().to_string()));
    events.extend(scan_table_names(
        &src.join("events_world_vehicles.rs"),
        "ExtEvent::",
    ));
    events.extend(carve_core::events_items::NAMES.split(", ").map(str::to_string));
    for name in events {
        let pat = format!("on {name}");
        let covered = DOCS
            .iter()
            .any(|d| d.cat == Cat::Event && d.patterns.iter().any(|p| covers(p, &name)));
        if !covered {
            warnings.push(format!("event `{name}` has no entry in docs_meta::DOCS"));
            entries.push(auto(Cat::Event, &pat, "registry"));
        }
    }
    for name in scan_names(&src.join("expr.rs"), "Ty::") {
        let covered = DOCS
            .iter()
            .any(|d| d.cat == Cat::Type && d.patterns.contains(&name.as_str()));
        if !covered {
            warnings.push(format!("type `{name}` has no entry in docs_meta::DOCS"));
            entries.push(auto(Cat::Type, &name, "registry"));
        }
    }

    entries.sort_by(|a, b| (a.cat, a.name.to_lowercase(), &a.id).cmp(&(b.cat, b.name.to_lowercase(), &b.id)));

    let mut changed = 0;
    let mut counts = BTreeMap::new();
    for cat in Cat::ALL {
        let list: Vec<&Entry> = entries.iter().filter(|e| e.cat == cat).collect();
        counts.insert(cat.slug(), list.len());
        let path = docs.join("reference").join(format!("{}.md", cat.slug()));
        changed += write_if_changed(&path, &render_page(cat, &list)) as usize;
    }
    let json = render_json(&entries);
    changed += write_if_changed(&docs.join("public/syntax.json"), &json) as usize;

    for w in &warnings {
        eprintln!("warning: {w}");
    }
    let summary: Vec<String> = counts.iter().map(|(k, v)| format!("{k} {v}")).collect();
    eprintln!(
        "gen_docs: {} entries ({}), {changed} file(s) changed",
        entries.len(),
        summary.join(", ")
    );
}

/// Effects or conditions: table entries claim registry patterns, the rest
/// become entries of their own.
fn registry_entries(cat: Cat, regs: &[Registry]) -> Vec<Entry> {
    let in_reg: BTreeSet<&str> = regs
        .iter()
        .filter(|r| r.cat == cat)
        .flat_map(|r| r.patterns.iter().copied())
        .collect();
    let mut claimed = HashSet::new();
    let mut out = Vec::new();
    for d in DOCS.iter().filter(|d| d.cat == cat) {
        let mut patterns = Vec::new();
        let (mut reg, mut hand) = (false, false);
        for p in d.patterns {
            if in_reg.contains(p) {
                if !claimed.insert(*p) {
                    continue;
                }
                reg = true;
            } else {
                hand = true;
            }
            patterns.push(p.to_string());
        }
        if patterns.is_empty() {
            continue;
        }
        out.push(Entry {
            id: d.id.to_string(),
            name: d.name.to_string(),
            cat,
            patterns,
            desc: d.desc.to_string(),
            example: d.example.to_string(),
            since: d.since.to_string(),
            page: d.page.to_string(),
            source: match (reg, hand) {
                (true, true) => "registry+hand",
                (true, false) => "registry",
                _ => "hand",
            },
        });
    }
    for p in in_reg {
        if !claimed.contains(p) {
            eprintln!(
                "warning: {} pattern `{p}` has no entry in docs_meta::DOCS",
                cat.name()
            );
            out.push(auto(cat, p, "registry"));
        }
    }
    out
}

fn auto(cat: Cat, pattern: &str, source: &'static str) -> Entry {
    Entry {
        id: slug(pattern),
        name: pattern.to_string(),
        cat,
        patterns: vec![pattern.to_string()],
        desc: String::new(),
        example: String::new(),
        since: "unreleased".into(),
        page: String::new(),
        source,
    }
}

fn slug(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(c.to_ascii_lowercase());
        } else if !out.ends_with('_') && !out.is_empty() {
            out.push('_');
        }
    }
    out.trim_end_matches('_').to_string()
}

/// `pub const X_PATTERNS: &[(&str, EffKind)] = &[ ... ];` in every source file,
/// as `(module::X_PATTERNS, category, patterns)`.
fn scan_registries(src: &Path) -> Vec<(String, Cat, Vec<String>)> {
    let mut files: Vec<PathBuf> = fs::read_dir(src)
        .expect("read src")
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.extension().is_some_and(|x| x == "rs"))
        .collect();
    files.sort();
    let mut out = Vec::new();
    for f in files {
        let text = fs::read_to_string(&f).unwrap_or_default();
        let module = f.file_stem().unwrap().to_string_lossy().to_string();
        let mut rest = text.as_str();
        while let Some(i) = rest.find("pub const ") {
            rest = &rest[i + "pub const ".len()..];
            let name: String = rest
                .chars()
                .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
                .collect();
            if !name.ends_with("_PATTERNS") {
                continue;
            }
            let Some(end) = rest.find("\n];") else { break };
            let block = &rest[..end];
            rest = &rest[end..];
            let head = block.lines().next().unwrap_or("");
            let cat = if head.contains("CondKind") {
                Cat::Condition
            } else if head.contains("EffKind") || head.contains("XpOp") {
                Cat::Effect
            } else {
                continue;
            };
            out.push((format!("{module}::{name}"), cat, tuple_strings(block)));
        }
    }
    out
}

/// First string literal of each `(` tuple in a const block.
fn tuple_strings(block: &str) -> Vec<String> {
    let mut out = Vec::new();
    let body = block.split_once("= &[").map(|x| x.1).unwrap_or("");
    let mut rest = body;
    while let Some(i) = rest.find('(') {
        rest = rest[i + 1..].trim_start();
        if let Some(s) = rest.strip_prefix('"') {
            let mut lit = String::new();
            let mut chars = s.char_indices();
            let mut end = s.len();
            while let Some((i, c)) = chars.next() {
                match c {
                    '\\' => lit.extend(chars.next().map(|x| x.1)),
                    '"' => {
                        end = i + 1;
                        break;
                    }
                    c => lit.push(c),
                }
            }
            rest = &s[end..];
            out.push(lit);
        }
    }
    out
}

/// Names from `Prefix::Variant => "name",` match arms, sorted and deduped.
/// First name per variant in a `("name", Enum::Variant)` table.
fn scan_table_names(file: &Path, prefix: &str) -> Vec<String> {
    let text = fs::read_to_string(file).unwrap_or_default();
    let mut seen = HashSet::new();
    let mut out = Vec::new();
    for line in text.lines() {
        let Some(rest) = line.trim().strip_prefix("(\"") else {
            continue;
        };
        let Some((name, tail)) = rest.split_once("\", ") else {
            continue;
        };
        let Some(variant) = tail.strip_prefix(prefix) else {
            continue;
        };
        let variant = variant.trim_end_matches([')', ',']).to_string();
        if seen.insert(variant) {
            out.push(name.to_string());
        }
    }
    out
}

fn scan_names(file: &Path, prefix: &str) -> Vec<String> {
    let text = fs::read_to_string(file).unwrap_or_default();
    let mut out = BTreeSet::new();
    for line in text.lines() {
        let l = line.trim();
        let Some(rest) = l.strip_prefix(prefix) else {
            continue;
        };
        let Some((_, rhs)) = rest.split_once("=> \"") else {
            continue;
        };
        if let Some((name, tail)) = rhs.split_once('"')
            && tail.trim() == ","
        {
            out.insert(name.to_string());
        }
    }
    out.into_iter().collect()
}

fn render_page(cat: Cat, list: &[&Entry]) -> String {
    let mut s = String::new();
    s.push_str("---\n");
    s.push_str(&format!("title: {}\n", cat.title()));
    s.push_str("editLink: false\n");
    s.push_str("---\n\n");
    s.push_str("<!-- Generated by `cargo run -p carve-core --example gen_docs`. Do not edit. -->\n\n");
    s.push_str(&format!("# {}\n\n", cat.title()));
    s.push_str(match cat {
        Cat::Event => "Events and other top-level lines: what starts a trigger.",
        Cat::Effect => "Effects are lines that do something.",
        Cat::Condition => "Conditions go after `if`, `else if` and `while`, or on a line of their own.",
        Cat::Expression => "Expressions are values you put into effects and conditions.",
        Cat::Type => "Value types, used in function parameters and return types.",
        Cat::Function => "Built-in functions, and how to write your own.",
    });
    s.push_str(&format!(
        " {} entries. Search all of them on the [reference home](./index.md).\n\n",
        list.len()
    ));
    s.push_str(
        "In a pattern, `%type%` is a value, `[...]` is optional and `(a|b)` means one of the words.\n\n",
    );
    for e in list {
        s.push_str(&format!("## {} {{#{}}}\n\n", escape_md(&e.name), e.id));
        s.push_str("```carve\n");
        for p in &e.patterns {
            s.push_str(p);
            s.push('\n');
        }
        s.push_str("```\n\n");
        if e.desc.is_empty() {
            s.push_str("_No description yet._\n\n");
        } else {
            s.push_str(&e.desc);
            s.push_str("\n\n");
        }
        if !e.example.is_empty() {
            s.push_str("```carve\n");
            s.push_str(&e.example);
            s.push_str("\n```\n\n");
        }
        let mut meta = vec![format!("Since **{}**", e.since)];
        if !e.page.is_empty() {
            meta.push(format!(
                "Guide: [{}](../{}.md)",
                e.page.rsplit('/').next().unwrap_or(""),
                e.page
            ));
        }
        s.push_str(&meta.join(" · "));
        s.push_str("\n\n");
    }
    s
}

/// Keeps names from turning into Markdown or Vue syntax.
fn escape_md(s: &str) -> String {
    s.replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('*', "\\*")
        .replace('_', "\\_")
}

fn render_json(entries: &[Entry]) -> String {
    let list: Vec<Value> = entries
        .iter()
        .map(|e| {
            json!({
                "id": e.id,
                "name": e.name,
                "category": e.cat.name(),
                "patterns": e.patterns,
                "description": e.desc,
                "example": e.example,
                "since": e.since,
                "page": e.page,
                "source": e.source,
            })
        })
        .collect();
    let cats: Vec<Value> = Cat::ALL
        .iter()
        .map(|c| json!({ "id": c.name(), "title": c.title(), "page": format!("reference/{}", c.slug()) }))
        .collect();
    let doc = json!({
        "version": env!("CARGO_PKG_VERSION"),
        "categories": cats,
        "entries": list,
    });
    let mut out = serde_json::to_string_pretty(&doc).expect("json");
    out.push('\n');
    out
}

fn write_if_changed(path: &Path, content: &str) -> bool {
    if fs::read_to_string(path).is_ok_and(|old| old == content) {
        return false;
    }
    if let Some(dir) = path.parent() {
        fs::create_dir_all(dir).expect("create output dir");
    }
    fs::write(path, content).expect("write output");
    true
}

/// Whether an event pattern like `on (right click|rightclick) [with %item%]`
/// covers the event name `right click`.
fn covers(pattern: &str, name: &str) -> bool {
    let Some(rest) = pattern.strip_prefix("on ") else {
        return false;
    };
    let word_end = |r: &str| r.is_empty() || r.starts_with([' ', '|', ')', '[']);
    rest.strip_prefix(name).is_some_and(word_end)
        || ["(", "|"].iter().any(|open| {
            rest.match_indices(&format!("{open}{name}"))
                .any(|(i, m)| word_end(&rest[i + m.len()..]))
        })
}

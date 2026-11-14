//! Micro benchmarks for the Carve core. No extra crates.
//!
//! Run with: `cargo run --release -p carve-core --example bench`
//!
//! The host here does no real work. Every call bumps an atomic counter, which
//! stands in for the cost of crossing the wasm component boundary, and is
//! counted so we can report host calls per run.

use std::hint::black_box;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Instant;

use carve_core::compile::compile_file;
use carve_core::{Engine, EventCtx, EventKind, Host, LogLevel, PlayerRef, Recipient};

static CALLS: AtomicU64 = AtomicU64::new(0);

fn hit() {
    CALLS.fetch_add(1, Ordering::Relaxed);
}

struct BenchHost {
    uuid: String,
    name: String,
}

impl Host for BenchHost {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        hit();
        black_box(text);
    }
    fn broadcast(&mut self, text: &str) {
        hit();
        black_box(text);
    }
    fn send_title(&mut self, _p: &PlayerRef, title: &str, sub: Option<&str>) {
        hit();
        black_box((title, sub));
    }
    fn send_action_bar(&mut self, _p: &PlayerRef, text: &str) {
        hit();
        black_box(text);
    }
    fn heal(&mut self, _p: &PlayerRef) {
        hit();
    }
    fn set_health(&mut self, _p: &PlayerRef, h: f64) {
        hit();
        black_box(h);
    }
    fn kill(&mut self, _p: &PlayerRef) {
        hit();
    }
    fn teleport(&mut self, _p: &PlayerRef, x: f64, y: f64, z: f64) {
        hit();
        black_box((x, y, z));
    }
    fn execute_command(&mut self, c: &str) {
        hit();
        black_box(c);
    }
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        hit();
        (uuid == self.uuid).then(|| self.name.clone())
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        hit();
        name.eq_ignore_ascii_case(&self.name).then(|| PlayerRef {
            uuid: self.uuid.clone(),
            name: self.name.clone(),
        })
    }
    fn has_permission(&mut self, _p: &PlayerRef, _node: &str) -> bool {
        hit();
        true
    }
    fn schedule_wait(&mut self, _ticks: u64, _token: u64) {
        hit();
    }
    fn now(&mut self) -> u64 {
        hit();
        0
    }
    fn log(&mut self, _level: LogLevel, msg: &str) {
        hit();
        black_box(msg);
    }
}

impl BenchHost {
    /// What the wasm plugin does to build an event ctx: read the player's
    /// UUID and name from the event (3 host calls in the real plugin).
    fn event_ctx(&mut self) -> EventCtx {
        hit();
        hit();
        hit();
        EventCtx::player(PlayerRef {
            uuid: self.uuid.clone(),
            name: self.name.clone(),
        })
    }
}

fn host() -> BenchHost {
    BenchHost {
        uuid: "069a79f4-44e9-4726-a5be-fca90e38aaf5".into(),
        name: "Steve".into(),
    }
}

fn engine(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("bench.carve".into(), src.into())]);
    if let Some(err) = errs.first() {
        eprintln!("bench script failed to compile: {err}");
        std::process::exit(1);
    }
    e
}

struct Row {
    name: &'static str,
    ns: f64,
    calls: f64,
}

fn measure(name: &'static str, iters: u32, mut f: impl FnMut()) -> Row {
    for _ in 0..iters / 10 + 1 {
        f();
    }
    // Best of 5 rounds, to smooth out noise from the OS.
    let mut best = f64::MAX;
    let mut calls = 0.0;
    for _ in 0..5 {
        CALLS.store(0, Ordering::Relaxed);
        let start = Instant::now();
        for _ in 0..iters {
            f();
        }
        best = best.min(start.elapsed().as_nanos() as f64);
        calls = CALLS.load(Ordering::Relaxed) as f64;
    }
    Row {
        name,
        ns: best / f64::from(iters),
        calls: calls / f64::from(iters),
    }
}

fn case_a() -> String {
    let mut s = String::from("on join:\n    add 1 to {joins::%player%}\n");
    let lines = [
        r#"send "Welcome back, %player%!" to player"#,
        r#"send "You joined %{joins::%player%}% times." to player"#,
        r#"send title "Hi %player%" with subtitle "Visit %{joins::%player%}%" to player"#,
        r#"send action bar "Have fun, %name of player%" to player"#,
        r#"broadcast "%player% joined (%{joins::%player%}%)""#,
        r#"set {last::%player%} to player"#,
        r#"send "Last seen: %{last::%player%}%" to player"#,
        r#"heal player"#,
        r#"set health of player to 20"#,
        r#"send "Health restored, %player%.""#,
    ];
    // 1 + 19 = 20 effects
    for i in 0..19 {
        s.push_str("    ");
        s.push_str(lines[i % lines.len()]);
        s.push('\n');
    }
    s
}

fn case_b() -> String {
    let mut s = String::from("on join:\n    set {_x} to 0\n");
    for _ in 0..50 {
        s.push_str("    set {_x} to {_x} + 1\n");
    }
    s
}

fn case_c() -> String {
    let mut s = String::from("on join:\n    set {_n} to 9\n");
    for i in 0..10 {
        let kw = if i == 0 { "if" } else { "else if" };
        s.push_str(&format!("    {kw} {{_n}} = {i}:\n        send \"branch {i}\"\n"));
    }
    s
}

fn main() {
    let mut rows = Vec::new();

    for (name, src) in [
        ("a) join, 20 effects", case_a()),
        ("b) 50x set {_x}+1", case_b()),
        ("c) if/else-if x10", case_c()),
    ] {
        let mut e = engine(&src);
        let mut h = host();
        rows.push(measure(name, 20_000, || {
            let ctx = h.event_ctx();
            black_box(e.fire(EventKind::Join, ctx, &mut h));
        }));
    }

    // CARVE_EXAMPLES overrides the folder, for running the bench under a wasm runtime.
    let dir = std::env::var("CARVE_EXAMPLES")
        .unwrap_or_else(|_| format!("{}/../../examples", env!("CARGO_MANIFEST_DIR")));
    let mut files: Vec<(String, String)> = Vec::new();
    if let Ok(rd) = std::fs::read_dir(&dir) {
        for entry in rd.flatten() {
            let p = entry.path();
            if p.extension().is_some_and(|x| x == "carve")
                && let Ok(src) = std::fs::read_to_string(&p)
            {
                files.push((p.display().to_string(), src));
            }
        }
    }
    files.sort();
    let nfiles = files.len();
    rows.push(measure("d) compile examples x1", 1_000, || {
        for (name, src) in &files {
            black_box(compile_file(name, src).is_ok());
        }
    }));

    let mut e = engine("on join:\n    send \"hi\"\n");
    let mut h = host();
    rows.push(measure("e) fire, no triggers", 10_000, || {
        // Same guard as the wasm event handler: only build the ctx when needed.
        if e.table().has_event(EventKind::Quit) {
            let ctx = h.event_ctx();
            black_box(e.fire(EventKind::Quit, ctx, &mut h));
        }
    }));

    println!("{:<26} {:>12} {:>14}", "case", "ns/run", "host calls/run");
    for r in &rows {
        println!("{:<26} {:>12.0} {:>14.1}", r.name, r.ns, r.calls);
    }
    println!("(d compiles {nfiles} example file(s) per run)");
}

//! Scripts that tried to freeze the server. Each one must stop with an error
//! that names the line, quickly, and without a stack overflow or a huge
//! allocation. The same commands run in-game in `tests/ingame/areas/freeze.js`.

use carve_core::compile::EventKind;
use carve_core::*;
use std::time::{Duration, Instant};

#[derive(Default)]
struct Mock {
    sent: Vec<String>,
    waits: Vec<(u64, u64)>,
    logs: Vec<(LogLevel, String)>,
    spawned: usize,
    /// Every spawn takes this long, like a busy server.
    spawn_cost: Duration,
}

impl Mock {
    fn errors(&self) -> Vec<&String> {
        self.logs
            .iter()
            .filter(|(l, _)| *l == LogLevel::Error)
            .map(|(_, m)| m)
            .collect()
    }
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.sent.push(text.to_string());
    }
    fn broadcast(&mut self, _text: &str) {}
    fn send_title(&mut self, _p: &PlayerRef, _t: &str, _s: Option<&str>) {}
    fn send_action_bar(&mut self, _p: &PlayerRef, _t: &str) {}
    fn heal(&mut self, _p: &PlayerRef) {}
    fn set_health(&mut self, _p: &PlayerRef, _h: f64) {}
    fn kill(&mut self, _p: &PlayerRef) {}
    fn teleport(&mut self, _p: &PlayerRef, _x: f64, _y: f64, _z: f64) {}
    fn execute_command(&mut self, _c: &str) {}
    fn resolve_player(&mut self, _uuid: &str) -> Option<String> {
        None
    }
    fn find_player(&mut self, _name: &str) -> Option<PlayerRef> {
        None
    }
    fn has_permission(&mut self, _p: &PlayerRef, _node: &str) -> bool {
        true
    }
    fn schedule_wait(&mut self, ticks: u64, token: u64) {
        self.waits.push((ticks, token));
    }
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn spawn_entity(&mut self, _kind: &str, _at: &world_effects::FxPos) -> Option<EntityRef> {
        self.spawned += 1;
        if !self.spawn_cost.is_zero() {
            std::thread::sleep(self.spawn_cost);
        }
        None
    }
}

fn engine(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.to_string())]);
    assert!(
        errs.is_empty(),
        "load errors: {:?}",
        errs.iter().map(|e| &e.msg).collect::<Vec<_>>()
    );
    e
}

/// Debug builds use far bigger stack frames than the plugin does, so the
/// 64-level recursion needs more room than a test thread has.
fn big<T: Send + 'static>(f: impl FnOnce() -> T + Send + 'static) -> T {
    std::thread::Builder::new()
        .stack_size(256 << 20)
        .spawn(f)
        .unwrap()
        .join()
        .unwrap_or_else(|e| std::panic::resume_unwind(e))
}

/// Run `/x` and return the host. The run must be quick.
fn run(src: &str) -> (Mock, Duration) {
    let src = src.to_string();
    big(move || {
        let mut e = engine(&src);
        let mut h = Mock::default();
        let t0 = Instant::now();
        e.run_command("x", Recipient::Console, "", &mut h);
        (h, t0.elapsed())
    })
}

fn cmd(body: &str) -> String {
    let mut s = String::from("command /x:\n    trigger:\n");
    for l in body.lines() {
        s.push_str("        ");
        s.push_str(l);
        s.push('\n');
    }
    s
}

/// The run stopped with an error that names `t.carve` and a line and mentions `what`.
fn stopped(h: &Mock, what: &str) {
    let errs = h.errors();
    assert!(
        errs.iter().any(|m| m.starts_with("t.carve:") && m.contains(what)),
        "wanted an error with '{what}', got {errs:?}"
    );
}

const QUICK: Duration = Duration::from_millis(1500);

#[test]
fn endless_loops_stop() {
    for body in [
        "while true is true:\n    set {_x} to 1",
        "loop 100000 times:\n    loop 100000 times:\n        set {_x} to 1",
        "loop 2147483647 times:\n    set {_x} to 1",
        "set {_n} to 1000000000000\nloop {_n} times:\n    set {_x} to 1",
        "while true is true:\n    wait 0 ticks",
    ] {
        let (h, took) = run(&cmd(body));
        stopped(&h, "more than 100000 instructions");
        assert!(took < QUICK, "{body}: {took:?}");
    }
}

#[test]
fn odd_loop_counts_do_nothing() {
    let (h, _) = run(&cmd(
        "set {_n} to -5\nloop {_n} times:\n    send \"ran\"\nsend \"done\"",
    ));
    assert_eq!(h.sent, ["done"]);
    assert!(h.errors().is_empty());
}

#[test]
fn recursion_stops() {
    let src = "function f(n: number) :: number:\n    return f({_n} + 1)\n\ncommand /x:\n    trigger:\n        send \"%f(1)%\"\n";
    let (h, _) = run(src);
    stopped(&h, "levels deep");
    let src = "function a(n: number) :: number:\n    return b({_n})\n\nfunction b(n: number) :: number:\n    return a({_n})\n\ncommand /x:\n    trigger:\n        send \"%a(1)%\"\n";
    let (h, _) = run(src);
    stopped(&h, "levels deep");
    // Two calls per level: still stopped by the budget, not by running for ever.
    let src = "function f(n: number):\n    if {_n} > 0:\n        f({_n} - 1)\n        f({_n} - 1)\n\ncommand /x:\n    trigger:\n        f(60)\n";
    let (h, took) = run(src);
    stopped(&h, "more than 100000 instructions");
    assert!(took < QUICK);
}

#[test]
fn text_cannot_double_forever() {
    let (h, took) = run(&cmd(
        "set {_t} to \"abcdefghijklmnop\"\nloop 60 times:\n    set {_t} to \"%{_t}%%{_t}%\"\nsend \"never\"",
    ));
    stopped(&h, "longer than");
    assert!(h.sent.is_empty());
    assert!(took < QUICK, "{took:?}");
    // A replace that would blow up is refused before it is built.
    let (h, took) = run(&cmd(
        "set {_t} to \"a\" repeated 1000000 times\nset {_r} to \"b\" repeated 1000 times\nreplace all \"a\" in {_t} with {_r}\nsend \"never\"",
    ));
    stopped(&h, "longer than");
    assert!(took < QUICK, "{took:?}");
    let (h, _) = run(&cmd("set {_t} to \"a\" repeated 5000000 times\nsend \"never\""));
    stopped(&h, "longer than");
}

#[test]
fn big_lists_are_charged() {
    for body in [
        "set {_l::*} to numbers from 1 to 1000000\nset {_t} to join {_l::*} with \", \"\nsend \"never\"",
        "set {_l::*} to numbers from 1 to 1000000000000\nsend \"never\"",
        "loop integers from 1 to 1000000000000:\n    set {_x} to 1",
        "set {_l::*} to numbers from 1 to 150000\nset {_s::*} to sorted {_l::*}\nset {_h::*} to shuffled {_s::*}\nsend \"never\"",
        "loop 100000000 times:\n    set {_l::%loop-number%} to loop-number",
    ] {
        let (h, took) = run(&cmd(body));
        assert!(h.sent.is_empty(), "{body}");
        assert!(!h.errors().is_empty(), "{body}");
        assert!(took < QUICK, "{body}: {took:?}");
    }
    // A list that fits is fine.
    let (h, _) = run(&cmd(
        "set {_l::*} to numbers from 1 to 20000\nsend \"%size of {_l::*}%\"",
    ));
    assert_eq!(h.sent, ["20000"]);
}

#[test]
fn regex_cannot_hang_or_overflow() {
    for body in [
        "if \"aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa!\" matches \"(a+)+$\":\n    send \"never\"",
        "set {_t} to \"a\" repeated 1000000 times\nif {_t} matches \"(a|aa)*b\":\n    send \"never\"",
        "set {_t} to \"a\" repeated 100000 times\nreplace all regex \"(a*)*b\" in {_t} with \"c\"",
    ] {
        let (h, took) = run(&cmd(body));
        stopped(&h, "too complex");
        assert!(took < QUICK, "{body}: {took:?}");
    }
    let pattern = "(".repeat(2500);
    let (h, _) = run(&cmd(&format!(
        "if \"a\" matches \"{pattern}\":\n    send \"never\""
    )));
    assert!(!h.errors().is_empty());
}

#[test]
fn waits_are_capped() {
    // One file may have `MAX_WAITS_PER_FILE` runs waiting; the next is stopped.
    let src = "on join:\n    wait 1 hour\n";
    let mut e = engine(src);
    let mut h = Mock::default();
    let p = PlayerRef {
        uuid: "u".into(),
        name: "P".into(),
    };
    for _ in 0..carve_core::engine::MAX_WAITS_PER_FILE + 10 {
        e.fire(EventKind::Join, EventCtx::player(p.clone()), &mut h);
    }
    assert_eq!(e.pending_waits(), carve_core::engine::MAX_WAITS_PER_FILE);
    assert!(
        h.errors()
            .iter()
            .any(|m| m.starts_with("t.carve:2:") && m.contains("too many runs are waiting"))
    );
    // Resuming frees room.
    let tok = h.waits[0].1;
    e.resume(tok, &mut h);
    assert_eq!(e.pending_waits(), carve_core::engine::MAX_WAITS_PER_FILE - 1);
    e.fire(EventKind::Join, EventCtx::player(p), &mut h);
    assert_eq!(e.pending_waits(), carve_core::engine::MAX_WAITS_PER_FILE);
}

#[test]
fn waits_are_shortened() {
    let (h, _) = run(&cmd("wait 100000000000 seconds\nsend \"never\""));
    assert_eq!(h.waits.len(), 1);
    assert_eq!(h.waits[0].0, carve_core::engine::MAX_WAIT_TICKS);
}

#[test]
fn spawning_is_charged_and_timed() {
    let (h, took) = run(&cmd(
        "loop 100000 times:\n    spawn 100 pigs at location(0, 70, 0)",
    ));
    stopped(&h, "too much work");
    assert!(h.spawned <= 1000, "{}", h.spawned);
    assert!(took < QUICK, "{took:?}");
    // Slow host work stops the run after `max_slice_ms`, whatever it costs in instructions.
    let mut e = engine(&cmd("loop 400 times:\n    spawn a pig at location(0, 70, 0)"));
    e.max_slice_ms = 60;
    let mut h = Mock {
        spawn_cost: Duration::from_millis(10),
        ..Mock::default()
    };
    let t0 = Instant::now();
    e.run_command("x", Recipient::Console, "", &mut h);
    assert!(t0.elapsed() < Duration::from_millis(600), "{:?}", t0.elapsed());
    stopped(&h, "took more than 60 ms");
    assert!(h.spawned < 40, "{}", h.spawned);
}

fn load(src: &str) -> (Engine, Vec<ScriptError>, Duration) {
    let src = src.to_string();
    big(move || {
        let mut e = Engine::default();
        let t0 = Instant::now();
        let (_, errs) = e.load(&[("t.carve".into(), src)]);
        (e, errs, t0.elapsed())
    })
}

#[test]
fn hostile_sources_load_quickly_with_errors() {
    let head = "command /x:\n    trigger:\n";
    let deep_if: String = {
        let mut s = String::from(head);
        for i in 0..400 {
            s.push_str(&format!("{}if true is true:\n", "    ".repeat(i + 2)));
        }
        s.push_str(&format!("{}send \"deep\"\n", "    ".repeat(402)));
        s
    };
    let cases = [
        format!("{head}        send {}1{}\n", "(".repeat(3900), ")".repeat(3900)),
        format!(
            "{head}        send {}1{}\n",
            "(".repeat(100_000),
            ")".repeat(100_000)
        ),
        format!("{head}        send {}1\n", "-".repeat(200_000)),
        format!(
            "{head}        send {}1{}\n",
            "abs(".repeat(50_000),
            ")".repeat(50_000)
        ),
        format!("{head}        send \"{}\"\n", "a".repeat(5_000_000)),
        format!("{head}        set {{_x}} to 1{}\n", "0".repeat(1_000_000)),
        deep_if,
        format!("{head}        send {}\n", "1 + ".repeat(1500) + "1"),
    ];
    for (i, src) in cases.iter().enumerate() {
        let (_, errs, took) = load(src);
        // Debug builds are about 30 times slower than the plugin.
        let limit = if cfg!(debug_assertions) { 15 } else { 2 };
        assert!(took < Duration::from_secs(limit), "case {i}: {took:?}");
        if i < 8 && i != 5 {
            assert!(!errs.is_empty(), "case {i} should be refused");
        }
    }
    // Deep but sane nesting still loads and runs.
    let (mut e, errs, _) = load(&format!(
        "{head}        send {}1{}\n",
        "(".repeat(30),
        ")".repeat(30)
    ));
    assert!(errs.is_empty());
    let mut h = Mock::default();
    e.run_command("x", Recipient::Console, "", &mut h);
    assert_eq!(h.sent, ["1"]);
}

#[test]
fn hostile_numbers_in_scripts() {
    // `1e999` is not a number Carve reads; a long digit string is a finite or infinite number, not a hang.
    let (h, took) = run(&cmd(&format!("set {{_x}} to 1{}\nsend \"ok\"", "0".repeat(400))));
    assert!(took < QUICK);
    assert!(h.sent.len() <= 1);
}

#[test]
fn script_files_calling_each_other_stop() {
    let a = "function fa(n: number) :: number:\n    return fb({_n})\n\ncommand /x:\n    trigger:\n        send \"%fa(1)%\"\n";
    let b = "function fb(n: number) :: number:\n    return fa({_n})\n";
    let h = big(move || {
        let mut e = Engine::default();
        let (_, errs) = e.load(&[("a.carve".into(), a.into()), ("b.carve".into(), b.into())]);
        assert!(errs.is_empty());
        let mut h = Mock::default();
        e.run_command("x", Recipient::Console, "", &mut h);
        h
    });
    assert!(h.errors().iter().any(|m| m.contains("levels deep")));
}

#[test]
fn run_line_is_guarded_too() {
    let mut e = engine(
        "function spin():
    while true is true:
        set {_x} to 1
",
    );
    let mut h = Mock::default();
    let t0 = Instant::now();
    assert!(e.run_line("spin()", Recipient::Console, &mut h).is_ok());
    assert!(t0.elapsed() < QUICK);
    assert!(
        h.errors()
            .iter()
            .any(|m| m.contains("more than 100000 instructions"))
    );
}

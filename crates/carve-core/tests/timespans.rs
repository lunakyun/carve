use carve_core::compile::{EventKind, compile_file};
use carve_core::*;

const NOW: i64 = 1_794_657_005_000; // 2026-11-14 11:10:05 UTC (approx, fixed)

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    waits: Vec<(u64, u64)>,
    logs: Vec<(LogLevel, String)>,
    clock: i64,
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let who = match to {
            Recipient::Player(p) => p.name.clone(),
            Recipient::Console => "console".into(),
        };
        self.calls.push(format!("send {who} {text}"));
    }
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, _: &str) {}
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
    }
    fn find_player(&mut self, _: &str) -> Option<PlayerRef> {
        None
    }
    fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, ticks: u64, token: u64) {
        self.waits.push((ticks, token));
    }
    fn now(&mut self) -> u64 {
        0
    }
    fn unix_millis(&mut self) -> i64 {
        self.clock
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn engine(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("test.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "unexpected errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

/// Run `on join:` with the body lines and return what was sent.
fn run(body: &str) -> Vec<String> {
    let src = format!(
        "on join:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    );
    let mut e = engine(&src);
    let mut h = Mock {
        clock: NOW,
        ..Mock::default()
    };
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let errs: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errs.is_empty(), "runtime errors: {errs:?}");
    h.calls
        .iter()
        .map(|c| c.strip_prefix("send Steve ").unwrap_or(c).to_string())
        .collect()
}

fn out(body: &str) -> String {
    run(body).join("|")
}

fn compile_errors(body: &str) -> usize {
    let src = format!(
        "on join:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    );
    compile_file("bad.carve", &src).err().map_or(0, |e| e.len())
}

#[test]
fn literals_print_as_text() {
    assert_eq!(out("send \"%5 minutes%\""), "5 minutes");
    assert_eq!(out("send \"%1 minute%\""), "1 minute");
    assert_eq!(out("send \"%1 hour and 30 seconds%\""), "1 hour and 30 seconds");
    assert_eq!(
        out("send \"%1 day, 2 hours and 3 minutes%\""),
        "1 day, 2 hours and 3 minutes"
    );
    assert_eq!(out("send \"%90 seconds%\""), "1 minute and 30 seconds");
    assert_eq!(out("send \"%2 ticks%\""), "0.1 seconds");
    assert_eq!(out("send \"%1.5 seconds%\""), "1.5 seconds");
    assert_eq!(out("send \"%2 weeks%\""), "14 days");
    assert_eq!(out("send \"%0 seconds%\""), "0 seconds");
}

#[test]
fn variables_hold_spans() {
    assert_eq!(out("set {t} to 5 minutes\nsend \"%{t}%\""), "5 minutes");
    assert_eq!(
        out("set {t} to 5 minutes + 30 seconds\nsend \"%{t}%\""),
        "5 minutes and 30 seconds"
    );
    assert_eq!(
        out("set {t} to 5 minutes\nset {u} to {t} + 1 minute\nsend \"%{u}%\""),
        "6 minutes"
    );
    assert_eq!(
        out("set {t} to 5 minutes\nset {u} to {t} * 2\nsend \"%{u}%\""),
        "10 minutes"
    );
    assert_eq!(
        out("set {t} to 10 minutes\nset {u} to {t} / 4\nsend \"%{u}%\""),
        "2 minutes and 30 seconds"
    );
    assert_eq!(out("send \"%1 hour / 15 minutes%\""), "4");
    assert_eq!(out("send \"%2 * 30 seconds%\""), "1 minute");
    assert_eq!(out("send \"%1 hour - 20 minutes%\""), "40 minutes");
}

#[test]
fn comparisons() {
    assert_eq!(out("if 90 seconds > 1 minute:\n    send \"yes\""), "yes");
    assert_eq!(out("if 1 minute is 60 seconds:\n    send \"eq\""), "eq");
    assert_eq!(out("if 2 ticks < 1 second:\n    send \"lt\""), "lt");
    assert_eq!(
        out("set {a} to 5 minutes\nset {b} to 2 hours\nif {a} < {b}:\n    send \"var\""),
        "var"
    );
    assert_eq!(out("if 1 minute is not 61 seconds:\n    send \"ne\""), "ne");
    assert_eq!(
        out("if 1 minute >= 2 minutes:\n    send \"bad\"\nsend \"end\""),
        "end"
    );
}

#[test]
fn time_since_and_dates() {
    // `now` is the host clock; ago/later shift it.
    assert_eq!(
        out("set {d} to 10 minutes ago\nsend \"%time since {d}%\""),
        "10 minutes"
    );
    assert_eq!(
        out("set {d} to 2 hours later\nsend \"%time until {d}%\""),
        "2 hours"
    );
    assert_eq!(out("set {d} to now\nsend \"%time since {d}%\""), "0 seconds");
    assert_eq!(out("set {d} to now + 1 day\nsend \"%{d} - now%\""), "1 day");
    assert_eq!(
        out("set {d} to now - 5 minutes\nif time since {d} > 4 minutes:\n    send \"old\""),
        "old"
    );
    assert_eq!(
        out("set {d} to 3 seconds ago\nif time since {d} < 1 minute:\n    send \"fresh\""),
        "fresh"
    );
    assert_eq!(
        out("set {d} to now\nset {e} to {d} + 90 seconds\nsend \"%{e} - {d}%\""),
        "1 minute and 30 seconds"
    );
    assert_eq!(
        out("set {d} to now\nset {e} to {d} + 1 hour\nif {e} > {d}:\n    send \"later\""),
        "later"
    );
}

#[test]
fn date_text_form() {
    // 2026-11-14 11:10:05 UTC
    let s = out("send \"%now%\"");
    assert_eq!(s.len(), 19);
    assert!(s.starts_with("2026-"), "{s}");
    assert_eq!(out("send \"%now - now%\""), "0 seconds");
}

#[test]
fn span_parts() {
    assert_eq!(out("send \"%minutes of 1 hour and 30 minutes%\""), "30");
    assert_eq!(out("send \"%total minutes of 1 hour and 30 minutes%\""), "90");
    assert_eq!(out("send \"%total seconds of 2 minutes%\""), "120");
    assert_eq!(out("send \"%total ticks of 1 second%\""), "20");
    assert_eq!(out("set {t} to 3 hours\nsend \"%hours of {t}%\""), "3");
}

#[test]
fn bad_math_is_a_compile_error() {
    assert_eq!(compile_errors("set {x} to now + now"), 1);
    assert_eq!(compile_errors("set {x} to 5 minutes * 5 minutes"), 1);
    assert_eq!(compile_errors("set {x} to time since 5"), 1);
    assert_eq!(compile_errors("set {x} to 5 minutes * 2"), 0);
}

#[test]
fn plain_numbers_unaffected() {
    assert_eq!(out("send \"%1 + 2%\""), "3");
    assert_eq!(out("set {x} to 5\nadd 2 to {x}\nsend \"%{x}%\""), "7");
}

#[test]
fn wait_with_any_span() {
    let mut e = engine(
        "on join:\n    send \"a\"\n    wait 1 minute and 30 seconds\n    send \"b\"\n    wait 5 ticks\n    send \"c\"\n    wait 2 seconds\n    set {w} to 3 seconds\n    wait {w}\n    send \"d\"\n    wait 1 tick + 1 tick\n    send \"e\"\n",
    );
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.waits[0].0, 1800);
    e.resume(h.waits[0].1, &mut h);
    assert_eq!(h.waits[1].0, 5);
    e.resume(h.waits[1].1, &mut h);
    assert_eq!(h.waits[2].0, 40);
    e.resume(h.waits[2].1, &mut h);
    assert_eq!(h.waits[3].0, 60);
    e.resume(h.waits[3].1, &mut h);
    assert_eq!(h.waits[4].0, 2);
    e.resume(h.waits[4].1, &mut h);
    assert_eq!(h.calls.len(), 5, "{:?}", h.calls);
    assert_eq!(e.pending_waits(), 0);
}

#[test]
fn wait_on_number_variable_is_an_error() {
    let mut e = engine("on join:\n    set {n} to 5\n    wait {n}\n    send \"x\"\n");
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.waits.is_empty());
    assert!(
        h.logs
            .iter()
            .any(|(l, m)| *l == LogLevel::Error && m.contains("can't wait number"))
    );
}

#[test]
fn spans_survive_save_and_load() {
    let mut v = Variables::default();
    v.set_str("t", Value::Timespan { millis: 90_000 });
    v.set_str("d", Value::Date { millis: NOW });
    let json = v.to_json();
    let back = Variables::from_json(&json).unwrap();
    assert_eq!(back.get("t"), Value::Timespan { millis: 90_000 });
    assert_eq!(back.get("d"), Value::Date { millis: NOW });
}

use carve_core::compile::{EventKind, compile_file};
use carve_core::*;

/// 2024-02-29 13:05:09 UTC.
const T: i64 = 1_709_211_909_000;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    wall: i64,
    errors: Vec<String>,
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let who = match to {
            Recipient::Player(p) => p.name.clone(),
            Recipient::Console => "console".into(),
        };
        self.calls.push(format!("send {who} {text}"));
    }
    fn broadcast(&mut self, text: &str) {
        self.calls.push(format!("broadcast {text}"));
    }
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
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn wall_clock_millis(&mut self) -> i64 {
        self.wall
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        if level == LogLevel::Error {
            self.errors.push(msg.to_string());
        }
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

fn join_at(e: &mut Engine, wall: i64) -> Vec<String> {
    let mut h = Mock {
        wall,
        ..Mock::default()
    };
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.errors.is_empty(), "runtime errors: {:?}", h.errors);
    h.calls
}

fn join(src: &str) -> Vec<String> {
    join_at(&mut engine(src), T)
}

fn errors_of(src: &str) -> Vec<String> {
    compile_file("bad.carve", src)
        .err()
        .unwrap_or_default()
        .iter()
        .map(|e| e.to_string())
        .collect()
}

#[test]
fn now_prints_and_formats() {
    let out = join(
        "on join:\n\tbroadcast \"%now%\"\n\tbroadcast \"%now formatted as \"yyyy-MM-dd HH:mm\"%\"\n\tset {_f} to now formatted as \"EEEE d MMMM\"\n\tbroadcast {_f}\n",
    );
    assert_eq!(
        out,
        vec![
            "broadcast 2024-02-29 13:05:09",
            "broadcast 2024-02-29 13:05",
            "broadcast Thursday 29 February",
        ]
    );
}

#[test]
fn formatted_prefix_and_default() {
    let out = join(
        "on join:\n\tbroadcast formatted now as \"dd/MM/yy\"\n\tset {d} to now\n\tbroadcast {d} formatted\n",
    );
    assert_eq!(out, vec!["broadcast 29/02/24", "broadcast 2024-02-29 13:05:09"]);
}

#[test]
fn unix_timestamp_both_ways() {
    let out = join(
        "on join:\n\tbroadcast unix timestamp of now\n\tset {d} to date from unix timestamp 86400\n\tbroadcast {d}\n\tbroadcast unix timestamp of {d}\n\tbroadcast unix date of 0\n",
    );
    assert_eq!(
        out,
        vec![
            "broadcast 1709211909",
            "broadcast 1970-01-02 00:00:00",
            "broadcast 86400",
            "broadcast 1970-01-01 00:00:00",
        ]
    );
}

#[test]
fn date_comparisons() {
    let src = "on join:\n\tset {a} to date from unix timestamp 100\n\tset {b} to date from unix timestamp 200\n\tif {a} < {b}:\n\t\tbroadcast \"lt\"\n\tif {b} > {a}:\n\t\tbroadcast \"gt\"\n\tif {a} = date from unix timestamp 100:\n\t\tbroadcast \"eq\"\n\tif {a} is not {b}:\n\t\tbroadcast \"ne\"\n\tif now > {a}:\n\t\tbroadcast \"after\"\n\tif {a} >= {b}:\n\t\tbroadcast \"bad\"\n";
    assert_eq!(
        join(src),
        vec![
            "broadcast lt",
            "broadcast gt",
            "broadcast eq",
            "broadcast ne",
            "broadcast after"
        ]
    );
}

#[test]
fn now_changes_with_the_clock() {
    let mut e = engine("on join:\n\tbroadcast unix timestamp of now\n");
    assert_eq!(join_at(&mut e, 5_000), vec!["broadcast 5"]);
    assert_eq!(join_at(&mut e, 9_500), vec!["broadcast 9.5"]);
}

#[test]
fn dates_survive_saving() {
    let mut e = engine("on join:\n\tset {seen} to now\n");
    join_at(&mut e, T);
    assert_eq!(e.vars.get("seen"), Value::Date(T));
    let json = e.vars.to_json();
    assert!(json.contains("\"date\""));
    let back = Variables::from_json(&json).unwrap();
    assert_eq!(back.get("seen"), Value::Date(T));

    // A later run can compare against the loaded date.
    let mut e2 = engine("on join:\n\tif now > {seen}:\n\t\tbroadcast \"later\"\n");
    e2.vars = back;
    assert_eq!(join_at(&mut e2, T + 1), vec!["broadcast later"]);
}

#[test]
fn typed_function_parameter() {
    let out = join(
        "function show(d: date):\n\tbroadcast \"%{_d} formatted as \"HH:mm\"%\"\non join:\n\tshow(now)\n",
    );
    assert_eq!(out, vec!["broadcast 13:05"]);
}

#[test]
fn date_errors() {
    assert!(!errors_of("on join:\n\tbroadcast unix timestamp of 5\n").is_empty());
    assert!(!errors_of("on join:\n\tbroadcast date from unix timestamp \"x\" + 1 + player\n").is_empty());
    assert!(!errors_of("on join:\n\tbroadcast formatted 5\n").is_empty());
}

#[test]
fn unset_date_formats_to_none() {
    let out = join("on join:\n\tbroadcast \"%{nothing} formatted as \"yyyy\"%\"\n");
    assert_eq!(out, vec!["broadcast <none>"]);
}

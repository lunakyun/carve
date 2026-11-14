use carve_core::compile::{EventKind, compile_file};
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
    list_name: Option<String>,
    order: Option<i32>,
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
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn player_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((10.5, 64.0, -3.2, "world".into()))
    }
    fn world_time(&mut self, world: &str) -> Option<u64> {
        (world == "world").then_some(24000 * 3 + 5000)
    }
    fn list_name(&mut self, _: &PlayerRef) -> Option<String> {
        self.list_name.clone()
    }
    fn set_list_name(&mut self, _: &PlayerRef, n: Option<&str>) {
        self.list_name = n.map(str::to_string);
    }
    fn set_list_order(&mut self, _: &PlayerRef, o: i32) {
        self.order = Some(o);
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn load(src: &str) -> Result<Engine, String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("test.carve".into(), src.into())]);
    if errs.is_empty() {
        Ok(e)
    } else {
        Err(errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"))
    }
}

fn run_with(e: &mut Engine, h: &mut Mock) -> Vec<String> {
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
    let errs: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errs.is_empty(), "runtime errors: {errs:?}");
    h.calls
        .iter()
        .map(|c| c.strip_prefix("send Steve ").unwrap_or(c).to_string())
        .collect()
}

fn src_of(body: &str) -> String {
    format!(
        "on join:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    )
}

/// Run `on join:` with the body lines and return what was sent.
fn run(body: &str) -> Vec<String> {
    let mut e = load(&src_of(body)).unwrap_or_else(|m| panic!("compile errors: {m}"));
    run_with(&mut e, &mut Mock::default())
}

fn compile_err(body: &str) -> String {
    load(&src_of(body)).err().unwrap_or_default()
}

#[test]
fn registry_ok() {
    carve_core::compile::check_registry().expect("registry");
    let _ = compile_file;
}

#[test]
fn limits_and_constants() {
    let out = run(concat!(
        "send \"%maximum integer value%\"\n",
        "send \"%min integer value%\"\n",
        "send \"%max long value%\"\n",
        "send \"%maximum float value%\"\n",
        "send \"%minimum double value%\"\n",
        "send \"%pi%\"\n",
        "send \"%positive infinity%\"\n",
        "send \"%negative infinity%\"\n",
        "send \"%infinity%\"\n",
        "send \"%nan%\"\n",
    ));
    assert_eq!(out[0], "2147483647");
    assert_eq!(out[1], "-2147483648");
    assert!(out[2].starts_with("9223372036854775"), "{}", out[2]);
    assert!(out[3].starts_with("340282"), "{}", out[3]);
    assert!(out[4].starts_with("-17976931348623"), "{}", out[4]);
    assert!(out[5].starts_with("3.14"), "{}", out[5]);
    assert!(out[6].to_lowercase().contains("inf"), "{}", out[6]);
    assert!(out[7].to_lowercase().contains("inf") && out[7].contains('-'), "{}", out[7]);
    assert!(out[8].to_lowercase().contains("inf"), "{}", out[8]);
    assert!(out[9].to_lowercase().contains("nan"), "{}", out[9]);
}

#[test]
fn constants_in_math() {
    let out = run(
        "set {_a} to 2 * pi\nif {_a} > 6.28:\n    send \"big\"\nif {_a} < max integer value:\n    send \"small\"\nsend \"%max(1, 2)%\"",
    );
    assert_eq!(out, vec!["big", "small", "2"]);
}

#[test]
fn numbers_from_to() {
    let out = run(concat!(
        "set {_l::*} to numbers from 1 to 5\n",
        "send \"%size of {_l::*}%\"\n",
        "set {_s} to 0\n",
        "loop integers from 1 to 4:\n",
        "    add loop-value to {_s}\n",
        "send \"%{_s}%\"\n",
        "set {_d::*} to numbers from 3 to 1\n",
        "send \"%join {_d::*} with \",\"%\"\n",
        "set {_h::*} to numbers from 0.5 to 3\n",
        "send \"%size of {_h::*}%\"\n",
        "set {_i::*} to integers from 0.5 to 3.5\n",
        "send \"%join {_i::*} with \",\"%\"",
    ));
    assert_eq!(out, vec!["5", "10", "3,2,1", "3", "1,2,3"]);
}

#[test]
fn numbers_too_big() {
    let mut e = load(&src_of("set {_l::*} to numbers from 1 to 5000000\nsend \"x\"")).unwrap();
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(
        h.logs.iter().any(|(l, m)| *l == LogLevel::Error && m.contains("million")),
        "{:?}",
        h.logs
    );
}

#[test]
fn percent_of() {
    let out = run(concat!(
        "set {_a} to 10% of 200\n",
        "send \"%{_a}%\"\n",
        "set {_x} to 50\n",
        "set {_b} to {_x}% of 3\n",
        "send \"%{_b}%\"\n",
        "set {_c} to 25 percent of 80 + 1\n",
        "send \"%{_c}%\"\n",
        "set {_d} to (50% of 10) * 2\n",
        "send \"%{_d}%\"",
    ));
    assert_eq!(out, vec!["20", "1.5", "21", "10"]);
}

#[test]
fn number_of_characters() {
    let out = run(concat!(
        "send \"%number of uppercase letters in \"Hello World\"%\"\n",
        "send \"%number of lowercase letters in \"Hello World\"%\"\n",
        "send \"%number of digits in \"a1b22\"%\"\n",
        "send \"%number of spaces in \"a b  c\"%\"",
    ));
    assert_eq!(out, vec!["2", "8", "3", "3"]);
}

#[test]
fn midpoint_and_middle() {
    let out = run(concat!(
        "set {_a} to location(0, 0, 0)\n",
        "set {_b} to location(10, 20, -4)\n",
        "set {_m} to midpoint of {_a} and {_b}\n",
        "send \"%x of {_m}% %y of {_m}% %z of {_m}%\"\n",
        "set {_c} to middle of location(3.9, 64.2, -1.1)\n",
        "send \"%x of {_c}% %y of {_c}% %z of {_c}%\"",
    ));
    assert_eq!(out, vec!["5 10 -2", "3.5 64.5 -1.5"]);
}

#[test]
fn max_stack_size() {
    let out = run(concat!(
        "send \"%max stack size of diamond%\"\n",
        "send \"%max stack size of diamond sword%\"\n",
        "send \"%max stack size of ender pearl%\"\n",
        "send \"%max stack size of oak sign%\"",
    ));
    assert_eq!(out, vec!["64", "1", "16", "16"]);
}

#[test]
fn moon_phase() {
    // Day 3 means the fourth phase: waning crescent.
    let out = run("send moon phase of world \"world\"\nsend moon phase of player");
    assert_eq!(out, vec!["waning crescent", "waning crescent"]);
}

#[test]
fn player_list_name() {
    let mut e = load(&src_of(concat!(
        "send player list name of player\n",
        "set player list name of player to \"&aSteve the Great\"\n",
        "send player's tab list name\n",
        "set player's player list priority to 5\n",
        "reset player list name of player\n",
        "send player's player list name",
    )))
    .unwrap_or_else(|m| panic!("{m}"));
    let mut h = Mock::default();
    let out = run_with(&mut e, &mut h);
    assert_eq!(out, vec!["Steve", "&aSteve the Great", "Steve"]);
    assert_eq!(h.order, Some(5));
}

#[test]
fn metadata_and_persistent_data() {
    let mut e = load(&src_of(concat!(
        "set metadata value \"mood\" of player to \"happy\"\n",
        "send metadata value \"mood\" of player\n",
        "set persistent data value \"Rank\" of player to 7\n",
        "send persistent data value \"rank\" of player\n",
        "delete metadata value \"mood\" of player\n",
        "send \"%metadata value \"mood\" of player%\"\n",
        "set persistent data value \"s\" of world \"world\" to \"w\"\n",
        "send persistent data value \"s\" of world \"world\"",
    )))
    .unwrap_or_else(|m| panic!("{m}"));
    let mut h = Mock::default();
    let out = run_with(&mut e, &mut h);
    assert_eq!(out, vec!["happy", "7", "<none>", "w"]);
    // Persistent data is saved with the variables; metadata is not.
    let json = e.vars.to_json();
    assert!(json.contains("rank"), "{json}");
    assert!(!json.contains("mood"), "{json}");
}

#[test]
fn parse_error() {
    let out = run("set {_n} to \"abc\" parsed as number\nsend \"%parse error%\"");
    assert_eq!(out, vec!["'abc' is not a number"]);
}

#[test]
fn plain_item() {
    let out = run("set {_i} to plain 3 diamonds\nsend \"%{_i}%\"");
    assert_eq!(out.len(), 1);
    assert!(out[0].contains("diamond"), "{out:?}");
}

#[test]
fn errors_are_clear() {
    assert!(compile_err("send \"%numbers from 1%\"").contains("numbers from"));
    assert!(compile_err("send \"%midpoint of location(1,1,1)%\"").contains("midpoint"));
}

#[test]
fn old_syntax_still_works() {
    let out = run(
        "send \"%max(3, 4)%\"\nset {_n} to 5\nsend \"%{_n} + 1%\"\nsend \"%random integer between 1 and 1%\"",
    );
    assert_eq!(out, vec!["4", "6", "1"]);
}

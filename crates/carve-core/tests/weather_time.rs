use carve_core::compile::{check_registry, compile_file};
use carve_core::weather_time::{self, Weather};
use carve_core::*;

/// A world with one clock, one weather, a few rules and a difficulty.
/// With `bare` set, the host answers nothing and uses the trait defaults.
struct Mock {
    time: i64,
    weather: Weather,
    rules: Vec<(String, String)>,
    difficulty: String,
    bare: bool,
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Mock {
    fn new() -> Self {
        Mock {
            time: 1000,
            weather: Weather::Clear,
            rules: vec![("keep_inventory".into(), "false".into()), ("random_tick_speed".into(), "3".into())],
            difficulty: "normal".into(),
            bare: false,
            calls: Vec::new(),
            logs: Vec::new(),
        }
    }
    fn errors(&self) -> Vec<&String> {
        self.logs.iter().filter(|(l, _)| *l == LogLevel::Error).map(|(_, m)| m).collect()
    }
}

fn w(world: Option<&str>) -> &str {
    world.unwrap_or("-")
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        if let Recipient::Player(p) = to {
            self.calls.push(format!("send {} {}", p.name, text));
        }
    }
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.calls.push(format!("cmd {c}"));
    }
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
        Some((0.0, 64.0, 0.0, "nether".into()))
    }

    fn world_time(&mut self, _: Option<&str>) -> Option<i64> {
        (!self.bare).then_some(self.time)
    }
    fn set_world_time(&mut self, world: Option<&str>, ticks: i64) {
        if self.bare {
            self.execute_command(&format!("time set {ticks}"));
        } else {
            self.time = ticks;
            self.calls.push(format!("time {} {ticks}", w(world)));
        }
    }
    fn weather(&mut self, _: Option<&str>) -> Option<Weather> {
        (!self.bare).then_some(self.weather)
    }
    fn set_weather(&mut self, world: Option<&str>, to: Weather, ticks: Option<u32>) {
        if self.bare {
            self.execute_command(&to.command(ticks));
        } else {
            self.weather = to;
            self.calls.push(format!("weather {} {} {:?}", w(world), to.name(), ticks));
        }
    }
    fn game_rule(&mut self, _: Option<&str>, rule: &str) -> Option<String> {
        if self.bare {
            return None;
        }
        self.rules.iter().find(|(r, _)| r == rule).map(|(_, v)| v.clone())
    }
    fn set_game_rule(&mut self, world: Option<&str>, rule: &str, value: &str) {
        if self.bare {
            self.execute_command(&format!("gamerule {rule} {value}"));
        } else {
            self.rules.retain(|(r, _)| r != rule);
            self.rules.push((rule.into(), value.into()));
            self.calls.push(format!("rule {} {rule} {value}", w(world)));
        }
    }
    fn difficulty(&mut self, _: Option<&str>) -> Option<String> {
        (!self.bare).then(|| self.difficulty.clone())
    }
    fn set_difficulty(&mut self, world: Option<&str>, to: &str) {
        if self.bare {
            self.execute_command(&format!("difficulty {to}"));
        } else {
            self.difficulty = to.into();
            self.calls.push(format!("difficulty {} {to}", w(world)));
        }
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run_with(src: &str, mut h: Mock) -> (Vec<String>, Mock) {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty(), "unexpected errors: {errs:?}");
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let calls = std::mem::take(&mut h.calls);
    (calls, h)
}

fn run(src: &str) -> Vec<String> {
    let (calls, h) = run_with(src, Mock::new());
    assert!(h.errors().is_empty(), "runtime errors: {:?}", h.errors());
    calls
}

fn compile_err(src: &str) -> String {
    compile_file("bad.carve", src)
        .err()
        .and_then(|e| e.first().map(ToString::to_string))
        .unwrap_or_default()
}

#[test]
fn registry_still_compiles() {
    check_registry().unwrap();
}

#[test]
fn reads_time_weather_difficulty() {
    let src = r#"on join:
    send "%time% %time in world "w"% %time of day%"
    send "%weather% %weather in world "w"%"
    send "%difficulty% %difficulty of world "w"%"
"#;
    assert_eq!(
        run(src),
        ["send Steve 1000 1000 1000", "send Steve clear clear", "send Steve normal normal"]
    );
}

#[test]
fn time_wraps_to_one_day() {
    let mut h = Mock::new();
    h.time = 24_000 * 5 + 6000;
    let (calls, _) = run_with("on join:\n    send \"%time%\"\n", h);
    assert_eq!(calls, ["send Steve 6000"]);
}

#[test]
fn clock_literals_are_ticks() {
    let src = r#"on join:
    send "%6:00% %12:00% %18:00% %0:00% %6:30%"
    send "%6 am% %6 pm% %12 am% %12:30 pm% %6:00 am%"
"#;
    // 6:00 = 0, noon = 6000, 18:00 = 12000, midnight = 18000, 6:30 = 500.
    assert_eq!(
        run(src),
        ["send Steve 0 6000 12000 18000 500", "send Steve 0 12000 18000 6500 0"]
    );
}

#[test]
fn clock_bad_values_are_errors() {
    assert!(!compile_err("on join:\n    set time to 25:00\n").is_empty());
    assert!(!compile_err("on join:\n    set time to 13 pm\n").is_empty());
}

#[test]
fn sets_time_by_word_number_and_clock() {
    let src = r#"on join:
    set time to day
    set the time to night
    set time to noon
    set time to midnight
    set time to 6:00
    set time to 18:30
    set time to 5 + 1000
    set time in world "arena" to sunrise
"#;
    assert_eq!(
        run(src),
        [
            "time - 1000",
            "time - 13000",
            "time - 6000",
            "time - 18000",
            "time - 0",
            "time - 12500",
            "time - 1005",
            "time arena 23000",
        ]
    );
}

#[test]
fn set_time_keeps_the_day_count() {
    let mut h = Mock::new();
    h.time = 24_000 * 3 + 100;
    let (calls, _) = run_with("on join:\n    set time to noon\n", h);
    assert_eq!(calls, ["time - 78000"]);
}

#[test]
fn big_time_numbers_wrap() {
    assert_eq!(run("on join:\n    set time to 25000\n"), ["time - 1000"]);
    assert_eq!(run("on join:\n    set time to -1\n"), ["time - 23999"]);
}

#[test]
fn adds_and_removes_time() {
    let src = r#"on join:
    add 500 to time
    remove 100 from the time
    subtract 1 second from time
    add 1 minute to time in world "w"
"#;
    assert_eq!(
        run(src),
        ["time - 1500", "time - 1400", "time - 1380", "time w 2580"]
    );
}

#[test]
fn time_add_wraps_below_zero() {
    let mut h = Mock::new();
    h.time = 100;
    let (calls, _) = run_with("on join:\n    remove 300 from time\n", h);
    assert_eq!(calls, ["time - 23800"]);
}

#[test]
fn time_defaults_to_commands() {
    let mut h = Mock::new();
    h.bare = true;
    let src = r#"on join:
    set time to noon
    add 100 to time
"#;
    let (calls, h) = run_with(src, h);
    assert!(h.errors().is_empty());
    assert_eq!(calls, ["cmd time set 6000", "cmd time add 100"]);
}

#[test]
fn sets_weather() {
    let src = r#"on join:
    set weather to rain
    set weather to thunder
    set the weather to clear
    set weather to sunny
    set weather in world "w" to storm
    set weather to rain for 5 minutes
    set weather to clear for 100
"#;
    assert_eq!(
        run(src),
        [
            "weather - rain None",
            "weather - thunder None",
            "weather - clear None",
            "weather - clear None",
            "weather w thunder None",
            "weather - rain Some(6000)",
            "weather - clear Some(100)",
        ]
    );
}

#[test]
fn weather_defaults_to_commands() {
    let mut h = Mock::new();
    h.bare = true;
    let (calls, _) = run_with(
        "on join:\n    set weather to thunder\n    set weather to rain for 10 seconds\n",
        h,
    );
    assert_eq!(calls, ["cmd weather thunder", "cmd weather rain 200"]);
}

#[test]
fn weather_conditions() {
    let src = r#"on join:
    if it is raining:
        send "raining"
    if it is thundering:
        send "thundering"
    if it is sunny:
        send "sunny"
    if it is not raining:
        send "not raining"
    if it isn't raining:
        send "isnt raining"
    if weather is clear:
        send "weather clear"
    if weather is not thunder:
        send "weather not thunder"
"#;
    let mut h = Mock::new();
    h.weather = Weather::Rain;
    let (calls, _) = run_with(src, h);
    assert_eq!(calls, ["send Steve raining", "send Steve weather not thunder"]);
    let mut h = Mock::new();
    h.weather = Weather::Thunder;
    let (calls, _) = run_with(src, h);
    assert_eq!(calls, ["send Steve raining", "send Steve thundering"]);
    let (calls, _) = run_with(src, Mock::new());
    assert_eq!(
        calls,
        [
            "send Steve sunny",
            "send Steve not raining",
            "send Steve isnt raining",
            "send Steve weather clear",
            "send Steve weather not thunder",
        ]
    );
}

#[test]
fn weather_condition_in_world_and_text_compare() {
    let src = r#"on join:
    if it is raining in world "w":
        send "a"
    if weather in world "w" is "clear":
        send "b"
    if weather in world "w" is clear:
        send "c"
"#;
    assert_eq!(run(src), ["send Steve b", "send Steve c"]);
}

#[test]
fn day_and_night_conditions() {
    let src = r#"on join:
    if it is day:
        send "day"
    if it is night:
        send "night"
    if it is not day:
        send "not day"
"#;
    for (t, want) in [
        (0, vec!["send Steve day"]),
        (12_999, vec!["send Steve day"]),
        (13_000, vec!["send Steve night", "send Steve not day"]),
        (22_999, vec!["send Steve night", "send Steve not day"]),
        (23_000, vec!["send Steve day"]),
        (24_000 * 2 + 14_000, vec!["send Steve night", "send Steve not day"]),
    ] {
        let mut h = Mock::new();
        h.time = t;
        let (calls, _) = run_with(src, h);
        assert_eq!(calls, want, "time {t}");
    }
}

#[test]
fn time_compares_as_a_number() {
    let src = r#"on join:
    if time in world "w" is 6:00 am:
        send "six"
    if time > 500:
        send "late"
    if time is between 0 and 2000:
        send "early"
"#;
    let mut h = Mock::new();
    h.time = 0;
    let (calls, _) = run_with(src, h);
    assert_eq!(calls, ["send Steve six", "send Steve early"]);
    assert_eq!(run(src), ["send Steve late", "send Steve early"]);
}

#[test]
fn gamerules() {
    let src = r#"on join:
    send "%gamerule keepInventory% %gamerule randomTickSpeed%"
    set gamerule keepInventory to true
    set gamerule randomTickSpeed to 10
    set gamerule "do-daylight-cycle" to false
    set the gamerule doDaylightCycle in world "w" to {_x}
    set gamerule keep_inventory to "false"
    send "%gamerule keepInventory%"
    if gamerule keepInventory is false:
        send "kept off"
"#;
    // `{_x}` is unset, which is an error; run the rest first.
    let src = src.replace("to {_x}", "to true");
    assert_eq!(
        run(&src),
        [
            "send Steve false 3",
            "rule - keep_inventory true",
            "rule - random_tick_speed 10",
            "rule - advance_time false",
            "rule w advance_time true",
            "rule - keep_inventory false",
            "send Steve false",
            "send Steve kept off",
        ]
    );
}

#[test]
fn gamerule_names() {
    use weather_time::rule_name;
    assert_eq!(rule_name("keepInventory"), Some("keep_inventory"));
    assert_eq!(rule_name("keep-inventory"), Some("keep_inventory"));
    assert_eq!(rule_name("KEEP_INVENTORY"), Some("keep_inventory"));
    assert_eq!(rule_name("minecraft:pvp"), Some("pvp"));
    assert_eq!(rule_name("doDaylightCycle"), Some("advance_time"));
    assert_eq!(rule_name("doMobSpawning"), Some("spawn_mobs"));
    assert_eq!(rule_name("mobGriefing"), Some("mob_griefing"));
    assert_eq!(rule_name("banana"), None);
}

#[test]
fn unknown_gamerule_is_a_compile_error() {
    let e = compile_err("on join:\n    set gamerule banana to true\n");
    assert!(e.contains("unknown game rule"), "{e}");
    let e = compile_err("on join:\n    send \"%gamerule banana%\"\n");
    assert!(e.contains("unknown game rule"), "{e}");
}

#[test]
fn bad_gamerule_value_is_a_runtime_error() {
    let (calls, h) = run_with(
        "on join:\n    set gamerule pvp to \"maybe\"\n    send \"after\"\n",
        Mock::new(),
    );
    assert!(calls.is_empty());
    assert_eq!(h.errors().len(), 1);
    assert!(h.errors()[0].contains("true, false or a whole number"), "{:?}", h.errors());
}

#[test]
fn gamerule_defaults_to_commands() {
    let mut h = Mock::new();
    h.bare = true;
    let (calls, _) = run_with(
        "on join:\n    set gamerule keepInventory to true\n    set gamerule randomTickSpeed to 0\n",
        h,
    );
    assert_eq!(calls, ["cmd gamerule keep_inventory true", "cmd gamerule random_tick_speed 0"]);
}

#[test]
fn difficulty_set_and_check() {
    let src = r#"on join:
    set difficulty to hard
    send "%difficulty%"
    set the difficulty to "peaceful"
    set difficulty in world "w" to 1
    if difficulty is easy:
        send "easy"
    if difficulty is not hard:
        send "not hard"
    if difficulty in world "w" is hard:
        send "no"
"#;
    assert_eq!(
        run(src),
        [
            "difficulty - hard",
            "send Steve hard",
            "difficulty - peaceful",
            "difficulty w easy",
            "send Steve easy",
            "send Steve not hard",
        ]
    );
}

#[test]
fn difficulty_default_and_bad_value() {
    let mut h = Mock::new();
    h.bare = true;
    let (calls, _) = run_with("on join:\n    set difficulty to normal\n", h);
    assert_eq!(calls, ["cmd difficulty normal"]);
    let (calls, h) = run_with("on join:\n    set difficulty to \"banana\"\n", Mock::new());
    assert!(calls.is_empty());
    assert!(h.errors()[0].contains("difficulty must be"), "{:?}", h.errors());
}

#[test]
fn world_can_be_a_player() {
    let src = "on join:\n    set weather of player to rain\n    send \"%time of player%\"\n";
    // `player` resolves to the world the host says they are in.
    assert_eq!(run(src), ["weather nether rain None", "send Steve 1000"]);
}

#[test]
fn unreadable_host_gives_none_and_false() {
    let mut h = Mock::new();
    h.bare = true;
    let src = r#"on join:
    if it is raining:
        send "yes"
    if it is not raining:
        send "no"
    send "%time%|%weather%"
"#;
    let (calls, _) = run_with(src, h);
    assert_eq!(calls, ["send Steve no", "send Steve <none>|<none>"]);
}

#[test]
fn bad_syntax_is_a_clear_error() {
    let e = compile_err("on join:\n    set weather to banana\n");
    assert!(e.contains("weather must be"), "{e}");
    let e = compile_err("on join:\n    set time\n");
    assert!(e.contains("expected 'set time"), "{e}");
    let e = compile_err("on join:\n    set time to\n");
    assert!(e.contains("missing value"), "{e}");
}

#[test]
fn time_since_still_works() {
    let src = "on join:\n    set {d} to now\n    if time since {d} < 5 seconds:\n        send \"recent\"\n";
    assert_eq!(run(src), ["send Steve recent"]);
}

#[test]
fn plain_list_add_is_unaffected() {
    let src = "on join:\n    add 5 to {n}\n    add 2 to {n}\n    remove 1 from {n}\n    send \"%{n}%\"\n";
    assert_eq!(run(src), ["send Steve 6"]);
}

#[test]
fn clock_parts() {
    use weather_time::clock_ticks;
    assert_eq!(clock_ticks(6.0, 0.0, None), Some(0.0));
    assert_eq!(clock_ticks(24.0, 0.0, None), Some(18000.0));
    assert_eq!(clock_ticks(0.0, 0.0, None), Some(18000.0));
    assert_eq!(clock_ticks(12.0, 0.0, Some(false)), Some(18000.0));
    assert_eq!(clock_ticks(12.0, 0.0, Some(true)), Some(6000.0));
    assert_eq!(clock_ticks(6.0, 60.0, None), None);
    assert_eq!(clock_ticks(6.5, 0.0, None), None);
    assert_eq!(clock_ticks(0.0, 0.0, Some(true)), None);
}

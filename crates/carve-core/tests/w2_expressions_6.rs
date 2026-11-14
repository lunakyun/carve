use carve_core::compile::{EventKind, check_registry, compile_file};
use carve_core::*;

/// A host with one player who has air, a respawn point, and a world.
struct Mock {
    sent: Vec<String>,
    errors: Vec<String>,
    air: i64,
    respawn: Option<(f64, f64, f64)>,
}

impl Mock {
    fn new() -> Self {
        Mock {
            sent: Vec::new(),
            errors: Vec::new(),
            air: 300,
            respawn: Some((1.0, 64.0, -3.0)),
        }
    }
}

impl Host for Mock {
    fn send_message(&mut self, _: &Recipient, text: &str) {
        self.sent.push(text.to_string());
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
        match uuid {
            "u1" => Some("Steve".to_string()),
            "u2" => Some("Alex".to_string()),
            _ => None,
        }
    }
    fn find_player(&mut self, _: &str) -> Option<PlayerRef> {
        None
    }
    fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
        true
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        12345
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        if level == LogLevel::Error {
            self.errors.push(msg.to_string());
        }
    }
    fn online_players(&mut self) -> Vec<PlayerRef> {
        vec![steve(), alex()]
    }
    fn player_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((0.0, 70.0, 0.0, "world".into()))
    }
    fn remaining_air(&mut self, _: &PlayerRef) -> Option<i64> {
        Some(self.air)
    }
    fn set_remaining_air(&mut self, _: &PlayerRef, t: i64) {
        self.air = t;
    }
    fn respawn_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64)> {
        self.respawn
    }
    fn set_respawn_location(&mut self, _: &PlayerRef, x: f64, y: f64, z: f64) {
        self.respawn = Some((x, y, z));
    }
    fn sea_level(&mut self, world: &str) -> Option<i64> {
        match world {
            "world" => Some(63),
            "nether" => Some(32),
            _ => None,
        }
    }
    fn simulation_distance(&mut self) -> Option<i64> {
        Some(10)
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn alex() -> PlayerRef {
    PlayerRef {
        uuid: "u2".into(),
        name: "Alex".into(),
    }
}

fn script(trigger: &str, body: &str) -> String {
    format!(
        "{trigger}:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    )
}

fn run_with(trigger: &str, kind: EventKind, ctx: EventCtx, body: &str) -> Mock {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), script(trigger, body))]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    let mut h = Mock::new();
    e.fire(kind, ctx, &mut h);
    assert!(h.errors.is_empty(), "runtime errors: {:?}", h.errors);
    h
}

fn run(body: &str) -> Mock {
    run_with("on join", EventKind::Join, EventCtx::player(steve()), body)
}

fn bad(line: &str) -> bool {
    compile_file("b.carve", &format!("on join:\n    {line}\n")).is_err()
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

// ---- special numbers ----

#[test]
fn pi_and_infinity() {
    let h = run(
        "set {_p} to pi\nsend \"%floor({_p} * 100)%\"\nif infinity > 1000000:\n    send \"inf\"\nif negative infinity < -1000000:\n    send \"neg\"\nset {_n} to NaN\nif {_n} is not {_n}:\n    send \"nan\"",
    );
    assert_eq!(h.sent[0], "314");
    assert!(h.sent.contains(&"inf".to_string()));
    assert!(h.sent.contains(&"neg".to_string()));
}

// ---- random character and uuid ----

#[test]
fn random_character_range() {
    let h = run(
        "loop 50 times:\n    set {_c} to a random character between \"a\" and \"e\"\n    if {_c} is not \"a\", \"b\", \"c\", \"d\" or \"e\":\n        send \"bad %{_c}%\"\nsend \"done\"",
    );
    assert_eq!(h.sent, vec!["done".to_string()]);
}

#[test]
fn random_alphanumeric_character() {
    let h = run(
        "loop 50 times:\n    set {_c} to random alphanumeric character\n    if length of {_c} is not 1:\n        send \"bad\"\nsend \"ok\"",
    );
    assert_eq!(h.sent, vec!["ok".to_string()]);
}

#[test]
fn random_character_plain_is_one_char() {
    let h = run("set {_c} to a random character\nsend \"%length of {_c}%\"");
    assert_eq!(h.sent, vec!["1".to_string()]);
}

#[test]
fn random_uuid_shape() {
    let h = run("set {_a} to random uuid\nset {_b} to a random uuid\nsend {_a}\nsend {_b}");
    for s in &h.sent {
        let p: Vec<&str> = s.split('-').collect();
        assert_eq!(p.iter().map(|x| x.len()).collect::<Vec<_>>(), vec![8, 4, 4, 4, 12], "{s}");
        assert!(p[2].starts_with('4'), "version nibble: {s}");
        assert!("89ab".contains(p[3].chars().next().unwrap()), "variant nibble: {s}");
        assert!(s.chars().all(|c| c == '-' || c.is_ascii_hexdigit()));
    }
    assert_ne!(h.sent[0], h.sent[1]);
}

// ---- raw string and repeat ----

#[test]
fn raw_string_blocks_colors() {
    let h = run("send raw \"&aHello\"\nsend \"&aHello\"");
    // The raw text keeps its `&a`; the plain text is left to the host to color.
    assert!(h.sent[0].starts_with("&\u{200B}a"), "{:?}", h.sent[0]);
    assert!(h.sent[1].contains('&') || h.sent[1].contains('§'));
    let h = run("set {_t} to raw string of \"<red>x\"\nsend {_t}");
    assert_eq!(h.sent[0], "<\u{200B}red>x");
}

#[test]
fn repeat_string() {
    let h = run("send \"ab\" repeated 3 times\nset {_n} to 2\nsend \"-\" repeated {_n} + 1 times\nsend \"x\" repeated 0 times");
    assert_eq!(h.sent, vec!["ababab".to_string(), "---".to_string(), "".to_string()]);
}

#[test]
fn repeat_too_long_is_a_runtime_error() {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[(
        "t.carve".into(),
        script("on join", "send \"abcdefghij\" repeated 1000000 times"),
    )]);
    assert!(errs.is_empty());
    let mut h = Mock::new();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.errors.len(), 1);
    assert!(h.errors[0].contains("longer than"), "{:?}", h.errors);
}

// ---- ternary ----

#[test]
fn ternary_picks_a_side() {
    let h = run(
        "set {_x} to 5\nset {_a} to \"big\" if {_x} > 3 else \"small\"\nset {_b} to \"big\" if {_x} > 9 otherwise \"small\"\nsend {_a}\nsend {_b}",
    );
    assert_eq!(h.sent, vec!["big".to_string(), "small".to_string()]);
}

#[test]
fn ternary_in_text_and_nested() {
    let h = run(
        "set {_x} to 5\nsend \"x is %\"\"even\"\" if {_x} is 4 else \"\"odd\"\"%\"\nset {_y} to 1 if {_x} < 3 else 2 if {_x} < 6 else 3\nsend {_y}",
    );
    assert_eq!(h.sent[1], "2");
}

#[test]
fn ternary_with_and_or_and_math() {
    let h = run("set {_x} to 5\nset {_r} to {_x} * 2 if {_x} > 1 and {_x} < 10 else 0\nsend {_r}");
    assert_eq!(h.sent, vec!["10".to_string()]);
}

#[test]
fn if_without_else_is_not_a_ternary() {
    assert!(bad("set {_a} to 1 if {_x} is 2"));
}

// ---- reduce ----

#[test]
fn reduce_sum_and_product() {
    let h = run(
        "set {_l::*} to 1, 2, 3 and 4\nset {_s} to reduce {_l::*} with [reduced value + input]\nset {_p} to {_l::*} reduced with [reduced value * input]\nsend {_s}\nsend {_p}",
    );
    assert_eq!(h.sent, vec!["10".to_string(), "24".to_string()]);
}

#[test]
fn reduce_text_and_single_and_empty() {
    let h = run(
        "set {_l::*} to \"a\", \"b\" and \"c\"\nset {_s} to reduce {_l::*} with [\"%reduced value%-%input%\"]\nsend {_s}\nset {_one::1} to 7\nsend reduce {_one::*} with [reduced value + input]\nif (reduce {_none::*} with [reduced value + input]) is not set:\n    send \"empty\"",
    );
    assert_eq!(h.sent, vec!["a-b-c".to_string(), "7".to_string(), "empty".to_string()]);
}

#[test]
fn reduce_with_where_filter_inside() {
    let h = run(
        "set {_l::*} to 1, 2, 3, 4 and 5\nsend reduce ({_l::*} where [input > 2]) with [reduced value + input]",
    );
    assert_eq!(h.sent, vec!["12".to_string()]);
}

// ---- recursive ----

#[test]
fn recursive_size_and_loop() {
    let h = run(
        "set {_t::1} to \"a\"\nset {_t::2::x} to \"b\"\nset {_t::2::y::z} to \"c\"\nsend size of {_t::*}\nsend recursive size of {_t::*}\nset {_n} to 0\nloop recursive {_t::*}:\n    add 1 to {_n}\nsend {_n}\nset {_all::*} to recursive {_t::*}\nsend size of {_all::*}",
    );
    assert_eq!(h.sent, vec!["1", "3", "3", "3"]);
}

#[test]
fn recursive_size_of_global_list() {
    let h = run("set {g::a} to 1\nset {g::b::c} to 2\nsend recursive size of {g::*}");
    assert_eq!(h.sent, vec!["2".to_string()]);
}

// ---- host reads ----

#[test]
fn remaining_air_read_and_write() {
    let h = run(
        "send remaining air of player\nset remaining air of player to 5 seconds\nsend player's remaining air\nadd 1 second to remaining air of player\nremove 2 ticks from player's remaining air",
    );
    // 300 ticks = 15 seconds; 100 ticks = 5 seconds; then +20 -2 ticks.
    assert_eq!(h.sent[0], h.sent[0].trim());
    assert!(h.sent[0].contains("15"), "{:?}", h.sent);
    assert!(h.sent[1].contains('5'), "{:?}", h.sent);
    assert_eq!(h.air, 118);
}

#[test]
fn respawn_location_read_and_write() {
    let h = run(
        "set {_l} to respawn location of player\nsend \"%x of {_l}% %y of {_l}% %z of {_l}%\"\nset player's respawn location to location(10, 70, 20)\nset {_m} to player's respawn location\nsend \"%x of {_m}% %z of {_m}%\"",
    );
    assert_eq!(h.sent, vec!["1 64 -3".to_string(), "10 20".to_string()]);
    assert_eq!(h.respawn, Some((10.0, 70.0, 20.0)));
}

#[test]
fn sea_level_of_world() {
    let h = run(
        "send sea level of world \"world\"\nsend sea level of world \"nether\"\nsend sea level of player's world\nsend sea level of world \"nope\"",
    );
    assert_eq!(h.sent[0], "63");
    assert_eq!(h.sent[1], "32");
    assert_eq!(h.sent[2], "63");
    assert_eq!(h.sent[3], "<none>");
}

#[test]
fn simulation_distance_reads() {
    let h = run("send simulation distance\nset {_d} to simulation distance * 2\nsend {_d}");
    assert_eq!(h.sent, vec!["10".to_string(), "20".to_string()]);
}

#[test]
fn tablisted_players_loops() {
    let h = run(
        "loop tablisted players:\n    send name of loop-value\nset {_p::*} to tablisted players\nsend size of {_p::*}",
    );
    assert_eq!(h.sent, vec!["Steve", "Alex", "2"]);
}

// ---- quit message ----

#[test]
fn quit_message_in_quit_event() {
    let mut ctx = EventCtx::player(steve());
    ctx.message = Some("Steve left the game".into());
    let h = run_with("on quit", EventKind::Quit, ctx, "send quit message");
    assert_eq!(h.sent, vec!["Steve left the game".to_string()]);
}

#[test]
fn quit_message_outside_quit_is_an_error() {
    assert!(bad("send quit message"));
}

// ---- nothing else broke ----

#[test]
fn old_syntax_still_works() {
    let h = run(
        "set {_x} to 3\nsend \"v=%{_x} + 1%\"\nset {_y} to {_z} ? 9\nsend {_y}\nsend \"%random integer between 1 and 1%\"",
    );
    assert_eq!(h.sent, vec!["v=4", "9", "1"]);
}

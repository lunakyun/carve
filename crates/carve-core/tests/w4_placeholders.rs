use carve_core::compile::compile_file;
use carve_core::xp_level::XpField;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let who = match to {
            Recipient::Player(p) => p.name.clone(),
            Recipient::Console => "console".into(),
        };
        self.calls.push(format!("{who}: {text}"));
    }
    fn broadcast(&mut self, t: &str) {
        self.calls.push(t.to_string());
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
        true
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn wall_clock_millis(&mut self) -> i64 {
        // 1970-01-02 03:04:05 UTC
        (86_400 + 3 * 3600 + 4 * 60 + 5) * 1000
    }
    fn log(&mut self, _: LogLevel, _: &str) {}
    fn player_count(&mut self) -> f64 {
        3.0
    }
    fn max_players(&mut self) -> Option<f64> {
        Some(20.0)
    }
    fn health(&mut self, _: &PlayerRef) -> Option<f64> {
        Some(17.5)
    }
    fn xp_get(&mut self, _: &PlayerRef, f: XpField) -> Option<f64> {
        (f == XpField::Level).then_some(12.0)
    }
    fn gamemode(&mut self, _: &PlayerRef) -> Option<String> {
        Some("creative".into())
    }
    fn player_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((10.7, 64.0, -3.2, "overworld".into()))
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

/// Run `command /t` as Steve and return what it broadcast.
fn run(e: &mut Engine) -> Vec<String> {
    let mut h = Mock::default();
    e.run_command("t", Recipient::Player(steve()), "", &mut h);
    h.calls
}

fn load_errors(src: &str) -> Vec<String> {
    compile_file("bad.carve", src)
        .err()
        .map(|e| e.iter().map(|x| x.to_string()).collect())
        .unwrap_or_default()
}

#[test]
fn builtin_placeholders() {
    let mut e = engine(
        r#"command /t:
    trigger:
        broadcast placeholder "player_name" for player
        broadcast placeholder "player_uuid" for player
        broadcast placeholder "player_health" for player
        broadcast placeholder "player_level" for player
        broadcast placeholder "player_gamemode" for player
        broadcast placeholder "player_world" for player
        broadcast placeholder "player_x" for player
        broadcast placeholder "player_y" for player
        broadcast placeholder "player_z" for player
        broadcast placeholder "server_online"
        broadcast placeholder "server_max_players"
        broadcast placeholder "server_time"
"#,
    );
    assert_eq!(
        run(&mut e),
        [
            "Steve", "u1", "17.5", "12", "creative", "overworld", "10", "64", "-4", "3", "20", "03:04:05"
        ]
    );
}

#[test]
fn parse_placeholders_in_text() {
    let mut e = engine(
        r#"command /t:
    trigger:
        broadcast parse placeholders in "Hi %%player_name%%, %%server_online%% online, %%nope%% stays" for player
        broadcast parse placeholders in "100%% sure %%player_level%%" for player
"#,
    );
    assert_eq!(
        run(&mut e),
        ["Hi Steve, 3 online, %nope% stays", "100% sure 12"]
    );
}

#[test]
fn player_defaults_to_event_player() {
    let mut e = engine(
        r#"command /t:
    trigger:
        broadcast placeholder "player_name"
        broadcast parse placeholders in "%%player_level%%"
"#,
    );
    assert_eq!(run(&mut e), ["Steve", "12"]);
}

#[test]
fn carve_var_placeholder() {
    let mut e = engine(
        r#"command /t:
    trigger:
        set {coins} to 42
        set {rank::main} to "vip"
        broadcast placeholder "carve_var_coins"
        broadcast parse placeholders in "%%carve_var_rank::main%% %%carve_var_missing%%"
"#,
    );
    assert_eq!(run(&mut e), ["42", "vip %carve_var_missing%"]);
}

#[test]
fn missing_player_leaves_player_placeholders_alone() {
    let mut e = engine(
        r#"every 1 second:
    broadcast parse placeholders in "%%player_name%% %%server_online%%"
"#,
    );
    let mut h = Mock::default();
    e.tick(20, &mut h);
    assert_eq!(h.calls, ["%player_name% 3"]);
}

#[test]
fn script_defined_placeholders() {
    let mut e = engine(
        r#"on placeholder request for prefix "myplugin":
    if the identifier is "greeting":
        set the result to "Hello %player%"

on placeholder request for prefix "other":
    set the result to "other:%the identifier%"

command /t:
    trigger:
        broadcast parse placeholders in "%%myplugin_greeting%%|%%myplugin_unknown%%|%%other_x_y%%|%%myplugin%%"
        broadcast placeholder "myplugin_greeting" for player
"#,
    );
    let out = run(&mut e);
    assert_eq!(out[0], "Hello Steve|%myplugin_unknown%|other:x_y|%myplugin%");
    assert_eq!(out[1], "Hello Steve");
}

#[test]
fn request_without_prefix_sees_every_unknown_name() {
    let mut e = engine(
        r#"on placeholder request:
    set the result to "[%the prefix%/%the identifier%]"

command /t:
    trigger:
        broadcast placeholder "foo_bar_baz"
        broadcast placeholder "player_name"
"#,
    );
    assert_eq!(run(&mut e), ["[foo/bar_baz]", "Steve"]);
}

#[test]
fn script_placeholders_can_nest_but_not_forever() {
    let mut e = engine(
        r#"on placeholder request for prefix "loop":
    set the result to parse placeholders in "%%loop_again%%+"

command /t:
    trigger:
        broadcast placeholder "loop_x"
"#,
    );
    let out = run(&mut e);
    assert_eq!(out.len(), 1);
    assert!(out[0].ends_with('+') || out[0].contains("%loop_again%"), "got {out:?}");
}

#[test]
fn ipc_parse_and_get() {
    let mut e = engine(
        r#"on placeholder request for prefix "ext":
    set the result to "x-%the identifier%"
"#,
    );
    let mut h = Mock::default();
    let r = placeholders::handle_ipc(&mut e, b"carve:parse\nu1\nHi %player_name% %ext_a%", &mut h);
    assert_eq!(r.unwrap(), b"Hi Steve x-a");
    let r = placeholders::handle_ipc(&mut e, b"carve:get\n\n%server_online%", &mut h);
    assert_eq!(r.unwrap(), b"3");
    let r = placeholders::handle_ipc(&mut e, b"carve:get\n\nplayer_name", &mut h);
    assert!(r.is_err(), "no player, no name");
    assert!(placeholders::handle_ipc(&mut e, b"nope\n\nx", &mut h).is_err());
}

#[test]
fn replace_all_edge_cases() {
    let f = |n: &str| (n == "a").then(|| "1".to_string());
    assert_eq!(placeholders::replace_all("%a%%a%", f), "11");
    assert_eq!(placeholders::replace_all("50% off %a%", f), "50% off 1");
    assert_eq!(placeholders::replace_all("%b% %a%", f), "%b% 1");
    assert_eq!(placeholders::replace_all("%%", f), "%%");
    assert_eq!(placeholders::replace_all("trailing %", f), "trailing %");
    assert_eq!(placeholders::replace_all("no percent", f), "no percent");
}

#[test]
fn errors_are_reported_and_the_rest_loads() {
    let errs = load_errors(
        r#"command /t:
    trigger:
        broadcast placeholder "x" for 5
        broadcast "ok"
"#,
    );
    assert_eq!(errs.len(), 1, "{errs:?}");
    assert!(errs[0].starts_with("bad.carve:3:"), "{errs:?}");

    let errs = load_errors(
        r#"on placeholder request for prefix "a":
    set the identifier to "x"
"#,
    );
    assert!(errs.iter().any(|e| e.contains("can't be changed")), "{errs:?}");
}

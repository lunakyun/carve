use std::collections::HashMap;

use carve_core::compile::{EventKind, check_registry, compile_file};
use carve_core::*;

/// Records server calls; the whitelist is kept so conditions can read it.
#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    online: HashMap<String, String>,
    whitelist: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Mock {
    fn with_steve() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m.online.insert("u2".into(), "Alex".into());
        m
    }
    fn take(&mut self) -> Vec<String> {
        std::mem::take(&mut self.calls)
    }
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
    fn execute_command(&mut self, c: &str) {
        self.calls.push(format!("cmd {c}"));
    }
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        self.online.get(uuid).cloned()
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        self.online
            .iter()
            .find(|(_, n)| n.eq_ignore_ascii_case(name))
            .map(|(u, n)| PlayerRef {
                uuid: u.clone(),
                name: n.clone(),
            })
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

    fn online_players(&mut self) -> Vec<PlayerRef> {
        let mut v: Vec<_> = self
            .online
            .iter()
            .map(|(u, n)| PlayerRef {
                uuid: u.clone(),
                name: n.clone(),
            })
            .collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }
    fn player_location(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        // Steve is in the overworld, Alex in the nether.
        let w = if p.name == "Alex" { "nether" } else { "world" };
        Some((0.0, 64.0, 0.0, w.into()))
    }
    fn teleport_to(&mut self, p: &PlayerRef, x: f64, y: f64, z: f64, world: Option<&str>) {
        self.calls
            .push(format!("tp {} {x} {y} {z} {}", p.name, world.unwrap_or("-")));
    }

    fn max_players(&mut self) -> Option<f64> {
        Some(20.0)
    }
    fn motd(&mut self) -> Option<String> {
        Some("Hello MOTD".into())
    }
    fn server_version(&mut self) -> Option<String> {
        Some("0.1.2".into())
    }
    fn is_whitelisted(&mut self, name: &str) -> bool {
        self.whitelist.iter().any(|n| n.eq_ignore_ascii_case(name))
    }
    fn set_whitelisted(&mut self, name: &str, uuid: Option<&str>, on: bool) {
        self.calls
            .push(format!("whitelist {name} {} {on}", uuid.unwrap_or("-")));
        self.whitelist.retain(|n| n != name);
        if on {
            self.whitelist.push(name.to_string());
        }
    }
    fn world_names(&mut self) -> Vec<String> {
        vec!["world".into(), "nether".into()]
    }
    fn world_spawn(&mut self, world: &str) -> Option<(f64, f64, f64)> {
        match world {
            "world" => Some((0.5, 70.0, 0.5)),
            "nether" => Some((8.5, 40.0, 8.5)),
            _ => None,
        }
    }
    fn border_size(&mut self, world: &str) -> Option<f64> {
        (world == "world").then_some(1000.0)
    }
    fn set_border_size(&mut self, world: &str, size: f64, seconds: f64) {
        self.calls.push(format!("border {world} {size} {seconds}"));
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

fn run_with(src: &str, h: &mut Mock) -> Vec<String> {
    let mut e = engine(src);
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
    let errs: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errs.is_empty(), "runtime errors: {errs:?}");
    h.take()
}

fn join(src: &str) -> Vec<String> {
    run_with(src, &mut Mock::with_steve())
}

fn compiles(line: &str) -> bool {
    compile_file("t.carve", &format!("on join:\n    {line}\n")).is_ok()
}

#[test]
fn registry_still_valid() {
    check_registry().unwrap();
}

#[test]
fn player_count_forms() {
    let out = join(
        "on join:\n    send \"%online players count%|%amount of players%|%number of online players%|%player count%\"\n",
    );
    assert_eq!(out, ["send Steve 2|2|2|2"]);
}

#[test]
fn count_in_math_and_conditions() {
    let src = "on join:\n    set {n} to amount of players + 1\n    send \"%{n}%\"\n    if online players count is 2:\n        send \"two\"\n    if amount of players > 5:\n        send \"many\"\n";
    assert_eq!(join(src), ["send Steve 3", "send Steve two"]);
}

#[test]
fn amount_of_characters_still_works() {
    let out = join("on join:\n    send \"%length of \"abc\"%\"\n");
    assert_eq!(out, ["send Steve 3"]);
}

#[test]
fn max_players_motd_version() {
    let out = join(
        "on join:\n    send \"%max players%|%maximum players%|%max player count%\"\n    send motd\n    send server version\n",
    );
    assert_eq!(
        out,
        ["send Steve 20|20|20", "send Steve Hello MOTD", "send Steve 0.1.2"]
    );
}

#[test]
fn max_function_unharmed() {
    let out = join("on join:\n    send \"%max(3, 9)%\"\n");
    assert_eq!(out, ["send Steve 9"]);
}

#[test]
fn whitelist_effects_and_condition() {
    let src = "on join:\n    if player is not whitelisted:\n        send \"no\"\n    add player to the whitelist\n    if player is whitelisted:\n        send \"yes\"\n    remove player from whitelist\n    if player isn't whitelisted:\n        send \"gone\"\n    whitelist \"Ghost\"\n    if \"ghost\" is on the whitelist:\n        send \"ghost in\"\n    unwhitelist \"Ghost\"\n";
    assert_eq!(
        join(src),
        [
            "send Steve no",
            "whitelist Steve u1 true",
            "send Steve yes",
            "whitelist Steve u1 false",
            "send Steve gone",
            "whitelist Ghost - true",
            "send Steve ghost in",
            "whitelist Ghost - false",
        ]
    );
}

#[test]
fn add_to_list_still_works() {
    let out = join("on join:\n    add 5 to {x}\n    add 2 to {x}\n    send \"%{x}%\"\n");
    assert_eq!(out, ["send Steve 7"]);
}

#[test]
fn remove_from_list_still_works() {
    let out =
        join("on join:\n    add 5 to {x::*}\n    remove 5 from {x::*}\n    send \"%size of {x::*}%\"\n");
    assert_eq!(out, ["send Steve 0"]);
}

#[test]
fn broadcast_to_world() {
    let out = join(
        "on join:\n    broadcast \"hi\" to world \"nether\"\n    broadcast \"all\"\n    broadcast \"here\" in player's world\n",
    );
    assert_eq!(out, ["send Alex hi", "broadcast all", "send Steve here"]);
}

#[test]
fn broadcast_with_to_inside_text_is_plain() {
    let out = join("on join:\n    broadcast \"going to town\"\n");
    assert_eq!(out, ["broadcast going to town"]);
}

#[test]
fn all_worlds_list() {
    let src = "on join:\n    loop all worlds:\n        send \"w %loop-value%\"\n    set {w::*} to all worlds\n    send \"%size of {w::*}%\"\n";
    assert_eq!(
        join(src),
        ["send Steve w world", "send Steve w nether", "send Steve 2"]
    );
}

#[test]
fn spawn_of_world() {
    let out = join(
        "on join:\n    set {s} to spawn of world \"nether\"\n    send \"%x of {s}% %y of {s}%\"\n    teleport player to spawn of world \"world\"\n",
    );
    assert_eq!(out, ["send Steve 8.5 40", "tp Steve 0.5 70 0.5 world"]);
}

#[test]
fn teleport_to_spawn_words() {
    let out = join(
        "on join:\n    teleport player to spawn\n    teleport player to the world spawn of world \"nether\"\n",
    );
    assert_eq!(out, ["tp Steve 0.5 70 0.5 world", "tp Steve 8.5 40 8.5 nether"]);
}

#[test]
fn teleport_to_location_unharmed() {
    assert!(compiles("teleport player to location(1, 2, 3)"));
}

#[test]
fn world_border_read_and_set() {
    let out = join(
        "on join:\n    send \"%world border size%\"\n    send \"%world border size of world \"nether\"%\"\n    set world border size to 500\n    set world border size of world \"nether\" to 200 over 30 seconds\n    set {n} to 700
    set the world border diameter to {n}\n",
    );
    assert_eq!(
        out,
        [
            "send Steve 1000",
            "send Steve <none>",
            "border world 500 0",
            "border nether 200 30",
            "border world 700 0",
        ]
    );
}

#[test]
fn world_border_size_is_clamped() {
    let out = join("on join:\n    set world border size to 99999999999\n    set world border size to -5\n");
    assert_eq!(out, ["border world 59999968 0", "border world 1 0"]);
}

#[test]
fn server_list_ping_event_compiles_and_fires() {
    let src = "on server list ping:\n    set {x} to online players count\n";
    let mut e = engine(src);
    let k = EventKind::Ext4(carve_core::events_world_vehicles::ExtEvent::ServerListPing);
    assert!(e.table().has_event(k));
    let mut h = Mock::with_steve();
    e.fire(k, EventCtx::console(), &mut h);
    assert!(h.logs.is_empty(), "{:?}", h.logs);
}

#[test]
fn bad_lines_are_errors() {
    assert!(!compiles("set world border size to"));
    assert!(!compiles("add to the whitelist"));
    assert!(!compiles("send spawn"));
    assert!(!compiles("send spawn of 5"));
}

#[test]
fn default_host_uses_commands() {
    // A host that only implements the required methods gets console commands.
    struct Bare(Vec<String>);
    impl Host for Bare {
        fn send_message(&mut self, _: &Recipient, _: &str) {}
        fn broadcast(&mut self, _: &str) {}
        fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
        fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
        fn heal(&mut self, _: &PlayerRef) {}
        fn set_health(&mut self, _: &PlayerRef, _: f64) {}
        fn kill(&mut self, _: &PlayerRef) {}
        fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
        fn execute_command(&mut self, c: &str) {
            self.0.push(c.to_string());
        }
        fn resolve_player(&mut self, _: &str) -> Option<String> {
            None
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
        fn log(&mut self, _: LogLevel, _: &str) {}
        fn default_world(&mut self) -> Option<String> {
            Some("world".into())
        }
    }
    let mut e = engine(
        "on join:\n    whitelist \"Bob\"\n    remove \"Bob\" from the whitelist\n    set world border size to 300 over 10 seconds\n",
    );
    let mut h = Bare(Vec::new());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(
        h.0,
        [
            "whitelist add Bob",
            "whitelist remove Bob",
            "worldborder set 300 10s"
        ]
    );
}

#[test]
fn doc_lines_compile() {
    for line in [
        "send \"%online player count% %number of players% %amount of online players%\"",
        "send \"%maximum players% %player limit% %server motd% %message of the day%\"",
        "send \"%pumpkin version% %version of the server%\"",
        "send \"%spawn point of world \"nether\"% %world spawn of player's world%\"",
        "send \"%border size% %world border diameter%\"",
        "teleport player to the spawn of world \"nether\"",
        "teleport player to spawn of world \"nether\"",
        "broadcast \"x\" in player's world",
        "set world border size of world \"nether\" to 200 in 30 seconds",
        "if player is on the whitelist:\n        send \"listed\"",
        "if \"Alex\" is not whitelisted:\n        send \"no\"",
    ] {
        assert!(compiles(line), "does not compile: {line}");
    }
}

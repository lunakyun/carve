use carve_core::compile::check_registry;
use carve_core::*;

/// Logs host calls. `Alex` is the second online player.
#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
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
        match uuid {
            "u1" => Some("Steve".into()),
            "u2" => Some("Alex".into()),
            _ => None,
        }
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        (name == "Alex").then(|| PlayerRef {
            uuid: "u2".into(),
            name: "Alex".into(),
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
    fn transfer(&mut self, p: &PlayerRef, host: &str, port: u16) {
        self.calls.push(format!("transfer {} {host} {port}", p.name));
    }
    fn respawn(&mut self, p: &PlayerRef) {
        self.calls.push(format!("respawn {}", p.name));
    }
    fn hide_player(&mut self, v: &PlayerRef, t: &PlayerRef) {
        self.calls.push(format!("hide {} from {}", t.name, v.name));
    }
    fn show_player(&mut self, v: &PlayerRef, t: &PlayerRef) {
        self.calls.push(format!("show {} to {}", t.name, v.name));
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run_h(src: &str) -> (Vec<String>, Vec<String>) {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty(), "unexpected errors: {errs:?}");
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let errors = h
        .logs
        .iter()
        .filter(|(l, _)| *l == LogLevel::Error)
        .map(|(_, m)| m.clone())
        .collect();
    (h.calls, errors)
}

fn run(src: &str) -> Vec<String> {
    let (calls, errs) = run_h(src);
    assert!(errs.is_empty(), "runtime errors: {errs:?}");
    calls
}

fn compile_errors(src: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    errs.iter().map(|x| x.msg.clone()).collect()
}

#[test]
fn registry_still_compiles() {
    check_registry().unwrap();
}

#[test]
fn connect_default_port() {
    let out = run("on join:\n    connect player to server \"play.example.net\"\n");
    assert_eq!(out, ["transfer Steve play.example.net 25565"]);
}

#[test]
fn connect_with_port_forms() {
    let out = run(
        "on join:\n    connect player to \"a.net:25570\"\n    connect player to the server \"b.net\" on port 1234\n",
    );
    assert_eq!(out, ["transfer Steve a.net 25570", "transfer Steve b.net 1234"]);
}

#[test]
fn connect_bad_address_is_a_runtime_error() {
    let (calls, errs) = run_h("on join:\n    connect player to \"\"\n");
    assert!(calls.is_empty());
    assert_eq!(errs.len(), 1);
    assert!(errs[0].contains("not a valid server address"), "{errs:?}");
}

#[test]
fn connect_default_uses_transfer_command() {
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
            Some("Steve".into())
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
    }
    let src = "on join:\n    connect player to \"x.net\"\n    equip player with diamond helmet\n    equip player with shield\n    equip player with stick\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty(), "{errs:?}");
    let mut h = Bare(Vec::new());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(
        h.0,
        [
            "transfer x.net 25565 Steve",
            "item replace entity Steve armor.head with minecraft:diamond_helmet",
            "item replace entity Steve weapon.offhand with minecraft:shield",
            "item replace entity Steve weapon.mainhand with minecraft:stick",
        ]
    );
}

#[test]
fn force_respawn_both_forms() {
    let out = run("on join:\n    force player to respawn\n    force respawn of player\n    force the respawn of player\n");
    assert_eq!(out, ["respawn Steve", "respawn Steve", "respawn Steve"]);
}

#[test]
fn hide_and_reveal() {
    let src = "on join:\n    set {_a} to \"Alex\"\n    hide player from {_a}\n    reveal player to {_a}\n";
    let out = run(src);
    assert_eq!(out, ["hide Steve from Alex", "show Steve to Alex"]);
}

#[test]
fn hide_offline_viewer_does_nothing() {
    let out = run("on join:\n    set {_n} to \"Nobody\"
    hide player from {_n}\n");
    assert!(out.is_empty());
}

#[test]
fn copy_single_value() {
    let out = run("on join:\n    set {a} to 5\n    copy {a} into {b}\n    set {a} to 6\n    send \"%{a}% %{b}%\"\n");
    assert_eq!(out, ["send Steve 6 5"]);
}

#[test]
fn copy_list_keeps_keys_and_is_independent() {
    let src = r#"on join:
    set {_x::first} to "a"
    set {_x::second} to "b"
    copy {_x::*} into {_y::*}
    add "c" to {_x::*}
    send "%{_y::first}% %{_y::second}% %size of {_y::*}% %size of {_x::*}%"
"#;
    assert_eq!(run(src), ["send Steve a b 2 3"]);
}

#[test]
fn copy_list_replaces_old_target_and_works_for_globals() {
    let src = r#"on join:
    set {g::*} to 1, 2 and 3
    set {h::old} to "gone"
    copy {g::*} to {h::*}
    send "%size of {h::*}% %{h::old}%"
    copy {g::*} into {g::*}
    send "%size of {g::*}%"
"#;
    assert_eq!(run(src), ["send Steve 3 <none>", "send Steve 3"]);
}

#[test]
fn copy_unset_clears_target() {
    let src = "on join:\n    set {b} to 1\n    copy {nothing} into {b}\n    send \"%{b}%\"\n";
    assert_eq!(run(src), ["send Steve <none>"]);
}

#[test]
fn copy_list_to_single_is_an_error() {
    let (_, errs) = run_h("on join:\n    copy {x::*} into {y}\n");
    assert_eq!(errs.len(), 1);
}

#[test]
fn bad_forms_do_not_compile() {
    assert!(!compile_errors("on join:\n    connect player\n").is_empty());
    assert!(!compile_errors("on join:\n    copy 5 into {x}\n").is_empty());
}

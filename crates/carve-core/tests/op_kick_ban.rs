use std::collections::HashMap;

use carve_core::compile::{EventKind, check_registry, compile_file};
use carve_core::*;

/// Records admin calls; ban list and ops are kept so conditions can read them.
#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    online: HashMap<String, String>,
    ops: Vec<String>,
    banned: Vec<String>,
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

    fn is_op(&mut self, p: &PlayerRef) -> bool {
        self.ops.contains(&p.name)
    }
    fn set_op(&mut self, name: &str, op: bool) {
        self.calls.push(format!("op {name} {op}"));
        if op {
            self.ops.push(name.to_string());
        } else {
            self.ops.retain(|n| n != name);
        }
    }
    fn kick(&mut self, name: &str, reason: Option<&str>) {
        self.calls.push(format!("kick {name} {}", reason.unwrap_or("-")));
    }
    fn ban(&mut self, name: &str, reason: Option<&str>, ip: bool) {
        self.calls.push(format!(
            "ban{} {name} {}",
            if ip { "-ip" } else { "" },
            reason.unwrap_or("-")
        ));
        self.banned.push(name.to_string());
    }
    fn unban(&mut self, name: &str, ip: bool) {
        self.calls
            .push(format!("unban{} {name}", if ip { "-ip" } else { "" }));
        self.banned.retain(|n| n != name);
    }
    fn is_banned(&mut self, name: &str) -> bool {
        self.banned.iter().any(|n| n.eq_ignore_ascii_case(name))
    }
    fn display_name(&mut self, p: &PlayerRef) -> Option<String> {
        self.calls.push(format!("get display {}", p.name));
        Some("Stevie".into())
    }
    fn set_display_name(&mut self, p: &PlayerRef, n: Option<&str>) {
        self.calls
            .push(format!("display {} {}", p.name, n.unwrap_or("<reset>")));
    }
    fn set_tab_list_name(&mut self, p: &PlayerRef, n: Option<&str>) {
        self.calls
            .push(format!("tab {} {}", p.name, n.unwrap_or("<reset>")));
    }
    fn ping(&mut self, _: &PlayerRef) -> Option<f64> {
        Some(42.0)
    }
    fn ip(&mut self, _: &PlayerRef) -> Option<String> {
        Some("10.0.0.7".into())
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
fn op_and_deop() {
    let out = join("on join:\n    op player\n    deop \"Alex\"\n");
    assert_eq!(out, ["op Steve true", "op Alex false"]);
}

#[test]
fn op_works_on_offline_names() {
    let out = join("on join:\n    op \"Ghost\"\n");
    assert_eq!(out, ["op Ghost true"]);
}

#[test]
fn is_op_condition() {
    let src = "on join:\n    if player is op:\n        send \"op\"\n    else:\n        send \"plain\"\n    if player is not op:\n        send \"not\"\n    if player isn't an operator:\n        send \"isnt\"\n";
    let out = join(src);
    assert_eq!(out, ["send Steve plain", "send Steve not", "send Steve isnt"]);
    let mut h = Mock::with_steve();
    h.ops.push("Steve".into());
    assert_eq!(run_with(src, &mut h), ["send Steve op"]);
}

#[test]
fn kick_with_and_without_reason() {
    let out = join(
        "on join:\n    kick player\n    kick player due to \"bad vibes\"\n    kick \"Alex\" because of \"afk\"\n    kick player for \"x\"\n",
    );
    assert_eq!(
        out,
        [
            "kick Steve -",
            "kick Steve bad vibes",
            "kick Alex afk",
            "kick Steve x"
        ]
    );
}

#[test]
fn kick_reason_can_use_expressions() {
    let out = join("on join:\n    set {r} to \"spam\"\n    kick player due to \"reason: %{r}%\"\n");
    assert_eq!(out, ["kick Steve reason: spam"]);
}

#[test]
fn kick_offline_is_a_no_op() {
    assert!(join("on join:\n    kick \"Ghost\"\n").is_empty());
}

#[test]
fn ban_and_unban() {
    let out = join(
        "on join:\n    ban player\n    ban \"Alex\" due to \"cheating\"\n    unban \"Alex\"\n    pardon player\n",
    );
    assert_eq!(
        out,
        ["ban Steve -", "ban Alex cheating", "unban Alex", "unban Steve"]
    );
}

#[test]
fn ban_by_ip() {
    let out =
        join("on join:\n    ban player by ip\n    ban ip \"Alex\" because \"bots\"\n    unban ip \"Alex\"\n");
    assert_eq!(out, ["ban-ip Steve -", "ban-ip Alex bots", "unban-ip Alex"]);
}

#[test]
fn is_banned_condition() {
    let src = "on join:\n    if \"Alex\" is banned:\n        send \"yes\"\n    else:\n        send \"no\"\n    ban \"Alex\"\n    if \"alex\" is banned:\n        send \"yes\"\n    if \"Alex\" is not banned:\n        send \"bad\"\n";
    let out = join(src);
    assert_eq!(out, ["send Steve no", "ban Alex -", "send Steve yes"]);
}

#[test]
fn display_name_get_and_set() {
    let out = join(
        "on join:\n    set display name of player to \"&6Boss\"\n    set player's nickname to \"Bo\"\n    set {n} to display name of player\n    send {n}\n    send nickname of player\n    send player's display name\n",
    );
    assert_eq!(
        out,
        [
            "display Steve &6Boss",
            "display Steve Bo",
            "get display Steve",
            "send Steve Stevie",
            "get display Steve",
            "send Steve Stevie",
            "get display Steve",
            "send Steve Stevie"
        ]
    );
}

#[test]
fn reset_names() {
    let out = join(
        "on join:\n    reset display name of player\n    reset player's tab list name\n    reset nickname of player\n",
    );
    assert_eq!(
        out,
        [
            "display Steve <reset>",
            "tab Steve <reset>",
            "display Steve <reset>"
        ]
    );
}

#[test]
fn tab_list_name_set_and_get_falls_back_to_name() {
    let out = join(
        "on join:\n    set tab list name of player to \"&a[VIP] Steve\"\n    set player's tab list name to \"x\"\n    send tab list name of player\n",
    );
    assert_eq!(
        out,
        ["tab Steve &a[VIP] Steve", "tab Steve x", "send Steve Steve"]
    );
}

#[test]
fn ping_and_ip() {
    let out = join(
        "on join:\n    send \"ping %ping of player%\"\n    send \"ip %ip of player%\"\n    send \"ip %player's ip address%\"\n    if ping of player < 100:\n        send \"fast\"\n",
    );
    assert_eq!(
        out,
        [
            "send Steve ping 42",
            "send Steve ip 10.0.0.7",
            "send Steve ip 10.0.0.7",
            "send Steve fast"
        ]
    );
}

#[test]
fn uuid_of_player_still_works() {
    assert_eq!(join("on join:\n    send uuid of player\n"), ["send Steve u1"]);
}

#[test]
fn default_host_uses_console_commands() {
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
            Some(steve())
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
    let mut e = engine(
        "on join:\n    op player\n    deop player\n    kick player due to \"bye\"\n    ban player due to \"no\"\n    unban player\n    ban player by ip\n    unban ip player\n    if player is op:\n        send \"never\"\n",
    );
    let mut h = Bare(Vec::new());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(
        h.0,
        [
            "op Steve",
            "deop Steve",
            "kick Steve bye",
            "ban Steve no",
            "pardon Steve",
            "ban-ip Steve",
            "pardon-ip Steve"
        ]
    );
}

#[test]
fn existing_syntax_is_unharmed() {
    assert!(compiles("set {x} to 5"));
    assert!(compiles("set {_a} to \"hi\""));
    assert!(compiles("set {b} to {a} + 2"));
    assert!(!compiles("kick"));
    assert!(!compiles("op"));
}

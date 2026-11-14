use std::collections::HashMap;

use carve_core::compile::{EventKind, check_registry, compile_file};
use carve_core::w2_expressions_3::PStat;
use carve_core::*;

struct Mock {
    stats: HashMap<PStat, f64>,
    sent: Vec<String>,
    errors: Vec<String>,
    flags: Vec<PlayerFlag>,
}

impl Mock {
    fn new() -> Self {
        let mut stats = HashMap::new();
        stats.insert(PStat::Exhaustion, 1.5);
        stats.insert(PStat::FallDistance, 3.0);
        stats.insert(PStat::FireTicks, 100.0);
        stats.insert(PStat::FreezeTicks, 40.0);
        stats.insert(PStat::Gliding, 0.0);
        stats.insert(PStat::Glowing, 1.0);
        stats.insert(PStat::Gravity, 1.0);
        stats.insert(PStat::HotbarSlot, 4.0);
        stats.insert(PStat::Yaw, 180.0);
        stats.insert(PStat::Pitch, 0.0);
        Mock {
            stats,
            sent: Vec::new(),
            errors: Vec::new(),
            flags: vec![PlayerFlag::Glowing],
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
            "u1" => Some("Steve".into()),
            "u2" => Some("Alex".into()),
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
        0
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
        Some((10.5, 64.0, -3.2, "world".into()))
    }
    fn pstat(&mut self, _: &PlayerRef, s: PStat) -> Option<f64> {
        self.stats.get(&s).copied()
    }
    fn set_pstat(&mut self, _: &PlayerRef, s: PStat, v: f64) {
        self.stats.insert(s, v);
    }
    fn player_flag(&mut self, _: &PlayerRef, f: PlayerFlag) -> bool {
        self.flags.contains(&f)
    }
    fn first_empty_slot(&mut self, _: &PlayerRef) -> Option<u32> {
        Some(2)
    }
    fn can_see(&mut self, _: &PlayerRef, other: &PlayerRef) -> bool {
        other.uuid != "u2"
    }
    fn system_memory(&mut self) -> Option<(u64, u64)> {
        Some((16 * 1_048_576, 6 * 1_048_576))
    }
    fn top_block_y(&mut self, _: Option<&str>, x: i32, z: i32) -> Option<i32> {
        Some(60 + x + z)
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

fn run(body: &str) -> Mock {
    let src = format!(
        "on join:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    );
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    let mut h = Mock::new();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.errors.is_empty(), "runtime errors: {:?}", h.errors);
    h
}

fn bad(line: &str) -> bool {
    compile_file("b.carve", &format!("on join:\n    {line}\n")).is_err()
}

/// Run one line that sends a value and return the text sent.
fn eval(expr: &str) -> String {
    let h = run(&format!("set {{_v}} to {expr}
send \"%{{_v}}%\" to player"));
    h.sent.join("|")
}

#[test]
fn registry_is_valid() {
    check_registry().unwrap();
}

#[test]
fn hashes() {
    assert_eq!(eval("\"abc\" hashed with MD5"), "900150983cd24fb0d6963f7d28e17f72");
    assert_eq!(
        eval("\"abc\" hashed with SHA-256"),
        "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"
    );
    assert_eq!(
        eval("\"abc\" hashed with sha-512").len(),
        128,
        "sha-512 is 64 bytes of hex"
    );
    assert_eq!(eval("\"abc\" hashed with SHA-1"), "a9993e364706816aba3e25717850c26c9cd0d89d");
    assert!(bad("send \"a\" hashed with CRC32 to player"));
}

#[test]
fn infinity_and_memory() {
    let h = run("if 5 < infinity:\n    send \"ok\" to player");
    assert_eq!(h.sent, vec!["ok"]);
    let h = run("if -5 > negative infinity:\n    send \"ok\" to player");
    assert_eq!(h.sent, vec!["ok"]);
    assert_eq!(eval("free memory"), "10");
    assert_eq!(eval("total memory"), "16");
    assert_eq!(eval("max memory"), "16");
}

#[test]
fn player_stats() {
    assert_eq!(eval("exhaustion of player"), "1.5");
    assert_eq!(eval("player's fall distance"), "3");
    assert_eq!(eval("hotbar slot of player"), "4");
    assert_eq!(eval("burn duration of player"), "5 seconds");
    assert_eq!(eval("freeze time of player"), "2 seconds");
    assert_eq!(eval("glowing state of player"), "true");
    assert_eq!(eval("gliding state of player"), "false");
    assert_eq!(eval("gravity of player"), "true");
    assert_eq!(eval("facing of player"), "north");
}

#[test]
fn set_add_remove() {
    let h = run("set exhaustion of player to 4\nadd 1 to exhaustion of player\nremove 0.5 from player's exhaustion");
    assert_eq!(h.stats[&PStat::Exhaustion], 4.5);
    let h = run("set exhaustion of player to 99");
    assert_eq!(h.stats[&PStat::Exhaustion], 40.0);
    let h = run("set fall distance of player to 0");
    assert_eq!(h.stats[&PStat::FallDistance], 0.0);
    let h = run("set burn duration of player to 3 seconds");
    assert_eq!(h.stats[&PStat::FireTicks], 60.0);
    let h = run("add 1 second to player's freeze time");
    assert_eq!(h.stats[&PStat::FreezeTicks], 60.0);
    let h = run("set glowing state of player to false\nset gravity of player to false");
    assert_eq!(h.stats[&PStat::Glowing], 0.0);
    assert_eq!(h.stats[&PStat::Gravity], 0.0);
    assert!(bad("set hotbar slot of player to 3"));
}

#[test]
fn flags() {
    let h = run("if player is glowing:\n    send \"glow\" to player\nif player is not gliding:\n    send \"walk\" to player");
    assert_eq!(h.sent, vec!["glow", "walk"]);
}

#[test]
fn locations() {
    assert_eq!(eval("y of head location of player"), "65.62");
    assert_eq!(eval("y of highest solid block at player"), "66");
    assert_eq!(eval("y of highest solid block at location(1, 5, 2)"), "63");
}

#[test]
fn players() {
    assert_eq!(eval("name of player from uuid \"u2\""), "Alex");
    let h = run("set {_p} to player from uuid \"nope\"\nif {_p} is not set:\n    send \"none\" to player");
    assert_eq!(h.sent, vec!["none"]);
    assert!(bad("send world from uuid \"u1\" to player"));
    let h = run("loop hidden players of player:\n    send \"%loop-value%\" to player");
    assert_eq!(h.sent.len(), 1);
    assert!(h.sent[0].contains("Alex"));
    assert_eq!(eval("first empty slot of player"), "2");
}

#[test]
fn lists() {
    let pre = "set {l::*} to \"a\", \"b\", \"a\", \"c\"\n";
    let h = run(&format!("{pre}send \"%first index of value \"a\" in {{l::*}}%\" to player"));
    assert_eq!(h.sent, vec!["1"]);
    let h = run(&format!("{pre}send \"%last index of value \"a\" in {{l::*}}%\" to player"));
    assert_eq!(h.sent, vec!["3"]);
    let h = run(&format!(
        "{pre}loop indices of value \"a\" in {{l::*}}:\n    send \"i%loop-value%\" to player"
    ));
    assert_eq!(h.sent, vec!["i1", "i3"]);
    let h = run(&format!(
        "{pre}set {{r::*}} to {{l::*}} except \"a\"\nsend \"%size of {{r::*}}%\" to player\nsend \"%join {{r::*}} with \",\"%\" to player"
    ));
    assert_eq!(h.sent, vec!["2", "b,c"]);
    let h = run(&format!(
        "{pre}set {{x::*}} to \"b\"\nset {{r::*}} to {{l::*}} except {{x::*}}\nsend \"%size of {{r::*}}%\" to player"
    ));
    assert_eq!(h.sent, vec!["3"]);
}

#[test]
fn misc() {
    assert_eq!(eval("hex code of \"red\""), "FF5555");
    assert_eq!(eval("hex code of \"#00ff7f\""), "00FF7F");
    assert_eq!(eval("id of diamond sword"), "minecraft:diamond_sword");
    assert_eq!(eval("exact item of 2 diamonds"), "2 diamond");
    assert_eq!(eval("1 hour and 5 seconds formatted"), "1:00:05");
    assert_eq!(eval("90 seconds formatted"), "1:30");
    let h = run("set {_t} to 3 minutes\nsend \"%{_t} formatted%\" to player");
    assert_eq!(h.sent, vec!["3:00"]);
}

#[test]
fn final_damage_in_damage_event() {
    let src = "on damage:\n    send \"%final damage%\" to player\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty());
    let mut h = Mock::new();
    let mut ctx = EventCtx::player(steve());
    ctx.damage = Some(6.0);
    e.fire(EventKind::Damage, ctx, &mut h);
    assert_eq!(h.sent, vec!["6"]);
}

//! Syntax from real server scripts: sounds at a location for everyone,
//! `on place of`, comparisons in words, `subtract`, chat edits, event
//! priorities, `region at` and the other lines that warn.

use std::collections::HashMap;

use carve_core::compile::{EventKind, check_registry};
use carve_core::expr_player_state::PStat;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    online: HashMap<String, String>,
    idle_ticks: f64,
}

impl Host for Mock {
    fn send_message(&mut self, _: &Recipient, text: &str) {
        self.calls.push(format!("send {text}"));
    }
    fn broadcast(&mut self, text: &str) {
        self.calls.push(text.to_string());
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
        true
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, _: LogLevel, _: &str) {}
    fn player_location(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        self.online
            .contains_key(&p.uuid)
            .then(|| (1.5, 64.0, -3.0, "world".to_string()))
    }
    fn online_players(&mut self) -> Vec<PlayerRef> {
        let mut v: Vec<PlayerRef> = self
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
    fn pstat(&mut self, _: &PlayerRef, s: PStat) -> Option<f64> {
        match s {
            PStat::IdleTicks => Some(self.idle_ticks),
            PStat::Yaw | PStat::Pitch => Some(0.0),
            _ => None,
        }
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn host() -> Mock {
    let mut h = Mock::default();
    h.online.insert("u1".into(), "Steve".into());
    h.online.insert("u2".into(), "Alex".into());
    h
}

fn load(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn errors(src: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    errs.iter().map(|e| e.to_string()).collect()
}

/// Run the lines in `on join` and return what the host saw.
fn run(lines: &str) -> Vec<String> {
    let src = format!(
        "on join:\n{}",
        lines.lines().map(|l| format!("    {l}\n")).collect::<String>()
    );
    let mut e = load(&src);
    let mut h = host();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    h.calls
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn sound_at_location_to_all_players() {
    let calls = run(
        "set {_l} to location(10, 20, 30, world \"world\")\nplay sound \"ENTITY_TNT_PRIMED\" with volume 0.7 and pitch 1 at {_l} to all players",
    );
    assert_eq!(
        calls,
        [
            "cmd playsound entity.tnt.primed master Alex 10 20 30 0.7 1",
            "cmd playsound entity.tnt.primed master Steve 10 20 30 0.7 1",
        ]
    );
}

#[test]
fn sound_to_all_players_plays_at_each_player() {
    let calls = run("play sound \"BLOCK_NOTE_BLOCK_PLING\" with volume 1 and pitch 2 to all players");
    assert_eq!(calls.len(), 2);
    assert!(calls[0].starts_with("cmd playsound block.note_block.pling master Alex 1.5 64 -3 1 2"));
}

#[test]
fn sound_at_a_moving_entity_location() {
    let calls = run("play sound \"entity.creeper.primed\" at player's location to player");
    assert_eq!(
        calls,
        ["cmd playsound entity.creeper.primed master Steve 1.5 64 -3 1 1"]
    );
}

fn place_ctx(block: &str) -> EventCtx {
    let mut c = EventCtx::player(steve());
    c.block = Some(block.into());
    c.block_at = Some((5.0, 70.0, 9.0, Some("world".into())));
    c
}

#[test]
fn on_place_of_matches_only_that_block() {
    let mut e = load(
        "on place of tnt:\n    broadcast \"tnt %event-block% at %event-block's location%\"\non place of chest or barrel:\n    broadcast \"box\"\non place:\n    broadcast \"any\"\n",
    );
    let mut h = host();
    e.fire(EventKind::Place, place_ctx("tnt"), &mut h);
    assert_eq!(h.calls.len(), 2);
    assert!(h.calls[0].starts_with("tnt tnt at "), "{:?}", h.calls);
    assert!(h.calls[0].contains("5") && h.calls[0].contains("70") && h.calls[0].contains("9"));
    assert_eq!(h.calls[1], "any");
    let mut h = host();
    e.fire(EventKind::Place, place_ctx("barrel"), &mut h);
    assert_eq!(h.calls, ["box", "any"]);
}

#[test]
fn event_block_compares_like_skript() {
    // `emerald` is the item, the placed block is `emerald_block`: not equal.
    let mut e = load(
        "on place:\n    if event-block is emerald:\n        broadcast \"item\"\n    if event-block is emerald block:\n        broadcast \"block\"\n",
    );
    let mut h = host();
    e.fire(EventKind::Place, place_ctx("emerald_block"), &mut h);
    assert_eq!(h.calls, ["block"]);
}

#[test]
fn event_block_location_in_break() {
    let mut e = load("on break:\n    broadcast \"%y-coordinate of event-block's location%\"\n");
    let mut h = host();
    let mut c = place_ctx("stone");
    c.block_at = Some((5.0, 70.0, 9.0, None));
    e.fire(EventKind::Break, c, &mut h);
    assert_eq!(h.calls, ["70"]);
}

#[test]
fn damage_cause_or_list() {
    let src = "on damage:\n    if damage cause is entity explosion or block explosion or primed tnt:\n        broadcast \"boom\"\n";
    let mut e = load(src);
    for (cause, hit) in [("explosion", true), ("player_explosion", true), ("fall", false)] {
        let mut h = host();
        let mut c = EventCtx::player(steve());
        c.damage = Some(2.0);
        c.damage_cause = Some(cause.into());
        e.fire(EventKind::Damage, c, &mut h);
        assert_eq!(h.calls.len(), usize::from(hit), "{cause}");
    }
}

#[test]
fn region_at_loads_with_a_warning_and_is_none() {
    let src = "on join:\n    if \"%region at player%\" contains \"none\":\n        broadcast \"nowhere\"\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty());
    assert_eq!(e.warnings.len(), 1);
    assert!(e.warnings[0].msg.contains("WorldGuard"));
    let mut h = host();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.calls, ["nowhere"]);
}

#[test]
fn prefix_and_yield_and_tab_complete_warn() {
    let src = "on chat:\n    set chat format to \"%player's prefix% %message%\"\non join:\n    set explosive yield of last spawned entity to 4\non tab complete:\n    if tab arg-1 contains \"x\":\n        cancel event\ncommand /bots [<text>]:\n    suggestions:\n        set arg-1's suggestions to \"a\", \"b\" and \"c\"\n    trigger:\n        send \"x\"\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty(), "{errs:?}");
    let msgs: Vec<&str> = e.warnings.iter().map(|w| w.msg.as_str()).collect();
    assert!(msgs.iter().any(|m| m.contains("prefix")), "{msgs:?}");
    assert!(msgs.iter().any(|m| m.contains("explosive yield")), "{msgs:?}");
    assert!(msgs.iter().any(|m| m.contains("tab complete")), "{msgs:?}");
    assert_eq!(
        msgs.iter().filter(|m| m.contains("suggestions")).count(),
        2,
        "{msgs:?}"
    );
}

#[test]
fn comparisons_in_words() {
    let calls = run(
        "set {_n} to 5\nif {_n} is greater than 3:\n    broadcast \"a\"\nif {_n} is less than 3:\n    broadcast \"b\"\nif {_n} is at least 5:\n    broadcast \"c\"\nif {_n} is at most 4:\n    broadcast \"d\"\nif {_n} is more than or equal to 6:\n    broadcast \"e\"\nif {_n} is not greater than 5:\n    broadcast \"f\"\nif player count is greater than 1:\n    broadcast \"g\"",
    );
    assert_eq!(calls, ["a", "c", "f", "g"]);
}

#[test]
fn subtract_numbers_and_times() {
    let calls = run(
        "set {_x} to 10\nsubtract 3 from {_x}\nbroadcast \"%{_x}%\"\nset {_t} to 5 minutes\nsubtract 1 minute from {_t}\nbroadcast \"%{_t}%\"",
    );
    assert_eq!(calls[0], "7");
    assert!(calls[1].contains('4'), "{calls:?}");
}

#[test]
fn coordinate_effects_change_one_axis() {
    let calls = run(
        "set {_l} to location(1, 2, 3, world \"w\")\nadd 1 to y-coordinate of {_l}\nbroadcast \"%y-coordinate of {_l}% %x-coordinate of {_l}%\"",
    );
    assert_eq!(calls, ["3 1"]);
}

#[test]
fn random_element_out_of_a_literal_list() {
    let calls = run(
        "set {_r} to a random element out of \"finn\" and \"soviet\"\nif {_r} is \"finn\" or \"soviet\":\n    broadcast \"ok\"",
    );
    assert_eq!(calls, ["ok"]);
}

#[test]
fn bracketed_default_on_the_left_of_a_comparison() {
    let calls = run("if ({x} ? 0) <= 0:\n    broadcast \"zero\"\nif ({x} ? 5) > 3:\n    broadcast \"five\"");
    assert_eq!(calls, ["zero", "five"]);
}

#[test]
fn chat_edits_go_out_as_slots() {
    let mut e = load(
        "on chat:\n    replace all \"ez\" in message with \"gg\"\n    set message to formatted message\n    set chat format to \"&7%player%: %message%\"\n",
    );
    let mut h = host();
    let o = e.fire(EventKind::Chat, EventCtx::chat(steve(), "ez win"), &mut h);
    let get = |slot: u8| {
        o.set
            .iter()
            .rev()
            .find(|(s, _)| *s == slot)
            .map(|(_, v)| v.to_string())
    };
    assert_eq!(
        get(carve_core::skript_compat::SLOT_CHAT_MESSAGE).as_deref(),
        Some("gg win")
    );
    assert_eq!(
        get(carve_core::skript_compat::SLOT_CHAT_FORMAT).as_deref(),
        Some("&7Steve: gg win")
    );
}

#[test]
fn priorities_run_lowest_first() {
    let src = "on join with priority highest:\n    broadcast \"highest\"\non join with priority monitor:\n    broadcast \"monitor\"\non join:\n    broadcast \"normal\"\non join with priority lowest:\n    broadcast \"lowest\"\non join with priority high:\n    broadcast \"high\"\n";
    let mut e = load(src);
    let mut h = host();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.calls, ["lowest", "normal", "high", "highest", "monitor"]);
    assert_eq!(e.table().event_priority(EventKind::Join), Some(5));
}

#[test]
fn priority_on_filtered_events_and_bad_priority() {
    let src = "on damage of player with priority highest:\n    cancel event\non death of player with priority highest:\n    broadcast \"x\"\n";
    assert!(errors(src).is_empty(), "{:?}", errors(src));
    let bad = errors("on join with priority loudest:\n    broadcast \"x\"\n");
    assert_eq!(bad.len(), 1);
    assert!(bad[0].contains("unknown priority"), "{bad:?}");
}

#[test]
fn idle_time_reads_the_host() {
    let src = "on join:\n    if idle time of player > 1 minute:\n        broadcast \"afk\"\n    else if idle time of player >= 5 seconds:\n        broadcast \"slow\"\n";
    let mut e = load(src);
    let mut h = host();
    h.idle_ticks = 2000.0;
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.calls, ["afk"]);
    let mut h = host();
    h.idle_ticks = 100.0;
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.calls, ["slow"]);
}

#[test]
fn items_kinds_and_colored_names() {
    let mut e = load(
        "on drop:\n    if item is sword or armor or zombie skull:\n        broadcast \"no\"\non join:\n    set block at location(1, 2, 3, world \"w\") to light blue wool\n    set {_s} to location 3 meters in front of player\n",
    );
    let mut h = host();
    let extra = carve_core::EventExtra {
        item: Some(("diamond_sword".into(), 1)),
        ..Default::default()
    };
    e.fire(EventKind::Drop, EventCtx::with_extra(steve(), extra), &mut h);
    assert_eq!(h.calls, ["no"]);
}

#[test]
fn unknown_function_is_named() {
    let e = errors("on join:\n    set {_a} to missingFunction({_b})\n");
    assert_eq!(e.len(), 1);
    assert!(e[0].contains("unknown function 'missingFunction'"), "{e:?}");
}

#[test]
fn six_argument_location_and_world_function() {
    let src = "function names() :: texts:\n    return \"a\", \"b\"\non join:\n    set {_l::*} to names()\n    set {s::finn} to location(0.5, 75, -149.5, world(\"world\"), 0, 0)\n    broadcast \"%size of {_l::*}% %y-coordinate of {s::finn}%\"\n";
    let mut e = load(src);
    let mut h = host();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.calls, ["2 75"]);
}

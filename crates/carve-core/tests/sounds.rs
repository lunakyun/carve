use std::collections::HashMap;

use carve_core::compile::{EventKind, check_registry, compile_file};
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    online: HashMap<String, String>,
    logs: Vec<(LogLevel, String)>,
}

impl Host for Mock {
    fn send_message(&mut self, _: &Recipient, _: &str) {}
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.calls.push(c.to_string());
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
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
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
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run(src: &str) -> (Vec<String>, Vec<String>) {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    let mut h = Mock::default();
    h.online.insert("u1".into(), "Steve".into());
    h.online.insert("u2".into(), "Alex".into());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let errors = h
        .logs
        .iter()
        .filter(|(l, _)| *l == LogLevel::Error)
        .map(|(_, m)| m.clone())
        .collect();
    (h.calls, errors)
}

fn ok(line: &str) -> Vec<String> {
    let (calls, errs) = run(&format!("on join:\n    {line}\n"));
    assert!(errs.is_empty(), "{errs:?}");
    calls
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn play_basic() {
    assert_eq!(
        ok(r#"play sound "entity.experience_orb.pickup" to player"#),
        ["playsound entity.experience_orb.pickup master Steve 1.5 64 -3 1 1"]
    );
}

#[test]
fn play_defaults_to_event_player() {
    assert_eq!(ok(r#"play sound "block.note_block.pling""#).len(), 1);
}

#[test]
fn play_volume_pitch() {
    assert_eq!(
        ok(r#"play sound "ui.toast.in" with volume 0.5 and pitch 2 to player"#),
        ["playsound ui.toast.in master Steve 1.5 64 -3 0.5 2"]
    );
    assert_eq!(
        ok(r#"play the sound "ui.toast.in" at volume 2 and at pitch 0.5 for player"#),
        ["playsound ui.toast.in master Steve 1.5 64 -3 2 0.5"]
    );
}

#[test]
fn pitch_is_clamped_and_id_lowercased() {
    assert_eq!(
        ok(r#"play sound "Minecraft:Entity.Cat.Meow" with pitch 9 to player"#),
        ["playsound minecraft:entity.cat.meow master Steve 1.5 64 -3 1 2"]
    );
}

#[test]
fn play_at_location_for_player() {
    assert_eq!(
        ok(r#"play sound "block.anvil.land" to player at location(10, 70, 20)"#),
        ["playsound block.anvil.land master Steve 10 70 20 1 1"]
    );
}

#[test]
fn play_at_location_for_everyone() {
    assert_eq!(
        ok(r#"play sound "block.anvil.land" at location(10, 70, 20)"#),
        [
            "playsound block.anvil.land master Alex 10 70 20 1 1",
            "playsound block.anvil.land master Steve 10 70 20 1 1"
        ]
    );
}

#[test]
fn play_to_each_looped_player() {
    assert_eq!(
        ok("loop all players:
        play sound \"block.anvil.land\" to loop-player"),
        [
            "playsound block.anvil.land master Alex 1.5 64 -3 1 1",
            "playsound block.anvil.land master Steve 1.5 64 -3 1 1"
        ]
    );
}

#[test]
fn stop_sounds() {
    assert_eq!(ok("stop all sounds"), ["stopsound Steve"]);
    assert_eq!(ok("stop all sounds for player"), ["stopsound Steve"]);
    assert_eq!(
        ok(r#"stop sound "music.game" from player"#),
        ["stopsound Steve * music.game"]
    );
}

#[test]
fn bad_sound_id_is_a_runtime_error() {
    let (calls, errs) = run("on join:\n    play sound \"bad id; op Steve\" to player\n");
    assert!(calls.is_empty());
    assert_eq!(errs.len(), 1);
    assert!(errs[0].contains("not a valid sound id"), "{errs:?}");
}

#[test]
fn variable_sound_name() {
    let (calls, errs) = run("on join:\n    set {s} to \"entity.player.levelup\"\n    play sound {s} to player\n");
    assert!(errs.is_empty());
    assert_eq!(calls, ["playsound entity.player.levelup master Steve 1.5 64 -3 1 1"]);
}

#[test]
fn bad_syntax_is_a_compile_error() {
    assert!(compile_file("b.carve", "on join:\n    play sound to player\n").is_err());
}

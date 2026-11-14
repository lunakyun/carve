use std::collections::HashMap;

use carve_core::compile::{EventKind, compile_file};
use carve_core::world_effects::{FxPos, ParticleFx};
use carve_core::*;

/// Records every world-effect call.
#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    online: HashMap<String, String>,
    effects: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Mock {
    fn steve() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m
    }
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, _text: &str) {}
    fn broadcast(&mut self, _text: &str) {}
    fn send_title(&mut self, _p: &PlayerRef, _t: &str, _s: Option<&str>) {}
    fn send_action_bar(&mut self, _p: &PlayerRef, _t: &str) {}
    fn heal(&mut self, _p: &PlayerRef) {}
    fn set_health(&mut self, _p: &PlayerRef, _h: f64) {}
    fn kill(&mut self, _p: &PlayerRef) {}
    fn teleport(&mut self, _p: &PlayerRef, _x: f64, _y: f64, _z: f64) {}
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
    fn has_permission(&mut self, _p: &PlayerRef, _n: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _t: u64, _k: u64) {}
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
    fn strike_lightning(&mut self, at: &FxPos, effect_only: bool) {
        self.calls
            .push(format!("lightning {} {} {} {effect_only}", at.x, at.y, at.z));
    }
    fn explode(&mut self, at: &FxPos, power: f64, fire: bool) {
        self.calls.push(format!(
            "explode {} {} {} p={power} fire={fire}",
            at.x, at.y, at.z
        ));
    }
    fn spawn_particle(&mut self, p: &ParticleFx) {
        self.calls.push(format!(
            "particle {} {} {} {} n={} spread={} speed={}",
            p.name, p.at.x, p.at.y, p.at.z, p.count, p.spread, p.speed
        ));
    }
    fn add_effect(&mut self, p: &PlayerRef, effect: &str, ticks: u32, amp: u8) {
        self.effects.push(effect.to_string());
        self.calls
            .push(format!("effect {} {effect} {ticks} {amp}", p.name));
    }
    fn clear_effects(&mut self, p: &PlayerRef, effect: Option<&str>) {
        self.effects.clear();
        self.calls
            .push(format!("clear {} {}", p.name, effect.unwrap_or("*")));
    }
    fn has_effect(&mut self, _p: &PlayerRef, effect: &str) -> bool {
        self.effects.iter().any(|e| e == effect)
    }
    fn set_fire(&mut self, p: &PlayerRef, ticks: i32) {
        self.calls.push(format!("fire {} {ticks}", p.name));
    }
}

/// A host that implements only the required methods, so the trait defaults run.
#[derive(Default)]
struct Bare {
    cmds: Vec<String>,
    warns: Vec<String>,
}

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
        self.cmds.push(c.to_string());
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
    fn log(&mut self, level: LogLevel, msg: &str) {
        if level == LogLevel::Warn {
            self.warns.push(msg.to_string());
        }
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn errors_of(src: &str) -> Vec<ScriptError> {
    compile_file("bad.carve", src).err().unwrap_or_default()
}

fn script(body: &str) -> String {
    let lines: String = body.lines().map(|l| format!("    {l}\n")).collect();
    format!("on join:\n{lines}")
}

fn load(body: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("test.carve".into(), script(body))]);
    assert!(
        errs.is_empty(),
        "errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn run(body: &str) -> Vec<String> {
    let mut e = load(body);
    let mut h = Mock::steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
    h.calls
}

fn run_bare(body: &str) -> Bare {
    let mut e = load(body);
    let mut h = Bare::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    h
}

#[test]
fn lightning() {
    assert_eq!(run("strike lightning at player"), ["lightning 1.5 64 -3 false"]);
    assert_eq!(
        run("strike lightning at location(1, 2, 3)"),
        ["lightning 1 2 3 false"]
    );
    assert_eq!(
        run("strike lightning effect at player"),
        ["lightning 1.5 64 -3 true"]
    );
    assert_eq!(
        run("show a lightning effect at location(0, 70, 0)"),
        ["lightning 0 70 0 true"]
    );
}

#[test]
fn lightning_default_uses_summon() {
    let h = run_bare("strike lightning at location(1, 2, 3)");
    assert_eq!(h.cmds, ["summon minecraft:lightning_bolt 1 2 3"]);
}

#[test]
fn explosion() {
    assert_eq!(
        run("create explosion of force 3 at player"),
        ["explode 1.5 64 -3 p=3 fire=false"]
    );
    assert_eq!(
        run("create an explosion with power 2 with fire at location(0, 1, 0)"),
        ["explode 0 1 0 p=2 fire=true"]
    );
    assert_eq!(
        run("make explosion at player"),
        ["explode 1.5 64 -3 p=3 fire=false"]
    );
    // Absurd force is clamped.
    assert_eq!(
        run("create explosion of force 9999 at player"),
        ["explode 1.5 64 -3 p=100 fire=false"]
    );
}

#[test]
fn unsupported_effects_warn_on_a_bare_host() {
    let h = run_bare("create explosion at location(1, 2, 3)\nignite player\nextinguish player");
    assert_eq!(h.warns.len(), 3, "{:?}", h.warns);
    assert!(h.cmds.is_empty());
}

#[test]
fn particles() {
    assert_eq!(
        run("play 5 flame particles at player"),
        ["particle flame 1.5 64 -3 n=5 spread=0 speed=0"]
    );
    assert_eq!(
        run("show 20 happy villager particles at location(1, 2, 3) with spread 0.5 with speed 0.1"),
        ["particle happy_villager 1 2 3 n=20 spread=0.5 speed=0.1"]
    );
    assert_eq!(
        run("show crit particles at player"),
        ["particle crit 1.5 64 -3 n=1 spread=0 speed=0"]
    );
    assert_eq!(
        run("set {_n} to 7\nplay {_n} flame particles at player"),
        ["particle flame 1.5 64 -3 n=7 spread=0 speed=0"]
    );
}

#[test]
fn defaults_use_console_commands() {
    let h = run_bare(
        "play 5 flame particles at location(1, 2, 3) with spread 0.5 with speed 0.1\napply speed 2 to player for 10 seconds\nremove all potion effects from player\nremove speed potion effect from player",
    );
    assert_eq!(
        h.cmds,
        [
            "particle minecraft:flame 1 2 3 0.5 0.5 0.5 0.1 5",
            "effect give Steve minecraft:speed 10 1 false",
            "effect clear Steve",
            "effect clear Steve minecraft:speed",
        ]
    );
}

#[test]
fn potions() {
    assert_eq!(
        run("apply potion of speed 2 to player for 10 seconds"),
        ["effect Steve speed 200 1"]
    );
    assert_eq!(run("apply speed to player"), ["effect Steve speed 300 0"]);
    assert_eq!(
        run("apply fire resistance of tier 3 to player for 2 minutes"),
        ["effect Steve fire_resistance 2400 2"]
    );
    assert_eq!(
        run("apply jump boost 5 to player for 40 ticks"),
        ["effect Steve jump_boost 40 4"]
    );
    assert_eq!(
        run("apply regen to player for 1 second"),
        ["effect Steve regeneration 20 0"]
    );
}

#[test]
fn clear_potions() {
    assert_eq!(
        run("apply speed to player\nremove all potion effects from player"),
        ["effect Steve speed 300 0", "clear Steve *"]
    );
    assert_eq!(run("clear potion effects of player"), ["clear Steve *"]);
    assert_eq!(
        run("remove the speed potion effect from player"),
        ["clear Steve speed"]
    );
}

#[test]
fn has_potion_effect() {
    let out = run(
        "if player has potion effect speed:\n    apply poison to player\napply speed to player\nif player has potion effect speed:\n    apply slowness to player\nif player does not have potion effect poison:\n    apply haste to player\nif player doesn't have the effect haste:\n    apply weakness to player",
    );
    assert_eq!(
        out,
        [
            "effect Steve speed 300 0",
            "effect Steve slowness 300 0",
            "effect Steve haste 300 0",
        ]
    );
}

#[test]
fn fire() {
    assert_eq!(run("ignite player for 5 seconds"), ["fire Steve 100"]);
    assert_eq!(run("ignite player"), ["fire Steve 100"]);
    assert_eq!(run("set player on fire for 3 seconds"), ["fire Steve 60"]);
    assert_eq!(run("extinguish player"), ["fire Steve 0"]);
}

#[test]
fn bad_input_is_a_compile_error() {
    let errs = errors_of("on join:\n    apply flying to player\n");
    assert!(
        errs.iter()
            .any(|e| e.to_string().contains("unknown potion effect")),
        "{errs:?}"
    );
    let errs = errors_of("on join:\n    play 3 sparkly particles at player\n");
    assert!(
        errs.iter().any(|e| e.to_string().contains("unknown particle")),
        "{errs:?}"
    );
    assert!(!errors_of("on join:\n    strike lightning at 5\n").is_empty());
    assert!(!errors_of("on join:\n    ignite player for soon\n").is_empty());
}

#[test]
fn old_syntax_still_works() {
    let src = "on join:\n    set {x} to 5\n    add 1 to {x}\n    remove 1 from {x}\n    if player has diamond:\n        stop\n";
    assert!(errors_of(src).is_empty());
}

#[test]
fn name_tables() {
    use carve_core::world_effects::{EFFECTS, PARTICLES};
    assert!(EFFECTS.contains(&"speed") && EFFECTS.contains(&"fire_resistance"));
    assert!(PARTICLES.contains(&"flame") && PARTICLES.contains(&"happy_villager"));
}

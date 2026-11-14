use std::collections::HashMap;

use carve_core::combat::{EntSlot, SLOT_DAMAGE, SLOT_DEATH_MESSAGE};
use carve_core::compile::{EventKind, compile_file};
use carve_core::*;

/// Records messages and knows a few players and entities by entity id.
#[derive(Default)]
struct Mock {
    out: Vec<String>,
    online: HashMap<String, String>,
    /// entity id -> (uuid, kind, custom name)
    ents: HashMap<i32, (String, String, Option<String>)>,
    lookups: usize,
    logs: Vec<(LogLevel, String)>,
}

impl Mock {
    fn new() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m.online.insert("u2".into(), "Alex".into());
        m.ents.insert(7, ("z7".into(), "zombie".into(), None));
        m.ents
            .insert(8, ("s8".into(), "skeleton".into(), Some("Bones".into())));
        m
    }
    fn take(&mut self) -> Vec<String> {
        std::mem::take(&mut self.out)
    }
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let who = match to {
            Recipient::Player(p) => p.name.clone(),
            Recipient::Console => "console".into(),
        };
        self.out.push(format!("{who}: {text}"));
    }
    fn broadcast(&mut self, text: &str) {
        self.out.push(format!("all: {text}"));
    }
    fn send_title(&mut self, _p: &PlayerRef, _t: &str, _s: Option<&str>) {}
    fn send_action_bar(&mut self, _p: &PlayerRef, _t: &str) {}
    fn heal(&mut self, _p: &PlayerRef) {}
    fn set_health(&mut self, _p: &PlayerRef, _h: f64) {}
    fn kill(&mut self, _p: &PlayerRef) {}
    fn teleport(&mut self, _p: &PlayerRef, _x: f64, _y: f64, _z: f64) {}
    fn execute_command(&mut self, _c: &str) {}
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
    fn online_players(&mut self) -> Vec<PlayerRef> {
        Vec::new()
    }
    fn entity_by_id(&mut self, id: i32) -> Option<EntityRef> {
        self.lookups += 1;
        self.ents.get(&id).map(|(u, k, _)| EntityRef {
            uuid: u.clone(),
            kind: k.clone(),
        })
    }
    fn entity_name(&mut self, e: &EntityRef) -> Option<String> {
        self.ents.values().find(|(u, _, _)| *u == e.uuid)?.2.clone()
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

fn engine(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "unexpected errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn hit(victim: Option<PlayerRef>, victim_id: i32, damage: f64) -> EventCtx {
    let mut c = match victim {
        Some(p) => EventCtx::player(p),
        None => EventCtx::entity_victim(EntSlot::Id(victim_id)),
    };
    c.damage = Some(damage);
    c.damage_cause = Some("mob_attack".into());
    c
}

fn num_out(o: &FireOutcome, slot: u8) -> Option<f64> {
    o.set
        .iter()
        .rev()
        .find(|(s, _)| *s == slot)
        .and_then(|(_, v)| match v {
            Value::Num(n) => Some(*n),
            _ => None,
        })
}

#[test]
fn damage_fires_for_mob_victims() {
    let mut e = engine(
        "on damage:\n    broadcast \"%type of victim% hurt %damage% player=%player%\"\n    if victim is a zombie:\n        broadcast \"zombie!\"",
    );
    let mut h = Mock::new();
    e.fire(EventKind::Damage, hit(None, 7, 4.0), &mut h);
    e.fire(EventKind::Damage, hit(Some(steve()), 0, 2.0), &mut h);
    assert_eq!(
        h.take(),
        [
            "all: zombie hurt 4 player=<none>",
            "all: zombie!",
            "all: player hurt 2 player=Steve"
        ]
    );
}

#[test]
fn victim_filters() {
    let mut e = engine(
        "on damage of player:\n    broadcast \"p %victim%\"\non damage of a zombie:\n    broadcast \"z %damage%\"\non damage of skeletons:\n    broadcast \"s\"",
    );
    let mut h = Mock::new();
    e.fire(EventKind::Damage, hit(None, 7, 3.0), &mut h);
    e.fire(EventKind::Damage, hit(Some(steve()), 0, 1.0), &mut h);
    e.fire(EventKind::Damage, hit(None, 8, 1.0), &mut h);
    assert_eq!(h.take(), ["all: z 3", "all: p Steve", "all: s"]);
    let errs = compile_file(
        "bad.carve",
        "on damage of dragons and stuff:\n    broadcast \"x\"",
    )
    .unwrap_err();
    assert!(errs[0].to_string().contains("not a valid name"), "{errs:?}");
}

#[test]
fn entity_lookup_is_lazy_and_once_per_trigger() {
    let mut e =
        engine("on damage:\n    broadcast \"%damage%\"\non damage:\n    broadcast \"%victim% %victim%\"");
    let mut h = Mock::new();
    e.fire(EventKind::Damage, hit(None, 7, 1.0), &mut h);
    assert_eq!(h.lookups, 1);
    assert_eq!(h.take(), ["all: 1", "all: zombie zombie"]);
}

#[test]
fn entity_attackers() {
    let mut e = engine(
        "on damage:\n    if attacker is a player:\n        broadcast \"player %attacker%\"\n    else if attacker is a zombie:\n        broadcast \"zombie %name of attacker% %type of attacker%\"\n    else if attacker is set:\n        broadcast \"other %name of attacker% %type of attacker%\"\n    else:\n        broadcast \"none\"",
    );
    let mut h = Mock::new();
    let mut c = hit(Some(steve()), 0, 1.0);
    c.attacker = Some(alex());
    e.fire(EventKind::Damage, c, &mut h);
    let mut c = hit(Some(steve()), 0, 1.0);
    c.set_entity_attacker(EntSlot::Id(7));
    e.fire(EventKind::Damage, c, &mut h);
    let mut c = hit(None, 7, 1.0);
    c.set_entity_attacker(EntSlot::Id(8));
    e.fire(EventKind::Damage, c, &mut h);
    let mut c = hit(None, 7, 1.0);
    c.set_entity_attacker(EntSlot::Id(99));
    e.fire(EventKind::Damage, c, &mut h);
    assert_eq!(
        h.take(),
        [
            "all: player Alex",
            "all: zombie zombie zombie",
            "all: other Bones skeleton",
            "all: none"
        ]
    );
}

#[test]
fn damage_can_be_changed() {
    let mut e = engine(
        "on damage:\n    set damage to 10\n    add 2 to damage\n    remove 5 from the damage\n    broadcast \"a %damage%\"\non damage:\n    broadcast \"b %damage%\"\n    remove 100 from damage",
    );
    let mut h = Mock::new();
    let o = e.fire(EventKind::Damage, hit(Some(steve()), 0, 3.0), &mut h);
    assert_eq!(h.take(), ["all: a 7", "all: b 7"]);
    assert_eq!(num_out(&o, SLOT_DAMAGE), Some(0.0));
    // No change: nothing to write back.
    let mut e = engine("on damage:\n    broadcast \"%damage%\"");
    let o = e.fire(EventKind::Damage, hit(Some(steve()), 0, 3.0), &mut h);
    assert!(o.set.is_empty());
}

#[test]
fn damage_change_after_wait_does_not_count() {
    let mut e = engine("on damage:\n    wait 1 tick\n    set damage to 1");
    let mut h = Mock::new();
    let o = e.fire(EventKind::Damage, hit(Some(steve()), 0, 3.0), &mut h);
    assert_eq!(num_out(&o, SLOT_DAMAGE), None);
}

#[test]
fn damage_effects_only_in_their_events() {
    for src in [
        "on join:\n    set damage to 1",
        "on death:\n    set damage to 1",
        "on damage:\n    set death message to \"x\"",
        "on damage:\n    set damage to \"lots\"",
    ] {
        assert!(compile_file("bad.carve", src).is_err(), "{src}");
    }
}

#[test]
fn death_attacker_and_message() {
    let mut e = engine(
        "on death:\n    if attacker is a zombie:\n        set death message to \"&c%player% was eaten by %attacker%\"\n    else if attacker is set:\n        set death message to \"%player% lost to %attacker%\"\n    broadcast \"%death message%\"\non death:\n    broadcast \"2 %death message%\"",
    );
    let mut h = Mock::new();
    let mut c = EventCtx::player(steve());
    c.death_message = Some("Steve died".into());
    c.set_entity_attacker(EntSlot::Id(7));
    let o = e.fire(EventKind::Death, c, &mut h);
    assert_eq!(
        h.take(),
        [
            "all: &cSteve was eaten by zombie",
            "all: 2 &cSteve was eaten by zombie"
        ]
    );
    assert_eq!(
        o.set.last(),
        Some(&(
            SLOT_DEATH_MESSAGE,
            Value::Text("&cSteve was eaten by zombie".into())
        ))
    );
    let mut c = EventCtx::player(steve());
    c.death_message = Some("Steve died".into());
    c.attacker = Some(alex());
    e.fire(EventKind::Death, c, &mut h);
    let mut c = EventCtx::player(steve());
    c.death_message = Some("Steve died".into());
    let o = e.fire(EventKind::Death, c, &mut h);
    assert_eq!(
        h.take(),
        [
            "all: Steve lost to Alex",
            "all: 2 Steve lost to Alex",
            "all: Steve died",
            "all: 2 Steve died"
        ]
    );
    assert!(o.set.is_empty());
}

#[test]
fn player_scripts_still_work() {
    let mut e = engine(
        "on damage:\n    send \"hit %damage% by %attacker%\" to victim\n    send \"you too\" to player\n    send \"ow\" to attacker",
    );
    let mut h = Mock::new();
    let mut c = hit(Some(steve()), 0, 3.0);
    c.attacker = Some(alex());
    e.fire(EventKind::Damage, c, &mut h);
    // Mob victim and mob attacker: nobody to send to, and no errors.
    let mut c = hit(None, 7, 3.0);
    c.set_entity_attacker(EntSlot::Id(8));
    e.fire(EventKind::Damage, c, &mut h);
    assert_eq!(h.take(), ["Steve: hit 3 by Alex", "Steve: you too", "Alex: ow"]);
    assert!(h.logs.iter().all(|(l, _)| *l != LogLevel::Error), "{:?}", h.logs);
}

use carve_core::compile::check_registry;
use carve_core::events_game::{Ev, SPECS, Slot, Who, make_ctx, parse, spec};
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        match to {
            Recipient::Player(p) => self.calls.push(format!("send {} {}", p.name, text)),
            Recipient::Console => self.calls.push(format!("console {text}")),
        }
    }
    fn broadcast(&mut self, t: &str) {
        self.calls.push(format!("bc {t}"));
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
        (uuid == "u1").then(|| "Steve".to_string())
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
        self.logs.push((level, msg.to_string()));
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn load(src: &str) -> (Engine, Vec<ScriptError>) {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    (e, errs)
}

/// Fire an event with a context and return the host calls and the cancel flag.
fn fire(src: &str, ev: Ev, ctx: EventCtx) -> (Vec<String>, bool) {
    let (mut e, errs) = load(src);
    assert!(errs.is_empty(), "unexpected errors: {errs:?}");
    let mut h = Mock::default();
    let out = e.fire(EventKind::Ext2(ev), ctx, &mut h);
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
    (h.calls, out.cancelled)
}

fn err_of(src: &str) -> String {
    let (_, errs) = load(src);
    assert!(!errs.is_empty(), "expected an error for {src:?}");
    errs[0].to_string()
}

#[test]
fn registry_still_compiles() {
    check_registry().unwrap();
}

#[test]
fn every_name_compiles() {
    for sp in SPECS {
        for n in sp.names {
            let body = if sp.cancel { "cancel event" } else { "stop" };
            let (_, errs) = load(&format!("on {n}:\n    {body}\n"));
            assert!(errs.is_empty(), "'on {n}': {errs:?}");
            assert_eq!(parse(n), Some(sp.ev));
        }
    }
}

#[test]
fn every_value_has_a_slot_inside_the_counts() {
    for sp in SPECS {
        for (name, slot) in sp.vals {
            let (ok, kind) = match *slot {
                Slot::Num(i) => (i < sp.counts.0, "num"),
                Slot::Text(i) => (i < sp.counts.1, "text"),
                Slot::Flag(i) => (i < sp.counts.2, "flag"),
            };
            assert!(ok, "{:?} '{name}' {kind} slot out of range", sp.ev);
        }
        // Every slot below the counts is reachable by some name.
        for i in 0..sp.counts.0 {
            assert!(
                sp.vals.iter().any(|(_, s)| *s == Slot::Num(i)),
                "{:?} num {i} unnamed",
                sp.ev
            );
        }
        for i in 0..sp.counts.1 {
            assert!(
                sp.vals.iter().any(|(_, s)| *s == Slot::Text(i)),
                "{:?} text {i} unnamed",
                sp.ev
            );
        }
        for i in 0..sp.counts.2 {
            assert!(
                sp.vals.iter().any(|(_, s)| *s == Slot::Flag(i)),
                "{:?} flag {i} unnamed",
                sp.ev
            );
        }
    }
}

#[test]
fn every_value_name_compiles_and_reads() {
    for sp in SPECS {
        for (name, slot) in sp.vals {
            let src = format!("on {}:\n    broadcast \"[%{name}%]\"\n", sp.names[0]);
            let (_, errs) = load(&src);
            assert!(errs.is_empty(), "{:?} '{name}': {errs:?}", sp.ev);
            let k = sp.counts;
            let nums: Vec<f64> = (0..k.0).map(|i| f64::from(i) + 1.0).collect();
            let texts: Vec<String> = (0..k.1).map(|i| format!("t{i}")).collect();
            let texts: Vec<&str> = texts.iter().map(String::as_str).collect();
            let flags = vec![true; k.2 as usize];
            let p = (sp.who != Who::Nobody).then(steve);
            let want = match *slot {
                Slot::Num(i) => format!("{}", i + 1),
                Slot::Text(i) => format!("t{i}"),
                Slot::Flag(_) => "true".to_string(),
            };
            let (calls, _) = fire(&src, sp.ev, make_ctx(sp.ev, p, &nums, &texts, &flags));
            assert_eq!(calls, [format!("bc [{want}]")], "{:?} '{name}'", sp.ev);
        }
    }
}

#[test]
fn heal_values_and_player() {
    let src = "on heal:\n    send \"%player% +%heal amount%\"\n    send \"id %event-entity%\"\n";
    let ctx = make_ctx(Ev::Heal, Some(steve()), &[2.5, 7.0], &[], &[]);
    let (calls, c) = fire(src, Ev::Heal, ctx);
    assert_eq!(calls, ["send Steve Steve +2.5", "send Steve id 7"]);
    assert!(!c);
}

#[test]
fn heal_works_for_non_players() {
    let src = "on heal:\n    broadcast \"healed %heal amount%\"\n";
    let ctx = make_ctx(Ev::Heal, None, &[4.0, 9.0], &[], &[]);
    assert_eq!(fire(src, Ev::Heal, ctx).0, ["bc healed 4"]);
}

#[test]
fn cancel_event_sets_the_flag() {
    let src = "on hunger meter change:\n    if new food level < 10:\n        cancel event\n";
    let low = make_ctx(Ev::Hunger, Some(steve()), &[5.0, 1.0], &[], &[]);
    let high = make_ctx(Ev::Hunger, Some(steve()), &[18.0, 1.0], &[], &[]);
    assert!(fire(src, Ev::Hunger, low).1);
    assert!(!fire(src, Ev::Hunger, high).1);
}

#[test]
fn text_and_math_on_values() {
    let src = "on grow:\n    if old block is \"wheat\":\n        broadcast \"%new block% at %block x%,%block y%,%block z%\"\n    set {_t} to block x + block y\n    broadcast \"%{_t}%\"\n";
    let ctx = make_ctx(Ev::Grow, None, &[1.0, 64.0, -3.0], &["wheat", "wheat_2"], &[]);
    assert_eq!(fire(src, Ev::Grow, ctx).0, ["bc wheat_2 at 1,64,-3", "bc 65"]);
}

#[test]
fn generic_event_value_names() {
    let src = "on item damage:\n    broadcast \"%event-item% %event-number%\"\n";
    let ctx = make_ctx(Ev::ItemDamage, Some(steve()), &[3.0], &["diamond_pickaxe"], &[]);
    assert_eq!(fire(src, Ev::ItemDamage, ctx).0, ["bc diamond_pickaxe 3"]);
}

#[test]
fn flags_work_in_conditions() {
    let src = "on gliding state change:\n    if gliding state is true:\n        broadcast \"up\"\n    else:\n        broadcast \"down\"\n";
    let up = make_ctx(Ev::Glide, Some(steve()), &[3.0], &[], &[true]);
    let down = make_ctx(Ev::Glide, Some(steve()), &[3.0], &[], &[false]);
    assert_eq!(fire(src, Ev::Glide, up).0, ["bc up"]);
    assert_eq!(fire(src, Ev::Glide, down).0, ["bc down"]);
}

#[test]
fn nobody_events_have_no_player() {
    let e = err_of("on breed:\n    broadcast \"%player%\"\n");
    assert!(e.contains("doesn't exist"), "{e}");
    let e = err_of("on explode:\n    broadcast \"%player%\"\n");
    assert!(e.contains("doesn't exist"), "{e}");
}

#[test]
fn player_events_allow_player() {
    let (_, errs) = load("on enchant:\n    send \"%player% %enchantment cost%\"\n");
    assert!(errs.is_empty(), "{errs:?}");
}

#[test]
fn values_are_not_available_in_other_events() {
    let e = err_of("on join:\n    broadcast \"%heal amount%\"\n");
    assert!(!e.is_empty());
}

#[test]
fn non_cancellable_events_say_so() {
    let e = err_of("on item break:\n    cancel event\n");
    assert!(e.contains("can't be cancelled"), "{e}");
    let e = err_of("on inventory close:\n    cancel event\n");
    assert!(e.contains("can't be cancelled"), "{e}");
}

#[test]
fn unknown_event_lists_new_names() {
    let e = err_of("on nonsense event:\n    stop\n");
    assert!(e.contains("heal") && e.contains("fishing"), "{e}");
}

#[test]
fn old_events_still_work() {
    let (mut e, errs) = load("on join:\n    broadcast \"hi %player%\"\n");
    assert!(errs.is_empty());
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.calls, ["bc hi Steve"]);
}

#[test]
fn has_event_per_kind() {
    let (e, errs) = load("on heal:\n    stop\n");
    assert!(errs.is_empty());
    assert!(e.table().has_event(EventKind::Ext2(Ev::Heal)));
    assert!(!e.table().has_event(EventKind::Ext2(Ev::Grow)));
    assert!(!e.table().has_event(EventKind::Join));
}

#[test]
fn spec_lookup_covers_all() {
    for sp in SPECS {
        assert_eq!(spec(sp.ev).ev, sp.ev);
    }
}

#[test]
fn names_do_not_clash_with_events_1() {
    for n in carve_core::events_game::known_names().split(", ") {
        assert!(
            carve_core::events_blocks_players::ExtEvent::parse(n).is_none(),
            "clash: {n}"
        );
    }
}

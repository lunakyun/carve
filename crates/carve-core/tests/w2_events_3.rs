use carve_core::compile::{EventKind, check_registry};
use carve_core::w2_events_3::Ev3;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    errors: Vec<String>,
}

impl Host for Mock {
    fn send_message(&mut self, _: &Recipient, text: &str) {
        self.calls.push(text.to_string());
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
    fn log(&mut self, level: LogLevel, msg: &str) {
        if level == LogLevel::Error {
            self.errors.push(msg.to_string());
        }
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
    let (_, errs) = e.load(&[("t.carve".into(), src.to_string())]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn errors_of(src: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.to_string())]);
    errs.iter().map(|e| e.to_string()).collect()
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn every_event_name_parses_and_loads() {
    for ev in Ev3::all() {
        let src = format!("on {}:\n    stop\n", ev.name());
        let e = engine(&src);
        assert!(e.table().has_event(EventKind::Ext3(ev)), "{}", ev.name());
    }
}

#[test]
fn kick_reason_and_cancel() {
    let mut e = engine("on kick:\n    send \"%player% kicked: %event-text%\"\n    cancel event\n");
    let mut h = Mock::default();
    let mut ctx = EventCtx::player(steve());
    ctx.message = Some("flying".into());
    let out = e.fire(EventKind::Ext3(Ev3::Kick), ctx, &mut h);
    assert!(out.cancelled);
    assert_eq!(h.calls, vec!["Steve kicked: flying"]);
}

#[test]
fn message_alias_works_with_text_events() {
    let mut e = engine("on language change:\n    send \"lang %message%\"\n");
    let mut h = Mock::default();
    let mut ctx = EventCtx::player(steve());
    ctx.message = Some("de_de".into());
    e.fire(EventKind::Ext3(Ev3::LanguageChange), ctx, &mut h);
    assert_eq!(h.calls, vec!["lang de_de"]);
}

#[test]
fn redstone_number_without_player() {
    let mut e =
        engine("on redstone:\n    if event-number > 7:\n        broadcast \"strong %event-number%\"\n");
    let mut h = Mock::default();
    let mut ctx = EventCtx::console();
    ctx.damage = Some(12.0);
    e.fire(EventKind::Ext3(Ev3::Redstone), ctx, &mut h);
    assert_eq!(h.calls, vec!["broadcast strong 12"]);
    let mut ctx = EventCtx::console();
    ctx.damage = Some(3.0);
    e.fire(EventKind::Ext3(Ev3::Redstone), ctx, &mut h);
    assert_eq!(h.calls.len(), 1);
}

#[test]
fn piston_direction() {
    let mut e = engine("on piston extend:\n    broadcast event-text\n");
    let mut h = Mock::default();
    let mut ctx = EventCtx::console();
    ctx.message = Some("up".into());
    e.fire(EventKind::Ext3(Ev3::PistonExtend), ctx, &mut h);
    assert_eq!(h.calls, vec!["broadcast up"]);
}

#[test]
fn aliases() {
    assert_eq!(Ev3::parse("locale change"), Some(Ev3::LanguageChange));
    assert_eq!(Ev3::parse("totem"), Some(Ev3::ResurrectAttempt));
    assert_eq!(Ev3::parse("nonsense"), None);
}

#[test]
fn values_only_where_they_exist() {
    assert!(!errors_of("on unleash:\n    send \"%event-number%\"\n").is_empty());
    assert!(!errors_of("on redstone:\n    send \"%event-text%\"\n").is_empty());
    assert!(!errors_of("on lightning strike:\n    send \"%me%\"\n").is_empty());
    assert!(!errors_of("on join:\n    send \"%event-text%\"\n").is_empty());
}

#[test]
fn unknown_event_is_an_error() {
    assert!(errors_of("on frobnicate:\n    stop\n")[0].contains("unknown event"));
}

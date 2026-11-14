use carve_core::compile::{EventKind, check_registry};
use carve_core::w4_events_movement::*;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
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
    fn log(&mut self, _: LogLevel, _: &str) {}
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

fn fire(e: &mut Engine, ev: MvEvent, ctx: EventCtx, h: &mut Mock) -> bool {
    e.fire(EventKind::Mv(ev), ctx, h).cancelled
}

fn loc(x: f64, w: &str) -> Value {
    Value::Location {
        x,
        y: 64.0,
        z: 0.0,
        world: Some(w.into()),
    }
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn every_event_loads() {
    for ev in MvEvent::all() {
        let e = engine(&format!("on {}:\n    stop\n", ev.name()));
        assert!(e.table().has_event(EventKind::Mv(ev)), "{}", ev.name());
    }
}

#[test]
fn aliases() {
    assert_eq!(MvEvent::parse("player move"), Some(MvEvent::Move));
    assert_eq!(MvEvent::parse("swap hands"), Some(MvEvent::SwapHands));
    assert_eq!(MvEvent::parse("first login"), Some(MvEvent::FirstJoin));
    assert_eq!(MvEvent::parse("rotate"), None);
}

#[test]
fn command_values_and_cancel() {
    let mut e = engine(
        "on command:\n    send \"cmd=%command% full=%full command%\"\n    if command is \"spawn\":\n        cancel event\n",
    );
    let mut h = Mock::default();
    let mut ctx = EventCtx::player(steve());
    ctx.ev = command_vals("/Spawn now please");
    assert!(fire(&mut e, MvEvent::Command, ctx, &mut h));
    assert_eq!(h.calls, vec!["cmd=spawn full=spawn now please"]);
    let mut ctx = EventCtx::player(steve());
    ctx.ev = command_vals("home");
    assert!(!fire(&mut e, MvEvent::Command, ctx, &mut h));
}

#[test]
fn command_filter_by_name() {
    let mut e = engine("on command \"/home\":\n    send \"home!\"\n");
    let mut h = Mock::default();
    for raw in ["home", "/home a b", "warp x"] {
        let mut ctx = EventCtx::player(steve());
        ctx.ev = command_vals(raw);
        fire(&mut e, MvEvent::Command, ctx, &mut h);
    }
    assert_eq!(h.calls, vec!["home!", "home!"]);
}

#[test]
fn command_filter_with_arguments_uses_full_command() {
    let mut e = engine("on command \"gamemode creative\":\n    send \"gm\"\n");
    let mut h = Mock::default();
    for raw in ["gamemode creative", "gamemode survival"] {
        let mut ctx = EventCtx::player(steve());
        ctx.ev = command_vals(raw);
        fire(&mut e, MvEvent::Command, ctx, &mut h);
    }
    assert_eq!(h.calls, vec!["gm"]);
}

#[test]
fn first_join() {
    let mut e = engine("on first join:\n    send \"welcome %player%\"\n");
    let mut h = Mock::default();
    fire(&mut e, MvEvent::FirstJoin, EventCtx::player(steve()), &mut h);
    assert_eq!(h.calls, vec!["welcome Steve"]);
}

#[test]
fn move_locations_and_cancel() {
    let mut e = engine(
        "on move:\n    send \"to %x of event-location% from %x of past event-location%\"\n    cancel event\n",
    );
    let mut h = Mock::default();
    let mut ctx = EventCtx::player(steve());
    ctx.ev = vec![loc(5.0, "world"), loc(4.0, "world")];
    assert!(fire(&mut e, MvEvent::Move, ctx, &mut h));
    assert_eq!(h.calls, vec!["to 5 from 4"]);
}

#[test]
fn teleport_values() {
    let mut e = engine("on teleport:\n    send \"%x of event-location%\"\n");
    let mut h = Mock::default();
    let mut ctx = EventCtx::player(steve());
    ctx.ev = loc_vals((10.0, 64.0, 0.0), (0.0, 64.0, 0.0), Some("world".into()));
    fire(&mut e, MvEvent::Teleport, ctx, &mut h);
    assert_eq!(h.calls, vec!["10"]);
}

#[test]
fn world_change_values() {
    let mut e = engine("on world change:\n    send \"%event-world% <- %past event-world%\"\n");
    let mut h = Mock::default();
    let mut ctx = EventCtx::player(steve());
    ctx.ev = world_vals("nether".into(), "world".into());
    assert!(!fire(&mut e, MvEvent::WorldChange, ctx, &mut h));
    assert_eq!(h.calls, vec!["nether <- world"]);
}

#[test]
fn swap_hands_cancel() {
    let mut e = engine("on swap hand items:\n    cancel event\n");
    let mut h = Mock::default();
    assert!(fire(
        &mut e,
        MvEvent::SwapHands,
        EventCtx::player(steve()),
        &mut h
    ));
}

#[test]
fn non_cancellable_events_report_it() {
    assert!(!errors_of("on first join:\n    cancel event\n").is_empty());
    assert!(!errors_of("on world change:\n    cancel event\n").is_empty());
}

#[test]
fn values_only_where_they_exist() {
    assert!(!errors_of("on move:\n    send command\n").is_empty());
    assert!(!errors_of("on join:\n    send full command\n").is_empty());
    assert!(!errors_of("on teleport:\n    send \"%event-world%\"\n").is_empty());
}

#[test]
fn bad_script_does_not_stop_the_rest() {
    let mut e = Engine::default();
    let src = "on move:\n    send nonsense_thing_here (\n\non first join:\n    send \"ok\"\n";
    let (_, errs) = e.load(&[("t.carve".into(), src.to_string())]);
    assert!(!errs.is_empty());
    assert!(e.table().has_event(EventKind::Mv(MvEvent::FirstJoin)));
}

#[test]
fn move_throttle() {
    assert!(!moved_enough((1.2, 64.0, 3.5), (1.9, 64.0, 3.9)));
    assert!(moved_enough((1.9, 64.0, 3.5), (2.1, 64.0, 3.5)));
    assert!(moved_enough((1.5, 64.0, 3.5), (1.5, 65.2, 3.5)));
    assert!(moved_enough((-0.1, 64.0, 0.0), (0.1, 64.0, 0.0)));
}

#[test]
fn split_command_cases() {
    assert_eq!(split_command("/Home  a"), ("home".into(), "home  a".into()));
    assert_eq!(split_command("   spawn"), ("spawn".into(), "spawn".into()));
    assert_eq!(split_command(""), (String::new(), String::new()));
}

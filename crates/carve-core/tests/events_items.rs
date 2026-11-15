use carve_core::compile::check_registry;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let who = match to {
            Recipient::Player(p) => p.name.clone(),
            Recipient::Console => "console".into(),
        };
        self.calls.push(format!("send {who} {text}"));
    }
    fn broadcast(&mut self, t: &str) {
        self.calls.push(format!("broadcast {t}"));
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

fn engine(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty(), "unexpected errors: {errs:?}");
    e
}

fn load_errs(src: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    errs.iter().map(|e| e.to_string()).collect()
}

fn fire(src: &str, kind: EventKind, ctx: EventCtx) -> (Vec<String>, bool) {
    let mut e = engine(src);
    let mut h = Mock::default();
    let out = e.fire(kind, ctx, &mut h);
    let errs: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errs.is_empty(), "runtime errors: {errs:?}");
    (h.calls, out.cancelled)
}

fn item(id: &str, n: u32) -> EventExtra {
    EventExtra {
        item: Some((id.into(), n)),
        ..Default::default()
    }
}

#[test]
fn registry_still_compiles() {
    check_registry().unwrap();
}

#[test]
fn drop_event_item_and_cancel() {
    let src = "on drop:\n    if event-item is diamond:\n        cancel event\n    send \"dropped\"\n";
    let (calls, c) = fire(
        src,
        EventKind::Drop,
        EventCtx::with_extra(steve(), item("diamond", 3)),
    );
    assert!(c);
    assert_eq!(calls, ["send Steve dropped"]);
    let (_, c) = fire(
        src,
        EventKind::Drop,
        EventCtx::with_extra(steve(), item("dirt", 1)),
    );
    assert!(!c);
}

#[test]
fn event_name_aliases() {
    for (name, kind) in [
        ("on drop", EventKind::Drop),
        ("on item drop", EventKind::Drop),
        ("on pick up", EventKind::PickUp),
        ("on pickup", EventKind::PickUp),
        ("on consume", EventKind::Consume),
        ("on eat", EventKind::Consume),
        ("on drink", EventKind::Consume),
        ("on craft", EventKind::Craft),
    ] {
        let src = format!("{name}:\n    send \"hit\"\n");
        let (calls, _) = fire(&src, kind, EventCtx::with_extra(steve(), item("bread", 1)));
        assert_eq!(calls, ["send Steve hit"], "{name}");
    }
}

#[test]
fn item_as_text() {
    let src = "on pick up:\n    set {_i} to event-item\n    send \"got %{_i}%\"\n";
    let (calls, _) = fire(
        src,
        EventKind::PickUp,
        EventCtx::with_extra(steve(), item("stone", 5)),
    );
    assert_eq!(calls.len(), 1);
    assert!(calls[0].contains("stone"), "{calls:?}");
}

#[test]
fn consume_event() {
    let src = "on eat:\n    if event-item is golden_apple:\n        send \"yum\"\n";
    let (calls, _) = fire(
        src,
        EventKind::Consume,
        EventCtx::with_extra(steve(), item("golden_apple", 1)),
    );
    assert_eq!(calls, ["send Steve yum"]);
}

fn spawn_extra(t: &str) -> EventExtra {
    EventExtra {
        entity: Some(t.into()),
        reason: Some("natural".into()),
        location: Some(EvPos {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            world: None,
        }),
        ..Default::default()
    }
}

#[test]
fn mob_spawn_filter_and_values() {
    let src = "on spawn of zombie:\n    send \"%event-entity% %spawn reason%\"\n";
    let (calls, _) = fire(
        src,
        EventKind::MobSpawn,
        EventCtx::console_with_extra(spawn_extra("zombie")),
    );
    assert_eq!(calls, ["send console zombie natural"]);
    let (calls, _) = fire(
        src,
        EventKind::MobSpawn,
        EventCtx::console_with_extra(spawn_extra("cow")),
    );
    assert!(calls.is_empty());
    let src = "on spawn of a zombie villager:\n    broadcast \"zv\"\n";
    let (calls, _) = fire(
        src,
        EventKind::MobSpawn,
        EventCtx::console_with_extra(spawn_extra("zombie_villager")),
    );
    assert_eq!(calls, ["broadcast zv"]);
}

#[test]
fn mob_spawn_plain_and_cancel() {
    let src = "on mob spawn:\n    if event-entity is creeper:\n        cancel event\n";
    let (_, c) = fire(
        src,
        EventKind::MobSpawn,
        EventCtx::console_with_extra(spawn_extra("creeper")),
    );
    assert!(c);
    let (_, c) = fire(
        src,
        EventKind::MobSpawn,
        EventCtx::console_with_extra(spawn_extra("cow")),
    );
    assert!(!c);
}

#[test]
fn mob_spawn_has_no_player() {
    let errs = load_errs("on mob spawn:\n    send \"x\" to player\n");
    assert!(errs.iter().any(|e| e.contains("player")), "{errs:?}");
}

#[test]
fn projectile_hit() {
    let src =
        "on projectile hit:\n    if hit player is set:\n        send \"hit %hit player%\" to hit player\n";
    let e = EventExtra {
        hit_player: Some(steve()),
        ..Default::default()
    };
    let (calls, _) = fire(src, EventKind::ProjectileHit, EventCtx::console_with_extra(e));
    assert_eq!(calls, ["send Steve hit Steve"]);
    let (calls, _) = fire(
        src,
        EventKind::ProjectileHit,
        EventCtx::console_with_extra(EventExtra::default()),
    );
    assert!(calls.is_empty());
}

fn modes(old: &str, new: &str) -> EventExtra {
    EventExtra {
        old_mode: Some(old.into()),
        new_mode: Some(new.into()),
        ..Default::default()
    }
}

#[test]
fn gamemode_change_values_and_filter() {
    let src = "on gamemode change:\n    send \"%old gamemode% -> %new game mode%\"\n";
    let ctx = EventCtx::with_extra(steve(), modes("survival", "creative"));
    let (calls, _) = fire(src, EventKind::GamemodeChange, ctx);
    assert_eq!(calls, ["send Steve survival -> creative"]);

    let src = "on gamemode change to creative:\n    send \"c\"\n";
    let ctx = EventCtx::with_extra(steve(), modes("survival", "creative"));
    let (calls, _) = fire(src, EventKind::GamemodeChange, ctx);
    assert_eq!(calls, ["send Steve c"]);
    let ctx = EventCtx::with_extra(steve(), modes("survival", "adventure"));
    let (calls, _) = fire(src, EventKind::GamemodeChange, ctx);
    assert!(calls.is_empty());
}

#[test]
fn level_change_not_cancellable() {
    let src = "on level change:\n    send \"%new level - old level%\"\n";
    let e = EventExtra {
        old_level: Some(4),
        new_level: Some(7),
        ..Default::default()
    };
    let (calls, _) = fire(src, EventKind::LevelChange, EventCtx::with_extra(steve(), e));
    assert_eq!(calls, ["send Steve 3"]);
    let errs = load_errs("on level change:\n    cancel event\n");
    assert!(errs.iter().any(|e| e.contains("cancel")), "{errs:?}");
}

#[test]
fn craft_recipe() {
    let src = "on craft:\n    send \"%event-recipe%\"\n    if event-item is stick:\n        cancel event\n";
    let e = EventExtra {
        recipe: Some("stick".into()),
        item: Some(("stick".into(), 1)),
        ..Default::default()
    };
    let (calls, c) = fire(src, EventKind::Craft, EventCtx::with_extra(steve(), e));
    assert_eq!(calls, ["send Steve stick"]);
    assert!(c);
}

#[test]
fn event_values_only_in_their_event() {
    assert!(!load_errs("on join:\n    send \"%event-item%\"\n").is_empty());
    assert!(!load_errs("on drop:\n    send \"%new level%\"\n").is_empty());
}

#[test]
fn bad_filters_are_errors() {
    assert!(!load_errs("on drop of cow:\n    stop\n").is_empty());
    assert!(!load_errs("on gamemode change to banana:\n    stop\n").is_empty());
}

/// Pumpkin fires its consume event for every item use; Carve only passes food
/// and drinks to `on consume`.
#[test]
fn only_edible_items_count_as_consumed() {
    use carve_core::w2_conditions_2::is_edible;
    assert!(is_edible("bread") && is_edible("potion") && is_edible("milk_bucket"));
    assert!(!is_edible("stick") && !is_edible("air") && !is_edible("bow"));
}

use carve_core::compile::{EventKind, check_registry};
use carve_core::conditions_mobs_items::{Check, EffFlag, Q};
use carve_core::events_blocks_players::ExtEvent;
use carve_core::events_game::Ev;
use carve_core::events_player_server::PlayerServerEvent;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    checks: Vec<Check>,
    worn: Vec<&'static str>,
    lid_open: Option<bool>,
    /// (effect, flag) pairs that are on.
    effects: Vec<(&'static str, EffFlag)>,
    glint: bool,
    respawn: Option<(f64, f64, f64)>,
    logs: Vec<(LogLevel, String)>,
}

impl Host for Mock {
    fn send_message(&mut self, _: &Recipient, text: &str) {
        self.calls.push(text.to_string());
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
    fn execute_command(&mut self, _: &str) {}
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        name.eq_ignore_ascii_case("steve").then(steve)
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
    fn default_world(&mut self) -> Option<String> {
        Some("world".into())
    }
    fn held_component(&mut self, _: &PlayerRef, _: carve_core::skbee_nbt::ItemComp) -> Option<Vec<u8>> {
        self.glint.then(|| vec![1])
    }
    fn respawn_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64)> {
        self.respawn
    }
    fn set_respawn_location(&mut self, _: &PlayerRef, x: f64, y: f64, z: f64) {
        self.respawn = Some((x, y, z));
    }
    fn mob_item_query(&mut self, q: Q<'_>) -> Option<Value> {
        Some(match q {
            Q::Check(c, _) => Value::Bool(self.checks.contains(&c)),
            Q::Worn(_) => Value::List(self.worn.iter().map(|s| Value::Text(s.to_string())).collect()),
            Q::Effect(_, e, f) => Value::Bool(self.effects.iter().any(|(n, g)| *n == e && *g == f)),
            Q::Lid(..) => Value::Bool(self.lid_open?),
            Q::Ops => Value::List(vec![Value::Player(steve())]),
            Q::BannedPlayers => Value::List(vec![Value::Player(PlayerRef {
                uuid: "u2".into(),
                name: "Griefer".into(),
            })]),
            Q::BannedIps => Value::List(vec![Value::Text("1.2.3.4".into())]),
            Q::Beacon(_, _, _, _, primary) => {
                if primary {
                    Value::Text("speed".into())
                } else {
                    return None;
                }
            }
        })
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn load(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".to_string(), src.to_string())]);
    assert!(
        errs.is_empty(),
        "errors: {:?}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>()
    );
    e
}

fn run_with(h: &mut Mock, src: &str) -> Vec<String> {
    let mut e = load(src);
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
    assert!(
        h.logs.iter().all(|(l, _)| *l != LogLevel::Error),
        "runtime errors: {:?}",
        h.logs
    );
    std::mem::take(&mut h.calls)
}

fn check_with(h: &mut Mock, cond: &str) -> bool {
    let src = format!("on join:\n    if {cond}:\n        send \"yes\"\n    else:\n        send \"no\"\n");
    run_with(h, &src)[0] == "yes"
}

fn check(cond: &str) -> bool {
    check_with(&mut Mock::default(), cond)
}

fn errors(src: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.to_string())]);
    errs.iter().map(|e| e.to_string()).collect()
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn stackable() {
    assert!(check("diamond is stackable"));
    assert!(check("64 of stone is stackable"));
    assert!(!check("diamond_sword is stackable"));
    assert!(check("diamond sword isn't stackable"));
    assert!(check("ender_pearl is stackable"));
    assert!(!check("totem_of_undying is stackable"));
}

#[test]
fn spawnable_and_tameable() {
    assert!(check("\"zombie\" is spawnable"));
    assert!(check("\"zombie\" is spawnable in world \"world\""));
    assert!(!check("\"player\" is spawnable"));
    assert!(check("\"fishing_bobber\" is not spawnable"));
    assert!(!check("\"not_a_mob\" is spawnable"));
    assert!(check("\"wolf\" is tameable"));
    assert!(check("\"pig\" is not tameable"));
    assert!(!check("player is tameable"));
}

#[test]
fn host_checks() {
    let mut h = Mock {
        checks: vec![Check::Saddled, Check::Tamed],
        ..Mock::default()
    };
    assert!(check_with(&mut h, "player is saddled"));
    assert!(check_with(&mut h, "player is properly saddled"));
    assert!(check_with(&mut h, "player is tamed"));
    assert!(!check_with(&mut h, "player isn't tamed"));
    assert!(!check_with(&mut h, "player is riptiding"));
    assert!(check_with(&mut h, "player is not riptiding"));
    assert!(!check_with(&mut h, "player is screaming"));
    assert!(check_with(&mut h, "player aren't screaming"));
    assert!(!check_with(&mut h, "player has a drop shadow"));
    assert!(check_with(&mut h, "text of player doesn't have text shadow"));
    assert!(check_with(&mut h, "player is not visible through blocks"));
    h.checks.push(Check::SeeThrough);
    assert!(check_with(&mut h, "text of player is visible through walls"));
}

#[test]
fn unanswered_is_false_both_ways() {
    // A host that cannot answer (no lid at that block): neither branch runs.
    let mut h = Mock::default();
    let src = "on join:\n    if lid of location(0, 0, 0) is open:\n        send \"a\"\n    if lid of location(0, 0, 0) is not open:\n        send \"b\"\n    if player is not tamed:\n        send \"c\"\n";
    let mut e = load(src);
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.calls, vec!["c".to_string()]);
}

#[test]
fn wearing() {
    let mut h = Mock {
        worn: vec!["diamond_helmet", "iron_boots"],
        ..Mock::default()
    };
    assert!(check_with(&mut h, "player is wearing diamond helmet"));
    assert!(check_with(&mut h, "player is wearing iron_boots"));
    assert!(!check_with(&mut h, "player is wearing diamond chestplate"));
    assert!(check_with(&mut h, "player is not wearing diamond chestplate"));
}

#[test]
fn glint_override() {
    let mut h = Mock::default();
    assert!(!check_with(
        &mut h,
        "player's tool has enchantment glint overridden"
    ));
    assert!(check_with(
        &mut h,
        "player's tool doesn't have enchantment glint override"
    ));
    h.glint = true;
    assert!(check_with(
        &mut h,
        "player's held item has an enchantment glint override"
    ));
    assert!(check_with(
        &mut h,
        "tool of player has overridden enchantment glint"
    ));
}

#[test]
fn lids() {
    let mut h = Mock {
        lid_open: Some(true),
        ..Mock::default()
    };
    assert!(check_with(&mut h, "lid of location(1, 2, 3) is open"));
    assert!(check_with(&mut h, "the lid of location(1, 2, 3) is opened"));
    assert!(!check_with(&mut h, "lid of location(1, 2, 3) is closed"));
    assert!(check_with(&mut h, "lid of location(1, 2, 3) isn't closed"));
    h.lid_open = Some(false);
    assert!(check_with(&mut h, "location(1, 2, 3)'s lid is closed"));
    // Not a lid block: false either way.
    h.lid_open = None;
    assert!(!check_with(&mut h, "lid of location(1, 2, 3) is open"));
    assert!(!check_with(&mut h, "lid of location(1, 2, 3) isn't open"));
}

#[test]
fn effect_flags() {
    let mut h = Mock {
        effects: vec![("speed", EffFlag::Icon), ("speed", EffFlag::Particles)],
        ..Mock::default()
    };
    assert!(check_with(
        &mut h,
        "potion effect of \"speed\" on player has an icon"
    ));
    assert!(check_with(&mut h, "effect of \"speed\" of player has particles"));
    assert!(!check_with(&mut h, "effect of \"speed\" of player is ambient"));
    assert!(check_with(&mut h, "player's \"speed\" effect isn't ambient"));
    assert!(check_with(&mut h, "player's \"speed\" potion effect has icon"));
    assert!(!check_with(&mut h, "player's \"slowness\" effect has particles"));
}

#[test]
fn any_of() {
    assert!(!check("2 is any of {_l::*}"));
    let src = "on join:\n    set {_l::*} to 1, 2 and 3\n    if 2 is any of {_l::*}:\n        send \"a\"\n    if 5 is any of {_l::*}:\n        send \"b\"\n    if 5 is not any of {_l::*}:\n        send \"c\"\n    if any of {_l::*} is 3:\n        send \"d\"\n    if 1 is one of {_l::*}:\n        send \"e\"\n    send \"%any of {_l::*}%\"\n";
    let calls = run_with(&mut Mock::default(), src);
    assert_eq!(calls, vec!["a", "c", "d", "e", "1, 2 and 3"]);
}

#[test]
fn ops_and_bans() {
    let src = "on join:\n    send \"%all operators%\"\n    set {_o::*} to all ops\n    send \"%size of {_o::*}%\"\n    send \"%all of the banned players%\"\n    send \"%banned ips%\"\n    loop all banned players:\n        send \"x %loop-value%\"\n    if player is any of all ops:\n        send \"op\"\n";
    let calls = run_with(&mut Mock::default(), src);
    assert_eq!(calls, vec!["Steve", "1", "Griefer", "1.2.3.4", "x Griefer", "op"]);
}

#[test]
fn beacon_effects() {
    let src = "on join:\n    send \"%primary beacon effect of location(0, 64, 0)%\"\n    send \"%secondary effect of location(0, 64, 0)%\"\n    send \"%beacon primary effect of location(0, 64, 0)%\"\n";
    let calls = run_with(&mut Mock::default(), src);
    assert_eq!(calls, vec!["speed", "<none>", "speed"]);
}

#[test]
fn bed_location() {
    let mut h = Mock {
        respawn: Some((1.0, 2.0, 3.0)),
        ..Mock::default()
    };
    let src = "on join:\n    send \"%x of bed location of player%\"\n    set player's bed location to location(5, 6, 7)\n    send \"%y of player's bed location%\"\n";
    let calls = run_with(&mut h, src);
    assert_eq!(calls, vec!["1", "6"]);
    assert_eq!(h.respawn, Some((5.0, 6.0, 7.0)));
}

#[test]
fn anvil_values() {
    let src =
        "on anvil prepare:\n    send \"%anvil text input%|%anvil repair cost%|%anvil item repair cost%\"\n";
    let mut e = load(src);
    let mut h = Mock::default();
    let mut ctx = EventCtx::player(steve());
    ctx.ev = vec![Value::Text("Sword".into()), Value::Num(1.0)];
    e.fire(EventKind::Ext(ExtEvent::AnvilPrepare), ctx, &mut h);
    assert_eq!(h.calls, vec!["Sword|1|1"]);
    assert!(errors("on join:\n    send \"%anvil text input%\"\n")[0].contains("anvil prepare"));
}

#[test]
fn resource_pack_states() {
    let src = "on resource pack response:\n    if the resource pack was successfully loaded:\n        send \"ok\"\n    if resource pack has been successfully loaded:\n        send \"ok2\"\n    if resource pack wasn't declined:\n        send \"nd\"\n    if resource pack was declined:\n        send \"d\"\n";
    let mut e = load(src);
    let mut h = Mock::default();
    let mut ctx = EventCtx::player(steve());
    ctx.message = Some("DownloadSuccess".into());
    e.fire(
        EventKind::PlayerServer(PlayerServerEvent::ResourcePackResponse),
        ctx.clone(),
        &mut h,
    );
    assert_eq!(h.calls, vec!["ok", "ok2", "nd"]);
    h.calls.clear();
    ctx.message = Some("Declined".into());
    e.fire(
        EventKind::PlayerServer(PlayerServerEvent::ResourcePackResponse),
        ctx,
        &mut h,
    );
    assert_eq!(h.calls, vec!["d"]);
}

#[test]
fn barter_values() {
    let src = "on piglin barter:\n    set {_d::*} to barter drops\n    send \"%barter input%|%size of {_d::*}%\"\n    add 2 diamonds to barter drops\n    remove gravel from barter drops\n    send \"%bartering drops%\"\n";
    let mut e = load(src);
    let mut h = Mock::default();
    let mut ctx = EventCtx::console();
    let item = |id: &str, n: u32| Value::Item {
        id: id.into(),
        count: n,
        meta: None,
    };
    ctx.ev = vec![
        item("gold_ingot", 1),
        Value::List(vec![item("gravel", 8), item("string", 3)]),
    ];
    let out = e.fire(
        EventKind::PlayerServer(PlayerServerEvent::PiglinBarter),
        ctx,
        &mut h,
    );
    assert_eq!(h.calls, vec!["gold ingot|2", "3 string and 2 diamond"]);
    let (slot, v) = out.set.last().expect("drops written back");
    assert_eq!(*slot, carve_core::conditions_mobs_items::DROPS_SLOT);
    let Value::List(l) = v else { panic!("list") };
    assert_eq!(l.len(), 2);
    assert!(matches!(&l[1], Value::Item { id, count: 2, .. } if id == "diamond"));

    let src = "on piglin barter:\n    clear barter drops\n";
    let mut e = load(src);
    let mut ctx = EventCtx::console();
    ctx.ev = vec![item("gold_ingot", 1), Value::List(vec![item("gravel", 8)])];
    let out = e.fire(
        EventKind::PlayerServer(PlayerServerEvent::PiglinBarter),
        ctx,
        &mut h,
    );
    assert_eq!(out.set.last().map(|s| s.1.clone()), Some(Value::List(vec![])));
    assert!(!errors("on join:\n    send \"%barter drops%\"\n").is_empty());
}

#[test]
fn applied_enchantments() {
    let src = "on enchant:\n    send \"%applied enchantments%\"\n    loop applied enchantments:\n        send \"e %loop-value%\"\n";
    let mut e = load(src);
    let mut h = Mock::default();
    let mut ctx = carve_core::events_game::make_ctx(
        Ev::Enchant,
        Some(steve()),
        &[0.0, 30.0, 2.0, 3.0],
        &["diamond_sword", "sharpness"],
        &[],
    );
    if let Some(x) = ctx.ext.as_mut() {
        x.texts.push("sharpness 3".into());
        x.texts.push("unbreaking 2".into());
    }
    e.fire(EventKind::Game(Ev::Enchant), ctx, &mut h);
    assert_eq!(h.calls[1..], ["e sharpness 3", "e unbreaking 2"]);
    assert!(!errors("on join:\n    send \"%applied enchantments%\"\n").is_empty());
}

#[test]
fn ingame_script_compiles() {
    let src = include_str!("../../../tests/ingame/areas/conditions-mobs-items/conditions-mobs-items.carve");
    let errs = errors(src);
    assert!(errs.is_empty(), "{errs:#?}");
}

#[test]
fn plain_words() {
    let mut h = Mock {
        effects: vec![("speed", EffFlag::Particles)],
        ..Mock::default()
    };
    assert!(check_with(&mut h, "player's speed effect has particles"));
    assert!(check_with(
        &mut h,
        "potion effect of speed on player has particles"
    ));
    assert!(check("\"zombie\" parsed as entity type is spawnable"));
}

use carve_core::compile::{EventKind, check_registry};
use carve_core::conditions_mob_block_state::{Bk, Ck, Q};
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    /// Entity checks that answer yes.
    yes: Vec<Ck>,
    /// Block checks that answer yes.
    blocks: Vec<Bk>,
    hands: (&'static str, &'static str),
    vehicle: Option<&'static str>,
    plugins: Vec<&'static str>,
    features: Vec<&'static str>,
    keys: u8,
    version: Option<&'static str>,
    block_id: &'static str,
    inside_border: bool,
    spawn: &'static str,
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
    fn player_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((20.0, 64.0, 5.0, "world".into()))
    }
    fn player_state(
        &mut self,
        _: &PlayerRef,
        state: carve_core::conditions_entity_block::PlayerState,
    ) -> bool {
        use carve_core::conditions_entity_block::PlayerState;
        match state {
            PlayerState::Silent => self.yes.contains(&Ck::Silent),
            PlayerState::Riding => self.yes.contains(&Ck::Riding),
            PlayerState::Poisoned => false,
        }
    }
    fn is_banned(&mut self, name: &str) -> bool {
        name == "Griefer"
    }
    fn mob_block_check(&mut self, q: Q<'_>) -> Option<Value> {
        Some(match q {
            Q::Check(c, _) => Value::Bool(self.yes.contains(&c)),
            Q::Block(k, ..) => Value::Bool(self.blocks.contains(&k)),
            Q::Plugin(n) => Value::Bool(self.plugins.contains(&n)),
            Q::Feature(n) => Value::Bool(self.features.contains(&n)),
            Q::Key(_, bit) => Value::Bool(self.keys & bit != 0),
            Q::Version => Value::Text(self.version?.into()),
            Q::Hands(_) => Value::List(vec![
                Value::Text(self.hands.0.into()),
                Value::Text(self.hands.1.into()),
            ]),
            Q::Vehicle(_) => match self.vehicle {
                Some(v) => Value::Text(v.into()),
                None => Value::None,
            },
            Q::InfiniteEffect(..) => Value::Bool(true),
            Q::Border(..) => Value::Bool(self.inside_border),
            Q::BlockId(..) => Value::Text(self.block_id.into()),
            Q::SpawnKind(_) => Value::Text(self.spawn.into()),
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

#[test]
fn registry_ok() {
    check_registry().unwrap();
    carve_core::conditions_mob_block_state::check_patterns().unwrap();
}

#[test]
fn every_pattern_compiles() {
    // A pattern that fails to compile is silently dropped from the table, so
    // count them: the table must hold every source.
    let n = carve_core::conditions_mob_block_state::pattern_sources().len();
    assert!(n > 70, "{n}");
}

#[test]
fn age() {
    let mut h = Mock::default();
    // The game says: not a baby.
    assert!(check_with(&mut h, "player is an adult"));
    assert!(check_with(&mut h, "player is not a baby"));
    assert!(!check_with(&mut h, "player is a baby"));
    h.yes.push(Ck::Baby);
    assert!(check_with(&mut h, "player is a baby"));
    assert!(check_with(&mut h, "player is not an adult"));
    assert!(!check_with(&mut h, "player is an adult"));
}

#[test]
fn entity_flags() {
    let mut h = Mock {
        yes: vec![
            Ck::Charged,
            Ck::Climbing,
            Ck::Pathfinding,
            Ck::Silent,
            Ck::Spawner,
        ],
        ..Mock::default()
    };
    for c in [
        "charged",
        "climbing",
        "pathfinding",
        "silent",
        "from a spawner",
        "from a mob spawner",
        "spawned by a trial spawner",
    ] {
        assert!(check_with(&mut h, &format!("player is {c}")), "{c}");
        assert!(!check_with(&mut h, &format!("player is not {c}")), "not {c}");
    }
    assert!(!check_with(&mut h, "player is igniting"));
    assert!(check_with(&mut h, "player isn't in the ignition process"));
    assert!(!check_with(&mut h, "player is riding"));
    assert!(check_with(&mut h, "player is not riding"));
}

#[test]
fn handedness() {
    let mut h = Mock::default();
    assert!(check_with(&mut h, "player is right handed"));
    assert!(check_with(&mut h, "player is right-handed"));
    assert!(!check_with(&mut h, "player is left handed"));
    h.yes.push(Ck::LeftHanded);
    assert!(check_with(&mut h, "player is left handed"));
    assert!(check_with(&mut h, "player is not right handed"));
}

#[test]
fn blocks() {
    let mut h = Mock {
        blocks: vec![Bk::Conditional, Bk::Occluding, Bk::Powered],
        ..Mock::default()
    };
    let at = "location(1, 2, 3)";
    assert!(check_with(&mut h, &format!("{at} is conditional")));
    assert!(check_with(&mut h, &format!("{at} is occluding")));
    assert!(check_with(&mut h, &format!("{at} is powered")));
    assert!(check_with(&mut h, &format!("{at} is redstone powered")));
    assert!(check_with(&mut h, &format!("{at} is indirectly powered")));
    assert!(!check_with(&mut h, &format!("{at} is persistent")));
    assert!(check_with(&mut h, &format!("{at} is not persistent")));
    assert!(!check_with(&mut h, &format!("{at} is lootable")));
}

#[test]
fn placed_block_traits() {
    let mut h = Mock {
        blocks: vec![Bk::Solid, Bk::Passable],
        ..Mock::default()
    };
    let at = "location(1, 2, 3)";
    assert!(check_with(&mut h, &format!("{at} is solid")));
    assert!(check_with(&mut h, &format!("{at} is passable")));
    assert!(check_with(&mut h, &format!("{at} is not transparent")));
}

#[test]
fn holding() {
    let mut h = Mock {
        hands: ("diamond_sword", "shield"),
        ..Mock::default()
    };
    assert!(check_with(&mut h, "player is holding diamond sword"));
    assert!(check_with(&mut h, "player is holding shield"));
    assert!(check_with(&mut h, "player is holding diamond sword in main hand"));
    assert!(!check_with(&mut h, "player is holding diamond sword in off hand"));
    assert!(check_with(&mut h, "player is holding shield in the off hand"));
    assert!(check_with(&mut h, "player is not holding stick"));
    h.hands = ("", "");
    assert!(check_with(&mut h, "player is not holding diamond sword"));
}

#[test]
fn riding_a_type() {
    let mut h = Mock {
        vehicle: Some("pig"),
        yes: vec![Ck::Riding],
        ..Mock::default()
    };
    assert!(check_with(&mut h, "player is riding a pig"));
    assert!(!check_with(&mut h, "player is riding a horse"));
    assert!(check_with(&mut h, "player is riding"));
    h.vehicle = None;
    assert!(!check_with(&mut h, "player is riding a pig"));
}

#[test]
fn plugins_and_features() {
    let mut h = Mock {
        plugins: vec!["Vault"],
        features: vec!["bundle"],
        ..Mock::default()
    };
    assert!(check_with(&mut h, "plugin \"Vault\" is enabled"));
    assert!(check_with(&mut h, "plugin \"Nope\" is not enabled"));
    assert!(!check_with(&mut h, "plugin \"Nope\" is enabled"));
    assert!(check_with(&mut h, "feature \"bundle\" is enabled"));
    assert!(check_with(
        &mut h,
        "experimental feature \"trade_rebalance\" is not enabled"
    ));
    assert!(check_with(
        &mut h,
        "player's world is using the feature \"bundle\""
    ));
}

#[test]
fn pressed_keys() {
    let mut h = Mock {
        keys: 16 | 32,
        ..Mock::default()
    };
    assert!(check_with(&mut h, "player is pressing jump key"));
    assert!(check_with(&mut h, "player is pressing the sneak key"));
    assert!(!check_with(&mut h, "player is pressing forward"));
    assert!(check_with(&mut h, "player is not pressing the sprint key"));
    assert!(carve_core::conditions_mob_block_state::keys_used());
}

#[test]
fn minecraft_version() {
    let mut h = Mock {
        version: Some("26.3"),
        ..Mock::default()
    };
    assert!(check_with(&mut h, "running minecraft \"26.3\""));
    assert!(check_with(&mut h, "running minecraft \"26\""));
    assert!(!check_with(&mut h, "running minecraft \"26.4\""));
    assert!(check_with(&mut h, "running above minecraft \"1.21.4\""));
    assert!(check_with(&mut h, "running below minecraft \"27\""));
    assert!(check_with(&mut h, "not running minecraft \"1.8\""));
    assert!(!check_with(&mut h, "running above mc \"26.3\""));
    // A host that does not know its version: false both ways.
    let mut none = Mock::default();
    assert!(!check_with(&mut none, "running minecraft \"26.3\""));
    assert!(!check_with(&mut none, "not running minecraft \"26.3\""));
}

#[test]
fn version_compare() {
    use carve_core::conditions_mob_block_state::{version_cmp, version_parts};
    use std::cmp::Ordering::*;
    assert_eq!(version_parts("1.20.4"), vec![1, 20, 4]);
    assert_eq!(version_parts("v26.3-rc1"), vec![26, 3]);
    assert_eq!(version_cmp("1.20", "1.20.0"), Equal);
    assert_eq!(version_cmp("1.21.4", "1.9"), Greater);
    assert_eq!(version_cmp("26.3", "26.10"), Less);
}

#[test]
fn normalized_vectors() {
    assert!(check("vector(1, 0, 0) is normalized"));
    assert!(check("vector(0, 0.6, 0.8) is normalized"));
    assert!(!check("vector(1, 1, 0) is normalized"));
    assert!(check("vector(2, 0, 0) is not normalized"));
    assert!(carve_core::conditions_mob_block_state::is_normalized(
        0.0, -1.0, 0.0
    ));
}

#[test]
fn infinite_potions() {
    let mut h = Mock::default();
    // The mock host says an active effect has duration 0, which counts as infinite.
    assert!(check_with(&mut h, "player's speed effect is infinite"));
    assert!(!check_with(
        &mut h,
        "player's speed potion effect is not infinite"
    ));
    let src = "on join:
    set {_b} to potion effect of speed for 10 seconds
    if {_b} is not infinite:
        send \"b\"
    if {_b} is infinite:
        send \"c\"
";
    assert_eq!(run_with(&mut h, src), vec!["b".to_string()]);
}

#[test]
fn banned() {
    let mut h = Mock::default();
    assert!(check_with(&mut h, "\"Griefer\" is banned"));
    assert!(check_with(&mut h, "\"Steve\" is not banned"));
}

#[test]
fn chunks_and_borders() {
    let mut h = Mock {
        inside_border: true,
        ..Mock::default()
    };
    // The mock player stands at x 20, z 5: chunk (1, 0).
    assert!(check_with(&mut h, "player is within chunk at 1, 0"));
    assert!(!check_with(&mut h, "player is within chunk at 0, 0"));
    assert!(check_with(&mut h, "player is not within chunk at 0, 0"));
    assert!(check_with(
        &mut h,
        "player is within world border of world \"world\""
    ));
    h.inside_border = false;
    assert!(!check_with(
        &mut h,
        "player is within world border of world \"world\""
    ));
    assert!(carve_core::conditions_mob_block_state::in_chunk(
        -0.5, 17.0, -1, 1
    ));
}

#[test]
fn of_type() {
    let mut h = Mock {
        block_id: "stone",
        ..Mock::default()
    };
    assert!(check_with(&mut h, "location(1, 2, 3) is of type stone"));
    assert!(!check_with(&mut h, "location(1, 2, 3) is of type dirt"));
    assert!(check_with(&mut h, "location(1, 2, 3) is not of type dirt"));
}

#[test]
fn data_tables() {
    use carve_core::data::item_tables::{FIRE_RESISTANT, FLAMMABLE_BLOCKS, FOOD, FUEL};
    for t in [FOOD, FUEL, FIRE_RESISTANT, FLAMMABLE_BLOCKS] {
        assert!(t.windows(2).all(|w| w[0] < w[1]), "tables must stay sorted");
    }
    assert!(check("bread is edible") && check("honey bottle is edible") && check("potion is edible"));
    assert!(check("stick is not edible") && check("diamond sword is not edible"));
    assert!(check("coal is fuel") && check("oak planks is fuel") && check("lava bucket is fuel"));
    assert!(check("diamond is not fuel"));
    assert!(check("netherite ingot is fire resistant") && check("iron ingot is not fire resistant"));
    assert!(check("oak planks is flammable") && check("tnt is flammable"));
    assert!(check("stone is not flammable"));
}

#[test]
fn respawn_point_kind() {
    let mut h = Mock {
        spawn: "bed",
        ..Mock::default()
    };
    assert!(check_with(&mut h, "the respawn location is a bed spawn"));
    assert!(check_with(
        &mut h,
        "the respawn location is not a respawn anchor spawn"
    ));
    assert!(!check_with(
        &mut h,
        "the respawn location is a respawn anchor spawn"
    ));
    h.spawn = "anchor";
    assert!(check_with(
        &mut h,
        "the respawn location was a respawn anchor spawn"
    ));
    h.spawn = "none";
    assert!(check_with(&mut h, "the respawn location isn't a bed spawn"));
}

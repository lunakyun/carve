use carve_core::block_ops::BlockPos;
use carve_core::compile::{EventKind, compile_file};
use carve_core::w5_chunk6::{
    BlockState, Get, Set, TextField, Warn, decode_state, egg_entity, egg_for, encode_state, format_data,
    parse_data, pickles_of, stored_decode, stored_encode, temperature,
};
use carve_core::world_effects::FxPos;
use carve_core::*;

/// First state id of sea_pickle in 26.3.
const PICKLE: u16 = 18624;

#[derive(Default)]
struct Mock {
    sent: Vec<String>,
    errors: Vec<String>,
    scale: f64,
    display_scale: Option<(f64, f64, f64)>,
    target: Option<EntityRef>,
    camera: Option<EntityRef>,
    align: String,
    width: f64,
    opacity: f64,
    warn_dist: f64,
    warn_time: f64,
    /// (id, state, first) of every block the script reads.
    block: Option<BlockState>,
    set_states: Vec<u16>,
    set_blocks: Vec<String>,
    biome: String,
    bar_viewers: Vec<String>,
    shown: Vec<String>,
    opened: u32,
    closed: u32,
    spawned: Vec<(String, f64, f64, f64)>,
    velocity: Option<(f64, f64, f64)>,
    display: bool,
}

fn zombie() -> EntityRef {
    EntityRef {
        uuid: "z1".into(),
        kind: "zombie".into(),
    }
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.sent.push(text.to_string());
    }
    fn broadcast(&mut self, _text: &str) {}
    fn send_title(&mut self, _p: &PlayerRef, _t: &str, _s: Option<&str>) {}
    fn send_action_bar(&mut self, _p: &PlayerRef, _t: &str) {}
    fn heal(&mut self, _p: &PlayerRef) {}
    fn set_health(&mut self, _p: &PlayerRef, _h: f64) {}
    fn kill(&mut self, _p: &PlayerRef) {}
    fn teleport(&mut self, _p: &PlayerRef, _x: f64, _y: f64, _z: f64) {}
    fn execute_command(&mut self, _c: &str) {}
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
    }
    fn find_player(&mut self, n: &str) -> Option<PlayerRef> {
        (n == "Steve").then(|| PlayerRef {
            uuid: "u1".into(),
            name: "Steve".into(),
        })
    }
    fn has_permission(&mut self, _p: &PlayerRef, _n: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _t: u64, _k: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn player_location(&mut self, _p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((10.0, 64.0, 10.0, "world".into()))
    }
    fn eye_location(&mut self, _p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((10.0, 65.62, 10.0, "world".into()))
    }
    fn world_names(&mut self) -> Vec<String> {
        vec!["world".into(), "nether".into()]
    }
    fn biome_at(&mut self, _w: &str, _x: i32, _y: i32, _z: i32) -> Option<String> {
        Some(self.biome.clone())
    }
    fn set_block(&mut self, _at: &BlockPos, id: &str) {
        self.set_blocks.push(id.to_string());
        if id == "oak_log" {
            self.block = Some(("oak_log".into(), 140, 139));
        }
    }
    fn spawn_entity(&mut self, kind: &str, at: &FxPos) -> Option<EntityRef> {
        self.spawned.push((kind.into(), at.x, at.y, at.z));
        Some(EntityRef {
            uuid: "a1".into(),
            kind: kind.into(),
        })
    }
    fn w2e7_query(&mut self, q: carve_core::w2_expressions_7::Query<'_>) -> Option<Value> {
        use carve_core::w2_expressions_7::Query;
        match q {
            Query::Yaw(_) | Query::Pitch(_) => Some(Value::Num(0.0)),
            _ => None,
        }
    }
    fn spawn_display(&mut self, _d: &carve_core::skbee_particles_display::DisplaySpec) {
        self.display = true;
    }
    fn last_display(&mut self) -> Option<EntityRef> {
        std::mem::take(&mut self.display).then(|| EntityRef {
            uuid: "d1".into(),
            kind: "text_display".into(),
        })
    }
    fn set_entity_velocity(&mut self, _e: &EntityRef, v: (f64, f64, f64)) {
        self.velocity = Some(v);
    }
    fn bossbar_show(&mut self, id: &str, p: &PlayerRef) {
        self.shown.push(format!("{id}:{}", p.name));
    }
    fn open_inventory(&mut self, _p: &PlayerRef, _inv: &carve_core::gaps::InvVal) {
        self.opened += 1;
    }
    fn close_inventory(&mut self, _p: &PlayerRef) {
        self.closed += 1;
    }
    fn c6_get(&mut self, q: Get<'_>) -> Option<Value> {
        Some(match q {
            Get::Scale(e) if e.kind == "text_display" => {
                let (x, y, z) = self.display_scale.unwrap_or((1.0, 1.0, 1.0));
                Value::Vector { x, y, z }
            }
            Get::Scale(_) => Value::Num(self.scale),
            Get::Target(_) => return self.target.clone().map(Value::Entity),
            Get::Camera(_) => return self.camera.clone().map(Value::Entity),
            Get::Text(_, TextField::Alignment) => Value::Text(self.align.clone()),
            Get::Text(_, TextField::LineWidth) => Value::Num(self.width),
            Get::Text(_, TextField::Opacity) => Value::Num(self.opacity),
            Get::Warning(_, Warn::Distance) => Value::Num(self.warn_dist),
            Get::Warning(_, Warn::Time) => Value::Num(self.warn_time),
            Get::BarTitle(id) => Value::Text(format!("title of {id}")),
            Get::BarStyle(_) => Value::Text("segmented_10".into()),
            Get::BarViewers(_) => Value::List(
                self.bar_viewers
                    .iter()
                    .map(|n| {
                        Value::Player(PlayerRef {
                            uuid: "u1".into(),
                            name: n.clone(),
                        })
                    })
                    .collect(),
            ),
            Get::MenuViewers(_) => Value::List(Vec::new()),
        })
    }
    fn c6_set(&mut self, s: Set<'_>) -> bool {
        match s {
            Set::Scale(_, Some(Value::Num(n))) => self.scale = *n,
            Set::Scale(_, Some(Value::Vector { x, y, z })) => self.display_scale = Some((*x, *y, *z)),
            Set::Scale(_, None) => self.scale = 1.0,
            Set::Target(_, t) => self.target = t.cloned(),
            Set::Camera(_, t) => self.camera = t.cloned(),
            Set::Text(_, TextField::Alignment, v) => self.align = v.to_string(),
            Set::Text(_, TextField::LineWidth, Value::Num(n)) => self.width = *n,
            Set::Text(_, TextField::Opacity, Value::Num(n)) => self.opacity = *n,
            Set::Warning(_, Warn::Distance, n) => self.warn_dist = n,
            Set::Warning(_, Warn::Time, n) => self.warn_time = n,
            Set::BlockState(_, s) => {
                self.set_states.push(s);
                if let Some(b) = &mut self.block {
                    b.1 = s;
                }
            }
            _ => return false,
        }
        true
    }
    fn c6_block_state(&mut self, _at: &BlockPos) -> Option<BlockState> {
        self.block.clone()
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

fn script_for(body: &str) -> String {
    format!(
        "on join:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    )
}

fn run_with(h: &mut Mock, body: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), script_for(body))]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
    assert!(h.errors.is_empty(), "{:?}", h.errors);
    std::mem::take(&mut h.sent)
}

fn run(body: &str) -> Vec<String> {
    run_with(&mut Mock::default(), body)
}

fn compile_err(src: &str) -> String {
    match compile_file("t.carve", src) {
        Ok(_) => String::new(),
        Err(e) => e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n"),
    }
}

fn compiles(body: &str) -> bool {
    compile_file("t.carve", &script_for(body)).is_ok()
}

// ---- block data ------------------------------------------------------------------

#[test]
fn sea_pickle_states_decode_in_vanilla_order() {
    assert_eq!(
        decode_state("sea_pickle", PICKLE, PICKLE),
        vec![("pickles", "1"), ("waterlogged", "true")]
    );
    assert_eq!(
        decode_state("sea_pickle", PICKLE + 1, PICKLE),
        vec![("pickles", "1"), ("waterlogged", "false")]
    );
    assert_eq!(pickles_of(PICKLE + 2, PICKLE), Some(2));
    assert_eq!(pickles_of(PICKLE + 7, PICKLE), Some(4));
}

#[test]
fn encode_keeps_the_other_properties() {
    // 2 pickles, dry -> 4 pickles, still dry.
    let s = encode_state(
        "sea_pickle",
        PICKLE,
        PICKLE + 3,
        &[("pickles".into(), "4".into())],
    )
    .unwrap();
    assert_eq!(s, PICKLE + 7);
    let e = encode_state("sea_pickle", PICKLE, PICKLE, &[("pickles".into(), "9".into())]).unwrap_err();
    assert!(e.contains("one of 1, 2, 3, 4"), "{e}");
    let e = encode_state("stone", 1, 1, &[("axis".into(), "x".into())]).unwrap_err();
    assert!(e.contains("no block data"), "{e}");
}

#[test]
fn block_data_text_round_trips() {
    // oak_log: axis x, y, z from 139.
    assert_eq!(format_data("oak_log", 139, 139), "oak_log[axis=x]");
    assert_eq!(format_data("stone", 1, 1), "stone");
    let (id, props) = parse_data("minecraft:oak_log[axis=z]").unwrap();
    assert_eq!(id, "oak_log");
    assert_eq!(encode_state(&id, 139, 140, &props).unwrap(), 141);
    assert!(parse_data("oak_log[axis=z").is_err());
    assert!(parse_data("oak_log[axis]").is_err());
}

#[test]
fn sea_pickles_read_and_change() {
    let mut h = Mock {
        block: Some(("sea_pickle".into(), PICKLE + 2, PICKLE)),
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "set {_b} to location(1, 64, 1)\nsend \"%sea pickles of {_b}%/%min pickles of {_b}%/%maximum sea pickle count of {_b}%\"\nadd 1 to pickle amount of {_b}\nsend \"%{_b}'s pickles%\"\nadd 5 to sea pickles of {_b}\nremove 9 from sea pickles of {_b}",
    );
    assert_eq!(out, vec!["2/1/4", "3"]);
    // 3 pickles waterlogged, then clamped to 4, then to 1.
    assert_eq!(h.set_states, vec![PICKLE + 4, PICKLE + 6, PICKLE]);
}

#[test]
fn sea_pickles_of_other_blocks_is_none() {
    let mut h = Mock {
        block: Some(("stone".into(), 1, 1)),
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "send \"%sea pickles of location(1, 2, 3)%\"\nset sea pickles of location(1, 2, 3) to 2",
    );
    assert_eq!(out, vec!["<none>"]);
    assert!(h.set_states.is_empty());
}

#[test]
fn block_data_read_and_set() {
    let mut h = Mock {
        block: Some(("sea_pickle".into(), PICKLE + 5, PICKLE)),
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "send \"%block data of location(0, 64, 0)%\"\nset block data of location(0, 64, 0) to \"sea_pickle[waterlogged=true]\"\nset location(0, 64, 0)'s block data to \"oak_log[axis=z]\"",
    );
    assert_eq!(out, vec!["sea_pickle[pickles=3,waterlogged=false]"]);
    assert_eq!(h.set_states, vec![PICKLE + 4, 141]);
    assert_eq!(h.set_blocks, vec!["oak_log"]);
}

// ---- entities ------------------------------------------------------------------

#[test]
fn scale_of_living_and_display() {
    let mut h = Mock {
        scale: 1.0,
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "send \"%scale of player%\"\nset scale of player to 2.5\nsend \"%player's scale%\"\nadd 100 to scale of player\nsend \"%scale of player%\"\nreset scale of player\nsend \"%scale of player%\"",
    );
    assert_eq!(out, vec!["1", "2.5", "16", "1"]);
    let mut h = Mock::default();
    let out = run_with(
        &mut h,
        "spawn text display at location(0, 64, 0)\nset scale of last spawned entity to vector(2, 3, 4)\nsend \"%scale of last spawned entity%\"",
    );
    assert_eq!(h.display_scale, Some((2.0, 3.0, 4.0)));
    assert_eq!(out, vec!["x: 2, y: 3, z: 4"]);
}

#[test]
fn target_get_set_reset() {
    let mut h = Mock::default();
    let out = run_with(
        &mut h,
        "spawn a zombie at location(0, 64, 0)\nset target of last spawned entity to player\nsend \"%target of last spawned entity%\"\ndelete target of last spawned entity\nsend \"%target of last spawned entity%\"",
    );
    assert_eq!(out[1], "<none>");
    assert!(h.target.is_none());
    h.target = Some(zombie());
    let out = run_with(
        &mut h,
        "send \"%type of target%\"\nsend \"%type of player's targeted entity%\"",
    );
    assert_eq!(out, vec!["zombie", "zombie"]);
}

#[test]
fn spectator_target() {
    let mut h = Mock::default();
    let out = run_with(
        &mut h,
        "spawn a pig at location(0, 64, 0)\nset spectator target of player to last spawned entity\nsend \"%type of spectator target%\"\nreset spectator target of player",
    );
    assert_eq!(out, vec!["pig"]);
    assert!(h.camera.is_none());
}

#[test]
fn text_display_fields() {
    let mut h = Mock {
        align: "center".into(),
        width: 200.0,
        opacity: 255.0,
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "spawn text display at location(0, 64, 0)\nset {_d} to last spawned entity\nset text alignment of {_d} to left\nset line width of {_d} to 80\nremove 55 from display opacity of {_d}\nsend \"%text alignment of {_d}%/%{_d}'s line width%/%text opacity of {_d}%\"\nif text alignment of {_d} is left:\n    send \"left\"\nreset line width of {_d}\nset opacity of {_d} to 999",
    );
    assert_eq!(out, vec!["left/80/200", "left"]);
    assert_eq!(h.width, 200.0);
    assert_eq!(h.opacity, 255.0);
}

#[test]
fn bad_text_alignment_is_a_runtime_error() {
    let mut h = Mock::default();
    let mut e = Engine::default();
    let (_, errs) = e.load(&[(
        "t.carve".into(),
        script_for("set text alignment of player to \"sideways\""),
    )]);
    assert!(errs.is_empty());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(
        h.errors.iter().any(|m| m.contains("left, center or right")),
        "{:?}",
        h.errors
    );
}

// ---- world border ----------------------------------------------------------------

#[test]
fn world_border_values_and_warnings() {
    let mut h = Mock {
        warn_dist: 5.0,
        warn_time: 15.0,
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "send \"%world border%|%world border of player%|%world \"nether\"'s world border%\"\nsend \"%world border warning distance%/%world border warning time of world border of player%\"\nset world border warning distance to 12\nadd 1 minute to world border warning time of world \"world\"\nsend \"%world border warning distance%/%world border warning time%\"\nreset world border warning time\nsend \"%world border warning time%\"",
    );
    assert_eq!(
        out,
        vec![
            "world border of 'world'|world border of 'world'|world border of 'nether'",
            "5/15 seconds",
            "12/1 minute and 15 seconds",
            "15 seconds",
        ]
    );
}

#[test]
fn world_border_size_still_parses() {
    assert!(compiles(
        "set world border size to 500 over 30 seconds\nsend \"%world border size%\""
    ));
}

// ---- boss bars and inventories ------------------------------------------------------

#[test]
fn boss_bar_title_style_viewers() {
    let mut h = Mock {
        bar_viewers: vec!["Steve".into()],
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "send \"%title of bossbar \"raid\"%\"\nsend \"%style of boss bar \"raid\"%\"\nsend \"%viewers of bossbar \"raid\"%\"\nadd player to viewers of bossbar \"raid\"\nset title of bossbar \"raid\" to \"x\"",
    );
    assert_eq!(out, vec!["title of raid", "segmented 10", "Steve"]);
    assert_eq!(h.shown, vec!["raid:Steve"]);
}

#[test]
fn inventory_title_and_viewers() {
    let mut h = Mock::default();
    let out = run_with(
        &mut h,
        "set {_i} to chest inventory with 3 rows named \"Shop\"\nsend \"%title of {_i}%\"\nset title of {_i} to \"Bank\"\nsend \"%{_i}'s titles%\"\nadd player to viewers of {_i}\nset {_v::*} to viewers of {_i}\nremove player from viewers of {_i}",
    );
    assert_eq!(out[..2], ["Shop".to_string(), "Bank".to_string()]);
    assert_eq!((h.opened, h.closed), (1, 1));
}

// ---- pure expressions -----------------------------------------------------------------

#[test]
fn all_values_of_a_type() {
    let out = run(
        "loop all boss bar styles:\n    add loop-value to {_s::*}\nsend \"%size of {_s::*}%\"\nsend \"%all of the moon phases%\"\nloop all difficulties:\n    send \"d %loop-value%\"",
    );
    assert_eq!(out[0], "5");
    assert!(out[1].starts_with("full moon, waning gibbous"), "{}", out[1]);
    assert_eq!(out[2..], ["d peaceful", "d easy", "d normal", "d hard"]);
}

#[test]
fn all_players_and_zombies_still_work() {
    assert!(compiles("loop all players:\n    send \"x\"\nkill all zombies"));
}

#[test]
fn spawn_egg_entity() {
    assert_eq!(egg_entity("zombie_spawn_egg"), Some("zombie"));
    assert_eq!(egg_entity("diamond"), None);
    assert_eq!(egg_for("cow").as_deref(), Some("cow_spawn_egg"));
    assert_eq!(egg_for("nothing"), None);
    let out = run(
        "set {_e} to 2 zombie spawn eggs\nsend \"%spawn egg entity of {_e}%\"\nset spawn egg entity of {_e} to \"pig\"\nset spawn egg entity of {_e} to cow\nsend \"%{_e}%\"\nsend \"%{_e}'s spawn egg entity%\"\nsend \"%spawn egg entity of diamond%\"",
    );
    assert_eq!(out, vec!["zombie", "2 cow spawn egg", "cow", "<none>"]);
}

#[test]
fn temperature_of_blocks() {
    assert_eq!(temperature("plains", 64), Some(0.8));
    assert_eq!(temperature("minecraft:desert", 120), Some(1.95));
    assert_eq!(temperature("unknown_biome", 64), None);
    let mut h = Mock {
        biome: "snowy_taiga".into(),
        ..Mock::default()
    };
    let out = run_with(&mut h, "send \"%temperature of location(0, 64, 0)%\"");
    assert_eq!(out, vec!["-0.5"]);
}

#[test]
fn slot_index() {
    let out = run("send \"%index of slot 5 of player's inventory%\"\nsend \"%index of \"b\" in \"abc\"%\"");
    assert_eq!(out, vec!["5", "2"]);
}

#[test]
fn stored_enchantments_bytes() {
    let b = stored_encode(&Value::List(vec![
        Value::Text("sharpness 5".into()),
        Value::Text("fire aspect".into()),
    ]))
    .unwrap();
    // sharpness is id 33 and fire_aspect 10 in the alphabetical 26.3 registry.
    assert_eq!(b, vec![2, 33, 5, 10, 1]);
    match stored_decode(&b) {
        Value::List(l) => {
            let t: Vec<String> = l.iter().map(ToString::to_string).collect();
            assert_eq!(t, vec!["sharpness 5", "fire_aspect 1"]);
        }
        v => panic!("{v:?}"),
    }
    assert!(stored_encode(&Value::Text("sharpness 999".into())).is_err());
    assert!(stored_encode(&Value::Text("bogus 2".into())).is_err());
    assert!(compiles(
        "set stored enchantments of player's tool to \"mending 1\"\nloop stored enchantments of player's tool:\n    send \"%loop-value%\"\ndelete stored enchantments of player's tool"
    ));
}

// ---- event values ------------------------------------------------------------------------

#[test]
fn event_value_names() {
    assert!(compile_err("on spread:\n    send \"%source block%\" to console").is_empty());
    assert!(compile_err("on tame:\n    send \"%tamer%\" to console").is_empty());
    assert!(compile_err("on shoot:\n    send \"%shooter%\" to console").is_empty());
    assert!(compile_err("on spawn:\n    send \"%spawning reason% %spawn reason%\" to console").is_empty());
    assert!(compile_err("on target:\n    send \"%target%\" to console").is_empty());
}

#[test]
fn tamer_is_the_owner_slot() {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[(
        "t.carve".into(),
        "on tame:\n    send \"tamed by %tamer%\" to player".into(),
    )]);
    assert!(errs.is_empty(), "{errs:?}");
    let mut ctx = EventCtx::player(steve());
    ctx.args = vec![Value::Text("wolf".into()), Value::Player(steve())];
    let mut h = Mock::default();
    e.fire(
        EventKind::Ext4(carve_core::w2_events_4::ExtEvent::Tame),
        ctx,
        &mut h,
    );
    assert_eq!(h.sent, vec!["tamed by Steve"]);
}

// ---- sections ----------------------------------------------------------------------------

#[test]
fn shoot_section_spawns_and_pushes() {
    let mut h = Mock::default();
    let out = run_with(
        &mut h,
        "shoot an arrow from player at speed 2:\n    send \"%type of event-entity%\"\n    send \"%type of shot entity%\"",
    );
    assert_eq!(out, vec!["arrow", "arrow"]);
    let (kind, _, y, _) = &h.spawned[0];
    assert_eq!(kind, "arrow");
    assert!(*y > 65.0, "spawned at the eyes: {y}");
    let v = h.velocity.expect("velocity set");
    let len = (v.0 * v.0 + v.1 * v.1 + v.2 * v.2).sqrt();
    assert!((len - 2.0).abs() < 1e-6, "{v:?}");
}

#[test]
fn shoot_section_defaults() {
    let mut h = Mock::default();
    run_with(&mut h, "shoot a snowball:\n    send \"x\"");
    assert_eq!(h.spawned[0].0, "snowball");
    let v = h.velocity.expect("velocity set");
    let len = (v.0 * v.0 + v.1 * v.1 + v.2 * v.2).sqrt();
    assert!((len - 5.0).abs() < 1e-6, "{v:?}");
    assert!(!compile_err("on join:\n    shoot from player:\n        send \"x\"").is_empty());
}

#[test]
fn suppress_type_hints_runs_its_body() {
    assert_eq!(run("suppress type hints:\n    send \"in\""), vec!["in"]);
}

#[test]
fn ingame_script_compiles() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/ingame/areas/w5-chunk6/w5-chunk6.carve"
    );
    let src = std::fs::read_to_string(path).expect("area script");
    assert_eq!(compile_err(&src), "");
}

#[test]
fn new_parameter_types() {
    for t in [
        "block data",
        "potion effect",
        "executable",
        "type",
        "enchantment type",
        "entity type with amount",
    ] {
        let src = format!("function f(x: {t}):\n    broadcast \"%{{_x}}%\"\non join:\n    f(\"a\")");
        assert!(compile_err(&src).is_empty(), "{t}: {}", compile_err(&src));
    }
}

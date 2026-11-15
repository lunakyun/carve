use carve_core::block_ops::BlockPos;
use carve_core::compile::{EventKind, compile_file};
use carve_core::potions_particles_queues::{Obj, PotionVal, Q, category, max_level, redstone_power};
use carve_core::world_effects::ParticleFx;
use carve_core::*;

#[derive(Default)]
struct Mock {
    sent: Vec<String>,
    errors: Vec<String>,
    warns: Vec<String>,
    commands: Vec<String>,
    particles: Vec<ParticleFx>,
    /// (effect, ticks, amplifier, ambient, particles)
    applied: Vec<(String, u32, u8, bool, bool)>,
    effects: Vec<PotionVal>,
    bar: Option<f64>,
    weather: Option<bool>,
    weather_sets: Vec<Option<bool>>,
    /// Block id, state id, default state id.
    block: Option<(String, u16, u16)>,
    state_sets: Vec<u16>,
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
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
    fn execute_command(&mut self, c: &str) {
        self.commands.push(c.to_string());
    }
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
    }
    fn find_player(&mut self, n: &str) -> Option<PlayerRef> {
        (n == "Steve").then(steve)
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
    fn spawn_particle(&mut self, p: &ParticleFx) {
        self.particles.push(p.clone());
    }
    fn add_effect(&mut self, _p: &PlayerRef, effect: &str, ticks: u32, amplifier: u8) {
        self.applied.push((effect.into(), ticks, amplifier, false, true));
    }
    fn w5c5(&mut self, q: Q<'_>) -> Value {
        match q {
            Q::Effects(_) => Value::List(
                self.effects
                    .iter()
                    .map(|p| Value::W5(Box::new(Obj::Potion(p.clone()))))
                    .collect(),
            ),
            Q::AddEffect(_, p) => {
                self.applied
                    .push((p.effect.clone(), p.ticks, p.amplifier, p.ambient, p.particles));
                Value::Bool(true)
            }
            Q::BarProgress("boss") => self.bar.map_or(Value::None, Value::Num),
            Q::BarExists(id) => Value::Bool(id == "boss"),
            Q::Weather(_) => match self.weather {
                Some(true) => Value::Text("rain".into()),
                Some(false) => Value::Text("clear".into()),
                None => Value::None,
            },
            Q::SetWeather(_, w) => {
                self.weather_sets.push(w);
                Value::Bool(true)
            }
            Q::BlockState(_) => match &self.block {
                Some((n, s, d)) => Value::List(vec![
                    Value::Text(format!("minecraft:{n}")),
                    Value::Num(f64::from(*s)),
                    Value::Num(f64::from(*d)),
                ]),
                None => Value::None,
            },
            Q::SetBlockState(_, s) => {
                self.state_sets.push(s);
                Value::Bool(true)
            }
            Q::Viewers(7, _) => Value::List(vec![Value::Player(steve())]),
            _ => Value::None,
        }
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        match level {
            LogLevel::Error => self.errors.push(msg.to_string()),
            LogLevel::Warn => self.warns.push(msg.to_string()),
            _ => {}
        }
    }
}

fn fire(h: &mut Mock, head: &str, kind: EventKind, ctx: EventCtx, body: &str) -> Vec<String> {
    let src = format!(
        "{head}\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    );
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e.fire(kind, ctx, h);
    assert!(h.errors.is_empty(), "{:?}", h.errors);
    std::mem::take(&mut h.sent)
}

fn run_with(h: &mut Mock, body: &str) -> Vec<String> {
    fire(h, "on join:", EventKind::Join, EventCtx::player(steve()), body)
}

fn run(body: &str) -> Vec<String> {
    run_with(&mut Mock::default(), body)
}

fn compiles(body: &str) -> bool {
    let src = format!("on join:\n    {}\n", body.replace('\n', "\n    "));
    compile_file("t.carve", &src).is_ok()
}

// ---- potion effects -----------------------------------------------------------

#[test]
fn new_potion_effect_and_parts() {
    let out = run("set {_e} to potion effect of speed of tier 2 for 10 seconds\n\
         send \"%{_e}%\"\n\
         send \"%potion amplifier of {_e}%/%potion tier of {_e}%/%potion duration of {_e}%\"");
    assert_eq!(out, vec!["speed 2 for 10 seconds", "1/2/10 seconds"]);
}

#[test]
fn new_potion_effect_forms() {
    let out = run("send \"%potion effect of jump boost%\"\n\
         send \"%an ambient potion effect of regeneration 3 without particles for 2 seconds%\"\n\
         send \"%a potion effect of strength tier 4%\"\n\
         send \"%infinite potion effect of night vision%\"\n\
         set {_t} to \"haste\"\n\
         send \"%potion effect of {_t} for 1 minute%\"");
    assert_eq!(
        out,
        vec![
            "jump boost 1 for 15 seconds",
            "ambient regeneration 3 for 2 seconds without particles",
            "strength 4 for 15 seconds",
            "infinite night vision 1",
            "haste 1 for 1 minute",
        ]
    );
}

#[test]
fn unknown_potion_effect_is_a_load_error() {
    assert!(!compiles("set {_e} to potion effect of flying"));
}

#[test]
fn set_parts_of_a_potion_variable() {
    let out = run("set {_e} to potion effect of speed\n\
         set potion amplifier of {_e} to 3\n\
         set potion duration of {_e} to 4 seconds\n\
         send \"%{_e}%\"");
    assert_eq!(out, vec!["speed 4 for 4 seconds"]);
}

#[test]
fn apply_a_potion_value() {
    let mut h = Mock::default();
    run_with(
        &mut h,
        "set {_e} to ambient potion effect of speed 2 without particles for 3 seconds\napply {_e} to player",
    );
    assert_eq!(h.applied, vec![("speed".into(), 60, 1, true, false)]);
}

#[test]
fn potion_effects_of_player() {
    let mut h = Mock {
        effects: vec![
            PotionVal {
                effect: "speed".into(),
                amplifier: 1,
                ticks: 200,
                ambient: false,
                particles: true,
            },
            PotionVal {
                effect: "jump_boost".into(),
                amplifier: 0,
                ticks: 40,
                ambient: false,
                particles: true,
            },
        ],
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "send \"%potion effects of player%\"\n\
         send \"%speed potion effect of player%\"\n\
         send \"%jump boost effect of player%\"\n\
         send \"%tier of speed of player%\"\n\
         send \"%potion tier of jump boost for player%\"\n\
         send \"%tier of haste of player%\"\n\
         set {_all::*} to active potion effects of player\n\
         send \"%size of {_all::*}%\"",
    );
    assert_eq!(
        out,
        vec![
            "speed 2 for 10 seconds and jump boost 1 for 2 seconds",
            "speed 2 for 10 seconds",
            "jump boost 1 for 2 seconds",
            "2",
            "1",
            "<none>",
            "2",
        ]
    );
}

#[test]
fn potion_effect_type_category() {
    let out = run("send \"%potion category of speed%\"\n\
         send \"%potion effect type category of poison%\"\n\
         if potion effect category of glowing is neutral:\n    send \"neutral\"");
    assert_eq!(out, vec!["beneficial", "harmful", "neutral"]);
    assert_eq!(category("wither"), "harmful");
}

#[test]
fn potion_items_give_potion_contents() {
    let mut h = Mock::default();
    run_with(
        &mut h,
        "give potion of speed to player\n\
         give splash potion of poison to player\n\
         give lingering potion of haste to player\n\
         set {_e} to potion effect of strength 2 for 5 seconds\n\
         give potion item of {_e} to player",
    );
    assert_eq!(
        h.commands,
        vec![
            "give Steve minecraft:potion[potion_contents={potion:\"minecraft:swiftness\"}] 1",
            "give Steve minecraft:splash_potion[potion_contents={potion:\"minecraft:poison\"}] 1",
            "give Steve minecraft:lingering_potion[potion_contents={custom_effects:[{id:\"minecraft:haste\",amplifier:0,duration:3600}]}] 1",
            "give Steve minecraft:potion[potion_contents={custom_effects:[{id:\"minecraft:strength\",amplifier:1,duration:100,ambient:0b,show_particles:1b}]}] 1",
        ]
    );
}

// ---- skulls, raw names, enchantment levels ------------------------------------------

#[test]
fn player_skull() {
    let mut h = Mock::default();
    let out = run_with(
        &mut h,
        "give skull of player to player\nsend \"%raw name of skull of \"Notch\"%\"",
    );
    assert_eq!(
        h.commands,
        vec!["give Steve minecraft:player_head[profile=\"Steve\"] 1"]
    );
    assert_eq!(out, vec!["minecraft:player_head"]);
}

#[test]
fn raw_name() {
    let out = run("send \"%raw name of diamond sword%\"\nsend \"%minecraft name of player's tool%\"");
    assert_eq!(out[0], "minecraft:diamond_sword");
}

#[test]
fn enchantment_levels() {
    let out = run("send \"%max enchantment level of sharpness%\"\n\
         send \"%minimum enchant level of mending%\"\n\
         send \"%maximum enchantment level of sweeping edge%\"\n\
         set {_e} to \"protection\"\n\
         send \"%max enchantment level of {_e}%\"");
    assert_eq!(out, vec!["5", "1", "3", "4"]);
    assert_eq!(max_level("piercing"), Some(4));
}

// ---- particles -----------------------------------------------------------------------

#[test]
fn particle_values_and_parts() {
    let out = run(
        "set {_p} to flame particle with offset vector(1, 2, 3) with particle speed 0.5\n\
         send \"%particle count of {_p}%/%particle speed of {_p}%/%extra value of {_p}%\"\n\
         send \"%particle offset of {_p}%\"\n\
         set {_v} to soul fire flame particle with a velocity of vector(0, 1, 0)\n\
         send \"%particle count of {_v}%/%particle distribution of {_v}%\"",
    );
    assert_eq!(out, vec!["1/0.5/0.5", "x: 1, y: 2, z: 3", "0/x: 0, y: 1, z: 0"]);
}

#[test]
fn set_particle_parts_and_draw() {
    let mut h = Mock::default();
    run_with(
        &mut h,
        "set {_p} to end rod particle\n\
         set particle count of {_p} to 12\n\
         set particle distribution of {_p} to vector(0.5, 1, 0.5)\n\
         set extra value of {_p} to 0.1\n\
         draw {_p} at player\n\
         play {_p} with an extra value of 0.2 at location(1, 2, 3)",
    );
    assert_eq!(h.particles.len(), 2);
    let p = &h.particles[0];
    assert_eq!((p.name.as_str(), p.count, p.speed), ("end_rod", 12, 0.1));
    assert_eq!(p.offset, Some([0.5, 1.0, 0.5]));
    assert_eq!((p.at.x, p.at.y), (10.0, 64.0));
    assert_eq!(h.particles[1].speed, 0.2);
    assert_eq!(h.particles[1].at.z, 3.0);
}

#[test]
fn data_particles_warn_instead_of_drawing() {
    let mut h = Mock::default();
    run_with(&mut h, "draw dust particle at player");
    assert!(h.particles.is_empty());
    assert_eq!(h.warns.len(), 1);
}

// ---- queues --------------------------------------------------------------------------

#[test]
fn queues_and_their_ends() {
    let out = run("set {_q} to a new queue of 1, 2 and 3\n\
         send \"%start of {_q}%/%end of {_q}%\"\n\
         set {_e} to queue\n\
         send \"%{_e}%/%start of {_e}%\"\n\
         set {_l::*} to \"a\", \"b\"\n\
         send \"%queue with {_l::*}%\"");
    assert_eq!(out, vec!["1/3", "queue of nothing/<none>", "queue of a, b"]);
}

// ---- boss bars, weather, blocks --------------------------------------------------------

#[test]
fn boss_bar_key_and_progress() {
    let mut h = Mock {
        bar: Some(40.0),
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "send \"%bossbar key of bossbar \"boss\"%/%boss bar id of \"nope\"%\"\nsend \"%progress of bossbar \"boss\"%\"",
    );
    assert_eq!(out, vec!["boss/<none>", "40"]);
}

#[test]
fn client_weather() {
    let mut h = Mock {
        weather: Some(true),
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "send \"%client weather of player%\"\n\
         if client weather of player is \"rain\":\n    send \"wet\"\n\
         set custom weather of player to clear\n\
         set client weather of player to rain\n\
         reset client weather of player",
    );
    assert_eq!(out, vec!["rain", "wet"]);
    assert_eq!(h.weather_sets, vec![Some(false), Some(true), None]);
    // `weather of player` and `set weather of player` are still the world's weather.
    assert!(compiles(
        "send \"%weather in world \"world\"%\"
send \"%weather of player%\""
    ));
}

#[test]
fn redstone_power_of_blocks() {
    // redstone_wire power 7 with all sides none: index 1160 - 0 + 7 * 9.
    let mut h = Mock {
        block: Some(("redstone_wire".into(), 6727 + 63, 6727)),
        ..Mock::default()
    };
    let out = run_with(&mut h, "send \"%redstone power of block at location(0, 64, 0)%\"");
    assert_eq!(out, vec!["7"]);
    assert_eq!(redstone_power("redstone_block", 0), 15);
    assert_eq!(redstone_power("lever", 9), 0);
    assert_eq!(redstone_power("lever", 8), 15);
    assert_eq!(redstone_power("stone", 0), 0);
}

#[test]
fn respawn_anchor_charges() {
    let mut h = Mock {
        block: Some(("respawn_anchor".into(), 25180, 25178)),
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "set {_b} to block at location(0, 64, 0)\n\
         send \"%charges of {_b}%/%max charges of {_b}%\"\n\
         set charges of {_b} to 4",
    );
    assert_eq!(out, vec!["2/4"]);
    assert_eq!(h.state_sets, vec![25182]);
}

#[test]
fn charges_of_other_blocks_are_none() {
    let mut h = Mock {
        block: Some(("stone".into(), 1, 1)),
        ..Mock::default()
    };
    let out = run_with(&mut h, "send \"%charges of block at location(0, 64, 0)%\"");
    assert_eq!(out, vec!["<none>"]);
    let _ = BlockPos {
        x: 0,
        y: 0,
        z: 0,
        world: None,
    };
}

// ---- inventories -----------------------------------------------------------------------

#[test]
fn inventory_rows_slots_holder() {
    let out = run("set {_g} to chest inventory with 3 rows named \"Shop\"\n\
         send \"%rows of {_g}%/%amount of slots of {_g}%/%holder of {_g}%\"\n\
         send \"%viewers of {_g}%\"");
    assert_eq!(out, vec!["3/27/<none>", "<none>"]);
}

// ---- events, errors, functions -------------------------------------------------------------

#[test]
fn respawn_reason() {
    let mut h = Mock::default();
    let mut ctx = EventCtx::player(steve());
    ctx.message = Some("death".into());
    let out = fire(
        &mut h,
        "on respawn:",
        EventKind::Respawn,
        ctx,
        "send \"%respawn reason%\"\nif respawn reason is death:\n    send \"died\"",
    );
    assert_eq!(out, vec!["death", "died"]);
    assert!(!compiles("send \"%respawn reason%\""));
}

#[test]
fn on_screen_kick_message() {
    let mut h = Mock::default();
    let mut ctx = EventCtx::player(steve());
    ctx.message = Some("bye".into());
    let out = fire(
        &mut h,
        "on kick:",
        EventKind::Ext3(carve_core::events_player_server::Ev3::Kick),
        ctx,
        "send \"%on-screen kick message%\"",
    );
    assert_eq!(out, vec!["bye"]);
    assert!(!compiles("send \"%on-screen kick message%\""));
}

#[test]
fn last_caught_errors() {
    let out = run(
        "catch runtime errors:\n    set {_x} to potion effect of {_none}\nsend \"%last caught runtime errors%\"\nsend \"%last caught errors%\"",
    );
    assert_eq!(out.len(), 2);
    assert!(out[0].contains("potion effect"), "{out:?}");
    assert_eq!(out[0], out[1]);
}

#[test]
fn result_of_function() {
    let mut h = Mock::default();
    let src = "function double(n: number) :: number:\n    return {_n} * 2\n\non join:\n    send \"%result of function \"double\" with arguments 21%\"\n    send \"%result of executing function double with argument 2%\"";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty(), "{errs:?}");
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.sent, vec!["42", "4"]);
    assert!(compile_file("t.carve", "on join:\n    send \"%result of function \"nope\"%\"").is_err());
}

#[test]
fn projectile_force_in_shoot_bow() {
    assert!(
        compile_file(
            "t.carve",
            "on entity shoot bow:\n    broadcast \"%projectile force%\"\n"
        )
        .is_ok()
    );
}

#[test]
fn values_are_not_saved() {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[(
        "t.carve".into(),
        "on join:\n    set {keep} to 1\n    set {pot} to potion effect of speed\n    set {par} to flame particle\n".into(),
    )]);
    assert!(errs.is_empty());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut Mock::default());
    let json = e.vars.to_json();
    assert!(
        json.contains("keep") && !json.contains("pot") && !json.contains("par"),
        "{json}"
    );
}

#[test]
fn ingame_script_compiles() {
    let path = concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/../../tests/ingame/areas/potions-particles-queues/potions-particles-queues.carve"
    );
    let src = std::fs::read_to_string(path).expect("in-game script");
    let c = carve_core::compile::compile_all(&[("potions-particles-queues.carve".to_string(), src)]);
    assert!(c.errors.is_empty(), "{:?}", c.errors);
}

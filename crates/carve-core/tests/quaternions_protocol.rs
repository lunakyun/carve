use carve_core::compile::{EventKind, compile_file};
use carve_core::quaternions_protocol::{
    Get, Set, Vf, protocol_of, quat_angle, quat_axis, rotate_quat, rotate_vec, sign_lines, skull_profile,
    spawner_type, times_list,
};
use carve_core::skbee_nbt::{Compound, Tag};
use carve_core::*;

#[derive(Default)]
struct Mock {
    sent: Vec<String>,
    errors: Vec<String>,
    portal: u32,
    whitelist: bool,
    rotation: (f64, f64),
    border: f64,
    commands: Vec<String>,
    nbt: Option<Vec<u8>>,
    sets: Vec<String>,
}

fn pig() -> EntityRef {
    EntityRef {
        uuid: "p1".into(),
        kind: "pig".into(),
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
    fn log(&mut self, _l: LogLevel, m: &str) {
        self.errors.push(m.to_string());
    }
    fn default_world(&mut self) -> Option<String> {
        Some("world".into())
    }
    fn border_size(&mut self, _w: &str) -> Option<f64> {
        Some(self.border)
    }
    fn set_border_size(&mut self, _w: &str, size: f64, _s: f64) {
        self.border = size;
    }
    fn entity_rotation(&mut self, _e: &EntityRef) -> Option<(f64, f64)> {
        Some(self.rotation)
    }
    fn block_entity_nbt(&mut self, _w: &str, _x: i32, _y: i32, _z: i32) -> Option<Vec<u8>> {
        self.nbt.clone()
    }
    fn w6c6_get(&mut self, q: Get<'_>) -> Option<Value> {
        match q {
            Get::Portal(_) => Some(Value::Num(f64::from(self.portal))),
            Get::Protocol(_) => Some(Value::Num(777.0)),
            Get::Vehicle(_) => Some(Value::Entity(pig())),
            Get::Passengers(_) => Some(Value::List(vec![Value::Entity(pig())])),
            Get::Villager(_, Vf::Level) => Some(Value::Num(3.0)),
            Get::Villager(_, Vf::Experience) => Some(Value::Num(40.0)),
            Get::Villager(_, Vf::Profession) => Some(Value::Text("librarian".into())),
            Get::Ringing(..) => Some(Value::Num(20.0)),
            Get::Whitelist => Some(Value::Bool(self.whitelist)),
            Get::WhitelistPlayers => Some(Value::List(vec![Value::Player(PlayerRef {
                uuid: "u2".into(),
                name: "Alex".into(),
            })])),
        }
    }
    fn w6c6_set(&mut self, s: Set<'_>) -> bool {
        match s {
            Set::Portal(_, t) => self.portal = t,
            Set::Whitelist(on) => self.whitelist = on,
            Set::Rotation(_, y, p) => {
                self.sets.push(format!("rot {y:?} {p:?}"));
                if let Some(y) = y {
                    self.rotation.0 = f64::from(y);
                }
                if let Some(p) = p {
                    self.rotation.1 = f64::from(p);
                }
            }
        }
        true
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
    std::mem::take(&mut h.sent)
}

fn run(body: &str) -> Vec<String> {
    run_with(&mut Mock::default(), body)
}

fn nbt_bytes(c: &Compound) -> Vec<u8> {
    c.to_bytes().unwrap()
}

fn compound(items: Vec<(&str, Tag)>) -> Compound {
    Compound(items.into_iter().map(|(k, v)| (k.to_string(), v)).collect())
}

// ---- pure maths ---------------------------------------------------------------

#[test]
fn quaternion_angle_and_axis() {
    assert!((quat_angle(0.5) - 120.0).abs() < 1e-9);
    assert!((quat_angle(1.0)).abs() < 1e-9);
    let (x, y, z) = quat_axis(0.0, 2.0, 0.0);
    assert_eq!((x, y, z), (0.0, 1.0, 0.0));
    assert_eq!(quat_axis(0.0, 0.0, 0.0), (0.0, 0.0, 1.0));
}

#[test]
fn rotations() {
    let (x, y, z) = rotate_vec((1.0, 0.0, 0.0), (0.0, 0.0, 1.0), 90.0);
    assert!(x.abs() < 1e-9 && (y - 1.0).abs() < 1e-9 && z.abs() < 1e-9);
    let (w, x, y, z) = rotate_quat((1.0, 0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 180.0);
    assert!(w.abs() < 1e-9 && x.abs() < 1e-9 && (y - 1.0).abs() < 1e-9 && z.abs() < 1e-9);
}

#[test]
fn times_is_one_to_n() {
    let Value::List(l) = times_list(3.0) else { panic!() };
    assert_eq!(l.len(), 3);
    assert!(matches!(times_list(-2.0), Value::List(l) if l.is_empty()));
}

#[test]
fn protocol_numbers() {
    assert_eq!(protocol_of("JavaMinecraftVersion::V263"), Some(777));
    assert_eq!(protocol_of("V1_21_11"), Some(774));
    assert_eq!(protocol_of("v-1-8"), Some(47));
    assert_eq!(protocol_of("Unknown"), None);
}

// ---- block entity nbt ---------------------------------------------------------------

#[test]
fn sign_lines_read_json_strings() {
    let msgs = Tag::List(vec![
        Tag::Str("\"first\"".into()),
        Tag::Str("{\"text\":\"second\"}".into()),
    ]);
    let nbt = compound(vec![(
        "front_text",
        Tag::Compound(compound(vec![("messages", msgs)])),
    )]);
    assert_eq!(
        sign_lines(&nbt),
        Some(vec![
            "first".into(),
            "second".into(),
            String::new(),
            String::new()
        ])
    );
    assert_eq!(sign_lines(&Compound::default()), None);
}

#[test]
fn skull_profile_and_spawner() {
    let prop = compound(vec![
        ("name", Tag::Str("textures".into())),
        ("value", Tag::Str("abc".into())),
    ]);
    let profile = compound(vec![
        ("name", Tag::Str("Notch".into())),
        ("properties", Tag::List(vec![Tag::Compound(prop)])),
    ]);
    let nbt = compound(vec![("profile", Tag::Compound(profile))]);
    assert_eq!(skull_profile(&nbt), (Some("Notch".into()), Some("abc".into())));
    assert_eq!(skull_profile(&Compound::default()), (None, None));
    let id = compound(vec![("id", Tag::Str("minecraft:zombie".into()))]);
    let entry = compound(vec![("entity", Tag::Compound(id))]);
    let nbt = compound(vec![("SpawnData", Tag::Compound(entry))]);
    assert_eq!(spawner_type(&nbt), Some("zombie".into()));
}

// ---- scripts -----------------------------------------------------------------------

#[test]
fn quaternion_reads_in_scripts() {
    let out = run(
        "set {_q} to quaternion(0.5, 0.5, 0.5, 0.5)\nsend \"%w of {_q}% %x of {_q}% %rotation angle of {_q}%\" to player",
    );
    assert_eq!(out, vec!["0.5 0.5 120"]);
}

#[test]
fn vector_components_change_in_variables() {
    let out = run(
        "set {_v} to vector(1, 2, 3)\nset x of {_v} to 9\nadd 1 to y of {_v}\nremove 1 from z of {_v}\nsend \"%x of {_v}% %y of {_v}% %z of {_v}%\" to player",
    );
    assert_eq!(out, vec!["9 3 2"]);
}

#[test]
fn yaw_and_pitch_of_a_vector_variable() {
    let out = run("set {_v} to vector(0, 0, 1)\nset yaw of {_v} to 90\nsend \"%x of {_v}%\" to player");
    assert_eq!(out, vec!["-1"]);
}

#[test]
fn n_times_list_and_plain_item() {
    let out = run("set {_l::*} to 4 times\nsend \"%size of {_l::*}% %{_l::4}%\" to player");
    assert_eq!(out, vec!["4 4"]);
    let out = run("set {_i} to diamond sword\nset {_p} to plain {_i}\nsend \"%{_p}%\" to player");
    assert_eq!(out, vec!["diamond sword"]);
}

#[test]
fn rotated_vector() {
    let out = run("set {_r} to vector(1, 0, 0) rotated around z by 90\nsend \"%y of {_r}%\" to player");
    assert_eq!(out, vec!["1"]);
}

#[test]
fn portal_cooldown_round_trip() {
    let mut h = Mock::default();
    let out = run_with(
        &mut h,
        "set portal cooldown of player to 7 seconds\nsend \"%portal cooldown of player%\" to player",
    );
    assert_eq!(h.portal, 140);
    assert_eq!(out, vec!["7 seconds"]);
}

#[test]
fn protocol_version_of_player_and_server() {
    assert_eq!(
        run("send \"%protocol version of player% %protocol version%\" to player"),
        vec!["777 777"]
    );
}

#[test]
fn whitelist_switch_and_list() {
    let mut h = Mock::default();
    let out = run_with(
        &mut h,
        "set whitelist to true\nset {_w::*} to whitelisted players\nsend \"%whitelist enabled% %size of {_w::*}%\" to player",
    );
    assert!(h.whitelist);
    assert_eq!(out, vec!["true 1"]);
}

#[test]
fn bell_ringing_time_reads_ticks() {
    let out = run("send \"%ringing time of location(1, 2, 3)%\" to player");
    assert_eq!(out, vec!["1 second"]);
}

#[test]
fn rotation_of_entities_goes_through_the_host() {
    let mut h = Mock {
        rotation: (10.0, 5.0),
        ..Mock::default()
    };
    run_with(&mut h, "add 20 to yaw of player\nset pitch of player to 100");
    assert_eq!(h.sets, vec!["rot Some(30.0) None", "rot None Some(90.0)"]);
}

#[test]
fn world_border_add_and_remove() {
    let mut h = Mock {
        border: 100.0,
        ..Mock::default()
    };
    run_with(
        &mut h,
        "add 10 to world border size\nremove 30 from world border size",
    );
    assert_eq!(h.border, 80.0);
}

#[test]
fn spawn_is_set_with_the_console_command() {
    let mut h = Mock::default();
    run_with(&mut h, "set spawn of world \"world\" to location(7.5, 100, -3.5)");
    assert_eq!(h.commands, vec!["setworldspawn 7 100 -4"]);
}

#[test]
fn nbt_reads_on_an_empty_block() {
    let mut h = Mock {
        nbt: Some(nbt_bytes(&Compound::default())),
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "send \"%loot table seed of location(1, 2, 3)% %spawner type of location(1, 2, 3)%\" to player",
    );
    assert_eq!(out, vec!["<none> <none>"]);
}

#[test]
fn bad_lines_are_rejected() {
    assert!(compile_file("t.carve", &script_for("set whitelist to true")).is_ok());
    assert!(compile_file("t.carve", &script_for("set portal cooldown to 5")).is_err());
}

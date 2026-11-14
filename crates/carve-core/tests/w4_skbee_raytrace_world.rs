use carve_core::compile::{EventKind, compile_file};
use carve_core::w2_expressions_7::Query;
use carve_core::w4_skbee_raytrace_world::{
    Env, RawBlock, RawEntity, RayRaw, RayReq, clean_world_name, look_dir, normalize_tag, tag_members,
};
use carve_core::*;

#[derive(Default)]
struct Mock {
    sent: Vec<String>,
    errors: Vec<String>,
    warns: Vec<String>,
    rays: Vec<RayReq>,
    /// What the next ray finds.
    raw: RayRaw,
    created: Vec<(String, Env)>,
    unloaded: Vec<String>,
    worlds: Vec<String>,
    loaded_chunks: Vec<(String, i32, i32)>,
    block: Option<String>,
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
    fn find_player(&mut self, _n: &str) -> Option<PlayerRef> {
        None
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
    fn w2e7_query(&mut self, q: Query<'_>) -> Option<Value> {
        match q {
            // Facing south: +z.
            Query::Yaw(_) => Some(Value::Num(0.0)),
            Query::Pitch(_) => Some(Value::Num(0.0)),
            _ => None,
        }
    }
    fn world_names(&mut self) -> Vec<String> {
        self.worlds.clone()
    }
    fn block_type(&mut self, _at: &carve_core::block_ops::BlockPos) -> Option<String> {
        self.block.clone()
    }
    fn ray_trace(&mut self, req: &RayReq) -> RayRaw {
        self.rays.push(req.clone());
        self.raw.clone()
    }
    fn create_world(&mut self, name: &str, env: Env) -> Result<(), String> {
        self.created.push((name.to_string(), env));
        self.worlds.push(name.to_string());
        Ok(())
    }
    fn unload_world(&mut self, name: &str) -> Result<(), String> {
        if name == "locked" {
            return Err("players are inside".into());
        }
        self.unloaded.push(name.to_string());
        self.worlds.retain(|w| w != name);
        Ok(())
    }
    fn chunk_loaded(&mut self, world: &str, x: i32, z: i32) -> Option<bool> {
        Some(self.loaded_chunks.contains(&(world.to_string(), x, z)))
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        match level {
            LogLevel::Error => self.errors.push(msg.to_string()),
            LogLevel::Warn => self.warns.push(msg.to_string()),
            _ => {}
        }
    }
}

fn run_with(h: &mut Mock, body: &str) -> Vec<String> {
    let src = format!(
        "on join:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    );
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    let p = PlayerRef { uuid: "u1".into(), name: "Steve".into() };
    e.fire(EventKind::Join, EventCtx::player(p), h);
    assert!(h.errors.is_empty(), "{:?}", h.errors);
    std::mem::take(&mut h.sent)
}

fn run(body: &str) -> Vec<String> {
    run_with(&mut Mock::default(), body)
}

fn compiles(body: &str) -> bool {
    let src = format!("on join:\n    {}\n", body.replace('\n', "\n    "));
    compile_file("t.carve", &src).is_ok()
}

fn block_hit(x: i32, y: i32, z: i32, face: &str, hit: (f64, f64, f64)) -> RayRaw {
    RayRaw { block: Some(RawBlock { pos: (x, y, z), face: face.into(), hit }), entity: None }
}

// ---- ray tracing ------------------------------------------------------------

#[test]
fn ray_from_player_uses_eye_and_look_direction() {
    let mut h = Mock { raw: block_hit(10, 65, 40, "north", (10.5, 65.6, 40.0)), ..Mock::default() };
    let out = run_with(
        &mut h,
        "set {_r} to ray trace from player with max distance 50\nsend \"%hit block of {_r}%\"\nsend \"%hit face of {_r}%\"",
    );
    assert_eq!(out, vec!["x: 10, y: 65, z: 40 in 'world'", "north"]);
    let req = &h.rays[0];
    assert_eq!(req.world.as_deref(), Some("world"));
    assert!((req.start.1 - 65.62).abs() < 1e-9);
    // Yaw 0 looks along +z, 50 blocks.
    assert!((req.end.2 - (10.0 + 50.0)).abs() < 1e-6);
    assert!((req.end.0 - 10.0).abs() < 1e-6);
    assert_eq!(req.exclude.as_deref(), Some("u1"));
    assert!(req.blocks && req.entities && !req.fluids);
}

#[test]
fn ray_default_distance_is_twenty() {
    let mut h = Mock::default();
    run_with(&mut h, "set {_r} to ray trace from player");
    assert!((h.rays[0].end.2 - 30.0).abs() < 1e-6);
}

#[test]
fn ray_miss_is_none() {
    let out = run("set {_r} to ray trace from player with max distance 5\nif {_r} is not set:\n    send \"miss\"");
    assert_eq!(out, vec!["miss"]);
    let out = run("set {_r} to ray trace from player\nsend \"%hit block of {_r}%\"");
    assert_eq!(out, vec!["<none>"]);
}

#[test]
fn ray_position_and_distance() {
    let mut h = Mock { raw: block_hit(10, 65, 14, "north", (10.0, 65.62, 14.0)), ..Mock::default() };
    let out = run_with(
        &mut h,
        "set {_r} to ray trace from player\nsend \"%hit position of {_r}%\"\nsend \"%hit distance of {_r}%\"",
    );
    assert_eq!(out[0], "x: 10, y: 65.62, z: 14 in 'world'");
    assert_eq!(out[1], "4");
}

#[test]
fn ray_entity_wins_when_nearer() {
    let mut h = Mock {
        raw: RayRaw {
            block: Some(RawBlock { pos: (10, 65, 40), face: "north".into(), hit: (10.0, 65.62, 40.0) }),
            entity: Some(RawEntity { uuid: "z1".into(), kind: "zombie".into(), hit: (10.0, 65.62, 13.0) }),
        },
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "set {_r} to ray trace from player\nsend \"%hit entity of {_r}%\"\nsend \"%hit block of {_r}%\"",
    );
    assert_eq!(out, vec!["zombie", "<none>"]);
}

#[test]
fn ray_block_wins_when_nearer_and_player_entity_is_a_player() {
    let mut h = Mock {
        raw: RayRaw {
            block: Some(RawBlock { pos: (1, 2, 3), face: "up".into(), hit: (1.0, 2.0, 12.0) }),
            entity: Some(RawEntity { uuid: "u1".into(), kind: "player".into(), hit: (10.0, 65.0, 40.0) }),
        },
        ..Mock::default()
    };
    let out = run_with(&mut h, "set {_r} to ray trace from player\nsend \"%hit entity of {_r}%\"");
    assert_eq!(out, vec!["<none>"]);
    h.raw.block = None;
    let out = run_with(&mut h, "set {_r} to ray trace from player\nsend \"%hit entity of {_r}%\"");
    assert_eq!(out, vec!["Steve"]);
}

#[test]
fn ray_from_location_needs_a_direction() {
    let mut h = Mock { raw: block_hit(0, 0, 0, "up", (0.0, 0.0, 0.0)), ..Mock::default() };
    let out = run_with(
        &mut h,
        "set {_r} to ray trace from location(0, 70, 0, world \"world\")\nif {_r} is not set:\n    send \"no direction\"",
    );
    assert_eq!(out, vec!["no direction"]);
    assert!(h.rays.is_empty());
}

#[test]
fn ray_in_direction_with_flags() {
    let mut h = Mock { raw: block_hit(0, 60, 0, "up", (0.5, 61.0, 0.5)), ..Mock::default() };
    let out = run_with(
        &mut h,
        "set {_r} to ray trace from location(0, 70, 0, world \"world\") in direction vector(0, -1, 0) with max distance 30 including fluids and ignoring entities\nsend \"%hit block face of {_r}%\"",
    );
    assert_eq!(out, vec!["up"]);
    let req = &h.rays[0];
    assert!(req.fluids && !req.entities && req.blocks);
    assert_eq!(req.world.as_deref(), Some("world"));
    assert!((req.end.1 - 40.0).abs() < 1e-6);
    assert_eq!(req.exclude, None);
}

#[test]
fn ray_distance_is_capped() {
    let mut h = Mock::default();
    run_with(&mut h, "set {_r} to ray trace from player with max distance 100000");
    assert!((h.rays[0].end.2 - (10.0 + 512.0)).abs() < 1e-6);
}

#[test]
fn ray_parse_errors() {
    assert!(!compiles("set {_r} to ray trace from player with ray size 2"));
    assert!(!compiles("set {_r} to ray trace from 5"));
    assert!(!compiles("set {_r} to ray trace from player with max distance \"far\""));
    assert!(compiles("set {_r} to ray trace from player ignoring blocks"));
}

#[test]
fn look_dir_matches_minecraft() {
    let (x, y, z) = look_dir(0.0, 0.0);
    assert!((x.abs() < 1e-9) && y.abs() < 1e-9 && (z - 1.0).abs() < 1e-9);
    let (x, _, z) = look_dir(90.0, 0.0);
    assert!((x + 1.0).abs() < 1e-9 && z.abs() < 1e-9);
    let (_, y, _) = look_dir(0.0, 90.0);
    assert!((y + 1.0).abs() < 1e-9);
}

// ---- worlds -----------------------------------------------------------------

#[test]
fn create_and_load_and_unload_worlds() {
    let mut h = Mock::default();
    run_with(
        &mut h,
        "create world \"arena\"\ncreate a new world named \"hell\" with environment nether\ncreate world \"sky\" with the dimension the end\nload world \"old\"\nunload world \"hell\"",
    );
    assert_eq!(
        h.created,
        vec![
            ("arena".to_string(), Env::Normal),
            ("hell".to_string(), Env::Nether),
            ("sky".to_string(), Env::End),
            ("old".to_string(), Env::Normal),
        ]
    );
    assert_eq!(h.unloaded, vec!["hell"]);
}

#[test]
fn world_loaded_condition() {
    let mut h = Mock { worlds: vec!["world".into()], ..Mock::default() };
    let out = run_with(
        &mut h,
        "if world \"world\" is loaded:\n    send \"a\"\nif world \"nope\" is not loaded:\n    send \"b\"\ncreate world \"nope\"\nif world \"nope\" is loaded:\n    send \"c\"",
    );
    assert_eq!(out, vec!["a", "b", "c"]);
}

#[test]
fn unload_failure_is_a_script_error_not_a_panic() {
    let mut h = Mock::default();
    let src = "on join:\n    unload world \"locked\"\n    send \"after\"\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty());
    let p = PlayerRef { uuid: "u1".into(), name: "Steve".into() };
    e.fire(EventKind::Join, EventCtx::player(p), &mut h);
    assert!(h.errors.iter().any(|m| m.contains("players are inside")), "{:?}", h.errors);
}

#[test]
fn bad_world_names_are_refused() {
    assert!(clean_world_name("arena_1").is_ok());
    assert!(clean_world_name("../etc").is_err());
    assert!(clean_world_name("a b").is_err());
    assert!(clean_world_name("").is_err());
    assert!(clean_world_name(".hidden").is_err());
    let mut h = Mock::default();
    let src = "on join:\n    create world \"../evil\"\n";
    let mut e = Engine::default();
    e.load(&[("t.carve".into(), src.into())]);
    let p = PlayerRef { uuid: "u1".into(), name: "Steve".into() };
    e.fire(EventKind::Join, EventCtx::player(p), &mut h);
    assert!(h.created.is_empty());
    assert!(h.errors.iter().any(|m| m.contains("not a valid world name")), "{:?}", h.errors);
}

// ---- chunks -----------------------------------------------------------------

#[test]
fn chunk_at_location_and_coordinates() {
    let out = run(
        "set {_c} to chunk at location(33, 64, -1, world \"world\")\nsend \"%chunk x of {_c}% %chunk z of {_c}%\"\nsend \"%world of {_c}%\"",
    );
    assert_eq!(out, vec!["2 -1", "world"]);
}

#[test]
fn chunk_at_player_and_of_player() {
    assert_eq!(run("set {_c} to chunk of player\nsend \"%chunk x of {_c}%\"")[0], "0");
    assert_eq!(run("set {_c} to chunk at player\nsend \"%chunk z of {_c}%\"")[0], "0");
}

#[test]
fn chunk_from_numbers() {
    let out = run("set {_c} to chunk at 3, -4 in world \"nether\"\nsend \"%{_c}%\"\nsend \"%chunk x of {_c}%\"");
    assert_eq!(out, vec!["chunk 3, -4 in 'nether'", "3"]);
}

#[test]
fn chunk_equality() {
    let out = run(
        "set {_a} to chunk at 1, 2\nset {_b} to chunk at location(16, 5, 32)\nif {_a} is {_b}:\n    send \"same\"\nset {_c} to chunk at 9, 9\nif {_a} is not {_c}:\n    send \"differ\"",
    );
    assert_eq!(out, vec!["same", "differ"]);
}

#[test]
fn chunk_loaded_condition() {
    let mut h = Mock { loaded_chunks: vec![("world".into(), 0, 0)], ..Mock::default() };
    let out = run_with(
        &mut h,
        "if chunk of player is loaded:\n    send \"here\"\nset {_far} to chunk at 50, 50 in world \"world\"\nif {_far} is not loaded:\n    send \"far unloaded\"",
    );
    assert_eq!(out, vec!["here", "far unloaded"]);
}

#[test]
fn chunks_in_radius() {
    let out = run("set {_l::*} to chunks in radius 1 around player\nsend \"%size of {_l::*}%\"");
    assert_eq!(out, vec!["9"]);
    let out = run("set {_l::*} to chunks in radius 2 around chunk at 5, 5\nsend \"%size of {_l::*}%\"");
    assert_eq!(out, vec!["25"]);
    let out = run("set {_l::*} to chunks in radius 999 around player\nsend \"%size of {_l::*}%\"");
    assert_eq!(out, vec!["1089"]);
}

// ---- tags -------------------------------------------------------------------

#[test]
fn block_is_tagged() {
    let mut h = Mock { block: Some("oak_log".into()), ..Mock::default() };
    let out = run_with(
        &mut h,
        "set {_b} to block at player\nif {_b} is tagged with minecraft tag \"logs\":\n    send \"log\"\nif {_b} is tagged with minecraft tag \"minecraft:planks\":\n    send \"planks\"\nif {_b} isn't tagged with tag \"leaves\":\n    send \"not leaves\"\nif {_b} is tagged as \"#logs\":\n    send \"short\"",
    );
    assert_eq!(out, vec!["log", "not leaves", "short"]);
}

#[test]
fn item_is_tagged() {
    let out = run(
        "if stone is tagged with minecraft tag \"base_stone_overworld\":\n    send \"a\"\nif oak planks is tagged with minecraft tag \"planks\":\n    send \"b\"\nif diamond is not tagged with minecraft tag \"planks\":\n    send \"c\"",
    );
    assert_eq!(out, vec!["a", "b", "c"]);
}

#[test]
fn unknown_tag_warns_and_is_false() {
    let mut h = Mock::default();
    let out = run_with(
        &mut h,
        "if stone is tagged with minecraft tag \"not_a_tag\":\n    send \"yes\"\nsend \"done\"",
    );
    assert_eq!(out, vec!["done"]);
    assert!(h.warns.iter().any(|w| w.contains("unknown minecraft tag 'not_a_tag'")), "{:?}", h.warns);
}

#[test]
fn tag_members_and_tags_of() {
    let out = run("set {_l::*} to minecraft tag \"wooden_slabs\"\nsend \"%size of {_l::*}%\"");
    assert_eq!(out, vec!["12"]);
    let out = run("set {_l::*} to minecraft block tag \"logs\"\nif {_l::*} contains oak log:\n    send \"has oak\"");
    assert_eq!(out, vec!["has oak"]);
    let out = run("set {_t::*} to minecraft tags of oak log\nif {_t::*} contains \"minecraft:logs\":\n    send \"yes\"");
    assert_eq!(out, vec!["yes"]);
}

#[test]
fn tag_data_is_sane() {
    assert_eq!(normalize_tag("Logs"), "minecraft:logs");
    assert_eq!(normalize_tag("#minecraft:logs"), "minecraft:logs");
    assert_eq!(normalize_tag("c:ores"), "c:ores");
    assert!(tag_members(carve_core::w4_skbee_raytrace_world::TagKind::Block, "logs").unwrap().contains("oak_log"));
    assert!(tag_members(carve_core::w4_skbee_raytrace_world::TagKind::Item, "nope").is_none());
    // Every tag member is a real item or block name.
    for table in [carve_core::w4_tags_data::BLOCK_TAGS, carve_core::w4_tags_data::ITEM_TAGS] {
        assert!(table.windows(2).all(|w| w[0].0 < w[1].0), "tag table must be sorted");
        for (tag, members) in table {
            for m in members.split(' ').filter(|m| !m.is_empty()) {
                assert!(carve_core::items::is_known(m), "{tag}: unknown name {m}");
            }
        }
    }
}

#[test]
fn errors_are_reported_per_line() {
    let src = "on join:\n    send \"ok\"\n    set {_r} to ray trace from player with ray size 3\n    send \"still ok\"\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert_eq!(errs.len(), 1);
    assert!(errs[0].to_string().contains("ray size"), "{}", errs[0]);
}

#[test]
fn registries_are_valid() {
    carve_core::compile::check_registry().unwrap();
}

use carve_core::block_ops::BlockPos;
use carve_core::compile::EventKind;
use carve_core::skbee_nbt::ItemComp;
use carve_core::w2_events_4::{self as ev4, ExtEvent};
use carve_core::w5_chunk1::{self as w5, HostOp, Ignite, PathTarget, PotionFlag};
use carve_core::world_effects::FxPos;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    commands: Vec<String>,
    ops: Vec<String>,
    logs: Vec<(LogLevel, String)>,
    clock: i64,
    border: f64,
    whitelist: Vec<String>,
    whitelist_off: bool,
    held: bool,
    comps: Vec<(ItemComp, Option<Vec<u8>>)>,
    file: Option<Vec<u8>>,
    leaves: bool,
    next: u32,
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let who = match to {
            Recipient::Player(p) => p.name.clone(),
            Recipient::Console => "console".into(),
        };
        self.calls.push(format!("send {who} {text}"));
    }
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.commands.push(c.to_string());
    }
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        match uuid {
            "00000001-0000-0000-0000-000000000002" => Some("Steve".into()),
            "u2" => Some("Alex".into()),
            _ => None,
        }
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        name.eq_ignore_ascii_case("steve").then(steve)
    }
    fn has_permission(&mut self, _: &PlayerRef, node: &str) -> bool {
        node == "skip.cd"
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn wall_clock_millis(&mut self) -> i64 {
        self.clock
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn online_players(&mut self) -> Vec<PlayerRef> {
        vec![
            steve(),
            PlayerRef {
                uuid: "u2".into(),
                name: "Alex".into(),
            },
        ]
    }
    fn player_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((0.5, 64.0, 0.5, "world".to_string()))
    }
    fn default_world(&mut self) -> Option<String> {
        Some("world".into())
    }
    fn held_item(&mut self, _: &PlayerRef) -> Option<(String, u32)> {
        self.held.then(|| ("leather_helmet".into(), 1))
    }
    fn set_held_component(&mut self, _: &PlayerRef, c: ItemComp, b: Option<Vec<u8>>) -> bool {
        self.comps.push((c, b));
        true
    }
    fn spawn_entity(&mut self, kind: &str, _: &FxPos) -> Option<EntityRef> {
        self.next += 1;
        Some(EntityRef {
            uuid: format!("0000000{}-0000-0000-0000-00000000000a", self.next),
            kind: kind.into(),
        })
    }
    fn entity_location(&mut self, _: &EntityRef) -> Option<(f64, f64, f64, String)> {
        Some((10.0, 70.0, -3.0, "world".into()))
    }
    fn entity_remove(&mut self, e: &EntityRef) {
        self.ops.push(format!("remove {}", e.kind));
    }
    fn explode(&mut self, at: &FxPos, power: f64, _: bool) {
        self.ops
            .push(format!("explode {} {} {} {power}", at.x, at.y, at.z));
    }
    fn is_whitelisted(&mut self, name: &str) -> bool {
        self.whitelist.iter().any(|n| n == name)
    }
    fn is_op(&mut self, _: &PlayerRef) -> bool {
        false
    }
    fn kick(&mut self, name: &str, reason: Option<&str>) {
        self.ops.push(format!("kick {name} {}", reason.unwrap_or("")));
    }
    fn border_size(&mut self, _: &str) -> Option<f64> {
        Some(self.border)
    }
    fn set_border_size(&mut self, w: &str, size: f64, secs: f64) {
        self.ops.push(format!("border {w} {size} {secs}"));
    }
    fn block_type(&mut self, _: &BlockPos) -> Option<String> {
        Some(if self.leaves { "oak_leaves" } else { "stone" }.into())
    }
    fn read_data_file(&mut self, path: &str) -> Option<Vec<u8>> {
        (path == "icon.png").then(|| self.file.clone()).flatten()
    }
    fn w5c1(&mut self, op: HostOp<'_>) -> Option<bool> {
        let kind = |v: &Value| match v {
            Value::Entity(e) => e.kind.clone(),
            Value::Player(_) => "player".into(),
            other => other.type_name().into(),
        };
        let (line, out) = match op {
            HostOp::Creeper(v, m) => (format!("creeper {} {m:?}", kind(v)), Some(kind(v) == "creeper")),
            HostOp::Ageable(v) => (format!("ageable {}", kind(v)), Some(kind(v) == "cow")),
            HostOp::Pathfind(v, t, s) => {
                let t = match t {
                    PathTarget::Pos(x, y, z) => format!("{x} {y} {z}"),
                    PathTarget::Entity(e) => kind(e),
                };
                (format!("path {} {t} {s}", kind(v)), Some(true))
            }
            HostOp::StopPath(v) => (format!("stop {}", kind(v)), Some(true)),
            HostOp::SignGlow(w, x, y, z, on) => (format!("sign {w} {x} {y} {z} {on}"), Some(true)),
            HostOp::Potion(p, f, on) => (format!("potion {} {f:?} {on}", p.name), Some(true)),
            HostOp::WhitelistOn => ("whitelist?".into(), Some(!self.whitelist_off)),
        };
        self.ops.push(line);
        out
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "00000001-0000-0000-0000-000000000002".into(),
        name: "Steve".into(),
    }
}

fn me() -> Recipient {
    Recipient::Player(steve())
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

fn errors(src: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".to_string(), src.to_string())]);
    errs.iter().map(|e| e.to_string()).collect()
}

/// Run `on join` with these lines and return the mock.
fn join(lines: &str, mut h: Mock) -> Mock {
    let body: String = lines.lines().map(|l| format!("    {l}\n")).collect();
    let mut e = load(&format!("on join:\n{body}"));
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(
        h.logs.iter().all(|(l, _)| *l != LogLevel::Error),
        "runtime errors: {:?}",
        h.logs
    );
    h
}

#[test]
fn patterns_compile() {
    let n = w5::check_patterns().unwrap();
    assert!(n > 90, "only {n} patterns");
    assert_eq!(w5::pattern_sources().len(), n);
}

#[test]
fn cooldowns() {
    let src = "command /cd:\n    cooldown: 10 seconds\n    cooldown message: wait %remaining time%\n    trigger:\n        send \"ran\"\n        if {skip} is set:\n            cancel the cooldown\n";
    let mut e = load(src);
    let mut h = Mock {
        clock: 1_000,
        ..Mock::default()
    };
    assert_eq!(e.run_command("cd", me(), "", &mut h), CommandOutcome::Ran);
    h.clock = 5_000;
    assert_eq!(
        e.run_command("cd", me(), "", &mut h),
        CommandOutcome::Usage("wait 6 seconds".into())
    );
    // The console has no cooldown.
    assert_eq!(
        e.run_command("cd", Recipient::Console, "", &mut h),
        CommandOutcome::Ran
    );
    h.clock = 11_000;
    assert_eq!(e.run_command("cd", me(), "", &mut h), CommandOutcome::Ran);
    // A cancelled cooldown does not count this use.
    e.vars.set_str("skip", Value::Bool(true));
    h.clock = 30_000;
    assert_eq!(e.run_command("cd", me(), "", &mut h), CommandOutcome::Ran);
    h.clock = 31_000;
    assert_eq!(e.run_command("cd", me(), "", &mut h), CommandOutcome::Ran);
}

#[test]
fn cooldown_default_message_and_bypass() {
    let mut e = load(
        "command /a:\n    cooldown: 1 minute\n    trigger:\n        send \"a\"\ncommand /b:\n    cooldown: 1 minute\n    cooldown bypass: skip.cd\n    trigger:\n        send \"b\"\n",
    );
    let mut h = Mock::default();
    e.run_command("a", me(), "", &mut h);
    match e.run_command("a", me(), "", &mut h) {
        CommandOutcome::Usage(m) => assert!(m.contains("1 minute") && m.contains("wait"), "{m}"),
        o => panic!("{o:?}"),
    }
    e.run_command("b", me(), "", &mut h);
    assert_eq!(e.run_command("b", me(), "", &mut h), CommandOutcome::Ran);
    assert!(
        errors("command /c:\n    cooldown: soon\n    trigger:\n        send \"x\"\n")[0].contains("cooldown")
    );
    assert!(!errors("command /c:\n    cooldown storage: {x}\n    trigger:\n        send \"x\"\n").is_empty());
    assert!(!errors("on join:\n    cancel the cooldown\n").is_empty());
}

#[test]
fn fail_command() {
    let mut e = load(
        "command /pay <number>:\n    trigger:\n        if arg-1 > 5:\n            fail the command execution with the error message \"too much: %arg-1%\"\n        send \"paid\"\n",
    );
    let mut h = Mock::default();
    assert_eq!(
        e.run_command("pay", me(), "9", &mut h),
        CommandOutcome::Usage("too much: 9".into())
    );
    assert!(h.calls.is_empty());
    assert_eq!(e.run_command("pay", me(), "2", &mut h), CommandOutcome::Ran);
    assert_eq!(h.calls.len(), 1);
    assert!(!errors("on join:\n    fail the command execution with error \"x\"\n").is_empty());
}

#[test]
fn drops_and_keep_inventory() {
    let mut e = load(
        "on break:\n    cancel the drops\non death:\n    keep the inventory and experience\n    clear drops of xp\n",
    );
    let mut h = Mock::default();
    let o = e.fire(EventKind::Break, EventCtx::player(steve()), &mut h);
    let slots: Vec<u8> = o.set.iter().map(|(s, _)| *s).collect();
    assert_eq!(slots, vec![w5::SLOT_NO_ITEM_DROPS, w5::SLOT_NO_XP_DROPS]);
    let o = e.fire(EventKind::Death, EventCtx::player(steve()), &mut h);
    let slots: Vec<u8> = o.set.iter().map(|(s, _)| *s).collect();
    assert_eq!(
        slots,
        vec![w5::SLOT_KEEP_INVENTORY, w5::SLOT_KEEP_XP, w5::SLOT_NO_XP_DROPS]
    );
    let mut e = load("on death:\n    keep the inventory\n");
    let o = e.fire(EventKind::Death, EventCtx::player(steve()), &mut h);
    assert_eq!(o.set.len(), 1);
    assert!(errors("on join:\n    cancel drops\n")[0].contains("drops"));
    assert!(errors("on break:\n    keep the inventory\n")[0].contains("death"));
}

#[test]
fn dye_and_glint() {
    let h = join(
        "dye player's tool red\npaint player's tool rgb(1, 2, 3)\nforce player's tool to glint\nmake player's tool stop glinting\nreset the enchantment glint override of player's tool",
        Mock {
            held: true,
            ..Mock::default()
        },
    );
    assert_eq!(h.comps.len(), 5);
    assert_eq!(h.comps[0].0, ItemComp::DyedColor);
    assert_eq!(h.comps[0].1.as_ref().unwrap()[0], 0);
    assert_eq!(h.comps[1].1, Some(vec![0, 1, 2, 3]));
    assert_eq!(h.comps[2], (ItemComp::GlintOverride, Some(vec![1])));
    assert_eq!(h.comps[3], (ItemComp::GlintOverride, Some(vec![0])));
    assert_eq!(h.comps[4], (ItemComp::GlintOverride, None));
    // Nothing held: nothing to change.
    assert!(join("dye player's tool red", Mock::default()).comps.is_empty());
    assert!(errors("on join:\n    dye {x} red\n")[0].contains("held item"));
}

#[test]
fn creepers_and_detonate() {
    let h = join(
        "spawn a creeper\ndetonate last spawned entity\nspawn a tnt\ndetonate last spawned entity\nspawn a cow\ndetonate last spawned entity\nspawn a creeper\nignite creeper last spawned entity\nstop ignition of last spawned entity\nexplode creeper last spawned entity instantly",
        Mock::default(),
    );
    assert_eq!(
        h.ops,
        vec![
            "creeper creeper Instant",
            "creeper tnt Instant",
            "remove tnt",
            "explode 10 70 -3 4",
            "creeper cow Instant",
            "creeper creeper Start",
            "creeper creeper Stop",
            "creeper creeper Instant",
        ]
    );
}

#[test]
fn drop_items() {
    let h = join(
        "drop 3 diamonds\ndrop stone at location(1, 2, 3) without velocity",
        Mock::default(),
    );
    assert_eq!(
        h.commands,
        vec![
            "summon minecraft:item 0.500 64.000 0.500 {Item:{id:\"minecraft:diamond\",count:3}}",
            "summon minecraft:item 1.000 2.000 3.000 {Item:{id:\"minecraft:stone\",count:1},Motion:[0.0d,0.0d,0.0d]}",
        ]
    );
}

#[test]
fn enforce_whitelist() {
    let h = join(
        "enforce the whitelist",
        Mock {
            whitelist: vec!["Steve".into()],
            ..Mock::default()
        },
    );
    assert_eq!(
        h.ops,
        vec!["whitelist?", "kick Alex You are not whitelisted on this server!"]
    );
    let h = join(
        "enforce whitelist",
        Mock {
            whitelist_off: true,
            ..Mock::default()
        },
    );
    assert_eq!(h.ops, vec!["whitelist?"]);
    assert!(
        join("unenforce the server white list", Mock::default())
            .ops
            .is_empty()
    );
}

#[test]
fn world_border() {
    let h = join(
        "expand the world border by 10\nshrink the radius of the world border of world \"world\" to 20 over 5 seconds\ngrow the diameter of the border to 50\ncontract world border by 400",
        Mock {
            border: 100.0,
            ..Mock::default()
        },
    );
    assert_eq!(
        h.ops,
        vec![
            "border world 110 0",
            "border world 40 5",
            "border world 50 0",
            "border world 1 0"
        ]
    );
}

#[test]
fn leash_and_persist() {
    assert_eq!(
        w5::uuid_ints("00000001-0000-0000-ffff-fffffffffffe"),
        Some([1, 0, -1, -2])
    );
    let h = join(
        "spawn a cow\nleash last spawned entity to player\nunleash last spawned entity\nmake last spawned entity persistent\nprevent last spawned entity from persisting",
        Mock::default(),
    );
    assert_eq!(
        h.commands,
        vec![
            "data merge entity 00000001-0000-0000-0000-00000000000a {leash:{UUID:[I;1,0,0,2]}}",
            "data merge entity 00000001-0000-0000-0000-00000000000a {leash:0b}",
            "data merge entity 00000001-0000-0000-0000-00000000000a {PersistenceRequired:1b}",
            "data merge entity 00000001-0000-0000-0000-00000000000a {PersistenceRequired:0b}",
        ]
    );
    let h = join(
        "make block at location(1, 2, 3) persistent",
        Mock {
            leaves: true,
            ..Mock::default()
        },
    );
    assert_eq!(
        h.commands,
        vec!["setblock 1 2 3 minecraft:oak_leaves[persistent=true]"]
    );
}

fn png(w: u32, h: u32) -> Vec<u8> {
    let mut b = b"\x89PNG\r\n\x1a\n\0\0\0\rIHDR".to_vec();
    b.extend_from_slice(&w.to_be_bytes());
    b.extend_from_slice(&h.to_be_bytes());
    b.extend_from_slice(&[8, 6, 0, 0, 0]);
    b
}

#[test]
fn server_icon() {
    assert!(
        w5::icon_data_url(&png(64, 64))
            .unwrap()
            .starts_with("data:image/png;base64,iVBORw0KGgo")
    );
    assert!(w5::icon_data_url(&png(32, 32)).unwrap_err().contains("64x64"));
    assert!(w5::icon_data_url(b"GIF89a").is_err());
    let mut e = load(
        "on load:\n    load the server icon from file \"icon.png\"\non server list ping:\n    set the server icon to the last loaded server icon\n",
    );
    let mut h = Mock {
        file: Some(png(64, 64)),
        ..Mock::default()
    };
    e.fire(EventKind::Load, EventCtx::console(), &mut h);
    let vals = vec![
        Value::Text("motd".into()),
        Value::Num(10.0),
        Value::Num(0.0),
        Value::Text("localhost".into()),
        Value::Text("127.0.0.1:1".into()),
        Value::None,
    ];
    let o = e.fire(
        EventKind::Ext4(ExtEvent::ServerListPing),
        ev4::ctx(None, vals),
        &mut h,
    );
    assert_eq!(o.set.len(), 1);
    assert_eq!(o.set[0].0, 5);
    assert!(matches!(&o.set[0].1, Value::Text(t) if t.starts_with("data:image/png;base64,")));
    assert!(!errors("on join:\n    set the server icon to \"x\"\n").is_empty());
    assert!(errors("on server list ping:\n    set event-icon to \"x\"\n").is_empty());
}

#[test]
fn mobs_signs_and_potions() {
    let h = join(
        "spawn a cow\nmake last spawned entity a baby\nforce last spawned entity to become an adult\nmake last spawned entity pathfind to location(1, 2, 3) at speed 1.5\nmake last spawned entity move towards player\nmake last spawned entity stop pathfinding\nmake block at location(4, 5, 6) have glowing text\nmake location(4, 5, 6) have normal text\nmake player's potion effects ambient\nmake the potion effects of player not infinite\nhide the particles of player's potion effects\nshow the potion icon for the active potion effects of player",
        Mock::default(),
    );
    assert_eq!(
        h.ops,
        vec![
            "path cow 1 2 3 1.5",
            "path cow player 1",
            "stop cow",
            "sign world 4 5 6 true",
            "sign world 4 5 6 false",
            "potion Steve Ambient true",
            "potion Steve Infinite false",
            "potion Steve Particles false",
            "potion Steve Icon true",
        ]
    );
    let _ = PotionFlag::Ambient;
    let _ = Ignite::Start;
}

#[test]
fn can_age() {
    let h = join(
        "spawn a cow\nif last spawned entity can age:\n    send \"cow ages\"\nspawn a zombie\nif last spawned entity can't grow up:\n    send \"zombie stays\"",
        Mock::default(),
    );
    assert_eq!(h.calls, vec!["send Steve cow ages", "send Steve zombie stays"]);
}

#[test]
fn prepare_craft_event() {
    let mut e = load("on preparing craft:\n    send \"crafting %event-item%\" to player\n");
    let mut h = Mock::default();
    e.fire(
        EventKind::Ext4(ExtEvent::PrepareCraft),
        ev4::ctx(
            Some(steve()),
            vec![Value::Text("stick".into()), Value::Text("minecraft:stick".into())],
        ),
        &mut h,
    );
    assert_eq!(h.calls, vec!["send Steve crafting stick"]);
    assert!(errors("on beginning crafting:\n    cancel event\n").is_empty());
}

#[test]
fn babies_and_breeding() {
    let h = join(
        "spawn a cow\nmake last spawned entity a baby\nforce last spawned entity to become an adult\nmake last spawned entity unbreedable\nmake last spawned entity breedable",
        Mock::default(),
    );
    let u = "00000001-0000-0000-0000-00000000000a";
    assert_eq!(
        h.commands,
        vec![
            format!("data merge entity {u} {{Age:-24000,IsBaby:1b}}"),
            format!("data merge entity {u} {{Age:0,IsBaby:0b}}"),
            format!("data merge entity {u} {{Age:6000}}"),
            format!("data merge entity {u} {{Age:0}}"),
        ]
    );
}

#[test]
fn launch_firework() {
    let h = join(
        "launch a firework at location(1, 2, 3)\nlaunch firework effect \"ball\" at location(4, 5, 6) timed 2",
        Mock::default(),
    );
    assert_eq!(
        h.commands,
        vec![
            "summon minecraft:firework_rocket 1.000 2.000 3.000",
            "summon minecraft:firework_rocket 4.000 5.000 6.000",
        ]
    );
}

#[test]
fn ingame_script_compiles() {
    let errs = errors(include_str!("../../../tests/ingame/areas/w5-chunk1/w5.carve"));
    assert!(errs.is_empty(), "{errs:?}");
}

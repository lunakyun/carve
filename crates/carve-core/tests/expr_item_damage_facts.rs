use carve_core::compile::{EventKind, check_registry, compile_file};
use carve_core::expr_item_damage_facts::DamageInfo;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    errors: Vec<String>,
    cooldown: Option<i64>,
    join_msg: Option<String>,
}

impl Host for Mock {
    fn send_message(&mut self, _: &Recipient, text: &str) {
        self.calls.push(text.to_string());
    }
    fn broadcast(&mut self, _: &str) {}
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
    fn held_item(&mut self, _: &PlayerRef) -> Option<(String, u32)> {
        Some(("diamond_pickaxe".into(), 1))
    }
    fn player_info(&mut self, _: &PlayerRef, info: carve_core::skbee_utilities::Info) -> Option<String> {
        (info == carve_core::skbee_utilities::Info::Locale).then(|| "en_us".into())
    }
    fn item_cooldown(&mut self, _: &PlayerRef, item: &str) -> Option<i64> {
        (item == "ender_pearl").then_some(self.cooldown).flatten()
    }
    fn set_item_cooldown(&mut self, _: &PlayerRef, item: &str, ticks: i64) {
        self.calls.push(format!("cooldown {item} {ticks}"));
        self.cooldown = Some(ticks);
    }
    fn last_death_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((1.0, 70.0, -4.0, "world".into()))
    }
    fn last_damage(&mut self, _: &PlayerRef) -> Option<DamageInfo> {
        Some(DamageInfo {
            amount: 3.5,
            cause: "player_attack".into(),
            attacker: Some(PlayerRef {
                uuid: "u1".into(),
                name: "Steve".into(),
            }),
        })
    }
    fn join_message(&mut self) -> Option<String> {
        self.join_msg.clone()
    }
    fn set_join_message(&mut self, text: &str) {
        self.join_msg = Some(text.to_string());
    }
    fn resource_pack_status(&mut self, _: &PlayerRef) -> Option<String> {
        Some("successfully_loaded".into())
    }
    fn solid_block_y(&mut self, world: &str, x: i32, z: i32, highest: bool) -> Option<i32> {
        (world == "world" && x == 3 && z == 4).then_some(if highest { 70 } else { -64 })
    }
    fn world_names(&mut self) -> Vec<String> {
        vec!["world".into()]
    }
    fn light_level(&mut self, _: &str, _: i32, y: i32, _: i32, kind: u8) -> Option<u8> {
        Some(
            match kind {
                0 => 7,
                1 => 15,
                _ => 15,
            } - u8::try_from(y.max(0) % 2).unwrap_or(0),
        )
    }
    fn player_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((3.5, 64.0, 4.2, "world".into()))
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run_with(mut h: Mock, body: &str) -> Mock {
    let src = format!(
        "on join:\n{}\n",
        body.lines()
            .map(|l| format!("    {l}"))
            .collect::<Vec<_>>()
            .join("\n")
    );
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.clone())]);
    assert!(
        errs.is_empty(),
        "errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.errors.is_empty(), "runtime: {:?}", h.errors);
    h
}

fn run(body: &str) -> Vec<String> {
    run_with(Mock::default(), body).calls
}

fn errs(body: &str) -> usize {
    let src = format!("on join:\n    {body}\n");
    compile_file("bad.carve", &src).err().map_or(0, |e| e.len())
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn inverse_boolean() {
    assert_eq!(
        run("set {_a} to not true\nset {_b} to not false\nsend \"%{_a}% %{_b}%\""),
        vec!["false true"]
    );
    assert_eq!(run("set {_a} to not (not true)\nsend \"%{_a}%\""), vec!["true"]);
}

#[test]
fn number_limits() {
    let out = run("send \"%maximum double value%\"\nsend \"%max float value%\"");
    assert_eq!(out.len(), 2);
    assert!(out[0].starts_with("17976931348623157"), "{}", out[0]);
    assert!(out[1].starts_with("340282346638528"), "{}", out[1]);
    assert_eq!(
        run("if maximum float value < maximum double value:\n    send \"ok\""),
        vec!["ok"]
    );
}

#[test]
fn item_amount_and_key() {
    assert_eq!(run("send \"%item amount of 5 diamonds%\""), vec!["5"]);
    assert_eq!(run("send \"%item amount of diamond%\""), vec!["1"]);
    assert_eq!(
        run("send \"%key of diamond sword%\""),
        vec!["minecraft:diamond_sword"]
    );
    assert_eq!(
        run("send \"%namespaced key of 3 stone%\""),
        vec!["minecraft:stone"]
    );
}

#[test]
fn durability_and_use_time() {
    assert_eq!(run("send \"%max durability of diamond pickaxe%\""), vec!["1561"]);
    assert_eq!(run("send \"%max durability of iron chestplate%\""), vec!["240"]);
    assert_eq!(run("send \"%max durability of netherite boots%\""), vec!["481"]);
    assert_eq!(run("send \"%max durability of turtle helmet%\""), vec!["275"]);
    assert_eq!(run("send \"%max durability of bow%\""), vec!["384"]);
    assert_eq!(run("send \"%max durability of dirt%\""), vec!["<none>"]);
    assert_eq!(
        run("send \"%total seconds of max item use time of bow%\""),
        vec!["3600"]
    );
    assert_eq!(
        run("send \"%total seconds of max item use time of bread%\""),
        vec!["1.6"]
    );
    assert_eq!(run("send \"%max durability of player's tool%\""), vec!["1561"]);
}

#[test]
fn last_color() {
    assert_eq!(run("send last color of \"&aHello &cWorld\""), vec!["red"]);
    assert_eq!(run("send last color of \"&aHello\""), vec!["green"]);
    assert_eq!(run("send last colors of \"&#ff8800hi\""), vec!["#ff8800"]);
    assert_eq!(run("send \"%last color of \"plain\"%\""), vec!["<none>"]);
}

#[test]
fn language_and_freeze() {
    assert_eq!(run("send \"%language of player%\""), vec!["en_us"]);
    assert_eq!(run("send \"%locale of player%\""), vec!["en_us"]);
    assert_eq!(
        run("send \"%total ticks of max freeze time of player%\""),
        vec!["140"]
    );
}

#[test]
fn cooldown_read_and_write() {
    let out = run(
        "set cooldown of ender pearl for player to 5 seconds\nsend \"%total ticks of cooldown of ender pearl for player%\"",
    );
    assert_eq!(out, vec!["cooldown ender_pearl 100", "100"]);
    let out = run(
        "set item cooldown of ender pearl for player to 2 ticks\nreset cooldown of ender pearl for player\nsend \"%total ticks of item cooldown of ender pearl for player%\"",
    );
    assert_eq!(out, vec!["cooldown ender_pearl 2", "cooldown ender_pearl 0", "0"]);
    // other items have none
    assert_eq!(
        run("send \"%total ticks of cooldown of stone for player%\""),
        vec!["0"]
    );
}

#[test]
fn last_hit_facts() {
    assert_eq!(run("send \"%last damage of player%\""), vec!["3.5"]);
    assert_eq!(
        run("send \"%last damage cause of player%\""),
        vec!["player_attack"]
    );
    assert_eq!(run("send \"%name of last attacker of player%\""), vec!["Steve"]);
    assert_eq!(
        run("set {_l} to last death location of player\nsend \"%x of {_l}% %y of {_l}% %z of {_l}%\""),
        vec!["1 70 -4"]
    );
}

#[test]
fn join_message_and_pack() {
    let h = run_with(
        Mock {
            join_msg: Some("Steve joined".into()),
            ..Mock::default()
        },
        "send join message\nset the join message to \"&aWelcome\"\nsend join message",
    );
    assert_eq!(h.calls, vec!["Steve joined", "&aWelcome"]);
    assert_eq!(
        run("send \"%resource pack response of player%\""),
        vec!["successfully_loaded"]
    );
    assert_eq!(
        run("send last resource pack response"),
        vec!["successfully_loaded"]
    );
}

#[test]
fn solid_blocks() {
    let out =
        run("set {_l} to highest block at location(3.5, 10, 4.9, world \"world\")\nsend \"%y of {_l}%\"");
    assert_eq!(out, vec!["70"]);
    let out =
        run("set {_l} to lowest solid block at location(3, 10, 4, world \"world\")\nsend \"%y of {_l}%\"");
    assert_eq!(out, vec!["-64"]);
    assert_eq!(
        run("set {_l} to highest block at location(9, 10, 9, world \"world\")\nsend \"%{_l}%\""),
        vec!["<none>"]
    );
}

#[test]
fn bad_input_is_an_error_not_a_panic() {
    assert!(errs("send \"%item amount of 5%\"") > 0);
    assert!(errs("send \"%language of 5%\"") > 0);
    assert!(errs("send \"%cooldown of ender pearl%\"") > 0);
    assert!(errs("set cooldown of ender pearl for player to \"x\"") > 0);
}

#[test]
fn existing_words_still_work() {
    assert_eq!(run("send \"%max(2, 5)%\""), vec!["5"]);
    assert_eq!(run("send \"%last 2 characters of \"hello\"%\""), vec!["lo"]);
    assert_eq!(run("send \"%first 2 characters of \"hello\"%\""), vec!["he"]);
}

#[test]
fn light_levels() {
    let at = "location(1, 64, 1, world \"world\")";
    assert_eq!(run(&format!("send \"%block light level at {at}%\"")), vec!["7"]);
    assert_eq!(run(&format!("send \"%sky light level of {at}%\"")), vec!["15"]);
    assert_eq!(run(&format!("send \"%light level at {at}%\"")), vec!["15"]);
    assert!(errs("send \"%light level at 5%\"") > 0);
}

#[test]
fn light_and_highest_without_world_use_the_default_world() {
    assert_eq!(
        run("send \"%sky light level at location(1, 64, 1)%\""),
        vec!["15"]
    );
    assert_eq!(
        run("set {_l} to lowest block at location(3, 64, 4)\nsend \"%y of {_l}%\""),
        vec!["-64"]
    );
}

#[test]
fn location_at() {
    assert_eq!(
        run("set {_l} to location at (1, 2.5, -3) in world \"world\"
send \"%x of {_l}% %y of {_l}% %z of {_l}%\""),
        vec!["1 2.5 -3"]
    );
    assert_eq!(
        run("set {_l} to location at (4, 5, 6)
send \"%z of {_l}%\""),
        vec!["6"]
    );
    // Without brackets works too (Skript's own form).
    assert_eq!(
        run("set {_l} to location at 1, 2, 3
send \"%y of {_l}%\""),
        vec!["2"]
    );
    assert!(errs("set {_l} to location at (1, 2)") > 0);
}

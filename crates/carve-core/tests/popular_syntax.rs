//! Core Skript syntax that popular public scripts use: command argument
//! types and defaults, `message`, `player command`, `exit trigger`, slot
//! lists, offline players and the small conditions around them.

use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    perms: bool,
    offline: Vec<PlayerRef>,
    held: Option<(String, u32)>,
    time_ms: i64,
}

impl Mock {
    fn take(&mut self) -> Vec<String> {
        std::mem::take(&mut self.calls)
    }
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let who = match to {
            Recipient::Player(p) => p.name.clone(),
            Recipient::Console => "console".into(),
        };
        self.calls.push(format!("send {who} {text}"));
    }
    fn send_console(&mut self, text: &str) {
        self.calls.push(format!("console-log {text}"));
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
    fn execute_command(&mut self, c: &str) {
        self.calls.push(format!("cmd {c}"));
    }
    fn execute_command_as(&mut self, p: &PlayerRef, c: &str) {
        self.calls.push(format!("cmd-as {} {c}", p.name));
    }
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        name.eq_ignore_ascii_case("steve").then(steve)
    }
    fn online_players(&mut self) -> Vec<PlayerRef> {
        vec![steve()]
    }
    fn offline_players(&mut self) -> Vec<PlayerRef> {
        self.offline.clone()
    }
    fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
        self.perms
    }
    fn default_world(&mut self) -> Option<String> {
        Some("world".into())
    }
    fn world_names(&mut self) -> Vec<String> {
        vec!["world".into(), "world_nether".into()]
    }
    fn held_item(&mut self, _: &PlayerRef) -> Option<(String, u32)> {
        self.held.clone()
    }
    fn clear_held_item(&mut self, p: &PlayerRef) {
        self.calls.push(format!("clear-held {}", p.name));
        self.held = None;
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn wall_clock_millis(&mut self) -> i64 {
        self.time_ms
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        if level == LogLevel::Error {
            self.calls.push(format!("ERROR {msg}"));
        }
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn load(src: &str) -> (Engine, Vec<ScriptError>) {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    (e, errs)
}

fn engine(src: &str) -> Engine {
    let (e, errs) = load(src);
    assert!(
        errs.iter().all(|e| e.warning),
        "unexpected errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn errors_of(src: &str) -> Vec<ScriptError> {
    load(src).1.into_iter().filter(|e| !e.warning).collect()
}

fn join(src: &str, h: &mut Mock) {
    let mut e = engine(src);
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
}

fn cmd(e: &mut Engine, name: &str, args: &str, h: &mut Mock) -> CommandOutcome {
    e.run_command(name, Recipient::Player(steve()), args, h)
}

// ---------- command arguments ----------

#[test]
fn command_argument_types() {
    let mut e = engine(
        "command /t <offline player> <boolean> <integer> <timespan>:\n    trigger:\n        send \"%arg 1% %arg 2% %arg 3% %arg-4%\"\n",
    );
    let mut h = Mock::default();
    assert_eq!(
        cmd(&mut e, "t", "Alex yes 4 5 minutes", &mut h),
        CommandOutcome::Ran
    );
    // The last argument is a timespan of two words only when it is the last text; here it is one word.
    h.take();
    assert!(matches!(
        cmd(&mut e, "t", "Alex maybe 4 5", &mut h),
        CommandOutcome::Usage(m) if m.contains("not a boolean")
    ));
    assert!(matches!(
        cmd(&mut e, "t", "Alex true 1.5 5", &mut h),
        CommandOutcome::Usage(m) if m.contains("not an integer")
    ));
}

#[test]
fn offline_player_argument_is_a_player_without_being_online() {
    let mut e = engine(
        "command /age <offline player>:\n    trigger:\n        send \"%name of arg-1% %uuid of arg-1%\"\n",
    );
    let mut h = Mock::default();
    assert_eq!(cmd(&mut e, "age", "Alex", &mut h), CommandOutcome::Ran);
    let out = h.take();
    assert_eq!(out.len(), 1);
    assert!(out[0].starts_with("send Steve Alex "), "{out:?}");
    // The offline-mode UUID of a name never changes.
    assert_eq!(
        carve_core::popular_syntax::offline_uuid("Alex"),
        carve_core::popular_syntax::offline_uuid("Alex")
    );
    assert_eq!(carve_core::popular_syntax::offline_uuid("Alex").len(), 36);
}

#[test]
fn online_player_still_found_for_offline_player_argument() {
    let mut e = engine("command /who <offline player>:\n    trigger:\n        send \"%arg 1's uuid%\"\n");
    let mut h = Mock::default();
    cmd(&mut e, "who", "steve", &mut h);
    assert_eq!(h.take(), ["send Steve u1"]);
}

#[test]
fn argument_defaults() {
    let mut e = engine(
        "command /d [<number=5>] <text=hello> [<player = %player%>]:\n    trigger:\n        send \"%arg 1% %arg 2% %arg-3%\"\n",
    );
    let mut h = Mock::default();
    cmd(&mut e, "d", "", &mut h);
    assert_eq!(h.take(), ["send Steve 5 hello Steve"]);
    cmd(&mut e, "d", "9 yo", &mut h);
    assert_eq!(h.take(), ["send Steve 9 yo Steve"]);
}

#[test]
fn defaults_count_as_optional() {
    let errs = errors_of("command /d <number=5> <text>:\n    trigger:\n        stop\n");
    assert!(errs.iter().any(|e| e.msg.contains("optional")), "{errs:?}");
}

#[test]
fn world_item_and_timespan_arguments() {
    let mut e =
        engine("command /w <world> <item> [<timespan>]:\n    trigger:\n        send \"%arg 1% %arg 2%\"\n");
    let mut h = Mock::default();
    assert_eq!(
        cmd(&mut e, "w", "world_nether stone", &mut h),
        CommandOutcome::Ran
    );
    assert!(matches!(
        cmd(&mut e, "w", "nowhere stone", &mut h),
        CommandOutcome::Usage(m) if m.contains("no world")
    ));
    assert!(matches!(
        cmd(&mut e, "w", "world notanitem", &mut h),
        CommandOutcome::Usage(m) if m.contains("not an item")
    ));
}

#[test]
fn unknown_argument_type_is_an_error_with_the_list() {
    let errs = errors_of("command /x <wibble>:\n    trigger:\n        stop\n");
    assert_eq!(errs.len(), 1);
    assert!(errs[0].msg.contains("<offline player>"), "{errs:?}");
}

#[test]
fn double_slash_command_names_load() {
    let (e, errs) = load("command //fill <text>:\n    trigger:\n        stop\n");
    assert!(errs.iter().all(|e| e.warning), "{errs:?}");
    assert!(e.table().command("fill").is_some());
}

// ---------- permission message ----------

#[test]
fn permission_message_goes_to_the_player() {
    let mut e = engine(
        "command /vip:\n    permission: a.b\n    permission message: &cNo VIP for you\n    trigger:\n        send \"hi\"\n",
    );
    let mut h = Mock::default();
    assert_eq!(
        cmd(&mut e, "vip", "", &mut h),
        CommandOutcome::NoPermission(Some("&cNo VIP for you".into()))
    );
    h.perms = true;
    assert_eq!(cmd(&mut e, "vip", "", &mut h), CommandOutcome::Ran);
}

// ---------- effects ----------

#[test]
fn message_effect_and_targets() {
    let mut h = Mock::default();
    join(
        "on join:\n    message \"hi %player%\"\n    message \"to console\" to console\n    message \"x\" to player\n    send \"all\" to all players\n    message \"all2\" to all players\n",
        &mut h,
    );
    assert_eq!(
        h.take(),
        [
            "send Steve hi Steve",
            "console-log to console",
            "send Steve x",
            "broadcast all",
            "broadcast all2"
        ]
    );
}

#[test]
fn player_and_console_commands() {
    let mut h = Mock::default();
    join(
        "on join:\n    player command \"/home\"\n    execute player command \"warp a\"\n    console command \"/say hi\"\n    execute console command \"say two\"\n    make player execute command \"/back\"\n    make player run command \"spawn\"\n    command \"/list\" by console\n    execute command \"/me waves\"\n",
        &mut h,
    );
    assert_eq!(
        h.take(),
        [
            "cmd-as Steve home",
            "cmd-as Steve warp a",
            "cmd say hi",
            "cmd say two",
            "cmd-as Steve back",
            "cmd-as Steve spawn",
            "cmd list",
            "cmd me waves"
        ]
    );
}

#[test]
fn clear_deletes_a_list_variable() {
    let mut h = Mock::default();
    join(
        "on join:\n    set {k.%uuid of player%::*} to 1, 2 and 3\n    clear {k.%uuid of player%::*}\n    send \"%size of {k.%uuid of player%::*}%\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["send Steve 0"]);
}

#[test]
fn teleport_at_a_location() {
    assert!(errors_of("on join:\n    teleport player at location(1, 2, 3)\n").is_empty());
}

// ---------- control flow ----------

#[test]
fn exit_trigger_and_sections() {
    let mut h = Mock::default();
    join(
        "on join:\n    send \"a\"\n    exit trigger\n    send \"never\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["send Steve a"]);
    join(
        "on join:\n    loop 3 times:\n        loop 3 times:\n            send \"in %loop-number%\"\n            exit 2 loops\n        send \"inner-after\"\n    send \"after\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["send Steve in 1", "send Steve after"]);
    join(
        "on join:\n    if 1 is 1:\n        if 2 is 2:\n            send \"deep\"\n            exit 2 sections\n        send \"skipped\"\n    send \"out\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["send Steve deep", "send Steve out"]);
    join(
        "on join:\n    loop 3 times:\n        if loop-number is 2:\n            exit 1 section\n        send \"n%loop-number%\"\n    send \"done\"\n",
        &mut h,
    );
    // `exit 1 section` leaves only the `if`, so the loop goes on.
    assert_eq!(
        h.take(),
        [
            "send Steve n1",
            "send Steve n2",
            "send Steve n3",
            "send Steve done"
        ]
    );
    join(
        "on join:\n    loop 3 times:\n        exit 1 section\n        send \"never\"\n    send \"done\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["send Steve done"]);
    join(
        "on join:\n    if 1 is 1:\n        exit all sections\n    send \"never\"\n",
        &mut h,
    );
    assert_eq!(h.take(), Vec::<String>::new());
}

#[test]
fn condition_section_without_if() {
    let mut h = Mock::default();
    join(
        "on join:\n    {_x} is not set:\n        send \"unset\"\n    1 is 1:\n        send \"one\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["send Steve unset", "send Steve one"]);
}

// ---------- expressions ----------

#[test]
fn random_integer_from_to() {
    let mut h = Mock::default();
    join(
        "on join:\n    loop 30 times:\n        set {_x} to random integer from 3 to 5\n        if {_x} is not between 3 and 5:\n            send \"bad\"\n    send \"ok\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["send Steve ok"]);
}

#[test]
fn mod_operator_and_rounded() {
    let mut h = Mock::default();
    join(
        "on join:\n    set {_a} to 7\n    set {_b} to 3\n    if {_a} mod {_b} = 1:\n        send \"mod\"\n    send \"%rounded 2.6%\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["send Steve mod", "send Steve 3"]);
}

#[test]
fn was_less_than_ago_and_difference() {
    let mut h = Mock {
        time_ms: 1_000_000_000,
        ..Mock::default()
    };
    join(
        "on join:\n    set {_when} to now\n    if {_when} was less than a minute ago:\n        send \"recent\"\n    if {_when} was more than 5 seconds ago:\n        send \"old\"\n    if difference between now and {_when} is less than 20 seconds:\n        send \"close\"\n    if difference between {_when} and now is greater than 1 hour:\n        send \"far\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["send Steve recent", "send Steve close"]);
}

#[test]
fn permission_forms() {
    let mut h = Mock::default();
    join(
        "on join:\n    if player doesn't have permission \"a.b\":\n        send \"no\"\n    player does not have permission \"a.c\"\n    send \"guard passed\"\n    if player have permission \"a.b\":\n        send \"has\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["send Steve no", "send Steve guard passed"]);
    h.perms = true;
    join(
        "on join:\n    if player have permission \"a.b\":\n        send \"has\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["send Steve has"]);
}

#[test]
fn world_comparisons_and_type_checks() {
    assert!(errors_of(
        "on join:\n    world is \"world\"\n    if player's world is \"world\" or \"world_nether\":\n        stop\n    if world is not \"x\":\n        stop\n"
    )
    .is_empty());
    let mut h = Mock::default();
    join(
        "on join:\n    set {_l::*} to 1, \"a\" and location(1, 2, 3)\n    loop {_l::*}:\n        if loop-value is a number:\n            send \"num\"\n        else if loop-value is text:\n            send \"text\"\n        else if loop-value is location:\n            send \"loc\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["send Steve num", "send Steve text", "send Steve loc"]);
}

#[test]
fn contains_any_of_and_not_set_forms() {
    let mut h = Mock::default();
    join(
        "on join:\n    set {_t} to \"hello world\"\n    if {_t} contains \"zzz\" or \"world\":\n        send \"any\"\n    if {_a} and {_b} are not set:\n        send \"both unset\"\n    set {_a} to 1\n    set {_b} to 2\n    if {_a} and {_b} are set:\n        send \"both set\"\n    if {_c} not set:\n        send \"c unset\"\n    set {_l::*} to 1 and 2\n    if {_l::*} do not contain 3:\n        send \"no 3\"\n",
        &mut h,
    );
    assert_eq!(
        h.take(),
        [
            "send Steve any",
            "send Steve both unset",
            "send Steve both set",
            "send Steve c unset",
            "send Steve no 3"
        ]
    );
}

#[test]
fn alternatives_of_names_that_are_not_values() {
    assert!(errors_of(
        "on damage:\n    if victim is not warden or wither or ender dragon:\n        stop\non break:\n    if event-block is not grass or tall grass or dead bush or sugar cane:\n        stop\n"
    )
    .is_empty());
}

// ---------- loops ----------

#[test]
fn loop_all_offline_players() {
    let mut h = Mock {
        offline: vec![PlayerRef {
            uuid: "u9".into(),
            name: "Gone".into(),
        }],
        ..Mock::default()
    };
    join(
        "on join:\n    loop all offline players:\n        send \"%name of loop-offline player% %loop-offlineplayer's uuid%\"\n",
        &mut h,
    );
    let out = h.take();
    assert!(out.contains(&"send Steve Gone u9".to_string()), "{out:?}");
    assert!(out.contains(&"send Steve Steve u1".to_string()), "{out:?}");
}

#[test]
fn loop_value_aliases_by_type() {
    assert!(errors_of(
        "on join:\n    loop blocks in radius 2 around player:\n        if loop-block is air:\n            stop\n    loop all players within 25 meters of player:\n        send \"%loop-player%\"\n"
    )
    .is_empty());
}

#[test]
fn loop_entities_in_a_world() {
    assert!(errors_of(
        "on join:\n    loop all entities in player's world:\n        send \"%loop-entity's type%\"\n    loop all zombies in world \"world\":\n        stop\n    set {_n} to amount of entities in radius 5 of player\n"
    )
    .is_empty());
}

// ---------- items and slots ----------

#[test]
fn set_slots_lists_and_ranges() {
    let mut h = Mock::default();
    join(
        "on join:\n    set {_g} to chest inventory with 3 rows named \"x\"\n    set slots 0, 1 and 2 of {_g} to stone named \"A\" with lore \"l1\", \"l2\"\n    set slot (integers between 3 and 5) of {_g} to dirt\n    set slot 8 of {_g} to gold block of mending named \"M\"\n    send \"%slot 1 of {_g}%|%slot 4 of {_g}%|%slot 8 of {_g}%|%slot 6 of {_g}%\"\n",
        &mut h,
    );
    let out = h.take();
    assert_eq!(out.len(), 1, "{out:?}");
    assert!(out[0].contains("stone"), "{out:?}");
    assert!(out[0].contains("dirt"), "{out:?}");
    assert!(
        out[0].contains("gold_block") || out[0].contains("gold block"),
        "{out:?}"
    );
}

#[test]
fn item_names_scripts_use() {
    assert!(errors_of(
        "on join:\n    set {_a} to dirt block named \"x\"\n    set {_b} to wheat seed named \"x\"\n    set {_c} to endstone named \"x\"\n    set {_d} to bottle o' enchanting named \"x\"\n    set {_e} to player's head named \"x\"\n    set {_f} to redstone dust named \"x\"\n    set {_g} to gold pickaxe\n    set {_h} to grass block named \"x\" with all flags hidden\n    set {_i} to diamond sword of sharpness 3 and unbreaking named \"x\"\n    set {_j::*} to \"\", lever, hopper, stone button and \"\"\n"
    )
    .is_empty());
}

#[test]
fn lore_on_a_loop_value() {
    assert!(errors_of(
        "on join:\n    set {_g} to chest inventory with 3 rows named \"x\"\n    set {_l::*} to stone and dirt\n    loop {_l::*}:\n        set slot loop-index parsed as number of {_g} to loop-value with lore \"x\"\n"
    )
    .is_empty());
}

#[test]
fn held_item_delete_and_lore_line() {
    let mut h = Mock {
        held: Some(("stone".into(), 3)),
        ..Mock::default()
    };
    join(
        "on join:\n    player is holding anything\n    delete player's held item\n    send \"gone\"\n    player is holding anything\n    send \"never\"\n",
        &mut h,
    );
    assert_eq!(h.take(), ["clear-held Steve", "send Steve gone"]);
    assert!(errors_of(
        "on join:\n    set 1st line of (player's held item)'s lore to \"x\"\n    set line 2 of player's tool's lore to \"y\"\n"
    )
    .is_empty());
}

#[test]
fn remove_all_of_an_item_from_a_player() {
    assert!(errors_of("on join:\n    remove all armor stand from the player\n").is_empty());
}

// ---------- worlds, time and events ----------

#[test]
fn world_time_possessive_and_events() {
    assert!(errors_of(
        "every 8 seconds:\n    time in \"world\" is between 06:30 and 17:30\n    remove 4 seconds from \"world\"'s time\n    set {_t} to \"world\"'s time\non damage of a zombie or a skeleton:\n    stop\non rightclick on chest or barrel or a bed:\n    stop\non food bar change:\n    stop\non walking on stone:\n    stop\non walking on (red terracotta, lime terracotta or blue terracotta):\n    stop\n"
    )
    .is_empty());
}

#[test]
fn mob_spawner_arguments_and_potion_effect_lists() {
    assert!(errors_of(
        "on rightclick on a mob spawner:\n    stop\non command:\n    command is \"mv\" or \"mvm\"\n    arguments is \"set spawn\" or \"setspawn\"\n    send \"%arguments%\"\non join:\n    add potion effect of speed 3 without particles for 5 seconds to potion effects of event-player\n    remove speed from potion effects of event-player\n"
    )
    .is_empty());
}

#[test]
fn event_world_in_events_without_a_player() {
    assert!(errors_of(
        "on explode:\n    world is \"world\"\n    if event-world is not \"x\":\n        stop\non redstone change:\n    if event-block's world is \"x\":\n        stop\non spawn of a skeleton:\n    world is \"world\" or \"world_nether\"\n    send \"%event-world%\" to console\n"
    )
    .is_empty());
    // `every` has no place, so no world.
    assert!(!errors_of("every 5 seconds:\n    send \"%event-world%\" to console\n").is_empty());
}

#[test]
fn event_world_reads_the_event_place_or_the_main_world() {
    let mut h = Mock::default();
    let mut e = engine(
        "on explode:\n    send \"w=%event-world%\" to console\non spawn of a zombie:\n    send \"s=%event-world% %event-entity%\" to console\n",
    );
    e.fire(
        EventKind::Game(carve_core::events_game::Ev::Explode),
        EventCtx::console(),
        &mut h,
    );
    let extra = EventExtra {
        entity: Some("zombie".into()),
        entity_id: Some(5),
        location: Some(EvPos {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            world: Some("nether".into()),
        }),
        ..Default::default()
    };
    e.fire(EventKind::MobSpawn, EventCtx::console_with_extra(extra), &mut h);
    assert_eq!(h.take(), ["send console w=world", "send console s=nether zombie"]);
}

#[test]
fn where_filter_with_distance_between() {
    assert!(errors_of(
        "on redstone change:\n    loop all players where [distance between input's location and event-location is smaller than 1]:\n        stop\n"
    )
    .is_empty());
}

#[test]
fn spawn_entity_and_vehicle() {
    assert!(errors_of(
        "on spawn of a skeleton:\n    wait 1 tick\n    vehicle of the entity is a skeleton horse\n    kill the entity's vehicle\n    kill the entity\n    send \"%event-entity% %the entity%\"\n"
    )
    .is_empty());
}

#[test]
fn rightclick_on_an_entity() {
    let (e, errs) = load(
        "on rightclick on a creature:\n    send \"any\" to player\non rightclick on a zombie or a villager:\n    send \"z\" to player\non rightclick on a chest:\n    send \"chest\" to player\n",
    );
    assert!(errs.is_empty(), "{errs:?}");
    let _ = e;
}

#[test]
fn item_lore_line_in_drop_is_read_with_a_warning() {
    let (e, errs) = load("on drop:\n    set the 1st line of the item's lore to \"x\"\n");
    assert!(errs.is_empty(), "{errs:?}");
    assert!(
        e.warnings.iter().any(|w| w.msg.contains("does nothing")),
        "{:?}",
        e.warnings
    );
}

#[test]
fn tab_complete_of_command_is_read_but_does_nothing() {
    let (e, errs) = load("on tab complete of \"/x\":\n    stop\n");
    assert!(errs.is_empty(), "{errs:?}");
    assert!(
        e.warnings.iter().any(|w| w.msg.contains("does nothing")),
        "{:?}",
        e.warnings
    );
}

#[test]
fn location_at_in_world_and_coordinates() {
    assert!(errors_of(
        "on join:\n    set {_b} to block at location at (1, 2, 3) in world player's world\n    set {_c} to \"%player's x-coordinate% %rounded player's y-coordinate%\"\n    reload skript \"x\"\n"
    )
    .is_empty());
}

#[test]
fn particle_draw_with_vector_offset() {
    assert!(errors_of(
        "on join:\n    draw 100 cloud at location 0.75 meter above player with offset vector(0.65, 0.5, 0.65) with extra 0\n"
    )
    .is_empty());
}

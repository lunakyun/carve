use std::collections::HashMap;

use carve_core::compile::{EventKind, check_registry, compile_file};
use carve_core::*;

/// Records commands and board draws.
#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    online: HashMap<String, String>,
    logs: Vec<(LogLevel, String)>,
}

impl Mock {
    fn with_steve() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m.online.insert("u2".into(), "Alex".into());
        m
    }
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
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        self.online.get(uuid).cloned()
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        self.online
            .iter()
            .find(|(_, n)| n.eq_ignore_ascii_case(name))
            .map(|(u, n)| PlayerRef {
                uuid: u.clone(),
                name: n.clone(),
            })
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
    fn board_show(&mut self, p: &PlayerRef, title: &str, lines: &[String]) {
        self.calls
            .push(format!("board {} [{title}] {}", p.name, lines.join("|")));
    }
    fn board_clear(&mut self, p: &PlayerRef) {
        self.calls.push(format!("board-clear {}", p.name));
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
    let (_, errs) = e.load(&[("test.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "unexpected errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn run_with(src: &str, h: &mut Mock) -> Vec<String> {
    let mut e = engine(src);
    e.fire(EventKind::Join, EventCtx::player(steve()), h);
    let errs: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errs.is_empty(), "runtime errors: {errs:?}");
    h.take()
}

/// Body lines, indented under `on join:`.
fn join(body: &str) -> Vec<String> {
    let src = format!(
        "on join:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    );
    run_with(&src, &mut Mock::with_steve())
}

/// Runs the script and returns the runtime error, if any.
fn join_err(body: &str) -> Option<String> {
    let src = format!(
        "on join:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    );
    let mut h = Mock::with_steve();
    let mut e = engine(&src);
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    h.logs
        .iter()
        .find(|(l, _)| *l == LogLevel::Error)
        .map(|(_, m)| m.clone())
}

fn compiles(line: &str) -> bool {
    compile_file("t.carve", &format!("on join:\n    {line}\n")).is_ok()
}

#[test]
fn registry_still_valid() {
    check_registry().unwrap();
}

// ---------- objectives ----------

#[test]
fn register_objective_defaults_to_dummy() {
    let out = join("register objective \"kills\"");
    assert_eq!(out, ["cmd scoreboard objectives add kills dummy"]);
}

#[test]
fn register_objective_with_criteria_and_display_name() {
    let out = join("register a new objective \"kills\" with criteria \"playerKillCount\" named \"&cKills\"");
    assert_eq!(
        out,
        ["cmd scoreboard objectives add kills playerKillCount {\"text\":\"\u{a7}cKills\"}"]
    );
}

#[test]
fn register_objective_display_name_marker_order_and_wording() {
    let out = join("create new objective named \"pts\" with display name \"Points\" with criteria \"dummy\"");
    assert_eq!(
        out,
        ["cmd scoreboard objectives add pts dummy {\"text\":\"Points\"}"]
    );
}

#[test]
fn register_objective_twice_is_an_error() {
    let e = join_err("register objective \"a\"\nregister objective \"a\"").unwrap();
    assert!(e.contains("already registered"), "{e}");
}

#[test]
fn bad_objective_name_is_an_error() {
    let e = join_err("register objective \"two words\"").unwrap();
    assert!(e.contains("not a valid objective name"), "{e}");
}

#[test]
fn unregister_objective_drops_scores() {
    let out = join(
        "register objective \"k\"\nset score of \"Bob\" in objective \"k\" to 4\nunregister objective \"k\"\nsend \"%score of \"\"Bob\"\" in \"\"k\"\"%\"",
    );
    assert_eq!(
        out,
        [
            "cmd scoreboard objectives add k dummy",
            "cmd scoreboard players set Bob k 4",
            "cmd scoreboard objectives remove k",
            "send Steve <none>",
        ]
    );
}

#[test]
fn objective_exists_condition() {
    let out = join(
        "if objective \"a\" doesn't exist:\n    send \"no\"\nregister objective \"a\"\nif objective \"a\" exists:\n    send \"yes\"\nif objective \"a\" does not exist:\n    send \"never\"",
    );
    assert_eq!(
        out,
        [
            "send Steve no",
            "cmd scoreboard objectives add a dummy",
            "send Steve yes"
        ]
    );
}

#[test]
fn objective_info_expressions() {
    let out = join(
        "register objective \"a\" with criteria \"dummy\" named \"Alpha\"\nsend objective criteria of \"a\"\nsend objective display name of \"a\"\nsend criteria of objective \"a\"",
    );
    assert_eq!(
        out,
        [
            "cmd scoreboard objectives add a dummy {\"text\":\"Alpha\"}",
            "send Steve dummy",
            "send Steve Alpha",
            "send Steve dummy",
        ]
    );
}

// ---------- scores ----------

#[test]
fn set_add_remove_score_of_player() {
    let out = join(
        "register objective \"k\"\nset score of player in objective \"k\" to 5\nadd 3 to score of player in \"k\"\nremove 10 from the score of player in \"k\"\nsend \"%score of player in \"\"k\"\"%\"",
    );
    assert_eq!(
        out,
        [
            "cmd scoreboard objectives add k dummy",
            "cmd scoreboard players set Steve k 5",
            "cmd scoreboard players set Steve k 8",
            "cmd scoreboard players set Steve k -2",
            "send Steve -2",
        ]
    );
}

#[test]
fn score_of_unset_is_none_and_default_works() {
    let out = join("register objective \"k\"\nset {_a} to score of player in \"k\" ? 7\nsend \"%{_a}%\"");
    assert_eq!(out, ["cmd scoreboard objectives add k dummy", "send Steve 7"]);
}

#[test]
fn score_can_be_used_in_math_and_conditions() {
    let out = join(
        "register objective \"k\"\nset score of \"Bob\" in \"k\" to 4\nif score of \"Bob\" in \"k\" > 3:\n    send \"big\"\nset {_x} to score of \"Bob\" in \"k\" + 1\nsend \"%{_x}%\"",
    );
    assert_eq!(
        out,
        [
            "cmd scoreboard objectives add k dummy",
            "cmd scoreboard players set Bob k 4",
            "send Steve big",
            "send Steve 5",
        ]
    );
}

#[test]
fn score_rounds_and_clamps() {
    let out = join("set score of player in \"k\" to 2.6\nset score of player in \"k\" to 99999999999");
    assert!(
        out.contains(&"cmd scoreboard players set Steve k 3".to_string()),
        "{out:?}"
    );
    assert!(
        out.contains(&"cmd scoreboard players set Steve k 2147483647".to_string()),
        "{out:?}"
    );
}

#[test]
fn unregistered_objective_warns_but_still_runs() {
    let mut h = Mock::with_steve();
    let out = run_with("on join:\n    set score of player in \"ext\" to 1\n", &mut h);
    assert_eq!(out, ["cmd scoreboard players set Steve ext 1"]);
    assert!(
        h.logs
            .iter()
            .any(|(l, m)| *l == LogLevel::Warn && m.contains("ext"))
    );
}

#[test]
fn reset_score_one_and_all() {
    let out = join(
        "register objective \"a\"\nregister objective \"b\"\nset score of player in \"a\" to 1\nset score of player in \"b\" to 2\nreset score of player in \"a\"\nsend \"%score of player in \"\"a\"\"%|%score of player in \"\"b\"\"%\"\nreset all scores of player\nsend \"%score of player in \"\"b\"\"%\"",
    );
    assert_eq!(
        &out[4..],
        [
            "cmd scoreboard players reset Steve a",
            "send Steve <none>|2",
            "cmd scoreboard players reset Steve",
            "send Steve <none>",
        ]
    );
}

#[test]
fn scores_with_text_holder_with_spaces_are_refused() {
    let e = join_err("set score of \"a b\" in \"k\" to 1").unwrap();
    assert!(e.contains("can't be used"), "{e}");
}

#[test]
fn scores_survive_a_restart_through_the_variables() {
    let mut h = Mock::with_steve();
    let src = "on join:\n    register objective \"k\"\n    set score of player in \"k\" to 9\n";
    let mut e = engine(src);
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let vars = e.vars.clone();
    let mut e2 = Engine::new(vars);
    let (_, errs) = e2.load(&[(
        "t.carve".into(),
        "on join:\n    send \"%score of player in \"\"k\"\"%\"\n".into(),
    )]);
    assert!(errs.is_empty());
    h.take();
    e2.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.take(), ["send Steve 9"]);
}

// ---------- teams ----------

#[test]
fn register_and_unregister_team() {
    let out = join(
        "register a new team \"red\"\nif team \"red\" exists:\n    send \"yes\"\nunregister team \"red\"\nif team \"red\" does not exist:\n    send \"gone\"",
    );
    assert_eq!(
        out,
        [
            "cmd team add red",
            "send Steve yes",
            "cmd team remove red",
            "send Steve gone"
        ]
    );
}

#[test]
fn joining_an_unknown_team_is_an_error() {
    let e = join_err("add player to team \"nope\"").unwrap();
    assert!(e.contains("not registered"), "{e}");
}

#[test]
fn join_leave_and_team_of() {
    let out = join(
        "register team \"red\"\nadd player to team \"red\"\nsend \"%team of player%\"\nif player is in team \"red\":\n    send \"in\"\nremove player from team \"red\"\nsend \"%team of player%\"\nif player is not in team \"red\":\n    send \"out\"\nif player isn't in team \"red\":\n    send \"out2\"",
    );
    assert_eq!(
        out,
        [
            "cmd team add red",
            "cmd team join red Steve",
            "send Steve red",
            "send Steve in",
            "cmd team leave Steve",
            "send Steve <none>",
            "send Steve out",
            "send Steve out2",
        ]
    );
}

#[test]
fn joining_another_team_leaves_the_old_one() {
    let out = join(
        "register team \"a\"\nregister team \"b\"\nadd player to team \"a\"\nadd player to team \"b\"\nsend \"%team size of \"\"a\"\"%\"\nsend \"%team of player%\"",
    );
    assert_eq!(&out[out.len() - 2..], ["send Steve 0", "send Steve b"]);
}

#[test]
fn members_and_size() {
    let out = join(
        "register team \"red\"\nadd player to team \"red\"\nadd \"Alex\" to team \"red\"\nsend \"%team size of \"\"red\"\"%\"\nset {_m::*} to members of team \"red\"\nsend \"%size of {_m::*}%\"\nloop members of team \"red\":\n    send \"m %loop-value%\"\nclear team \"red\"\nsend \"%team size of \"\"red\"\"%\"",
    );
    assert!(out.contains(&"send Steve 2".to_string()), "{out:?}");
    assert!(out.contains(&"send Steve m Alex".to_string()), "{out:?}");
    assert!(out.contains(&"send Steve m Steve".to_string()), "{out:?}");
    assert!(out.contains(&"cmd team empty red".to_string()));
    assert_eq!(out.last().unwrap(), "send Steve 0");
}

#[test]
fn removing_a_team_forgets_members_and_settings() {
    let out = join(
        "register team \"red\"\nset team prefix of team \"red\" to \"[R] \"\nadd player to team \"red\"\nunregister team \"red\"\nsend \"%team of player%\"\nregister team \"red\"\nsend \"%prefix of team \"\"red\"\"%|\"",
    );
    assert_eq!(out[out.len() - 3], "send Steve <none>");
    assert_eq!(out[out.len() - 1], "send Steve |");
}

#[test]
fn team_settings_run_team_modify() {
    let out = join(
        "register team \"red\"\nset prefix of team \"red\" to \"&c[R] \"\nset team suffix of \"red\" to \" &7!\"\nset display name of team \"red\" to \"Reds\"\nset color of team \"red\" to dark red\nset friendly fire of team \"red\" to false\nset see friendly invisibles of team \"red\" to true\nset nametag visibility of team \"red\" to hide for other teams\nset collision rule of team \"red\" to push own team",
    );
    assert_eq!(
        out,
        [
            "cmd team add red",
            "cmd team modify red prefix {\"text\":\"\u{a7}c[R] \"}",
            "cmd team modify red suffix {\"text\":\" \u{a7}r\u{a7}7!\"}",
            "cmd team modify red displayName {\"text\":\"Reds\"}",
            "cmd team modify red color dark_red",
            "cmd team modify red friendlyFire false",
            "cmd team modify red seeFriendlyInvisibles true",
            "cmd team modify red nametagVisibility hideForOtherTeams",
            "cmd team modify red collisionRule pushOwnTeam",
        ]
    );
}

#[test]
fn team_settings_read_back() {
    let out = join(
        "register team \"red\"\nsend \"%prefix of team \"\"red\"\"%|%team friendly fire of \"\"red\"\"%|%team color of \"\"red\"\"%|%team display name of \"\"red\"\"%\"\nset color of team \"red\" to \"light purple\"\nset friendly fire of team \"red\" to false\nset team nametag visibility of \"red\" to never\nsend \"%team color of \"\"red\"\"%|%team friendly fire of \"\"red\"\"%|%team nametag visibility of \"\"red\"\"%\"",
    );
    let n = out.len();
    assert_eq!(out[1], "send Steve |true|reset|red");
    assert_eq!(out[n - 1], "send Steve light purple|false|never");
}

#[test]
fn color_names_and_reset() {
    let out = join(
        "register team \"t\"\nset color of team \"t\" to \"Pink\"\nset color of team \"t\" to reset\nset color of team \"t\" to \"dark-grey\"",
    );
    assert_eq!(
        &out[1..],
        [
            "cmd team modify t color light_purple",
            "cmd team modify t color reset",
            "cmd team modify t color dark_gray",
        ]
    );
}

#[test]
fn bad_setting_values_are_errors() {
    let e = join_err("register team \"t\"\nset color of team \"t\" to \"plaid\"").unwrap();
    assert!(e.contains("not a team color"), "{e}");
    let e = join_err("register team \"t\"\nset collision rule of team \"t\" to \"sometimes\"").unwrap();
    assert!(e.contains("collision rule"), "{e}");
    let e = join_err("set prefix of team \"ghost\" to \"x\"").unwrap();
    assert!(e.contains("not registered"), "{e}");
}

#[test]
fn display_name_of_non_team_still_means_the_old_effect() {
    // `set display name of player to ...` must keep working.
    let out = join("set display name of player to \"Boss\"");
    assert!(
        out.is_empty() || !out.iter().any(|l| l.starts_with("cmd team")),
        "{out:?}"
    );
    assert!(compiles("set display name of player to \"Boss\""));
}

// ---------- boards ----------

#[test]
fn board_title_and_lines_draw_the_whole_board() {
    let out = join(
        "set title of board of player to \"&6Stats\"\nset line 1 of board of player to \"Kills: 3\"\nset line 3 of board of player to \"Deaths: 1\"",
    );
    assert_eq!(
        out,
        [
            "board Steve [&6Stats] ",
            "board Steve [&6Stats] Kills: 3",
            "board Steve [&6Stats] Kills: 3||Deaths: 1",
        ]
    );
}

#[test]
fn board_all_lines_at_once_and_possessive() {
    let out = join(
        "set title of player's board to \"T\"\nset lines of board of player to \"a\", \"b\" and \"c\"\ndelete line 2 of board of player",
    );
    assert_eq!(
        out,
        [
            "board Steve [T] ",
            "board Steve [T] a|b|c",
            "board Steve [T] a||c"
        ]
    );
}

#[test]
fn board_without_a_title_gets_a_blank_one_and_clear_removes_it() {
    let out = join(
        "set line 1 of the board of player to \"x\"\ndelete board of player\nset line 1 of board of player to \"y\"",
    );
    assert_eq!(
        out,
        ["board Steve [ ] x", "board-clear Steve", "board Steve [ ] y"]
    );
}

#[test]
fn board_line_range_is_checked() {
    let e = join_err("set line 16 of board of player to \"x\"").unwrap();
    assert!(e.contains("1 to 15"), "{e}");
    let e = join_err("set line 0 of board of player to \"x\"").unwrap();
    assert!(e.contains("1 to 15"), "{e}");
}

#[test]
fn board_too_many_lines_is_an_error() {
    let lines = (1..=16)
        .map(|i| format!("\"l{i}\""))
        .collect::<Vec<_>>()
        .join(", ");
    let e = join_err(&format!("set lines of board of player to {lines}")).unwrap();
    assert!(e.contains("at most 15"), "{e}");
}

#[test]
fn board_on_an_offline_player_does_nothing() {
    let out = join("set title of board of \"Ghost\" to \"x\"");
    assert!(out.is_empty(), "{out:?}");
}

// ---------- compile-time behavior ----------

#[test]
fn old_syntax_is_not_taken_over() {
    assert!(compiles("add 5 to {x}"));
    assert!(compiles("remove 5 from {x}"));
    assert!(compiles("set {x} to 5"));
    assert!(compiles("delete {x}"));
    assert!(compiles("send \"team\""));
    assert!(compiles("if {team} is 5:\n        send \"x\""));
    assert!(compiles("set {team} to \"x\""));
    assert!(compiles("add 1 to {score}"));
}

#[test]
fn bad_lines_fail_to_compile_with_a_normal_error() {
    assert!(!compiles("set score of player in to 5"));
    assert!(!compiles("register team"));
    assert!(!compiles("add player to team"));
}

#[test]
fn score_needs_in() {
    assert!(compile_file("t.carve", "on join:\n    send \"%score of player%\"\n").is_err());
}

#[test]
fn reset_team_of_player_leaves_the_team() {
    let out = join(
        "register team \"a\"
add player to team \"a\"
reset team of player
send \"%team of player%\"",
    );
    assert_eq!(&out[2..], ["cmd team leave Steve", "send Steve <none>"]);
}

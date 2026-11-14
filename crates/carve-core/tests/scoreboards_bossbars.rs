use std::collections::HashMap;

use carve_core::compile::{EventKind, compile_file};
use carve_core::scoreboards_bossbars::{BarColor, BarFlag, BarStyle};
use carve_core::*;

/// Records every scoreboard and boss bar call.
#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    online: HashMap<String, String>,
    logs: Vec<(LogLevel, String)>,
}

impl Mock {
    fn two() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m.online.insert("u2".into(), "Alex".into());
        m
    }
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, _text: &str) {}
    fn broadcast(&mut self, _text: &str) {}
    fn send_title(&mut self, p: &PlayerRef, t: &str, s: Option<&str>) {
        self.calls.push(format!("title {} {t} {}", p.name, s.unwrap_or("-")));
    }
    fn send_action_bar(&mut self, p: &PlayerRef, t: &str) {
        self.calls.push(format!("actionbar {} {t}", p.name));
    }
    fn heal(&mut self, _p: &PlayerRef) {}
    fn set_health(&mut self, _p: &PlayerRef, _h: f64) {}
    fn kill(&mut self, _p: &PlayerRef) {}
    fn teleport(&mut self, _p: &PlayerRef, _x: f64, _y: f64, _z: f64) {}
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
    fn has_permission(&mut self, _p: &PlayerRef, _n: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _t: u64, _k: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn online_players(&mut self) -> Vec<PlayerRef> {
        let mut v: Vec<_> = self
            .online
            .iter()
            .map(|(u, n)| PlayerRef {
                uuid: u.clone(),
                name: n.clone(),
            })
            .collect();
        v.sort_by(|a, b| a.name.cmp(&b.name));
        v
    }
    fn send_title_timed(&mut self, p: &PlayerRef, t: &str, s: Option<&str>, a: u32, b: u32, c: u32) {
        self.calls
            .push(format!("timed {} {t} {} {a}/{b}/{c}", p.name, s.unwrap_or("-")));
    }
    fn clear_title(&mut self, p: &PlayerRef, reset: bool) {
        self.calls.push(format!("cleartitle {} {reset}", p.name));
    }
    fn set_tab_list(&mut self, p: &PlayerRef, h: Option<&str>, f: Option<&str>) {
        self.calls
            .push(format!("tab {} {:?} {:?}", p.name, h, f));
    }
    fn sidebar_set_title(&mut self, p: &PlayerRef, t: &str) {
        self.calls.push(format!("side-title {} {t}", p.name));
    }
    fn sidebar_set_line(&mut self, p: &PlayerRef, n: u32, t: &str) {
        self.calls.push(format!("side-line {} {n} {t}", p.name));
    }
    fn sidebar_clear_line(&mut self, p: &PlayerRef, n: u32) {
        self.calls.push(format!("side-clear {} {n}", p.name));
    }
    fn sidebar_remove(&mut self, p: &PlayerRef) {
        self.calls.push(format!("side-remove {}", p.name));
    }
    fn bossbar_create(&mut self, id: &str, t: &str, c: BarColor, s: BarStyle, p: f64) {
        self.calls
            .push(format!("bar-create {id} {t} {} {} {p}", c.name(), s.name()));
    }
    fn bossbar_set_title(&mut self, id: &str, t: &str) {
        self.calls.push(format!("bar-title {id} {t}"));
    }
    fn bossbar_set_progress(&mut self, id: &str, f: f64) {
        self.calls.push(format!("bar-progress {id} {f}"));
    }
    fn bossbar_set_color(&mut self, id: &str, c: BarColor) {
        self.calls.push(format!("bar-color {id} {}", c.name()));
    }
    fn bossbar_set_style(&mut self, id: &str, s: BarStyle) {
        self.calls.push(format!("bar-style {id} {}", s.name()));
    }
    fn bossbar_set_flag(&mut self, id: &str, f: BarFlag, on: bool) {
        self.calls.push(format!("bar-flag {id} {f:?} {on}"));
    }
    fn bossbar_show(&mut self, id: &str, p: &PlayerRef) {
        self.calls.push(format!("bar-show {id} {}", p.name));
    }
    fn bossbar_hide(&mut self, id: &str, p: &PlayerRef) {
        self.calls.push(format!("bar-hide {id} {}", p.name));
    }
    fn bossbar_remove(&mut self, id: &str) {
        self.calls.push(format!("bar-remove {id}"));
    }
}

/// A host that implements only the required methods, so the trait defaults run.
#[derive(Default)]
struct Bare {
    cmds: Vec<String>,
    titles: Vec<String>,
}

impl Host for Bare {
    fn send_message(&mut self, _: &Recipient, _: &str) {}
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, t: &str, _: Option<&str>) {
        self.titles.push(t.to_string());
    }
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.cmds.push(c.to_string());
    }
    fn resolve_player(&mut self, _: &str) -> Option<String> {
        Some("Steve".into())
    }
    fn find_player(&mut self, _: &str) -> Option<PlayerRef> {
        None
    }
    fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, _: LogLevel, _: &str) {}
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn errors_of(src: &str) -> Vec<ScriptError> {
    compile_file("bad.carve", &script(src)).err().unwrap_or_default()
}

fn script(body: &str) -> String {
    let lines: String = body.lines().map(|l| format!("    {l}\n")).collect();
    format!("on join:\n{lines}")
}

fn load(body: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("test.carve".into(), script(body))]);
    assert!(
        errs.is_empty(),
        "errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn run_logs(body: &str) -> Mock {
    let mut e = load(body);
    let mut h = Mock::two();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    h
}

fn run(body: &str) -> Vec<String> {
    let h = run_logs(body);
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
    h.calls
}

fn run_bare(body: &str) -> Bare {
    let mut e = load(body);
    let mut h = Bare::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    h
}

#[test]
fn registry_still_ok() {
    carve_core::compile::check_registry().unwrap();
}

// ----- sidebar -----

#[test]
fn sidebar_title_and_lines() {
    assert_eq!(
        run("set title of player's scoreboard to \"&6Stats\""),
        ["side-title Steve &6Stats"]
    );
    assert_eq!(
        run("set line 1 of player's scoreboard to \"Coins: 5\""),
        ["side-line Steve 1 Coins: 5"]
    );
    assert_eq!(
        run("set line 3 of scoreboard of player to \"x\""),
        ["side-line Steve 3 x"]
    );
    assert_eq!(
        run("set title of scoreboard of player to \"T\""),
        ["side-title Steve T"]
    );
    assert_eq!(
        run("set player's scoreboard title to \"T2\""),
        ["side-title Steve T2"]
    );
    assert_eq!(
        run("set player's scoreboard line 2 to \"L2\""),
        ["side-line Steve 2 L2"]
    );
}

#[test]
fn sidebar_line_number_can_be_a_variable() {
    assert_eq!(
        run("set {_i} to 4\nset line {_i} of player's scoreboard to \"v\""),
        ["side-line Steve 4 v"]
    );
}

#[test]
fn sidebar_clear_and_remove() {
    assert_eq!(run("clear line 2 of player's scoreboard"), ["side-clear Steve 2"]);
    assert_eq!(run("clear player's scoreboard"), ["side-remove Steve"]);
    assert_eq!(run("clear scoreboard of player"), ["side-remove Steve"]);
    assert_eq!(run("hide player's sidebar"), ["side-remove Steve"]);
    assert_eq!(run("reset player's scoreboard"), ["side-remove Steve"]);
}

#[test]
fn sidebar_line_out_of_range_is_an_error() {
    let h = run_logs("set line 16 of player's scoreboard to \"x\"");
    assert!(h.calls.is_empty());
    assert!(
        h.logs
            .iter()
            .any(|(l, m)| *l == LogLevel::Error && m.contains("1 to 15")),
        "{:?}",
        h.logs
    );
    let h = run_logs("set line 0 of player's scoreboard to \"x\"");
    assert!(h.calls.is_empty());
}

#[test]
fn sidebar_bad_syntax() {
    let e = errors_of("set line of player's scoreboard to \"x\"");
    assert!(!e.is_empty());
    let e = errors_of("set colour of player's scoreboard to \"x\"");
    assert!(!e.is_empty());
}

// ----- boss bars -----

#[test]
fn bossbar_create_defaults() {
    assert_eq!(
        run("create bossbar \"boss\""),
        ["bar-create boss boss pink solid 1"]
    );
}

#[test]
fn bossbar_create_full() {
    assert_eq!(
        run("create a new bossbar named \"b1\" with title \"&cDragon\" with color red with style notched 10 with progress 50"),
        ["bar-create b1 &cDragon red notched 10 0.5"]
    );
    assert_eq!(
        run("create boss bar \"b2\" titled \"Hi\" colored blue styled solid"),
        ["bar-create b2 Hi blue solid 1"]
    );
    assert_eq!(
        run("create bossbar with id \"b3\" with title \"T\" and color green and style segmented 6"),
        ["bar-create b3 T green notched 6 1"]
    );
}

#[test]
fn bossbar_create_color_from_text() {
    assert_eq!(
        run("set {_c} to \"yellow\"\ncreate bossbar \"b\" with color {_c}"),
        ["bar-create b b yellow solid 1"]
    );
    let h = run_logs("set {_c} to \"mauve\"\ncreate bossbar \"b\" with color {_c}");
    assert!(h.calls.is_empty());
    assert!(h.logs.iter().any(|(_, m)| m.contains("unknown bossbar color 'mauve'")));
}

#[test]
fn bossbar_unknown_color_or_style_fails_to_compile() {
    let e = errors_of("create bossbar \"b\" with color mauve");
    assert!(e.iter().any(|e| e.to_string().contains("unknown bossbar color")), "{e:?}");
    let e = errors_of("create bossbar \"b\" with style wavy");
    assert!(e.iter().any(|e| e.to_string().contains("unknown bossbar style")), "{e:?}");
}

#[test]
fn bossbar_set_props() {
    assert_eq!(
        run("set progress of bossbar \"b\" to 25"),
        ["bar-progress b 0.25"]
    );
    assert_eq!(
        run("set progress of the boss bar \"b\" to 150"),
        ["bar-progress b 1"]
    );
    assert_eq!(run("set progress of bossbar \"b\" to -5"), ["bar-progress b 0"]);
    assert_eq!(
        run("set title of bossbar \"b\" to \"New\""),
        ["bar-title b New"]
    );
    assert_eq!(run("set color of bossbar \"b\" to purple"), ["bar-color b purple"]);
    assert_eq!(
        run("set style of bossbar \"b\" to notched 20"),
        ["bar-style b notched 20"]
    );
    assert_eq!(
        run("set bossbar \"b\"'s progress to 10"),
        ["bar-progress b 0.1"]
    );
}

#[test]
fn bossbar_progress_from_variable_math() {
    assert_eq!(
        run("set {_hp} to 3\nset progress of bossbar \"b\" to {_hp} * 10"),
        ["bar-progress b 0.3"]
    );
}

#[test]
fn bossbar_show_hide_remove() {
    assert_eq!(run("show bossbar \"b\" to player"), ["bar-show b Steve"]);
    assert_eq!(run("hide bossbar \"b\" from player"), ["bar-hide b Steve"]);
    assert_eq!(run("remove bossbar \"b\""), ["bar-remove b"]);
    assert_eq!(run("delete boss bar \"b\""), ["bar-remove b"]);
    assert_eq!(run("add player to bossbar \"b\""), ["bar-show b Steve"]);
    assert_eq!(run("remove player from bossbar \"b\""), ["bar-hide b Steve"]);
}

#[test]
fn bossbar_show_to_all_players() {
    assert_eq!(
        run("show bossbar \"b\" to all players"),
        ["bar-show b Alex", "bar-show b Steve"]
    );
    assert_eq!(
        run("hide bossbar \"b\" from everyone"),
        ["bar-hide b Alex", "bar-hide b Steve"]
    );
}

#[test]
fn bossbar_flags() {
    assert_eq!(
        run("add darken sky flag to bossbar \"b\""),
        ["bar-flag b DarkenSky true"]
    );
    assert_eq!(
        run("remove fog flag from bossbar \"b\""),
        ["bar-flag b Fog false"]
    );
    assert_eq!(
        run("add dragon bar flag to bossbar \"b\""),
        ["bar-flag b Dragon true"]
    );
}

#[test]
fn bossbar_id_from_variable() {
    assert_eq!(
        run("set {_n} to 1\nshow bossbar \"raid-%{_n}%\" to player"),
        ["bar-show raid-1 Steve"]
    );
}

#[test]
fn bossbar_bad_syntax() {
    assert!(!errors_of("create bossbar").is_empty());
    assert!(!errors_of("show bossbar \"b\"").is_empty());
    assert!(!errors_of("set flavor of bossbar \"b\" to 3").is_empty());
}

// ----- titles and action bar -----

#[test]
fn title_with_time() {
    assert_eq!(
        run("send title \"Hi\" to player for 3 seconds"),
        ["timed Steve Hi - 10/60/20"]
    );
    assert_eq!(
        run("send title \"a\" with subtitle \"b\" to player for 3 seconds"),
        ["timed Steve a b 10/60/20"]
    );
    assert_eq!(
        run("send title \"a\" with subtitle \"b\" to player for 1 second with fade in 5 ticks and fade out 2 seconds"),
        ["timed Steve a b 5/20/40"]
    );
}

#[test]
fn title_plain_still_works() {
    assert_eq!(run("send title \"Hi\" to player"), ["title Steve Hi -"]);
    assert_eq!(
        run("send title \"Hi\" with subtitle \"there\" to player"),
        ["title Steve Hi there"]
    );
}

#[test]
fn title_to_everyone() {
    assert_eq!(
        run("send title \"Hi\" to all players"),
        ["title Alex Hi -", "title Steve Hi -"]
    );
}

#[test]
fn title_default_uses_command() {
    let h = run_bare("send title \"Hi\" to player for 2 seconds");
    assert_eq!(h.cmds, ["title Steve times 10 40 20"]);
    assert_eq!(h.titles, ["Hi"]);
}

#[test]
fn title_clear_and_reset() {
    assert_eq!(run("clear title of player"), ["cleartitle Steve false"]);
    assert_eq!(run("reset title of player"), ["cleartitle Steve true"]);
    let h = run_bare("clear title of player");
    assert_eq!(h.cmds, ["title Steve clear"]);
    let h = run_bare("reset title of player");
    assert_eq!(h.cmds, ["title Steve reset"]);
}

#[test]
fn action_bar_forms() {
    assert_eq!(run("send action bar \"x\" to player"), ["actionbar Steve x"]);
    assert_eq!(run("send actionbar \"y\" to player"), ["actionbar Steve y"]);
    assert_eq!(
        run("send actionbar \"z\" to all players"),
        ["actionbar Alex z", "actionbar Steve z"]
    );
}

// ----- tab list -----

#[test]
fn tab_list_header_and_footer() {
    assert_eq!(
        run("set tab list header of player to \"H\""),
        ["tab Steve Some(\"H\") None"]
    );
    assert_eq!(
        run("set tablist footer of player to \"F\""),
        ["tab Steve None Some(\"F\")"]
    );
    assert_eq!(
        run("set player's tab list header to \"H2\""),
        ["tab Steve Some(\"H2\") None"]
    );
    assert_eq!(
        run("set tab list header and footer of player to \"B\""),
        ["tab Steve Some(\"B\") Some(\"B\")"]
    );
}

#[test]
fn tab_list_all_players_and_reset() {
    assert_eq!(
        run("set tab list header for all players to \"Welcome\""),
        ["tab Alex Some(\"Welcome\") None", "tab Steve Some(\"Welcome\") None"]
    );
    assert_eq!(
        run("reset tab list of player"),
        ["tab Steve Some(\"\") Some(\"\")"]
    );
    assert_eq!(
        run("clear tab list footer of player"),
        ["tab Steve None Some(\"\")"]
    );
}

#[test]
fn tab_list_bad_syntax() {
    assert!(!errors_of("set tab list header of player").is_empty());
}

// ----- the rest of Carve is unharmed -----

#[test]
fn words_in_strings_and_variables_are_not_taken() {
    assert_eq!(
        run("set {_s} to \"scoreboard bossbar\"\nset {scoreboard} to 1\nsend title \"Hi\" to player"),
        ["title Steve Hi -"]
    );
}

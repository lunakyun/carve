//! A user's CS2-style case opening script (`case_chest.carve`) end to end,
//! plus the pieces of it that used to fail silently.

use carve_core::compile::EventKind;
use carve_core::player_flags::PlayerFlag;
use carve_core::support::Feature;
use carve_core::*;

const CASE: &str = include_str!("case_chest.carve");

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
    waits: Vec<u64>,
    sneaking: bool,
    offline: bool,
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.calls.push(format!("msg {text}"));
    }
    fn broadcast(&mut self, text: &str) {
        self.calls.push(format!("broadcast {text}"));
    }
    fn send_title(&mut self, _p: &PlayerRef, _t: &str, _s: Option<&str>) {}
    fn send_action_bar(&mut self, _p: &PlayerRef, _t: &str) {}
    fn heal(&mut self, _p: &PlayerRef) {}
    fn set_health(&mut self, _p: &PlayerRef, _h: f64) {}
    fn kill(&mut self, _p: &PlayerRef) {}
    fn teleport(&mut self, _p: &PlayerRef, _x: f64, _y: f64, _z: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.calls.push(format!("cmd {c}"));
    }
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1" && !self.offline).then(|| "Steve".to_string())
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        (name.eq_ignore_ascii_case("steve") && !self.offline).then(steve)
    }
    fn has_permission(&mut self, _p: &PlayerRef, _n: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _t: u64, k: u64) {
        self.waits.push(k);
    }
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn player_flag(&mut self, _p: &PlayerRef, f: PlayerFlag) -> bool {
        matches!(f, PlayerFlag::Sneaking) && self.sneaking
    }
    fn player_location(&mut self, _p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((0.0, 64.0, 0.0, "overworld".into()))
    }
    fn play_sound(&mut self, _p: &PlayerRef, s: &str, v: f64, _pi: f64, _at: Option<(f64, f64, f64)>) {
        self.calls.push(format!("sound {s} {v}"));
    }
    fn give_item(&mut self, _p: &PlayerRef, id: &str, n: u32) {
        self.calls.push(format!("give {id} {n} -"));
    }
    fn give_item_meta(&mut self, _p: &PlayerRef, id: &str, n: u32, m: &item_meta::ItemMeta) {
        self.calls
            .push(format!("give {id} {n} {}", m.name.as_deref().unwrap_or("-")));
    }
    fn open_chest(&mut self, _p: &PlayerRef, rows: u8, title: &str) {
        self.calls.push(format!("open {rows} {title}"));
    }
    fn inv_set(&mut self, _p: &PlayerRef, c: bool, slot: i64, item: Option<(&str, u32)>) {
        self.calls.push(format!("slot {c} {slot} {item:?} -"));
    }
    fn inv_set_item(&mut self, _p: &PlayerRef, c: bool, slot: i64, item: &Value) {
        let (id, name) = match item {
            Value::Item { id, meta, .. } => (
                id.clone(),
                meta.as_ref().and_then(|m| m.name.clone()).unwrap_or("-".into()),
            ),
            _ => ("none".into(), "-".into()),
        };
        self.calls.push(format!("slot {c} {slot} {id} {name}"));
    }
    fn close_inventory(&mut self, _p: &PlayerRef) {
        self.calls.push("close".into());
    }
    fn strike_lightning(&mut self, _at: &world_effects::FxPos, effect: bool) {
        self.calls.push(format!("lightning {effect}"));
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn engine_with(src: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("case.carve".into(), src.to_string())]);
    assert!(errs.is_empty(), "{errs:?}");
    e
}

fn click(e: &mut Engine, h: &mut Mock, block: &str) -> bool {
    let mut ctx = EventCtx::player(steve());
    ctx.block = Some(block.into());
    e.fire(EventKind::RightClick, ctx, h).cancelled
}

/// Resume every wait until the script is done.
fn finish(e: &mut Engine, h: &mut Mock) {
    for _ in 0..200 {
        let Some(k) = h.waits.pop() else { return };
        e.resume(k, h);
    }
    panic!("the case never finished");
}

#[test]
fn case_opening_runs_to_the_end() {
    let mut e = engine_with(CASE);
    let mut h = Mock::default();
    assert!(click(&mut e, &mut h, "chest"), "cancel event");
    finish(&mut e, &mut h);
    let c = &h.calls;
    assert_eq!(c[0], "open 3 &6&lCase Opening");
    // The filler keeps its blank name, the markers theirs.
    assert!(
        c.contains(&"slot true 0 gray_stained_glass_pane  ".to_string()),
        "{c:?}"
    );
    assert!(c.contains(&"slot true 4 red_stained_glass_pane &c&lv".to_string()));
    // 31 spins of 9 reel slots, each a named item from dropFor().
    let reel: Vec<_> = c
        .iter()
        .filter(|s| {
            let n: i64 = s.split(' ').nth(2).and_then(|n| n.parse().ok()).unwrap_or(-1);
            s.starts_with("slot true ") && (9..=17).contains(&n)
        })
        .collect();
    // The gray filler first, then 31 spins.
    assert_eq!(reel.len(), 32 * 9);
    assert!(
        reel.iter().all(|s| !s.ends_with(" -") && !s.contains(" none ")),
        "{reel:?}"
    );
    assert_eq!(
        c.iter()
            .filter(|s| s.starts_with("sound ui.button.click 0.6"))
            .count(),
        31
    );
    // The winner (reel slot 35, seen in slot 13 at the last spin) is given with its name.
    let given = c.iter().find(|s| s.starts_with("give ")).expect("a prize");
    let last_mid = reel
        .iter()
        .rev()
        .find(|s| s.starts_with("slot true 13 "))
        .unwrap();
    let won = &last_mid["slot true 13 ".len()..];
    assert_eq!(*given, format!("give {}", won.replacen(' ', " 1 ", 1)));
    assert!(
        c.iter()
            .any(|s| s.starts_with("msg &8[&6Case&8]&r &7You unboxed: &f"))
    );
    assert_eq!(c.last().unwrap(), "close");
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l != LogLevel::Info).collect();
    assert!(errors.is_empty(), "{errors:?}");
}

#[test]
fn sneaking_skips_the_case() {
    let mut e = engine_with(CASE);
    let mut h = Mock {
        sneaking: true,
        ..Mock::default()
    };
    assert!(!click(&mut e, &mut h, "chest"));
    assert!(h.calls.is_empty(), "{:?}", h.calls);
    assert_eq!(e.vars.get("caseview::u1"), Value::None);
}

#[test]
fn second_click_while_spinning_stops_and_vars_delete_clears_it() {
    let mut e = engine_with(CASE);
    let mut h = Mock::default();
    click(&mut e, &mut h, "chest");
    assert_eq!(e.vars.get("caseview::u1"), Value::Bool(true));
    let n = h.calls.len();
    click(&mut e, &mut h, "chest");
    assert_eq!(h.calls.len(), n, "a second case while one is open");
    // `/carve vars delete caseview::*` reaches the list entries.
    e.vars.delete("caseview::*");
    assert_eq!(e.vars.get("caseview::u1"), Value::None);
}

#[test]
fn right_click_on_block_filter_ignores_case() {
    let mut e = engine_with("on right click on Chest:\n    send \"hit\" to player\n");
    let mut h = Mock::default();
    click(&mut e, &mut h, "chest");
    click(&mut e, &mut h, "barrel");
    assert_eq!(h.calls, ["msg hit"]);
}

#[test]
fn item_functions_and_list_index_keep_names() {
    let src = "function pane(r: number) :: itemtype:\n    return lime stained glass pane named \"&a%{_r}%\"\n\
               on join:\n    set {_l::1} to pane(1)\n    set {_l::2} to diamond named \"D\"\n    set {_i} to 1\n    \
               set {_j} to {_i} + 1\n    set slot 0 of player's current inventory to {_l::%{_i}%}\n    \
               set slot 1 of player's current inventory to {_l::%{_j}%}\n    set slot 2 of player's current inventory to stone named \" \"\n";
    let mut e = engine_with(src);
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(
        h.calls,
        [
            "slot true 0 lime_stained_glass_pane &a1",
            "slot true 1 diamond D",
            "slot true 2 stone  "
        ]
    );
}

#[test]
fn default_host_slot_drops_only_the_meta() {
    struct Plain(Vec<String>);
    impl Host for Plain {
        fn send_message(&mut self, _: &Recipient, _: &str) {}
        fn broadcast(&mut self, _: &str) {}
        fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
        fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
        fn heal(&mut self, _: &PlayerRef) {}
        fn set_health(&mut self, _: &PlayerRef, _: f64) {}
        fn kill(&mut self, _: &PlayerRef) {}
        fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
        fn execute_command(&mut self, _: &str) {}
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
        fn inv_set(&mut self, _: &PlayerRef, c: bool, slot: i64, item: Option<(&str, u32)>) {
            self.0.push(format!("{c} {slot} {item:?}"));
        }
    }
    let mut e = engine_with(
        "on join:\n    set slot 3 of player's current inventory to 2 diamonds named \"x\"\n    set slot 4 of player's current inventory to air\n",
    );
    let mut h = Plain(Vec::new());
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.0, ["true 3 Some((\"diamond\", 2))", "true 4 None"]);
}

#[test]
fn unsupported_syntax_warns_at_load() {
    let mut e = Engine::default();
    e.set_unsupported(vec![
        (Feature::CloseInventory, "no close call".into()),
        (
            Feature::PlayerFlag(PlayerFlag::Blocking),
            "no blocking state".into(),
        ),
    ]);
    let src = format!("{CASE}\non join:\n    if player is blocking:\n        stop\n");
    let blocking_line = src.lines().position(|l| l.contains("is blocking")).unwrap() + 1;
    let (_, errs) = e.load(&[("case.carve".into(), src)]);
    assert!(errs.is_empty(), "{errs:?}");
    let w: Vec<String> = e
        .warnings
        .iter()
        .map(|w| format!("{}:{}", w.line, w.msg))
        .collect();
    assert_eq!(
        w,
        [
            "127:'close inventory' does nothing on this server: no close call".to_string(),
            format!("{blocking_line}:'player is blocking' does nothing on this server: no blocking state"),
        ]
    );
    assert!(e.warnings.iter().all(|w| w.warning));
    assert_eq!(e.warnings[0].code, "close {_p}'s inventory");
    assert!(e.warnings[0].skript_lines()[0].starts_with("&6Line 127"));
    // Supported on this host: no warnings.
    let mut e = Engine::default();
    e.load(&[("case.carve".into(), CASE.to_string())]);
    assert!(e.warnings.is_empty());
}

#[test]
fn flag_of_offline_player_is_reported() {
    let mut e = engine_with(
        "on join:\n    set {_p} to \"Nobody\"\n    {_p} is not sneaking\n    send \"ran\" to console\n",
    );
    let mut h = Mock {
        offline: true,
        ..Mock::default()
    };
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert!(h.calls.is_empty(), "{:?} {:?}", h.calls, h.logs);
    assert!(
        h.logs.iter().any(|(l, m)| *l == LogLevel::Warn
            && m.contains("'is sneaking'")
            && m.contains("not an online player")),
        "{:?}",
        h.logs
    );
}

use carve_core::block_ops::BlockPos;
use carve_core::compile::{EventKind, compile_file};
use carve_core::w5_chunk6::{BlockState, Set, decode_state, encode_state};
use carve_core::w6_chunk4::{
    BANNER_PATTERN_ITEMS, Prop, banner_pattern_item, prop_max, prop_value, range, take_ends,
};
use carve_core::*;

const FIRST: u16 = 1000;

#[derive(Default)]
struct Mock {
    sent: Vec<String>,
    errors: Vec<String>,
    block: Option<BlockState>,
    set_states: Vec<u16>,
}

impl Mock {
    fn with(id: &str, prop: &str, v: i64) -> Mock {
        let s = encode_state(id, FIRST, FIRST, &[(prop.into(), v.to_string())]).unwrap();
        Mock {
            block: Some((id.into(), s, FIRST)),
            ..Mock::default()
        }
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
    fn world_names(&mut self) -> Vec<String> {
        vec!["world".into()]
    }
    fn c6_set(&mut self, s: Set<'_>) -> bool {
        match s {
            Set::BlockState(_, s) => {
                self.set_states.push(s);
                if let Some(b) = &mut self.block {
                    b.1 = s;
                }
                true
            }
            _ => false,
        }
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

fn compile_err(body: &str) -> String {
    match compile_file("t.carve", &script_for(body)) {
        Ok(_) => String::new(),
        Err(e) => e.iter().map(|x| x.to_string()).collect::<Vec<_>>().join("\n"),
    }
}

// ---- pure helpers -------------------------------------------------------------

#[test]
fn ends_and_ranges() {
    let l: Vec<Value> = (1..=5).map(|n| Value::Num(f64::from(n))).collect();
    assert_eq!(take_ends(&l, 2, true), l[..2].to_vec());
    assert_eq!(take_ends(&l, 2, false), l[3..].to_vec());
    assert_eq!(take_ends(&l, 9, true).len(), 5);
    assert_eq!(range(&l, 2, 4), l[1..4].to_vec());
    assert_eq!(range(&l, 4, 2), l[1..4].to_vec());
    assert_eq!(range(&l, 0, 2), l[..2].to_vec());
    assert_eq!(range(&l, 4, 99), l[3..].to_vec());
    assert!(range(&l, 7, 9).is_empty());
}

#[test]
fn banner_items() {
    assert_eq!(banner_pattern_item("Creeper"), Some("creeper_banner_pattern"));
    assert_eq!(
        banner_pattern_item("minecraft:field masoned"),
        Some("field_masoned_banner_pattern")
    );
    assert_eq!(banner_pattern_item("stripe_top"), None);
    assert_eq!(BANNER_PATTERN_ITEMS.len(), 10);
    for (_, item) in BANNER_PATTERN_ITEMS {
        assert!(carve_core::items::is_known(item), "{item}");
    }
}

#[test]
fn property_ranges() {
    assert_eq!(prop_max("beehive", Prop::Honey), Some(5));
    assert_eq!(prop_max("bee_nest", Prop::Honey), Some(5));
    assert_eq!(prop_max("suspicious_sand", Prop::Brushing), Some(3));
    assert_eq!(prop_max("wheat", Prop::Age), Some(7));
    assert_eq!(prop_max("stone", Prop::Age), None);
    let s = encode_state("wheat", FIRST, FIRST, &[("age".into(), "5".into())]).unwrap();
    assert_eq!(prop_value("wheat", s, FIRST, Prop::Age), Some(5));
    assert!(decode_state("wheat", s, FIRST).contains(&("age", "5")));
}

// ---- expressions ---------------------------------------------------------------

#[test]
fn first_last_and_range_of_a_list() {
    let out = run("set {l::*} to 1, 2, 3, 4, 5\n\
         send \"a=%first 2 elements of {l::*}%\"\n\
         send \"b=%last 2 elements of {l::*}%\"\n\
         send \"c=%elements 2 to 4 of {l::*}%\"\n\
         send \"d=%size of first 3 elements of {l::*}%\"\n\
         send \"e=%first element of {l::*}% %last element of {l::*}%\"\n\
         loop last 2 elements of {l::*}:\n    send \"l=%loop-value%\"");
    assert_eq!(
        out,
        [
            "a=1 and 2",
            "b=4 and 5",
            "c=2, 3 and 4",
            "d=3",
            "e=1 5",
            "l=4",
            "l=5"
        ]
    );
}

#[test]
fn dequeue_pops_the_front() {
    let out = run("set {_q} to a new queue of 1, 2, 3, 4\n\
         set {_a} to dequeue {_q}\n\
         set {_b} to de-queue {_q}\n\
         set {_c::*} to dequeue first 2 elements of {_q}\n\
         send \"%{_a}% %{_b}% %size of {_c::*}% %{_c::1}% %{_c::2}%\"\n\
         set {_e} to dequeue {_q}\n\
         send \"empty=%{_e}%\"");
    assert_eq!(out, ["1 2 2 3 4", "empty=<none>"]);
}

#[test]
fn banner_pattern_item_expression() {
    let out = run("set {_a} to banner pattern item of \"creeper\"\n\
         send \"%{_a}%\"\n\
         set {_b} to banner pattern item of \"x\"\n\
         send \"%{_b}%\"");
    assert_eq!(out, ["creeper banner pattern", "<none>"]);
}

#[test]
fn persistent_data_keys() {
    let out = run("set persistent data value \"b\" of player to 1\n\
         set persistent data value \"a\" of player to 2\n\
         send \"%persistent data keys of player%\"\n\
         set {_k::*} to all persistent data keys of player\n\
         send \"%size of {_k::*}%\"\n\
         delete persistent data value \"a\" of player\n\
         send \"%persistent data keys of player%\"");
    assert_eq!(out, ["a and b", "2", "b"]);
}

#[test]
fn honey_level_reads_and_writes() {
    let mut h = Mock::with("beehive", "honey_level", 3);
    let out = run_with(
        &mut h,
        "send \"%honey level of location(1, 2, 3)% %max honey level of location(1, 2, 3)%\"\n\
         set honey level of location(1, 2, 3) to 5\n\
         add 9 to honey level of location(1, 2, 3)\n\
         send \"%beehive honey level of location(1, 2, 3)%\"\n\
         remove 2 from honey level of location(1, 2, 3)\n\
         send \"%honey level of location(1, 2, 3)%\"\n\
         reset honey level of location(1, 2, 3)\n\
         send \"%honey level of location(1, 2, 3)%\"",
    );
    assert_eq!(out, ["3 5", "5", "3", "0"]);
    assert_eq!(h.set_states.len(), 4);
}

#[test]
fn brushing_stage_and_block_age() {
    let mut h = Mock::with("suspicious_sand", "dusted", 1);
    let out = run_with(
        &mut h,
        "send \"%brushing stage of location(0, 0, 0)% %max brushing stage of location(0, 0, 0)%\"\n\
         set brushing stage of location(0, 0, 0) to 3\n\
         send \"%brushing progress of location(0, 0, 0)%\"",
    );
    assert_eq!(out, ["1 3", "3"]);
    let mut h = Mock::with("wheat", "age", 2);
    let out = run_with(
        &mut h,
        "add 2 to age of location(0, 0, 0)\n\
         send \"%age of location(0, 0, 0)% / %max age of location(0, 0, 0)%\"",
    );
    assert_eq!(out, ["4 / 7"]);
}

#[test]
fn a_block_without_the_property_gives_nothing() {
    let mut h = Mock {
        block: Some(("stone".into(), FIRST, FIRST)),
        ..Mock::default()
    };
    let out = run_with(
        &mut h,
        "send \"%honey level of location(0, 0, 0)%\"\nset honey level of location(0, 0, 0) to 2",
    );
    assert_eq!(out, ["<none>"]);
    assert!(h.set_states.is_empty());
}

#[test]
fn bad_syntax_errors_clearly() {
    assert!(compile_err("send \"%first 2 elements of 5%\"").contains("list variable"));
    assert!(!compile_err("set {_x} to dequeue 5").is_empty());
}

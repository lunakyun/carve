//! Semantics that the fast paths for lists, function calls and number
//! comparisons must keep.

use std::time::{Duration, Instant};

use carve_core::*;

#[derive(Default)]
struct Mock {
    sent: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.sent.push(text.to_string());
    }
    fn broadcast(&mut self, _t: &str) {}
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
        1234
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
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
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "unexpected errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn fire(e: &mut Engine) -> Vec<String> {
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
    h.sent
}

/// Run `src` (a whole script) on join and return what it sent.
fn run(src: &str) -> Vec<String> {
    fire(&mut engine(src))
}

/// Run `body` (unindented lines) in `on join`.
fn join(body: &str) -> Vec<String> {
    let mut src = String::from("on join:\n");
    for l in body.lines() {
        src.push_str("    ");
        src.push_str(l);
        src.push('\n');
    }
    run(&src)
}

/// Both a local and a global copy of `body`, with `L` standing for the list name.
fn both(body: &str) -> (Vec<String>, Vec<String>) {
    (join(&body.replace('L', "_l")), join(&body.replace('L', "g")))
}

const DUMP: &str = "loop {L::*}:\n    send \"%loop-index%=%loop-value%\"\nsend \"size %size of {L::*}%\"";

#[test]
fn add_after_deletes_takes_next_free_index() {
    let body = format!(
        "delete {{L::*}}\nadd \"a\" to {{L::*}}\nadd \"b\" to {{L::*}}\nadd \"c\" to {{L::*}}\n\
         delete {{L::2}}\nadd \"d\" to {{L::*}}\nadd \"e\" to {{L::*}}\n{DUMP}"
    );
    let (l, g) = both(&body);
    // Size 2 after the delete, index 3 is taken, so `d` goes to 4, `e` to 5.
    let want = ["1=a", "3=c", "4=d", "5=e", "size 4"];
    assert_eq!(l, want);
    assert_eq!(g, want);
}

#[test]
fn add_after_delete_all_starts_at_one() {
    let body = format!(
        "add 1 to {{L::*}}\nadd 2 to {{L::*}}\ndelete {{L::*}}\nsend \"size %size of {{L::*}}%\"\n\
         add 9 to {{L::*}}\n{DUMP}"
    );
    let (l, g) = both(&body);
    let want = ["size 0", "1=9", "size 1"];
    assert_eq!(l, want);
    assert_eq!(g, want);
}

#[test]
fn text_and_odd_indexes() {
    let body = format!(
        "delete {{L::*}}\nset {{L::x}} to 5\nset {{L::01}} to \"z\"\nadd 7 to {{L::*}}\nadd 8 to {{L::*}}\n{DUMP}"
    );
    let (l, g) = both(&body);
    // Size 2 -> 3 is free for 7; `01` counts as index 1 but doesn't block 3 or 4.
    let want = ["01=z", "3=7", "4=8", "x=5", "size 4"];
    assert_eq!(l, want);
    assert_eq!(g, want);

    // `01` blocks index 1 when the count lands on it.
    let body = format!("delete {{L::*}}\nset {{L::01}} to \"z\"\nadd \"q\" to {{L::*}}\n{DUMP}");
    let (l, g) = both(&body);
    let want = ["01=z", "2=q", "size 2"];
    assert_eq!(l, want);
    assert_eq!(g, want);
}

#[test]
fn numeric_order_of_many_items() {
    let body = "delete {L::*}\nloop 12 times:\n    add loop-number to {L::*}\nset {_s} to \"\"\nloop {L::*}:\n    set {_s} to \"%{_s}%%loop-index%,\"\nsend {_s}";
    let (l, g) = both(body);
    let want = ["1,2,3,4,5,6,7,8,9,10,11,12,"];
    assert_eq!(l, want);
    assert_eq!(g, want);
}

#[test]
fn nested_lists() {
    let body = "delete {L::*}\nset {L::a::1} to \"x\"\nadd \"y\" to {L::a::*}\nadd \"top\" to {L::*}\n\
                send \"%size of {L::*}% %size of {L::a::*}%\"\nsend \"%{L::a::*}%\"\n\
                delete {L::a::*}\nsend \"%size of {L::*}% %size of {L::a::*}%\"\n\
                add \"z\" to {L::a::*}\nsend \"%{L::a::1}% %{L::1}%\"";
    let (l, g) = both(body);
    let want = ["1 2", "x and y", "1 0", "z top"];
    assert_eq!(l, want);
    assert_eq!(g, want);
}

#[test]
fn remove_and_set_none_update_size() {
    let body = "delete {L::*}\nadd 1 to {L::*}\nadd 2 to {L::*}\nadd 1 to {L::*}\nremove 1 from {L::*}\n\
                send \"%size of {L::*}% %{L::*}%\"\ndelete {L::2}\nsend \"%size of {L::*}%\"\n\
                add 5 to {L::*}\nadd 6 to {L::*}\nsend \"%{L::1}% %{L::2}%\"";
    let (l, g) = both(body);
    let want = ["1 2", "0", "5 6"];
    assert_eq!(l, want);
    assert_eq!(g, want);
}

#[test]
fn globals_survive_json_round_trip() {
    let mut e = engine(
        "on join:\n    delete {g::*}\n    add \"a\" to {g::*}\n    add \"b\" to {g::*}\n    delete {g::1}\n    set {g::sub::1} to 3\n",
    );
    fire(&mut e);
    let json = e.save_now().expect("dirty vars are saved");
    let vars = Variables::from_json(&json).expect("valid json");
    assert_eq!(vars.list_len("g::"), 1);
    assert_eq!(vars.list_len("g::sub::"), 1);
    let mut e2 = Engine::new(vars);
    let (_, errs) = e2.load(&[(
        "t.carve".into(),
        "on join:\n    add \"c\" to {g::*}\n    send \"%size of {g::*}% %{g::*}% %{g::sub::1}%\"\n".into(),
    )]);
    assert!(errs.is_empty());
    // Size 1 -> index 2 is taken by `b`, so `c` lands on 3.
    assert_eq!(fire(&mut e2), ["2 b and c 3"]);
    let json2 = e2.save_now().expect("saved");
    assert!(json2.contains("\"g::3\": \"c\""), "{json2}");
}

#[test]
fn variables_api_counts() {
    let mut v = Variables::default();
    v.set("l::1".into(), Value::Num(1.0));
    v.set("l::2".into(), Value::Num(2.0));
    v.set("l::x::1".into(), Value::Num(3.0));
    assert_eq!(v.list_len("l::"), 2);
    v.set("l::1".into(), Value::None);
    assert_eq!(v.list_len("l::"), 1);
    v.list_push("l::", Value::Num(4.0));
    assert_eq!(
        v.list("l::").iter().map(|(k, _)| k.as_str()).collect::<Vec<_>>(),
        ["2", "3"]
    );
    v.delete("l::*");
    assert_eq!(v.list_len("l::"), 0);
    assert_eq!(v.list_len("l::x::"), 0);
    assert!(v.is_empty());
}

#[test]
fn function_return_values() {
    let out = run(concat!(
        "function inc(x: number) :: number:\n",
        "    return {_x} + 1\n",
        "function fact(n: number) :: number:\n",
        "    if {_n} <= 1:\n",
        "        return 1\n",
        "    return {_n} * fact({_n} - 1)\n",
        "function total(xs: numbers) :: number:\n",
        "    set {_t} to 0\n",
        "    loop {_xs::*}:\n",
        "        add loop-value to {_t}\n",
        "    return {_t}\n",
        "function greet(name: text = \"you\") :: text:\n",
        "    return \"hi %{_name}%\"\n",
        "function sizes() :: number:\n",
        "    add 1 to {_mine::*}\n",
        "    return size of {_mine::*}\n",
        "on join:\n",
        "    set {_x} to 10\n",
        "    add 1 to {_mine::*}\n",
        "    add 2 to {_mine::*}\n",
        "    set {_s} to 0\n",
        "    loop 100 times:\n",
        "        set {_s} to {_s} + inc(loop-number)\n",
        "    send \"%{_s}% %fact(6)% %{_x}%\"\n",
        "    set {_l::*} to 1, 2 and 3\n",
        "    send \"%total({_l::*})% %greet()% %greet(\"bob\")%\"\n",
        "    send \"%sizes()% %sizes()% %size of {_mine::*}%\"\n",
    ));
    // Each call gets fresh locals; the caller's locals come back untouched.
    assert_eq!(out, ["5150 720 10", "6 hi you hi bob", "1 1 2"]);
}

#[test]
fn negative_literal_comparisons() {
    let mut src = String::from("on join:\n    loop 3 times:\n");
    for i in 1..4 {
        let kw = if i == 1 { "if" } else { "else if" };
        src.push_str(&format!(
            "        {kw} loop-number is -{i}:\n            send \"neg {i}\"\n"
        ));
    }
    src.push_str("        else if loop-number is 2:\n            send \"two\"\n");
    src.push_str("        else:\n            send \"other %loop-number%\"\n");
    src.push_str(concat!(
        "    set {_x} to -2\n",
        "    if {_x} is -2:\n",
        "        send \"x is -2\"\n",
        "    if {_x} is not -2:\n",
        "        send \"bad\"\n",
        "    if {_x} < -1:\n",
        "        send \"x < -1\"\n",
        "    if {_x} >= -2:\n",
        "        send \"x >= -2\"\n",
        "    set {_t} to \"-3\"\n",
        "    if {_t} is -3:\n",
        "        send \"text -3\"\n",
        "    set {_v::*} to -1, 5 and \"-1\"\n",
        "    loop {_v::*}:\n",
        "        if loop-value is -1:\n",
        "            send \"v%loop-index%\"\n",
        "    loop 2 times:\n",
        "        if loop-iteration > -1:\n",
        "            send \"it %loop-iteration%\"\n",
        "        if loop-number is not -1:\n",
        "            send \"not -1\"\n",
        "    if {_unset} is -1:\n",
        "        send \"bad unset\"\n",
        "    if {_unset} is not -1:\n",
        "        send \"unset is not -1\"\n",
    ));
    assert_eq!(
        run(&src),
        [
            "other 1",
            "two",
            "other 3",
            "x is -2",
            "x < -1",
            "x >= -2",
            "text -3",
            "v1",
            "v3",
            "it 1",
            "not -1",
            "it 2",
            "not -1",
            "unset is not -1",
        ]
    );
}

#[test]
fn hundred_thousand_list_adds_are_fast() {
    let mut e = engine(
        "on join:\n    loop 100000 times:\n        add loop-number to {_l::*}\n        add loop-number to {g::*}\n    send \"%size of {_l::*}% %size of {g::*}% %{_l::100000}% %{g::99999}%\"\n",
    );
    e.max_instructions = 10_000_000;
    let start = Instant::now();
    let out = fire(&mut e);
    let took = start.elapsed();
    assert_eq!(out, ["100000 100000 100000 99999"]);
    assert!(took < Duration::from_secs(5), "200k list adds took {took:?}");
}

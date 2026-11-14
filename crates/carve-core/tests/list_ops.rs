use carve_core::compile::compile_file;
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

/// Run `body` (indented lines) in `on join` and return what it sent.
fn run(src: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "unexpected errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
    h.sent
}

fn join(body: &str) -> Vec<String> {
    let mut src = String::from("on join:\n");
    for l in body.lines() {
        src.push_str("    ");
        src.push_str(l);
        src.push('\n');
    }
    run(&src)
}

fn compile_errors(src: &str) -> Vec<ScriptError> {
    compile_file("bad.carve", src).err().unwrap_or_default()
}

#[test]
fn sorted_numbers_and_text() {
    let out = join(
        "set {_n::*} to 10, 2, 33 and 4\nset {_s::*} to sorted {_n::*}\nsend \"%{_s::*}%\"\nset {_t::*} to \"pear\", \"Apple\" and \"fig\"\nsend \"%sorted {_t::*}%\"",
    );
    assert_eq!(out, ["2, 4, 10 and 33", "Apple, fig and pear"]);
}

#[test]
fn reversed_keeps_source() {
    let out = join("set {_n::*} to 1, 2, 3\nset {_r::*} to reversed {_n::*}\nsend \"%{_r::*}% / %{_n::*}%\"");
    assert_eq!(out, ["3, 2 and 1 / 1, 2 and 3"]);
}

#[test]
fn shuffled_has_same_items() {
    let out = join(
        "set {_n::*} to 1, 2, 3, 4, 5, 6, 7, 8\nset {_s::*} to shuffled {_n::*}\nsend \"%size of {_s::*}% %sum of {_s::*}%\"\nset {_o::*} to sorted {_s::*}\nsend \"%{_o::*}%\"",
    );
    assert_eq!(out, ["8 36", "1, 2, 3, 4, 5, 6, 7 and 8"]);
}

#[test]
fn where_filters() {
    let out = join(
        "set {_n::*} to 1, 2, 3, 4, 5\nsend \"%{_n::*} where [input > 2]%\"\nset {_f::*} to {_n::*} where [input > 1 and input < 5]\nsend \"%{_f::*}%\"\nset {_g::*} to {_n::*} where [input is 2 or input is 4]\nsend \"%{_g::*}%\"\nset {_h::*} to {_n::*} where [input is not 3]\nsend \"%{_h::*}%\"\nset {_k::*} to {_n::*} where input is greater than 3\nsend \"%{_k::*}%\"\nsend \"%size of ({_n::*} where [input >= 4])%\"",
    );
    assert_eq!(
        out,
        [
            "3, 4 and 5",
            "2, 3 and 4",
            "2 and 4",
            "1, 2, 4 and 5",
            "4 and 5",
            "2"
        ]
    );
}

#[test]
fn where_on_text() {
    let out = join(
        "set {_w::*} to \"apple\", \"banana\", \"avocado\"\nset {_a::*} to {_w::*} where [input starts with \"a\"]\nsend \"%{_a::*}%\"\nset {_b::*} to {_w::*} where [input contains \"nan\"]\nsend \"%{_b::*}%\"\nset {_c::*} to {_w::*} where [input does not contain \"a\"]\nsend \"%size of {_c::*}%\"",
    );
    assert_eq!(out, ["apple and avocado", "banana", "0"]);
}

#[test]
fn nested_filter_and_sort() {
    let out = join(
        "set {_n::*} to 9, 3, 7, 1, 5\nset {_s::*} to sorted {_n::*} where [input > 2]\nsend \"%{_s::*}%\"\nset {_r::*} to reversed sorted {_n::*}\nsend \"%{_r::*}%\"",
    );
    assert_eq!(out, ["3, 5, 7 and 9", "9, 7, 5, 3 and 1"]);
}

#[test]
fn aggregates() {
    let out = join(
        "set {_n::*} to 4, 8, 6\nsend \"%sum of {_n::*}% %min of {_n::*}% %max of {_n::*}% %average of {_n::*}%\"\nset {_t} to sum of {_n::*} + 1\nsend \"%{_t}%\"\nif sum of {_n::*} where [input > 5] is 14:\n    send \"ok\"\nsend \"%sum of {_none::*}%\"\nsend \"%max of {_none::*}%\"",
    );
    assert_eq!(out, ["18 4 8 6", "19", "ok", "0", "<none>"]);
}

#[test]
fn random_elements() {
    let out = join(
        "set {_n::*} to 1, 2, 3, 4, 5\nset {_r::*} to random 3 elements of {_n::*}\nsend \"%size of {_r::*}%\"\nset {_o::*} to sorted {_r::*}\nloop {_o::*}:\n    if {_n::*} does not contain loop-value:\n        send \"bad\"\nset {_all::*} to random 9 elements of {_n::*}\nsend \"%size of {_all::*}%\"\nsend \"%size of (random 2 items out of {_n::*})%\"",
    );
    assert_eq!(out, ["3", "5", "2"]);
}

#[test]
fn sort_effect() {
    let out = join(
        "set {_n::*} to 5, 1, 4\nsort {_n::*}\nsend \"%{_n::*}%\"\nsort {_n::*} in descending order\nsend \"%{_n::*}%\"\nsort {_n::*} in ascending order\nsend \"%{_n::*}%\"",
    );
    assert_eq!(out, ["1, 4 and 5", "5, 4 and 1", "1, 4 and 5"]);
}

#[test]
fn loop_over_list_ops() {
    let out = join(
        "set {_n::*} to 3, 1, 2\nloop sorted {_n::*}:\n    send \"%loop-index%:%loop-value%\"\nloop {_n::*} where [input > 1]:\n    send \"v%loop-value%\"",
    );
    assert_eq!(out, ["1:1", "2:2", "3:3", "v3", "v2"]);
}

#[test]
fn global_lists_and_concat() {
    let out = join(
        "set {g::*} to 3, 1\nset {h::*} to {g::*}, 7 and {g::*}\nsend \"%{h::*}%\"\nsort {h::*}\nsend \"%{h::*}%\"\nset {g::*} to sorted {g::*}\nsend \"%{g::*}%\"",
    );
    assert_eq!(out, ["3, 1, 7, 3 and 1", "1, 1, 3, 3 and 7", "1 and 3"]);
}

#[test]
fn set_list_replaces_and_handles_empty() {
    let out = join(
        "set {_a::*} to 1, 2, 3\nset {_a::*} to 9\nsend \"%{_a::*}%\"\nset {_a::*} to {_missing::*}\nsend \"%size of {_a::*}%\"",
    );
    assert_eq!(out, ["9", "0"]);
}

#[test]
fn list_functions() {
    let src = "function top(nums: numbers, n: number) :: numbers:\n    set {_s::*} to reversed sorted {_nums::*}\n    return random 0 elements of {_s::*}, {_s::1}, {_s::2}\n\nfunction total(nums: numbers) :: number:\n    return sum of {_nums::*}\n\nfunction evens(nums: numbers) :: numbers:\n    return {_nums::*} where [mod(input, 2) = 0]\n\non join:\n    set {_x::*} to 5, 8, 2, 9\n    set {_t::*} to top({_x::*}, 2)\n    send \"%{_t::*}%\"\n    send \"%total({_x::*})%\"\n    send \"%total(sorted {_x::*} where [input > 4])%\"\n    set {_e::*} to evens({_x::*})\n    send \"%{_e::*}% %size of {_e::*}%\"\n    loop evens({_x::*}):\n        send \"e%loop-value%\"\n    send \"%sum of evens({_x::*})%\"\n";
    let out = run(src);
    assert_eq!(out, ["9 and 8", "24", "22", "8 and 2 2", "e8", "e2", "10"]);
}

#[test]
fn list_param_is_a_copy() {
    let src = "function clear(nums: numbers):\n    delete {_nums::*}\n    add 1 to {_nums::*}\n\non join:\n    set {_x::*} to 5, 6\n    clear({_x::*})\n    send \"%{_x::*}%\"\n";
    assert_eq!(run(src), ["5 and 6"]);
}

#[test]
fn bad_syntax_has_errors() {
    for bad in [
        "set {_a::*} to sorted",
        "set {_a::*} to {_b::*} where [input >]",
        "set {_a::*} to {_b::*} where [input > 2",
        "send \"%sum of%\"",
        "sort {_a::*} sideways",
        "set {_a::*} to random 2 of {_b::*}",
    ] {
        let src = format!("on join:\n    {bad}\n");
        assert!(!compile_errors(&src).is_empty(), "should fail: {bad}");
    }
    let src = "function f(a: numbers) :: number:\n    return 1\n\non join:\n    send \"%f(1, 2)%\"\n";
    assert!(!compile_errors(src).is_empty());
}

#[test]
fn old_syntax_still_works() {
    let out = join(
        "add 3 to {_l::*}\nadd 4 to {_l::*}\nsend \"%size of {_l::*}% %{_l::*}%\"\nsend \"%max(2, 5)%\"\nloop {_l::*}:\n    send \"%loop-value%\"",
    );
    assert_eq!(out, ["2 3 and 4", "5", "3", "4"]);
}

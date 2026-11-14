use carve_core::compile::{EventKind, compile_file};
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.calls.push(text.to_string());
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, _: &str) {}
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
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
    }
}

fn wrap(body: &str) -> String {
    format!(
        "on join:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    )
}

fn run(body: &str) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), wrap(body))]);
    assert!(
        errs.is_empty(),
        "{}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    let mut h = Mock::default();
    let p = PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    };
    e.fire(EventKind::Join, EventCtx::player(p), &mut h);
    let errs: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errs.is_empty(), "runtime errors: {errs:?}");
    h.calls
}

fn compile_errors(body: &str) -> usize {
    compile_file("bad.carve", &wrap(body)).err().map_or(0, |e| e.len())
}

#[test]
fn set_list_literal() {
    let out = run("set {_l::*} to 1, 2 and 3\nsend \"%{_l::*}% (%size of {_l::*}%)\"");
    assert_eq!(out, ["1, 2 and 3 (3)"]);
}

#[test]
fn set_list_literal_replaces_old_items() {
    let out = run("set {_l::*} to \"a\", \"b\"\nset {_l::*} to \"c\" or \"d\"\nsend \"%{_l::*}%\"");
    assert_eq!(out, ["c and d"]);
}

#[test]
fn set_list_copy() {
    let out = run(
        "set {_a::*} to 1, 2, 3\nset {_b::*} to {_a::*}\nadd 4 to {_a::*}\nsend \"%{_a::*}% / %{_b::*}%\"",
    );
    assert_eq!(out, ["1, 2, 3 and 4 / 1, 2 and 3"]);
}

#[test]
fn set_list_self_reference_sees_old_list() {
    let out = run("set {_l::*} to 1, 2\nset {_l::*} to {_l::*}, 3\nsend \"%{_l::*}%\"");
    assert_eq!(out, ["1, 2 and 3"]);
}

#[test]
fn add_many() {
    let out = run("add 1, 2 and 3 to {_l::*}\nadd \"x\" to {_l::*}\nsend \"%{_l::*}%\"");
    assert_eq!(out, ["1, 2, 3 and x"]);
}

#[test]
fn add_list_to_list() {
    let out = run(
        "set {_a::*} to 1, 2\nadd {_a::*} to {_b::*}\nadd {_a::*} to {_b::*}\nsend \"%size of {_b::*}%\"",
    );
    assert_eq!(out, ["4"]);
}

#[test]
fn remove_many() {
    let out = run("set {_l::*} to 1, 2, 3, 4\nremove 2 and 4 from {_l::*}\nsend \"%{_l::*}%\"");
    assert_eq!(out, ["1 and 3"]);
}

#[test]
fn items_with_between_and_stay_whole() {
    let out = run("set {_l::*} to 5, random integer between 1 and 1, 7\nsend \"%{_l::*}%\"");
    assert_eq!(out, ["5, 1 and 7"]);
}

#[test]
fn global_lists_work_too() {
    let out = run(
        "set {g::*} to \"a\", \"b\"\nadd \"c\", \"d\" to {g::*}\nsend \"%size of {g::*}%\"\ndelete {g::*}\nsend \"%size of {g::*}%\"",
    );
    assert_eq!(out, ["4", "0"]);
}

#[test]
fn size_amount_number_of() {
    let out = run(
        "set {_l::*} to 1, 2, 3\nsend \"%size of {_l::*}% %amount of {_l::*}% %number of {_l::*}%\"\nif number of {_l::*} is 3:\n    send \"three\"",
    );
    assert_eq!(out, ["3 3 3", "three"]);
}

#[test]
fn contains() {
    let out = run(
        "set {_l::*} to \"a\", \"b\"\nif {_l::*} contains \"b\":\n    send \"yes\"\nif {_l::*} does not contain \"z\":\n    send \"no z\"",
    );
    assert_eq!(out, ["yes", "no z"]);
}

#[test]
fn indices_of() {
    let out = run(
        "set {_l::a} to 1\nset {_l::b} to 2\nsend \"%indices of {_l::*}%\"\nloop indices of {_l::*}:\n    send \"i=%loop-value%\"\nset {_i::*} to indices of {_l::*}\nsend \"%size of {_i::*}%\"",
    );
    assert_eq!(out, ["a and b", "i=a", "i=b", "2"]);
}

#[test]
fn first_last_and_nth_element() {
    let out = run(
        "set {_l::*} to \"x\", \"y\", \"z\"\nsend \"%first element of {_l::*}% %last element of {_l::*}% %element 2 of {_l::*}%\"\nset {_n} to 3\nsend \"%element {_n} of {_l::*}%\"\nsend \"%{_l::%{_n}%}%\"\nsend \"%element 9 of {_l::*}%\"",
    );
    assert_eq!(out, ["x z y", "z", "z", "<none>"]);
}

#[test]
fn first_element_of_empty_is_none() {
    let out = run("send \"%first element of {_e::*}% %last element of {_e::*}%\"");
    assert_eq!(out, ["<none> <none>"]);
}

#[test]
fn loop_literals() {
    let out = run(
        "loop 1, 2 and 3:\n    send \"n=%loop-value%\"\nloop \"a\" or \"b\":\n    send \"%loop-value%\"",
    );
    assert_eq!(out, ["n=1", "n=2", "n=3", "a", "b"]);
}

#[test]
fn loop_literal_with_between_and() {
    let out = run("loop 4, random integer between 2 and 2:\n    send \"%loop-value%\"");
    assert_eq!(out, ["4", "2"]);
}

#[test]
fn loop_two_lists() {
    let out = run(
        "set {_a::*} to 1, 2\nset {_b::*} to 3\nloop {_a::*} and {_b::*}:\n    send \"%loop-value%\"",
    );
    assert_eq!(out, ["1", "2", "3"]);
}

#[test]
fn delete_list() {
    let out = run("set {_l::*} to 1, 2\ndelete {_l::*}\nsend \"%size of {_l::*}%\"");
    assert_eq!(out, ["0"]);
}

#[test]
fn bad_list_literal_is_an_error() {
    assert_eq!(compile_errors("set {_l::*} to 1, , 2"), 1);
    assert_eq!(compile_errors("send \"%first element of {x}%\""), 1);
}

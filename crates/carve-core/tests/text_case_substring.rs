use carve_core::compile::{EventKind, compile_file};
use carve_core::*;

#[derive(Default)]
struct Mock {
    sent: Vec<String>,
    errors: Vec<String>,
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
    fn find_player(&mut self, _n: &str) -> Option<PlayerRef> {
        None
    }
    fn has_permission(&mut self, _p: &PlayerRef, _n: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _t: u64, _k: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        if level == LogLevel::Error {
            self.errors.push(msg.to_string());
        }
    }
}

fn run(body: &str) -> Vec<String> {
    let src = format!(
        "on join:\n{}",
        body.lines().map(|l| format!("    {l}\n")).collect::<String>()
    );
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src)]);
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
    assert!(h.errors.is_empty(), "{:?}", h.errors);
    h.sent
}

fn one(expr: &str) -> String {
    run(&format!("set {{_x}} to {expr}\nsend \"%{{_x}}%\"")).remove(0)
}

fn fails(expr: &str) -> bool {
    let src = format!("on join:\n    set {{_x}} to {expr}\n");
    compile_file("bad.carve", &src).is_err()
}

#[test]
fn case_forms() {
    assert_eq!(one("uppercase of \"Hello\""), "HELLO");
    assert_eq!(one("lowercase of \"Hello\""), "hello");
    assert_eq!(one("upper case of \"abc\""), "ABC");
    assert_eq!(one("lower case of \"ABC\""), "abc");
    assert_eq!(one("\"abc\" in upper case"), "ABC");
    assert_eq!(one("\"ABC\" in lower case"), "abc");
    assert_eq!(one("\"hello wORLD\" in proper case"), "Hello World");
    assert_eq!(one("capitalized \"hello world\""), "Hello world");
    assert_eq!(one("uppercase of player"), "STEVE");
}

#[test]
fn case_on_variable_and_in_template() {
    let out = run("set {_n} to \"steve\"\nsend \"hi %uppercase of {_n}%!\"\nsend \"%{_n} in upper case%\"");
    assert_eq!(out, vec!["hi STEVE!", "STEVE"]);
}

#[test]
fn substrings() {
    assert_eq!(one("substring of \"hello world\" from 1 to 5"), "hello");
    assert_eq!(one("subtext of \"hello world\" from 7 to 11"), "world");
    assert_eq!(one("substring of \"hello\" from 3"), "llo");
    assert_eq!(one("substring of \"hello\" from 0 to 99"), "hello");
    assert_eq!(one("substring of \"hello\" from 4 to 2"), "");
    assert_eq!(one("substring of \"hello\" from 2 + 1 to 2 * 2"), "ll");
}

#[test]
fn first_last_characters() {
    assert_eq!(one("first 3 characters of \"abcdef\""), "abc");
    assert_eq!(one("last 2 characters of \"abcdef\""), "ef");
    assert_eq!(one("first 99 characters of \"abc\""), "abc");
    assert_eq!(one("last 0 characters of \"abc\""), "");
    assert_eq!(one("first 1 character of \"xyz\""), "x");
}

#[test]
fn character_at() {
    assert_eq!(one("character at index 2 of \"abc\""), "b");
    assert_eq!(one("character at index 9 of \"abc\""), "<none>");
    assert_eq!(one("character at index 0 of \"abc\""), "<none>");
}

#[test]
fn index_of() {
    assert_eq!(one("index of \"l\" in \"hello\""), "3");
    assert_eq!(one("first index of \"l\" in \"hello\""), "3");
    assert_eq!(one("last index of \"l\" in \"hello\""), "4");
    assert_eq!(one("index of \"z\" in \"hello\""), "0");
    let out = run("if index of \"lo\" in \"hello\" is 4:\n    send \"yes\"");
    assert_eq!(out, vec!["yes"]);
}

#[test]
fn newline_trim_reverse() {
    assert_eq!(one("\"a%nl%b\""), "a\nb");
    assert_eq!(one("\"a%new line%b\""), "a\nb");
    assert_eq!(one("trimmed \"  hi  \""), "hi");
    assert_eq!(one("reversed \"abc\""), "cba");
    assert_eq!(one("reversed text of \"abc\""), "cba");
    assert_eq!(one("index of \"b\" in reversed \"abc\""), "2");
}

#[test]
fn unicode_is_by_character() {
    assert_eq!(one("first 2 characters of \"héllo\""), "hé");
    assert_eq!(one("character at index 2 of \"héllo\""), "é");
    assert_eq!(one("index of \"l\" in \"héllo\""), "3");
}

#[test]
fn none_in_none_out() {
    assert_eq!(one("uppercase of {_missing}"), "<none>");
}

#[test]
fn bad_syntax_is_rejected() {
    assert!(fails("substring of \"abc\" 1 to 2"));
    assert!(fails("first 3 of \"abc\""));
    assert!(fails("index of \"a\" \"abc\""));
}

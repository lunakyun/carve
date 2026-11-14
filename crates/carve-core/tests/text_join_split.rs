use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Host for Mock {
    fn send_message(&mut self, to: &Recipient, text: &str) {
        let who = match to {
            Recipient::Player(p) => p.name.clone(),
            Recipient::Console => "console".into(),
        };
        self.calls.push(format!("send {who} {text}"));
    }
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

fn run_host(src: &str) -> Mock {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("test.carve".into(), src.into())]);
    assert!(errs.is_empty(), "load errors: {errs:?}");
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    h
}

fn run(src: &str) -> Vec<String> {
    let h = run_host(src);
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
    h.calls
        .iter()
        .map(|c| c.strip_prefix("send Steve ").unwrap_or(c).to_string())
        .collect()
}

#[test]
fn join_list() {
    let out = run(r#"on join:
    add "a" to {l::*}
    add "b" to {l::*}
    add 3 to {l::*}
    send join {l::*} with ", "
    set {_s} to join {l::*} with "-"
    send "[%{_s}%]"
    set {e} to join {nothing::*} with ","
    send "<%{e}%>"
"#);
    assert_eq!(out, ["a, b, 3", "[a-b-3]", "<>"]);
}

#[test]
fn split_into_list_and_loop() {
    let out = run(r#"on join:
    set {p::*} to split "a,b,c" at ","
    send "%size of {p::*}% %{p::2}%"
    loop split "x-y" at "-":
        send "item %loop-value%"
    set {_c::*} to split "abc" at ""
    send "%size of {_c::*}%"
    set {p::*} to split "q" at ","
    send "%size of {p::*}%"
    send join {p::*} with "+"
"#);
    assert_eq!(out, ["3 b", "item x", "item y", "3", "1", "q"]);
}

#[test]
fn length_of_text() {
    let out = run(r#"on join:
    set {n} to "hello"
    send "%length of {n}%"
    send "%amount of characters in {n}%"
    send "%length of "ab" + 1%"
    if length of {n} > 3:
        send "long"
"#);
    assert_eq!(out, ["5", "5", "3", "long"]);
}

#[test]
fn replace_effect() {
    let out = run(r#"on join:
    set {t} to "a-b-c"
    replace all "-" with "+" in {t}
    send {t}
    replace first "+" in {t} with "_"
    send {t}
    replace "c" in {t} with "dd"
    send {t}
    set {_l} to "xx"
    replace all "x" in {_l} with "yy"
    send {_l}
    replace "q" with "z" in {unset}
    send "done"
"#);
    assert_eq!(out, ["a+b+c", "a_b+c", "a_b+dd", "yyyy", "done"]);
}

#[test]
fn replaced_expression() {
    let out = run(r#"on join:
    set {t} to "hello world"
    send {t} with "o" replaced with "0"
    set {u} to "aaa" with first "a" replaced by "b"
    send {u}
    send "%length of ("abc" with all "b" replaced with "xyz")%"
"#);
    assert_eq!(out, ["hell0 w0rld", "baa", "5"]);
}

#[test]
fn split_into_plain_variable_is_runtime_error() {
    let h = run_host("on join:\n    set {x} to split \"a\" at \",\"\n");
    assert!(
        h.logs
            .iter()
            .any(|(l, m)| *l == LogLevel::Error && m.contains("split"))
    );
}

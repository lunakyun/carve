use carve_core::compile::compile_file;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
}

impl Mock {
    fn take(&mut self) -> Vec<String> {
        std::mem::take(&mut self.calls)
    }
    fn errors(&self) -> Vec<&String> {
        self.logs
            .iter()
            .filter(|(l, _)| *l == LogLevel::Error)
            .map(|(_, m)| m)
            .collect()
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
    fn broadcast(&mut self, _: &str) {}
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

const CMD: &str = r#"command /who:
    trigger:
        if sender is a player:
            send "player %sender%" to sender
        else:
            send "console!" to sender
        if sender is not console:
            send "not console"
        if sender is the console:
            send "is console"
        send "exec %executor%"
"#;

fn run(e: &mut Engine, who: Recipient) -> Vec<String> {
    let mut h = Mock::default();
    e.run_command("who", who, "", &mut h);
    assert!(h.errors().is_empty(), "{:?}", h.errors());
    h.take()
}

#[test]
fn sender_and_conditions_as_player() {
    let mut e = engine(CMD);
    assert_eq!(
        run(&mut e, Recipient::Player(steve())),
        ["send Steve player Steve", "send Steve not console", "send Steve exec Steve"]
    );
}

#[test]
fn sender_and_conditions_as_console() {
    let mut e = engine(CMD);
    assert_eq!(
        run(&mut e, Recipient::Console),
        ["send console console!", "send console is console", "send console exec console"]
    );
}

fn run_line(line: &str, sender: Recipient) -> Vec<String> {
    let mut e = Engine::default();
    let mut h = Mock::default();
    if let Err(errs) = e.run_line(line, sender, &mut h) {
        panic!("{}", errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n"));
    }
    assert!(h.errors().is_empty(), "{:?}", h.errors());
    h.take()
}

#[test]
fn send_to_console_from_player_goes_to_log() {
    assert_eq!(
        run_line(r#"send "hi" to console"#, Recipient::Player(steve())),
        ["console-log hi"]
    );
    assert_eq!(
        run_line(r#"send "hi" to console"#, Recipient::Console),
        ["send console hi"]
    );
}

#[test]
fn make_player_execute_command() {
    let p = Recipient::Player(steve());
    assert_eq!(run_line(r#"make player execute command "/spawn""#, p.clone()), ["cmd-as Steve spawn"]);
    assert_eq!(run_line(r#"make me execute "home base""#, p.clone()), ["cmd-as Steve home base"]);
    assert_eq!(run_line(r#"make sender execute command "x""#, p.clone()), ["cmd-as Steve x"]);
    assert_eq!(run_line(r#"force player to execute command "y""#, p.clone()), ["cmd-as Steve y"]);
    assert_eq!(run_line(r#"execute command "/z" by player"#, p.clone()), ["cmd-as Steve z"]);
    assert_eq!(run_line(r#"make console execute command "say hi""#, p.clone()), ["cmd say hi"]);
    assert_eq!(run_line(r#"execute console command "say a""#, p.clone()), ["cmd say a"]);
    assert_eq!(run_line(r#"execute command "say b""#, p), ["cmd say b"]);
}

#[test]
fn me_is_the_player_and_console_in_text() {
    let p = Recipient::Player(steve());
    assert_eq!(run_line(r#"send "hi %me%" to me"#, p.clone()), ["send Steve hi Steve"]);
    assert_eq!(run_line(r#"send "who: %console%""#, p), ["send Steve who: console"]);
}

#[test]
fn variables_can_hold_console() {
    let mut e = engine(
        "command /who:
    trigger:
        set {c} to console
        send \"x\" to {c}
        if {c} is console:
            send \"yes\" to {c}
",
    );
    assert_eq!(run(&mut e, Recipient::Player(steve())), ["console-log x", "console-log yes"]);
}

#[test]
fn sender_is_not_allowed_in_functions() {
    let src = "function f():\n    send \"x\" to sender\n";
    assert!(compile_file("bad.carve", src).is_err());
}

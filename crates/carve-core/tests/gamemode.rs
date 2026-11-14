use carve_core::compile::{check_registry, compile_file};
use carve_core::*;

/// Tracks one player's game mode and logs calls.
struct Mock {
    mode: String,
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
    /// Use the trait's default `set_gamemode` (console command).
    use_default_set: bool,
}

impl Mock {
    fn new(mode: &str) -> Self {
        Mock {
            mode: mode.into(),
            calls: Vec::new(),
            logs: Vec::new(),
            use_default_set: false,
        }
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
        if let Recipient::Player(p) = to {
            self.calls.push(format!("send {} {}", p.name, text));
        }
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
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        (uuid == "u1").then(|| "Steve".to_string())
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
    fn gamemode(&mut self, _: &PlayerRef) -> Option<String> {
        Some(self.mode.clone())
    }
    fn set_gamemode(&mut self, p: &PlayerRef, mode: &str) {
        if self.use_default_set {
            self.execute_command(&format!("gamemode {mode} {}", p.name));
        } else {
            self.mode = mode.to_string();
            self.calls.push(format!("gm {} {mode}", p.name));
        }
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn run_with(src: &str, mut h: Mock) -> (Vec<String>, Mock) {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(errs.is_empty(), "unexpected errors: {:?}", errs);
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let calls = std::mem::take(&mut h.calls);
    (calls, h)
}

fn run(src: &str, mode: &str) -> Vec<String> {
    let (calls, h) = run_with(src, Mock::new(mode));
    assert!(h.errors().is_empty(), "runtime errors: {:?}", h.errors());
    calls
}

#[test]
fn registry_still_compiles() {
    check_registry().unwrap();
}

#[test]
fn reads_gamemode() {
    let out = run(
        "on join:\n    send \"%gamemode of player% %player's gamemode% %game mode of player%\"\n",
        "creative",
    );
    assert_eq!(out, ["send Steve creative creative creative"]);
}

#[test]
fn compares_gamemode() {
    let src = r#"on join:
    if gamemode of player is creative:
        send "a"
    if player's gamemode is survival:
        send "b"
    if player's gamemode is not spectator:
        send "c"
    if player's gamemode = Creative:
        send "d"
"#;
    assert_eq!(run(src, "creative"), ["send Steve a", "send Steve c", "send Steve d"]);
    assert_eq!(run(src, "survival"), ["send Steve b", "send Steve c"]);
    assert_eq!(run(src, "spectator"), Vec::<String>::new());
}

#[test]
fn is_in_conditions() {
    let src = r#"on join:
    if player is in creative:
        send "creative"
    if player is in adventure mode:
        send "adventure"
    if player is not in survival:
        send "not survival"
    if player isn't in survival mode:
        send "isnt"
"#;
    assert_eq!(
        run(src, "creative"),
        ["send Steve creative", "send Steve not survival", "send Steve isnt"]
    );
    assert_eq!(run(src, "adventure"), ["send Steve adventure", "send Steve not survival", "send Steve isnt"]);
    assert!(run(src, "survival").is_empty());
}

#[test]
fn sets_gamemode() {
    let src = r#"on join:
    set gamemode of player to creative
    send "%gamemode of player%"
    set player's gamemode to Adventure
    send "%gamemode of player%"
    set game mode of player to spectator mode
    set player's game mode to "survival"
    set gamemode of player to 1
    send "%gamemode of player%"
"#;
    assert_eq!(
        run(src, "survival"),
        [
            "gm Steve creative",
            "send Steve creative",
            "gm Steve adventure",
            "send Steve adventure",
            "gm Steve spectator",
            "gm Steve survival",
            "gm Steve creative",
            "send Steve creative",
        ]
    );
}

#[test]
fn set_default_uses_console_command() {
    let mut h = Mock::new("survival");
    h.use_default_set = true;
    let (calls, h) = run_with("on join:\n    set gamemode of player to creative\n", h);
    assert!(h.errors().is_empty());
    assert_eq!(calls, ["cmd gamemode creative Steve"]);
}

#[test]
fn bad_mode_is_a_runtime_error() {
    let (calls, h) = run_with(
        "on join:\n    set gamemode of player to \"banana\"\n    send \"after\"\n",
        Mock::new("survival"),
    );
    assert!(calls.is_empty());
    assert_eq!(h.errors().len(), 1);
    assert!(h.errors()[0].contains("not a game mode"), "{:?}", h.errors());
}

#[test]
fn gamemode_of_non_player_is_a_compile_error() {
    let errs = compile_file("bad.carve", "on join:\n    send \"%gamemode of 5%\"\n")
        .err()
        .unwrap_or_default();
    assert!(!errs.is_empty());
}

#[test]
fn mode_words_work_as_values() {
    let src = r#"on join:
    set {_m} to creative
    send "%{_m}%"
    if gamemode of player is {_m}:
        send "same"
"#;
    assert_eq!(run(src, "creative"), ["send Steve creative", "send Steve same"]);
}

#[test]
fn normalize_accepts_numbers_and_shorts() {
    use carve_core::gamemode::normalize;
    assert_eq!(normalize("0"), Some("survival"));
    assert_eq!(normalize("C"), Some("creative"));
    assert_eq!(normalize("sp"), Some("spectator"));
    assert_eq!(normalize("Adventure mode"), Some("adventure"));
    assert_eq!(normalize("x"), None);
}

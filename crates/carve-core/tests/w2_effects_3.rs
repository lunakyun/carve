use carve_core::compile::{EventKind, check_registry};
use carve_core::w2_effects_3::pack_uuid;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    logs: Vec<(LogLevel, String)>,
    others: Vec<PlayerRef>,
}

fn p(name: &str) -> PlayerRef {
    PlayerRef {
        uuid: format!("u-{}", name.to_lowercase()),
        name: name.into(),
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
        match uuid {
            "u-steve" => Some("Steve".into()),
            "u-alex" => Some("Alex".into()),
            _ => None,
        }
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        match name.to_ascii_lowercase().as_str() {
            "steve" => Some(p("Steve")),
            "alex" => Some(p("Alex")),
            _ => None,
        }
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
    fn online_players(&mut self) -> Vec<PlayerRef> {
        self.others.clone()
    }
    fn set_visible_to(&mut self, v: &PlayerRef, t: &PlayerRef, vis: bool) {
        self.calls.push(format!("vis {} sees {} {vis}", v.name, t.name));
    }
    fn set_sprinting(&mut self, pl: &PlayerRef, on: bool) {
        self.calls.push(format!("sprint {} {on}", pl.name));
    }
    fn eject_passengers(&mut self, pl: &PlayerRef) {
        self.calls.push(format!("eject {}", pl.name));
    }
    fn set_name_visible(&mut self, pl: &PlayerRef, v: bool) {
        self.calls.push(format!("name {} {v}", pl.name));
    }
    fn send_resource_pack(&mut self, pl: &PlayerRef, url: &str, hash: &str, id: &str) {
        self.calls.push(format!("pack {} {url} [{hash}] {id}", pl.name));
    }
}

fn run_with(src: &str, others: Vec<PlayerRef>) -> Vec<String> {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "errors: {:?}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>()
    );
    let mut h = Mock {
        others,
        ..Mock::default()
    };
    e.fire(EventKind::Join, EventCtx::player(p("Steve")), &mut h);
    assert!(
        h.logs.iter().all(|(l, _)| *l != LogLevel::Error),
        "runtime errors: {:?}",
        h.logs
    );
    h.calls
}

fn run(body: &str) -> Vec<String> {
    let src = format!("on join:\n    {}\n", body.replace('\n', "\n    "));
    run_with(&src, vec![p("Steve"), p("Alex")])
}

/// Runs `body` with `loop-player` set to Alex only.
fn run_alex(body: &str) -> Vec<String> {
    run(&format!(
        "loop all players:\n    if \"%loop-player%\" is \"Alex\":\n        {}",
        body.replace('\n', "\n        ")
    ))
}

fn compile_errors(body: &str) -> usize {
    let mut e = Engine::default();
    let src = format!("on join:\n    {body}\n");
    e.load(&[("t.carve".into(), src)]).1.len()
}

#[test]
fn registry_ok() {
    check_registry().unwrap();
}

#[test]
fn poison_default_one_second() {
    assert_eq!(
        run("poison player"),
        ["cmd effect give Steve minecraft:poison 1 0 true"]
    );
}

#[test]
fn poison_for_timespan() {
    assert_eq!(
        run("poison player for 10 seconds"),
        ["cmd effect give Steve minecraft:poison 10 0 true"]
    );
    assert_eq!(
        run("poison player for 1 minute"),
        ["cmd effect give Steve minecraft:poison 60 0 true"]
    );
    assert_eq!(
        run_alex("poison loop-player for 2500 milliseconds"),
        ["cmd effect give Alex minecraft:poison 3 0 true"]
    );
}

#[test]
fn cure_forms() {
    for line in ["cure player", "cure player of poison", "unpoison player"] {
        assert_eq!(run(line), ["cmd effect clear Steve minecraft:poison"], "{line}");
    }
}

#[test]
fn hide_from_everyone_skips_self() {
    assert_eq!(run("hide player"), ["vis Alex sees Steve false"]);
    assert_eq!(run("reveal player"), ["vis Alex sees Steve true"]);
}

#[test]
fn save_and_stop() {
    for l in ["save the world", "save world", "save all worlds", "save worlds"] {
        assert_eq!(run(l), ["cmd save-all"], "{l}");
    }
    for l in [
        "stop the server",
        "stop server",
        "shutdown server",
        "shut down the server",
    ] {
        assert_eq!(run(l), ["cmd stop"], "{l}");
    }
}

#[test]
fn stop_still_ends_the_trigger() {
    assert_eq!(run("send \"a\"\nstop\nsend \"b\""), ["send Steve a"]);
}

#[test]
fn sprinting() {
    assert_eq!(run("make player start sprinting"), ["sprint Steve true"]);
    assert_eq!(run("make player sprint"), ["sprint Steve true"]);
    assert_eq!(run("make player stop sprinting"), ["sprint Steve false"]);
}

#[test]
fn swing() {
    assert_eq!(run("make player swing"), ["cmd swing Steve mainhand"]);
    assert_eq!(run("make player swing main hand"), ["cmd swing Steve mainhand"]);
    assert_eq!(
        run("make player swing their off hand"),
        ["cmd swing Steve offhand"]
    );
}

#[test]
fn clear_and_reset_title() {
    assert_eq!(run("clear title of player"), ["cmd title Steve clear"]);
    assert_eq!(run("clear the title"), ["cmd title Steve clear"]);
    assert_eq!(run_alex("reset title of loop-player"), ["cmd title Alex reset"]);
    assert_eq!(run("reset player's title"), ["cmd title Steve reset"]);
}

#[test]
fn riding() {
    assert_eq!(run("dismount player"), ["cmd ride Steve dismount"]);
    assert_eq!(run("make player dismount"), ["cmd ride Steve dismount"]);
    assert_eq!(run("eject player"), ["eject Steve"]);
    assert_eq!(run("eject all passengers of player"), ["eject Steve"]);
    assert_eq!(
        run_alex("make player ride loop-player"),
        ["cmd ride Steve mount Alex"]
    );
    assert!(run("make player ride player").is_empty());
}

#[test]
fn custom_name_visibility() {
    assert_eq!(run("hide custom name of player"), ["name Steve false"]);
    assert_eq!(run("show the custom name of player"), ["name Steve true"]);
    assert_eq!(run("hide player's custom name"), ["name Steve false"]);
}

#[test]
fn resource_pack() {
    let out = run("send resource pack \"https://x.test/p.zip\" with hash \"abc\" to player");
    let id = pack_uuid("https://x.test/p.zip");
    assert_eq!(out, [format!("pack Steve https://x.test/p.zip [abc] {id}")]);
    let out = run("send the resource pack \"https://x.test/p.zip\" to player");
    assert_eq!(out, [format!("pack Steve https://x.test/p.zip [] {id}")]);
}

#[test]
fn pack_uuid_shape_and_stability() {
    let a = pack_uuid("a");
    assert_eq!(a.len(), 36);
    assert_eq!(a.matches('-').count(), 4);
    assert_eq!(a, pack_uuid("a"));
    assert_ne!(a, pack_uuid("b"));
    assert_eq!(&a[14..15], "4");
}

#[test]
fn toggle_variable() {
    let out = run(
        "toggle {x}\nsend \"%{x}%\"\ntoggle {x}\nsend \"%{x}%\"\nset {y} to true\nswitch {y}\nsend \"%{y}%\"\nset {_z} to false\ntoggle {_z}\nsend \"%{_z}%\"",
    );
    assert_eq!(
        out,
        [
            "send Steve true",
            "send Steve false",
            "send Steve false",
            "send Steve true"
        ]
    );
}

#[test]
fn toggle_non_boolean_is_a_runtime_error() {
    let src = "on join:\n    set {x} to 5\n    toggle {x}\n";
    let mut e = Engine::default();
    assert!(e.load(&[("t.carve".into(), src.into())]).1.is_empty());
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(p("Steve")), &mut h);
    assert!(
        h.logs
            .iter()
            .any(|(l, m)| *l == LogLevel::Error && m.contains("toggle"))
    );
}

#[test]
fn transform_list() {
    let out = run(
        "set {l::*} to 1, 2 and 3\ntransform {l::*} with (input * 2)\nsend \"%{l::1}% %{l::2}% %{l::3}%\"",
    );
    assert_eq!(out, ["send Steve 2 4 6"]);
}

#[test]
fn transform_local_list() {
    let out = run("set {_l::*} to 10 and 20\ntransform {_l::*} with input + 1\nsend \"%{_l::1}% %{_l::2}%\"");
    assert_eq!(out, ["send Steve 11 21"]);
}

#[test]
fn transform_without_with_is_a_compile_error() {
    assert!(compile_errors("transform {l::*}") > 0);
}

#[test]
fn bad_syntax_is_still_an_error() {
    assert!(compile_errors("poison") > 0);
    assert!(compile_errors("toggle 5") > 0);
    assert!(compile_errors("make player swing sideways") > 0);
}

#[test]
fn offline_target_is_skipped() {
    // Only names of online players resolve; an unknown name is skipped.
    assert!(
        run("set {n} to \"nobody\"
make {n} start sprinting")
        .is_empty()
    );
}

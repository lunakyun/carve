use carve_core::compile::{EventKind, compile_file};
use carve_core::world_effects::FxPos;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    errors: Vec<String>,
    next: u32,
    names: Vec<(String, String)>,
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.calls.push(text.to_string());
    }
    fn broadcast(&mut self, text: &str) {
        self.calls.push(text.to_string());
    }
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, _: &str) {}
    fn resolve_player(&mut self, uuid: &str) -> Option<String> {
        match uuid {
            "u1" => Some("Steve".into()),
            "123e4567-e89b-12d3-a456-426614174000" => Some("Alex".into()),
            _ => None,
        }
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        (name == "Steve").then(steve)
    }
    fn has_permission(&mut self, _: &PlayerRef, _: &str) -> bool {
        false
    }
    fn player_location(&mut self, _: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        Some((0.0, 64.0, 0.0, "world".to_string()))
    }
    fn schedule_wait(&mut self, _: u64, _: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        if level == LogLevel::Error {
            self.errors.push(msg.to_string());
        }
    }
    fn spawn_entity(&mut self, kind: &str, _at: &FxPos) -> Option<EntityRef> {
        self.next += 1;
        Some(EntityRef {
            uuid: format!("e{}", self.next),
            kind: kind.into(),
        })
    }
    fn set_entity_name(&mut self, e: &EntityRef, name: Option<&str>) {
        self.names.push((e.uuid.clone(), name.unwrap_or("").to_string()));
    }
    fn entity_name(&mut self, e: &EntityRef) -> Option<String> {
        self.names
            .iter()
            .rev()
            .find(|n| n.0 == e.uuid)
            .map(|n| n.1.clone())
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
    let (_, errs) = e.load(&[("t.carve".into(), src.into())]);
    assert!(
        errs.is_empty(),
        "load errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    let mut h = Mock::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    h
}

fn run(src: &str) -> Vec<String> {
    let h = run_host(src);
    assert!(h.errors.is_empty(), "runtime errors: {:?}", h.errors);
    h.calls
}

fn lines(body: &[&str]) -> Vec<String> {
    let mut src = String::from("on join:\n");
    for b in body {
        src.push_str("\tbroadcast ");
        src.push_str(b);
        src.push('\n');
    }
    run(&src)
}

fn errors_of(src: &str) -> Vec<String> {
    compile_file("bad.carve", src)
        .err()
        .unwrap_or_default()
        .iter()
        .map(|e| e.to_string())
        .collect()
}

#[test]
fn trig_in_degrees() {
    let out = lines(&[
        "\"%round(sin(30) * 100)%\"",
        "\"%round(cos(60) * 100)%\"",
        "\"%round(tan(45))%\"",
        "\"%asin(1)%\"",
        "\"%acos(0)%\"",
        "\"%atan(1)%\"",
        "\"%atan2(1, 1)%\"",
    ]);
    assert_eq!(out, vec!["50", "50", "1", "90", "90", "45", "45"]);
}

#[test]
fn exp_log_root() {
    let out = lines(&[
        "\"%round(exp(0))%\"",
        "\"%round(ln(1))%\"",
        "\"%round(log(1000))%\"",
        "\"%round(log(8, 2))%\"",
        "\"%root(27, 3)%\"",
        "\"%root(16)%\"",
    ]);
    assert_eq!(out, vec!["1", "0", "3", "3", "3", "4"]);
}

#[test]
fn combinatorics() {
    let out = lines(&[
        "\"%factorial(5)%\"",
        "\"%factorial(0)%\"",
        "\"%combinations(5, 2)%\"",
        "\"%permutations(5, 2)%\"",
        "\"%isNaN(factorial(-1))%\"",
        "\"%isNaN(3)%\"",
    ]);
    assert_eq!(out, vec!["120", "1", "10", "20", "true", "false"]);
}

#[test]
fn clamp_and_aggregates() {
    let out = lines(&[
        "\"%clamp(15, 0, 10)%\"",
        "\"%clamp(-3, 0, 10)%\"",
        "\"%clamp(4, 0, 10)%\"",
        "\"%median(1, 9, 5)%\"",
        "\"%median(1, 2, 3, 10)%\"",
        "\"%product(2, 3, 4)%\"",
    ]);
    assert_eq!(out, vec!["10", "0", "4", "5", "2.5", "24"]);
}

#[test]
fn aggregates_over_lists() {
    let out = run(
        "on join:\n\tset {_l::*} to 4, 1 and 7\n\tbroadcast \"%median({_l::*})%\"\n\tbroadcast \"%product({_l::*})%\"\n\tbroadcast concat({_l::*})\n\tset {_c} to concat(\"a\", 1, \"b\")
	broadcast {_c}\n",
    );
    assert_eq!(out, vec!["4", "28", "417", "a1b"]);
}

#[test]
fn bases_and_formatting() {
    let out = lines(&[
        "fromBase(\"ff\", 16)",
        "fromBase(\"101\", 2)",
        "toBase(255, 16)",
        "toBase(5, 2)",
        "formatNumber(1234567.891, \"#,##0.00\")",
        "formatNumber(0.5, \"0.#\")",
        "formatNumber(2, \"0.00\")",
        "formatNumber(0.256, \"#%%\")",
        "formatNumber(7, \"000\")",
        "formatNumber(-1234.5, \"$#,##0.0\")",
    ]);
    assert_eq!(
        out,
        vec![
            "255",
            "5",
            "ff",
            "101",
            "1,234,567.89",
            "0.5",
            "2.00",
            "26%",
            "007",
            "-$1,234.5"
        ]
    );
}

#[test]
fn xp_rgb_uuid() {
    let out = lines(&[
        "calcExperience(0)",
        "calcExperience(10)",
        "calcExperience(30)",
        "calcExperience(40)",
        "rgb(255, 128, 0)",
        "uuid(\"123E4567E89B12D3A456426614174000\")",
        "uuid(\"nope\")",
    ]);
    assert_eq!(
        out,
        vec![
            "0",
            "160",
            "1395",
            "2920",
            "#ff8000",
            "123e4567-e89b-12d3-a456-426614174000",
            "<none>"
        ]
    );
}

#[test]
fn player_lookup() {
    let out = lines(&[
        "name of player(\"Steve\")",
        "name of player(\"123e4567-e89b-12d3-a456-426614174000\")",
        "\"%player(\"Nobody\")%\"",
        "name of offlineplayer(\"Steve\")",
    ]);
    assert_eq!(out, vec!["Steve", "Alex", "<none>", "Steve"]);
}

#[test]
fn vector_function_exists() {
    let out = lines(&["\"%x of vector(1, 2, 3)%\""]);
    assert_eq!(out, vec!["1"]);
}

#[test]
fn clamp_with_bad_range_is_a_runtime_error() {
    let h = run_host("on join:\n\tbroadcast clamp(1, 5, 2)\n");
    assert_eq!(h.errors.len(), 1, "{:?}", h.errors);
}

#[test]
fn function_argument_errors_are_reported() {
    let e = errors_of("on join:\n\tset {_x} to sin()\n");
    assert!(!e.is_empty());
    let e = errors_of("on join:\n\tset {_x} to clamp(1, 2)\n");
    assert!(!e.is_empty());
    let e = errors_of("on join:\n\tset {_x} to sin(\"x\")\n");
    assert!(!e.is_empty());
}

#[test]
fn user_function_wins_over_builtin() {
    let out = run(
        "function clamp(a: number, b: number, c: number) :: number:\n\treturn 42\n\non join:\n\tbroadcast clamp(1, 2, 3)\n",
    );
    assert_eq!(out, vec!["42"]);
}

#[test]
fn filter_section_keeps_matching() {
    let out = run(
        "on join:\n\tset {_l::*} to 1, 2, 3, 4, 5, 6\n\tfilter {_l::*} to match:\n\t\tinput > 2\n\t\tinput < 6\n\tbroadcast \"%{_l::*}%\"\n\tbroadcast size of {_l::*}\n",
    );
    assert_eq!(out.last().map(String::as_str), Some("3"));
}

#[test]
fn filter_section_values() {
    let out = run(
        "on join:\n\tset {_l::*} to 1, 2, 3, 4, 5, 6\n\tfilter {_l::*} to match:\n\t\tinput > 3\n\tloop {_l::*}:\n\t\tbroadcast loop-value\n",
    );
    assert_eq!(out, vec!["4", "5", "6"]);
}

#[test]
fn filter_section_text_and_strings_untouched() {
    let out = run(
        "on join:\n\tset {_l::*} to \"apple\", \"input\", \"pear\"\n\tfilter {_l::*} to match:\n\t\t\"%input%\" is not \"input\"\n\tloop {_l::*}:\n\t\tbroadcast loop-value\n",
    );
    assert_eq!(out, vec!["apple", "pear"]);
}

#[test]
fn filter_section_errors() {
    let e = errors_of("on join:\n\tfilter {x} to match:\n\t\tinput > 1\n");
    assert!(e.iter().any(|m| m.contains("needs a list variable")), "{e:?}");
    let e = errors_of("on join:\n\tfilter {_x::*} to match:\n\t\tinput >\n");
    assert!(!e.is_empty());
}

#[test]
fn spawn_section_runs_body_on_the_new_entity() {
    let out = run(
        "on join:\n\tspawn a pig at player:\n\t\tset name of event-entity to \"Bob\"\n\tbroadcast name of last spawned entity\n",
    );
    assert_eq!(out, vec!["Bob"]);
}

#[test]
fn spawn_section_bad_entity_is_an_error() {
    let e = errors_of("on join:\n\tspawn a flurb at player:\n\t\tbroadcast \"x\"\n");
    assert!(e.iter().any(|m| m.contains("unknown entity")), "{e:?}");
}

#[test]
fn suppress_type_hints_runs_body() {
    let out = run("on join:\n\tsuppress type hints:\n\t\tbroadcast \"in\"\n");
    assert_eq!(out, vec!["in"]);
}

#[test]
fn shoot_section_compiles() {
    // The section works since wave 5 (see tests/w5_chunk6.rs).
    let e = errors_of("on join:\n\tshoot an arrow from player:\n\t\tbroadcast \"x\"\n");
    assert!(e.is_empty(), "{e:?}");
}

#[test]
fn auto_reload_structure_is_accepted() {
    let out = run("auto reload:
	interval: 5 seconds

on join:
	broadcast \"ok\"
");
    assert_eq!(out, vec!["ok"]);
}

#[test]
fn entity_function_loads_since_wave_6() {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[(
        "t.carve".into(),
        "on join:
	broadcast entity(\"x\")
"
        .into(),
    )]);
    assert!(errs.is_empty(), "{errs:?}");
}

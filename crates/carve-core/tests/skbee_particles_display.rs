use std::collections::HashMap;

use carve_core::compile::{EventKind, compile_file};
use carve_core::skbee_particles_display::{DisplayKind, DisplaySpec, ParticleData};
use carve_core::world_effects::{FxPos, ParticleFx};
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    online: HashMap<String, String>,
    logs: Vec<(LogLevel, String)>,
    data_ok: bool,
}

impl Mock {
    fn steve() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m
    }
}

impl Host for Mock {
    fn send_message(&mut self, _: &Recipient, _: &str) {}
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
        self.online.get(uuid).cloned()
    }
    fn find_player(&mut self, name: &str) -> Option<PlayerRef> {
        self.online
            .iter()
            .find(|(_, n)| n.eq_ignore_ascii_case(name))
            .map(|(u, n)| PlayerRef {
                uuid: u.clone(),
                name: n.clone(),
            })
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
    fn player_location(&mut self, p: &PlayerRef) -> Option<(f64, f64, f64, String)> {
        self.online
            .contains_key(&p.uuid)
            .then(|| (1.5, 64.0, -3.0, "world".to_string()))
    }
    fn supports_particle_data(&mut self) -> bool {
        self.data_ok
    }
    fn spawn_particle(&mut self, p: &ParticleFx) {
        self.calls.push(format!(
            "particle {} n={} off={:?} speed={} force={} data={:?}",
            p.name,
            p.count,
            p.offsets(),
            p.speed,
            p.force,
            p.data
        ));
    }
    fn spawn_display(&mut self, d: &DisplaySpec) {
        self.calls.push(format!(
            "display {:?} {:?} text={:?} id={:?}",
            d.kind,
            (d.at.x, d.at.y, d.at.z),
            d.text,
            d.id
        ));
        self.calls.push(format!("  {d:?}"));
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn script(body: &str) -> String {
    let lines: String = body.lines().map(|l| format!("    {l}\n")).collect();
    format!("on join:\n{lines}")
}

fn errors_of(body: &str) -> Vec<String> {
    compile_file("bad.carve", &script(body))
        .err()
        .unwrap_or_default()
        .iter()
        .map(|e| e.to_string())
        .collect()
}

fn load(body: &str) -> Engine {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("test.carve".into(), script(body))]);
    assert!(
        errs.is_empty(),
        "errors: {}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>().join("\n")
    );
    e
}

fn run_with(body: &str, mut h: Mock) -> Mock {
    let mut e = load(body);
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    h
}

fn run(body: &str) -> Vec<String> {
    let h = run_with(body, Mock::steve());
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
    h.calls
}

fn run_data(body: &str) -> Vec<String> {
    let mut m = Mock::steve();
    m.data_ok = true;
    let h = run_with(body, m);
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
    h.calls
}

#[test]
fn plain_particles_unchanged() {
    assert_eq!(
        run("play 5 flame particles at player with spread 2 with speed 0.5"),
        ["particle flame n=5 off=[2.0, 2.0, 2.0] speed=0.5 force=false data=None"]
    );
}

#[test]
fn extra_is_speed() {
    assert_eq!(
        run("play 3 flame particles at player with extra 0.1"),
        ["particle flame n=3 off=[0.0, 0.0, 0.0] speed=0.1 force=false data=None"]
    );
}

#[test]
fn per_axis_offset_and_force() {
    assert_eq!(
        run("play 4 cloud particles at player with offset 1, 0.5, 2 with force"),
        ["particle cloud n=4 off=[1.0, 0.5, 2.0] speed=0 force=true data=None"]
    );
}

#[test]
fn offset_is_clamped() {
    let c = run("play 1 cloud particles at player with offset 1000, -5, 2");
    assert_eq!(
        c,
        ["particle cloud n=1 off=[64.0, 0.0, 2.0] speed=0 force=false data=None"]
    );
}

#[test]
fn dust_color_and_size() {
    assert_eq!(
        run_data("play 10 dust particles at player with color rgb 255, 128, 0 with size 2"),
        [
            "particle dust n=10 off=[0.0, 0.0, 0.0] speed=0 force=false data=Dust { color: [255, 128, 0], size: 2.0 }"
        ]
    );
}

#[test]
fn dust_color_names_and_hex() {
    let c = run_data("play dust particles at player with color red");
    assert!(c[0].contains("color: [255, 85, 85]"), "{c:?}");
    let c = run_data("play dust particles at player with color \"#ff8000\"");
    assert!(c[0].contains("color: [255, 128, 0]"), "{c:?}");
    let c = run_data("play dust particles at player with color \"00ff00\"");
    assert!(c[0].contains("color: [0, 255, 0]"), "{c:?}");
}

#[test]
fn dust_defaults_to_red() {
    let c = run_data("play dust particles at player");
    assert!(c[0].contains("Dust { color: [255, 0, 0], size: 1.0 }"), "{c:?}");
}

#[test]
fn size_is_clamped() {
    let c = run_data("play dust particles at player with size 99");
    assert!(c[0].contains("size: 4.0"), "{c:?}");
}

#[test]
fn dust_transition() {
    let c = run_data(
        "play dust_color_transition particles at player with color rgb 255, 0, 0 with to color rgb 0, 0, 255 with size 1.5",
    );
    assert!(
        c[0].contains("DustTransition { from: [255, 0, 0], to: [0, 0, 255], size: 1.5 }"),
        "{c:?}"
    );
}

#[test]
fn block_and_item_data() {
    let c = run_data("play 20 block particles at player with block oak planks");
    assert!(c[0].contains("Block(\"oak_planks\")"), "{c:?}");
    let c = run_data("play 2 item particles at player with item diamond sword");
    assert!(c[0].contains("Item(\"diamond_sword\")"), "{c:?}");
    let c = run_data("play 2 falling_dust particles at player with block \"minecraft:sand\"");
    assert!(c[0].contains("Block(\"sand\")"), "{c:?}");
}

#[test]
fn data_particles_skipped_when_host_cannot_send_data() {
    let h = run_with("play 10 dust particles at player with color red", Mock::steve());
    assert!(h.calls.is_empty());
    assert!(
        h.logs
            .iter()
            .any(|(l, m)| *l == LogLevel::Warn && m.contains("dust")),
        "{:?}",
        h.logs
    );
}

#[test]
fn default_command_has_offsets_and_force() {
    let p = ParticleFx {
        name: "flame".into(),
        at: FxPos {
            x: 1.0,
            y: 2.0,
            z: 3.0,
            world: None,
        },
        count: 5,
        spread: 1.0,
        speed: 0.0,
        offset: Some([1.0, 2.0, 3.0]),
        force: true,
        data: ParticleData::None,
    };
    assert_eq!(
        carve_core::skbee_particles_display::particle_command(&p),
        "particle minecraft:flame 1 2 3 1 2 3 0 5 force"
    );
}

#[test]
fn particle_option_errors() {
    assert!(
        errors_of("play 5 dust particles at player with color")
            .iter()
            .any(|e| e.contains("color"))
    );
    assert!(
        errors_of("play 5 dust particles at player with color rgb 1, 2")
            .iter()
            .any(|e| e.contains("color"))
    );
}

#[test]
fn bad_color_is_a_runtime_error() {
    let h = run_with("play dust particles at player with color \"nope\"", {
        let mut m = Mock::steve();
        m.data_ok = true;
        m
    });
    assert!(
        h.logs
            .iter()
            .any(|(l, m)| *l == LogLevel::Error && m.contains("unknown color")),
        "{:?}",
        h.logs
    );
}

// ---------- displays ----------

fn spec_of(body: &str) -> String {
    run(body).join("\n")
}

#[test]
fn text_display_basic() {
    let c = run("spawn text display at player with text \"Hello\"");
    assert_eq!(
        c[0],
        "display Text (1.5, 64.0, -3.0) text=Some(\"Hello\") id=None"
    );
}

#[test]
fn text_display_showing_and_location() {
    let c = run("spawn text display showing \"Hi\" at location(1, 2, 3)");
    assert!(c[0].contains("(1.0, 2.0, 3.0) text=Some(\"Hi\")"), "{c:?}");
}

#[test]
fn text_display_options() {
    let s = spec_of(
        "spawn text display at player with text \"x\" with billboard center with scale 2 with view range 3 \
         with background rgb 0, 0, 0 with opacity 128 with line width 200 with alignment left with shadow \
         with see through with default background with glow red with brightness 15 with shadow radius 1.5",
    );
    for want in [
        "billboard: Some(\"center\")",
        "scale: Some([2.0, 2.0, 2.0])",
        "view_range: Some(3.0)",
        "background: Some(-16777216)",
        "opacity: Some(128)",
        "line_width: Some(200)",
        "alignment: Some(\"left\")",
        "shadow: true",
        "see_through: true",
        "default_background: true",
        "glow: Some(16733525)",
        "brightness: Some(15)",
        "shadow_radius: Some(1.5)",
    ] {
        assert!(s.contains(want), "missing {want} in {s}");
    }
}

#[test]
fn scale_three_axes_translation_and_clamp() {
    let s =
        spec_of("spawn text display at player with text \"x\" with scale 1, 2, 3 with translation 0, 1.5, 0");
    assert!(s.contains("scale: Some([1.0, 2.0, 3.0])"), "{s}");
    assert!(s.contains("translation: Some([0.0, 1.5, 0.0])"), "{s}");
    let s = spec_of(
        "spawn text display at player with text \"x\" with scale 5000 with brightness 99 with opacity 999",
    );
    assert!(s.contains("[100.0, 100.0, 100.0]"), "{s}");
    assert!(s.contains("brightness: Some(15)"), "{s}");
    assert!(s.contains("opacity: Some(255)"), "{s}");
}

#[test]
fn background_transparent() {
    let s = spec_of("spawn text display at player with text \"x\" with background transparent");
    assert!(s.contains("background: Some(0)"), "{s}");
}

#[test]
fn text_with_variable_and_color_codes() {
    let s = spec_of("set {_n} to \"Bob\"\nspawn text display at player with text \"&aHi %{_n}%\"");
    assert!(s.contains("text=Some(\"&aHi Bob\")"), "{s}");
}

#[test]
fn block_display() {
    let c = run("spawn block display of stone at player with scale 0.5 with billboard fixed");
    assert!(
        c[0].contains("display Block") && c[0].contains("id=Some(\"stone\")"),
        "{c:?}"
    );
    assert!(c[1].contains("scale: Some([0.5, 0.5, 0.5])"), "{c:?}");
}

#[test]
fn item_display() {
    let c = run("spawn item display of diamond sword at player with mode head");
    assert!(
        c[0].contains("display Item") && c[0].contains("id=Some(\"diamond_sword\")"),
        "{c:?}"
    );
    assert!(c[1].contains("item_mode: Some(\"head\")"), "{c:?}");
    let c = run("spawn item display of diamond at player with item mode third person left hand");
    assert!(c[1].contains("thirdperson_lefthand"), "{c:?}");
}

#[test]
fn other_verbs_and_articles() {
    assert!(run("summon a text display at player with text \"a\"")[0].starts_with("display Text"));
    assert!(run("create a new text display at player with text \"a\"")[0].starts_with("display Text"));
}

#[test]
fn display_errors() {
    assert!(
        errors_of("spawn text display with text \"x\"")
            .iter()
            .any(|e| e.contains("place"))
    );
    assert!(
        errors_of("spawn block display at player")
            .iter()
            .any(|e| e.contains("id"))
    );
    assert!(
        errors_of("spawn text display at player with billboard sideways")
            .iter()
            .any(|e| e.contains("billboard"))
    );
    assert!(
        errors_of("spawn text display at player with banana 3")
            .iter()
            .any(|e| e.contains("unknown display option"))
    );
    assert!(
        errors_of("spawn block display of stone at player with text \"x\"")
            .iter()
            .any(|e| e.contains("only text"))
    );
    assert!(
        errors_of("spawn item display of diamond at player with mode sideways")
            .iter()
            .any(|e| e.contains("mode"))
    );
    assert!(
        errors_of("spawn item display of notanitem at player")
            .iter()
            .any(|e| e.contains("unknown item"))
    );
}

struct Bare(Vec<String>, Vec<String>);

impl Host for Bare {
    fn send_message(&mut self, _: &Recipient, _: &str) {}
    fn broadcast(&mut self, _: &str) {}
    fn send_title(&mut self, _: &PlayerRef, _: &str, _: Option<&str>) {}
    fn send_action_bar(&mut self, _: &PlayerRef, _: &str) {}
    fn heal(&mut self, _: &PlayerRef) {}
    fn set_health(&mut self, _: &PlayerRef, _: f64) {}
    fn kill(&mut self, _: &PlayerRef) {}
    fn teleport(&mut self, _: &PlayerRef, _: f64, _: f64, _: f64) {}
    fn execute_command(&mut self, c: &str) {
        self.0.push(c.to_string());
    }
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
        if level == LogLevel::Warn {
            self.1.push(msg.to_string());
        }
    }
}

#[test]
fn text_display_default_summon_command() {
    let mut e = load(
        "spawn text display at location(1, 2, 3) with text \"&aHi it's\" with billboard center with scale 2 with shadow\n\
         spawn block display of stone at location(1, 2, 3)",
    );
    let mut h = Bare(vec![], vec![]);
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.0.len(), 1, "{:?}", h.0);
    let cmd = &h.0[0];
    assert!(
        cmd.starts_with("summon minecraft:text_display 1 2 3 {text:'"),
        "{cmd}"
    );
    assert!(cmd.contains("\"color\":\"#55ff55\""), "{cmd}");
    assert!(cmd.contains("it\\'s"), "{cmd}");
    assert!(cmd.contains("billboard:\"center\""), "{cmd}");
    assert!(cmd.contains("scale:[2f,2f,2f]"), "{cmd}");
    assert!(cmd.contains("shadow:1b"), "{cmd}");
    assert!(h.1.iter().any(|m| m.contains("block_display")), "{:?}", h.1);
}

#[test]
fn default_particle_command_runs() {
    let mut e = load("play 5 flame particles at location(1, 2, 3) with offset 1, 2, 3 with force");
    let mut h = Bare(vec![], vec![]);
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    assert_eq!(h.0, ["particle minecraft:flame 1 2 3 1 2 3 0 5 force"]);
}

#[test]
fn kind_names() {
    assert_eq!(DisplayKind::Text.entity_id(), "text_display");
}

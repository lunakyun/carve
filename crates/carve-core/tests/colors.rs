use carve_core::colors::{Style, colorize, parse, strip};
use carve_core::compile::compile_file;
use carve_core::*;

#[derive(Default)]
struct Mock {
    calls: Vec<String>,
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.calls.push(text.to_string());
    }
    fn broadcast(&mut self, text: &str) {
        self.calls.push(text.to_string());
    }
    fn send_title(&mut self, _p: &PlayerRef, title: &str, _sub: Option<&str>) {
        self.calls.push(title.to_string());
    }
    fn send_action_bar(&mut self, _p: &PlayerRef, text: &str) {
        self.calls.push(text.to_string());
    }
    fn heal(&mut self, _p: &PlayerRef) {}
    fn set_health(&mut self, _p: &PlayerRef, _h: f64) {}
    fn kill(&mut self, _p: &PlayerRef) {}
    fn teleport(&mut self, _p: &PlayerRef, _x: f64, _y: f64, _z: f64) {}
    fn execute_command(&mut self, _c: &str) {}
    fn resolve_player(&mut self, _uuid: &str) -> Option<String> {
        Some("Steve".into())
    }
    fn find_player(&mut self, _name: &str) -> Option<PlayerRef> {
        None
    }
    fn has_permission(&mut self, _p: &PlayerRef, _node: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _ticks: u64, _token: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, _level: LogLevel, _msg: &str) {}
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
        "{:?}",
        errs.iter().map(|e| e.to_string()).collect::<Vec<_>>()
    );
    let mut h = Mock::default();
    let p = PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    };
    e.fire(EventKind::Join, EventCtx::player(p), &mut h);
    h.calls
}

#[test]
fn legacy_codes() {
    let s = parse("&aGreen &lbold&r plain");
    assert_eq!(s.len(), 3);
    assert_eq!(s[0].style.color, Some((0x55, 0xFF, 0x55)));
    assert!(!s[0].style.bold);
    assert!(s[1].style.bold);
    assert_eq!(s[1].style.color, Some((0x55, 0xFF, 0x55)));
    assert_eq!(s[2].style, Style::default());
    assert_eq!(s[2].text, " plain");
}

#[test]
fn hex_forms() {
    for src in ["<#ff8000>x", "&#ff8000x", "&x&f&f&8&0&0&0x", "§x§f§f§8§0§0§0x"] {
        let s = parse(src);
        assert_eq!(s.len(), 1, "{src}");
        assert_eq!(s[0].style.color, Some((0xff, 0x80, 0)), "{src}");
        assert_eq!(s[0].text, "x", "{src}");
    }
}

#[test]
fn named_tags() {
    let s = parse("<red>a<bold>b</bold>c</red>d<unknown>");
    assert_eq!(s.len(), 4);
    assert_eq!(s[0].style.color, Some((0xFF, 0x55, 0x55)));
    assert!(s[1].style.bold);
    assert!(!s[2].style.bold && s[2].style.color.is_some());
    assert_eq!(s[3].style, Style::default().clone());
    assert_eq!(s[3].text, "d<unknown>");
}

#[test]
fn plain_text_untouched() {
    for t in ["a & b", "1 < 2 > 0", "Tom&Jerry", "&", "<", "<#12345>", "end &"] {
        assert_eq!(strip(t), t);
        let sp = parse(t);
        assert_eq!(sp.len(), 1);
        assert_eq!(sp[0].text, t);
    }
}

#[test]
fn strip_removes_everything() {
    assert_eq!(strip("&aHi <#00ff00>there<bold>!&r"), "Hi there!");
    assert_eq!(strip("§cred"), "red");
}

#[test]
fn colorize_is_idempotent() {
    let once = colorize("&aHi <#123456>there <bold>!");
    assert!(once.contains("§a"));
    assert!(once.contains("§x§1§2§3§4§5§6"));
    assert!(!once.contains('&') && !once.contains('<'));
    assert_eq!(colorize(&once), once);
    assert_eq!(strip(&once), "Hi there !");
}

#[test]
fn script_colored_and_uncolored() {
    let out = run(
        "set {_t} to \"<red>Hello &lworld\"\nsend colored {_t}\nsend uncolored {_t}\nsend unformatted \"&aA\"\nset {_u} to uncolored \"<#ff0000>abc\"\nsend \"len: %{_u}%\"",
    );
    assert_eq!(out[0], "§cHello §r§c§lworld");
    assert_eq!(out[1], "Hello world");
    assert_eq!(out[2], "A");
    assert_eq!(out[3], "len: abc");
}

#[test]
fn bad_use_is_an_error() {
    assert!(compile_file("b.carve", "on join:\n    send colored\n").is_err());
}

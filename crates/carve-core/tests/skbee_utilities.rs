use carve_core::compile::{EventKind, compile_file};
use carve_core::skbee_utilities::{self, Info, UtilFn};
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
    fn player_info(&mut self, _p: &PlayerRef, i: Info) -> Option<String> {
        Some(
            match i {
                Info::Brand => "vanilla",
                Info::Locale => "en_us",
                Info::ViewDistance => "12",
                Info::Pose => "standing",
                Info::MainHand => "right",
            }
            .to_string(),
        )
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
fn commas() {
    assert_eq!(one("1234567 with commas"), "1,234,567");
    assert_eq!(one("999 with commas"), "999");
    assert_eq!(one("1000 with commas"), "1,000");
    assert_eq!(one("(-1234567) with commas"), "-1,234,567");
    assert_eq!(one("1234.5 with commas"), "1,234.5");
    assert_eq!(
        one("1234567.891 with commas and 2 decimal places"),
        "1,234,567.89"
    );
    assert_eq!(one("5 with commas and 2 decimals"), "5.00");
}

#[test]
fn commas_in_template_and_variable() {
    let out = run("set {_n} to 2500000\nsend \"you have %{_n} with commas% coins\"");
    assert_eq!(out, vec!["you have 2,500,000 coins"]);
}

#[test]
fn rounding() {
    assert_eq!(one("3.14159 rounded to 2 decimal places"), "3.14");
    assert_eq!(one("2.5 rounded to 0 decimals"), "3");
    assert_eq!(one("1.005 rounded to 1 decimal place"), "1");
    let out = run("set {_n} to 7.777\nsend \"%{_n} rounded to 1 decimal place%\"");
    assert_eq!(out, vec!["7.8"]);
}

#[test]
fn abbreviated_numbers() {
    assert_eq!(one("abbreviated 999"), "999");
    assert_eq!(one("abbreviated 1500"), "1.5K");
    assert_eq!(one("abbreviated 2000000"), "2M");
    assert_eq!(one("abbreviated 999999"), "1M");
    assert_eq!(one("abbreviated 3400000000"), "3.4B");
}

#[test]
fn roman_numerals() {
    assert_eq!(one("roman numeral of 4"), "IV");
    assert_eq!(one("roman numeral of 1994"), "MCMXCIV");
    assert_eq!(one("roman of 3999"), "MMMCMXCIX");
    assert_eq!(one("number from roman numeral \"XIV\""), "14");
    assert_eq!(one("number from roman \"mcmxciv\""), "1994");
    assert_eq!(one("number from roman numeral \"IIII\" ? 0"), "0");
    assert_eq!(one("roman numeral of 0 ? \"none\""), "none");
    assert_eq!(one("roman numeral of 4000 ? \"none\""), "none");
}

#[test]
fn hex() {
    assert_eq!(one("hex of 255"), "ff");
    assert_eq!(one("hex of 0"), "0");
    assert_eq!(one("hex of -16"), "-10");
    assert_eq!(one("number from hex \"ff\""), "255");
    assert_eq!(one("number from hex \"#FF00FF\""), "16711935");
    assert_eq!(one("number from hex \"0x1A\""), "26");
    assert_eq!(one("number from hex \"zz\" ? -1"), "-1");
}

#[test]
fn json() {
    assert_eq!(one("json of \"hi\""), "\"hi\"");
    assert_eq!(one("json of \"a \"\"q\"\" b\""), "\"a \\\"q\\\" b\"");
    assert_eq!(one("json of 5"), "5");
    assert_eq!(one("json of true"), "true");
    assert_eq!(one("json of player"), "\"Steve\"");
    assert_eq!(one("json of \"line1%nl%line2\""), "\"line1\\nline2\"");
}

#[test]
fn random_uuid() {
    let out = run("set {_a} to random uuid\nset {_b} to random uuid\nsend \"%{_a}%\"\nsend \"%{_b}%\"");
    assert_eq!(out.len(), 2);
    for u in &out {
        assert_eq!(u.len(), 36, "{u}");
        let parts: Vec<&str> = u.split('-').collect();
        assert_eq!(
            parts.iter().map(|p| p.len()).collect::<Vec<_>>(),
            vec![8, 4, 4, 4, 12]
        );
        assert!(parts[2].starts_with('4'), "{u}");
        assert!("89ab".contains(parts[3].chars().next().unwrap()), "{u}");
        assert!(u.chars().all(|c| c == '-' || c.is_ascii_hexdigit()));
    }
    assert_ne!(out[0], out[1]);
}

#[test]
fn player_properties() {
    assert_eq!(one("client brand of player"), "vanilla");
    assert_eq!(one("player's client brand"), "vanilla");
    assert_eq!(one("locale of player"), "en_us");
    assert_eq!(one("player's locale"), "en_us");
    assert_eq!(one("view distance of player"), "12");
    assert_eq!(one("player's client view distance"), "12");
    assert_eq!(one("pose of player"), "standing");
    assert_eq!(one("main hand of player"), "right");
    assert_eq!(one("(view distance of player) + 1"), "13");
}

#[test]
fn bad_input_gives_errors_or_none() {
    assert!(fails("hex of \"x\""));
    assert!(fails("abbreviated \"x\""));
    assert!(fails("client brand of 5"));
    assert_eq!(one("hex of {_nothing} ? \"unset\""), "unset");
}

#[test]
fn pure_helpers() {
    assert_eq!(skbee_utilities::group_commas("1234567"), "1,234,567");
    assert_eq!(skbee_utilities::group_commas("12"), "12");
    assert_eq!(skbee_utilities::to_roman(9), Some("IX".into()));
    assert_eq!(skbee_utilities::from_roman("MMXXVI"), Some(2026));
    assert_eq!(skbee_utilities::from_roman("VX"), None);
    assert!(UtilFn::Round.returns_number());
    assert!(!UtilFn::Hex.returns_number());
}

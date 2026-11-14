use carve_core::skbee_text::{Click, Hover, Kind, Part, escape, is_rich, parse, plain};
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

fn text_of(p: &Part) -> String {
    match &p.kind {
        Kind::Text(t) => t.clone(),
        Kind::Key(k) => format!("key:{k}"),
        Kind::Lang(k, _) => format!("lang:{k}"),
    }
}

#[test]
fn hover_show_text() {
    let p = parse("<hover:show_text:'<red>tip'>Hi</hover> bye");
    assert_eq!(p.len(), 2);
    assert_eq!(text_of(&p[0]), "Hi");
    let Some(h) = &p[0].hover else { panic!("no hover") };
    let Hover::Text(inner) = &**h else {
        panic!("not text")
    };
    assert_eq!(inner[0].style.color, Some((0xFF, 0x55, 0x55)));
    assert_eq!(text_of(&inner[0]), "tip");
    assert!(p[1].hover.is_none());
    assert_eq!(text_of(&p[1]), " bye");
}

#[test]
fn click_actions() {
    let cases = [
        ("<click:run_command:'/spawn'>x", Click::Run("/spawn".into())),
        ("<click:suggest_command:/msg >x", Click::Suggest("/msg".into())),
        (
            "<click:open_url:https://example.com/a?b=1>x",
            Click::Url("https://example.com/a?b=1".into()),
        ),
        (
            "<click:copy_to_clipboard:'hello world'>x",
            Click::Copy("hello world".into()),
        ),
        ("<click:change_page:3>x", Click::Page(3)),
    ];
    for (src, want) in cases {
        let p = parse(src);
        assert_eq!(p.len(), 1, "{src}");
        assert_eq!(p[0].click, Some(want), "{src}");
    }
}

#[test]
fn click_ends_at_close_tag() {
    let p = parse("<click:run_command:'/a'>one</click>two");
    assert!(p[0].click.is_some());
    assert!(p[1].click.is_none());
}

#[test]
fn insertion_key_and_lang() {
    let p = parse("<insertion:'shift me'>a</insertion><key:key.jump><lang:chat.type.text:'Steve':'<red>hi'>");
    assert_eq!(p[0].insertion.as_deref(), Some("shift me"));
    assert_eq!(p[1].kind, Kind::Key("key.jump".into()));
    assert!(p[1].insertion.is_none());
    let Kind::Lang(k, args) = &p[2].kind else { panic!() };
    assert_eq!(k, "chat.type.text");
    assert_eq!(args.len(), 2);
    assert_eq!(text_of(&args[0][0]), "Steve");
    assert_eq!(args[1][0].style.color, Some((0xFF, 0x55, 0x55)));
}

#[test]
fn hover_item_and_entity() {
    let p = parse("<hover:show_item:diamond:3>x</hover><hover:show_entity:pig:'0000-1':'<gold>Bob'>y");
    assert_eq!(
        p[0].hover.as_deref(),
        Some(&Hover::Item("{id:\"minecraft:diamond\",count:3}".into()))
    );
    let Some(h) = &p[1].hover else { panic!() };
    let Hover::Entity { kind, id, name } = &**h else {
        panic!()
    };
    assert_eq!((kind.as_str(), id.as_str()), ("pig", "0000-1"));
    assert!(name.is_some());
}

#[test]
fn styles_carry_into_events() {
    let p = parse("<red><bold><click:run_command:'/x'>go</click> more");
    assert_eq!(p[0].style.color, Some((0xFF, 0x55, 0x55)));
    assert!(p[0].style.bold);
    assert!(p[1].style.bold && p[1].click.is_none());
}

#[test]
fn gradient_spreads_colors() {
    let p = parse("<gradient:#000000:#ffffff>abc</gradient>d");
    assert_eq!(p.len(), 4);
    assert_eq!(p[0].style.color, Some((0, 0, 0)));
    assert_eq!(p[1].style.color, Some((128, 128, 128)));
    assert_eq!(p[2].style.color, Some((255, 255, 255)));
    assert_eq!(p[3].style.color, None);
}

#[test]
fn rainbow_starts_red() {
    let p = parse("<rainbow>abc");
    assert_eq!(p.len(), 3);
    assert_eq!(p[0].style.color, Some((255, 0, 0)));
    assert_ne!(p[1].style.color, p[0].style.color);
}

#[test]
fn escapes_and_plain() {
    assert_eq!(plain(&escape("a <red>b</red> \\ c")), "a <red>b</red> \\ c");
    assert_eq!(
        plain("<hover:show_text:'x'>shown</hover> <red>red</red>"),
        "shown red"
    );
    assert!(is_rich(&escape("<b>")));
    assert!(!is_rich("<red>plain colors &a"));
    assert!(is_rich("<click:run_command:'/x'>y"));
}

#[test]
fn bad_tags_stay_text() {
    let p = parse("<click:nope:x>text <hover:bad:y>");
    assert_eq!(
        p.iter().map(text_of).collect::<String>(),
        "<click:nope:x>text <hover:bad:y>"
    );
}

#[test]
fn script_text_component_escapes() {
    let out = run("set {_c} to text component from \"<red>x\"\nsend {_c}");
    assert_eq!(out, vec!["\\<red>x"]);
}

#[test]
fn script_hover_and_click_event() {
    let out = run(concat!(
        "set {_c} to text component from \"Click me\"\n",
        "set hover event of {_c} to hover event showing \"It's a tip\"\n",
        "set click event of {_c} to click event to run command \"/spawn\"\n",
        "send component {_c}"
    ));
    assert_eq!(out.len(), 1);
    let p = parse(&out[0]);
    assert_eq!(p.len(), 1, "{out:?}");
    assert_eq!(text_of(&p[0]), "Click me");
    assert_eq!(p[0].click, Some(Click::Run("/spawn".into())));
    let Some(h) = &p[0].hover else { panic!() };
    let Hover::Text(t) = &**h else { panic!() };
    assert_eq!(text_of(&t[0]), "It's a tip");
}

#[test]
fn script_click_kinds() {
    for (phrase, want) in [
        ("open url \"https://a.b/c\"", Click::Url("https://a.b/c".into())),
        ("suggest command \"/hi\"", Click::Suggest("/hi".into())),
        ("copy to clipboard \"it's\"", Click::Copy("it's".into())),
        ("change page 2", Click::Page(2)),
    ] {
        let out = run(&format!(
            "set {{_c}} to text component from \"x\"\nset click event of {{_c}} to click event to {phrase}\nsend {{_c}}"
        ));
        assert_eq!(parse(&out[0])[0].click, Some(want), "{phrase}");
    }
}

#[test]
fn script_insertion() {
    let out = run("set {_c} to text component from \"x\"\nset insertion of {_c} to \"abc\"\nsend {_c}");
    assert_eq!(parse(&out[0])[0].insertion.as_deref(), Some("abc"));
}

#[test]
fn script_translate_keybind_merge() {
    let out = run(concat!(
        "set {_t} to translate component \"chat.type.text\" with \"Steve\" and \"hello\"\n",
        "set {_k} to keybind component \"key.jump\"\n",
        "set {_m} to merged components of {_t}, \" \" and {_k}\n",
        "send component {_m}"
    ));
    let p = parse(&out[0]);
    assert_eq!(p.len(), 3, "{out:?}");
    let Kind::Lang(k, args) = &p[0].kind else { panic!() };
    assert_eq!(k, "chat.type.text");
    assert_eq!(args.len(), 2);
    assert_eq!(p[2].kind, Kind::Key("key.jump".into()));
}

#[test]
fn script_merge_list_variable() {
    let out = run("set {_l::*} to \"<red>a\", \"b\" and \"c\"\nsend merged components of {_l::*}");
    assert_eq!(out, vec!["<red>abc"]);
}

#[test]
fn script_mini_message_and_plain_text() {
    let out = run(
        "send mini message from \"<click:run_command:'/x'>go</click>\"\nsend plain text of \"<red>hi</red>\"",
    );
    assert_eq!(out[0], "<click:run_command:'/x'>go</click>");
    assert_eq!(out[1], "hi");
}

#[test]
fn script_hover_item() {
    let out = run(
        "set {_c} to text component from \"x\"\nset hover event of {_c} to hover event showing 2 diamonds\nsend {_c}",
    );
    assert_eq!(
        parse(&out[0])[0].hover.as_deref(),
        Some(&Hover::Item("{id:\"minecraft:diamond\",count:2}".into()))
    );
}

#[test]
fn script_send_component_forms() {
    let out = run(
        "send component \"a\" to player\nbroadcast component \"b\"\nsend action bar component \"c\" to player",
    );
    assert_eq!(out, vec!["a", "b", "c"]);
}

#[test]
fn script_errors_are_clear() {
    for body in [
        "set hover event of {_c::*} to hover event showing \"x\"",
        "set hover event of \"x\" to hover event showing \"x\"",
        "set {_c} to click event to nothing \"x\"",
    ] {
        let src = format!("on join:\n    {body}\n");
        let mut e = Engine::default();
        let (_, errs) = e.load(&[("t.carve".into(), src)]);
        assert!(!errs.is_empty(), "{body}");
    }
}

#[test]
fn registry_still_valid() {
    assert!(carve_core::compile::check_registry().is_ok());
}

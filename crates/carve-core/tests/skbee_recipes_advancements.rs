use std::collections::HashMap;

use carve_core::compile::{EventKind, compile_file};
use carve_core::skbee_recipes_advancements::{CookKind, Ingredient, RecipeDef, RecipeKind};
use carve_core::*;

/// Records every call this feature makes.
#[derive(Default)]
struct Mock {
    calls: Vec<String>,
    online: HashMap<String, String>,
    recipes: Vec<RecipeDef>,
    advs: Vec<String>,
    stats: HashMap<String, i64>,
    logs: Vec<(LogLevel, String)>,
    msgs: Vec<String>,
}

impl Mock {
    fn steve() -> Self {
        let mut m = Mock::default();
        m.online.insert("u1".into(), "Steve".into());
        m
    }
}

impl Host for Mock {
    fn send_message(&mut self, _to: &Recipient, text: &str) {
        self.msgs.push(text.to_string());
    }
    fn broadcast(&mut self, _text: &str) {}
    fn send_title(&mut self, _p: &PlayerRef, _t: &str, _s: Option<&str>) {}
    fn send_action_bar(&mut self, _p: &PlayerRef, _t: &str) {}
    fn heal(&mut self, _p: &PlayerRef) {}
    fn set_health(&mut self, _p: &PlayerRef, _h: f64) {}
    fn kill(&mut self, _p: &PlayerRef) {}
    fn teleport(&mut self, _p: &PlayerRef, _x: f64, _y: f64, _z: f64) {}
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
    fn has_permission(&mut self, _p: &PlayerRef, _n: &str) -> bool {
        false
    }
    fn schedule_wait(&mut self, _t: u64, _k: u64) {}
    fn now(&mut self) -> u64 {
        0
    }
    fn log(&mut self, level: LogLevel, msg: &str) {
        self.logs.push((level, msg.to_string()));
    }
    fn advancement(&mut self, p: &PlayerRef, id: &str, crit: Option<&str>, grant: bool) -> bool {
        self.calls.push(format!(
            "adv {} {} {id} {}",
            if grant { "grant" } else { "revoke" },
            p.name,
            crit.unwrap_or("*")
        ));
        if grant {
            self.advs.push(id.to_string());
        } else {
            self.advs.retain(|a| a != id);
        }
        true
    }
    fn has_advancement(&mut self, _p: &PlayerRef, id: &str) -> bool {
        self.advs.iter().any(|a| a == id)
    }
    fn register_recipe(&mut self, r: &RecipeDef) -> Result<(), String> {
        self.recipes.push(r.clone());
        Ok(())
    }
    fn statistic(&mut self, _p: &PlayerRef, cat: &str, name: &str) -> Option<i64> {
        Some(*self.stats.get(&format!("{cat}:{name}")).unwrap_or(&0))
    }
    fn set_statistic(&mut self, _p: &PlayerRef, cat: &str, name: &str, v: i64) -> bool {
        self.stats.insert(format!("{cat}:{name}"), v);
        true
    }
}

/// A host that implements only the required methods, so the trait defaults run.
#[derive(Default)]
struct Bare {
    cmds: Vec<String>,
    warns: Vec<String>,
}

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
        self.cmds.push(c.to_string());
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
            self.warns.push(msg.to_string());
        }
    }
}

fn steve() -> PlayerRef {
    PlayerRef {
        uuid: "u1".into(),
        name: "Steve".into(),
    }
}

fn errors_of(src: &str) -> Vec<ScriptError> {
    compile_file("bad.carve", src).err().unwrap_or_default()
}

fn script(body: &str) -> String {
    let lines: String = body.lines().map(|l| format!("    {l}\n")).collect();
    format!("on join:\n{lines}")
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

fn run(body: &str) -> Mock {
    let mut e = load(body);
    let mut h = Mock::steve();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    let errors: Vec<_> = h.logs.iter().filter(|(l, _)| *l == LogLevel::Error).collect();
    assert!(errors.is_empty(), "runtime errors: {errors:?}");
    h
}

fn run_bare(body: &str) -> Bare {
    let mut e = load(body);
    let mut h = Bare::default();
    e.fire(EventKind::Join, EventCtx::player(steve()), &mut h);
    h
}

fn first_error(body: &str) -> String {
    errors_of(&script(body))
        .first()
        .map(|e| e.to_string())
        .unwrap_or_default()
}

fn item(s: &str) -> Ingredient {
    Ingredient::Item(s.into())
}

#[test]
fn grant_and_revoke_advancement() {
    let h = run(
        "grant advancement \"story/mine_stone\" to player\nrevoke advancement \"minecraft:story/mine_stone\" from player",
    );
    assert_eq!(
        h.calls,
        [
            "adv grant Steve minecraft:story/mine_stone *",
            "adv revoke Steve minecraft:story/mine_stone *"
        ]
    );
}

#[test]
fn award_and_criterion() {
    let h = run(
        "award advancement \"a/b\" to player\ngrant criterion \"c1\" of advancement \"a/b\" to player\nrevoke criterion \"c1\" of advancement \"a/b\" from player",
    );
    assert_eq!(
        h.calls,
        [
            "adv grant Steve minecraft:a/b *",
            "adv grant Steve minecraft:a/b c1",
            "adv revoke Steve minecraft:a/b c1"
        ]
    );
}

#[test]
fn advancement_default_uses_command() {
    let h = run_bare("grant advancement \"story/root\" to player");
    assert_eq!(h.cmds, ["advancement grant Steve only minecraft:story/root"]);
    let h = run_bare("revoke criterion \"x\" of advancement \"story/root\" from player");
    assert_eq!(h.cmds, ["advancement revoke Steve only minecraft:story/root x"]);
}

#[test]
fn has_advancement_condition() {
    let h = run(
        "grant advancement \"a/b\" to player\nif player has advancement \"a/b\":\n    send \"yes\" to player\nif player has not completed advancement \"a/c\":\n    send \"no c\" to player\nif player doesn't have advancement \"a/b\":\n    send \"wrong\" to player\nif player does not have advancement \"a/c\":\n    send \"no c2\" to player",
    );
    assert_eq!(h.msgs, ["yes", "no c", "no c2"]);
}

#[test]
fn bad_advancement_lines() {
    assert!(first_error("grant advancement \"a\"").contains("expected 'to <player>'"));
    assert!(first_error("revoke advancement \"a\" to player").contains("expected 'from <player>'"));
}

#[test]
fn shaped_recipe() {
    let h = run(
        "register shaped recipe \"super_sword\" for diamond sword with shape \" d \", \" d \", \" s \" where d is diamond block and s is stick",
    );
    assert_eq!(h.recipes.len(), 1);
    let r = &h.recipes[0];
    assert_eq!(r.id, "carve:super_sword");
    assert_eq!(r.result, "minecraft:diamond_sword");
    assert_eq!(r.count, 1);
    assert_eq!(
        r.kind,
        RecipeKind::Shaped {
            rows: vec![" d ".into(), " d ".into(), " s ".into()],
            key: vec![('d', item("minecraft:diamond_block")), ('s', item("minecraft:stick"))],
        }
    );
}

#[test]
fn shaped_recipe_with_text_ingredients_and_tag() {
    let h = run(
        "register shaped recipe \"my:chest\" for 2 chests with shape \"pp\", \"pp\" where p is \"#minecraft:planks\"",
    );
    let r = &h.recipes[0];
    assert_eq!(r.id, "my:chest");
    assert_eq!(r.count, 2);
    assert_eq!(
        r.kind,
        RecipeKind::Shaped {
            rows: vec!["pp".into(), "pp".into()],
            key: vec![('p', Ingredient::Tag("minecraft:planks".into()))],
        }
    );
}

#[test]
fn shaped_recipe_errors() {
    let pre = "register shaped recipe \"x\" for stick with shape ";
    assert!(first_error(&format!("{pre}\"ab\", \"a\" where a is stick and b is stick")).contains("same length"));
    assert!(first_error(&format!("{pre}\"ab\" where a is stick")).contains("'b'"));
    assert!(first_error(&format!("{pre}\"a\", \"a\", \"a\", \"a\" where a is stick")).contains("1 to 3 rows"));
    assert!(first_error(&format!("{pre}\"a\" where a is stick and a is dirt")).contains("twice"));
    assert!(first_error(&format!("{pre}\"a\" where a is banana")).contains("expected an item"));
    assert!(first_error("register shaped recipe for stick with shape \"a\"").contains("plain text id"));
}

#[test]
fn shapeless_recipe() {
    let h = run("register shapeless recipe \"torches\" for 4 torches using coal and stick");
    let r = &h.recipes[0];
    assert_eq!(r.count, 4);
    assert_eq!(
        r.kind,
        RecipeKind::Shapeless(vec![item("minecraft:coal"), item("minecraft:stick")])
    );
    let h = run("register shapeless recipe \"mix\" for stone using 3 dirt, gravel");
    assert_eq!(
        h.recipes[0].kind,
        RecipeKind::Shapeless(vec![
            item("minecraft:dirt"),
            item("minecraft:dirt"),
            item("minecraft:dirt"),
            item("minecraft:gravel")
        ])
    );
}

#[test]
fn shapeless_limit() {
    let src = "register shapeless recipe \"x\" for stone using 5 dirt, 5 gravel";
    assert!(first_error(src).contains("1 to 9"));
}

#[test]
fn cooking_recipes() {
    let h = run(
        "register furnace recipe \"fast_iron\" for iron ingot using raw iron with xp 0.7 cooking time 5 seconds",
    );
    assert_eq!(
        h.recipes[0].kind,
        RecipeKind::Cooking {
            kind: CookKind::Smelting,
            ingredient: item("minecraft:raw_iron"),
            experience: 0.7,
            ticks: 100,
        }
    );
    let h = run("register blasting recipe \"b\" for iron ingot using raw iron");
    assert_eq!(
        h.recipes[0].kind,
        RecipeKind::Cooking {
            kind: CookKind::Blasting,
            ingredient: item("minecraft:raw_iron"),
            experience: 0.0,
            ticks: 100,
        }
    );
    let h = run("register campfire recipe \"c\" for cooked beef using beef cooking time 300 ticks");
    assert!(matches!(
        h.recipes[0].kind,
        RecipeKind::Cooking {
            kind: CookKind::Campfire,
            ticks: 300,
            ..
        }
    ));
    let h = run("register smoking recipe \"s\" for cooked beef using beef");
    assert!(matches!(
        h.recipes[0].kind,
        RecipeKind::Cooking {
            kind: CookKind::Smoking,
            ticks: 100,
            ..
        }
    ));
}

#[test]
fn smithing_is_blocked_with_a_reason() {
    assert!(first_error("register smithing recipe \"x\" for stick using stick").contains("smithing"));
}

#[test]
fn discover_recipes_default_uses_command() {
    let h = run_bare(
        "discover recipe \"crafting_table\" for player\nundiscover recipe \"my:x\" for player\nunlock recipe \"stick\" to player\nlock recipe \"stick\" from player",
    );
    assert_eq!(
        h.cmds,
        [
            "recipe give Steve minecraft:crafting_table",
            "recipe take Steve my:x",
            "recipe give Steve minecraft:stick",
            "recipe take Steve minecraft:stick"
        ]
    );
}

#[test]
fn give_item_is_not_taken_by_recipe_syntax() {
    let h = run_bare("give 5 diamonds to player");
    assert_eq!(h.cmds, ["give Steve minecraft:diamond 5"]);
}

#[test]
fn statistics() {
    let h = run(
        "set statistic \"custom:jump\" of player to 5\nadd 3 to statistic \"jump\" of player\nremove 1 from statistic \"custom:jump\" of player\nset {_n} to statistic \"jump\" of player\nsend \"n=%{_n}%\" to player\nset statistic \"mined:stone\" of player to 10\nset statistic \"play time\" of player to 7",
    );
    assert_eq!(h.msgs, ["n=7"]);
    assert_eq!(h.stats.get("custom:jump"), Some(&7));
    assert_eq!(h.stats.get("mined:stone"), Some(&10));
    assert_eq!(h.stats.get("custom:play_time"), Some(&7));
}

#[test]
fn statistic_removal_stops_at_zero() {
    let h = run("remove 5 from statistic \"custom:jump\" of player");
    assert_eq!(h.stats.get("custom:jump"), Some(&0));
}

#[test]
fn statistic_names() {
    use carve_core::skbee_recipes_advancements::split_stat;
    assert_eq!(split_stat("mined:minecraft:stone").unwrap(), ("mined".into(), "stone".into()));
    assert_eq!(split_stat("minecraft:jump").unwrap(), ("custom".into(), "jump".into()));
    assert_eq!(split_stat("Walk One Cm").unwrap(), ("custom".into(), "walk_one_cm".into()));
    assert!(split_stat("").is_err());
}

#[test]
fn statistics_default_host_warns() {
    let h = run_bare("set statistic \"jump\" of player to 1");
    assert_eq!(h.warns.len(), 1);
}

#[test]
fn advancement_event() {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[(
        "t.carve".into(),
        "on advancement done:\n    send \"got %advancement id%\" to player\n    send \"also %advancement id%\" to player\n"
            .into(),
    )]);
    assert!(errs.is_empty(), "{errs:?}");
    let mut h = Mock::steve();
    e.fire(
        EventKind::AdvancementDone,
        EventCtx::chat(steve(), "minecraft:story/root"),
        &mut h,
    );
    assert_eq!(h.msgs, ["got minecraft:story/root", "also minecraft:story/root"]);
}

#[test]
fn recipe_discover_event() {
    let mut e = Engine::default();
    let (_, errs) = e.load(&[(
        "t.carve".into(),
        "on recipe discover:\n    send \"found %recipe id%\" to player\n".into(),
    )]);
    assert!(errs.is_empty(), "{errs:?}");
    let mut h = Mock::steve();
    e.fire(
        EventKind::RecipeDiscover,
        EventCtx::chat(steve(), "minecraft:stick"),
        &mut h,
    );
    assert_eq!(h.msgs, ["found minecraft:stick"]);
}

#[test]
fn event_ids_only_in_their_events() {
    let errs = errors_of("on join:\n    send \"%advancement id%\" to player\n");
    assert!(!errs.is_empty());
}

#[test]
fn event_names() {
    for (name, kind) in [
        ("advancement done", EventKind::AdvancementDone),
        ("advancement", EventKind::AdvancementDone),
        ("recipe discover", EventKind::RecipeDiscover),
    ] {
        let mut e = Engine::default();
        let (_, errs) = e.load(&[("t.carve".into(), format!("on {name}:\n    send \"x\" to player\n"))]);
        assert!(errs.is_empty(), "{name}: {errs:?}");
        let mut h = Mock::steve();
        e.fire(kind, EventCtx::player(steve()), &mut h);
        assert_eq!(h.msgs, ["x"], "{name}");
    }
}

#[test]
fn registry_still_valid() {
    carve_core::compile::check_registry().unwrap();
}

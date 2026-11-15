//! Two real scripts from a minigame server must load with zero errors.
//! Functions the scripts take from other files are stubbed here.

use carve_core::*;

const STUBS: &str = "\
function zombGui(p: player):
  send \"x\" to {_p}
function stopGame():
  broadcast \"x\"
function giveItems(p: player):
  send \"x\" to {_p}
function vipReward(p: player, n: number) :: number:
  return {_n}
function queueAudit(a: text, b: text, c: text):
  broadcast {_a}
function clearZombieClassLock(p: player):
  send \"x\" to {_p}
function replaceAlphaZombie():
  broadcast \"x\"
function findZombie():
  broadcast \"x\"
function restartGame():
  broadcast \"x\"
function createBossbar():
  broadcast \"x\"
function shopGui(p: player):
  send \"x\" to {_p}
function giveWalkerKit(p: player):
  send \"x\" to {_p}
function giveLobbyGravel(p: player):
  send \"x\" to {_p}
function reconcileZombieClassLocks():
  broadcast \"x\"
";

/// Functions the third script takes from its server's other file.
const STUBS_KARELIA: &str = "function kMapList() :: objects:
  return \"a\", \"b\"
function kSpawnFor(t: text) :: location:
  return location(0, 0, 0)
function kMode() :: text:
  return \"x\"
function kFlagName(f: object) :: text:
  return \"x\"
function kBreakFlag() :: number:
  return 1
function kSiteActive(f: object) :: boolean:
  return true
function kDist(a: location, b: location) :: number:
  return 0
function kGroundAt(x: number, y: number, z: number, w: world) :: location:
  return location({_x}, {_y}, {_z}, {_w})
function kOther(t: text) :: text:
  return {_t}
function kKit(p: player):
  broadcast \"x\"
function kCount(t: text) :: number:
  return 0
function kAttacker(t: text) :: boolean:
  return true
function kPlaceAt(l: location, k: text):
  broadcast \"x\"
function kGround(l: object) :: location:
  return location(0, 0, 0)
function kPlace(b: player, l: location, k: text) :: number:
  return 1
function kBuildCost(k: text) :: number:
  return 1
function kClassLimit(c: text) :: number:
  return 1
function kCoverAt(l: location) :: location:
  return {_l}
function kPlant(l: location):
  broadcast \"x\"
function kRankName(p: player) :: text:
  return \"x\"
function kGlobalRank(p: player) :: number:
  return 1
";

#[test]
fn user_scripts_load_cleanly() {
    let mut e = Engine::default();
    let files = vec![
        ("stubs.carve".to_string(), STUBS.to_string()),
        (
            "core.carve".to_string(),
            include_str!("user_core.carve").to_string(),
        ),
        (
            "combat.carve".to_string(),
            include_str!("user_combat.carve").to_string(),
        ),
        ("karelia_stubs.carve".to_string(), STUBS_KARELIA.to_string()),
        (
            "kareliabots.carve".to_string(),
            include_str!("user_kareliabots.carve").to_string(),
        ),
    ];
    let (_, errs) = e.load(&files);
    let msg: Vec<String> = errs
        .iter()
        .map(|e| format!("{}:{} {}", e.file, e.line, e.msg))
        .collect();
    assert!(errs.is_empty(), "{} errors:\n{}", msg.len(), msg.join("\n"));
    // Only the lines that need another plugin or something Pumpkin can't do
    // may warn: WorldGuard `region at`, a chat plugin's `prefix`, the tab
    // complete event and suggestions, and `explosive yield`.
    let mut got: Vec<(String, usize)> = e.warnings.iter().map(|w| (w.file.clone(), w.line)).collect();
    got.sort();
    let mut want: Vec<(String, usize)> = [
        ("core.carve", 11),
        ("core.carve", 13),
        ("core.carve", 17),
        ("core.carve", 19),
        ("core.carve", 23),
        ("core.carve", 24),
        ("core.carve", 226),
        ("core.carve", 311),
        ("core.carve", 399),
        ("core.carve", 530),
        ("core.carve", 533),
        ("core.carve", 535),
        ("core.carve", 537),
        ("core.carve", 552),
        ("core.carve", 609),
        ("kareliabots.carve", 115),
        ("kareliabots.carve", 119),
        ("kareliabots.carve", 128),
    ]
    .iter()
    .map(|(f, l)| (f.to_string(), *l))
    .collect();
    want.sort();
    assert_eq!(got, want, "warnings: {:#?}", e.warnings);
    for w in &e.warnings {
        let m = &w.msg;
        assert!(
            m.contains("WorldGuard")
                || m.contains("chat or permission plugin")
                || m.contains("tab complete")
                || m.contains("suggestions")
                || m.contains("explosive yield"),
            "unexpected warning: {m}"
        );
    }
}

/// A trigger or section with only a comment under it still loads the rest.
#[test]
fn empty_section_with_only_a_comment_warns() {
    let src = "on join:\n  if {x} is true:\n    # nothing yet\n  send \"hi\"\n";
    let mut e = Engine::default();
    let (_, errs) = e.load(&[("t.carve".to_string(), src.to_string())]);
    assert!(errs.is_empty(), "{errs:?}");
    assert_eq!(e.warnings.len(), 1);
    assert_eq!(e.warnings[0].line, 2);
}

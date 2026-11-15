//! Extra game events (heal, grow, explode, fishing, furnace, items, ...).
//!
//! All of them share one `EventKind::Ext2(Ev)` variant, so the hot event enum
//! stays small. Each event has a [`Spec`]: its names in `on ...:`, whether it
//! has a player, whether it can be cancelled, and its event values. Values
//! live in an [`ExtVals`] box on the `EventCtx`, read by `Expr::Ev2Val`.

use crate::engine::EventCtx;
use crate::expr::{Env, Expr, Ty};
use crate::lexer::Tok;
use crate::value::Value;

/// Where a value sits in `ExtVals`: a number, a text or a yes/no flag.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Slot {
    Num(u8),
    Text(u8),
    Flag(u8),
}

impl Slot {
    pub fn ty(self) -> Ty {
        match self {
            Slot::Num(_) => Ty::Number,
            Slot::Text(_) => Ty::Text,
            Slot::Flag(_) => Ty::Boolean,
        }
    }
}

/// Values carried by one event, read with `Expr::Ev2Val`.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ExtVals {
    pub nums: Vec<f64>,
    pub texts: Vec<String>,
    pub flags: Vec<bool>,
}

/// Read one value from a context. Missing values are `none`.
pub fn read(ctx: &EventCtx, s: Slot) -> Value {
    let Some(v) = ctx.ext.as_deref() else {
        return Value::None;
    };
    match s {
        Slot::Num(i) => v.nums.get(i as usize).map(|n| Value::Num(*n)),
        Slot::Text(i) => v.texts.get(i as usize).map(|t| Value::Text(t.clone())),
        Slot::Flag(i) => v.flags.get(i as usize).map(|b| Value::Bool(*b)),
    }
    .unwrap_or(Value::None)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ev {
    Enchant,
    EnchantPrepare,
    ChangeBlock,
    Breed,
    Dismount,
    Mount,
    PotionEffect,
    ShootBow,
    Transform,
    XpChange,
    XpCooldown,
    Explode,
    ExplosionPrime,
    Fade,
    FireworkExplode,
    Fishing,
    Flow,
    Form,
    FuelBurn,
    FurnaceExtract,
    Glide,
    Grow,
    Harvest,
    Heal,
    HorseJump,
    Hunger,
    Ignition,
    InvDrag,
    InvMove,
    InvOpen,
    InvPickup,
    ItemBreak,
    ItemDamage,
    ItemDespawn,
    ItemMend,
    ItemMerge,
    ItemSpawn,
}

/// Does the event have a player?
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Who {
    /// Always a player: `player` works.
    Player,
    /// Any entity; `player` is set only when it is a player.
    Maybe,
    /// Never a player (`player` is an error in the script).
    Nobody,
}

pub struct Spec {
    pub ev: Ev,
    /// Names after `on `, lowercase. The first one is the display name.
    pub names: &'static [&'static str],
    pub cancel: bool,
    pub who: Who,
    /// How many numbers, texts and flags the host must fill in.
    pub counts: (u8, u8, u8),
    /// Event values by name (lowercase words). The first of each kind is also
    /// reachable as `event-number`, `event-text` and `event-boolean`.
    pub vals: &'static [(&'static str, Slot)],
}

use Slot::{Flag as F, Num as N, Text as T};
use Who::{Maybe, Nobody, Player};

const XYZ: &[(&str, Slot)] = &[("block x", N(0)), ("block y", N(1)), ("block z", N(2))];

pub const SPECS: &[Spec] = &[
    Spec {
        ev: Ev::Enchant,
        names: &["enchant", "enchant item", "item enchant"],
        cancel: true,
        who: Player,
        counts: (4, 2, 0),
        vals: &[
            ("enchanted item", T(0)),
            ("event-item", T(0)),
            ("enchant item", T(0)),
            ("enchant option", N(0)),
            ("enchantment cost", N(1)),
            ("enchant cost", N(1)),
            // Wave 5 chunk 4: Skript's names for the cost and the hint.
            ("enchanting cost", N(1)),
            ("experience cost", N(1)),
            ("exp cost", N(1)),
            ("xp cost", N(1)),
            ("displayed experience cost", N(1)),
            ("displayed exp cost", N(1)),
            ("enchantment count", N(2)),
            ("enchantment hint", T(1)),
            ("enchant hint", T(1)),
            ("enchantment hint level", N(3)),
            ("enchant hint level", N(3)),
        ],
    },
    Spec {
        ev: Ev::EnchantPrepare,
        names: &["enchant prepare", "prepare enchant", "prepare item enchant"],
        cancel: true,
        who: Player,
        // Numbers 2-10: cost, enchantment id and level of the three offers.
        counts: (11, 1, 0),
        vals: &[
            ("enchanted item", T(0)),
            ("event-item", T(0)),
            ("enchant item", T(0)),
            ("bookshelf count", N(0)),
            ("bookshelves", N(0)),
            ("enchantment bonus", N(0)),
            ("enchanting bonus", N(0)),
            ("enchant bonus", N(0)),
            ("enchant offer count", N(1)),
            ("first offer cost", N(2)),
            ("first offer enchantment id", N(3)),
            ("first offer level", N(4)),
            ("second offer cost", N(5)),
            ("second offer enchantment id", N(6)),
            ("second offer level", N(7)),
            ("third offer cost", N(8)),
            ("third offer enchantment id", N(9)),
            ("third offer level", N(10)),
        ],
    },
    Spec {
        ev: Ev::ChangeBlock,
        names: &["enderman/sheep/silverfish/falling block", "entity change block"],
        cancel: true,
        who: Nobody,
        counts: (4, 1, 0),
        vals: &[
            ("new block", T(0)),
            ("event-entity", N(0)),
            ("entity id", N(0)),
            ("block x", N(1)),
            ("block y", N(2)),
            ("block z", N(3)),
        ],
    },
    Spec {
        ev: Ev::Breed,
        names: &["entity breed", "breed", "breeding"],
        cancel: true,
        who: Nobody,
        counts: (3, 0, 0),
        vals: &[
            ("child id", N(2)),
            ("event-entity", N(2)),
            ("father id", N(0)),
            ("mother id", N(1)),
        ],
    },
    Spec {
        ev: Ev::Dismount,
        names: &["entity dismount", "dismount", "vehicle exit"],
        cancel: true,
        who: Maybe,
        counts: (2, 0, 0),
        vals: &[
            ("event-entity", N(0)),
            ("entity id", N(0)),
            ("dismounted id", N(1)),
            ("vehicle id", N(1)),
        ],
    },
    Spec {
        ev: Ev::Mount,
        names: &["entity mount", "mount", "vehicle enter"],
        cancel: true,
        who: Maybe,
        counts: (2, 0, 0),
        vals: &[
            ("event-entity", N(0)),
            ("entity id", N(0)),
            ("mounted id", N(1)),
            ("vehicle id", N(1)),
        ],
    },
    Spec {
        ev: Ev::PotionEffect,
        names: &["entity potion effect", "potion effect", "potion effect applied"],
        cancel: true,
        who: Maybe,
        counts: (3, 1, 0),
        vals: &[
            ("effect name", T(0)),
            ("effect duration", N(0)),
            ("effect amplifier", N(1)),
            ("event-entity", N(2)),
            ("entity id", N(2)),
        ],
    },
    Spec {
        ev: Ev::ShootBow,
        names: &["entity shoot bow", "shoot bow", "entity shoot"],
        cancel: true,
        who: Maybe,
        counts: (2, 1, 0),
        vals: &[
            ("bow item", T(0)),
            ("bow force", N(0)),
            ("event-entity", N(1)),
            ("entity id", N(1)),
        ],
    },
    Spec {
        ev: Ev::Transform,
        names: &["entity transform", "transform"],
        cancel: true,
        who: Maybe,
        counts: (2, 1, 0),
        vals: &[
            ("transform reason", T(0)),
            ("event-entity", N(0)),
            ("entity id", N(0)),
            ("new entity id", N(1)),
        ],
    },
    Spec {
        ev: Ev::XpChange,
        names: &["experience change", "xp change", "exp change"],
        cancel: false,
        who: Player,
        counts: (1, 0, 0),
        vals: &[
            ("experience amount", N(0)),
            ("xp amount", N(0)),
            ("exp amount", N(0)),
        ],
    },
    Spec {
        ev: Ev::XpCooldown,
        names: &[
            "experience cooldown change",
            "xp cooldown change",
            "exp cooldown change",
        ],
        cancel: true,
        who: Player,
        counts: (1, 0, 0),
        vals: &[("new cooldown", N(0)), ("experience cooldown", N(0))],
    },
    Spec {
        ev: Ev::Explode,
        names: &["explode", "explosion", "entity explode", "block explode"],
        cancel: true,
        who: Nobody,
        counts: (5, 0, 0),
        vals: &[
            ("explosion yield", N(0)),
            ("explosion x", N(1)),
            ("explosion y", N(2)),
            ("explosion z", N(3)),
            ("event-entity", N(4)),
            ("entity id", N(4)),
        ],
    },
    Spec {
        ev: Ev::ExplosionPrime,
        names: &["explosion prime", "prime explosion", "tnt prime"],
        cancel: true,
        who: Nobody,
        counts: (2, 0, 1),
        vals: &[
            ("explosion radius", N(0)),
            ("explosion fire", F(0)),
            ("event-entity", N(1)),
            ("entity id", N(1)),
        ],
    },
    Spec {
        ev: Ev::Fade,
        names: &["fade", "block fade"],
        cancel: true,
        who: Nobody,
        counts: (3, 0, 0),
        vals: XYZ,
    },
    Spec {
        ev: Ev::FireworkExplode,
        names: &["firework explode", "firework"],
        cancel: true,
        who: Nobody,
        counts: (1, 0, 0),
        vals: &[("event-entity", N(0)), ("entity id", N(0))],
    },
    Spec {
        ev: Ev::Fishing,
        names: &["fishing", "fish", "player fish"],
        cancel: true,
        who: Player,
        counts: (1, 2, 0),
        vals: &[
            ("fishing state", T(0)),
            ("caught type", T(1)),
            ("fishing xp", N(0)),
            ("fishing experience", N(0)),
        ],
    },
    Spec {
        ev: Ev::Flow,
        names: &["flow", "liquid flow", "block flow"],
        cancel: true,
        who: Nobody,
        counts: (6, 0, 0),
        vals: &[
            ("flow source x", N(0)),
            ("flow source y", N(1)),
            ("flow source z", N(2)),
            ("flow target x", N(3)),
            ("flow target y", N(4)),
            ("flow target z", N(5)),
        ],
    },
    Spec {
        ev: Ev::Form,
        names: &["form", "block form"],
        cancel: true,
        who: Nobody,
        counts: (3, 0, 0),
        vals: XYZ,
    },
    Spec {
        ev: Ev::FuelBurn,
        names: &["fuel burn", "furnace burn"],
        cancel: true,
        who: Nobody,
        counts: (4, 1, 0),
        vals: &[
            ("fuel item", T(0)),
            ("event-item", T(0)),
            ("burn time", N(3)),
            ("block x", N(0)),
            ("block y", N(1)),
            ("block z", N(2)),
        ],
    },
    Spec {
        ev: Ev::FurnaceExtract,
        names: &["furnace item extract", "furnace extract"],
        cancel: false,
        who: Player,
        counts: (5, 1, 0),
        vals: &[
            ("extracted item", T(0)),
            ("event-item", T(0)),
            ("extracted amount", N(3)),
            ("extracted xp", N(4)),
            ("block x", N(0)),
            ("block y", N(1)),
            ("block z", N(2)),
        ],
    },
    Spec {
        ev: Ev::Glide,
        names: &[
            "gliding state change",
            "glide toggle",
            "toggle glide",
            "gliding change",
        ],
        cancel: true,
        who: Maybe,
        counts: (1, 0, 1),
        vals: &[
            ("gliding state", F(0)),
            ("event-entity", N(0)),
            ("entity id", N(0)),
        ],
    },
    Spec {
        ev: Ev::Grow,
        names: &["grow", "block grow", "plant grow"],
        cancel: true,
        who: Nobody,
        counts: (3, 2, 0),
        vals: &[
            ("new block", T(1)),
            ("old block", T(0)),
            ("block x", N(0)),
            ("block y", N(1)),
            ("block z", N(2)),
        ],
    },
    Spec {
        ev: Ev::Harvest,
        names: &["harvest block", "harvest", "player harvest block"],
        cancel: true,
        who: Player,
        counts: (4, 0, 0),
        vals: &[
            ("harvested count", N(3)),
            ("block x", N(0)),
            ("block y", N(1)),
            ("block z", N(2)),
        ],
    },
    Spec {
        ev: Ev::Heal,
        names: &["heal", "entity heal", "regain health", "entity regain health"],
        cancel: true,
        who: Maybe,
        counts: (2, 0, 0),
        vals: &[("heal amount", N(0)), ("event-entity", N(1)), ("entity id", N(1))],
    },
    Spec {
        ev: Ev::HorseJump,
        names: &["horse jump"],
        cancel: true,
        who: Nobody,
        counts: (2, 0, 0),
        vals: &[("jump power", N(0)), ("event-entity", N(1)), ("entity id", N(1))],
    },
    Spec {
        ev: Ev::Hunger,
        names: &[
            "hunger meter change",
            "hunger change",
            "food level change",
            "food change",
        ],
        cancel: true,
        who: Maybe,
        counts: (2, 0, 0),
        vals: &[
            ("new food level", N(0)),
            ("food level", N(0)),
            ("event-entity", N(1)),
            ("entity id", N(1)),
        ],
    },
    Spec {
        ev: Ev::Ignition,
        names: &["ignition", "block ignite", "ignite"],
        cancel: true,
        who: Nobody,
        counts: (3, 0, 0),
        vals: XYZ,
    },
    Spec {
        ev: Ev::InvDrag,
        names: &["inventory drag", "drag inventory"],
        cancel: true,
        who: Player,
        counts: (0, 0, 0),
        vals: &[],
    },
    Spec {
        ev: Ev::InvMove,
        names: &["inventory item move", "inventory move", "item move"],
        cancel: true,
        who: Nobody,
        counts: (7, 1, 0),
        vals: &[
            ("moved item", T(0)),
            ("event-item", T(0)),
            ("moved amount", N(6)),
            ("move source x", N(0)),
            ("move source y", N(1)),
            ("move source z", N(2)),
            ("move target x", N(3)),
            ("move target y", N(4)),
            ("move target z", N(5)),
        ],
    },
    Spec {
        ev: Ev::InvOpen,
        names: &["inventory open", "open inventory"],
        cancel: true,
        who: Player,
        counts: (0, 0, 0),
        vals: &[],
    },
    Spec {
        ev: Ev::InvPickup,
        names: &["inventory pickup", "inventory pick up", "hopper pickup"],
        cancel: true,
        who: Nobody,
        counts: (4, 1, 0),
        vals: &[
            ("picked up item", T(0)),
            ("event-item", T(0)),
            ("item entity id", N(3)),
            ("block x", N(0)),
            ("block y", N(1)),
            ("block z", N(2)),
        ],
    },
    Spec {
        ev: Ev::ItemBreak,
        names: &["item break", "tool break"],
        cancel: false,
        who: Player,
        counts: (0, 1, 0),
        vals: &[("broken item", T(0)), ("event-item", T(0))],
    },
    Spec {
        ev: Ev::ItemDamage,
        names: &["item damage", "tool damage"],
        cancel: true,
        who: Player,
        counts: (1, 1, 0),
        vals: &[
            ("damaged item", T(0)),
            ("event-item", T(0)),
            ("item damage amount", N(0)),
        ],
    },
    Spec {
        ev: Ev::ItemDespawn,
        names: &["item despawn", "dropped item despawn"],
        cancel: true,
        who: Nobody,
        counts: (1, 0, 0),
        vals: &[("event-entity", N(0)), ("entity id", N(0))],
    },
    Spec {
        ev: Ev::ItemMend,
        names: &["item mend", "mending"],
        cancel: true,
        who: Player,
        counts: (2, 1, 0),
        vals: &[
            ("mended item", T(0)),
            ("event-item", T(0)),
            ("repair amount", N(0)),
            ("mending xp", N(1)),
        ],
    },
    Spec {
        ev: Ev::ItemMerge,
        names: &["item merge", "dropped item merge"],
        cancel: true,
        who: Nobody,
        counts: (2, 0, 0),
        vals: &[
            ("event-entity", N(0)),
            ("entity id", N(0)),
            ("merge target id", N(1)),
        ],
    },
    Spec {
        ev: Ev::ItemSpawn,
        names: &["item spawn", "dropped item spawn"],
        cancel: true,
        who: Nobody,
        counts: (4, 1, 0),
        vals: &[
            ("spawned item", T(0)),
            ("event-item", T(0)),
            ("event-entity", N(0)),
            ("entity id", N(0)),
            ("block x", N(1)),
            ("block y", N(2)),
            ("block z", N(3)),
        ],
    },
];

pub fn spec(ev: Ev) -> &'static Spec {
    SPECS.iter().find(|s| s.ev == ev).expect("every event has a spec")
}

/// The event named by the text after `on ` (lowercase, trimmed).
pub fn parse(s: &str) -> Option<Ev> {
    SPECS.iter().find(|sp| sp.names.contains(&s)).map(|sp| sp.ev)
}

/// All names, for the "known events" error text.
pub fn known_names() -> String {
    SPECS.iter().map(|s| s.names[0]).collect::<Vec<_>>().join(", ")
}

/// Fix the `Env` flags for one of these events.
pub fn apply_env(env: &mut Env, ev: Ev) {
    env.ext_event = Some(ev);
    env.no_player = spec(ev).who == Who::Nobody;
}

/// Try to read an event value at `word` followed by the tokens in `rest`.
/// Returns the expression and how many extra tokens it used.
pub fn parse_value(ev: Ev, word: &str, rest: &[Tok], need_more: &mut bool) -> Option<(Expr, usize)> {
    let sp = spec(ev);
    match word {
        "event-player" if sp.who != Who::Nobody => return Some((Expr::Player, 0)),
        "event-number" => {
            return sp
                .vals
                .iter()
                .find(|(_, s)| matches!(s, Slot::Num(_)))
                .map(|(_, s)| (Expr::Ev2Val(*s), 0));
        }
        "event-text" => {
            return sp
                .vals
                .iter()
                .find(|(_, s)| matches!(s, Slot::Text(_)))
                .map(|(_, s)| (Expr::Ev2Val(*s), 0));
        }
        "event-boolean" => {
            return sp
                .vals
                .iter()
                .find(|(_, s)| matches!(s, Slot::Flag(_)))
                .map(|(_, s)| (Expr::Ev2Val(*s), 0));
        }
        _ => {}
    }
    let mut best: Option<(Slot, usize)> = None;
    for (name, slot) in sp.vals {
        let mut words = name.split(' ');
        if words.next() != Some(word) {
            continue;
        }
        let mut used = 0;
        let mut ok = true;
        for w in words {
            match rest.get(used) {
                Some(t) if t.is_lit(w) => used += 1,
                // The span ended inside a longer name: a longer span may match.
                None => {
                    *need_more = true;
                    ok = false;
                    break;
                }
                _ => {
                    ok = false;
                    break;
                }
            }
        }
        if ok && best.is_none_or(|(_, u)| used > u) {
            best = Some((*slot, used));
        }
    }
    best.map(|(s, u)| (Expr::Ev2Val(s), u))
}

/// Build a context. `nums`, `texts` and `flags` must match the spec counts.
pub fn make_ctx(
    ev: Ev,
    player: Option<crate::value::PlayerRef>,
    nums: &[f64],
    texts: &[&str],
    flags: &[bool],
) -> EventCtx {
    let mut c = match player {
        Some(p) => EventCtx::player(p),
        None => EventCtx::console(),
    };
    debug_assert_eq!(
        (nums.len(), texts.len(), flags.len()),
        {
            let k = spec(ev).counts;
            (k.0 as usize, k.1 as usize, k.2 as usize)
        },
        "wrong value counts for {ev:?}"
    );
    c.ext = Some(Box::new(ExtVals {
        nums: nums.to_vec(),
        texts: texts.iter().map(|s| s.to_string()).collect(),
        flags: flags.to_vec(),
    }));
    c
}

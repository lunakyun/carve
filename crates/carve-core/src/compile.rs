//! Compiles a script file into triggers holding flat instruction lists.

use std::sync::{Arc, OnceLock};

use crate::blocks::{Line, parse_blocks};
use crate::error::ScriptError;
use crate::expr::{Env, Expr, LoopKind, Sig, Ty, VarRef, parse_expr, parse_template, parse_varref};
use crate::lexer::{Tok, lex};
use crate::list_ops::ListSrc;
use crate::op_kick_ban::{AdminCond, AdminEff};
use crate::pattern::{Capture, Memo, Pattern, closest};
use crate::player_flags::{PLAYER_FLAG_PATTERNS, PlayerFlag};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum EventKind {
    Join,
    Quit,
    Chat,
    Break,
    Place,
    Death,
    Damage,
    Respawn,
    /// Any click (left or right).
    Click,
    RightClick,
    LeftClick,
    /// After scripts load or reload.
    Load,
    SneakToggle,
    SprintToggle,
    FlightToggle,
    Drop,
    PickUp,
    Consume,
    ProjectileHit,
    MobSpawn,
    GamemodeChange,
    Craft,
    LevelChange,
    /// A player clicks a slot in an open inventory.
    InventoryClick,
    InventoryClose,
    /// A player walks into a bound.
    BoundEnter,
    /// A player walks out of a bound.
    BoundExit,
    /// `on advancement done`: `advancement id` is the advancement.
    AdvancementDone,
    /// `on recipe discover`: `recipe id` is the recipe.
    RecipeDiscover,
    /// Events and `at <time>:` triggers of `w2_events_1`.
    Ext(crate::w2_events_1::ExtEvent),
    /// The extra events of `w2_events_2`.
    Ext2(crate::w2_events_2::Ev),
    /// Extra events, see `w2_events_3`.
    Ext3(crate::w2_events_3::Ev3),
    /// Events of `w2_events_4`.
    Ext4(crate::w2_events_4::ExtEvent),
    /// Movement and command events of `w4_events_movement`.
    Mv(crate::w4_events_movement::MvEvent),
}

impl EventKind {
    pub fn name(self) -> &'static str {
        match self {
            EventKind::Join => "join",
            EventKind::Quit => "quit",
            EventKind::Chat => "chat",
            EventKind::Break => "break",
            EventKind::Place => "place",
            EventKind::Death => "death",
            EventKind::Damage => "damage",
            EventKind::Respawn => "respawn",
            EventKind::Click => "click",
            EventKind::RightClick => "right click",
            EventKind::LeftClick => "left click",
            EventKind::Load => "load",
            EventKind::SneakToggle => "sneak toggle",
            EventKind::SprintToggle => "sprint toggle",
            EventKind::FlightToggle => "flight toggle",
            EventKind::Drop => "drop",
            EventKind::PickUp => "pick up",
            EventKind::Consume => "consume",
            EventKind::ProjectileHit => "projectile hit",
            EventKind::MobSpawn => "mob spawn",
            EventKind::GamemodeChange => "gamemode change",
            EventKind::Craft => "craft",
            EventKind::LevelChange => "level change",
            EventKind::InventoryClick => "inventory click",
            EventKind::InventoryClose => "inventory close",
            EventKind::BoundEnter => "bound enter",
            EventKind::BoundExit => "bound exit",
            EventKind::AdvancementDone => "advancement done",
            EventKind::RecipeDiscover => "recipe discover",
            EventKind::Ext(e) => e.name(),
            EventKind::Ext2(e) => crate::w2_events_2::spec(e).names[0],
            EventKind::Ext3(e) => e.name(),
            EventKind::Ext4(e) => e.name(),
            EventKind::Mv(e) => e.name(),
        }
    }

    fn parse(s: &str) -> Option<Self> {
        Some(match s {
            "join" | "player join" | "login" => EventKind::Join,
            "quit" | "leave" | "disconnect" | "player quit" => EventKind::Quit,
            "chat" | "player chat" => EventKind::Chat,
            "break" | "block break" | "mine" => EventKind::Break,
            "place" | "block place" => EventKind::Place,
            "death" | "death of player" | "player death" => EventKind::Death,
            "damage" | "damage of player" | "player damage" => EventKind::Damage,
            "respawn" | "player respawn" => EventKind::Respawn,
            "click" | "rightclick" | "right click" => {
                if s == "click" {
                    EventKind::Click
                } else {
                    EventKind::RightClick
                }
            }
            "leftclick" | "left click" => EventKind::LeftClick,
            "load" | "script load" | "server start" | "skript load" => EventKind::Load,
            "sneak toggle" | "toggle sneak" | "player toggle sneak" => EventKind::SneakToggle,
            "sprint toggle" | "toggle sprint" | "player toggle sprint" => EventKind::SprintToggle,
            "flight toggle" | "toggle flight" | "player toggle flight" => EventKind::FlightToggle,
            "inventory click" | "inventory clicking" | "inventory click event" => EventKind::InventoryClick,
            "inventory close" | "inventory closing" | "inventory close event" | "close inventory" => {
                EventKind::InventoryClose
            }
            "advancement done"
            | "advancement"
            | "player advancement"
            | "advancement complete"
            | "advancement completed"
            | "advancement get" => EventKind::AdvancementDone,
            "recipe discover" | "recipe discovered" | "player recipe discover" | "recipe unlock" => {
                EventKind::RecipeDiscover
            }
            _ => {
                return crate::events_items::parse_kind(s)
                    .or_else(|| crate::skbee_bound::parse_event(s))
                    .or_else(|| crate::w2_events_4::ExtEvent::parse(s).map(EventKind::Ext4))
                    .or_else(|| crate::w2_events_1::ExtEvent::parse(s).map(EventKind::Ext))
                    .or_else(|| crate::w2_events_2::parse(s).map(EventKind::Ext2))
                    .or_else(|| crate::w2_events_3::Ev3::parse(s).map(EventKind::Ext3))
                    .or_else(|| crate::w4_events_movement::MvEvent::parse(s).map(EventKind::Mv));
            }
        })
    }

    /// Whether `cancel event` means anything here.
    pub fn cancellable(self) -> bool {
        match self {
            EventKind::Ext(e) => e.cancellable(),
            EventKind::Ext2(e) => crate::w2_events_2::spec(e).cancel,
            EventKind::Ext4(e) => e.cancellable(),
            EventKind::Mv(e) => e.cancellable(),
            _ => !matches!(
                self,
                EventKind::Respawn
                    | EventKind::Load
                    | EventKind::LevelChange
                    | EventKind::InventoryClose
                    | EventKind::BoundEnter
                    | EventKind::BoundExit
            ),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ArgTy {
    Text,
    Number,
    Player,
}

#[derive(Debug, Clone, PartialEq)]
pub struct ArgDef {
    pub ty: ArgTy,
    pub optional: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct CommandDef {
    pub name: String,
    pub permission: Option<String>,
    pub description: Option<String>,
    pub args: Vec<ArgDef>,
    /// Other names for the command (`aliases: /h, /hi`).
    pub aliases: Vec<String>,
    /// Message for wrong arguments instead of the generated usage.
    pub usage_msg: Option<String>,
    /// `executable by: players` (true) or `console` (false).
    pub players_only: Option<bool>,
    /// `cooldown:`, `cooldown message:` and `cooldown bypass:`.
    pub cooldown: Option<Box<crate::w5_chunk1::Cooldown>>,
}

impl CommandDef {
    pub fn usage(&self) -> String {
        let mut s = format!("/{}", self.name);
        for a in &self.args {
            let t = match a.ty {
                ArgTy::Text => "text",
                ArgTy::Number => "number",
                ArgTy::Player => "player",
            };
            if a.optional {
                s.push_str(&format!(" [<{t}>]"));
            } else {
                s.push_str(&format!(" <{t}>"));
            }
        }
        s
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum TriggerKind {
    Event(EventKind),
    Command(CommandDef),
    /// `every 5 seconds:`, period in ticks.
    Every(u64),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CmpOp {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Cond {
    PlayerFlag {
        player: Expr,
        flag: PlayerFlag,
        negate: bool,
    },
    Cmp(Expr, CmpOp, Expr),
    /// Rare conditions, boxed so this enum stays small.
    Rare3(Box<crate::w2_conditions_3::W2Cond>),
    Rare4(Box<crate::w4_conditions_1::W4Cond>),
    /// Conditions from `w5_chunk2`.
    W5c2(Box<crate::w5_chunk2::C2Cond>),
    /// Tag and loaded checks (`w4_skbee_raytrace_world`).
    Rtw(Box<crate::w4_skbee_raytrace_world::RtwCond>),
    /// Mob, item, block and effect checks (`w5_chunk3`).
    W5c3(Box<crate::w5_chunk3::W5Cond>),
    /// `can age` (`w5_chunk1`).
    W5c1(Box<crate::w5_chunk1::CanAge>),
    /// `{_gui} contains diamond` (see `gaps`).
    Gap(Box<crate::gaps::GapCond>),
    HasPerm {
        player: Expr,
        node: Expr,
        negate: bool,
    },
    IsSet(Expr, bool),
    /// Rare conditions (boxed to keep this enum small).
    Rare(Box<crate::w2_conditions_2::Rare>),
    /// `chance of 25%`
    Chance(Expr),
    /// `player has 2 diamonds`, `player's inventory contains stone`.
    HasItem {
        player: Expr,
        item: Expr,
        negate: bool,
    },
    /// `item is enchanted [with sharpness 5]`; `ench` is `None` for any enchantment.
    ItemEnch {
        item: Expr,
        ench: Option<Expr>,
        negate: bool,
    },
    /// Every part is true (`a and b`).
    All(Vec<Cond>),
    /// At least one part is true (`a or b`).
    Any(Vec<Cond>),
    /// `x is a or b`: equal to any of the values (negated: equal to none).
    AnyOf {
        value: Expr,
        options: Vec<Expr>,
        negate: bool,
    },
    Text {
        left: Expr,
        op: TextOp,
        right: Expr,
        negate: bool,
    },
    Between {
        value: Expr,
        low: Expr,
        high: Expr,
        negate: bool,
    },
    /// `sender is a player` / `sender is console` (and the negations).
    SenderIs {
        value: Expr,
        console: bool,
        negate: bool,
    },
    /// `player is op`, `player is banned`.
    Admin(AdminCond, Expr, bool),
    /// `player has potion effect speed`.
    World(Box<crate::world_effects::WorldCond>),
    /// `block at loc is stone`.
    Block(Box<crate::block_ops::BlockCond>),
    /// Bound conditions (`is within bound`, `bound exists`, ...).
    Bound(Box<crate::skbee_bound::BoundCond>),
    /// `it is raining`, `it is night`, `weather is clear`.
    Wt(Box<crate::weather_time::WtCond>),
    /// `team "x" exists`, `player is in team "x"`; the bool is negation.
    Sb(Box<crate::skbee_scoreboard_teams::SbCond>, bool),
    /// `{_nbt} has tag "x"`.
    Nbt(Box<crate::skbee_nbt::NbtCond>),
    /// `last spawned entity is a zombie`.
    Ent(Box<crate::entities::EntCond>),
    /// `player has advancement "x"`.
    Skb(Box<crate::skbee_recipes_advancements::SkbCond>),
    /// `player is whitelisted`.
    Server(Box<crate::server_misc::ServerCond>),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TextOp {
    Contains,
    StartsWith,
    EndsWith,
}

#[derive(Debug, Clone, PartialEq)]
pub enum Effect {
    Send {
        text: Expr,
        to: Option<Expr>,
    },
    Broadcast(Expr),
    Title {
        title: Expr,
        subtitle: Option<Expr>,
        to: Expr,
    },
    ActionBar {
        text: Expr,
        to: Expr,
    },
    Heal(Expr),
    Hf(Box<crate::health_food::HfEffect>),
    /// NBT and item component changes.
    Nbt(Box<crate::skbee_nbt::NbtEffect>),
    W2(Box<crate::w2_effects_1::W2Effect>),
    W2b(Box<crate::w2_effects_2::W2Effect>),
    X2(crate::w2_expressions_2::W2Eff, Box<(Expr, Expr)>),
    /// `set exhaustion of player to 2` and friends.
    W2c(Box<crate::w2_expressions_3::StatEffect>),
    W4(Box<crate::w2_expressions_4::W4Effect>),
    W2e3(Box<crate::w2_effects_3::W2Eff>),
    W2Transform(Box<crate::w2_effects_3::Transform>),
    W6(Box<crate::w2_expressions_6::W6Effect>),
    /// `create world`, `unload world` (`w4_skbee_raytrace_world`).
    Rtw(Box<crate::w4_skbee_raytrace_world::RtwEffect>),
    /// Potion effect, particle and weather effects (`w5_chunk5`).
    W5c5(Box<crate::w5_chunk5::W5Effect>),
    /// Barter drop changes (`w5_chunk3`).
    W5c3(Box<crate::w5_chunk3::W5Effect>),
    /// Changes of `w5_chunk6` expressions (scale, target, sea pickles, ...).
    C6(Box<crate::w5_chunk6::C6Effect>),
    /// Effects of `w5_chunk1` (cooldowns, drops, creepers, leashes, ...).
    W5c1(Box<crate::w5_chunk1::Fx>),
    /// Effects of `w6_chunk2` (horns, silence, trees, script switching, ...).
    W6c2(Box<crate::w6_chunk2::Fx>),
    SetHealth(Expr, Expr),
    SetGamemode(Expr, Expr),
    /// Experience change; the two expressions are in pattern order.
    Xp(crate::xp_level::XpOp, Expr, Expr),
    /// Effects of `w2_events_4`.
    Ext4(Box<crate::w2_events_4::ExtEffect>),
    W5(Box<crate::w2_expressions_5::W5Effect>),
    Kill(Expr),
    Teleport(Expr, Expr, Expr, Expr),
    TeleportTo(Expr, Expr),
    /// player, amount (None: the item's own amount), item
    Give(Expr, Option<Expr>, Expr),
    ConsoleCommand(Expr),
    /// who, command (`make player execute command "/x"`)
    ExecAs(Expr, Expr),
    Cancel,
    /// amount times ticks-per-unit
    Wait(Expr, u64),
    Set(VarRef, Expr),
    /// `set {x::*} to split <text> at <sep>`.
    SetSplit(VarRef, Expr, Expr),
    /// `replace all <from> in {var} with <to>`: var, from, to, all.
    Replace(VarRef, Expr, Expr, bool),
    Add(VarRef, Expr),
    Remove(VarRef, Expr),
    Delete(VarRef),
    /// `add x to {list::*}`: append at the next free number index.
    ListAdd(VarRef, Expr),
    /// `remove x from {list::*}`: remove every item equal to x.
    ListRemove(VarRef, Expr),
    PlaySound(Box<PlaySound>),
    /// sound (None: all sounds), player (None: the event player)
    StopSound(Option<Expr>, Option<Expr>),
    /// `set {x::*} to a, b and c`.
    ListSet(VarRef, Vec<Expr>),
    /// `add a, b and c to {x::*}`.
    ListAddAll(VarRef, Vec<Expr>),
    /// `remove a, b from {x::*}`.
    ListRemoveAll(VarRef, Vec<Expr>),
    /// `set {x::*} to sorted {y::*}` and other list expressions.
    SetList(VarRef, ListSrc),
    /// `sort {x::*}` (true: descending).
    SortList(VarRef, bool),
    /// op, kick, ban, names: target and an optional text (reason or name).
    Admin(AdminEff, Expr, Option<Expr>),
    /// Lightning, explosions, particles, potion effects, fire.
    World(Box<crate::world_effects::WorldFx>),
    /// `set block at loc to stone`, `break block at loc`.
    Block(Box<crate::block_ops::BlockFx>),
    /// Chest menus, slots, clear, add and remove items.
    Inv(Box<crate::inventories::InvEff>),
    /// Inventory values, held item edits and the other `gaps` effects.
    Gap(Box<crate::gaps::GapEff>),
    /// Effects of `w5_chunk4` (set block data, apply a potion effect value, ...).
    C4(Box<crate::w5_chunk4::C4Eff>),
    /// Create, delete and change bounds.
    Bound(Box<crate::skbee_bound::BoundEff>),
    /// Time, weather, game rule and difficulty changes.
    Wt(Box<crate::weather_time::WtEffect>),
    /// Scoreboards, boss bars, timed titles, tab list.
    Sb(Box<crate::scoreboards_bossbars::SbFx>),
    /// Scoreboard objectives, scores, teams and boards.
    Sbt(Box<crate::skbee_scoreboard_teams::SbFx>),
    /// Spawn, kill, push and name entities.
    Ent(Box<crate::entities::EntFx>),
    /// Recipes, advancements, statistics.
    Skb(Box<crate::skbee_recipes_advancements::SkbFx>),
    /// Whitelist, world broadcast, world border, teleport to spawn.
    Server(Box<crate::server_misc::ServerFx>),
    /// `set damage to`, `set death message to` (see `combat`).
    Combat(Box<crate::combat::CombatFx>),
}

/// What a `loop` goes over.
/// `play sound`, boxed to keep [`Effect`] small.
#[derive(Debug, Clone, PartialEq)]
pub struct PlaySound {
    pub sound: Expr,
    pub volume: Option<Expr>,
    pub pitch: Option<Expr>,
    pub to: Option<Expr>,
    pub at: Option<Expr>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum LoopSrc {
    Times(Expr),
    /// `loop split <text> at <sep>`.
    Split(Expr, Expr),
    List(VarRef),
    /// `loop sorted {x::*}:` and other list expressions.
    Items(ListSrc),
    Players,
    Values(Vec<Expr>),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Instr {
    Eff(Effect),
    /// Jump to target when the condition is false.
    JumpIfNot(Cond, usize),
    Jump(usize),
    /// Start a loop: take a snapshot of its items. Falls through to `LoopNext`.
    LoopInit(LoopSrc),
    /// Move to the next item, or end the loop and jump to the target.
    LoopNext(usize),
    /// `exit loop`: end the innermost loop and jump to the target.
    LoopBreak(usize),
    /// `stop`: end the trigger, or return from a function.
    Stop,
    /// A function call used as a statement (its result is dropped).
    Call(usize, Vec<Expr>),
    /// `return [value]` from a function.
    Return(Option<Expr>),
    /// A condition on its own line: stop here when it is false (Skript style).
    StopUnless(Cond),
    /// Start of `catch runtime errors:`; an error inside jumps to the target.
    CatchBegin(usize),
    /// End of the section (also emitted before a jump that leaves it).
    CatchEnd,
}

/// A compiled body: a trigger's or a function's instructions.
#[derive(Debug, Clone)]
pub struct Code {
    pub file: Arc<str>,
    pub line: usize,
    pub instrs: Vec<Instr>,
    /// Source line of each instruction.
    pub lines: Vec<usize>,
}

#[derive(Debug, Clone)]
pub struct Trigger {
    pub file: Arc<str>,
    pub line: usize,
    pub kind: TriggerKind,
    pub code: Arc<Code>,
}

#[derive(Debug, Clone)]
pub struct Function {
    pub sig: Sig,
    /// Default value of each parameter, if any.
    pub defaults: Vec<Option<Expr>>,
    pub code: Arc<Code>,
}

/// Everything compiled from a set of files.
#[derive(Debug, Default)]
pub struct Compiled {
    /// Triggers per file: every trigger whose header parsed (broken lines are left out).
    pub files: Vec<(String, Vec<Trigger>)>,
    /// By function id; `None` when the function's header had errors.
    pub functions: Vec<Option<Arc<Function>>>,
    pub errors: Vec<ScriptError>,
    /// `variables:` defaults: (applies per player, code). See `structures`.
    pub var_defaults: Vec<(bool, Trigger)>,
}

// ---- syntax registry ----

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffKind {
    Title,
    ActionBar,
    Send,
    Broadcast,
    Heal,
    Hf(crate::health_food::HfOp),
    W2(crate::w2_effects_1::W2Op),
    W2b(crate::w2_effects_2::W2Op),
    X2(crate::w2_expressions_2::W2Eff),
    W2c(crate::w2_expressions_3::PStat, crate::w2_expressions_3::Mode),
    W4(crate::w2_expressions_4::W4Eff),
    W2e3(crate::w2_effects_3::W2Op),
    W6(crate::w2_expressions_6::W6Op),
    Rtw(crate::w4_skbee_raytrace_world::EOp),
    W5c5(crate::w5_chunk5::EOp),
    W5c3(crate::w5_chunk3::EOp),
    SetHealth,
    SetGamemode,
    Xp(crate::xp_level::XpOp),
    W5(crate::w2_expressions_5::W5Op),
    Kill,
    Teleport,
    TeleportTo,
    Give,
    GiveCount,
    GiveTo,
    ConsoleCommand,
    ExecAs,
    ExecAsBy,
    Cancel,
    WaitTicks,
    WaitSeconds,
    Set,
    ReplaceAllIn,
    ReplaceAllWith,
    ReplaceFirstIn,
    ReplaceFirstWith,
    Add,
    Remove,
    Delete,
    PlaySound,
    StopSound,
    StopAllSounds,
    Admin(AdminEff),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CondKind {
    IsSet,
    NotSet,
    Ne,
    Eq,
    Ge,
    Le,
    Gt,
    Lt,
    HasPerm,
    NotHasPerm,
    InGamemode,
    NotInGamemode,
    Contains,
    NotContains,
    StartsWith,
    NotStartsWith,
    EndsWith,
    NotEndsWith,
    Between,
    NotBetween,
    HasItem,
    NotHasItem,
    /// `item is enchanted [with x]`.
    ItemEnchanted {
        any: bool,
        negate: bool,
    },
    InvContains,
    IsPlayer,
    IsNotPlayer,
    IsConsole,
    IsNotConsole,
    /// `player is sneaking`; the bool is negation.
    Flag(PlayerFlag, bool),
    /// negated
    Admin(AdminCond, bool),
    /// Rare checks from `w2_conditions_2`; the bool is negation.
    W2(crate::w2_conditions_2::W2Kind, bool),
    /// Rare conditions from `w2_conditions_3`.
    W3(crate::w2_conditions_3::W2Kind),
    /// Conditions from `w4_conditions_1`; the bool is negation.
    W4(crate::w4_conditions_1::Kind, bool),
    /// Tag and loaded checks from `w4_skbee_raytrace_world`; the bool is negation.
    Rtw(crate::w4_skbee_raytrace_world::CKind, bool),
    /// Conditions from `w5_chunk3`; the bool is negation.
    W5c3(crate::w5_chunk3::Kind, bool),
}

/// Text effects (`replace ...`). Kept apart from `EFFECT_PATTERNS` so the
/// text slice stays self-contained; tried after the main list.
pub const TEXT_EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    (
        "replace first %text% in %variable% with %text%",
        EffKind::ReplaceFirstIn,
    ),
    (
        "replace first %text% with %text% in %variable%",
        EffKind::ReplaceFirstWith,
    ),
    (
        "replace [all] %text% in %variable% with %text%",
        EffKind::ReplaceAllIn,
    ),
    (
        "replace [all] %text% with %text% in %variable%",
        EffKind::ReplaceAllWith,
    ),
];

/// Effect patterns, tried in order (more specific first).
pub const EFFECT_PATTERNS: &[(&str, EffKind)] = &[
    (
        "send title %text% [with subtitle %text%] to %player%",
        EffKind::Title,
    ),
    ("send action bar %text% to %player%", EffKind::ActionBar),
    ("send %text% [to %player%]", EffKind::Send),
    ("broadcast %text%", EffKind::Broadcast),
    ("heal %player%", EffKind::Heal),
    ("set health of %player% to %number%", EffKind::SetHealth),
    ("kill %player%", EffKind::Kill),
    (
        "teleport %player% to %number%, %number%, %number%",
        EffKind::Teleport,
    ),
    ("teleport %player% to %location%", EffKind::TeleportTo),
    ("give %number% [of] %item% to %player%", EffKind::GiveCount),
    ("give %item% to %player%", EffKind::GiveTo),
    ("give %player% %item%", EffKind::Give),
    ("execute console command %text%", EffKind::ConsoleCommand),
    ("cancel [the] event", EffKind::Cancel),
    ("wait %number% (tick|ticks)", EffKind::WaitTicks),
    ("wait %number% (second|seconds)", EffKind::WaitSeconds),
    ("set %variable% to %object%", EffKind::Set),
    ("add %object% to %variable%", EffKind::Add),
    ("remove %object% from %variable%", EffKind::Remove),
    ("delete %variable%", EffKind::Delete),
];

pub const CONDITION_PATTERNS: &[(&str, CondKind)] = &[
    ("%object% is set", CondKind::IsSet),
    ("%object% is not set", CondKind::NotSet),
    ("%object% (is not|!=) %object%", CondKind::Ne),
    ("%object% (is|=) %object%", CondKind::Eq),
    ("%number% >= %number%", CondKind::Ge),
    ("%number% <= %number%", CondKind::Le),
    ("%number% > %number%", CondKind::Gt),
    ("%number% < %number%", CondKind::Lt),
    ("%player% has permission %text%", CondKind::HasPerm),
    ("%player% does not have permission %text%", CondKind::NotHasPerm),
    ("%player%'s inventory contains %item%", CondKind::InvContains),
    ("inventory of %player% contains %item%", CondKind::InvContains),
    ("%player% has %item%", CondKind::HasItem),
    ("%player% (does not|doesn't) have %item%", CondKind::NotHasItem),
    ("%number% is between %number% and %number%", CondKind::Between),
    (
        "%number% is not between %number% and %number%",
        CondKind::NotBetween,
    ),
    ("%text% contains %text%", CondKind::Contains),
    ("%text% (does not|doesn't) contain %text%", CondKind::NotContains),
    ("%text% starts with %text%", CondKind::StartsWith),
    (
        "%text% (does not|doesn't) start with %text%",
        CondKind::NotStartsWith,
    ),
    ("%text% ends with %text%", CondKind::EndsWith),
    ("%text% (does not|doesn't) end with %text%", CondKind::NotEndsWith),
];

struct Registry {
    effects: Vec<Pattern<EffKind>>,
    conditions: Vec<Pattern<CondKind>>,
}

fn registry() -> &'static Registry {
    static R: OnceLock<Registry> = OnceLock::new();
    R.get_or_init(|| Registry {
        // The pattern strings are constants covered by tests; a bad one is skipped.
        effects: crate::skbee_text::EFFECT_PATTERNS
            .iter()
            .chain(crate::op_kick_ban::EFFECT_PATTERNS)
            .filter_map(|(s, k)| Pattern::new(s, *k).ok())
            .chain(
                crate::xp_level::XP_EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, o)| Pattern::new(s, EffKind::Xp(*o)).ok()),
            )
            .chain(
                crate::senders::SENDER_EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::gamemode::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::sounds::SOUND_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                TEXT_EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::health_food::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::w2_effects_1::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::w2_effects_2::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::w2_expressions_2::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::w2_expressions_3::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::w2_expressions_4::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::w2_effects_3::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::w2_expressions_5::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::w2_expressions_6::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::w4_skbee_raytrace_world::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::w5_chunk5::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .chain(
                crate::w5_chunk3::EFFECT_PATTERNS
                    .iter()
                    .filter_map(|(s, k)| Pattern::new(s, *k).ok()),
            )
            .collect(),
        conditions: crate::op_kick_ban::CONDITION_PATTERNS
            .iter()
            .chain(crate::w2_conditions_2::CONDITION_PATTERNS)
            .chain(crate::senders::SENDER_CONDITION_PATTERNS)
            .chain(crate::gamemode::CONDITION_PATTERNS)
            .chain(crate::item_meta::CONDITION_PATTERNS)
            .chain(PLAYER_FLAG_PATTERNS)
            .chain(crate::w2_conditions_2::FLAG_PATTERNS)
            .chain(crate::w2_conditions_3::W2_CONDITION_PATTERNS)
            .chain(crate::w4_conditions_1::CONDITION_PATTERNS)
            .chain(crate::w4_skbee_raytrace_world::CONDITION_PATTERNS)
            .chain(crate::w5_chunk3::CONDITION_PATTERNS)
            .chain(CONDITION_PATTERNS)
            .filter_map(|(s, k)| Pattern::new(s, *k).ok())
            .collect(),
    })
}

/// Checks every registered pattern compiles. Used by tests.
pub fn check_registry() -> Result<(), String> {
    for (s, k) in crate::skbee_text::EFFECT_PATTERNS
        .iter()
        .chain(crate::op_kick_ban::EFFECT_PATTERNS)
        .chain(crate::senders::SENDER_EFFECT_PATTERNS)
        .chain(crate::gamemode::EFFECT_PATTERNS)
        .chain(EFFECT_PATTERNS)
        .chain(crate::sounds::SOUND_PATTERNS)
        .chain(TEXT_EFFECT_PATTERNS)
        .chain(crate::health_food::EFFECT_PATTERNS)
        .chain(crate::w2_effects_1::EFFECT_PATTERNS)
        .chain(crate::w2_effects_2::EFFECT_PATTERNS)
        .chain(crate::w2_expressions_2::EFFECT_PATTERNS)
        .chain(crate::w2_expressions_3::EFFECT_PATTERNS)
        .chain(crate::w2_expressions_4::EFFECT_PATTERNS)
        .chain(crate::w2_effects_3::EFFECT_PATTERNS)
        .chain(crate::w2_expressions_5::EFFECT_PATTERNS)
        .chain(crate::w2_expressions_6::EFFECT_PATTERNS)
        .chain(crate::w4_skbee_raytrace_world::EFFECT_PATTERNS)
        .chain(crate::w5_chunk5::EFFECT_PATTERNS)
        .chain(crate::w5_chunk3::EFFECT_PATTERNS)
    {
        Pattern::new(s, *k)?;
    }
    for (s, o) in crate::xp_level::XP_EFFECT_PATTERNS {
        Pattern::new(s, EffKind::Xp(*o))?;
    }
    for (s, k) in crate::op_kick_ban::CONDITION_PATTERNS
        .iter()
        .chain(crate::senders::SENDER_CONDITION_PATTERNS)
        .chain(crate::gamemode::CONDITION_PATTERNS)
        .chain(crate::item_meta::CONDITION_PATTERNS)
        .chain(PLAYER_FLAG_PATTERNS)
        .chain(crate::w2_conditions_2::CONDITION_PATTERNS)
        .chain(crate::w2_conditions_2::FLAG_PATTERNS)
        .chain(crate::w2_conditions_3::W2_CONDITION_PATTERNS)
        .chain(crate::w4_conditions_1::CONDITION_PATTERNS)
        .chain(crate::w4_skbee_raytrace_world::CONDITION_PATTERNS)
        .chain(crate::w5_chunk3::CONDITION_PATTERNS)
        .chain(CONDITION_PATTERNS)
    {
        Pattern::new(s, *k)?;
    }
    Ok(())
}

/// Normalized text for "did you mean" distance: values become `x`.
fn norm_line(toks: &[Tok]) -> String {
    toks.iter()
        .map(|t| match t {
            Tok::Word(w) => w.to_ascii_lowercase(),
            Tok::Sym(s) => s.to_string(),
            _ => "x".into(),
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn norm_pattern(src: &str) -> String {
    let mut out = String::new();
    let mut in_slot = false;
    for c in src.chars() {
        match c {
            '%' => {
                if !in_slot {
                    out.push('x');
                }
                in_slot = !in_slot;
            }
            _ if in_slot => {}
            '[' | ']' | '(' | ')' => {}
            '|' => out.push(' '),
            _ => out.push(c),
        }
    }
    out.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn suggest(toks: &[Tok], srcs: &[&'static str]) -> Vec<String> {
    let line = norm_line(toks);
    let normed: Vec<(String, &str)> = srcs.iter().map(|s| (norm_pattern(s), *s)).collect();
    let best = closest(&line, normed.iter().map(|(n, _)| n.as_str()), 3);
    best.iter()
        .filter_map(|b| normed.iter().find(|(n, _)| n == b).map(|(_, s)| s.to_string()))
        .collect()
}

// ---- compiler ----

/// A loop being compiled: where `continue` jumps and which jumps `exit loop` patches.
struct LoopCx {
    /// Has a runtime frame (every loop except `while`).
    frame: bool,
    next: usize,
    breaks: Vec<usize>,
    /// `catch runtime errors` sections open around the loop itself.
    catch: usize,
}

pub(crate) struct Cx<'a> {
    pub(crate) file: &'a str,
    pub(crate) env: Env,
    pub(crate) is_command: bool,
    /// Return type of the function being compiled (`Some(None)`: returns nothing).
    pub(crate) func_ret: Option<Option<Ty>>,
    /// Why `cancel event` is not allowed here, if it isn't.
    pub(crate) no_cancel: Option<String>,
    pub(crate) code: Vec<Instr>,
    pub(crate) lines: Vec<usize>,
    pub(crate) errors: Vec<ScriptError>,
    loops: Vec<LoopCx>,
    /// `catch runtime errors` sections open around the current line.
    pub(crate) catch_depth: usize,
}

impl<'a> Cx<'a> {
    /// An empty compile state for a body that is not a trigger or function.
    pub(crate) fn new(file: &'a str, env: Env) -> Self {
        Cx {
            file,
            env,
            is_command: false,
            func_ret: None,
            no_cancel: None,
            code: Vec::new(),
            lines: Vec::new(),
            errors: Vec::new(),
            loops: Vec::new(),
            catch_depth: 0,
        }
    }
}

impl Cx<'_> {
    pub(crate) fn err(&mut self, line: usize, msg: impl Into<String>) {
        self.errors.push(ScriptError::new(self.file, line, msg));
    }

    pub(crate) fn emit(&mut self, i: Instr, line: usize) -> usize {
        self.code.push(i);
        self.lines.push(line);
        self.code.len() - 1
    }

    pub(crate) fn block(&mut self, lines: &[Line]) {
        let mut i = 0;
        while i < lines.len() {
            let line = &lines[i];
            let lower = line.text.to_ascii_lowercase();
            if lower.starts_with("if ") && line.opens_block() {
                let mut ends = Vec::new();
                let mut cur = line;
                let mut cond_text = line.head().get(3..).unwrap_or("");
                // Like Skript: a section whose condition fails to parse is
                // reported and its whole block skipped. The chain ends there,
                // so a following `else` is reported instead of attaching.
                while let Some(c) = self.cond(cur.num, cond_text) {
                    let jif = self.emit(Instr::JumpIfNot(c, usize::MAX), cur.num);
                    self.block(&cur.children);
                    let next = lines.get(i + 1);
                    let next_lower = next.map(|l| l.text.to_ascii_lowercase()).unwrap_or_default();
                    let has_more = next.is_some_and(|l| l.opens_block())
                        && (next_lower.starts_with("else if ") || next_lower == "else:");
                    if has_more {
                        ends.push(self.emit(Instr::Jump(usize::MAX), cur.num));
                    }
                    let here = self.code.len();
                    if let Some(Instr::JumpIfNot(_, t)) = self.code.get_mut(jif) {
                        *t = here;
                    }
                    if !has_more {
                        break;
                    }
                    i += 1;
                    let Some(n) = lines.get(i) else { break };
                    if next_lower == "else:" {
                        self.block(&n.children);
                        break;
                    }
                    cur = n;
                    cond_text = n.head().get(8..).unwrap_or("");
                }
                let end = self.code.len();
                for e in ends {
                    if let Some(Instr::Jump(t)) = self.code.get_mut(e) {
                        *t = end;
                    }
                }
                i += 1;
                continue;
            }
            if (lower.starts_with("loop ") || lower.starts_with("while ")) && line.opens_block() {
                self.loop_block(line);
                i += 1;
                continue;
            }
            if line.opens_block() && crate::structures::section(self, line, &lower) {
                i += 1;
                continue;
            }
            if lower.starts_with("else") {
                self.err(
                    line.num,
                    "'else' has to be placed just after another 'if' or 'else if' section",
                );
                i += 1;
                continue;
            }
            if line.opens_block() {
                self.err(
                    line.num,
                    format!(
                        "unknown section '{}' (only 'if', 'else if' and 'else' open blocks here)",
                        line.head()
                    ),
                );
                i += 1;
                continue;
            }
            if crate::structures::inline_if(self, line) {
                i += 1;
                continue;
            }
            if self.control(line.num, &line.text, &lower) {
                i += 1;
                continue;
            }
            self.effect(line.num, &line.text);
            i += 1;
        }
    }

    /// `stop`, `stop loop` / `exit loop`, `continue`. Returns true if handled.
    pub(crate) fn control(&mut self, num: usize, text: &str, lower: &str) -> bool {
        // Most lines are effects: check the first word before collecting all.
        let first = lower.split_whitespace().next().unwrap_or("");
        if !matches!(first, "stop" | "exit" | "break" | "continue" | "return") {
            return false;
        }
        let words: Vec<&str> = lower.split_whitespace().collect();
        match words.as_slice() {
            ["stop"] | ["stop", "trigger"] | ["exit"] => {
                self.emit(Instr::Stop, num);
            }
            ["stop", "loop"] | ["exit", "loop"] | ["exit", "this", "loop"] | ["break"] => {
                let Some(lp) = self.loops.last() else {
                    self.err(num, "'exit loop' only works inside a loop");
                    return true;
                };
                let (frame, open) = (lp.frame, self.catch_depth - lp.catch);
                for _ in 0..open {
                    self.emit(Instr::CatchEnd, num);
                }
                let instr = if frame {
                    Instr::LoopBreak(usize::MAX)
                } else {
                    Instr::Jump(usize::MAX)
                };
                let at = self.emit(instr, num);
                if let Some(lp) = self.loops.last_mut() {
                    lp.breaks.push(at);
                }
            }
            ["return", ..] => {
                self.return_stmt(num, text);
            }
            ["continue"] | ["continue", "loop"] => {
                let Some((next, open)) = self.loops.last().map(|l| (l.next, self.catch_depth - l.catch))
                else {
                    self.err(num, "'continue' only works inside a loop");
                    return true;
                };
                for _ in 0..open {
                    self.emit(Instr::CatchEnd, num);
                }
                self.emit(Instr::Jump(next), num);
            }
            _ => return false,
        }
        true
    }

    fn return_stmt(&mut self, num: usize, text: &str) {
        let Some(ret) = self.func_ret else {
            self.err(
                num,
                "'return' only works inside a function (use 'stop' to end a trigger)",
            );
            return;
        };
        let rest = text.get(6..).unwrap_or("").trim();
        if rest.is_empty() {
            if ret.is_some() {
                self.err(num, "this function must return a value");
            }
            self.emit(Instr::Return(None), num);
            return;
        }
        let Some(want) = ret else {
            self.err(
                num,
                "this function has no return type; add ':: <type>' to its first line",
            );
            return;
        };
        let toks = match lex(rest) {
            Ok(t) => t,
            Err(e) => {
                self.err(num, expr_error(rest, e, "the returned value"));
                return;
            }
        };
        if want == Ty::List {
            match crate::list_ops::parse_src_all(&toks, &self.env, true) {
                Ok(s) => {
                    self.emit(Instr::Return(Some(Expr::Items(Box::new(s)))), num);
                }
                Err(e) => self.err(num, format!("can't understand the returned list: {e}")),
            }
            return;
        }
        match parse_expr(&toks, &self.env) {
            Ok(e) if want.accepts(e.ty(&self.env)) => {
                self.emit(Instr::Return(Some(e)), num);
            }
            Ok(e) => self.err(
                num,
                format!(
                    "this function returns a {}, not a {}",
                    want.name(),
                    e.ty(&self.env).name()
                ),
            ),
            Err(e) => self.err(num, expr_error(rest, e, "the returned value")),
        }
    }

    /// A line that is just `name(args)`.
    fn call_stmt(&mut self, num: usize, toks: &[Tok]) -> bool {
        let is_call = match toks {
            [Tok::Word(w), Tok::Sym("("), .., Tok::Sym(")")] => {
                self.env.functions.iter().any(|f| f.name.eq_ignore_ascii_case(w))
            }
            _ => false,
        };
        if !is_call {
            return false;
        }
        match parse_expr(toks, &self.env) {
            Ok(Expr::Call(id, args)) => {
                self.emit(Instr::Call(id, args), num);
            }
            Ok(_) => return false,
            Err(e) => self.err(num, e),
        }
        true
    }

    fn loop_block(&mut self, line: &Line) {
        let head = line.head();
        let lower = head.to_ascii_lowercase();
        if let Some(cond_text) = head.get(6..).filter(|_| lower.starts_with("while ")) {
            let start = self.code.len();
            // A `while` whose condition fails to parse skips its block.
            let Some(c) = self.cond(line.num, cond_text) else {
                return;
            };
            let jif = self.emit(Instr::JumpIfNot(c, usize::MAX), line.num);
            self.loops.push(LoopCx {
                frame: false,
                next: start,
                breaks: Vec::new(),
                catch: self.catch_depth,
            });
            self.block(&line.children);
            self.emit(Instr::Jump(start), line.num);
            let end = self.code.len();
            if let Some(Instr::JumpIfNot(_, t)) = self.code.get_mut(jif) {
                *t = end;
            }
            self.finish_loop(end);
            return;
        }
        let rest = head.get(5..).unwrap_or("").trim();
        let Some((src, kind)) = self.loop_source(line.num, rest) else {
            return;
        };
        self.emit(Instr::LoopInit(src), line.num);
        let next = self.emit(Instr::LoopNext(usize::MAX), line.num);
        self.loops.push(LoopCx {
            frame: true,
            next,
            breaks: Vec::new(),
            catch: self.catch_depth,
        });
        self.env.loops.push(kind);
        self.block(&line.children);
        self.env.loops.pop();
        self.emit(Instr::Jump(next), line.num);
        let end = self.code.len();
        if let Some(Instr::LoopNext(t)) = self.code.get_mut(next) {
            *t = end;
        }
        self.finish_loop(end);
    }

    fn finish_loop(&mut self, end: usize) {
        if let Some(lp) = self.loops.pop() {
            for b in lp.breaks {
                if let Some(Instr::LoopBreak(t) | Instr::Jump(t)) = self.code.get_mut(b) {
                    *t = end;
                }
            }
        }
    }

    fn loop_source(&mut self, num: usize, rest: &str) -> Option<(LoopSrc, LoopKind)> {
        let lower = rest.to_ascii_lowercase();
        if matches!(
            lower.as_str(),
            "all players" | "all online players" | "players" | "online players"
        ) {
            return Some((LoopSrc::Players, LoopKind::Players));
        }
        let toks = match lex(rest) {
            Ok(t) => t,
            Err(e) => {
                self.err(num, e);
                return None;
            }
        };
        if let Some((last, body)) = toks.split_last()
            && last.is_lit("times")
        {
            return match parse_expr(body, &self.env) {
                Ok(e) if Ty::Number.accepts(e.ty(&self.env)) => Some((LoopSrc::Times(e), LoopKind::Times)),
                Ok(e) => {
                    self.err(
                        num,
                        format!("'loop ... times' needs a number, got {}", e.ty(&self.env).name()),
                    );
                    None
                }
                Err(e) => {
                    let body_text = rest.rsplit_once(' ').map_or(rest, |(b, _)| b).trim();
                    self.err(num, expr_error(body_text, e, "the loop count"));
                    None
                }
            };
        }
        if toks.first().is_some_and(|t| t.is_lit("split")) {
            return match parse_expr(&toks, &self.env) {
                Ok(Expr::Split(a, b)) => Some((LoopSrc::Split(*a, *b), LoopKind::Values)),
                Ok(_) => {
                    self.err(num, "can't understand 'loop split ...'".to_string());
                    None
                }
                Err(e) => {
                    self.err(num, e);
                    None
                }
            };
        }
        if let Some(e) = crate::entities::parse_list(&toks, &self.env) {
            return Some((LoopSrc::Items(ListSrc::One(e)), LoopKind::List));
        }
        // `loop items in {_gui}:`
        if toks.first().is_some_and(|t| t.is_lit("items") || t.is_lit("all"))
            && let Ok(e @ Expr::Gap(_)) = parse_expr(&toks, &self.env)
        {
            return Some((LoopSrc::Items(ListSrc::One(e)), LoopKind::List));
        }
        // `loop blocks in radius 3 of player:`, `loop ender chest of player:`
        if let Ok(e @ Expr::C4(_)) = parse_expr(&toks, &self.env)
            && e.ty(&self.env) == Ty::List
        {
            return Some((LoopSrc::Items(ListSrc::One(e)), LoopKind::List));
        }
        if crate::list_ops::looks_like_list_op(&toks, &self.env) {
            return match crate::list_ops::parse_src_all(&toks, &self.env, true) {
                Ok(s) => Some((LoopSrc::Items(s), LoopKind::List)),
                Err(e) => {
                    self.err(num, format!("can't understand 'loop {rest}': {e}"));
                    None
                }
            };
        }
        if let [Tok::Var(raw)] = toks.as_slice() {
            return match parse_varref(raw, &self.env) {
                Ok(v) if v.is_list() => Some((LoopSrc::List(v), LoopKind::List)),
                Ok(_) => {
                    self.err(num, format!("'loop {rest}' needs a list variable like {{x::*}}"));
                    None
                }
                Err(e) => {
                    self.err(num, e);
                    None
                }
            };
        }
        // `loop 1, 2 and 3:` / `loop "a" and "b":` / `loop indices of {x::*}:`
        let Some(items) = crate::list_literals::split_items(&toks, &self.env) else {
            self.err(
                num,
                format!("can't understand 'loop {rest}' (use 'loop N times', 'loop {{x::*}}', 'loop all players' or 'loop a, b and c')"),
            );
            return None;
        };
        Some((LoopSrc::Values(items), LoopKind::Values))
    }

    pub(crate) fn cond(&mut self, num: usize, text: &str) -> Option<Cond> {
        let toks = match lex(text) {
            Ok(t) => t,
            Err(e) => {
                self.err(num, e);
                return None;
            }
        };
        if let Some(c) = self.parse_cond(&toks) {
            return Some(c);
        }
        let mut type_err = None;
        let mut memo = Memo::default();
        let rtw = crate::w4_skbee_raytrace_world::line_may_match(&toks);
        let w5c3 = crate::w5_chunk3::line_may_match(&toks);
        for p in &registry().conditions {
            if !rtw && matches!(p.kind, CondKind::Rtw(..)) || !w5c3 && matches!(p.kind, CondKind::W5c3(..)) {
                continue;
            }
            match p.matches_memo(&toks, &self.env, &mut memo) {
                Ok(caps) => return Some(build_cond(p.kind, caps)),
                Err(Some(e)) if type_err.is_none() => type_err = Some(e),
                Err(_) => {}
            }
        }
        // A broken template or variable explains the failure best.
        let type_err = inner_error(&toks, &self.env).or(type_err);
        let srcs: Vec<&'static str> = crate::op_kick_ban::CONDITION_PATTERNS
            .iter()
            .chain(PLAYER_FLAG_PATTERNS)
            .chain(crate::w2_conditions_2::CONDITION_PATTERNS)
            .chain(crate::w2_conditions_2::FLAG_PATTERNS)
            .chain(crate::w2_conditions_3::W2_CONDITION_PATTERNS)
            .chain(crate::w4_conditions_1::CONDITION_PATTERNS)
            .chain(crate::w4_skbee_raytrace_world::CONDITION_PATTERNS)
            .chain(crate::w5_chunk3::CONDITION_PATTERNS)
            .chain(CONDITION_PATTERNS)
            .map(|(s, _)| *s)
            .collect();
        let mut e = ScriptError::new(
            self.file,
            num,
            match type_err {
                Some(t) => format!("can't understand this condition: '{text}' ({t})"),
                None => format!("can't understand this condition: '{text}'"),
            },
        );
        e.suggestions = suggest(&toks, &srcs);
        self.errors.push(e);
        None
    }

    /// `a or b`, `a and b`, `a, b and c`, `(a or b) and c`, `x is a or b`, or one condition.
    fn parse_cond(&self, toks: &[Tok]) -> Option<Cond> {
        let toks = strip_parens(toks);
        // `chance of 25%`
        if let [first, second, mid @ .., last] = toks
            && first.is_lit("chance")
            && second.is_lit("of")
            && last.is_lit("%")
        {
            return parse_expr(mid, &self.env)
                .ok()
                .filter(|e| Ty::Number.accepts(e.ty(&self.env)))
                .map(Cond::Chance);
        }
        // Most conditions are a single check: try that first when there is
        // nothing to split on.
        let joins = toks
            .iter()
            .any(|t| t.is_lit("or") || t.is_lit("and") || t.is_lit(","));
        if !joins {
            return self.single_cond(toks);
        }
        if let Some(parts) = self.split_conds(toks, &["or"]) {
            return Some(Cond::Any(parts));
        }
        if let Some(parts) = self.split_conds(toks, &["and", ","]) {
            return Some(Cond::All(parts));
        }
        if let Some(c) = self.single_cond(toks) {
            return Some(c);
        }
        self.alternatives(toks)
    }

    fn single_cond(&self, toks: &[Tok]) -> Option<Cond> {
        if let Some(c) = crate::gaps::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if let Some(c) = crate::inventories::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if self.env.event_vals != 0
            && let Some(c) = crate::events_items::parse_entity_cond(toks, &self.env)
        {
            return Some(c);
        }
        if let Some(c) = crate::world_effects::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if let Some(c) = crate::server_misc::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if let Some(c) = crate::block_ops::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if let Some(c) = crate::skbee_bound::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if let Some(c) = crate::weather_time::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if let Some(c) = crate::skbee_scoreboard_teams::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if let Some(c) = crate::skbee_nbt::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if let Some(c) = crate::entities::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if let Some(c) = crate::skbee_recipes_advancements::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if let Some(c) = crate::w5_chunk2::parse_cond(toks, &self.env) {
            return Some(c);
        }
        if let Some(c) = crate::w5_chunk1::parse_cond(toks, &self.env) {
            return Some(c);
        }
        let mut memo = Memo::default();
        let w5c3 = crate::w5_chunk3::line_may_match(toks);
        registry()
            .conditions
            .iter()
            .filter(|p| w5c3 || !matches!(p.kind, CondKind::W5c3(..)))
            .find_map(|p| {
                p.matches_memo(toks, &self.env, &mut memo)
                    .ok()
                    .map(|caps| build_cond(p.kind, caps))
            })
            .or_else(|| self.span_cmp(toks))
    }

    /// `time since {d} > 5 minutes`: ordering for spans and dates.
    fn span_cmp(&self, toks: &[Tok]) -> Option<Cond> {
        let (at, op) = toks.iter().enumerate().find_map(|(i, t)| match t {
            Tok::Sym(">=") => Some((i, CmpOp::Ge)),
            Tok::Sym("<=") => Some((i, CmpOp::Le)),
            Tok::Sym(">") => Some((i, CmpOp::Gt)),
            Tok::Sym("<") => Some((i, CmpOp::Lt)),
            _ => None,
        })?;
        let a = parse_expr(toks.get(..at)?, &self.env).ok()?;
        let b = parse_expr(toks.get(at + 1..)?, &self.env).ok()?;
        let timey = |t: Ty| matches!(t, Ty::Timespan | Ty::Date);
        let (ta, tb) = (a.ty(&self.env), b.ty(&self.env));
        let ok = (timey(ta) || timey(tb)) && (ta == tb || ta == Ty::Object || tb == Ty::Object);
        ok.then_some(Cond::Cmp(a, op, b))
    }

    /// `wait 5 minutes`, `wait {span}`: the pattern table only has number forms.
    fn wait_span(&self, toks: &[Tok]) -> Option<Effect> {
        if !toks.first()?.is_lit("wait") {
            return None;
        }
        let e = parse_expr(toks.get(1..)?, &self.env).ok()?;
        matches!(e.ty(&self.env), Ty::Timespan | Ty::Object).then_some(Effect::Wait(e, 0))
    }

    /// Split at top-level separators so that every part is a condition.
    /// Tries split points in order, so `x is between 1 and 5 and y is 2` works.
    fn split_conds(&self, toks: &[Tok], seps: &[&str]) -> Option<Vec<Cond>> {
        let cuts = top_level(toks, seps);
        if cuts.is_empty() {
            return None;
        }
        self.split_from(toks, &cuts, 0, seps)
    }

    fn split_from(&self, toks: &[Tok], cuts: &[usize], start: usize, seps: &[&str]) -> Option<Vec<Cond>> {
        for (i, &cut) in cuts.iter().enumerate() {
            if cut <= start {
                continue;
            }
            let Some(first) = toks.get(start..cut).and_then(|t| self.parse_part(t, seps)) else {
                continue;
            };
            let rest_cuts = cuts.get(i + 1..).unwrap_or_default();
            let tail_start = cut + 1;
            // The rest as one condition, or split further.
            if let Some(last) = toks.get(tail_start..).and_then(|t| self.parse_part(t, seps)) {
                return Some(vec![first, last]);
            }
            if let Some(mut more) = self.split_from(toks, rest_cuts, tail_start, seps) {
                more.insert(0, first);
                return Some(more);
            }
        }
        None
    }

    /// A part of an `or` may itself hold `and`; a part of an `and` is one condition.
    fn parse_part(&self, toks: &[Tok], seps: &[&str]) -> Option<Cond> {
        if toks.is_empty() {
            return None;
        }
        if seps.contains(&"or")
            && let Some(parts) = self.split_conds(strip_parens(toks), &["and", ","])
        {
            return Some(Cond::All(parts));
        }
        let inner = strip_parens(toks);
        if inner.len() != toks.len() {
            return self.parse_cond(inner);
        }
        self.single_cond(toks).or_else(|| self.alternatives(toks))
    }

    /// `x is "a" or "b"`, `x is not 1, 2 or 3`.
    fn alternatives(&self, toks: &[Tok]) -> Option<Cond> {
        let pos = toks
            .iter()
            .position(|t| t.is_lit("is") || t.is_lit("=") || t.is_lit("!="))?;
        let (negate, rest_at) = match toks.get(pos) {
            Some(t) if t.is_lit("!=") => (true, pos + 1),
            Some(t) if t.is_lit("=") => (false, pos + 1),
            _ if toks.get(pos + 1).is_some_and(|t| t.is_lit("not")) => (true, pos + 2),
            _ => (false, pos + 1),
        };
        let value = parse_expr(toks.get(..pos)?, &self.env).ok()?;
        let rest = toks.get(rest_at..)?;
        let mut options = Vec::new();
        for part in rest.split(|t| t.is_lit("or") || t.is_lit(",") || t.is_lit("and")) {
            options.push(parse_expr(part, &self.env).ok()?);
        }
        (options.len() >= 2).then_some(Cond::AnyOf {
            value,
            options,
            negate,
        })
    }

    pub(crate) fn effect(&mut self, num: usize, text: &str) {
        let toks = match lex(text) {
            Ok(t) => t,
            Err(e) => {
                self.err(num, e);
                return;
            }
        };
        if self.call_stmt(num, &toks) {
            return;
        }
        match crate::w6_chunk2::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::w5_chunk4::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::w5_chunk1::parse_effect(&toks, &self.env, self.is_command) {
            Ok(Some((eff, stop))) => {
                self.emit(Instr::Eff(eff), num);
                if stop {
                    self.emit(Instr::Stop, num);
                }
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::gaps::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::w5_chunk6::parse_effect(&toks, &self.env) {
            Some(Ok(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Some(Err(msg)) => {
                self.err(num, msg);
                return;
            }
            None => {}
        }
        if self.env.has_attacker {
            match crate::combat::parse_effect(&toks, &self.env) {
                Some(Ok(eff)) => {
                    self.emit(Instr::Eff(eff), num);
                    return;
                }
                Some(Err(e)) => {
                    self.err(num, e);
                    return;
                }
                None => {}
            }
        }
        if self.env.ext4.is_some() {
            match crate::w2_events_4::parse_effect(&toks, &self.env) {
                Some(Ok(eff)) => {
                    self.emit(Instr::Eff(eff), num);
                    return;
                }
                Some(Err(e)) => {
                    self.err(num, e);
                    return;
                }
                None => {}
            }
        }
        let ops_first = toks.first().is_some_and(|t| t.is_lit("sort"))
            || (toks.first().is_some_and(|t| t.is_lit("set"))
                && toks
                    .get(3..)
                    .is_some_and(|r| crate::list_ops::looks_like_list_op(r, &self.env)));
        let parsed = if let Some(t) = crate::w2_effects_3::parse_transform(&toks, &self.env) {
            Some(t)
        } else if ops_first {
            crate::list_ops::parse_effect(&toks, &self.env)
                .or_else(|| crate::list_literals::parse_list_effect(&toks, &self.env))
        } else {
            crate::list_literals::parse_list_effect(&toks, &self.env)
                .or_else(|| crate::list_ops::parse_effect(&toks, &self.env))
        };
        match parsed {
            Some(Ok(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Some(Err(e)) => {
                self.err(num, e);
                return;
            }
            None => {}
        }
        match crate::inventories::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::block_ops::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::weather_time::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::skbee_scoreboard_teams::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::skbee_nbt::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::world_effects::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::server_misc::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::skbee_bound::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::skbee_text::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::scoreboards_bossbars::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::entities::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        match crate::skbee_recipes_advancements::parse_effect(&toks, &self.env) {
            Ok(Some(eff)) => {
                self.emit(Instr::Eff(eff), num);
                return;
            }
            Ok(None) => {}
            Err(msg) => {
                self.err(num, msg);
                return;
            }
        }
        let mut type_err = None;
        let mut memo = Memo::default();
        for p in &registry().effects {
            match p.matches_memo(&toks, &self.env, &mut memo) {
                Ok(caps) => {
                    if p.kind == EffKind::Cancel && self.is_command {
                        self.err(num, "commands have no event to cancel");
                        return;
                    }
                    if p.kind == EffKind::Cancel && self.func_ret.is_some() {
                        self.err(num, "functions have no event to cancel");
                        return;
                    }
                    if p.kind == EffKind::Cancel
                        && let Some(why) = self.no_cancel.clone()
                    {
                        self.err(num, why);
                        return;
                    }
                    if matches!(p.kind, EffKind::Add | EffKind::Remove) {
                        let not_list = matches!(caps.get(1), Some(Some(Capture::Var(v))) if !v.is_list());
                        let ty = match caps.first() {
                            Some(Some(Capture::Expr(e))) => e.ty(&self.env),
                            _ => Ty::Object,
                        };
                        if not_list && !Ty::Number.accepts(ty) {
                            self.err(num, format!("can't add a {} to a single variable (only numbers; use a list like {{x::*}} for other values)", ty.name()));
                            return;
                        }
                    }
                    if let Some(eff) = build_effect(p.kind, caps) {
                        self.emit(Instr::Eff(eff), num);
                    }
                    return;
                }
                Err(Some(e)) if type_err.is_none() => type_err = Some(e),
                Err(_) => {}
            }
        }
        if let Some(eff) = self.wait_span(&toks) {
            self.emit(Instr::Eff(eff), num);
            return;
        }
        // A condition on its own line stops the trigger when false.
        if let Some(c) = self.parse_cond(&toks) {
            self.emit(Instr::StopUnless(c), num);
            return;
        }
        let type_err = inner_error(&toks, &self.env).or(type_err);
        let srcs: Vec<&'static str> = EFFECT_PATTERNS
            .iter()
            .chain(TEXT_EFFECT_PATTERNS)
            .chain(crate::health_food::EFFECT_PATTERNS)
            .chain(crate::w2_effects_1::EFFECT_PATTERNS)
            .chain(crate::w2_effects_2::EFFECT_PATTERNS)
            .chain(crate::w2_expressions_2::EFFECT_PATTERNS)
            .chain(crate::w2_expressions_3::EFFECT_PATTERNS)
            .chain(crate::w2_expressions_4::EFFECT_PATTERNS)
            .chain(crate::w2_effects_3::EFFECT_PATTERNS)
            .chain(crate::w2_expressions_6::EFFECT_PATTERNS)
            .chain(crate::w4_skbee_raytrace_world::EFFECT_PATTERNS)
            .chain(crate::w5_chunk5::EFFECT_PATTERNS)
            .chain(crate::w5_chunk3::EFFECT_PATTERNS)
            .map(|(s, _)| *s)
            .collect();
        let mut e = ScriptError::new(
            self.file,
            num,
            match type_err {
                Some(t) => format!("can't understand this effect: '{text}' ({t})"),
                None => format!("can't understand this effect: '{text}'"),
            },
        );
        e.suggestions = suggest(&toks, &srcs);
        self.errors.push(e);
    }
}

/// Skript's wording for an expression that doesn't parse at all; specific
/// errors (unknown function, wrong type, ...) keep their own text.
fn expr_error(text: &str, e: String, what: &str) -> String {
    if e.starts_with("unexpected") {
        format!("can't understand this expression: '{text}'")
    } else {
        format!("can't understand {what}: {e}")
    }
}

/// `( ... )` around the whole token list, removed.
fn strip_parens(toks: &[Tok]) -> &[Tok] {
    let mut t = toks;
    while let (Some(first), Some(last)) = (t.first(), t.last()) {
        if !(first.is_lit("(") && last.is_lit(")")) {
            break;
        }
        // The first '(' must close at the very end.
        let mut depth = 0i32;
        let mut closes_at_end = true;
        for (i, tok) in t.iter().enumerate() {
            if tok.is_lit("(") {
                depth += 1;
            } else if tok.is_lit(")") {
                depth -= 1;
                if depth == 0 && i + 1 != t.len() {
                    closes_at_end = false;
                    break;
                }
            }
        }
        if !closes_at_end {
            break;
        }
        t = t.get(1..t.len() - 1).unwrap_or_default();
    }
    t
}

/// Positions of separator words outside brackets.
fn top_level(toks: &[Tok], seps: &[&str]) -> Vec<usize> {
    let mut depth = 0i32;
    let mut out = Vec::new();
    for (i, t) in toks.iter().enumerate() {
        if t.is_lit("(") {
            depth += 1;
        } else if t.is_lit(")") {
            depth -= 1;
        } else if depth == 0 && seps.iter().any(|s| t.is_lit(s)) {
            out.push(i);
        }
    }
    out
}

/// A broken text template or variable name inside the line, if any.
fn inner_error(toks: &[Tok], env: &Env) -> Option<String> {
    toks.iter().find_map(|t| match t {
        Tok::Str(s) | Tok::Var(s) => parse_template(s, env).err(),
        _ => None,
    })
}

fn take_expr(caps: &mut [Option<Capture>], i: usize) -> Option<Expr> {
    match caps.get_mut(i).and_then(Option::take) {
        Some(Capture::Expr(e)) => Some(e),
        _ => None,
    }
}

fn take_var(caps: &mut [Option<Capture>], i: usize) -> Option<VarRef> {
    match caps.get_mut(i).and_then(Option::take) {
        Some(Capture::Var(v)) => Some(v),
        _ => None,
    }
}

fn build_cond(kind: CondKind, mut c: Vec<Option<Capture>>) -> Cond {
    if let CondKind::Rtw(k, negate) = kind {
        let a = take_expr(&mut c, 0).unwrap_or(Expr::Num(0.0));
        let b = take_expr(&mut c, 1);
        return Cond::Rtw(Box::new(crate::w4_skbee_raytrace_world::build_cond(
            k, negate, a, b,
        )));
    }
    if let CondKind::W5c3(k, negate) = kind {
        return Cond::W5c3(Box::new(crate::w5_chunk3::build_cond(k, negate, c)));
    }
    if let CondKind::W3(k) = kind {
        return match crate::w2_conditions_3::build(k, c) {
            Some(w) => Cond::Rare3(Box::new(w)),
            None => Cond::Cmp(Expr::Num(0.0), CmpOp::Eq, Expr::Num(1.0)),
        };
    }
    let a = take_expr(&mut c, 0).unwrap_or(Expr::Num(0.0));
    let b = take_expr(&mut c, 1).unwrap_or(Expr::Num(0.0));
    let op = match kind {
        CondKind::IsPlayer | CondKind::IsNotPlayer | CondKind::IsConsole | CondKind::IsNotConsole => {
            return Cond::SenderIs {
                value: a,
                console: matches!(kind, CondKind::IsConsole | CondKind::IsNotConsole),
                negate: matches!(kind, CondKind::IsNotPlayer | CondKind::IsNotConsole),
            };
        }
        CondKind::Flag(flag, negate) => {
            return Cond::PlayerFlag {
                player: a,
                flag,
                negate,
            };
        }
        CondKind::Admin(k, negate) => return Cond::Admin(k, a, negate),
        CondKind::W2(kind, negate) => {
            let b = (kind_has_b(kind)).then_some(b);
            return Cond::Rare(Box::new(crate::w2_conditions_2::build(kind, negate, a, b)));
        }
        CondKind::W4(kind, negate) => {
            let a = if crate::w4_conditions_1::has_a(kind) {
                a
            } else {
                Expr::Num(0.0)
            };
            let b = crate::w4_conditions_1::has_b(kind).then_some(b);
            return Cond::Rare4(Box::new(crate::w4_conditions_1::build(kind, negate, a, b)));
        }
        CondKind::W3(_) | CondKind::Rtw(..) | CondKind::W5c3(..) => CmpOp::Eq,
        CondKind::IsSet => return Cond::IsSet(a, true),
        CondKind::NotSet => return Cond::IsSet(a, false),
        CondKind::Ne => CmpOp::Ne,
        CondKind::Eq => CmpOp::Eq,
        CondKind::Ge => CmpOp::Ge,
        CondKind::Le => CmpOp::Le,
        CondKind::Gt => CmpOp::Gt,
        CondKind::Lt => CmpOp::Lt,
        CondKind::InGamemode | CondKind::NotInGamemode => {
            let op = if kind == CondKind::InGamemode {
                CmpOp::Eq
            } else {
                CmpOp::Ne
            };
            let mode = Expr::Prop(crate::expr::Prop::Gamemode, Box::new(a));
            return Cond::Cmp(mode, op, b);
        }
        CondKind::HasPerm => {
            return Cond::HasPerm {
                player: a,
                node: b,
                negate: false,
            };
        }
        CondKind::NotHasPerm => {
            return Cond::HasPerm {
                player: a,
                node: b,
                negate: true,
            };
        }
        CondKind::HasItem | CondKind::InvContains => {
            return Cond::HasItem {
                player: a,
                item: b,
                negate: false,
            };
        }
        CondKind::NotHasItem => {
            return Cond::HasItem {
                player: a,
                item: b,
                negate: true,
            };
        }
        CondKind::ItemEnchanted { any, negate } => {
            return Cond::ItemEnch {
                item: a,
                ench: (!any).then_some(b),
                negate,
            };
        }
        CondKind::Between | CondKind::NotBetween => {
            let high = take_expr(&mut c, 2).unwrap_or(Expr::Num(0.0));
            return Cond::Between {
                value: a,
                low: b,
                high,
                negate: kind == CondKind::NotBetween,
            };
        }
        CondKind::Contains
        | CondKind::NotContains
        | CondKind::StartsWith
        | CondKind::NotStartsWith
        | CondKind::EndsWith
        | CondKind::NotEndsWith => {
            let op = match kind {
                CondKind::Contains | CondKind::NotContains => TextOp::Contains,
                CondKind::StartsWith | CondKind::NotStartsWith => TextOp::StartsWith,
                _ => TextOp::EndsWith,
            };
            let negate = matches!(
                kind,
                CondKind::NotContains | CondKind::NotStartsWith | CondKind::NotEndsWith
            );
            return Cond::Text {
                left: a,
                op,
                right: b,
                negate,
            };
        }
    };
    Cond::Cmp(a, op, b)
}

fn build_effect(kind: EffKind, mut c: Vec<Option<Capture>>) -> Option<Effect> {
    let c = &mut c;
    Some(match kind {
        EffKind::Title => Effect::Title {
            title: take_expr(c, 0)?,
            subtitle: take_expr(c, 1),
            to: take_expr(c, 2)?,
        },
        EffKind::ActionBar => Effect::ActionBar {
            text: take_expr(c, 0)?,
            to: take_expr(c, 1)?,
        },
        EffKind::Send => Effect::Send {
            text: take_expr(c, 0)?,
            to: take_expr(c, 1),
        },
        EffKind::Broadcast => Effect::Broadcast(take_expr(c, 0)?),
        EffKind::Hf(op) => return crate::health_food::build(op, c),
        EffKind::W2(op) => return crate::w2_effects_1::build(op, c),
        EffKind::W2b(op) => return crate::w2_effects_2::build(op, c),
        EffKind::X2(k) => return crate::w2_expressions_2::build(k, c),
        EffKind::W2c(stat, mode) => {
            return crate::w2_expressions_3::build(stat, mode, c).map(|e| Effect::W2c(Box::new(e)));
        }
        EffKind::W4(op) => return crate::w2_expressions_4::build(op, c),
        EffKind::W2e3(op) => return crate::w2_effects_3::build(op, c),
        EffKind::W5(op) => return crate::w2_expressions_5::build(op, c),
        EffKind::W6(op) => return crate::w2_expressions_6::build(op, c),
        EffKind::Rtw(op) => return crate::w4_skbee_raytrace_world::build(op, c),
        EffKind::W5c5(op) => return crate::w5_chunk5::build(op, c),
        EffKind::W5c3(op) => return crate::w5_chunk3::build_effect(op, c),
        EffKind::Heal => Effect::Heal(take_expr(c, 0)?),
        EffKind::SetHealth => Effect::SetHealth(take_expr(c, 0)?, take_expr(c, 1)?),
        EffKind::SetGamemode => Effect::SetGamemode(take_expr(c, 0)?, take_expr(c, 1)?),
        EffKind::Xp(op) => Effect::Xp(op, take_expr(c, 0)?, take_expr(c, 1)?),
        EffKind::Kill => Effect::Kill(take_expr(c, 0)?),
        EffKind::Teleport => Effect::Teleport(
            take_expr(c, 0)?,
            take_expr(c, 1)?,
            take_expr(c, 2)?,
            take_expr(c, 3)?,
        ),
        EffKind::TeleportTo => Effect::TeleportTo(take_expr(c, 0)?, take_expr(c, 1)?),
        EffKind::GiveCount => {
            let n = take_expr(c, 0)?;
            let item = take_expr(c, 1)?;
            Effect::Give(take_expr(c, 2)?, Some(n), item)
        }
        EffKind::GiveTo => {
            let item = take_expr(c, 0)?;
            Effect::Give(take_expr(c, 1)?, None, item)
        }
        EffKind::Give => {
            let p = take_expr(c, 0)?;
            Effect::Give(p, None, take_expr(c, 1)?)
        }
        EffKind::ConsoleCommand => Effect::ConsoleCommand(take_expr(c, 0)?),
        EffKind::PlaySound => Effect::PlaySound(Box::new(PlaySound {
            sound: take_expr(c, 0)?,
            volume: take_expr(c, 1),
            pitch: take_expr(c, 2),
            to: take_expr(c, 3),
            at: take_expr(c, 4),
        })),
        EffKind::StopSound => Effect::StopSound(Some(take_expr(c, 0)?), take_expr(c, 1)),
        EffKind::StopAllSounds => Effect::StopSound(None, take_expr(c, 0)),
        EffKind::ExecAs => {
            let who = take_expr(c, 0)?;
            Effect::ExecAs(who, take_expr(c, 1)?)
        }
        EffKind::ExecAsBy => {
            let cmd = take_expr(c, 0)?;
            Effect::ExecAs(take_expr(c, 1)?, cmd)
        }
        EffKind::Cancel => Effect::Cancel,
        EffKind::WaitTicks => Effect::Wait(take_expr(c, 0)?, 1),
        EffKind::WaitSeconds => Effect::Wait(take_expr(c, 0)?, 20),
        EffKind::Set => {
            let v = take_var(c, 0)?;
            match take_expr(c, 1)? {
                Expr::Split(a, b) if v.is_list() => Effect::SetSplit(v, *a, *b),
                e => Effect::Set(v, e),
            }
        }
        EffKind::ReplaceAllIn => Effect::Replace(take_var(c, 1)?, take_expr(c, 0)?, take_expr(c, 2)?, true),
        EffKind::ReplaceFirstIn => {
            Effect::Replace(take_var(c, 1)?, take_expr(c, 0)?, take_expr(c, 2)?, false)
        }
        EffKind::ReplaceAllWith => Effect::Replace(take_var(c, 2)?, take_expr(c, 0)?, take_expr(c, 1)?, true),
        EffKind::ReplaceFirstWith => {
            Effect::Replace(take_var(c, 2)?, take_expr(c, 0)?, take_expr(c, 1)?, false)
        }
        EffKind::Add => {
            let v = take_var(c, 1)?;
            let e = take_expr(c, 0)?;
            if v.is_list() {
                Effect::ListAdd(v, e)
            } else {
                Effect::Add(v, e)
            }
        }
        EffKind::Remove => {
            let v = take_var(c, 1)?;
            let e = take_expr(c, 0)?;
            if v.is_list() {
                Effect::ListRemove(v, e)
            } else {
                Effect::Remove(v, e)
            }
        }
        EffKind::Delete => Effect::Delete(take_var(c, 0)?),
        EffKind::Admin(k) => Effect::Admin(k, take_expr(c, 0)?, take_expr(c, 1)),
    })
}

fn parse_command_header(file: &str, num: usize, head: &str) -> Result<CommandDef, ScriptError> {
    let rest = head.get("command".len()..).unwrap_or("").trim();
    let mut parts = rest.split_whitespace();
    let name = parts.next().unwrap_or("");
    let name = name.strip_prefix('/').unwrap_or(name).to_ascii_lowercase();
    if name.is_empty()
        || !name
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-')
    {
        return Err(ScriptError::new(
            file,
            num,
            "expected 'command /name:' with a simple name",
        ));
    }
    let mut args = Vec::new();
    for p in parts {
        let (inner, optional) = match p.strip_prefix('[').and_then(|s| s.strip_suffix(']')) {
            Some(s) => (s, true),
            None => (p, false),
        };
        let ty = match inner {
            "<text>" | "<string>" => ArgTy::Text,
            "<number>" | "<integer>" => ArgTy::Number,
            "<player>" => ArgTy::Player,
            _ => {
                return Err(ScriptError::new(
                    file,
                    num,
                    format!(
                        "unknown argument '{p}', use <text>, <number>, <player> or [<text>] for optional"
                    ),
                ));
            }
        };
        if !optional && args.iter().any(|a: &ArgDef| a.optional) {
            return Err(ScriptError::new(
                file,
                num,
                "required arguments can't come after optional ones",
            ));
        }
        args.push(ArgDef { ty, optional });
    }
    Ok(CommandDef {
        name,
        permission: None,
        description: None,
        args,
        aliases: Vec::new(),
        usage_msg: None,
        players_only: None,
        cooldown: None,
    })
}

/// Compile one file on its own (no functions from other files).
pub fn compile_file(file: &str, src: &str) -> Result<Vec<Trigger>, Vec<ScriptError>> {
    let mut c = compile_all(&[(file.to_string(), src.to_string())]);
    if c.errors.is_empty() {
        Ok(c.files.pop().map(|(_, t)| t).unwrap_or_default())
    } else {
        Err(c.errors)
    }
}

/// Parsed file: its lines after options were filled in.
struct Parsed {
    name: String,
    tree: Vec<Line>,
    structs: crate::structures::FileStructs,
}

/// Compile a set of files together, so functions can be called across files.
pub fn compile_all(files: &[(String, String)]) -> Compiled {
    let mut out = Compiled::default();
    // Pass 1: block structure, options and every function signature.
    let mut parsed: Vec<Parsed> = Vec::new();
    let mut sigs: Vec<Sig> = Vec::new();
    let mut func_src: Vec<(usize, usize)> = Vec::new(); // (parsed index, top line index)
    let mut defaults_src: Vec<Vec<Option<String>>> = Vec::new();
    // Per function: declared `local function`, and the index of its file.
    let mut sig_scope: Vec<(bool, usize)> = Vec::new();
    for (name, src) in files {
        // Like Skript, errors never stop a file: what is broken is reported
        // and left out, the rest loads.
        let (tree, errs) = parse_blocks(name, src);
        out.errors.extend(errs);
        // Most files have no options; skip the extra pass for them.
        let has_options = src.contains("{@") || tree.iter().any(|t| t.text.eq_ignore_ascii_case("options:"));
        let tree = if has_options {
            let (t, errs) = apply_options(name, tree);
            out.errors.extend(errs);
            t
        } else {
            tree
        };
        let (tree, structs, errs) = crate::structures::extract(name, tree);
        out.errors.extend(errs);
        for (ti, top) in tree.iter().enumerate() {
            if let Some((head, local)) = crate::structures::function_head(top.head()) {
                match parse_function_header(name, top.num, head) {
                    Ok((sig, defs)) => {
                        if crate::structures::function_clash(
                            &sigs,
                            &sig_scope,
                            &sig.name,
                            local,
                            parsed.len(),
                        ) {
                            out.errors.push(ScriptError::new(
                                name,
                                top.num,
                                format!("function {}() is defined twice", sig.name),
                            ));
                            continue;
                        }
                        sigs.push(sig);
                        sig_scope.push((local, parsed.len()));
                        defaults_src.push(defs);
                        func_src.push((parsed.len(), ti));
                    }
                    Err(e) => out.errors.push(e),
                }
            }
        }
        parsed.push(Parsed {
            name: name.clone(),
            tree,
            structs,
        });
    }
    // Pass 1b: default values. They are part of the header: a function with a
    // bad default doesn't load, and calls to it are errors where they are.
    let mut all_defaults: Vec<Vec<Option<Expr>>> = Vec::with_capacity(func_src.len());
    {
        let env = Env {
            functions: Arc::new(sigs.clone()),
            in_function: true,
            ..Env::default()
        };
        for (id, &(pi, ti)) in func_src.iter().enumerate() {
            let mut defaults = Vec::new();
            let (Some(p), Some(top)) = (parsed.get(pi), parsed.get(pi).and_then(|p| p.tree.get(ti))) else {
                all_defaults.push(defaults);
                continue;
            };
            let mut bad = false;
            // Defaults are plain values: compile them without parameters in scope.
            for d in defaults_src.get(id).map(Vec::as_slice).unwrap_or_default() {
                defaults.push(match d {
                    Some(text) => match lex(text).and_then(|t| parse_expr(&t, &env)) {
                        Ok(e) => Some(e),
                        Err(e) => {
                            out.errors.push(ScriptError::new(
                                &p.name,
                                top.num,
                                format!("bad default value '{text}': {e}"),
                            ));
                            bad = true;
                            None
                        }
                    },
                    None => None,
                });
            }
            if bad && let Some(s) = sigs.get_mut(id) {
                s.failed = true;
            }
            all_defaults.push(defaults);
        }
    }
    // Each file sees the global functions plus its own local ones.
    let views = crate::structures::function_views(&sigs, &sig_scope, parsed.len());

    // Pass 2: function bodies.
    for (id, &(pi, ti)) in func_src.iter().enumerate() {
        let (Some(p), Some(sig)) = (parsed.get(pi), sigs.get(id)) else {
            out.functions.push(None);
            continue;
        };
        let Some(top) = p.tree.get(ti).filter(|_| !sig.failed) else {
            out.functions.push(None);
            continue;
        };
        let env = Env {
            trigger_name: format!("function {}()", sig.name),
            functions: views.get(pi).cloned().unwrap_or_default(),
            in_function: true,
            ..Env::default()
        };
        let defaults = all_defaults.get_mut(id).map(std::mem::take).unwrap_or_default();
        let mut cx = Cx {
            file: &p.name,
            env,
            is_command: false,
            func_ret: Some(sig.ret),
            no_cancel: None,
            code: Vec::new(),
            lines: Vec::new(),
            errors: Vec::new(),
            loops: Vec::new(),
            catch_depth: 0,
        };
        cx.block(&top.children);
        out.errors.append(&mut cx.errors);
        out.functions.push(Some(Arc::new(Function {
            sig: sig.clone(),
            defaults,
            code: Arc::new(Code {
                file: Arc::from(p.name.as_str()),
                line: top.num,
                instrs: cx.code,
                lines: cx.lines,
            }),
        })));
    }

    // Pass 3: triggers.
    for (pi, p) in parsed.iter().enumerate() {
        let Some(view) = views.get(pi) else { continue };
        let (trigs, mut errs) = compile_triggers(&p.name, &p.tree, view);
        let defaults = crate::structures::compile_defaults(&p.name, &p.structs, view, &mut errs);
        out.errors.extend(errs);
        out.var_defaults.extend(defaults);
        out.files.push((p.name.clone(), trigs));
    }
    crate::error::fill_code(&mut out.errors, files);
    out
}

/// Replace `{@name}` with the values from the file's `options:` blocks.
/// A line with a broken option is reported and left out (with its block).
fn apply_options(file: &str, tree: Vec<Line>) -> (Vec<Line>, Vec<ScriptError>) {
    let mut opts: Vec<(String, String)> = Vec::new();
    let mut rest = Vec::new();
    let mut errors = Vec::new();
    for top in tree {
        if top.text.eq_ignore_ascii_case("options:") {
            for o in &top.children {
                match o.text.split_once(':') {
                    Some((k, v)) if !k.trim().is_empty() => {
                        opts.push((k.trim().to_ascii_lowercase(), v.trim().to_string()))
                    }
                    _ => errors.push(ScriptError::new(file, o.num, "options look like 'name: value'")),
                }
            }
        } else {
            rest.push(top);
        }
    }
    /// False when the line itself has a broken option.
    fn fill(line: &mut Line, opts: &[(String, String)], file: &str, errors: &mut Vec<ScriptError>) -> bool {
        while let Some(start) = line.text.find("{@") {
            let Some(len) = line.text.get(start..).and_then(|t| t.find('}')) else {
                errors.push(ScriptError::new(
                    file,
                    line.num,
                    "unclosed option: missing '}' after '{@'",
                ));
                return false;
            };
            let name = line
                .text
                .get(start + 2..start + len)
                .unwrap_or("")
                .trim()
                .to_ascii_lowercase();
            match opts.iter().find(|(k, _)| *k == name) {
                Some((_, v)) => line.text.replace_range(start..start + len + 1, v),
                None => {
                    errors.push(ScriptError::new(
                        file,
                        line.num,
                        format!("unknown option {{@{name}}}"),
                    ));
                    return false;
                }
            }
        }
        line.children.retain_mut(|c| fill(c, opts, file, errors));
        true
    }
    rest.retain_mut(|l| fill(l, &opts, file, &mut errors));
    (rest, errors)
}

pub fn type_from_name(name: &str) -> Option<Ty> {
    Some(match name.trim().to_ascii_lowercase().as_str() {
        "text" | "string" => Ty::Text,
        "number" | "integer" | "int" => Ty::Number,
        "player" | "offline player" | "offlineplayer" => Ty::Player,
        "boolean" | "bool" => Ty::Boolean,
        "item" | "itemtype" | "item type" | "block" => Ty::Item,
        "location" => Ty::Location,
        "world" => Ty::World,
        "timespan" | "time span" => Ty::Timespan,
        "vector" => Ty::Vector,
        "date" => Ty::Date,
        "object" => Ty::Object,
        "numbers" | "integers" | "texts" | "strings" | "players" | "booleans" | "items" | "itemtypes"
        | "locations" | "worlds" | "objects" => Ty::List,
        _ => return crate::types_1::ty_from_name(name),
    })
}

/// Split at commas that are outside quotes and brackets.
fn split_commas(s: &str) -> Vec<&str> {
    let mut out = Vec::new();
    let (mut depth, mut in_str, mut last) = (0i32, false, 0usize);
    for (i, c) in s.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '(' | '{' if !in_str => depth += 1,
            ')' | '}' if !in_str => depth -= 1,
            ',' if !in_str && depth == 0 => {
                out.push(s.get(last..i).unwrap_or(""));
                last = i + 1;
            }
            _ => {}
        }
    }
    out.push(s.get(last..).unwrap_or(""));
    out
}

/// `function name(a: text, b: number = 5) :: text` (the trailing `:` already removed).
fn parse_function_header(
    file: &str,
    num: usize,
    head: &str,
) -> Result<(Sig, Vec<Option<String>>), ScriptError> {
    let err = |m: String| ScriptError::new(file, num, m);
    let rest = head.get("function".len()..).unwrap_or("").trim();
    let (name, after) = rest
        .split_once('(')
        .ok_or_else(|| err("expected 'function name(...)'".into()))?;
    let name = name.trim();
    if name.is_empty() || !name.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(err(format!(
            "bad function name '{name}': use letters, digits and _"
        )));
    }
    let close = after
        .rfind(')')
        .ok_or_else(|| err("missing ')' in the function header".into()))?;
    let params_src = after.get(..close).unwrap_or("");
    let tail = after.get(close + 1..).unwrap_or("").trim();
    let ret = if tail.is_empty() {
        None
    } else {
        let t = tail.strip_prefix("::").ok_or_else(|| {
            err(format!(
                "unexpected '{tail}' after the parameters (use ':: type' for the return type)"
            ))
        })?;
        Some(type_from_name(t).ok_or_else(|| err(format!("unknown return type '{}'", t.trim())))?)
    };
    let mut params = Vec::new();
    let mut defaults = Vec::new();
    if !params_src.trim().is_empty() {
        for p in split_commas(params_src) {
            let (decl, default) = match p.split_once('=') {
                Some((d, v)) => (d, Some(v.trim().to_string())),
                None => (p, None),
            };
            let (pname, pty) = decl.split_once(':').ok_or_else(|| {
                err(format!(
                    "parameter '{}' needs a type, like 'name: text'",
                    decl.trim()
                ))
            })?;
            let pname = pname.trim().to_ascii_lowercase();
            let ty = type_from_name(pty).ok_or_else(|| {
                err(format!("unknown type '{}' for parameter {pname} (use text, number, player, boolean, item, location, world or object; for lists use numbers, texts, players, objects and so on)", pty.trim()))
            })?;
            if default.is_none() && defaults.iter().any(Option::is_some) {
                return Err(err(format!(
                    "parameter {pname} needs a default, because the one before it has one"
                )));
            }
            params.push((format!("_{pname}"), ty, default.is_some()));
            defaults.push(default);
        }
    }
    Ok((
        Sig {
            name: name.to_string(),
            params,
            ret,
            failed: false,
        },
        defaults,
    ))
}

/// `right click with tnt on barrel` -> ("right click", [("with", "tnt"), ("on", "barrel")]).
fn split_click_filters(ev: &str) -> (&str, Vec<(&'static str, String)>) {
    let mut base = ev;
    let mut filters = Vec::new();
    // Find the first ` with ` / ` on ` and split from there.
    let first = [" with ", " on "].iter().filter_map(|k| ev.find(k)).min();
    if let Some(i) = first {
        base = ev.get(..i).unwrap_or(ev).trim();
        let mut rest = ev.get(i..).unwrap_or("").trim();
        while !rest.is_empty() {
            let (key, after) = if let Some(r) = rest.strip_prefix("with ") {
                ("with", r)
            } else if let Some(r) = rest.strip_prefix("on ") {
                ("on", r)
            } else {
                break;
            };
            let next = [" with ", " on "]
                .iter()
                .filter_map(|k| after.find(k))
                .min()
                .unwrap_or(after.len());
            filters.push((key, after.get(..next).unwrap_or(after).trim().to_string()));
            rest = after.get(next..).unwrap_or("").trim();
        }
    }
    (base, filters)
}

/// `5 seconds`, `1 minute`, `20 ticks`, `second` -> ticks.
fn parse_every(s: &str) -> Option<u64> {
    let mut parts = s.split_whitespace();
    let (n, unit) = match (parts.next(), parts.next(), parts.next()) {
        (Some(n), Some(u), None) => (n.parse::<f64>().ok()?, u),
        (Some(u), None, None) => (1.0, u),
        _ => return None,
    };
    let per = match unit {
        "tick" | "ticks" => 1.0,
        "second" | "seconds" => 20.0,
        "minute" | "minutes" => 1200.0,
        "hour" | "hours" => 72000.0,
        _ => return None,
    };
    let ticks = (n * per).round();
    (ticks >= 1.0 && ticks.is_finite()).then_some(ticks as u64)
}

fn compile_triggers(file: &str, tree: &[Line], sigs: &Arc<Vec<Sig>>) -> (Vec<Trigger>, Vec<ScriptError>) {
    let file_arc: Arc<str> = Arc::from(file);
    let mut triggers = Vec::new();
    let mut errors = Vec::new();
    let mut click_filters: Vec<(&'static str, String)> = Vec::new();
    for top in tree {
        click_filters.clear();
        let lower = top.head().to_ascii_lowercase();
        if crate::structures::function_head(top.head()).is_some() {
            continue;
        }
        if !top.opens_block() {
            errors.push(ScriptError::new(
                file,
                top.num,
                format!(
                    "expected a trigger like 'on join:' or 'command /name:', got '{}'",
                    top.text
                ),
            ));
            continue;
        }
        let (kind, body, env, is_command): (TriggerKind, &[Line], Env, bool) = if let Some(ev) =
            lower.strip_prefix("on ")
        {
            let (ev, ev_filter) = crate::events_items::split_filter(ev.trim());
            let (ev, ph_prefix) = crate::placeholders::split_header(ev);
            let (ev, cmd_filter) = crate::w4_events_movement::split_command_filter(ev);
            let (ev, mut filters) = split_click_filters(ev);
            filters.extend(ev_filter);
            filters.extend(ph_prefix.map(|p| ("prefix", p)));
            filters.extend(cmd_filter.map(|t| ("command", t)));
            let Some(k) = EventKind::parse(ev) else {
                errors.push(ScriptError::new(
                        file,
                        top.num,
                        format!(
                            "unknown event '{ev}', known: join, quit, chat, break, place, death, damage, respawn, click, right click, left click, inventory click, inventory close, sneak toggle, sprint toggle, flight toggle, bound enter, bound exit, load, {}, {}, and the extra events in docs/syntax/w2-events-3.md (and 'every N seconds:')",
                            crate::events_items::NAMES,
                            crate::w2_events_2::known_names()
                        ),
                    ));
                continue;
            };
            if filters.iter().any(|(w, _)| !crate::events_items::filter_ok(k, w)) {
                errors.push(ScriptError::new(
                    file,
                    top.num,
                    "'on ... with/on <item>' only works for click events, 'of <mob>' for spawns and 'to <mode>' for gamemode changes",
                ));
                continue;
            }
            let mut env = Env {
                has_message: k == EventKind::Chat || matches!(k, EventKind::Ext3(e) if e.has_text()),
                arg_types: vec![],
                trigger_name: format!("'on {}'", k.name()),
                no_player: matches!(
                    k,
                    EventKind::Load | EventKind::MobSpawn | EventKind::ProjectileHit
                ) || matches!(k, EventKind::Ext(e) if !e.has_player() && !e.maybe_player())
                    || matches!(k, EventKind::Ext3(e) if !e.has_player()),
                event_vals: crate::events_items::mask(k),
                ext4: if let EventKind::Ext4(e) = k { Some(e) } else { None },
                ev_fields: match k {
                    EventKind::Ext(e) => e.fields(),
                    EventKind::Mv(e) => e.fields(),
                    _ => &[],
                },
                has_damage: k == EventKind::Damage || matches!(k, EventKind::Ext3(e) if e.has_number()),
                has_attacker: matches!(k, EventKind::Damage | EventKind::Death),
                has_block: matches!(k, EventKind::Click | EventKind::RightClick | EventKind::LeftClick),
                has_inv: k == EventKind::InventoryClick,
                has_bound: matches!(k, EventKind::BoundEnter | EventKind::BoundExit),
                ..Env::default()
            };
            if let EventKind::Ext2(e) = k {
                crate::w2_events_2::apply_env(&mut env, e);
            }
            click_filters.extend(filters);
            (TriggerKind::Event(k), &top.children, env, false)
        } else if let Some(minute) = crate::w2_events_4::parse_at_real_time(&lower) {
            let k = EventKind::Ext4(crate::w2_events_4::ExtEvent::SystemTime(minute));
            let env = Env {
                trigger_name: format!("'at {:02}:{:02} in real time'", minute / 60, minute % 60),
                no_player: true,
                ext4: Some(crate::w2_events_4::ExtEvent::SystemTime(minute)),
                ..Env::default()
            };
            (TriggerKind::Event(k), &top.children, env, false)
        } else if let Some(rest) = lower.strip_prefix("at ") {
            let Some(t) = crate::w2_events_1::parse_at_time(rest.trim().trim_end_matches(':')) else {
                errors.push(ScriptError::new(
                    file,
                    top.num,
                    format!("can't understand 'at {rest}', use 'at 18:00:', 'at 6:30 pm:', 'at noon:' or 'at midnight:'"),
                ));
                continue;
            };
            let env = Env {
                trigger_name: format!("'at {}'", rest.trim().trim_end_matches(':')),
                no_player: true,
                ..Env::default()
            };
            (
                TriggerKind::Event(EventKind::Ext(crate::w2_events_1::ExtEvent::At(t))),
                &top.children,
                env,
                false,
            )
        } else if let Some(rest) = lower.strip_prefix("every ") {
            let Some(ticks) = parse_every(rest) else {
                errors.push(ScriptError::new(
                        file,
                        top.num,
                        format!("can't understand 'every {rest}', use 'every 5 seconds:', 'every 20 ticks:' or 'every 2 minutes:'"),
                    ));
                continue;
            };
            let env = Env {
                trigger_name: format!("'every {rest}'"),
                no_player: true,
                ..Env::default()
            };
            (TriggerKind::Every(ticks), &top.children, env, false)
        } else if lower.starts_with("command") {
            let mut def = match parse_command_header(file, top.num, top.head()) {
                Ok(d) => d,
                Err(e) => {
                    errors.push(e);
                    continue;
                }
            };
            let mut body: Option<&[Line]> = None;
            for entry in &top.children {
                let el = entry.text.to_ascii_lowercase();
                if el == "trigger:" {
                    body = Some(&entry.children);
                } else if el.starts_with("permission:") {
                    let v = entry.text.get("permission:".len()..).unwrap_or("").trim();
                    def.permission = Some(v.to_string()).filter(|s| !s.is_empty());
                } else if el.starts_with("aliases:") {
                    let v = entry.text.get("aliases:".len()..).unwrap_or("");
                    def.aliases = v
                        .split(',')
                        .map(|a| a.trim().trim_start_matches('/').to_ascii_lowercase())
                        .filter(|a| !a.is_empty())
                        .collect();
                } else if el.starts_with("usage:") {
                    def.usage_msg = Some(entry.text.get("usage:".len()..).unwrap_or("").trim().to_string());
                } else if el.starts_with("executable by:") {
                    let v = el.get("executable by:".len()..).unwrap_or("").trim().to_string();
                    def.players_only = match v.as_str() {
                        "players" | "player" => Some(true),
                        "console" => Some(false),
                        "players and console" | "console and players" => None,
                        _ => {
                            errors.push(ScriptError::new(
                                file,
                                entry.num,
                                "use 'executable by: players' or 'console'",
                            ));
                            None
                        }
                    };
                } else if el.starts_with("cooldown") {
                    match crate::w5_chunk1::command_entry(&mut def.cooldown, &el, &entry.text) {
                        Ok(true) => {}
                        Ok(false) => errors.push(ScriptError::new(
                            file,
                            entry.num,
                            "use cooldown:, cooldown message: or cooldown bypass:",
                        )),
                        Err(e) => errors.push(ScriptError::new(file, entry.num, e)),
                    }
                } else if el.starts_with("description:") {
                    def.description = Some(
                        entry
                            .text
                            .get("description:".len()..)
                            .unwrap_or("")
                            .trim()
                            .to_string(),
                    );
                } else {
                    errors.push(ScriptError::new(
                        file,
                        entry.num,
                        format!(
                            "unknown command entry '{}', use permission:, description:, aliases:, usage:, executable by: or trigger:",
                            entry.text
                        ),
                    ));
                }
            }
            let Some(body) = body else {
                errors.push(ScriptError::new(
                    file,
                    top.num,
                    "command has no 'trigger:' section",
                ));
                continue;
            };
            let arg_types = def
                .args
                .iter()
                .map(|a| match a.ty {
                    ArgTy::Text => Ty::Text,
                    ArgTy::Number => Ty::Number,
                    ArgTy::Player => Ty::Player,
                })
                .collect();
            let env = Env {
                has_message: false,
                arg_types,
                trigger_name: format!("command /{}", def.name),
                ..Env::default()
            };
            (TriggerKind::Command(def), body, env, true)
        } else {
            errors.push(ScriptError::new(
                file,
                top.num,
                format!(
                    "unknown trigger '{}', expected 'on <event>:' or 'command /name:'",
                    top.head()
                ),
            ));
            continue;
        };
        let no_cancel = match &kind {
            TriggerKind::Event(k) if !k.cancellable() => {
                Some(format!("'on {}' can't be cancelled", k.name()))
            }
            TriggerKind::Every(_) => Some("'every' has no event to cancel".to_string()),
            _ => None,
        };
        let mut cx = Cx {
            file,
            env,
            is_command,
            func_ret: None,
            no_cancel,
            code: Vec::new(),
            lines: Vec::new(),
            errors: Vec::new(),
            loops: Vec::new(),
            catch_depth: 0,
        };
        cx.env.functions = sigs.clone();
        // `on right click with diamond on stone:` checks first.
        let mut guards = Vec::new();
        let mut header_ok = true;
        for (what, text) in std::mem::take(&mut click_filters) {
            if what == "command" {
                let (left, right) = crate::w4_events_movement::command_guard(&text);
                let at = cx.emit(
                    Instr::JumpIfNot(Cond::Cmp(left, CmpOp::Eq, right), usize::MAX),
                    top.num,
                );
                guards.push(at);
                continue;
            }
            let item = if let Some(e) = crate::events_items::filter_rhs(what, &text) {
                e
            } else if matches!(what, "entity" | "mode" | "victim") {
                cx.err(top.num, format!("'{text}' is not a valid name here"));
                header_ok = false;
                continue;
            } else {
                match lex(&text).and_then(|t| parse_expr(&t, &cx.env)) {
                    Ok(e) if e.ty(&cx.env) == Ty::Item => e,
                    _ => {
                        cx.err(top.num, format!("'{text}' is not an item or block name"));
                        header_ok = false;
                        continue;
                    }
                }
            };
            let left = if what == "with" {
                Expr::Prop(crate::expr::Prop::Tool, Box::new(Expr::Player))
            } else if let Some(l) = crate::events_items::filter_lhs(what) {
                l
            } else {
                Expr::ClickedBlock
            };
            let at = cx.emit(
                Instr::JumpIfNot(Cond::Cmp(left, CmpOp::Eq, item), usize::MAX),
                top.num,
            );
            guards.push(at);
        }
        // A broken header skips the whole trigger, like Skript.
        if !header_ok {
            errors.append(&mut cx.errors);
            continue;
        }
        cx.block(body);
        let end = cx.code.len();
        for g in guards {
            if let Some(Instr::JumpIfNot(_, t)) = cx.code.get_mut(g) {
                *t = end;
            }
        }
        errors.append(&mut cx.errors);
        triggers.push(Trigger {
            file: file_arc.clone(),
            line: top.num,
            kind,
            code: Arc::new(Code {
                file: file_arc.clone(),
                line: top.num,
                instrs: cx.code,
                lines: cx.lines,
            }),
        });
    }
    (triggers, errors)
}

fn kind_has_b(k: crate::w2_conditions_2::W2Kind) -> bool {
    use crate::w2_conditions_2::W2Kind::*;
    matches!(k, Divisible | InWorld | OfType)
}

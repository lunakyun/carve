//! Extra event triggers: kick, language change, leash, piston, portal,
//! redstone and more. Each one is a plain event kind. The game side fills
//! the event context (`player`, `event-text`, `event-number`), and the rest
//! of the engine treats them like `on join`.
//!
//! * `event-text` is the text value of the event (kick reason, locale,
//!   piston direction, ...). It shares the slot of `message`.
//! * `event-number` is the number value (repair amount, new redstone
//!   power, ...). It shares the slot of `damage`.

use crate::expr::Ty;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerServerEvent {
    Kick,
    LanguageChange,
    Leash,
    Unleash,
    LeavesDecay,
    LightningStrike,
    LoveModeEnter,
    PiglinBarter,
    PistonExtend,
    PistonRetract,
    Portal,
    PortalEnter,
    PortalCreate,
    Redstone,
    ResourcePackResponse,
    ResurrectAttempt,
    SendCommandList,
    LootGenerate,
    PlayerInput,
}

/// Name, aliases (all lowercase), has `event-text`, has `event-number`,
/// has a player.
type Row = (PlayerServerEvent, &'static [&'static str], bool, bool, bool);

const ROWS: &[Row] = &[
    (
        PlayerServerEvent::Kick,
        &["kick", "player kick"],
        true,
        false,
        true,
    ),
    (
        PlayerServerEvent::LanguageChange,
        &["language change", "locale change", "player language change"],
        true,
        false,
        true,
    ),
    (
        PlayerServerEvent::Leash,
        &["leash", "player leash"],
        false,
        false,
        true,
    ),
    (PlayerServerEvent::Unleash, &["unleash"], true, false, false),
    (
        PlayerServerEvent::LeavesDecay,
        &["leaves decay"],
        false,
        false,
        false,
    ),
    (
        PlayerServerEvent::LightningStrike,
        &["lightning strike", "lightning"],
        false,
        false,
        false,
    ),
    (
        PlayerServerEvent::LoveModeEnter,
        &["love mode enter", "enter love mode"],
        false,
        true,
        true,
    ),
    (
        PlayerServerEvent::PiglinBarter,
        &["piglin barter"],
        false,
        false,
        false,
    ),
    (
        PlayerServerEvent::PistonExtend,
        &["piston extend"],
        true,
        false,
        false,
    ),
    (
        PlayerServerEvent::PistonRetract,
        &["piston retract"],
        true,
        false,
        false,
    ),
    (
        PlayerServerEvent::Portal,
        &["portal", "entity portal"],
        false,
        false,
        true,
    ),
    (
        PlayerServerEvent::PortalEnter,
        &["portal enter", "entity portal enter"],
        false,
        false,
        true,
    ),
    (
        PlayerServerEvent::PortalCreate,
        &["portal create"],
        true,
        false,
        false,
    ),
    (
        PlayerServerEvent::Redstone,
        &["redstone", "redstone change"],
        false,
        true,
        false,
    ),
    (
        PlayerServerEvent::ResourcePackResponse,
        &[
            "resource pack response",
            "resource pack request response",
            "resourcepack response",
        ],
        true,
        false,
        true,
    ),
    (
        PlayerServerEvent::ResurrectAttempt,
        &["resurrect attempt", "resurrect", "totem"],
        false,
        false,
        true,
    ),
    (
        PlayerServerEvent::SendCommandList,
        &["send command list", "command list send"],
        true,
        false,
        true,
    ),
    (
        PlayerServerEvent::LootGenerate,
        &["loot generate"],
        true,
        false,
        false,
    ),
    (
        PlayerServerEvent::PlayerInput,
        &["player input", "input"],
        true,
        true,
        true,
    ),
];

impl PlayerServerEvent {
    pub fn parse(s: &str) -> Option<PlayerServerEvent> {
        ROWS.iter().find(|r| r.1.contains(&s)).map(|r| r.0)
    }

    fn row(self) -> &'static Row {
        ROWS.iter()
            .find(|r| r.0 == self)
            .expect("every PlayerServerEvent has a row")
    }

    /// The main script name.
    pub fn name(self) -> &'static str {
        self.row().1[0]
    }

    /// `event-text` / `message` is available.
    pub fn has_text(self) -> bool {
        self.row().2
    }

    /// `event-number` is available.
    pub fn has_number(self) -> bool {
        self.row().3
    }

    /// The event can have a player; `player` is unset when the entity is not
    /// one.
    pub fn has_player(self) -> bool {
        self.row().4
    }

    pub fn all() -> impl Iterator<Item = PlayerServerEvent> {
        ROWS.iter().map(|r| r.0)
    }

    /// The `event-...` values of the event, in the order the host fills
    /// `EventCtx::ev`.
    pub fn fields(self) -> &'static [(&'static str, Ty)] {
        const LOC: &[(&str, Ty)] = &[("event-location", Ty::Location)];
        const ENT: &[(&str, Ty)] = &[("event-entity", Ty::Object)];
        const BARTER: &[(&str, Ty)] = &[("event-item", Ty::Item), ("event-items", Ty::List)];
        match self {
            PlayerServerEvent::LeavesDecay
            | PlayerServerEvent::LightningStrike
            | PlayerServerEvent::PistonExtend
            | PlayerServerEvent::PistonRetract
            | PlayerServerEvent::PortalCreate
            | PlayerServerEvent::Redstone => LOC,
            PlayerServerEvent::LoveModeEnter | PlayerServerEvent::Leash | PlayerServerEvent::Unleash => ENT,
            PlayerServerEvent::PiglinBarter => BARTER,
            _ => &[],
        }
    }
}

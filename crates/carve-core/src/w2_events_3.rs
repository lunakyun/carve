//! Extra event triggers: kick, language change, leash, piston, portal,
//! redstone and more. Each one is a plain event kind. The game side fills
//! the event context (`player`, `event-text`, `event-number`), and the rest
//! of the engine treats them like `on join`.
//!
//! * `event-text` is the text value of the event (kick reason, locale,
//!   piston direction, ...). It shares the slot of `message`.
//! * `event-number` is the number value (repair amount, new redstone
//!   power, ...). It shares the slot of `damage`.

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ev3 {
    ItemMend,
    ItemMerge,
    ItemSpawn,
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
type Row = (Ev3, &'static [&'static str], bool, bool, bool);

const ROWS: &[Row] = &[
    (Ev3::ItemMend, &["item mend", "player item mend", "mend"], true, true, true),
    (Ev3::ItemMerge, &["item merge"], false, false, false),
    (Ev3::ItemSpawn, &["item spawn"], true, false, false),
    (Ev3::Kick, &["kick", "player kick"], true, false, true),
    (
        Ev3::LanguageChange,
        &["language change", "locale change", "player language change"],
        true,
        false,
        true,
    ),
    (Ev3::Leash, &["leash", "player leash"], false, false, true),
    (Ev3::Unleash, &["unleash"], true, false, false),
    (Ev3::LeavesDecay, &["leaves decay"], false, false, false),
    (
        Ev3::LightningStrike,
        &["lightning strike", "lightning"],
        false,
        false,
        false,
    ),
    (
        Ev3::LoveModeEnter,
        &["love mode enter", "enter love mode"],
        false,
        true,
        true,
    ),
    (Ev3::PiglinBarter, &["piglin barter"], false, false, false),
    (Ev3::PistonExtend, &["piston extend"], true, false, false),
    (Ev3::PistonRetract, &["piston retract"], true, false, false),
    (Ev3::Portal, &["portal", "entity portal"], false, false, true),
    (Ev3::PortalEnter, &["portal enter", "entity portal enter"], false, false, true),
    (Ev3::PortalCreate, &["portal create"], true, false, false),
    (Ev3::Redstone, &["redstone", "redstone change"], false, true, false),
    (
        Ev3::ResourcePackResponse,
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
        Ev3::ResurrectAttempt,
        &["resurrect attempt", "resurrect", "totem"],
        false,
        false,
        true,
    ),
    (
        Ev3::SendCommandList,
        &["send command list", "command list send"],
        true,
        false,
        true,
    ),
    (Ev3::LootGenerate, &["loot generate"], true, false, false),
    (Ev3::PlayerInput, &["player input", "input"], true, true, true),
];

impl Ev3 {
    pub fn parse(s: &str) -> Option<Ev3> {
        ROWS.iter().find(|r| r.1.contains(&s)).map(|r| r.0)
    }

    fn row(self) -> &'static Row {
        ROWS.iter().find(|r| r.0 == self).expect("every Ev3 has a row")
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

    pub fn all() -> impl Iterator<Item = Ev3> {
        ROWS.iter().map(|r| r.0)
    }
}

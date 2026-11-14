//! Player state checks: `player is sneaking`, `player is flying`, and so on.

use crate::compile::CondKind;

/// A yes/no state of a player that the host can report.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PlayerFlag {
    Flying,
    CanFly,
    Sneaking,
    Sprinting,
    Swimming,
    OnGround,
    Blocking,
    Sleeping,
    Burning,
    CustomNameVisible,
    Frozen,
    Gliding,
    Invisible,
    Invulnerable,
    ResourcePack,
    Glowing,
}

impl PlayerFlag {
    pub fn name(self) -> &'static str {
        match self {
            PlayerFlag::Flying => "flying",
            PlayerFlag::CanFly => "can fly",
            PlayerFlag::Sneaking => "sneaking",
            PlayerFlag::Sprinting => "sprinting",
            PlayerFlag::Swimming => "swimming",
            PlayerFlag::OnGround => "on ground",
            PlayerFlag::Blocking => "blocking",
            PlayerFlag::Sleeping => "sleeping",
            PlayerFlag::Burning => "burning",
            PlayerFlag::CustomNameVisible => "custom name visible",
            PlayerFlag::Frozen => "frozen",
            PlayerFlag::Gliding => "gliding",
            PlayerFlag::Invisible => "invisible",
            PlayerFlag::Invulnerable => "invulnerable",
            PlayerFlag::ResourcePack => "resource pack",
            PlayerFlag::Glowing => "glowing",
        }
    }
}

/// Condition patterns for player states. Tried before the general
/// condition patterns so `player is flying` is not read as a comparison.
pub const PLAYER_FLAG_PATTERNS: &[(&str, CondKind)] = &[
    ("%player% is flying", CondKind::Flag(PlayerFlag::Flying, false)),
    ("%player% (is not|isn't) flying", CondKind::Flag(PlayerFlag::Flying, true)),
    ("%player% is sneaking", CondKind::Flag(PlayerFlag::Sneaking, false)),
    ("%player% (is not|isn't) sneaking", CondKind::Flag(PlayerFlag::Sneaking, true)),
    ("%player% is sprinting", CondKind::Flag(PlayerFlag::Sprinting, false)),
    ("%player% (is not|isn't) sprinting", CondKind::Flag(PlayerFlag::Sprinting, true)),
    ("%player% is swimming", CondKind::Flag(PlayerFlag::Swimming, false)),
    ("%player% (is not|isn't) swimming", CondKind::Flag(PlayerFlag::Swimming, true)),
    ("%player% is blocking", CondKind::Flag(PlayerFlag::Blocking, false)),
    ("%player% (is not|isn't) blocking", CondKind::Flag(PlayerFlag::Blocking, true)),
    ("%player% is sleeping", CondKind::Flag(PlayerFlag::Sleeping, false)),
    ("%player% (is not|isn't) sleeping", CondKind::Flag(PlayerFlag::Sleeping, true)),
    ("%player% is (gliding|fall flying)", CondKind::Flag(PlayerFlag::Gliding, false)),
    ("%player% (is not|isn't) (gliding|fall flying)", CondKind::Flag(PlayerFlag::Gliding, true)),
    ("%player% is glowing", CondKind::Flag(PlayerFlag::Glowing, false)),
    ("%player% (is not|isn't) glowing", CondKind::Flag(PlayerFlag::Glowing, true)),
    ("%player% is on [the] ground", CondKind::Flag(PlayerFlag::OnGround, false)),
    ("%player% (is not|isn't) on [the] ground", CondKind::Flag(PlayerFlag::OnGround, true)),
    ("%player% can fly", CondKind::Flag(PlayerFlag::CanFly, false)),
    ("%player% (can't|cannot|can not) fly", CondKind::Flag(PlayerFlag::CanFly, true)),
];

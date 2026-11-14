//! Carve core: a small Skript-like language. No game dependencies; the
//! game is reached only through the [`Host`] trait.

pub mod admin;
pub mod blocks;
pub mod colors;
pub mod compile;
pub mod engine;
pub mod error;
pub mod expr;
pub mod gamemode;
pub mod health_food;
pub mod host;
pub mod items;
pub mod lexer;
pub mod list_literals;
pub mod list_ops;
pub mod pattern;
pub mod sounds;
pub mod text_join_split;
pub mod text_case_substring;
pub mod time_dates;
pub mod senders;
pub mod player_flags;
pub mod timespans;
pub mod value;
pub mod vars;
pub mod w2_expressions_4;
pub mod xp_level;

pub use compile::{CommandDef, EventKind};
pub use engine::{CommandOutcome, Engine, EventCtx, FireOutcome, LoadSummary};
pub use error::ScriptError;
pub use host::{Host, LogLevel};
pub use player_flags::PlayerFlag;
pub use value::{PlayerRef, Recipient, Value};
pub use vars::Variables;

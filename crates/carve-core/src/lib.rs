//! Carve core: a small Skript-like language. No game dependencies; the
//! game is reached only through the [`Host`] trait.

pub mod admin;
pub mod block_ops;
pub mod blocks;
pub mod colors;
pub mod compile;
pub mod engine;
pub mod error;
pub mod events_items;
pub mod expr;
pub mod gamemode;
pub mod health_food;
pub mod fx_names;
pub mod host;
pub mod items;
pub mod lexer;
pub mod list_literals;
pub mod list_ops;
pub mod op_kick_ban;
pub mod pattern;
pub mod sounds;
pub mod text_join_split;
pub mod text_case_substring;
pub mod time_dates;
pub mod senders;
pub mod skbee_bound;
pub mod player_flags;
pub mod timespans;
pub mod value;
pub mod vars;
pub mod xp_level;
pub mod inventories;
pub mod world_effects;

pub use compile::{CommandDef, EventKind};
pub use events_items::{EvPos, EventExtra};
pub use engine::{CommandOutcome, Engine, EventCtx, FireOutcome, LoadSummary};
pub use error::ScriptError;
pub use host::{Host, LogLevel};
pub use player_flags::PlayerFlag;
pub use value::{PlayerRef, Recipient, Value};
pub use vars::Variables;

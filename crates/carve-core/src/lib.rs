//! Carve core: a small Skript-like language. No game dependencies; the
//! game is reached only through the [`Host`] trait.

pub mod admin;
pub mod blocks;
pub mod compile;
pub mod engine;
pub mod error;
pub mod expr;
pub mod host;
pub mod items;
pub mod lexer;
pub mod list_literals;
pub mod pattern;
pub mod value;
pub mod vars;

pub use compile::{CommandDef, EventKind};
pub use engine::{CommandOutcome, Engine, EventCtx, FireOutcome, LoadSummary};
pub use error::ScriptError;
pub use host::{Host, LogLevel};
pub use value::{PlayerRef, Recipient, Value};
pub use vars::Variables;

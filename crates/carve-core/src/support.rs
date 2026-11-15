//! Syntax that parses but the server can't carry out.
//!
//! A host lists what it can't do with [`crate::Engine::set_unsupported`].
//! Every load then warns at each line that uses one of those, like Skript
//! warns at load time, so a line never silently does nothing.

use std::sync::Arc;

use crate::compile::{Code, Cond, Effect, Instr};
use crate::error::ScriptError;
use crate::player_flags::PlayerFlag;

/// Something a script can ask for that a host may not support.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Feature {
    /// `close player's inventory`.
    CloseInventory,
    /// A player state condition (`player is blocking`, ...).
    PlayerFlag(PlayerFlag),
}

impl Feature {
    fn what(self) -> String {
        match self {
            Feature::CloseInventory => "'close inventory'".to_string(),
            Feature::PlayerFlag(f) if f.name().starts_with("can ") => format!("'player {}'", f.name()),
            Feature::PlayerFlag(f) => format!("'player is {}'", f.name()),
        }
    }
}

fn cond_features(c: &Cond, out: &mut Vec<Feature>) {
    match c {
        Cond::PlayerFlag { flag, .. } => out.push(Feature::PlayerFlag(*flag)),
        Cond::All(v) | Cond::Any(v) => v.iter().for_each(|c| cond_features(c, out)),
        _ => {}
    }
}

fn instr_features(i: &Instr, out: &mut Vec<Feature>) {
    match i {
        Instr::Eff(Effect::Inv(e)) if matches!(**e, crate::inventories::InvEff::Close(_)) => {
            out.push(Feature::CloseInventory);
        }
        Instr::JumpIfNot(c, _) | Instr::StopUnless(c) => cond_features(c, out),
        _ => {}
    }
}

/// One warning per line and feature that the host can't do.
pub(crate) fn scan(codes: &[Arc<Code>], unsupported: &[(Feature, String)]) -> Vec<ScriptError> {
    let mut out: Vec<ScriptError> = Vec::new();
    if unsupported.is_empty() {
        return out;
    }
    let mut found = Vec::new();
    for code in codes {
        for (n, instr) in code.instrs.iter().enumerate() {
            found.clear();
            instr_features(instr, &mut found);
            for f in &found {
                let Some((_, why)) = unsupported.iter().find(|(u, _)| u == f) else {
                    continue;
                };
                let line = code.lines.get(n).copied().unwrap_or(code.line);
                let msg = format!("{} does nothing on this server: {why}", f.what());
                if !out
                    .iter()
                    .any(|w| w.line == line && *w.file == *code.file && w.msg == msg)
                {
                    out.push(ScriptError::warning(&code.file, line, msg));
                }
            }
        }
    }
    out.sort_by(|a, b| (&a.file, a.line).cmp(&(&b.file, b.line)));
    out
}

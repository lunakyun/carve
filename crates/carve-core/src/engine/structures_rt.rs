//! Running the structures: `catch runtime errors:` and `variables:` defaults
//! (see `structures.rs` for the syntax side).

use super::{Engine, EventCtx, Exec, SILENT};
use crate::compile::EventKind;
use crate::host::Host;
use crate::value::{PlayerRef, Value};

/// Local variable that holds the message of the last caught error.
pub(crate) const ERROR_KEY: &str = "_runtime error";

/// One open `catch runtime errors:` section.
pub(super) struct Catch {
    /// Where the run goes after the section when an error is caught.
    end: usize,
    /// Call depth the section was opened at.
    pub(super) depth: usize,
    /// Loops that were running when the section opened.
    loops: usize,
}

#[inline(never)]
pub(super) fn catch_begin(ex: &mut Exec, end: usize) {
    ex.catches.push(Catch {
        end,
        depth: ex.callers.len(),
        loops: ex.loops.len(),
    });
    ex.locals.set(ERROR_KEY, Value::None);
}

impl Engine {
    /// An error happened. If an open section can take it, jump to its end.
    /// `base` is the call depth of the running value-call (0 for the trigger).
    #[inline(never)]
    pub(super) fn catch_error(&mut self, ex: &mut Exec, e: &str, base: usize) -> bool {
        let msg = if e == SILENT {
            std::mem::take(&mut self.caught_stash)
        } else {
            e.to_string()
        };
        let Some(c) = ex.catches.last() else {
            return false;
        };
        if c.depth < base {
            // The section is in a caller of the running value-call: pass it up.
            self.caught_stash = msg;
            return false;
        }
        let (end, depth, loops) = (c.end, c.depth, c.loops);
        while ex.callers.len() > depth {
            ex.pop_call();
        }
        ex.loops.truncate(loops);
        ex.catches.pop();
        ex.pc = end;
        let msg = if msg.is_empty() {
            "unknown error".to_string()
        } else {
            msg
        };
        ex.locals.set(ERROR_KEY, Value::Text(msg));
        true
    }

    /// Before an event: apply pending `variables:` defaults, and the
    /// per-player ones to the player who joins.
    pub(super) fn defaults_for(&mut self, kind: EventKind, ctx: &EventCtx, host: &mut dyn Host) {
        if self.pending_defaults {
            self.apply_defaults(host);
        }
        if kind == EventKind::Join
            && let Some(p) = &ctx.player
        {
            self.player_defaults(p, host);
        }
    }

    /// Set every `variables:` default that has no value yet. Runs by itself
    /// before the first event after a load or reload; hosts may also call it.
    pub fn apply_defaults(&mut self, host: &mut dyn Host) {
        self.pending_defaults = false;
        let table = self.table.clone();
        let mut any_player = false;
        for (per_player, trig) in &table.var_defaults {
            if *per_player {
                any_player = true;
            } else {
                let ex = Exec::new(trig, EventCtx::console());
                self.start(ex, host);
            }
        }
        if any_player {
            for p in host.online_players() {
                self.player_defaults(&p, host);
            }
        }
    }

    fn player_defaults(&mut self, p: &PlayerRef, host: &mut dyn Host) {
        let table = self.table.clone();
        for (per_player, trig) in &table.var_defaults {
            if *per_player {
                let ex = Exec::new(trig, EventCtx::player(p.clone()));
                self.start(ex, host);
            }
        }
    }
}

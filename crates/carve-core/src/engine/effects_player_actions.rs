//! Runs the `effects-2` player effects (see `effects_player_actions.rs`).

use super::{Engine, Exec};
use crate::effects_player_actions::{self as actions, ActionEffect, ActionOp};
use crate::expr::Expr;
use crate::host::Host;
use crate::value::Value;

impl Engine {
    pub(super) fn run_player_actions(
        &mut self,
        e: &ActionEffect,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        let arg = |i: usize| e.args.get(i).and_then(Option::as_ref);
        let player = |this: &mut Engine, ex: &mut Exec, host: &mut dyn Host, i: usize| match arg(i) {
            Some(x) => this.player(x, ex, host),
            None => Ok(None),
        };
        match e.op {
            ActionOp::Suppress => {}
            ActionOp::Log => {
                let text = self.text(arg(0).ok_or("log needs text")?, ex, host)?;
                let file = match arg(1) {
                    Some(f) => actions::clean_log_name(&self.text(f, ex, host)?)?,
                    None => actions::DEFAULT_LOG_FILE.to_string(),
                };
                host.log_to_file(&file, &text);
            }
            ActionOp::Fly(on) => {
                if let Some(p) = player(self, ex, host, 0)? {
                    host.set_flight(&p, on);
                }
            }
            ActionOp::Invulnerable(on) => {
                if let Some(p) = player(self, ex, host, 0)? {
                    host.set_invulnerable(&p, on);
                }
            }
            ActionOp::Invisible(on) => {
                if let Some(p) = player(self, ex, host, 0)? {
                    host.set_invisible(&p, on);
                }
            }
            ActionOp::Ignite => {
                let ticks = match arg(1) {
                    Some(t) => self.span_ticks(t, ex, host)?,
                    None => i64::from(actions::DEFAULT_FIRE_TICKS),
                };
                if let Some(p) = player(self, ex, host, 0)? {
                    host.set_fire(&p, ticks.clamp(0, i64::from(i32::MAX)) as i32);
                }
            }
            ActionOp::Extinguish => {
                if let Some(p) = player(self, ex, host, 0)? {
                    host.set_fire(&p, 0);
                }
            }
            ActionOp::KnockAway | ActionOp::KnockToward => {
                let strength = match arg(2) {
                    Some(s) => Some(self.num(s, ex, host)?),
                    None => None,
                };
                let loc = self.eval(arg(1).ok_or("knock needs a location")?, ex, host)?;
                let Some((lx, _, lz, _)) = self.as_location(&loc, ex, host) else {
                    return Err(format!("expected a location, got {} ({loc})", loc.type_name()));
                };
                if let Some(p) = player(self, ex, host, 0)?
                    && let Some((px, _, pz, _)) = host.player_location(&p)
                {
                    // The game wants the direction from the player toward the pusher.
                    let (dx, dz) = (lx - px, lz - pz);
                    let (dx, dz) = if e.op == ActionOp::KnockAway {
                        (dx, dz)
                    } else {
                        (-dx, -dz)
                    };
                    host.apply_knockback(&p, actions::clean_strength(strength), dx, dz);
                }
            }
            ActionOp::LookAt => {
                let loc = self.eval(arg(1).ok_or("look at needs a location")?, ex, host)?;
                let Some((x, y, z, _)) = self.as_location(&loc, ex, host) else {
                    return Err(format!("expected a location, got {} ({loc})", loc.type_name()));
                };
                if let Some(p) = player(self, ex, host, 0)?
                    && let Some((px, py, pz, _)) = host.player_location(&p)
                {
                    // Eye height is about 1.62 above the stored position.
                    let (yaw, pitch) = actions::look_angles((px, py + 1.62, pz), (x, y, z));
                    host.set_rotation(&p, yaw, pitch);
                }
            }
            ActionOp::Say => {
                let text = self.text(arg(1).ok_or("say needs text")?, ex, host)?;
                if let Some(p) = player(self, ex, host, 0)? {
                    host.say_as(&p, &text);
                }
            }
            ActionOp::OpenBook(off) => {
                if let Some(p) = player(self, ex, host, 0)? {
                    host.open_book(&p, off);
                }
            }
        }
        Ok(())
    }

    fn span_ticks(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<i64, String> {
        match self.eval(e, ex, host)? {
            Value::Timespan { millis } => Ok(millis.max(0) / 50),
            v => Err(format!(
                "expected a timespan like '5 seconds', got {} ({v})",
                v.type_name()
            )),
        }
    }
}

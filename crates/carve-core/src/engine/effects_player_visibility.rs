//! Running the effects from `effects_player_visibility.rs`.

use super::{Engine, Exec};
use crate::effects_player_visibility::{W2Effect, W2Op, equip_slot, split_host};
use crate::host::Host;
use crate::value::Value;

impl Engine {
    #[inline(never)]
    pub(super) fn run_w2(&mut self, w: &W2Effect, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        if w.op == W2Op::Copy {
            return self.run_copy(w, ex, host);
        }
        let (Some(a), Some(b)) = (&w.a, w.b.as_ref()) else {
            // `force ... to respawn` has only the player.
            if let (W2Op::Respawn, Some(a)) = (w.op, &w.a)
                && let Some(p) = self.player(a, ex, host)?
            {
                host.respawn(&p);
            }
            return Ok(());
        };
        match w.op {
            W2Op::Connect => {
                let target = self.text(b, ex, host)?;
                let port = match &w.c {
                    Some(c) => Some(self.num(c, ex, host)?),
                    None => None,
                };
                let Some((h, port)) = split_host(&target, port) else {
                    return Err(format!("'{target}' is not a valid server address"));
                };
                if let Some(p) = self.player(a, ex, host)? {
                    host.transfer(&p, &h, port);
                }
            }
            W2Op::Hide | W2Op::Reveal => {
                // `hide %target% from %viewer%`: the first slot is the one hidden.
                let target = self.player(a, ex, host)?;
                let viewer = self.player(b, ex, host)?;
                if let (Some(t), Some(v)) = (target, viewer) {
                    if w.op == W2Op::Hide {
                        host.hide_player(&v, &t);
                    } else {
                        host.show_player(&v, &t);
                    }
                }
            }
            W2Op::Equip => {
                let item = self.eval(b, ex, host)?;
                let Value::Item { id, .. } = item else {
                    return Err(format!("expected an item, got {} ({item})", item.type_name()));
                };
                if let Some(p) = self.player(a, ex, host)? {
                    host.equip(&p, equip_slot(&id), &id);
                }
            }
            W2Op::Respawn => {
                if let Some(p) = self.player(a, ex, host)? {
                    host.respawn(&p);
                }
            }
            W2Op::Copy => {}
        }
        Ok(())
    }

    /// `copy {a::*} into {b::*}`: a list copies key by key, a single value copies as is.
    fn run_copy(&mut self, w: &W2Effect, ex: &mut Exec, host: &mut dyn Host) -> Result<(), String> {
        let (Some(from), Some(to)) = (&w.from, &w.to) else {
            return Ok(());
        };
        if from.list != to.list {
            return Err("copy needs a list on both sides or a single value on both sides".into());
        }
        if !from.list {
            let fk = self.key(from, ex, host)?.into_owned();
            let val = if from.local {
                ex.locals.get(&fk).cloned()
            } else {
                self.vars.get_ref(&fk).cloned()
            };
            let tk = self.key(to, ex, host)?.into_owned();
            match val {
                Some(v) if v != Value::None => self.store(to.local, &tk, v, ex, host),
                _ => {
                    if to.local {
                        ex.locals.delete(&tk);
                    } else {
                        self.vars.delete(&tk);
                        self.mark_dirty(host);
                    }
                }
            }
            return Ok(());
        }
        // Read everything first so `copy {x::*} to {x::*}` is a no-op, not a wipe.
        let items = self.list_items(from, ex, host)?;
        let tk = self.key(to, ex, host)?.into_owned();
        let prefix = tk.strip_suffix('*').unwrap_or(&tk).to_string();
        if to.local {
            ex.locals.delete(&tk);
            for (k, v) in items {
                ex.locals.set(&format!("{prefix}{k}"), v);
            }
        } else {
            self.vars.delete(&tk);
            for (k, v) in items {
                self.vars.set_str(&format!("{prefix}{k}"), v);
            }
            self.mark_dirty(host);
        }
        Ok(())
    }
}

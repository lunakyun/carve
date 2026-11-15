//! Running the `gaps` syntax: inventory values, held item edits, regex
//! replace, quit message, push and velocity.

use super::{Engine, Exec, online};
use crate::expr::{Expr, VarRef};
use crate::gaps::{GapCond, GapEff, GapExpr, HeldOp, InvVal, next_id, regex_replace_text, slot_item};
use crate::host::{Host, LogLevel};
use crate::inventories::InvTarget;
use crate::value::{EntityRef, PlayerRef, Value};

const MAX_SPEED: f64 = 20.0;

fn flatten(v: Value, out: &mut Vec<Value>) {
    match v {
        Value::List(items) => items.into_iter().for_each(|x| flatten(x, out)),
        Value::None => {}
        other => out.push(other),
    }
}

fn clamp_speed(n: f64) -> f64 {
    if n.is_finite() {
        n.clamp(-MAX_SPEED, MAX_SPEED)
    } else {
        0.0
    }
}

impl Engine {
    fn slot_no(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<u32, String> {
        let n = self.num(e, ex, host)?;
        if !n.is_finite() || n < 0.0 || n >= 54.0 {
            return Err(format!("slot {n} is out of range, a chest has slots 0 to 53"));
        }
        Ok(n.floor() as u32)
    }

    /// The player whose open menu `event-inventory` is.
    fn viewer(&mut self, uuid: &str, ex: &mut Exec, host: &mut dyn Host) -> Option<PlayerRef> {
        let p = Value::Player(PlayerRef {
            uuid: uuid.to_string(),
            name: String::new(),
        });
        online(&p, &mut ex.seen, host)
    }

    /// `event-inventory`: the menu of the click, its contents read live.
    fn event_inv(&mut self, clicked: bool, ex: &mut Exec) -> Value {
        let (Some(c), Some(p)) = (ex.ctx.inv.as_deref(), ex.ctx.player.as_ref()) else {
            return Value::None;
        };
        let Some((id, title, rows)) = c.menu.clone() else {
            return Value::None;
        };
        if clicked && !(0..i32::from(rows) * 9).contains(&c.slot) {
            return Value::None;
        }
        let mut inv = InvVal::new(id, rows, title);
        inv.slots.clear();
        inv.viewer = Some(p.uuid.clone());
        Value::Inventory(Box::new(inv))
    }

    /// The slots of an inventory value; a live one is read from the menu.
    fn inv_slots(&mut self, inv: &InvVal, ex: &mut Exec, host: &mut dyn Host) -> Vec<Value> {
        let Some(u) = &inv.viewer else {
            return inv.slots.clone();
        };
        let Some(p) = self.viewer(u, ex, host) else {
            return Vec::new();
        };
        (0..inv.size() as i64)
            .map(|s| {
                host.inv_get(&p, true, s)
                    .map_or(Value::None, |(id, count)| Value::Item {
                        id,
                        count,
                        meta: None,
                    })
            })
            .collect()
    }

    fn as_inv(
        &mut self,
        e: &Expr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Option<Box<InvVal>>, String> {
        match self.eval(e, ex, host)? {
            Value::Inventory(i) => Ok(Some(i)),
            Value::None => Ok(None),
            other => Err(format!(
                "expected an inventory, got {} ({other})",
                other.type_name()
            )),
        }
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn gap_expr(
        &mut self,
        g: &GapExpr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        Ok(match g {
            GapExpr::NewChest { rows, title } => {
                let rows = match rows {
                    Some(r) => self.num(r, ex, host)?,
                    None => 3.0,
                };
                if !rows.is_finite() || !(1.0..=6.0).contains(&rows.floor()) {
                    return Err("a chest has 1 to 6 rows".into());
                }
                let title = match title {
                    Some(t) => self.text(t, ex, host)?,
                    None => "Chest".to_string(),
                };
                let id = next_id(host.wall_clock_millis());
                Value::Inventory(Box::new(InvVal::new(id, rows.floor() as u8, title)))
            }
            GapExpr::EventInv(clicked) => self.event_inv(*clicked, ex),
            GapExpr::Slot { slot, inv } => {
                let s = self.slot_no(slot, ex, host)?;
                let Some(inv) = self.as_inv(inv, ex, host)? else {
                    return Ok(Value::None);
                };
                if let Some(u) = &inv.viewer {
                    return Ok(match self.viewer(u, ex, host) {
                        Some(p) => host
                            .inv_get(&p, true, i64::from(s))
                            .map_or(Value::None, |(id, count)| Value::Item {
                                id,
                                count,
                                meta: None,
                            }),
                        None => Value::None,
                    });
                }
                inv.slots.get(s as usize).cloned().unwrap_or(Value::None)
            }
            GapExpr::Items(inv) => match self.as_inv(inv, ex, host)? {
                Some(i) => Value::List(
                    self.inv_slots(&i, ex, host)
                        .into_iter()
                        .filter(|v| *v != Value::None)
                        .collect(),
                ),
                None => Value::List(Vec::new()),
            },
            GapExpr::PlayerItems(InvTarget { who, current }) => match self.player(who, ex, host)? {
                Some(p) if *current => Value::List(
                    (0..54)
                        .filter_map(|s| host.inv_get(&p, true, s))
                        .map(|(id, count)| Value::Item {
                            id,
                            count,
                            meta: None,
                        })
                        .collect(),
                ),
                Some(p) => Value::List(host.inv_items(&p)),
                None => Value::List(Vec::new()),
            },
            GapExpr::Regex { text, pat, with, all } => {
                let t = self.text(text, ex, host)?;
                let p = self.text(pat, ex, host)?;
                let w = self.text(with, ex, host)?;
                Value::Text(regex_replace_text(&t, &p, &w, *all)?)
            }
        })
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn gap_cond(
        &mut self,
        c: &GapCond,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<bool, String> {
        let hay = self.eval(&c.inv, ex, host)?;
        let want = self.eval(&c.item, ex, host)?;
        let hit = match (&hay, &want) {
            (Value::Inventory(inv), Value::Item { id, count, .. }) => {
                let have: u32 = self
                    .inv_slots(inv, ex, host)
                    .iter()
                    .filter_map(|v| match v {
                        Value::Item { id: i, count: n, .. } if i == id => Some(*n),
                        _ => None,
                    })
                    .sum();
                have >= (*count).max(1)
            }
            (Value::Inventory(_), _) => false,
            // Not an inventory: the plain text `contains`.
            _ => hay
                .to_string()
                .to_lowercase()
                .contains(&want.to_string().to_lowercase()),
        };
        Ok(hit != c.negate)
    }

    /// Change the inventory value in a variable, or the live menu of
    /// `event-inventory`.
    fn edit_inv(
        &mut self,
        target: &Expr,
        ex: &mut Exec,
        host: &mut dyn Host,
        f: impl FnOnce(&mut InvVal) -> Vec<(u32, Value)>,
    ) -> Result<(), String> {
        if let Expr::Var(v) = target {
            let k = self.key(v, ex, host)?.into_owned();
            let cur = if v.local {
                ex.locals.get(&k).cloned()
            } else {
                self.vars.get_ref(&k).cloned()
            };
            let Some(Value::Inventory(mut inv)) = cur else {
                return Err(match cur {
                    None | Some(Value::None) => "the variable holds no inventory".to_string(),
                    Some(o) => format!("expected an inventory, got {} ({o})", o.type_name()),
                });
            };
            let changed = f(&mut inv);
            let (id, live) = (inv.id, inv.viewer.clone());
            self.store(v.local, &k, Value::Inventory(inv), ex, host);
            return self.push_changes(id, live, changed, ex, host);
        }
        let Some(mut inv) = self.as_inv(target, ex, host)? else {
            return Ok(());
        };
        let changed = f(&mut inv);
        self.push_changes(inv.id, inv.viewer.clone(), changed, ex, host)
    }

    fn push_changes(
        &mut self,
        id: u64,
        live: Option<String>,
        changed: Vec<(u32, Value)>,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        match live {
            Some(u) => {
                if let Some(p) = self.viewer(&u, ex, host) {
                    for (s, item) in &changed {
                        host.menu_set(&p, *s, item);
                    }
                }
            }
            None if id != 0 => {
                for (s, item) in &changed {
                    host.menu_update(id, *s, item);
                }
            }
            None => {}
        }
        Ok(())
    }

    fn held_texts(
        &mut self,
        list: &[Expr],
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Vec<String>, String> {
        let mut vals = Vec::new();
        for e in list {
            match e {
                Expr::Var(v) if v.list => {
                    vals.extend(self.list_items(v, ex, host)?.into_iter().map(|(_, x)| x))
                }
                e => flatten(self.eval(e, ex, host)?, &mut vals),
            }
        }
        Ok(vals.iter().map(Value::to_string).collect())
    }

    fn held_edit(
        &mut self,
        who: &Expr,
        op: &HeldOp,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        // Evaluate first, so an error leaves the item alone.
        let mut line_no = 1usize;
        let texts = match op {
            HeldOp::Name(Some(e)) => vec![self.text(e, ex, host)?],
            HeldOp::Lore(l) | HeldOp::AddLore(l) | HeldOp::Enchant(l) => self.held_texts(l, ex, host)?,
            HeldOp::LoreLine(n, t) => {
                let n = self.num(n, ex, host)?;
                if !n.is_finite() || !(1.0..=64.0).contains(&n) {
                    return Err(format!("lore line {n} is out of range, use 1 to 64"));
                }
                line_no = n as usize;
                vec![self.text(t, ex, host)?]
            }
            HeldOp::Name(None) | HeldOp::Disenchant | HeldOp::Delete => Vec::new(),
        };
        let Some(p) = self.player(who, ex, host)? else {
            return Ok(());
        };
        if matches!(op, HeldOp::Delete) {
            host.clear_held_item(&p);
            return Ok(());
        }
        match host.held_item(&p) {
            Some((id, _)) if id != "air" => {}
            _ => return Ok(()),
        }
        let mut m = host.held_item_meta(&p).unwrap_or_default();
        match op {
            HeldOp::Name(None) => m.name = None,
            HeldOp::Name(Some(_)) => m.name = texts.into_iter().next(),
            HeldOp::Lore(_) => m.lore = texts,
            HeldOp::AddLore(_) => m.lore.extend(texts),
            HeldOp::LoreLine(..) => {
                while m.lore.len() < line_no {
                    m.lore.push(String::new());
                }
                if let (Some(slot), Some(t)) = (m.lore.get_mut(line_no - 1), texts.into_iter().next()) {
                    *slot = t;
                }
            }
            HeldOp::Delete => {}
            HeldOp::Enchant(_) => {
                for t in &texts {
                    let Some((id, level)) = crate::item_meta::split_spec(t) else {
                        return Err(format!("'{t}' is not an enchantment"));
                    };
                    m.enchant(&id, level.max(1));
                }
            }
            HeldOp::Disenchant => m.enchants.clear(),
        }
        if !host.set_held_item_meta(&p, &m) {
            host.log(LogLevel::Warn, "this server can't change the held item");
        }
        Ok(())
    }

    /// A handle for an entity, or an online player.
    fn gap_handle(&mut self, v: &Value, ex: &mut Exec, host: &mut dyn Host) -> Option<EntityRef> {
        match v {
            Value::Entity(r) => Some(r.clone()),
            Value::Player(_) | Value::Text(_) => online(v, &mut ex.seen, host).map(|p| EntityRef {
                uuid: p.uuid,
                kind: "player".into(),
            }),
            _ => None,
        }
    }

    fn gap_pos(&mut self, v: &Value, ex: &mut Exec, host: &mut dyn Host) -> Option<(f64, f64, f64)> {
        match v {
            Value::Location { x, y, z, .. } => Some((*x, *y, *z)),
            Value::Entity(r) => host.entity_location(r).map(|(x, y, z, _)| (x, y, z)),
            Value::Player(_) | Value::Text(_) => {
                let p = online(v, &mut ex.seen, host)?;
                host.player_location(&p).map(|(x, y, z, _)| (x, y, z))
            }
            _ => None,
        }
    }

    fn targets(&mut self, e: &Expr, ex: &mut Exec, host: &mut dyn Host) -> Result<Vec<Value>, String> {
        let mut out = Vec::new();
        flatten(self.eval(e, ex, host)?, &mut out);
        Ok(out)
    }

    fn speed(&mut self, s: &Option<Expr>, ex: &mut Exec, host: &mut dyn Host) -> Result<f64, String> {
        Ok(match s {
            Some(e) => clamp_speed(self.num(e, ex, host)?),
            None => 1.0,
        })
    }

    fn add_velocity(&mut self, r: &EntityRef, d: (f64, f64, f64), host: &mut dyn Host) {
        let cur = host.entity_velocity(r).unwrap_or((0.0, 0.0, 0.0));
        host.set_entity_velocity(r, (cur.0 + d.0, cur.1 + d.1, cur.2 + d.2));
    }

    #[cold]
    #[inline(never)]
    pub(crate) fn gap_effect(
        &mut self,
        g: &GapEff,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        match g {
            GapEff::SetSlot { slot, inv, item } => {
                let s = self.slot_no(slot, ex, host)?;
                let item = slot_item(self.eval(item, ex, host)?)?;
                let mut bad = false;
                self.edit_inv(inv, ex, host, |i| {
                    if i.viewer.is_none() {
                        match i.slots.get_mut(s as usize) {
                            Some(slot) => *slot = item.clone(),
                            None => {
                                bad = true;
                                return Vec::new();
                            }
                        }
                    } else if s as usize >= i.size() {
                        bad = true;
                        return Vec::new();
                    }
                    vec![(s, item)]
                })?;
                if bad {
                    return Err(format!("slot {s} is outside this inventory"));
                }
            }
            GapEff::SetSlots { slots, inv, item } => {
                let mut nums: Vec<u32> = Vec::new();
                for e in slots {
                    let v = self.eval(e, ex, host)?;
                    let items = match v {
                        Value::List(l) => l,
                        other => vec![other],
                    };
                    for x in items {
                        let n = super::to_num_opt(&x).ok_or_else(|| format!("slot {x} is not a number"))?;
                        if !n.is_finite() || !(0.0..54.0).contains(&n) {
                            return Err(format!("slot {n} is out of range, a chest has slots 0 to 53"));
                        }
                        nums.push(n.floor() as u32);
                    }
                }
                let item = slot_item(self.eval(item, ex, host)?)?;
                let mut bad = None;
                self.edit_inv(inv, ex, host, |i| {
                    let mut changes = Vec::new();
                    for &s in &nums {
                        if i.viewer.is_none() {
                            match i.slots.get_mut(s as usize) {
                                Some(slot) => *slot = item.clone(),
                                None => {
                                    bad = Some(s);
                                    return Vec::new();
                                }
                            }
                        } else if s as usize >= i.size() {
                            bad = Some(s);
                            return Vec::new();
                        }
                        changes.push((s, item.clone()));
                    }
                    changes
                })?;
                if let Some(s) = bad {
                    return Err(format!("slot {s} is outside this inventory"));
                }
            }
            GapEff::NoOp => {}
            GapEff::Open { inv, to } => {
                let inv = self.as_inv(inv, ex, host)?;
                if let (Some(inv), Some(p)) = (inv, self.player(to, ex, host)?) {
                    if inv.viewer.is_some() {
                        return Err(
                            "an open menu can't be opened again, build it with 'chest inventory'".into(),
                        );
                    }
                    host.open_inventory(&p, &inv);
                }
            }
            GapEff::Clear(target) => {
                if let Expr::Var(v) = target {
                    let k = self.key(v, ex, host)?.into_owned();
                    let is_inv = if v.local {
                        matches!(ex.locals.get(&k), Some(Value::Inventory(_)))
                    } else {
                        matches!(self.vars.get_ref(&k), Some(Value::Inventory(_)))
                    };
                    if !is_inv {
                        self.delete_var(v, &k, ex, host);
                        return Ok(());
                    }
                }
                self.edit_inv(target, ex, host, |i| {
                    let n = i.size();
                    if i.viewer.is_none() {
                        i.slots.iter_mut().for_each(|s| *s = Value::None);
                    }
                    (0..n as u32).map(|s| (s, Value::None)).collect()
                })?;
            }
            GapEff::Held { who, op } => self.held_edit(who, op, ex, host)?,
            GapEff::QuitMessage(t) => {
                let t = self.text(t, ex, host)?;
                host.set_quit_message(&t);
                ex.ctx.message = Some(t);
            }
            GapEff::Regex { pat, var, with, all } => {
                let p = self.text(pat, ex, host)?;
                let w = self.text(with, ex, host)?;
                let k = self.key(var, ex, host)?.into_owned();
                let cur = if var.local {
                    ex.locals.get(&k).cloned()
                } else {
                    self.vars.get_ref(&k).cloned()
                };
                let Some(Value::Text(t)) = cur else {
                    return Ok(());
                };
                let out = regex_replace_text(&t, &p, &w, *all)?;
                self.store(var.local, &k, Value::Text(out), ex, host);
            }
            GapEff::SetVelocity { who, to } => {
                let Value::Vector { x, y, z } = self.eval(to, ex, host)? else {
                    return Err("the velocity must be a vector, like vector(0, 1, 0)".into());
                };
                let v = (clamp_speed(x), clamp_speed(y), clamp_speed(z));
                for t in self.targets(who, ex, host)? {
                    if let Some(r) = self.gap_handle(&t, ex, host) {
                        host.set_entity_velocity(&r, v);
                    }
                }
            }
            GapEff::PushFacing { who, back, speed } => {
                let s = self.speed(speed, ex, host)? * if *back { -1.0 } else { 1.0 };
                for t in self.targets(who, ex, host)? {
                    let Some(r) = self.gap_handle(&t, ex, host) else {
                        continue;
                    };
                    let Some((yaw, pitch)) = host.entity_rotation(&r) else {
                        continue;
                    };
                    let (yaw, pitch) = (yaw.to_radians(), pitch.to_radians());
                    let d = (-yaw.sin() * pitch.cos(), -pitch.sin(), yaw.cos() * pitch.cos());
                    self.add_velocity(&r, (d.0 * s, d.1 * s, d.2 * s), host);
                }
            }
            GapEff::PushFrom {
                who,
                from,
                toward,
                speed,
            } => {
                let s = self.speed(speed, ex, host)?;
                let from = self.eval(from, ex, host)?;
                let Some(c) = self.gap_pos(&from, ex, host) else {
                    return Ok(());
                };
                for t in self.targets(who, ex, host)? {
                    let (Some(r), Some(p)) = (self.gap_handle(&t, ex, host), self.gap_pos(&t, ex, host))
                    else {
                        continue;
                    };
                    let d = (p.0 - c.0, p.1 - c.1, p.2 - c.2);
                    let len = (d.0 * d.0 + d.1 * d.1 + d.2 * d.2).sqrt();
                    if len < 1e-9 {
                        continue;
                    }
                    let k = if *toward { -s / len } else { s / len };
                    self.add_velocity(&r, (d.0 * k, d.1 * k, d.2 * k), host);
                }
            }
        }
        Ok(())
    }

    fn delete_var(&mut self, v: &VarRef, k: &str, ex: &mut Exec, host: &mut dyn Host) {
        if v.local {
            ex.locals.delete(k);
        } else {
            self.vars.delete(k);
            self.mark_dirty(host);
        }
    }
}

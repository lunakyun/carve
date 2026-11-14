//! `on damage` and `on death`: any living victim, any attacking entity, and
//! the `damage` / `death message` scripts set, written back to Pumpkin.
//!
//! Pumpkin fires a plain damage event for every hit, then, only when the hit
//! has a source or cause entity (`living.rs` `damage_with_context`), a
//! by-entity event. Scripts run once per hit: from the by-entity event when
//! one follows (so `attacker` is known), else from the plain one.

use carve_core::combat::{EntSlot, SLOT_DAMAGE, SLOT_DEATH_MESSAGE};
use carve_core::{EventCtx, EventKind, FireOutcome, Value};
use pumpkin_plugin_api::events::{
    EntityDamageByEntityEvent, EntityDamageEvent, EventData, EventHandler, PlayerDeathEvent,
};
use pumpkin_plugin_api::{DamageType, Server};

use super::{EventAccess, State, TickHost, WasmHost, text, with_state};

/// How long a hit counts as the cause of a death (5 seconds, like Skript).
const KILL_CREDIT_TICKS: u64 = 100;

/// Damage types that come with an attacking entity, so a by-entity event
/// usually follows.
fn has_attacker(t: DamageType) -> bool {
    matches!(
        t,
        DamageType::Arrow
            | DamageType::Fireworks
            | DamageType::MaceSmash
            | DamageType::MobAttack
            | DamageType::MobAttackNoAggro
            | DamageType::MobProjectile
            | DamageType::PlayerAttack
            | DamageType::PlayerExplosion
            | DamageType::SonicBoom
            | DamageType::Spear
            | DamageType::Spit
            | DamageType::Sting
            | DamageType::Thrown
            | DamageType::WindCharge
    )
}

// Not in the list: fireballs, thorns, tridents and wither skulls. Pumpkin
// deals those with `damage()`, which never passes an entity, so no by-entity
// event follows and scripts run from the plain event.

/// A hit whose by-entity event is expected but may not come (`/damage`, the
/// ender dragon, snowballs). Run late, when the next event shows it never came.
pub struct LateHit {
    victim: i32,
    damage: f32,
    cause: String,
    tick: u64,
}

fn out_num(o: &FireOutcome) -> Option<f32> {
    o.set.iter().rev().find_map(|(s, v)| match (*s, v) {
        (SLOT_DAMAGE, Value::Num(n)) => Some(*n as f32),
        _ => None,
    })
}

fn out_text(o: &FireOutcome) -> Option<&str> {
    o.set.iter().rev().find_map(|(s, v)| match (*s, v) {
        (SLOT_DEATH_MESSAGE, Value::Text(t)) => Some(t.as_str()),
        _ => None,
    })
}

impl State {
    /// The player with this entity id as `player`, else the entity.
    fn victim_ctx(&mut self, host: &mut WasmHost, id: i32) -> EventCtx {
        match self.player_by_entity(host, id) {
            Some(p) => EventCtx::player(p),
            None => EventCtx::entity_victim(EntSlot::Id(id)),
        }
    }

    fn set_attacker(&mut self, host: &mut WasmHost, ctx: &mut EventCtx, id: i32) {
        match self.player_by_entity(host, id) {
            Some(p) => ctx.attacker = Some(p),
            None => ctx.set_entity_attacker(EntSlot::Id(id)),
        }
    }

    fn fire_damage(
        &mut self,
        host: &mut WasmHost,
        victim: i32,
        damage: f32,
        attacker: Option<i32>,
        cause: String,
    ) -> FireOutcome {
        let mut ctx = self.victim_ctx(host, victim);
        ctx.damage = Some(f64::from(damage));
        ctx.damage_cause = Some(cause);
        if let Some(a) = attacker {
            self.set_attacker(host, &mut ctx, a);
        }
        let mut th = TickHost {
            inner: host,
            hs: &mut self.hs,
        };
        self.engine.fire(EventKind::Damage, ctx, &mut th)
    }

    /// Run a hit whose by-entity event never came. Too late to cancel it or
    /// change its damage.
    #[cold]
    fn run_late(&mut self, host: &mut WasmHost, l: LateHit) {
        if self.engine.table().has_event(EventKind::Damage) {
            self.fire_damage(host, l.victim, l.damage, None, l.cause);
        }
    }

    /// Run a waiting hit, unless `keep` says this event is its by-entity event.
    pub fn flush_late(&mut self, host: &mut WasmHost, keep: Option<i32>) {
        let Some(l) = self.late_hit.take() else { return };
        if keep == Some(l.victim) && l.tick == self.hs.tick {
            return;
        }
        self.run_late(host, l);
    }
}

pub struct OnDamage;

impl EventHandler<EntityDamageEvent> for OnDamage {
    fn handle(&self, server: Server, mut data: EventData<EntityDamageEvent>) -> EventData<EntityDamageEvent> {
        let out = with_state(&server, None, |st, host| {
            st.flush_late(host, None);
            st.track_hit(host, data.entity_id, data.damage, Some(data.damage_type), None);
            if !st.engine.table().has_event(EventKind::Damage) {
                return None;
            }
            let cause = carve_core::w2_expressions_2::snake_case(&format!("{:?}", data.damage_type));
            if has_attacker(data.damage_type) {
                st.damage_pending = Some((data.entity_id, false, st.hs.tick));
                st.late_hit = Some(LateHit {
                    victim: data.entity_id,
                    damage: data.damage,
                    cause,
                    tick: st.hs.tick,
                });
                return None;
            }
            st.damage_pending = Some((data.entity_id, true, st.hs.tick));
            Some(st.fire_damage(host, data.entity_id, data.damage, None, cause))
        });
        if let Some(Some(o)) = out {
            if o.cancelled {
                data.cancelled = true;
            }
            if let Some(d) = out_num(&o) {
                data.damage = d;
            }
        }
        data
    }
}

pub struct OnDamageByEntity;

impl EventHandler<EntityDamageByEntityEvent> for OnDamageByEntity {
    fn handle(
        &self,
        server: Server,
        mut data: EventData<EntityDamageByEntityEvent>,
    ) -> EventData<EntityDamageByEntityEvent> {
        let out = with_state(&server, None, |st, host| {
            st.flush_late(host, Some(data.entity_id));
            st.track_hit(host, data.entity_id, data.damage, None, Some(data.damager_id));
            if !st.engine.table().has_event(EventKind::Damage) {
                return None;
            }
            // Already handled by the plain damage event of this hit.
            if let Some((id, true, tick)) = st.damage_pending
                && id == data.entity_id
                && tick == st.hs.tick
            {
                return None;
            }
            st.damage_pending = None;
            let cause = data
                .cause
                .strip_prefix("minecraft:")
                .unwrap_or(&data.cause)
                .to_string();
            Some(st.fire_damage(host, data.entity_id, data.damage, Some(data.damager_id), cause))
        });
        if let Some(Some(o)) = out {
            if o.cancelled {
                data.cancelled = true;
            }
            if let Some(d) = out_num(&o) {
                data.damage = d;
            }
        }
        data
    }
}

pub struct OnDeath;

impl EventHandler<PlayerDeathEvent> for OnDeath {
    fn handle(&self, server: Server, mut data: EventData<PlayerDeathEvent>) -> EventData<PlayerDeathEvent> {
        let out = with_state(&server, None, |st, host| {
            // The killing hit may still wait for its by-entity event.
            st.flush_late(host, None);
            PlayerDeathEvent::observe(&data, &mut st.hs);
            let mut ctx = PlayerDeathEvent::ctx(&data);
            let last = ctx.player.as_ref().and_then(|p| st.hs.w4.hit_by.remove(&p.uuid));
            if !st.engine.table().has_event(EventKind::Death) {
                return None;
            }
            if let Some((id, tick)) = last
                && st.hs.tick.saturating_sub(tick) <= KILL_CREDIT_TICKS
            {
                st.set_attacker(host, &mut ctx, id);
            }
            let mut th = TickHost {
                inner: host,
                hs: &mut st.hs,
            };
            Some(st.engine.fire(EventKind::Death, ctx, &mut th))
        });
        if let Some(Some(o)) = out {
            if o.cancelled {
                data.cancelled = true;
            }
            if let Some(t) = out_text(&o) {
                data.death_message = text(t);
            }
        }
        data
    }
}

//! Host side of `carve_core::entity_block_properties`: scale, targets, the spectator
//! camera, text display fields, world border warnings, block states, boss
//! bar reads and inventory viewers.

use carve_core::block_ops::BlockPos;
use carve_core::entity_block_properties::{BlockState, Get, Set, TextField, Warn};
use carve_core::{EntityRef, Value};
use pumpkin_plugin_api::display::{TextAlignment, Vector3f};
use pumpkin_plugin_api::world::{BlockFlags, Entity};
use pumpkin_plugin_api::{Attribute, EntityDisplayExt, uuid};

use super::{WasmHost, WitPos, player_ref};

/// How far `target of player` looks for an entity.
const PLAYER_TARGET_RANGE: f64 = 100.0;

fn align_name(a: TextAlignment) -> &'static str {
    match a {
        TextAlignment::Left => "left",
        TextAlignment::Right => "right",
        TextAlignment::Center => "center",
    }
}

impl WasmHost<'_> {
    /// A script value for a live entity: a player value for players.
    fn prop_value(&mut self, e: &Entity) -> Value {
        let id = uuid::to_string(e.get_uuid());
        if let Some(p) = self.get_by_uuid(&id) {
            return Value::Player(player_ref(p));
        }
        Value::Entity(EntityRef {
            uuid: id,
            kind: crate::ents::entity_name(e.get_type()).to_string(),
        })
    }

    #[cold]
    pub(super) fn property_get_impl(&mut self, q: Get<'_>) -> Option<Value> {
        match q {
            Get::Scale(r) => {
                let e = self.find_entity(&r.uuid)?;
                if let Some(d) = e.as_display() {
                    let s = d.get_transformation().scale;
                    return Some(Value::Vector {
                        x: f64::from(s.x),
                        y: f64::from(s.y),
                        z: f64::from(s.z),
                    });
                }
                Some(Value::Num(e.as_living()?.get_attribute_value(Attribute::Scale)))
            }
            Get::Target(r) => {
                let e = self.find_entity(&r.uuid)?;
                let t = match e.as_mob() {
                    Some(m) => m.get_target()?,
                    None => e.get_target_entity(PLAYER_TARGET_RANGE)?,
                };
                Some(self.prop_value(&t))
            }
            Get::Camera(p) => {
                let pl = self.get(p)?;
                let cam = pl.get_camera_entity_id();
                if cam == pl.as_entity().get_id() {
                    return None;
                }
                let r = self.entity_by_id(i32::try_from(cam).ok()?)?;
                let e = self.find_entity(&r.uuid)?;
                Some(self.prop_value(&e))
            }
            Get::Text(r, f) => {
                let t = self.find_entity(&r.uuid)?.as_text_display()?;
                Some(match f {
                    TextField::Alignment => Value::Text(align_name(t.get_alignment()).into()),
                    TextField::LineWidth => Value::Num(f64::from(t.get_line_width())),
                    TextField::Opacity => Value::Num(f64::from(t.get_text_opacity() as u8)),
                })
            }
            Get::Warning(world, w) => {
                let b = self.server.get_world_by_name(world)?.get_world_border();
                Some(Value::Num(f64::from(match w {
                    Warn::Distance => b.get_warning_distance(),
                    Warn::Time => b.get_warning_time(),
                })))
            }
            Get::BarTitle(id) => crate::sb::bar_title(id).map(Value::Text),
            Get::BarStyle(id) => crate::sb::bar_style(id).map(|s| Value::Text(s.into())),
            Get::BarViewers(id) => {
                let uuids = crate::sb::bar_viewers(id)?;
                Some(Value::List(self.prop_players(&uuids)))
            }
            Get::MenuViewers(id) => {
                let uuids = crate::inv::viewers_of(id);
                Some(Value::List(self.prop_players(&uuids)))
            }
        }
    }

    fn prop_players(&mut self, uuids: &[String]) -> Vec<Value> {
        uuids
            .iter()
            .filter_map(|u| self.get_by_uuid(u).map(|p| Value::Player(player_ref(p))))
            .collect()
    }

    #[cold]
    pub(super) fn property_set_impl(&mut self, s: Set<'_>) -> bool {
        match s {
            Set::Scale(r, v) => {
                let Some(e) = self.find_entity(&r.uuid) else {
                    return false;
                };
                if let Some(d) = e.as_display() {
                    let (x, y, z) = match v {
                        Some(Value::Vector { x, y, z }) => (*x, *y, *z),
                        Some(Value::Num(n)) => (*n, *n, *n),
                        _ => (1.0, 1.0, 1.0),
                    };
                    let mut t = d.get_transformation();
                    t.scale = Vector3f {
                        x: x as f32,
                        y: y as f32,
                        z: z as f32,
                    };
                    d.set_transformation(t);
                    return true;
                }
                let Some(l) = e.as_living() else {
                    return false;
                };
                match v {
                    Some(Value::Num(n)) => l.set_attribute_base(Attribute::Scale, *n),
                    None => l.reset_attribute(Attribute::Scale),
                    _ => return false,
                }
                true
            }
            Set::Target(r, to) => {
                let Some(m) = self.find_entity(&r.uuid).and_then(|e| e.as_mob()) else {
                    return false;
                };
                let t = to.and_then(|t| self.find_entity(&t.uuid));
                m.set_target(t);
                true
            }
            Set::Camera(p, to) => {
                let t = to.and_then(|t| self.find_entity(&t.uuid));
                let Some(pl) = self.get(p) else {
                    return false;
                };
                match t {
                    Some(e) => pl.set_camera(Some(e)),
                    None => pl.reset_camera(),
                }
                true
            }
            Set::Text(r, f, v) => {
                let Some(t) = self.find_entity(&r.uuid).and_then(|e| e.as_text_display()) else {
                    return false;
                };
                match f {
                    TextField::Alignment => t.set_alignment(match v.to_string().as_str() {
                        "left" => TextAlignment::Left,
                        "right" => TextAlignment::Right,
                        _ => TextAlignment::Center,
                    }),
                    TextField::LineWidth => t.set_line_width(v.as_num().unwrap_or(200.0) as i32),
                    TextField::Opacity => t.set_text_opacity(v.as_num().unwrap_or(255.0) as u8 as i8),
                }
                true
            }
            Set::Warning(world, w, n) => {
                let Some(world) = self.server.get_world_by_name(world) else {
                    return false;
                };
                let b = world.get_world_border();
                match w {
                    Warn::Distance => b.set_warning_distance(n as i32),
                    Warn::Time => b.set_warning_time(n as i32),
                }
                true
            }
            Set::BlockState(at, state) => {
                let Some(w) = self.block_world(at) else {
                    return false;
                };
                let flags = BlockFlags::NOTIFY_NEIGHBORS | BlockFlags::NOTIFY_LISTENERS;
                w.set_block_state(wit_pos(at), state, flags);
                true
            }
        }
    }

    #[cold]
    pub(super) fn block_state_prop_impl(&mut self, at: &BlockPos) -> Option<BlockState> {
        let w = self.block_world(at)?;
        let b = w.get_block(wit_pos(at));
        let state = w.get_block_state_id(wit_pos(at));
        let first = b.state_ids.iter().copied().min().unwrap_or(b.default_state_id);
        let id = b.name.strip_prefix("minecraft:").unwrap_or(&b.name).to_string();
        Some((id, state, first))
    }
}

fn wit_pos(at: &BlockPos) -> WitPos {
    WitPos {
        x: at.x,
        y: at.y,
        z: at.z,
    }
}

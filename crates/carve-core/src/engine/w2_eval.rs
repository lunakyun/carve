//! Running the expressions of `w2_expressions_1`.

use super::{Engine, Exec, online};
use crate::host::Host;
use crate::value::Value;
use crate::w2_expressions_1::{self as w2, W2};

impl Engine {
    #[inline(never)]
    pub(super) fn eval_w2(&mut self, w: &W2, ex: &mut Exec, host: &mut dyn Host) -> Result<Value, String> {
        Ok(match w {
            W2::Eternity => Value::Timespan {
                millis: w2::ETERNITY_MS,
            },
            W2::Angle(_, e) => Value::Num(self.num(e, ex, host)?),
            W2::Altitude(e) => {
                let v = self.eval(e, ex, host)?;
                match self.as_location(&v, ex, host) {
                    Some((_, y, _, _)) => Value::Num(y),
                    None => Value::None,
                }
            }
            W2::Channel(ch, e) => {
                let t = self.text(e, ex, host)?;
                w2::channel_of(&t, *ch).map_or(Value::None, Value::Num)
            }
            W2::ItemCount(item, player) => {
                let item = self.eval(item, ex, host)?;
                let p = self.eval(player, ex, host)?;
                let id = match &item {
                    Value::Item { id, .. } => id.clone(),
                    Value::Text(t) => t
                        .trim()
                        .trim_start_matches("minecraft:")
                        .replace(' ', "_")
                        .to_ascii_lowercase(),
                    _ => return Ok(Value::None),
                };
                match online(&p, &mut ex.seen, host) {
                    Some(p) => Value::Num(f64::from(host.count_item(&p, &id))),
                    None => Value::None,
                }
            }
            W2::Biome(e) => {
                let v = self.eval(e, ex, host)?;
                let Some((x, y, z, world)) = self.as_location(&v, ex, host) else {
                    return Ok(Value::None);
                };
                let world = world.unwrap_or_default();
                host.biome_at(&world, x.floor() as i32, y.floor() as i32, z.floor() as i32)
                    .map_or(Value::None, Value::Text)
            }
            W2::Armor(slot, e) => {
                let v = self.eval(e, ex, host)?;
                match online(&v, &mut ex.seen, host).and_then(|p| host.armor_item(&p, *slot)) {
                    Some((id, count)) => Value::Item {
                        id,
                        count,
                        meta: None,
                    },
                    None => Value::None,
                }
            }
            W2::BlockEntity(field, e) => {
                let v = self.eval(e, ex, host)?;
                let Some((x, y, z, world)) = self.as_location(&v, ex, host) else {
                    return Ok(Value::None);
                };
                let world = world.unwrap_or_default();
                let bytes =
                    host.block_entity_nbt(&world, x.floor() as i32, y.floor() as i32, z.floor() as i32);
                match bytes.and_then(|b| w2::block_entity_value(*field, &b)) {
                    Some(n) if field.is_span() => Value::Timespan { millis: n as i64 },
                    Some(n) => Value::Num(n),
                    None => Value::None,
                }
            }
            W2::AllScripts => Value::List(self.sources.keys().map(|k| Value::Text(k.clone())).collect()),
        })
    }
}

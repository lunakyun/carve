//! Running list operations (see `list_ops.rs` for the syntax side).

use std::cmp::Ordering;

use super::{Engine, Exec, to_num_opt};
use crate::expr::VarRef;
use crate::host::Host;
use crate::list_ops::{Agg, INPUT_KEY, ListSrc};
use crate::value::Value;

/// Order for `sorted`: numbers first (by value), then text (ignoring case),
/// then anything else by its printed form.
fn compare(a: &Value, b: &Value) -> Ordering {
    fn rank(v: &Value) -> u8 {
        match v {
            Value::Num(_) => 0,
            Value::Text(_) => 1,
            _ => 2,
        }
    }
    match (a, b) {
        (Value::Num(x), Value::Num(y)) => x.total_cmp(y),
        (Value::Text(x), Value::Text(y)) => x.to_lowercase().cmp(&y.to_lowercase()),
        _ => rank(a)
            .cmp(&rank(b))
            .then_with(|| a.to_string().to_lowercase().cmp(&b.to_string().to_lowercase())),
    }
}

/// Times and dates sort by their number, everything else as `compare` does.
fn sort_key(v: Value) -> Value {
    match v {
        Value::Timespan { millis } | Value::Date(millis) => Value::Num(millis as f64),
        Value::Bool(b) => Value::Num(f64::from(u8::from(b))),
        other => other,
    }
}

impl Engine {
    pub(super) fn eval_list(
        &mut self,
        s: &ListSrc,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Vec<Value>, String> {
        Ok(match s {
            ListSrc::Var(v) => self
                .list_items(v, ex, host)?
                .into_iter()
                .map(|(_, x)| x)
                .collect(),
            ListSrc::One(e) => match self.eval(e, ex, host)? {
                Value::List(v) => v,
                Value::None => Vec::new(),
                v => vec![v],
            },
            ListSrc::Concat(parts) => {
                let mut out = Vec::new();
                for p in parts {
                    out.extend(self.eval_list(p, ex, host)?);
                }
                out
            }
            ListSrc::Sorted(i) => {
                let mut v = self.eval_list(i, ex, host)?;
                v.sort_by(compare);
                v
            }
            ListSrc::SortedBy(i, key) => self.sorted_by(i, key, ex, host)?,
            ListSrc::SortedAlpha(i) => {
                let mut v = self.eval_list(i, ex, host)?;
                v.sort_by_cached_key(|x| x.to_string().to_lowercase());
                v
            }
            ListSrc::Reversed(i) => {
                let mut v = self.eval_list(i, ex, host)?;
                v.reverse();
                v
            }
            ListSrc::Shuffled(i) => {
                let mut v = self.eval_list(i, ex, host)?;
                self.shuffle(&mut v, host);
                v
            }
            ListSrc::Random(n, i) => {
                let n = self.num(n, ex, host)?;
                let mut v = self.eval_list(i, ex, host)?;
                self.shuffle(&mut v, host);
                v.truncate(if n.is_finite() && n > 0.0 { n as usize } else { 0 });
                v
            }
            ListSrc::Recursive(v) => self.recursive_items(v, ex, host)?,
            ListSrc::Filter(i, c) => {
                let items = self.eval_list(i, ex, host)?;
                let saved = ex.locals.get(INPUT_KEY).cloned();
                let mut out = Vec::new();
                let mut failed = None;
                for it in items {
                    ex.locals.set(INPUT_KEY, it.clone());
                    match self.cond(c, ex, host) {
                        Ok(true) => out.push(it),
                        Ok(false) => {}
                        Err(e) => {
                            failed = Some(e);
                            break;
                        }
                    }
                }
                ex.locals.set(INPUT_KEY, saved.unwrap_or(Value::None));
                if let Some(e) = failed {
                    return Err(e);
                }
                out
            }
        })
    }

    /// `sorted {x::*} by <key>`: `input` is each item while the key runs.
    /// Items whose key is `<none>` go last, in their old order.
    fn sorted_by(
        &mut self,
        i: &ListSrc,
        key: &crate::expr::Expr,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Vec<Value>, String> {
        let items = self.eval_list(i, ex, host)?;
        let saved = ex.locals.get(INPUT_KEY).cloned();
        let mut keyed = Vec::with_capacity(items.len());
        let mut failed = None;
        for it in items {
            ex.locals.set(INPUT_KEY, it.clone());
            match self.eval(key, ex, host) {
                Ok(k) => keyed.push((sort_key(k), it)),
                Err(e) => {
                    failed = Some(e);
                    break;
                }
            }
        }
        ex.locals.set(INPUT_KEY, saved.unwrap_or(Value::None));
        if let Some(e) = failed {
            return Err(e);
        }
        keyed.sort_by(|(a, _), (b, _)| match (a, b) {
            (Value::None, Value::None) => Ordering::Equal,
            (Value::None, _) => Ordering::Greater,
            (_, Value::None) => Ordering::Less,
            _ => compare(a, b),
        });
        Ok(keyed.into_iter().map(|(_, v)| v).collect())
    }

    /// Fisher-Yates.
    fn shuffle(&mut self, v: &mut [Value], host: &mut dyn Host) {
        for i in (1..v.len()).rev() {
            let j = ((self.next_random(host) * (i + 1) as f64) as usize).min(i);
            v.swap(i, j);
        }
    }

    pub(super) fn eval_agg(
        &mut self,
        a: Agg,
        s: &ListSrc,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<Value, String> {
        let items = self.eval_list(s, ex, host)?;
        if a == Agg::Size {
            return Ok(Value::Num(items.len() as f64));
        }
        let nums: Vec<f64> = items.iter().filter_map(to_num_opt).collect();
        Ok(match a {
            Agg::Sum => Value::Num(nums.iter().sum()),
            _ if nums.is_empty() => Value::None,
            Agg::Avg => Value::Num(nums.iter().sum::<f64>() / nums.len() as f64),
            Agg::Min => Value::Num(nums.iter().copied().fold(f64::INFINITY, f64::min)),
            Agg::Max => Value::Num(nums.iter().copied().fold(f64::NEG_INFINITY, f64::max)),
            Agg::Size => Value::None,
        })
    }

    /// Replace the list `v` with `items`, numbered from 1.
    pub(super) fn write_list(
        &mut self,
        v: &VarRef,
        items: Vec<Value>,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        let key = self.key(v, ex, host)?.into_owned();
        let prefix = key.strip_suffix('*').unwrap_or(&key).to_string();
        if v.local {
            ex.locals.delete(&key);
            for (i, x) in items.into_iter().enumerate() {
                ex.locals.set(&format!("{prefix}{}", i + 1), x);
            }
        } else {
            self.vars.delete(&key);
            for (i, x) in items.into_iter().enumerate() {
                self.vars.set_str(&format!("{prefix}{}", i + 1), x);
            }
            self.mark_dirty(host);
        }
        Ok(())
    }

    pub(super) fn exec_set_list(
        &mut self,
        v: &VarRef,
        s: &ListSrc,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        let items = self.eval_list(s, ex, host)?;
        self.write_list(v, items, ex, host)
    }

    pub(super) fn exec_sort_list(
        &mut self,
        v: &VarRef,
        desc: bool,
        ex: &mut Exec,
        host: &mut dyn Host,
    ) -> Result<(), String> {
        let mut items: Vec<Value> = self
            .list_items(v, ex, host)?
            .into_iter()
            .map(|(_, x)| x)
            .collect();
        items.sort_by(compare);
        if desc {
            items.reverse();
        }
        self.write_list(v, items, ex, host)
    }
}

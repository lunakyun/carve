//! Text helpers for `join`, `split`, `replace` and `length of`.
//! Pure functions: the engine calls them with already evaluated values.

use crate::value::Value;

/// Join the printed items of a list with `sep`. `<none>` items are skipped.
pub fn join_values(items: &[(String, Value)], sep: &str) -> String {
    let mut out = String::new();
    let mut first = true;
    for (_, v) in items {
        if matches!(v, Value::None) {
            continue;
        }
        if !first {
            out.push_str(sep);
        }
        first = false;
        v.push_to(&mut out);
    }
    out
}

/// Split `text` at every `sep` (exact match). An empty separator splits
/// into single characters. Empty text gives an empty list.
pub fn split_text(text: &str, sep: &str) -> Vec<Value> {
    if text.is_empty() {
        return Vec::new();
    }
    if sep.is_empty() {
        return text.chars().map(|c| Value::Text(c.to_string())).collect();
    }
    text.split(sep).map(|s| Value::Text(s.to_string())).collect()
}

/// Replace `from` by `to` (exact, case sensitive): every match, or only the
/// first. An empty `from` changes nothing.
pub fn replace_text(text: &str, from: &str, to: &str, all: bool) -> Result<String, String> {
    if from.is_empty() {
        return Ok(text.to_string());
    }
    if all {
        // Work out the size first: `text.replace` would build it before we could say no.
        if to.len() > from.len() {
            let hits = text.matches(from).count();
            if text.len() + hits * (to.len() - from.len()) > crate::engine::MAX_TEXT_BYTES {
                return Err(format!(
                    "this replace would make a text longer than {} characters",
                    crate::engine::MAX_TEXT_BYTES
                ));
            }
        }
        Ok(text.replace(from, to))
    } else {
        Ok(text.replacen(from, to, 1))
    }
}

/// Number of characters (not bytes).
pub fn char_len(text: &str) -> usize {
    text.chars().count()
}

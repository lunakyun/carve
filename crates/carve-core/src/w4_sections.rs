//! Wave 4 sections: `filter {list::*} to match:`, `spawn <entity> at <loc>:`,
//! `suppress type hints:` and the refusal of `shoot ...:`.
//!
//! All of them turn into lines that the normal compiler already understands,
//! so they cost nothing at run time.

use crate::blocks::Line;
use crate::compile::Cx;

/// Replace the word `from` by `to` in code (not in text, but inside `%...%`
/// of a text, and not inside `{variables}`).
fn rewrite(text: &str, from: &str, to: &str) -> String {
    let chars: Vec<char> = text.chars().collect();
    let from_c: Vec<char> = from.chars().collect();
    let word = |c: char| c.is_alphanumeric() || c == '_' || c == '-';
    let (mut in_str, mut in_pct, mut braces) = (false, false, 0usize);
    let mut out = String::with_capacity(text.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '"' if !in_pct => in_str = !in_str,
            '%' if in_str => in_pct = !in_pct,
            '{' if !in_str || in_pct => braces += 1,
            '}' if (!in_str || in_pct) && braces > 0 => braces -= 1,
            _ => {}
        }
        let live = (!in_str || in_pct) && braces == 0;
        if live
            && chars.get(i..i + from_c.len()).is_some_and(|s| {
                s.iter().zip(&from_c).all(|(a, b)| a.eq_ignore_ascii_case(b))
            })
            && (i == 0 || !word(chars[i - 1]))
            && chars.get(i + from_c.len()).is_none_or(|n| !word(*n))
        {
            out.push_str(to);
            i += from_c.len();
            continue;
        }
        out.push(c);
        i += 1;
    }
    out
}

fn rewrite_lines(lines: &[Line], pairs: &[(&str, &str)]) -> Vec<Line> {
    lines
        .iter()
        .map(|l| {
            let mut text = l.text.clone();
            for (a, b) in pairs {
                text = rewrite(&text, a, b);
            }
            Line {
                num: l.num,
                text,
                children: rewrite_lines(&l.children, pairs),
            }
        })
        .collect()
}

fn line(num: usize, text: String, children: Vec<Line>) -> Line {
    Line { num, text, children }
}

/// Returns true when the line was one of these sections.
pub(crate) fn section(cx: &mut Cx, l: &Line, lower: &str) -> bool {
    let head = lower.trim_end_matches(':').trim();
    if matches!(
        head,
        "suppress type hints"
            | "suppress local variable type hints"
            | "suppress the type hints"
            | "suppress the local variable type hints"
    ) {
        cx.block(&l.children);
        return true;
    }
    if head.starts_with("spawn ") {
        let spawn = l.head();
        cx.effect(l.num, spawn);
        let body = rewrite_lines(&l.children, &[("event-entity", "last spawned entity")]);
        cx.block(&body);
        return true;
    }
    if head.starts_with("shoot ") {
        cx.err(
            l.num,
            "the 'shoot' section is not available: Pumpkin has no way for plugins to launch projectiles",
        );
        return true;
    }
    if head.starts_with("filter ") && head.ends_with(" to match") {
        filter(cx, l);
        return true;
    }
    false
}

/// `filter {list::*} to match:` keeps the elements for which every condition
/// below holds. `input` is the element, `input index` its index.
fn filter(cx: &mut Cx, l: &Line) {
    let head = l.head();
    let list = head
        .get("filter ".len()..head.len() - " to match".len())
        .unwrap_or("")
        .trim();
    let is_list = list.starts_with('{') && list.ends_with("::*}") && list.matches('{').count() == 1;
    if !is_list {
        cx.err(
            l.num,
            format!("'filter' needs a list variable like {{_list::*}}, got '{list}'"),
        );
        return;
    }
    if l.children.is_empty() {
        cx.err(l.num, "'filter ... to match:' needs at least one condition below it");
        return;
    }
    if let Some(bad) = l.children.iter().find(|c| c.opens_block()) {
        cx.err(
            bad.num,
            "conditions of a 'filter' section are single lines (use 'or' / 'and' inside one line)",
        );
        return;
    }
    let tmp = format!("{{_w4filter{}::*}}", l.num);
    let conds = rewrite_lines(&l.children, &[("input index", "loop-index"), ("input", "loop-value")]);
    let mut inner = vec![line(l.num, format!("add loop-value to {list}"), Vec::new())];
    for c in conds.into_iter().rev() {
        inner = vec![line(c.num, format!("if {}:", c.text), inner)];
    }
    let lines = vec![
        line(l.num, format!("set {tmp} to {list}"), Vec::new()),
        line(l.num, format!("delete {list}"), Vec::new()),
        line(l.num, format!("loop {tmp}:"), inner),
        line(l.num, format!("delete {tmp}"), Vec::new()),
    ];
    cx.block(&lines);
}

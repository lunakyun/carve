//! Turns a script file into a tree of indented lines.

use crate::error::ScriptError;
use crate::lexer::strip_comment;

#[derive(Debug, Clone)]
pub struct Line {
    pub num: usize,
    pub text: String,
    pub children: Vec<Line>,
}

impl Line {
    pub fn opens_block(&self) -> bool {
        self.text.ends_with(':')
    }
    /// Text without the trailing `:`.
    pub fn head(&self) -> &str {
        self.text.strip_suffix(':').unwrap_or(&self.text).trim()
    }
}

#[derive(Clone, Copy, PartialEq)]
enum Unit {
    Tab,
    Spaces(usize),
}

/// The line tree and the errors of the lines that were left out.
pub fn parse_blocks(file: &str, src: &str) -> (Vec<Line>, Vec<ScriptError>) {
    let mut errors = Vec::new();
    let mut unit: Option<Unit> = None;
    let mut flat: Vec<(usize, usize, String)> = Vec::new(); // level, line, text
    for (i, raw) in src.lines().enumerate() {
        let num = i + 1;
        let raw = raw.trim_start_matches('\u{feff}');
        let line = strip_comment(raw).trim_end();
        if line.trim().is_empty() {
            continue;
        }
        let indent: String = line.chars().take_while(|c| *c == ' ' || *c == '\t').collect();
        let text = line[indent.len()..].to_string();
        let level = if indent.is_empty() {
            0
        } else {
            let tabs = indent.chars().all(|c| c == '\t');
            let spaces = indent.chars().all(|c| c == ' ');
            if !tabs && !spaces {
                errors.push(ScriptError::new(
                    file,
                    num,
                    "mixed tabs and spaces in indentation",
                ));
                continue;
            }
            let u = *unit.get_or_insert(if tabs {
                Unit::Tab
            } else {
                Unit::Spaces(indent.len())
            });
            match u {
                Unit::Tab if tabs => indent.len(),
                Unit::Spaces(n) if spaces && indent.len().is_multiple_of(n) => indent.len() / n,
                Unit::Tab => {
                    errors.push(ScriptError::new(
                        file,
                        num,
                        "this file is indented with tabs, but this line uses spaces",
                    ));
                    continue;
                }
                Unit::Spaces(n) => {
                    if tabs {
                        errors.push(ScriptError::new(
                            file,
                            num,
                            "this file is indented with spaces, but this line uses tabs",
                        ));
                    } else {
                        errors.push(ScriptError::new(
                            file,
                            num,
                            format!("indentation must be a multiple of {n} spaces"),
                        ));
                    }
                    continue;
                }
            }
        };
        flat.push((level, num, text));
    }

    // Like Skript: a badly indented line or an empty section is reported and
    // left out; everything else still loads.
    let mut kept: Vec<(usize, usize, String)> = Vec::with_capacity(flat.len());
    for item in flat {
        let (level, num, _) = &item;
        let allowed = match kept.last() {
            None => 0,
            Some((pl, _, pt)) => {
                if pt.ends_with(':') {
                    pl + 1
                } else {
                    *pl
                }
            }
        };
        if *level > allowed {
            errors.push(ScriptError::new(
                file,
                *num,
                "unexpected indentation (the line above does not end with ':')",
            ));
            continue;
        }
        if let Some((pl, pn, pt)) = kept.last()
            && opens(pt)
            && *level <= *pl
        {
            errors.push(ScriptError::new(
                file,
                *pn,
                "empty block: nothing is indented under this line",
            ));
            kept.pop();
        }
        kept.push(item);
    }
    if let Some((_, pn, pt)) = kept.last()
        && opens(pt)
    {
        errors.push(ScriptError::new(
            file,
            *pn,
            "empty block: nothing is indented under this line",
        ));
        kept.pop();
    }
    let mut i = 0;
    (build(&kept, &mut i, 0), errors)
}

/// A line that must have an indented block. `key: value:` (an option whose
/// value ends with `:`) is a plain line: only the last `:` is outside quotes,
/// but another `: ` comes before it.
fn opens(line: &str) -> bool {
    let Some(body) = line.strip_suffix(':') else {
        return false;
    };
    let mut in_str = false;
    let mut prev = ' ';
    for c in body.chars() {
        if c == '"' {
            in_str = !in_str;
        }
        // `name: value` inside the line, not `::` of a list or a return type.
        if !in_str && prev == ':' && c == ' ' && !body.contains("::") {
            return false;
        }
        prev = c;
    }
    true
}

fn build(flat: &[(usize, usize, String)], i: &mut usize, level: usize) -> Vec<Line> {
    let mut out: Vec<Line> = Vec::new();
    while let Some((l, num, text)) = flat.get(*i) {
        if *l < level {
            break;
        }
        if *l > level {
            // child of the last line
            let children = build(flat, i, level + 1);
            if let Some(last) = out.last_mut() {
                last.children = children;
            }
            continue;
        }
        out.push(Line {
            num: *num,
            text: text.clone(),
            children: Vec::new(),
        });
        *i += 1;
    }
    out
}

use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptError {
    pub file: String,
    pub line: usize,
    pub msg: String,
    /// "did you mean" suggestions (closest patterns).
    pub suggestions: Vec<String>,
    /// The offending source line, trimmed (empty when not known).
    pub code: String,
}

impl ScriptError {
    pub fn new(file: &str, line: usize, msg: impl Into<String>) -> Self {
        ScriptError {
            file: file.to_string(),
            line,
            msg: msg.into(),
            suggestions: Vec::new(),
            code: String::new(),
        }
    }

    /// Skript's error layout, with `&` color codes:
    ///
    /// ```text
    /// Line 9: (bank.carve)
    ///     Can't understand this expression: 'd...'
    ///     Line: return d...
    /// ```
    pub fn skript_lines(&self) -> Vec<String> {
        let mut out = vec![format!("&4Line {}: &c({})", self.line, self.file)];
        out.push(format!("    &c{}", capitalize(&self.msg)));
        if !self.code.is_empty() {
            out.push(format!("    &7Line: {}", self.code));
        }
        if !self.suggestions.is_empty() {
            out.push(format!("    &7Did you mean: &f{}", self.suggestions.join(" &7|&f ")));
        }
        out
    }

    /// [`skript_lines`](Self::skript_lines) without color codes, for the console log.
    pub fn plain_lines(&self) -> Vec<String> {
        self.skript_lines().iter().map(|l| strip_codes(l)).collect()
    }
}

/// Fill in [`ScriptError::code`] from the sources, where it is still empty.
pub fn fill_code(errors: &mut [ScriptError], files: &[(String, String)]) {
    for e in errors.iter_mut().filter(|e| e.code.is_empty() && e.line > 0) {
        let Some((_, src)) = files.iter().find(|(n, _)| *n == e.file) else {
            continue;
        };
        if let Some(raw) = src.lines().nth(e.line - 1) {
            let raw = raw.trim_start_matches('\u{feff}');
            e.code = crate::lexer::strip_comment(raw).trim().to_string();
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().chain(c).collect(),
        None => String::new(),
    }
}

/// Remove `&x` color codes.
pub fn strip_codes(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut it = s.chars().peekable();
    while let Some(c) = it.next() {
        if c == '&' && it.peek().is_some_and(|n| n.is_ascii_hexdigit() || "klmnorKLMNOR".contains(*n)) {
            it.next();
            continue;
        }
        out.push(c);
    }
    out
}

impl fmt::Display for ScriptError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.file, self.line, self.msg)?;
        if !self.suggestions.is_empty() {
            write!(f, " (did you mean: {})", self.suggestions.join(" | "))?;
        }
        Ok(())
    }
}

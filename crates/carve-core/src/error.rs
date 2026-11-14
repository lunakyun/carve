use std::fmt;

#[derive(Debug, Clone, PartialEq)]
pub struct ScriptError {
    pub file: String,
    pub line: usize,
    pub msg: String,
    /// "did you mean" suggestions (closest patterns).
    pub suggestions: Vec<String>,
}

impl ScriptError {
    pub fn new(file: &str, line: usize, msg: impl Into<String>) -> Self {
        ScriptError {
            file: file.to_string(),
            line,
            msg: msg.into(),
            suggestions: Vec::new(),
        }
    }
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

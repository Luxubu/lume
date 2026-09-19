//! Lume compile errors. Every error names a line and column, says what went
//! wrong in plain words, and, where possible, suggests the fix (principle 6).

use std::fmt;

#[derive(Debug, Clone)]
pub struct LumeError {
    pub line: usize,
    pub col: usize,
    pub msg: String,
    pub help: Option<String>,
}

impl LumeError {
    pub fn new(line: usize, col: usize, msg: impl Into<String>) -> Self {
        LumeError { line, col, msg: msg.into(), help: None }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(help.into());
        self
    }

    /// Render the error the way the CLI prints it, with the offending source
    /// line and a caret under the column.
    pub fn render(&self, file: &str, src: &str) -> String {
        let mut out = String::new();
        out.push_str(&format!("error: {}\n", self.msg));
        out.push_str(&format!("  --> {}:{}:{}\n", file, self.line, self.col));
        if let Some(text) = src.lines().nth(self.line.saturating_sub(1)) {
            let num = format!("{}", self.line);
            let pad = " ".repeat(num.len());
            out.push_str(&format!("{} |\n", pad));
            out.push_str(&format!("{} | {}\n", num, text));
            out.push_str(&format!(
                "{} | {}^\n",
                pad,
                " ".repeat(self.col.saturating_sub(1))
            ));
        }
        if let Some(h) = &self.help {
            out.push_str(&format!("  help: {}\n", h));
        }
        out
    }
}

impl fmt::Display for LumeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}:{}: {}", self.line, self.col, self.msg)
    }
}

pub type Result<T> = std::result::Result<T, LumeError>;

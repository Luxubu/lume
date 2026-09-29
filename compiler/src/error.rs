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

/// "a `Int`" is written in many messages with the type filled in later;
/// the article follows the name: "an `Int`", "an `Error`", "a `User`".
pub fn fix_articles(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let b: Vec<char> = s.chars().collect();
    let mut i = 0;
    while i < b.len() {
        // " a `X" or a message starting "a `X", with X a vowel sound
        let at_word = i == 0 || b[i - 1] == ' ' || b[i - 1] == '(';
        if at_word && b[i] == 'a' && b.get(i + 1) == Some(&' ') && b.get(i + 2) == Some(&'`') {
            if let Some(c) = b.get(i + 3) {
                if "aeioAEIO".contains(*c) {
                    out.push_str("an");
                    i += 1;
                    continue;
                }
            }
        }
        out.push(b[i]);
        i += 1;
    }
    out
}

impl LumeError {
    pub fn new(line: usize, col: usize, msg: impl Into<String>) -> Self {
        LumeError { line, col, msg: fix_articles(&msg.into()), help: None }
    }

    pub fn with_help(mut self, help: impl Into<String>) -> Self {
        self.help = Some(fix_articles(&help.into()));
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

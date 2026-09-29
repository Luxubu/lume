//! Diagnostics as data, for editors: `lume check --json` and `lume lsp`.
//!
//! The compiler renders every error the same way (`error: …`, then
//! `  --> file:line:col`, the source line, and `  help: …`), so a rendered
//! message is read back into its parts here rather than threading a second
//! error type through every pass. Warnings, which the driver prints as it
//! goes, are collected instead while a sink is open.
//!
//! An editor also holds text it has not saved. `set_overlay` gives the
//! loader that text in place of the file on disk.

use std::cell::RefCell;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

thread_local! {
    static OVERLAY: RefCell<HashMap<PathBuf, String>> = RefCell::new(HashMap::new());
    static SINK: RefCell<Option<Vec<String>>> = const { RefCell::new(None) };
    static INDEX: RefCell<Option<Vec<Use>>> = const { RefCell::new(None) };
}

/// One name the compiler resolved while checking: where it is used, what
/// to show on hover, and where it was defined (a column of 0 means "find
/// the name on that line").
#[derive(Clone, Debug)]
pub struct Use {
    pub file: String,
    pub line: usize,
    pub col: usize,
    pub name: String,
    pub hover: String,
    pub def: Option<(String, usize, usize)>,
}

/// Is a check being indexed? Cheap enough to ask before building a note.
pub fn indexing() -> bool {
    INDEX.with(|i| i.borrow().is_some())
}

pub fn note(u: Use) {
    INDEX.with(|i| {
        if let Some(v) = i.borrow_mut().as_mut() {
            v.push(u);
        }
    });
}

/// Runs `f` with every resolved name recorded.
pub fn indexed<T>(f: impl FnOnce() -> T) -> (T, Vec<Use>) {
    INDEX.with(|i| *i.borrow_mut() = Some(Vec::new()));
    let r = f();
    let got = INDEX.with(|i| i.borrow_mut().take().unwrap_or_default());
    (r, got)
}

fn key(path: &Path) -> PathBuf {
    std::fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf())
}

/// The text of `path`: the editor's copy when it has one, else the file.
pub fn read_source(path: &Path) -> std::io::Result<String> {
    if let Some(s) = OVERLAY.with(|o| o.borrow().get(&key(path)).cloned()) {
        return Ok(s);
    }
    std::fs::read_to_string(path)
}

pub fn set_overlay(path: &Path, text: String) {
    OVERLAY.with(|o| o.borrow_mut().insert(key(path), text));
}

pub fn clear_overlay(path: &Path) {
    OVERLAY.with(|o| o.borrow_mut().remove(&key(path)));
}

/// Prints a rendered warning, or keeps it when a sink is open.
pub fn warn(rendered: String) {
    let kept = SINK.with(|s| match s.borrow_mut().as_mut() {
        Some(v) => {
            v.push(rendered.clone());
            true
        }
        None => false,
    });
    if !kept {
        eprint!("{}", rendered);
    }
}

/// Runs `f` with warnings collected rather than printed.
pub fn collecting<T>(f: impl FnOnce() -> T) -> (T, Vec<String>) {
    SINK.with(|s| *s.borrow_mut() = Some(Vec::new()));
    let r = f();
    let got = SINK.with(|s| s.borrow_mut().take().unwrap_or_default());
    (r, got)
}

#[derive(Clone, Debug, PartialEq)]
pub struct Diagnostic {
    /// "error" or "warning".
    pub severity: String,
    pub file: Option<String>,
    pub line: usize,
    pub col: usize,
    pub message: String,
    pub help: Option<String>,
}

/// Every diagnostic in rendered text: each starts at a line beginning
/// `error:` or `warning:`.
pub fn parse(rendered: &str) -> Vec<Diagnostic> {
    let mut out: Vec<Diagnostic> = Vec::new();
    for line in rendered.lines() {
        let head = line.strip_prefix("error: ").map(|m| ("error", m)).or_else(|| line.strip_prefix("warning: ").map(|m| ("warning", m)));
        if let Some((sev, msg)) = head {
            out.push(Diagnostic { severity: sev.to_string(), file: None, line: 0, col: 0, message: msg.to_string(), help: None });
            continue;
        }
        let d = match out.last_mut() {
            Some(d) => d,
            None => continue,
        };
        if let Some(loc) = line.strip_prefix("  --> ") {
            if d.file.is_none() {
                // `path:line:col`, where the path may itself hold a `:`
                let mut parts = loc.rsplitn(3, ':');
                let col = parts.next().and_then(|c| c.trim().parse().ok());
                let ln = parts.next().and_then(|l| l.trim().parse().ok());
                if let (Some(col), Some(ln), Some(file)) = (col, ln, parts.next()) {
                    d.file = Some(file.to_string());
                    d.line = ln;
                    d.col = col;
                }
            }
        } else if let Some(h) = line.strip_prefix("  help: ") {
            d.help = Some(h.to_string());
        } else if d.file.is_none() && d.help.is_none() && !line.trim().is_empty() && !line.starts_with(' ') {
            // a message that runs on (the rustc banner, a usage line)
            d.message.push('\n');
            d.message.push_str(line);
        }
    }
    out
}

pub fn to_json(d: &Diagnostic) -> serde_json::Value {
    serde_json::json!({
        "severity": d.severity,
        "file": d.file,
        "line": d.line,
        "col": d.col,
        "message": d.message,
        "help": d.help,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_rendered_error() {
        let r = "error: unknown name `x`\n  --> a/b.lume:3:5\n  |\n3 |   x\n  |   ^\n  help: did you mean `y`?\n";
        let d = parse(r);
        assert_eq!(d.len(), 1);
        assert_eq!(d[0].file.as_deref(), Some("a/b.lume"));
        assert_eq!((d[0].line, d[0].col), (3, 5));
        assert_eq!(d[0].help.as_deref(), Some("did you mean `y`?"));
    }
}

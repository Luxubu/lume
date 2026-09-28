//! `lume lsp`: a language server over stdin and stdout.
//!
//! It speaks just enough of the Language Server Protocol to put Lume's own
//! errors and warnings in an editor as the file is typed: the editor sends
//! the whole text on every change, the server checks the program that text
//! belongs to — the package's `main.lume`, or the program whose imports reach
//! the file, or the file on its own — and publishes what `lume check` would
//! print, file by file. The checking is the compiler's, unchanged; unsaved
//! text reaches it through `diag::set_overlay`.

use std::collections::{HashMap, HashSet};
use std::io::{BufRead, Write};
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::{diag, loader, manifest};

pub fn serve() {
    let stdin = std::io::stdin();
    let mut input = stdin.lock();
    let mut server = Server { open: HashMap::new(), published: HashMap::new(), shutting_down: false };
    loop {
        let msg = match read_message(&mut input) {
            Some(m) => m,
            None => return,
        };
        if let Some(code) = server.handle(&msg) {
            std::process::exit(code);
        }
    }
}

/// One message: `Content-Length: n`, a blank line, then `n` bytes of JSON.
fn read_message(input: &mut impl BufRead) -> Option<Value> {
    let mut len: Option<usize> = None;
    loop {
        let mut line = String::new();
        if input.read_line(&mut line).ok()? == 0 {
            return None;
        }
        let line = line.trim_end();
        if line.is_empty() {
            break;
        }
        if let Some(v) = line.strip_prefix("Content-Length:") {
            len = v.trim().parse().ok();
        }
    }
    let mut buf = vec![0u8; len?];
    input.read_exact(&mut buf).ok()?;
    serde_json::from_slice(&buf).ok()
}

fn send(msg: &Value) {
    let body = msg.to_string();
    let mut out = std::io::stdout().lock();
    let _ = write!(out, "Content-Length: {}\r\n\r\n{}", body.len(), body);
    let _ = out.flush();
}

struct Server {
    /// uri -> the file it names, for every document the editor has open
    open: HashMap<String, PathBuf>,
    /// for each program checked (by its entry file), the uris last given
    /// diagnostics, so a fixed file is cleared and another program's are not
    published: HashMap<PathBuf, HashSet<String>>,
    shutting_down: bool,
}

impl Server {
    /// Handles one message; `Some(code)` when the server should exit.
    fn handle(&mut self, msg: &Value) -> Option<i32> {
        let method = msg.get("method").and_then(|m| m.as_str()).unwrap_or("");
        let id = msg.get("id").cloned();
        let params = msg.get("params").cloned().unwrap_or(Value::Null);
        match method {
            "initialize" => {
                send(&json!({ "jsonrpc": "2.0", "id": id, "result": {
                    "capabilities": {
                        "textDocumentSync": { "openClose": true, "change": 1, "save": { "includeText": false } }
                    },
                    "serverInfo": { "name": "lume", "version": env!("CARGO_PKG_VERSION") }
                }}));
            }
            "shutdown" => {
                self.shutting_down = true;
                send(&json!({ "jsonrpc": "2.0", "id": id, "result": null }));
            }
            "exit" => return Some(if self.shutting_down { 0 } else { 1 }),
            "textDocument/didOpen" => {
                let uri = str_at(&params, &["textDocument", "uri"]);
                let text = str_at(&params, &["textDocument", "text"]);
                if let Some(path) = uri_to_path(&uri) {
                    diag::set_overlay(&path, text);
                    self.open.insert(uri.clone(), path.clone());
                    self.check(&path);
                }
            }
            "textDocument/didChange" => {
                let uri = str_at(&params, &["textDocument", "uri"]);
                // full sync: the last change holds the whole text
                let text = params.get("contentChanges").and_then(|c| c.as_array()).and_then(|a| a.last()).and_then(|c| c.get("text")).and_then(|t| t.as_str()).map(|s| s.to_string());
                if let (Some(path), Some(text)) = (uri_to_path(&uri), text) {
                    diag::set_overlay(&path, text);
                    self.check(&path);
                }
            }
            "textDocument/didSave" => {
                if let Some(path) = uri_to_path(&str_at(&params, &["textDocument", "uri"])) {
                    self.check(&path);
                }
            }
            "textDocument/didClose" => {
                let uri = str_at(&params, &["textDocument", "uri"]);
                if let Some(path) = self.open.remove(&uri) {
                    diag::clear_overlay(&path);
                }
                // a closed file's messages go with it
                let mut had = false;
                for set in self.published.values_mut() {
                    had |= set.remove(&uri);
                }
                if had {
                    publish(&uri, Vec::new());
                }
            }
            _ => {
                // a request we do not answer still gets an answer
                if id.is_some() && !method.is_empty() {
                    send(&json!({ "jsonrpc": "2.0", "id": id, "error": { "code": -32601, "message": format!("`{}` is not supported", method) } }));
                }
            }
        }
        None
    }

    /// Checks the program `path` belongs to and publishes its diagnostics.
    fn check(&mut self, path: &Path) {
        let entry = entry_for(path);
        let run = std::panic::catch_unwind(|| diag::collecting(|| crate::compile_to_rust(&entry, true)));
        let mut ds: Vec<diag::Diagnostic> = Vec::new();
        match run {
            Ok((r, warnings)) => {
                ds.extend(warnings.iter().flat_map(|w| diag::parse(w)));
                if let Err(e) = r {
                    ds.extend(diag::parse(&e));
                }
            }
            Err(_) => ds.push(diag::Diagnostic {
                severity: "error".into(),
                file: Some(entry.display().to_string()),
                line: 1,
                col: 1,
                message: "the Lume compiler stopped while checking this program; `lume check` on it will show where".into(),
                help: None,
            }),
        }
        // by file; a message with no place goes on the file being edited
        let cwd = std::env::current_dir().unwrap_or_default();
        let mut by_uri: HashMap<String, Vec<Value>> = HashMap::new();
        for d in &ds {
            let file = match &d.file {
                Some(f) => cwd.join(f),
                None => path.to_path_buf(),
            };
            let file = std::fs::canonicalize(&file).unwrap_or(file);
            let text = diag::read_source(&file).unwrap_or_default();
            by_uri.entry(path_to_uri(&file)).or_default().push(to_lsp(d, &text));
        }
        let now: HashSet<String> = by_uri.keys().cloned().collect();
        let before = self.published.remove(&entry).unwrap_or_default();
        for stale in before.difference(&now) {
            publish(stale, Vec::new());
        }
        // the edited file is always published, so a fix clears it at once
        let here = path_to_uri(&std::fs::canonicalize(path).unwrap_or(path.to_path_buf()));
        if !now.contains(&here) && !before.contains(&here) {
            publish(&here, Vec::new());
        }
        for (uri, list) in by_uri {
            publish(&uri, list);
        }
        self.published.insert(entry, now);
    }
}

fn publish(uri: &str, diagnostics: Vec<Value>) {
    send(&json!({ "jsonrpc": "2.0", "method": "textDocument/publishDiagnostics", "params": { "uri": uri, "diagnostics": diagnostics } }));
}

/// A Lume diagnostic as the protocol has it: zero-based, in UTF-16 units,
/// covering the word the error points at.
fn to_lsp(d: &diag::Diagnostic, text: &str) -> Value {
    let line = d.line.saturating_sub(1);
    let src_line = text.lines().nth(line).unwrap_or("");
    let chars: Vec<char> = src_line.chars().collect();
    let start = d.col.saturating_sub(1).min(chars.len());
    let mut end = start;
    while end < chars.len() && (chars[end].is_alphanumeric() || chars[end] == '_' || chars[end] == '?' || chars[end] == '!') {
        end += 1;
    }
    if end == start {
        end = (start + 1).min(chars.len().max(start + 1));
    }
    let u16_at = |i: usize| chars[..i.min(chars.len())].iter().map(|c| c.len_utf16()).sum::<usize>() + i.saturating_sub(chars.len());
    let message = match &d.help {
        Some(h) => format!("{}\nhelp: {}", d.message, h),
        None => d.message.clone(),
    };
    json!({
        "range": {
            "start": { "line": line, "character": u16_at(start) },
            "end": { "line": line, "character": u16_at(end) }
        },
        "severity": if d.severity == "warning" { 2 } else { 1 },
        "source": "lume",
        "message": message
    })
}

/// Which file to check so that `path` is checked as part of its program:
/// in a package, its `main.lume` (or `lib.lume`); otherwise a `main.lume` in
/// this folder or one above whose imports reach `path`; otherwise `path`.
fn entry_for(path: &Path) -> PathBuf {
    let dir = path.parent().unwrap_or(Path::new("."));
    if let Some(toml) = manifest::find(dir) {
        let root = toml.parent().map(|p| p.to_path_buf()).unwrap_or_default();
        let root = if root.as_os_str().is_empty() { PathBuf::from(".") } else { root };
        let root = std::fs::canonicalize(&root).unwrap_or(root);
        for f in ["main.lume", "lib.lume"] {
            if root.join(f).is_file() {
                return root.join(f);
            }
        }
    }
    let me = std::fs::canonicalize(path).unwrap_or(path.to_path_buf());
    let mut d = me.parent().map(|p| p.to_path_buf());
    while let Some(dir) = d {
        let main = dir.join("main.lume");
        if main.is_file() && main != me {
            if let Ok(l) = loader::load(&main) {
                if l.modules.iter().any(|m| std::fs::canonicalize(&m.path).ok().as_deref() == Some(me.as_path())) {
                    return main;
                }
            }
        }
        d = dir.parent().map(|p| p.to_path_buf());
    }
    path.to_path_buf()
}

fn str_at(v: &Value, keys: &[&str]) -> String {
    let mut cur = v;
    for k in keys {
        cur = match cur.get(k) {
            Some(x) => x,
            None => return String::new(),
        };
    }
    cur.as_str().unwrap_or("").to_string()
}

/// `file:///a/b%20c.lume` -> `/a/b c.lume`.
fn uri_to_path(uri: &str) -> Option<PathBuf> {
    let rest = uri.strip_prefix("file://")?;
    let bytes = rest.as_bytes();
    let mut out = Vec::with_capacity(bytes.len());
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(b) = u8::from_str_radix(&rest[i + 1..i + 3], 16) {
                out.push(b);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i]);
        i += 1;
    }
    Some(PathBuf::from(String::from_utf8_lossy(&out).to_string()))
}

fn path_to_uri(p: &Path) -> String {
    let mut s = String::from("file://");
    for b in p.display().to_string().bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'_' | b'.' | b'~' => s.push(b as char),
            _ => s.push_str(&format!("%{:02X}", b)),
        }
    }
    s
}

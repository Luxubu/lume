//! `lume.toml`: what makes a folder a package. The file is TOML, read here
//! by hand because Lume needs only a small part of it: `[package]` with a
//! name and a version, `[dependencies]` of other Lume packages by path, and
//! `[rust]` crates with the requirement Cargo will be given.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::error::LumeError;
use crate::fetch::{self, Fetcher};

pub const FILE: &str = "lume.toml";

pub struct Manifest {
    pub name: String,
    /// The folder that holds `lume.toml`: where module paths start.
    pub root: PathBuf,
    /// `lume.toml` itself, and its text, for errors that point into it.
    pub path: PathBuf,
    pub src: String,
    pub deps: Vec<DepSpec>,
    /// `regex = "1"` under `[rust]`: the crate, what Cargo is given, the line.
    pub rust: Vec<(String, String, usize)>,
    /// For a package from git: what was asked for, the commit it resolved
    /// to, and the package that asked. Set by `load_all`.
    pub origin: Option<Origin>,
}

pub struct Origin {
    pub source: String,
    pub commit: String,
    pub asked_by: String,
}

pub struct DepSpec {
    pub name: String,
    pub source: Source,
    pub line: usize,
}

pub enum Source {
    Path(PathBuf),
    /// `git = "…"` with at most one of `tag`, `rev`, `branch`.
    Git { url: String, reference: GitRef },
}

#[derive(Clone)]
pub enum GitRef {
    Tag(String),
    Rev(String),
    Branch(String),
    /// Neither: the repository's default branch.
    Default,
}

impl GitRef {
    /// How the lock file and messages name it: `?tag=v1`, or nothing.
    pub fn query(&self) -> String {
        match self {
            GitRef::Tag(t) => format!("?tag={}", t),
            GitRef::Rev(r) => format!("?rev={}", r),
            GitRef::Branch(b) => format!("?branch={}", b),
            GitRef::Default => String::new(),
        }
    }
}

/// The nearest `lume.toml` in `dir` or a folder above it, as Cargo finds
/// `Cargo.toml`. The path is relative to the working directory when it can
/// be, so messages name files the way the user wrote them.
pub fn find(dir: &Path) -> Option<PathBuf> {
    let start = if dir.as_os_str().is_empty() { PathBuf::from(".") } else { dir.to_path_buf() };
    let mut d = std::fs::canonicalize(&start).ok()?;
    loop {
        let f = d.join(FILE);
        if f.is_file() {
            return Some(relative(&f));
        }
        if !d.pop() {
            return None;
        }
    }
}

/// `abs` as seen from the working directory: `shelf/lume.toml`, or
/// `../lume.toml` for a folder above it.
fn relative(abs: &Path) -> PathBuf {
    let cwd = match std::env::current_dir().ok().and_then(|c| std::fs::canonicalize(c).ok()) {
        Some(c) => c,
        None => return abs.to_path_buf(),
    };
    let mut up = PathBuf::new();
    let mut base = cwd.as_path();
    loop {
        if let Ok(rest) = abs.strip_prefix(base) {
            return up.join(rest);
        }
        match base.parent() {
            Some(p) => {
                base = p;
                up.push("..");
            }
            None => return abs.to_path_buf(),
        }
    }
}

/// `a/b/../c` as `a/c`, without asking the file system.
pub fn normalize(p: &Path) -> PathBuf {
    use std::path::Component;
    let mut out: Vec<Component> = Vec::new();
    for c in p.components() {
        match c {
            Component::CurDir => {}
            Component::ParentDir => match out.last() {
                Some(Component::Normal(_)) => {
                    out.pop();
                }
                _ => out.push(c),
            },
            _ => out.push(c),
        }
    }
    out.iter().collect()
}

/// A package name follows the rules for a Lume name: lower case, digits
/// and `_`, not starting with a digit. `rust` names crates, so it is taken.
pub fn check_name(name: &str) -> Option<String> {
    let ok = !name.is_empty()
        && name.chars().next().map(|c| c.is_ascii_lowercase() || c == '_').unwrap_or(false)
        && name.chars().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == '_');
    if !ok {
        return Some(format!("`{}` cannot name a package: use lower case letters, digits and `_`, starting with a letter", name));
    }
    if name == "rust" {
        return Some("`rust` cannot name a package: `import rust.…` means a Rust crate".into());
    }
    None
}

/// A quoted string at the start of `s`, and what follows it.
fn string_at(s: &str) -> Option<(String, &str)> {
    let s = s.strip_prefix('"')?;
    let end = s.find('"')?;
    Some((s[..end].to_string(), &s[end + 1..]))
}

/// The line without a trailing `# comment` (a `#` inside quotes stays).
fn strip_comment(line: &str) -> &str {
    let mut in_str = false;
    for (i, c) in line.char_indices() {
        match c {
            '"' => in_str = !in_str,
            '#' if !in_str => return &line[..i],
            _ => {}
        }
    }
    line
}

/// `{ path = "../tally", tag = "v1" }` as key/value pairs.
fn inline_table(s: &str) -> Option<Vec<(String, String)>> {
    let inner = s.trim().strip_prefix('{')?.strip_suffix('}')?;
    let mut out = Vec::new();
    for part in inner.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }
        let (k, v) = part.split_once('=')?;
        let (v, rest) = string_at(v.trim())?;
        if !rest.trim().is_empty() {
            return None;
        }
        out.push((k.trim().to_string(), v));
    }
    Some(out)
}

pub fn read(path: &Path) -> Result<Manifest, String> {
    let src = crate::diag::read_source(path).map_err(|e| format!("error: cannot read `{}`: {}", path.display(), e))?;
    let shown = path.display().to_string();
    let err = |line: usize, msg: String, help: Option<String>| {
        let mut e = LumeError::new(line, 1, msg);
        if let Some(h) = help {
            e = e.with_help(h);
        }
        e.render(&shown, &src)
    };
    let root = path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    let mut section = String::new();
    let mut name: Option<(String, usize)> = None;
    let mut deps = Vec::new();
    let mut rust = Vec::new();
    let mut seen: HashSet<(String, String)> = HashSet::new();
    let mut sections_seen: HashSet<String> = HashSet::new();
    for (i, raw) in src.lines().enumerate() {
        let n = i + 1;
        let line = strip_comment(raw).trim();
        if line.is_empty() {
            continue;
        }
        if let Some(h) = line.strip_prefix('[') {
            let h = h.strip_suffix(']').ok_or_else(|| err(n, "a section header ends with `]`".into(), None))?.trim();
            if !matches!(h, "package" | "dependencies" | "rust") {
                return Err(err(n, format!("unknown section `[{}]`", h), Some("`lume.toml` has `[package]`, `[dependencies]` and `[rust]`".into())));
            }
            if !sections_seen.insert(h.to_string()) {
                return Err(err(n, format!("`[{}]` appears twice", h), None));
            }
            section = h.to_string();
            continue;
        }
        let (key, value) = line.split_once('=').ok_or_else(|| err(n, "expected `key = value`".into(), None))?;
        let key = key.trim().to_string();
        let value = value.trim();
        if section.is_empty() {
            return Err(err(n, format!("`{}` is outside any section", key), Some("start the file with `[package]`".into())));
        }
        if !seen.insert((section.clone(), key.clone())) {
            return Err(err(n, format!("`{}` is given twice in `[{}]`", key, section), None));
        }
        match section.as_str() {
            "package" => {
                let (v, rest) = string_at(value).ok_or_else(|| err(n, format!("`{}` takes a quoted string", key), None))?;
                if !rest.trim().is_empty() {
                    return Err(err(n, format!("`{}` takes a quoted string", key), None));
                }
                match key.as_str() {
                    "name" => {
                        if let Some(m) = check_name(&v) {
                            return Err(err(n, m, None));
                        }
                        name = Some((v, n));
                    }
                    // informational until there is a registry to give it meaning
                    "version" => {}
                    _ => return Err(err(n, format!("unknown key `{}` in `[package]`", key), Some("`[package]` has `name` and `version`".into()))),
                }
            }
            "dependencies" => {
                if let Some(m) = check_name(&key) {
                    return Err(err(n, m, None));
                }
                let table = inline_table(value).ok_or_else(|| {
                    err(n, format!("`{}` needs a source", key), Some(format!("write `{} = {{ path = \"../{}\" }}`", key, key)))
                })?;
                let mut path = None;
                let mut git = None;
                let mut reference = GitRef::Default;
                let mut refs = 0;
                for (k, v) in table {
                    match k.as_str() {
                        "path" => path = Some(v),
                        "git" => git = Some(v),
                        "tag" => {
                            reference = GitRef::Tag(v);
                            refs += 1;
                        }
                        "rev" => {
                            reference = GitRef::Rev(v);
                            refs += 1;
                        }
                        "branch" => {
                            reference = GitRef::Branch(v);
                            refs += 1;
                        }
                        _ => return Err(err(n, format!("unknown key `{}` for dependency `{}`", k, key), Some("a dependency has a `path`, or a `git` URL with a `tag`, `rev` or `branch`".into()))),
                    }
                }
                let source = match (path, git) {
                    (Some(_), Some(_)) => return Err(err(n, format!("`{}` has both a `path` and a `git` source", key), Some("a dependency comes from one place: `path` for a folder on disk, or `git` for a repository".into()))),
                    (Some(p), None) => {
                        if refs > 0 {
                            return Err(err(n, format!("`{}` comes from a path, so it has no `tag`, `rev` or `branch`", key), None));
                        }
                        Source::Path(normalize(&root.join(p)))
                    }
                    (None, Some(url)) => {
                        if refs > 1 {
                            return Err(err(n, format!("`{}` names more than one of `tag`, `rev` and `branch`", key), Some("give one: it says which commit to use".into())));
                        }
                        Source::Git { url: git_url(&root, &url), reference }
                    }
                    (None, None) => return Err(err(n, format!("`{}` needs a source", key), Some(format!("write `{} = {{ path = \"../{}\" }}`, or `{} = {{ git = \"https://…\", tag = \"…\" }}`", key, key, key)))),
                };
                deps.push(DepSpec { name: key, source, line: n });
            }
            _ => {
                // a requirement, or a whole inline table, handed to Cargo as written
                let ok = match string_at(value) {
                    Some((_, rest)) => rest.trim().is_empty(),
                    None => value.starts_with('{') && value.ends_with('}'),
                };
                if !ok {
                    return Err(err(n, format!("`{}` takes a version such as `\"1\"`, or a table such as `{{ version = \"1\", features = [\"…\"] }}`", key), None));
                }
                let v = string_at(value).map(|(s, _)| s).unwrap_or_else(|| value.to_string());
                rust.push((key, v, n));
            }
        }
    }
    let (name, _) = name.ok_or_else(|| err(1, "`lume.toml` has no package name".into(), Some("add `[package]` with `name = \"…\"`".into())))?;
    Ok(Manifest { name, root, path: path.to_path_buf(), src, deps, rust, origin: None })
}

/// A git URL as written, except a repository on this machine given by a
/// relative path, which is taken from the folder of `lume.toml`.
fn git_url(root: &Path, url: &str) -> String {
    let remote = url.contains("://") || url.contains('@') || Path::new(url).is_absolute();
    if remote {
        return url.to_string();
    }
    // from the folder, which exists, not the repository, which may not:
    // the lock file must name it the same way whether or not it is there
    let base = std::fs::canonicalize(if root.as_os_str().is_empty() { Path::new(".") } else { root }).unwrap_or(root.to_path_buf());
    normalize(&base.join(url)).display().to_string()
}

/// The folder below `dir`, at most `depth` levels down, holding a
/// `lume.toml` whose package is `name`.
fn find_package(dir: &Path, name: &str, depth: usize) -> Option<PathBuf> {
    let mut entries: Vec<PathBuf> = std::fs::read_dir(dir).ok()?.flatten().map(|e| e.path()).filter(|p| p.is_dir()).collect();
    entries.sort();
    for sub in &entries {
        let hidden = sub.file_name().map(|n| n.to_string_lossy().starts_with('.')).unwrap_or(true);
        if hidden {
            continue;
        }
        let f = sub.join(FILE);
        if f.is_file() {
            if let Ok(m) = read(&f) {
                if m.name == name {
                    return Some(sub.clone());
                }
            }
        }
    }
    if depth > 1 {
        for sub in &entries {
            if sub.file_name().map(|n| !n.to_string_lossy().starts_with('.')).unwrap_or(false) {
                if let Some(found) = find_package(sub, name, depth - 1) {
                    return Some(found);
                }
            }
        }
    }
    None
}

/// Every package a program uses: the root first, then each dependency once.
/// Refuses a dependency whose folder names another package, two copies of
/// one package, and packages that depend on each other.
pub fn load_all(root: Manifest, fetcher: &mut Fetcher) -> Result<Vec<Manifest>, String> {
    let mut all = vec![root];
    let mut by_name: HashMap<String, usize> = HashMap::new();
    by_name.insert(all[0].name.clone(), 0);
    let mut i = 0;
    while i < all.len() {
        let mut found = Vec::new();
        for d in &all[i].deps {
            let at = |msg: String, help: Option<String>| {
                let mut e = LumeError::new(d.line, 1, msg);
                if let Some(h) = help {
                    e = e.with_help(h);
                }
                e.render(&all[i].path.display().to_string(), &all[i].src)
            };
            let (dir, origin) = match &d.source {
                Source::Path(p) => (p.clone(), None),
                Source::Git { url, reference } => {
                    let (dir, commit) = fetcher.checkout(&d.name, url, reference).map_err(|(m, h)| at(m, h))?;
                    (dir, Some(Origin { source: fetch::source_key(url, reference), commit, asked_by: all[i].name.clone() }))
                }
            };
            // a repository may hold its package in a folder of its own, or
            // several: find the one with this name, as Cargo does
            let dir = if origin.is_some() && !dir.join(FILE).is_file() { find_package(&dir, &d.name, 3).unwrap_or(dir) } else { dir };
            let file = dir.join(FILE);
            if !file.is_file() {
                if let (Source::Git { url, reference }, Some(_)) = (&d.source, &origin) {
                    return Err(at(
                        format!("{}{} has no package named `{}`", url, reference.query(), d.name),
                        Some(format!("a repository's packages are found by the name in their `lume.toml`, at its top or up to three folders down; check that one is named `{}`", d.name)),
                    ));
                }
                let whose = if origin.is_some() { "that commit of the repository" } else { "the folder" };
                return Err(at(format!("`{}` has no `lume.toml`", dir.display()), Some(format!("a dependency is a package: {} needs a `lume.toml` whose name is `{}`", whose, d.name))));
            }
            let mut m = read(&file)?;
            if m.name != d.name {
                return Err(at(format!("`{}` is the package `{}`, not `{}`", dir.display(), m.name, d.name), Some(match &d.source {
                    Source::Path(_) => format!("name it by its own name: `{} = {{ path = \"…\" }}`", m.name),
                    Source::Git { .. } => format!("name it by its own name: `{} = {{ git = \"…\" }}`", m.name),
                })));
            }
            m.origin = origin;
            found.push((d.name.clone(), m, d.line));
        }
        for (name, m, line) in found {
            match by_name.get(&name) {
                Some(&j) => {
                    let here = |msg: String, help: &str| LumeError::new(line, 1, msg).with_help(help.to_string()).render(&all[i].path.display().to_string(), &all[i].src);
                    match (&all[j].origin, &m.origin) {
                        (Some(a), Some(b)) => {
                            if a.commit != b.commit {
                                return Err(here(
                                    format!("package `{}` is asked for at two commits: {} by `{}`, and {} by `{}`", name, fetch::short(&a.commit), a.asked_by, fetch::short(&b.commit), b.asked_by),
                                    &format!("a program has one copy of each package: make `{}` and `{}` ask for the same one ({} and {})", a.asked_by, b.asked_by, a.source, b.source),
                                ));
                            }
                        }
                        (None, None) => {
                            let a = std::fs::canonicalize(&all[j].root).unwrap_or(all[j].root.clone());
                            let b = std::fs::canonicalize(&m.root).unwrap_or(m.root.clone());
                            if a != b {
                                return Err(here(
                                    format!("package `{}` comes from two places: `{}` and `{}`", name, all[j].root.display(), m.root.display()),
                                    "a program has one copy of each package: point every `lume.toml` at the same folder",
                                ));
                            }
                        }
                        _ => {
                            let (git, path) = if m.origin.is_some() { (&m, &all[j]) } else { (&all[j], &m) };
                            return Err(here(
                                format!("package `{}` comes from two places: {} and `{}`", name, git.origin.as_ref().unwrap().source, path.root.display()),
                                "a program has one copy of each package: have every `lume.toml` take it from the same place",
                            ));
                        }
                    }
                }
                None => {
                    by_name.insert(name, all.len());
                    all.push(m);
                }
            }
        }
        i += 1;
    }
    // packages depend on each other one way only
    fn visit(k: usize, all: &[Manifest], by_name: &HashMap<String, usize>, path: &mut Vec<usize>, done: &mut HashSet<usize>) -> Result<(), String> {
        if done.contains(&k) {
            return Ok(());
        }
        if let Some(p) = path.iter().position(|&x| x == k) {
            let names: Vec<&str> = path[p..].iter().chain([&k]).map(|&x| all[x].name.as_str()).collect();
            let from = path[path.len() - 1];
            let line = all[from].deps.iter().find(|d| d.name == all[k].name).map(|d| d.line).unwrap_or(1);
            let e = LumeError::new(line, 1, format!("packages depend on each other: {}", names.join(" -> ")))
                .with_help("move what they share into a third package that both depend on");
            return Err(e.render(&all[from].path.display().to_string(), &all[from].src));
        }
        path.push(k);
        for d in &all[k].deps {
            visit(by_name[&d.name], all, by_name, path, done)?;
        }
        path.pop();
        done.insert(k);
        Ok(())
    }
    let mut done = HashSet::new();
    visit(0, &all, &by_name, &mut Vec::new(), &mut done)?;
    Ok(all)
}

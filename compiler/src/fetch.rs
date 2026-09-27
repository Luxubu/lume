//! Packages from git, and `lume.lock`.
//!
//! Each repository is fetched once into a bare copy under
//! `~/.lume/git/db/` (or `$LUME_HOME/git/db/`), and each commit a program
//! uses is checked out once, read-only in spirit, under `git/checkouts/`.
//! `lume.lock` at the root of the program records the commit every git
//! dependency resolved to, so the next build uses the same one, on any
//! machine, without asking the network. `lume update` moves them on.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::manifest::GitRef;

pub const LOCK_FILE: &str = "lume.lock";

/// Which locked commits a load may move.
#[derive(Clone, PartialEq)]
pub enum Update {
    No,
    All,
    One(String),
}

#[derive(Clone, PartialEq)]
pub struct LockEntry {
    pub name: String,
    /// `git+<url>?tag=v1`: what `lume.toml` asked for.
    pub source: String,
    pub commit: String,
}

pub struct Fetcher {
    home: PathBuf,
    locked: Vec<LockEntry>,
    update: Update,
    /// What this load resolved, for the new lock file.
    pub used: Vec<LockEntry>,
    /// URLs already fetched by this load: once is enough.
    fetched: std::collections::HashSet<String>,
}

/// `$LUME_HOME`, or `~/.lume`.
fn lume_home() -> PathBuf {
    if let Ok(h) = std::env::var("LUME_HOME") {
        return PathBuf::from(h);
    }
    std::env::var("HOME").map(|h| PathBuf::from(h).join(".lume")).unwrap_or_else(|_| PathBuf::from(".lume-home"))
}

/// A short name for a URL that stays the same from one run of lume to the
/// next (FNV-1a), so the cache is found again.
fn url_hash(url: &str) -> String {
    let mut h: u64 = 0xcbf29ce484222325;
    for b in url.bytes() {
        h ^= b as u64;
        h = h.wrapping_mul(0x100000001b3);
    }
    format!("{:016x}", h)
}

fn git(args: &[&str], dir: Option<&Path>) -> Result<String, String> {
    let mut c = Command::new("git");
    if let Some(d) = dir {
        c.arg("-C").arg(d);
    }
    let out = c
        .args(args)
        .env("GIT_TERMINAL_PROMPT", "0")
        .output()
        .map_err(|e| format!("cannot run git: {}", e))?;
    if out.status.success() {
        Ok(String::from_utf8_lossy(&out.stdout).trim().to_string())
    } else {
        Err(String::from_utf8_lossy(&out.stderr).trim().to_string())
    }
}

pub fn source_key(url: &str, reference: &GitRef) -> String {
    format!("git+{}{}", url, reference.query())
}

impl Fetcher {
    /// Reads the program's `lume.lock`, if it has one.
    pub fn new(root: &Path, update: Update) -> Result<Self, String> {
        let path = root.join(LOCK_FILE);
        let mut locked = Vec::new();
        if let Ok(text) = std::fs::read_to_string(&path) {
            let mut cur: Option<(Option<String>, Option<String>, Option<String>)> = None;
            let mut flush = |c: &mut Option<(Option<String>, Option<String>, Option<String>)>, locked: &mut Vec<LockEntry>| {
                if let Some((Some(name), Some(source), Some(commit))) = c.take() {
                    locked.push(LockEntry { name, source, commit });
                }
            };
            for (i, line) in text.lines().enumerate() {
                let t = line.trim();
                if t.is_empty() || t.starts_with('#') {
                    continue;
                }
                if t == "[[package]]" {
                    flush(&mut cur, &mut locked);
                    cur = Some((None, None, None));
                    continue;
                }
                let bad = || format!("error: `{}` line {} is not what lume writes\n  help: delete the file; the next build writes it again\n", path.display(), i + 1);
                let (k, v) = t.split_once('=').ok_or_else(bad)?;
                let v = v.trim().trim_matches('"').to_string();
                let c = cur.as_mut().ok_or_else(bad)?;
                match k.trim() {
                    "name" => c.0 = Some(v),
                    "source" => c.1 = Some(v),
                    "commit" => c.2 = Some(v),
                    _ => return Err(bad()),
                }
            }
            flush(&mut cur, &mut locked);
        }
        Ok(Fetcher { home: lume_home(), locked, update, used: Vec::new(), fetched: Default::default() })
    }

    /// The folder holding `url` at the commit the lock file, or else the
    /// reference, gives; and that commit. The error is a message and a help.
    pub fn checkout(&mut self, name: &str, url: &str, reference: &GitRef) -> Result<(PathBuf, String), (String, Option<String>)> {
        let key = source_key(url, reference);
        let moving = match &self.update {
            Update::No => false,
            Update::All => true,
            Update::One(n) => n == name,
        };
        let db = self.home.join("git").join("db").join(format!("{}-{}", name, url_hash(url)));
        let locked = if moving { None } else { self.locked.iter().find(|e| e.name == name && e.source == key).map(|e| e.commit.clone()) };
        let commit = match locked {
            Some(c) => {
                let dir = self.checkout_dir(name, url, &c);
                if !dir.is_dir() && !has_commit(&db, &c) {
                    self.fetch(name, url, &db)?;
                    if !has_commit(&db, &c) {
                        return Err((
                            format!("`{}` is locked to commit {}, which {} no longer has", name, short(&c), url),
                            Some(format!("run `lume update {}` to take the commit `lume.toml` asks for now", name)),
                        ));
                    }
                }
                c
            }
            None => {
                self.fetch(name, url, &db)?;
                resolve(&db, reference).map_err(|_| {
                    let what = match reference {
                        GitRef::Tag(t) => format!("no tag `{}`", t),
                        GitRef::Rev(r) => format!("no commit `{}`", r),
                        GitRef::Branch(b) => format!("no branch `{}`", b),
                        GitRef::Default => "no default branch".to_string(),
                    };
                    (format!("`{}` has {} at {}", name, what, url), Some("check the name in `lume.toml` against the repository".to_string()))
                })?
            }
        };
        let dir = self.checkout_dir(name, url, &commit);
        if !dir.is_dir() {
            make_checkout(&db, &commit, &dir).map_err(|e| (format!("cannot check out `{}` at {}: {}", name, short(&commit), e), None))?;
        }
        let entry = LockEntry { name: name.to_string(), source: key, commit: commit.clone() };
        if !self.used.contains(&entry) {
            self.used.push(entry);
        }
        Ok((dir, commit))
    }

    fn checkout_dir(&self, name: &str, url: &str, commit: &str) -> PathBuf {
        self.home.join("git").join("checkouts").join(format!("{}-{}", name, url_hash(url))).join(short(commit))
    }

    /// Brings the bare copy of `url` up to date: the only step that asks the
    /// network.
    fn fetch(&mut self, name: &str, url: &str, db: &Path) -> Result<(), (String, Option<String>)> {
        if !self.fetched.insert(url.to_string()) {
            return Ok(());
        }
        eprintln!("fetching `{}` from {}", name, url);
        let failed = |e: String| {
            let first = e.lines().last().unwrap_or("").to_string();
            (format!("cannot fetch `{}` from {}: {}", name, url, first), Some("check the URL, and that git can reach it: `git ls-remote <url>`".to_string()))
        };
        if db.is_dir() {
            git(&["fetch", "--quiet", "--force", "--prune", url, "+refs/heads/*:refs/heads/*", "+refs/tags/*:refs/tags/*"], Some(db)).map_err(failed)?;
        } else {
            let parent = db.parent().unwrap();
            std::fs::create_dir_all(parent).map_err(|e| (format!("cannot create `{}`: {}", parent.display(), e), None))?;
            let tmp = parent.join(format!(".{}.tmp", db.file_name().unwrap().to_string_lossy()));
            let _ = std::fs::remove_dir_all(&tmp);
            git(&["clone", "--quiet", "--bare", url, &tmp.display().to_string()], None).map_err(failed)?;
            std::fs::rename(&tmp, db).map_err(|e| (format!("cannot create `{}`: {}", db.display(), e), None))?;
        }
        Ok(())
    }

    /// Writes `lume.lock` when what was resolved differs from what it says.
    /// Returns (name, old commit, new commit) for each package that moved.
    pub fn write_lock(&self, root: &Path) -> Result<Vec<(String, Option<String>, String)>, String> {
        let mut used = self.used.clone();
        used.sort_by(|a, b| a.name.cmp(&b.name));
        let mut moved = Vec::new();
        for e in &used {
            let old = self.locked.iter().find(|o| o.name == e.name).map(|o| o.commit.clone());
            if old.as_deref() != Some(e.commit.as_str()) {
                moved.push((e.name.clone(), old, e.commit.clone()));
            }
        }
        let path = root.join(LOCK_FILE);
        if used.is_empty() && !path.exists() {
            return Ok(moved);
        }
        let mut text = String::from("# lume.lock: written by lume. The exact commit of every package that\n# comes from git; keep it with the program. `lume update` moves them on.\n");
        for e in &used {
            text.push_str(&format!("\n[[package]]\nname = \"{}\"\nsource = \"{}\"\ncommit = \"{}\"\n", e.name, e.source, e.commit));
        }
        if std::fs::read_to_string(&path).ok().as_deref() != Some(text.as_str()) {
            std::fs::write(&path, text).map_err(|e| format!("error: cannot write `{}`: {}\n", path.display(), e))?;
        }
        Ok(moved)
    }
}

pub fn short(commit: &str) -> &str {
    &commit[..commit.len().min(12)]
}

fn has_commit(db: &Path, commit: &str) -> bool {
    db.is_dir() && git(&["cat-file", "-e", &format!("{}^{{commit}}", commit)], Some(db)).is_ok()
}

fn resolve(db: &Path, reference: &GitRef) -> Result<String, String> {
    let spec = match reference {
        GitRef::Tag(t) => format!("refs/tags/{}^{{commit}}", t),
        GitRef::Branch(b) => format!("refs/heads/{}^{{commit}}", b),
        GitRef::Rev(r) => format!("{}^{{commit}}", r),
        GitRef::Default => "HEAD^{commit}".to_string(),
    };
    git(&["rev-parse", "--verify", "--quiet", &spec], Some(db))
}

/// The files of `commit`, in a folder of their own; made beside `dir` and
/// moved into place, so a half-made checkout is never used.
fn make_checkout(db: &Path, commit: &str, dir: &Path) -> Result<(), String> {
    let parent = dir.parent().unwrap();
    std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    let tmp = parent.join(format!(".{}.tmp", dir.file_name().unwrap().to_string_lossy()));
    let _ = std::fs::remove_dir_all(&tmp);
    git(&["clone", "--quiet", "--no-checkout", &db.display().to_string(), &tmp.display().to_string()], None)?;
    git(&["checkout", "--quiet", commit], Some(&tmp))?;
    let _ = std::fs::remove_dir_all(tmp.join(".git"));
    std::fs::rename(&tmp, dir).map_err(|e| e.to_string())
}

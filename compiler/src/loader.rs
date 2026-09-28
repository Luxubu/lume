//! Module loader: resolves `import a.b` to `a/b.lume`, loads the import
//! graph, rejects cycles, and returns the modules in dependency order
//! (entry last).
//!
//! Without a `lume.toml`, paths start at the folder of the entry file. In a
//! package they start at the package root, and an import whose first part
//! names a dependency reaches into that package — but only as far as its
//! `lib.lume` makes public.

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::{Import, Item};
use crate::error::LumeError;
use crate::fetch::{Fetcher, Update};
use crate::manifest::{self, Manifest};
use crate::{lexer, parser};

pub struct Module {
    /// Dotted id: "users.model"; the entry file is "main". A dependency's
    /// modules carry its name first: "tally", "tally.counts".
    pub id: String,
    pub path: PathBuf,
    pub src: String,
    pub items: Vec<Item>,
    pub imports: Vec<Resolved>,
    /// The package the file belongs to; `None` without a `lume.toml`.
    pub package: Option<String>,
    /// A module of a dependency, not of the package being built.
    pub from_dependency: bool,
}

#[derive(Clone)]
pub enum Resolved {
    /// `import users.model [as m]`
    Module { alias: String, id: String, line: usize, col: usize },
    /// `import users.model.User [as U]`
    Single { local: String, id: String, item: String, line: usize, col: usize },
}

impl Module {
    pub fn rust_mod(&self) -> String {
        // a dependency's modules sit apart, so its `util` and ours differ
        if self.from_dependency {
            format!("pkg_{}", self.id.replace('.', "_"))
        } else {
            self.id.replace('.', "_")
        }
    }
}

/// Everything a program is made of.
pub struct Loaded {
    pub modules: Vec<Module>,
    /// The root package first, then its dependencies; empty without a
    /// `lume.toml`.
    pub packages: Vec<Manifest>,
}

/// Renders an error against the file it belongs to.
fn render(e: &LumeError, path: &Path, src: &str) -> String {
    e.render(&path.display().to_string(), src)
}

fn parse_file(path: &Path) -> Result<(String, Vec<Item>), String> {
    let src = crate::diag::read_source(path).map_err(|e| format!("error: cannot read `{}`: {}", path.display(), e))?;
    let toks = lexer::lex(&src).map_err(|e| render(&e, path, &src))?;
    let items = parser::parse_program(toks).map_err(|e| render(&e, path, &src))?;
    Ok((src, items))
}

type Failure = (String, Option<String>);

/// Why an import failed: a message for the import's own line, or an error
/// already written against another file (one the import led to).
enum Fail {
    At(String, Option<String>),
    Rendered(String),
}

impl From<Failure> for Fail {
    fn from(f: Failure) -> Self {
        Fail::At(f.0, f.1)
    }
}

/// Where `import a.b.c` may point inside one folder: the module file
/// `a/b/c.lume`, or the item `c` of the module file `a/b.lume`. Ids are
/// given `prefix` first, and the file to load comes back too.
fn resolve_local(root: &Path, prefix: &str, imp: &Import, where_help: &str) -> Result<(Resolved, PathBuf), Failure> {
    let with_prefix = |segs: &[String]| if prefix.is_empty() { segs.join(".") } else { format!("{}.{}", prefix, segs.join(".")) };
    let as_module: PathBuf = root.join(format!("{}.lume", imp.path.join("/")));
    let alias_default = |segs: &[String]| segs.last().cloned().unwrap_or_default();
    if as_module.exists() {
        return Ok((
            Resolved::Module {
                alias: imp.alias.clone().unwrap_or_else(|| alias_default(&imp.path)),
                id: with_prefix(&imp.path),
                line: imp.line,
                col: imp.col,
            },
            as_module,
        ));
    }
    if imp.path.len() >= 2 {
        let module_segs = &imp.path[..imp.path.len() - 1];
        let as_item_module: PathBuf = root.join(format!("{}.lume", module_segs.join("/")));
        if as_item_module.exists() {
            let item = imp.path.last().unwrap().clone();
            return Ok((
                Resolved::Single {
                    local: imp.alias.clone().unwrap_or_else(|| item.clone()),
                    id: with_prefix(module_segs),
                    item,
                    line: imp.line,
                    col: imp.col,
                },
                as_item_module,
            ));
        }
        return Err((
            format!("no module `{}`: neither `{}` nor `{}` exists", imp.path.join("."), as_module.display(), as_item_module.display()),
            Some(where_help.into()),
        ));
    }
    Err((format!("no module `{}`: `{}` does not exist", imp.path.join("."), as_module.display()), Some(where_help.into())))
}

const SINGLE_FILE_HELP: &str = "module paths are relative to the directory of the file that holds `main`";
const PACKAGE_HELP: &str = "in a package, module paths start at the folder that holds `lume.toml`";

struct Loader {
    packages: Vec<Manifest>,
    by_name: HashMap<String, usize>,
    /// The folder paths start from without a `lume.toml`.
    single_root: PathBuf,
    loaded: HashMap<String, Module>,
    order: Vec<String>,
    visiting: Vec<String>,
}

/// Loads the entry file and everything it imports, transitively.
pub fn load(entry: &Path) -> Result<Loaded, String> {
    load_updating(entry, Update::No).map(|(l, _)| l)
}

/// Loads, moving the git dependencies `update` names to the commits their
/// `lume.toml` asks for now; writes `lume.lock`, and says which moved.
pub fn load_updating(entry: &Path, update: Update) -> Result<(Loaded, Vec<(String, Option<String>, String)>), String> {
    let dir = entry.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    let mut moved = Vec::new();
    let packages = match manifest::find(&dir) {
        Some(file) => {
            let root = manifest::read(&file)?;
            let root_dir = root.root.clone();
            let mut fetcher = Fetcher::new(&root_dir, update)?;
            let all = manifest::load_all(root, &mut fetcher)?;
            moved = fetcher.write_lock(&root_dir)?;
            all
        }
        None => Vec::new(),
    };
    let by_name = packages.iter().enumerate().map(|(i, m)| (m.name.clone(), i)).collect();
    let mut l = Loader { packages, by_name, single_root: dir, loaded: HashMap::new(), order: Vec::new(), visiting: Vec::new() };
    let root_pkg = l.packages.first().map(|m| m.name.clone());
    let stem = entry.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "main".into());
    // a package's `lib.lume` is known by the package's name wherever it is
    // loaded from, so running it and importing it are one module
    let entry_id = match (&root_pkg, stem.as_str()) {
        (Some(p), "lib") if l.is_lib(0, entry) => p.clone(),
        _ => stem,
    };
    l.load_one(root_pkg.as_deref(), entry, &entry_id)?;
    let mut out = Vec::new();
    for id in std::mem::take(&mut l.order) {
        if let Some(m) = l.loaded.remove(&id) {
            out.push(m);
        }
    }
    Ok((Loaded { modules: out, packages: l.packages }, moved))
}

impl Loader {
    fn root_of(&self, pkg: Option<&str>) -> PathBuf {
        match pkg {
            Some(p) => self.packages[self.by_name[p]].root.clone(),
            None => self.single_root.clone(),
        }
    }

    /// Ids of a package's modules start with its name, except the root
    /// package's, which stay as they are without a `lume.toml`.
    fn prefix_of(&self, pkg: Option<&str>) -> String {
        match pkg {
            Some(p) if self.by_name[p] != 0 => p.to_string(),
            _ => String::new(),
        }
    }

    fn lib_path(&self, k: usize) -> PathBuf {
        self.packages[k].root.join("lib.lume")
    }

    fn is_lib(&self, k: usize, path: &Path) -> bool {
        let a = std::fs::canonicalize(self.lib_path(k)).ok();
        a.is_some() && a == std::fs::canonicalize(path).ok()
    }

    fn load_one(&mut self, pkg: Option<&str>, path: &Path, id: &str) -> Result<(), String> {
        if self.loaded.contains_key(id) {
            return Ok(());
        }
        self.visiting.push(id.to_string());
        let (src, items) = parse_file(path)?;
        let mut imports = Vec::new();
        let mut seen_aliases: HashSet<String> = HashSet::new();
        for item in &items {
            if let Item::Import(imp) = item {
                if imp.is_rust {
                    continue;
                }
                let at = |msg: String, help: Option<String>| {
                    let mut e = LumeError::new(imp.line, imp.col, msg);
                    if let Some(h) = help {
                        e = e.with_help(h);
                    }
                    render(&e, path, &src)
                };
                let (resolved, to_load) = self.resolve(pkg, imp, id).map_err(|f| match f {
                    Fail::At(m, h) => at(m, h),
                    Fail::Rendered(e) => e,
                })?;
                let (dep_id, local) = match &resolved {
                    Resolved::Module { alias, id, .. } => (id.clone(), alias.clone()),
                    Resolved::Single { local, id, .. } => (id.clone(), local.clone()),
                };
                if !seen_aliases.insert(local.clone()) {
                    return Err(at(format!("`{}` is imported twice", local), Some("use `as` to give one of them another name".into())));
                }
                if dep_id == id {
                    return Err(at("a module cannot import itself".into(), None));
                }
                if self.visiting.contains(&dep_id) {
                    let cycle: Vec<String> = self.visiting.iter().skip_while(|v| **v != dep_id).cloned().chain([dep_id.clone()]).collect();
                    return Err(at(format!("circular import: {}", cycle.join(" -> ")), Some("move the shared definitions into a third module that both import".into())));
                }
                if let Some((dep_pkg, dep_path)) = to_load {
                    self.load_one(dep_pkg.as_deref(), &dep_path, &dep_id)?;
                }
                imports.push(resolved);
            }
        }
        self.visiting.pop();
        self.order.push(id.to_string());
        let from_dependency = pkg.map(|p| self.by_name[p] != 0).unwrap_or(false);
        let package = pkg.map(|p| p.to_string());
        self.loaded.insert(id.to_string(), Module { id: id.to_string(), path: path.to_path_buf(), src, items, imports, package, from_dependency });
        Ok(())
    }

    /// What one import means, and the file to load for it when it is not
    /// loaded already.
    #[allow(clippy::type_complexity)]
    fn resolve(&mut self, pkg: Option<&str>, imp: &Import, from_id: &str) -> Result<(Resolved, Option<(Option<String>, PathBuf)>), Fail> {
        let first = imp.path[0].clone();
        let root = self.root_of(pkg);
        if let Some(p) = pkg {
            let k = self.by_name[p];
            let is_dep = self.packages[k].deps.iter().any(|d| d.name == first);
            let is_self = first == p && self.lib_path(k).is_file();
            if is_dep || is_self {
                let (file, dir) = (root.join(format!("{}.lume", first)), root.join(&first));
                if file.exists() || dir.is_dir() {
                    let what = if is_dep { "a dependency" } else { "this package's name" };
                    let clash = if file.exists() { file } else { dir };
                    return Err(Fail::At(
                        format!("`{}` is both {} and a module of this package", first, what),
                        Some(format!("`import {}` always means the package, so the module can never be imported: rename `{}`", first, clash.display())),
                    ));
                }
                return self.resolve_in_package(&first, imp, from_id).map(|r| (r, None));
            }
            if imp.path.len() == 1 && first == "lib" && self.lib_path(k).is_file() {
                return Err(Fail::At(format!("import the library by its package's name: `import {}`", p), Some("`lib.lume` is the package itself, so it has the package's name".into())));
            }
            let mut prefix = self.prefix_of(pkg);
            // A module of the root package may share its name with a package
            // further down the tree (a dependency's dependency), as a Rust
            // crate may have a `mod` named like a crate it does not use. Its
            // id must still differ from that package's.
            if prefix.is_empty() && self.by_name.get(&first).map(|&j| j != 0).unwrap_or(false) {
                prefix = "self".to_string();
            }
            return match resolve_local(&root, &prefix, imp, PACKAGE_HELP) {
                Ok((r, file)) => Ok((r, Some((Some(p.to_string()), file)))),
                Err((msg, help)) => {
                    // a package further down the tree, not listed here
                    if let Some(&j) = self.by_name.get(&first) {
                        let whose: Vec<&str> = self.packages.iter().filter(|m| m.deps.iter().any(|d| d.name == first)).map(|m| m.name.as_str()).collect();
                        return Err(Fail::At(
                            format!("`{}` is not a dependency of `{}`", first, p),
                            Some(format!(
                                "`{}` is a dependency of `{}`; to use it here, add `{} = {{ path = \"…\" }}` under `[dependencies]` in `{}` (it is at `{}`)",
                                first,
                                whose.join("` and `"),
                                first,
                                self.packages[k].path.display(),
                                self.packages[j].root.display()
                            )),
                        ));
                    }
                    Err(Fail::At(msg, help))
                }
            };
        }
        resolve_local(&root, "", imp, SINGLE_FILE_HELP).map(|(r, file)| (r, Some((None, file)))).map_err(Fail::from)
    }

    /// `import tally`, `import tally.total`, `import tally.counts`: through
    /// the package's `lib.lume`, which says what the package offers.
    fn resolve_in_package(&mut self, name: &str, imp: &Import, from_id: &str) -> Result<Resolved, Fail> {
        let k = self.by_name[name];
        let lib = self.lib_path(k);
        if !lib.is_file() {
            return Err(Fail::At(
                format!("package `{}` has no `lib.lume`, so nothing can be imported from it", name),
                Some(format!("a library's root is `lib.lume` next to its `lume.toml`: `{}`", lib.display())),
            ));
        }
        if self.visiting.iter().any(|v| v == name) {
            let cycle: Vec<String> = self.visiting.iter().skip_while(|v| *v != name).cloned().chain([name.to_string()]).collect();
            return Err(Fail::At(format!("circular import: {}", cycle.join(" -> ")), Some("move the shared definitions into a third module that both import".into())));
        }
        if name == from_id {
            return Err(Fail::At("a module cannot import itself".into(), None));
        }
        let pkg = name.to_string();
        self.load_one(Some(&pkg), &lib, name).map_err(Fail::Rendered)?;
        let alias = |d: &str| imp.alias.clone().unwrap_or_else(|| d.to_string());
        let (line, col) = (imp.line, imp.col);
        if imp.path.len() == 1 {
            return Ok(Resolved::Module { alias: alias(name), id: name.to_string(), line, col });
        }
        let rest = &imp.path[1..];
        // what `lib.lume` passes on, by the name it passes it on under
        let libm = &self.loaded[name];
        // each by the name it is passed on under (`event`) and by the path
        // `lib.lume` wrote for it (`model.event`): either reaches it
        let mut passed: Vec<(Vec<String>, Resolved)> = Vec::new();
        for it in &libm.items {
            if let Item::Import(pi) = it {
                if pi.public && !pi.is_rust {
                    let n = pi.alias.clone().unwrap_or_else(|| pi.path.last().cloned().unwrap_or_default());
                    for r in &libm.imports {
                        let hit = match r {
                            Resolved::Module { alias, .. } => *alias == n,
                            Resolved::Single { local, .. } => *local == n,
                        };
                        if hit {
                            passed.push((vec![n.clone()], r.clone()));
                            if pi.alias.is_none() && pi.path.len() > 1 {
                                passed.push((pi.path.clone(), r.clone()));
                            }
                        }
                    }
                }
            }
        }
        for (path, r) in &passed {
            let n = path.len();
            if rest.len() < n || rest[..n] != path[..] {
                continue;
            }
            match (r, rest.len() - n) {
                (Resolved::Module { id, .. }, 0) => return Ok(Resolved::Module { alias: alias(&rest[n - 1]), id: id.clone(), line, col }),
                (Resolved::Module { id, .. }, 1) => return Ok(Resolved::Single { local: alias(&rest[n]), id: id.clone(), item: rest[n].clone(), line, col }),
                (Resolved::Single { id, item, .. }, 0) => return Ok(Resolved::Single { local: alias(&rest[n - 1]), id: id.clone(), item: item.clone(), line, col }),
                _ => {}
            }
        }
        let root = self.packages[k].root.clone();
        let is_module = root.join(format!("{}.lume", rest[0])).exists() || root.join(&rest[0]).is_dir();
        if is_module {
            return Err(Fail::At(
                format!("module `{}` of package `{}` is private", rest.join("."), name),
                Some(format!("only what `{}` makes public can be reached from outside the package; it would need `pub import {}`", lib.display(), rest.join("."))),
            ));
        }
        if rest.len() == 1 {
            // an item of `lib.lume`; the compiler checks it exists and is `pub`
            return Ok(Resolved::Single { local: alias(&rest[0]), id: name.to_string(), item: rest[0].clone(), line, col });
        }
        Err(Fail::At(
            format!("package `{}` has no public module `{}`", name, rest[..rest.len() - 1].join(".")),
            Some(format!("a package offers what its `lib.lume` defines with `pub` and passes on with `pub import`: see `{}`", lib.display())),
        ))
    }
}

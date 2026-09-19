//! Module loader: resolves `import a.b` to `a/b.lume` relative to the entry
//! file's directory, loads the import graph, rejects cycles, and returns the
//! modules in dependency order (entry last).

use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

use crate::ast::{Import, Item};
use crate::error::LumeError;
use crate::{lexer, parser};

pub struct Module {
    /// Dotted id: "users.model"; the entry file is "main".
    pub id: String,
    pub path: PathBuf,
    pub src: String,
    pub items: Vec<Item>,
    pub imports: Vec<Resolved>,
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
        self.id.replace('.', "_")
    }
}

/// Renders an error against the file it belongs to.
fn render(e: &LumeError, path: &Path, src: &str) -> String {
    e.render(&path.display().to_string(), src)
}

fn parse_file(path: &Path) -> Result<(String, Vec<Item>), String> {
    let src = std::fs::read_to_string(path).map_err(|e| format!("error: cannot read `{}`: {}", path.display(), e))?;
    let toks = lexer::lex(&src).map_err(|e| render(&e, path, &src))?;
    let items = parser::parse_program(toks).map_err(|e| render(&e, path, &src))?;
    Ok((src, items))
}

/// Where `import a.b.c` may point: the module file `a/b/c.lume`, or the
/// item `c` of the module file `a/b.lume`.
fn resolve_import(root: &Path, imp: &Import) -> Result<Resolved, (String, Option<String>)> {
    let as_module: PathBuf = root.join(format!("{}.lume", imp.path.join("/")));
    let alias_default = |segs: &[String]| segs.last().cloned().unwrap_or_default();
    if as_module.exists() {
        return Ok(Resolved::Module {
            alias: imp.alias.clone().unwrap_or_else(|| alias_default(&imp.path)),
            id: imp.path.join("."),
            line: imp.line,
            col: imp.col,
        });
    }
    if imp.path.len() >= 2 {
        let module_segs = &imp.path[..imp.path.len() - 1];
        let as_item_module: PathBuf = root.join(format!("{}.lume", module_segs.join("/")));
        if as_item_module.exists() {
            let item = imp.path.last().unwrap().clone();
            return Ok(Resolved::Single {
                local: imp.alias.clone().unwrap_or_else(|| item.clone()),
                id: module_segs.join("."),
                item,
                line: imp.line,
                col: imp.col,
            });
        }
        return Err((
            format!("no module `{}`: neither `{}` nor `{}` exists", imp.path.join("."), as_module.display(), as_item_module.display()),
            Some("module paths are relative to the directory of the file that holds `main`".into()),
        ));
    }
    Err((
        format!("no module `{}`: `{}` does not exist", imp.path.join("."), as_module.display()),
        Some("module paths are relative to the directory of the file that holds `main`".into()),
    ))
}

/// Loads the entry file and everything it imports, transitively.
pub fn load(entry: &Path) -> Result<Vec<Module>, String> {
    let root = entry.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    let mut loaded: HashMap<String, Module> = HashMap::new();
    let mut order: Vec<String> = Vec::new();
    let mut visiting: Vec<String> = Vec::new();
    let entry_id = entry.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or_else(|| "main".into());
    load_one(&root, entry, &entry_id, &mut loaded, &mut order, &mut visiting)?;
    let mut out = Vec::new();
    for id in order {
        if let Some(m) = loaded.remove(&id) {
            out.push(m);
        }
    }
    Ok(out)
}

fn load_one(
    root: &Path,
    path: &Path,
    id: &str,
    loaded: &mut HashMap<String, Module>,
    order: &mut Vec<String>,
    visiting: &mut Vec<String>,
) -> Result<(), String> {
    if loaded.contains_key(id) {
        return Ok(());
    }
    visiting.push(id.to_string());
    let (src, items) = parse_file(path)?;
    let mut imports = Vec::new();
    let mut seen_aliases: HashSet<String> = HashSet::new();
    for item in &items {
        if let Item::Import(imp) = item {
            if imp.is_rust {
                continue;
            }
            let resolved = resolve_import(root, imp).map_err(|(msg, help)| {
                let mut e = LumeError::new(imp.line, imp.col, msg);
                if let Some(h) = help {
                    e = e.with_help(h);
                }
                render(&e, path, &src)
            })?;
            let (dep_id, local, line, col) = match &resolved {
                Resolved::Module { alias, id, line, col } => (id.clone(), alias.clone(), *line, *col),
                Resolved::Single { local, id, line, col, .. } => (id.clone(), local.clone(), *line, *col),
            };
            if !seen_aliases.insert(local.clone()) {
                let e = LumeError::new(line, col, format!("`{}` is imported twice", local)).with_help("use `as` to give one of them another name");
                return Err(render(&e, path, &src));
            }
            if dep_id == id {
                let e = LumeError::new(line, col, "a module cannot import itself");
                return Err(render(&e, path, &src));
            }
            if visiting.contains(&dep_id) {
                let cycle: Vec<String> = visiting.iter().skip_while(|v| **v != dep_id).cloned().chain([dep_id.clone()]).collect();
                let e = LumeError::new(line, col, format!("circular import: {}", cycle.join(" -> ")))
                    .with_help("move the shared definitions into a third module that both import");
                return Err(render(&e, path, &src));
            }
            let dep_path = root.join(format!("{}.lume", dep_id.replace('.', "/")));
            load_one(root, &dep_path, &dep_id, loaded, order, visiting)?;
            imports.push(resolved);
        }
    }
    visiting.pop();
    order.push(id.to_string());
    loaded.insert(id.to_string(), Module { id: id.to_string(), path: path.to_path_buf(), src, items, imports });
    Ok(())
}

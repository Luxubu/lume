//! `lume` command-line tool.
//!
//!   lume build <file.lume> [-o <binary>]   compile to a native binary
//!   lume run   <file.lume> [-- args...]    compile and run
//!   lume emit  <file.lume>                 print the generated Rust
//!   lume check <file.lume>                 parse and check, emit nothing
//!   lume test  <file.lume>                 build and run the `test` blocks
//!   lume fmt   <file.lume> [--check|--stdout]  rewrite in the canonical layout
//!   lume crate <file.lume> <crate>         what a crate offers, in Lume types
//!   lume clean <file.lume>                 remove the program's build directory
//!   lume new   <name> [--lib]              make a package
//!
//! In a package (a folder with `lume.toml`) the file may be left out.
//!
//! Builds go to `.lume/` next to the source file. Crates and async programs
//! are built through cargo into one shared cache for the whole machine
//! (`~/.cache/lume/target`, or `$LUME_CACHE_DIR/target`).

mod ast;
mod bridge;
mod codegen;
mod error;
mod fmt;
mod lexer;
mod loader;
mod manifest;
mod parser;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::time::Instant;

fn help_text() -> String {
    format!(
        "lume {}\n\nusage:\n  \
         lume run   <file.lume> [-- <args>...]   compile and run it\n  \
         lume build <file.lume> [-o <binary>]    compile it, fully optimised\n  \
         lume test  <file.lume>                  run the file's `test` blocks\n  \
         lume check <file.lume>                  parse and type-check only\n  \
         lume fmt   <file.lume> [--check | --stdout]   rewrite in the canonical layout\n  \
         lume emit  <file.lume>                  print the generated Rust\n  \
         lume crate <file.lume> <crate>          what a Rust crate offers, in Lume types\n  \
         lume clean <file.lume> [--cache]        remove build output\n  \
         lume new   <name> [--lib]               make a package: a folder with lume.toml\n\n\
         In a package, leave out the file: the command uses the package the\n\
         current folder is in (main.lume, or lib.lume for test and check).\n\n\
         docs: docs/README.md   a tour: docs/tour.md",
        env!("CARGO_PKG_VERSION")
    )
}

fn usage() -> ! {
    eprintln!("{}", help_text());
    process::exit(2);
}

struct Compiled {
    rust: String,
    deps: Vec<(String, String)>,
    has_rust_blocks: bool,
}

fn rustc_failed_banner(has_rust_blocks: bool, file: &Path, tool: &str, err: &str) -> String {
    let why = if has_rust_blocks {
        "This program has `rust:` blocks; if the message below points inside one, the problem is in that Rust code. Otherwise it is a gap in the Lume compiler."
    } else {
        "This is a gap in the Lume compiler, not necessarily in your program."
    };
    format!("error: the generated Rust did not compile. {}\n  generated file: {}\n\n{} said:\n{}", why, file.display(), tool, err)
}

/// Loads the entry file and its imports, compiles each module in dependency
/// order, and concatenates the Rust: imported modules as `mod` blocks, then
/// the entry file's items.
fn compile_to_rust(path: &Path, test_mode: bool) -> Result<Compiled, String> {
    let loaded = loader::load(path)?;
    let modules = loaded.modules;
    let crate_imports = crate_requirements(&modules, &loaded.packages)?;
    let mut any_async = false;
    for m in &modules {
        any_async |= m.src.contains("async ") || m.src.contains("spawn:") || m.src.contains("await ");
    }
    let package_of: std::collections::HashMap<String, String> = modules.iter().filter_map(|m| m.package.clone().map(|p| (m.id.clone(), p))).collect();
    let mut crate_infos: std::collections::HashMap<(String, String), bridge::CrateInfo> = std::collections::HashMap::new();
    if !crate_imports.is_empty() {
        let mut pre_deps: Vec<(String, String)> = crate_imports.iter().map(|(k, v, _)| (k.clone(), v.clone())).collect();
        pre_deps.dedup();
        if any_async {
            pre_deps.push(tokio_dep());
        }
        let proj = write_cargo_project(path, &pre_deps)?;
        for (k, _, alias) in &crate_imports {
            let key = (k.clone(), alias.clone());
            if !crate_infos.contains_key(&key) {
                let info = bridge::load_crate(&proj, k, alias, &shared_target_dir())?;
                crate_infos.insert(key, info);
            }
        }
    }
    // `pub import seq` in a module means its own importers get `seq` too,
    // under the name it used. Keyed by module id.
    // name, module id, and the single item when it is `pub import a.b.Name`
    type ReExport = (String, String, Option<String>);
    let mut reexports: std::collections::HashMap<String, Vec<ReExport>> = std::collections::HashMap::new();
    for m in &modules {
        let mut out: Vec<ReExport> = Vec::new();
        for imp in m.items.iter().filter_map(|it| match it {
            ast::Item::Import(imp) if imp.public && !imp.is_rust => Some(imp),
            _ => None,
        }) {
            let name = imp.alias.clone().unwrap_or_else(|| imp.path.last().cloned().unwrap_or_default());
            for res in &m.imports {
                match res {
                    loader::Resolved::Module { alias, id, .. } if *alias == name => out.push((name.clone(), id.clone(), None)),
                    loader::Resolved::Single { local, id, item, .. } if *local == name => out.push((name.clone(), id.clone(), Some(item.clone()))),
                    _ => {}
                }
            }
        }
        if !out.is_empty() {
            reexports.insert(m.id.clone(), out);
        }
    }
    let mut exports: std::collections::HashMap<String, codegen::Exports> = std::collections::HashMap::new();
    let mut rust = String::new();
    let mut deps: Vec<(String, String)> = Vec::new();
    let mut has_rust_blocks = false;
    let n = modules.len();
    for (i, m) in modules.iter().enumerate() {
        let is_entry = i + 1 == n;
        let file = m.path.display().to_string();
        let mut dep_list: Vec<codegen::Dep> = m
            .imports
            .iter()
            .map(|r| match r {
                loader::Resolved::Module { alias, id, line, col } => codegen::Dep::Module { alias: alias.clone(), id: id.clone(), exports: &exports[id], line: *line, col: *col },
                loader::Resolved::Single { local, id, item, line, col } => codegen::Dep::Single { local: local.clone(), id: id.clone(), item: item.clone(), exports: &exports[id], line: *line, col: *col },
            })
            .collect();
        // a module this one imports may pass others on with `pub import`
        {
            let mut seen: Vec<String> = dep_list
                .iter()
                .filter_map(|d| match d {
                    codegen::Dep::Module { alias, .. } => Some(alias.clone()),
                    _ => None,
                })
                .collect();
            // each entry keeps the import in this file that led to it, so
            // an error points at a line someone can act on
            let mut queue: Vec<(String, usize, usize)> = m
                .imports
                .iter()
                .map(|r| match r {
                    loader::Resolved::Module { id, line, col, .. } | loader::Resolved::Single { id, line, col, .. } => (id.clone(), *line, *col),
                })
                .collect();
            let mut extra: Vec<ReExport> = Vec::new();
            // where each re-exported name came from, so two modules passing
            // on different things under one name is caught rather than
            // silently resolved to whichever arrived last
            let mut origin: std::collections::HashMap<String, (String, String)> = std::collections::HashMap::new();
            while let Some((id, at_line, at_col)) = queue.pop() {
                for (name, rid, item) in reexports.get(&id).map(|v| v.as_slice()).unwrap_or(&[]) {
                    if let Some((other_id, via)) = origin.get(name) {
                        if other_id != rid {
                            let e = error::LumeError::new(at_line, at_col, format!("`{}` is passed on by two modules and means something different in each", name))
                                .with_help(format!("`{}` passes on `{}` and `{}` passes on `{}`; import the one you want directly, or have them agree on a name", via, other_id, id, rid));
                            return Err(e.render(&file, &m.src));
                        }
                        continue;
                    }
                    if seen.contains(name) {
                        continue;
                    }
                    seen.push(name.clone());
                    origin.insert(name.clone(), (rid.clone(), id.clone()));
                    extra.push((name.clone(), rid.clone(), item.clone()));
                    queue.push((rid.clone(), at_line, at_col));
                }
            }
            for (name, rid, item) in extra {
                if let Some(ex) = exports.get(&rid) {
                    match item {
                        Some(it) => dep_list.push(codegen::Dep::Single { local: name, id: rid, item: it, exports: ex, line: 0, col: 0 }),
                        None => dep_list.push(codegen::Dep::Module { alias: name, id: rid, exports: ex, line: 0, col: 0 }),
                    }
                }
            }
        }
        for item in &m.items {
            if let ast::Item::Import(imp) = item {
                if imp.is_rust {
                    let alias = imp.alias.clone().unwrap_or_else(|| imp.krate().to_string());
                    if let Some(info) = crate_infos.get(&(imp.krate().to_string(), alias.clone())) {
                        dep_list.push(codegen::Dep::Rust { alias, info });
                    }
                }
            }
        }
        if is_entry {
            for info in crate_infos.values() {
                dep_list.push(codegen::Dep::RustTypes { info });
            }
        }
        let rust_mod = if is_entry { None } else { Some(m.rust_mod()) };
        let (out, ex) = codegen::generate_module(&m.items, rust_mod.as_deref(), &m.id, &package_of, &dep_list, test_mode, &m.src, &file).map_err(|e| e.render(&file, &m.src))?;
        for w in out.warnings {
            eprint!("{}", w.render(&file, &m.src).replacen("error:", "warning:", 1));
        }
        if is_entry {
            // the entry carries the prelude, so it goes first; modules follow
            rust = format!("{}\n{}", out.rust, rust);
        } else {
            rust.push_str(&out.rust);
            rust.push('\n');
        }
        for mut d in out.deps {
            // in a package the version is `lume.toml`'s, not the file's
            if let Some((_, v, _)) = crate_imports.iter().find(|(k, v, _)| *k == d.0 && !v.is_empty()) {
                d.1 = v.clone();
            }
            if !deps.contains(&d) {
                deps.push(d);
            }
        }
        has_rust_blocks |= out.has_rust_blocks;
        exports.insert(m.id.clone(), ex);
    }
    if test_mode {
        // a package's tests, not its dependencies', as `cargo test` does
        let own: Vec<&loader::Module> = modules.iter().filter(|m| !m.from_dependency).collect();
        rust.push_str(&test_runner(&own));
    }
    Ok(Compiled { rust, deps, has_rust_blocks })
}

/// Every `import rust.x` in the program, as (crate, requirement, alias).
/// Without a `lume.toml` the file names the version. In a package the
/// version lives in `[rust]` of `lume.toml`, as in `Cargo.toml`, and two
/// packages asking for one crate must agree on a version Cargo can give both.
fn crate_requirements(modules: &[loader::Module], packages: &[manifest::Manifest]) -> Result<Vec<(String, String, String)>, String> {
    let mut out: Vec<(String, String, String)> = Vec::new();
    // crate -> (requirement, package, lume.toml line) of the first package asking
    let mut chosen: std::collections::HashMap<String, (String, usize, usize)> = std::collections::HashMap::new();
    for m in modules {
        let pkg = m.package.as_ref().and_then(|p| packages.iter().position(|x| x.name == *p));
        for item in &m.items {
            let imp = match item {
                ast::Item::Import(imp) if imp.is_rust => imp,
                _ => continue,
            };
            let krate = imp.krate().to_string();
            let alias = imp.alias.clone().unwrap_or_else(|| krate.clone());
            let k = match pkg {
                None => {
                    out.push((krate, imp.version.clone().unwrap_or_else(|| "*".into()), alias));
                    continue;
                }
                Some(k) => k,
            };
            let man = &packages[k];
            let file = m.path.display().to_string();
            if let Some(v) = &imp.version {
                let e = error::LumeError::new(imp.line, imp.col, format!("in a package, the version of `{}` goes in `lume.toml`", krate))
                    .with_help(format!("write `import rust.{}` here, and `{} = \"{}\"` under `[rust]` in `{}`", krate, krate, v, man.path.display()));
                return Err(e.render(&file, &m.src));
            }
            let (req, line) = match man.rust.iter().find(|(c, _, _)| *c == krate) {
                Some((_, r, l)) => (r.clone(), *l),
                None => {
                    let e = error::LumeError::new(imp.line, imp.col, format!("crate `{}` is not in `[rust]` of `{}`", krate, man.path.display()))
                        .with_help(format!("add `{} = \"<version>\"` under `[rust]` there", krate));
                    return Err(e.render(&file, &m.src));
                }
            };
            match chosen.get(&krate).cloned() {
                None => {
                    chosen.insert(krate.clone(), (req.clone(), k, line));
                }
                Some((prev, pk, pline)) if prev != req => match merge_requirements(&prev, &req) {
                    Some(r) => {
                        let (pk2, l2) = if r == req { (k, line) } else { (pk, pline) };
                        chosen.insert(krate.clone(), (r, pk2, l2));
                    }
                    None => {
                        let e = error::LumeError::new(line, 1, format!("`{}` asks for crate `{}` at \"{}\", and `{}` at \"{}\"", man.name, krate, req, packages[pk].name, prev))
                            .with_help("a program is one Rust crate, so it has one version of each crate: make the two requirements ones Cargo can meet together, such as \"1\" and \"1.5\"");
                        return Err(e.render(&man.path.display().to_string(), &man.src));
                    }
                },
                _ => {}
            }
            out.push((krate, String::new(), alias));
        }
    }
    for (k, v, _) in out.iter_mut() {
        if let Some((r, _, _)) = chosen.get(k) {
            *v = r.clone();
        }
    }
    Ok(out)
}

/// Two semver requirements Cargo would meet with one version (`"1"` and
/// `"1.5"`: both allow 1.5.x), as the tighter of them; `None` when they
/// cannot share one, or are not plain versions.
fn merge_requirements(a: &str, b: &str) -> Option<String> {
    fn parse(r: &str) -> Option<Vec<u64>> {
        let v = r.trim().trim_start_matches('^');
        if v.is_empty() || v.starts_with('{') {
            return None;
        }
        v.split('.').map(|p| p.parse::<u64>().ok()).collect()
    }
    // semver-compatible: the same numbers up to and including the first
    // that is not zero
    fn compat(v: &[u64]) -> Vec<u64> {
        let mut out = Vec::new();
        for &n in v {
            out.push(n);
            if n != 0 {
                break;
            }
        }
        out
    }
    let (x, y) = (parse(a)?, parse(b)?);
    let (cx, cy) = (compat(&x), compat(&y));
    let n = cx.len().min(cy.len());
    if cx[..n] != cy[..n] || (cx.last() != Some(&0) && cy.last() != Some(&0) && cx.len() != cy.len()) {
        return None;
    }
    let pad = |v: &[u64]| (0..3).map(|i| v.get(i).copied().unwrap_or(0)).collect::<Vec<_>>();
    Some(if pad(&x) >= pad(&y) { a.to_string() } else { b.to_string() })
}

/// `fn main` for `lume test`: runs every `test` block of every module,
/// catching failures so the rest still run, and reports a summary.
fn test_runner(modules: &[&loader::Module]) -> String {
    let n = modules.len();
    let mut entries = Vec::new();
    for (i, m) in modules.iter().enumerate() {
        let is_entry = i + 1 == n;
        let mut k = 0usize;
        for item in &m.items {
            if let ast::Item::Test(t) = item {
                let path = if is_entry { format!("lume_test_{}", k) } else { format!("{}::lume_test_{}", m.rust_mod(), k) };
                let label = if is_entry { t.name.clone() } else { format!("{}: {}", m.id, t.name) };
                let file = m.path.file_name().map(|f| f.to_string_lossy().to_string()).unwrap_or_default();
                entries.push(format!("(\"{}\", \"{}\", {} as fn())", label.replace('\\', "\\\\").replace('"', "\\\""), file, path));
                k += 1;
            }
        }
    }
    format!(
        r#"
fn main() {{
    let tests: Vec<(&str, &str, fn())> = vec![{entries}];
    static LAST: std::sync::Mutex<Option<String>> = std::sync::Mutex::new(None);
    std::panic::set_hook(Box::new(|info| {{
        let msg = if let Some(a) = info.payload().downcast_ref::<LumeAssert>() {{
            a.0.clone()
        }} else if let Some(s) = info.payload().downcast_ref::<String>() {{
            format!(": stopped: {{}}", s)
        }} else if let Some(s) = info.payload().downcast_ref::<&str>() {{
            format!(": stopped: {{}}", s)
        }} else {{
            String::from(": stopped")
        }};
        *LAST.lock().unwrap() = Some(msg);
    }}));
    let mut failed = 0usize;
    for (name, file, f) in &tests {{
        print!("test {{}} ... ", name);
        use std::io::Write;
        let _ = std::io::stdout().flush();
        match std::panic::catch_unwind(f) {{
            Ok(()) => println!("ok"),
            Err(_) => {{
                failed += 1;
                println!("FAILED");
                let msg = LAST.lock().unwrap().take().unwrap_or_default();
                for (i, l) in msg.lines().enumerate() {{
                    if i == 0 {{
                        println!("    {{}}{{}}", file, l);
                    }} else {{
                        println!("    {{}}", l);
                    }}
                }}
            }}
        }}
    }}
    let _ = std::panic::take_hook();
    let total = tests.len();
    if total == 0 {{
        println!("no tests");
    }} else {{
        println!();
        println!("{{}} test{{}}: {{}} passed, {{}} failed", total, if total == 1 {{ "" }} else {{ "s" }}, total - failed, failed);
    }}
    if failed > 0 {{
        std::process::exit(1);
    }}
}}
"#,
        entries = entries.join(", ")
    )
}

fn tokio_dep() -> (String, String) {
    ("tokio".to_string(), "{ version = \"1\", features = [\"rt-multi-thread\", \"macros\", \"time\", \"sync\"] }".to_string())
}

/// The cargo project for a program: `.lume/cargo-<stem>/` next to it.
fn cargo_project_dir(path: &Path) -> (String, PathBuf) {
    let stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or("out".into());
    let dir = path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    (stem.clone(), dir.join(".lume").join(format!("cargo-{}", stem)))
}

/// Writes the project's Cargo.toml (only when it changed, so cached crate
/// signatures stay valid) and makes sure a main.rs exists.
fn write_cargo_project(path: &Path, deps: &[(String, String)]) -> Result<PathBuf, String> {
    let (stem, proj) = cargo_project_dir(path);
    let src_dir = proj.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("error: cannot create `{}`: {}", src_dir.display(), e))?;
    let pkg = stem.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
    let mut toml = format!("[package]\nname = \"{}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[dependencies]\n", pkg);
    for (k, v) in deps {
        // a version, or a whole inline table such as `{ version = "1", features = [...] }`
        if v.trim_start().starts_with('{') {
            toml.push_str(&format!("{} = {}\n", k, v));
        } else {
            toml.push_str(&format!("{} = \"{}\"\n", k, v));
        }
    }
    // `release` is what `lume run` uses: the program at opt-level 1 with an
    // incremental cache, every crate at opt-level 3. `ship` is `lume build`.
    toml.push_str("\n[profile.release]\nopt-level = 1\ndebug = false\nincremental = true\noverflow-checks = true\n\n[profile.release.package.\"*\"]\nopt-level = 3\noverflow-checks = false\n\n[profile.ship]\ninherits = \"release\"\nopt-level = 3\nincremental = false\n");
    let toml_path = proj.join("Cargo.toml");
    if fs::read_to_string(&toml_path).ok().as_deref() != Some(toml.as_str()) {
        fs::write(&toml_path, toml).map_err(|e| format!("error: cannot write Cargo.toml: {}", e))?;
    }
    let main_rs = src_dir.join("main.rs");
    if !main_rs.exists() {
        fs::write(&main_rs, "fn main() {}\n").map_err(|e| format!("error: cannot write main.rs: {}", e))?;
    }
    Ok(proj)
}

/// One build directory for every Lume program on the machine, so a crate
/// compiled for one program serves the next. `LUME_CACHE_DIR` overrides it.
fn shared_target_dir() -> PathBuf {
    if let Ok(d) = env::var("LUME_CACHE_DIR") {
        return PathBuf::from(d).join("target");
    }
    let home = env::var("XDG_CACHE_HOME").map(PathBuf::from).or_else(|_| env::var("HOME").map(|h| PathBuf::from(h).join(".cache"))).unwrap_or_else(|_| PathBuf::from(".lume-cache"));
    home.join("lume").join("target")
}

/// A cargo command inside a program's project, building into the shared cache.
fn cargo_in(proj: &Path) -> Command {
    let mut c = Command::new("cargo");
    c.current_dir(proj).env("CARGO_TARGET_DIR", shared_target_dir());
    c
}

/// Builds through cargo when the program imports Rust crates or uses async.
fn build_with_cargo(path: &Path, rust: &str, deps: &[(String, String)], bin: &Path, has_rust_blocks: bool, mode: Mode) -> Result<(), String> {
    let (stem, _) = cargo_project_dir(path);
    let proj = write_cargo_project(path, deps)?;
    let src_dir = proj.join("src");
    let pkg = stem.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
    fs::write(src_dir.join("main.rs"), rust).map_err(|e| format!("error: cannot write main.rs: {}", e))?;
    let profile = if mode == Mode::Ship { "ship" } else { "release" };
    let out = cargo_in(&proj)
        .args(["build", "--profile", profile, "-q"])
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("error: cannot run cargo: {}\n  help: install Rust from https://rustup.rs", e))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(rustc_failed_banner(has_rust_blocks, &src_dir.join("main.rs"), "cargo", &err));
    }
    let built = shared_target_dir().join(profile).join(&pkg);
    fs::copy(&built, bin).map_err(|e| format!("error: cannot copy `{}` to `{}`: {}", built.display(), bin.display(), e))?;
    Ok(())
}

/// `Iterate` is `lume run`/`lume test`: the fastest turnaround that still
/// runs at full speed (rustc keeps an incremental cache; through cargo the
/// program crate is built at opt-level 1 on top of fully optimised crates).
/// `Ship` is `lume build`: everything at opt-level 3, no incremental state.
#[derive(Clone, Copy, PartialEq)]
enum Mode {
    Iterate,
    Ship,
}

/// A small stable hash of the generated program, to skip compiling when
/// nothing changed since the binary was made.
fn fingerprint(text: &str) -> String {
    use std::hash::{Hash, Hasher};
    let mut h = std::collections::hash_map::DefaultHasher::new();
    text.hash(&mut h);
    format!("{:016x}", h.finish())
}

fn build(path: &Path, out: Option<PathBuf>, quiet: bool, test_mode: bool, mode: Mode) -> Result<PathBuf, String> {
    let t0 = Instant::now();
    let compiled = compile_to_rust(path, test_mode)?;
    let rust = compiled.rust;
    let mut stem = path.file_stem().map(|s| s.to_string_lossy().to_string()).unwrap_or("out".into());
    if test_mode {
        stem.push_str("-test");
    }
    let dir = path.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from("."));
    let build_dir = dir.join(".lume");
    fs::create_dir_all(&build_dir).map_err(|e| format!("error: cannot create `{}`: {}", build_dir.display(), e))?;
    let rs_path = build_dir.join(format!("{}.rs", stem));
    let header = format!("// Generated by lume from {} — edits here are overwritten.\n", path.display());
    let rust = format!("{}{}", header, rust);
    fs::write(&rs_path, &rust).map_err(|e| format!("error: cannot write `{}`: {}", rs_path.display(), e))?;
    let bin = out.unwrap_or_else(|| build_dir.join(&stem));

    // Nothing changed since the last build of this program in this mode? Run what we have.
    let stamp_path = build_dir.join(format!("{}.stamp", stem));
    let stamp = format!("{} {} {:?}\n", fingerprint(&rust), fingerprint(&format!("{:?}", compiled.deps)), mode == Mode::Ship);
    if bin.exists() && fs::read_to_string(&stamp_path).ok().as_deref() == Some(stamp.as_str()) {
        if !quiet {
            eprintln!("up to date: {}", bin.display());
        }
        return Ok(bin);
    }
    let _ = fs::remove_file(&stamp_path);

    if !compiled.deps.is_empty() {
        build_with_cargo(path, &rust, &compiled.deps, &bin, compiled.has_rust_blocks, mode)?;
        let _ = fs::write(&stamp_path, &stamp);
        if !quiet {
            eprintln!("compiled {} -> {} in {:.2}s (cargo, {} crate{})", path.display(), bin.display(), t0.elapsed().as_secs_f64(), compiled.deps.len(), if compiled.deps.len() == 1 { "" } else { "s" });
        }
        return Ok(bin);
    }

    let mut cmd = Command::new("rustc");
    // overflow checks stay on in every mode: Int arithmetic that overflows stops the program
    cmd.args(["--edition", "2021", "-O", "-C", "debuginfo=0", "-C", "overflow-checks=on"]);
    if mode == Mode::Iterate {
        // rustc's incremental cache: a rebuild after an edit takes a fraction of a fresh compile
        let inc = build_dir.join(format!("inc-{}", stem));
        cmd.arg("-C").arg(format!("incremental={}", inc.display()));
    }
    let status = cmd
        .arg("-o")
        .arg(&bin)
        .arg(&rs_path)
        .stdout(Stdio::inherit())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("error: cannot run rustc: {}\n  help: install Rust from https://rustup.rs", e))?;
    if !status.status.success() {
        let err = String::from_utf8_lossy(&status.stderr);
        return Err(rustc_failed_banner(compiled.has_rust_blocks, &rs_path, "rustc", &err));
    }
    let _ = fs::write(&stamp_path, &stamp);
    if !quiet {
        eprintln!("compiled {} -> {} in {:.2}s", path.display(), bin.display(), t0.elapsed().as_secs_f64());
    }
    Ok(bin)
}

/// Formats one file; false when it could not, or `--check` found it
/// needs formatting.
fn fmt_file(file: &Path, mode: &str) -> bool {
    let src = match fs::read_to_string(file) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("error: cannot read `{}`: {}", file.display(), e);
            return false;
        }
    };
    let formatted = match fmt::format_source(&src) {
        Ok(f) => f,
        Err(e) => {
            eprint!("{}", e.render(&file.display().to_string(), &src));
            return false;
        }
    };
    match mode {
        "--stdout" => print!("{}", formatted),
        "--check" => {
            if formatted != src {
                eprintln!("would reformat {}", file.display());
                return false;
            }
        }
        _ => {
            if formatted != src {
                if let Err(e) = fs::write(file, &formatted) {
                    eprintln!("error: cannot write `{}`: {}", file.display(), e);
                    return false;
                }
                eprintln!("formatted {}", file.display());
            }
        }
    }
    true
}

/// Every `.lume` file of the package at `dir`: not build output, and not a
/// package kept inside it (a folder with a `lume.toml` of its own).
fn package_files(dir: &Path, out: &mut Vec<PathBuf>) {
    // the package in the current folder has the root `` (so paths print as `main.lume`)
    let entries = match fs::read_dir(if dir.as_os_str().is_empty() { Path::new(".") } else { dir }) {
        Ok(e) => e,
        Err(_) => return,
    };
    for e in entries.flatten() {
        let p = dir.join(e.file_name());
        let name = e.file_name().to_string_lossy().to_string();
        if p.is_dir() {
            if !name.starts_with('.') && !p.join(manifest::FILE).exists() {
                package_files(&p, out);
            }
        } else if p.extension().map(|x| x == "lume").unwrap_or(false) {
            out.push(p);
        }
    }
}

/// `lume run` and the rest with no file: the package the current folder is
/// in, and the file of it the command means.
fn package_entry(cmd: &str) -> Result<(PathBuf, PathBuf), String> {
    let toml = match manifest::find(Path::new(".")) {
        Some(t) => t,
        None => {
            return Err(format!(
                "error: `lume {}` needs a file, or a package: no `lume.toml` here or in a folder above\n  help: name the file, as in `lume {} main.lume`, or make a package with `lume new <name>`\n",
                cmd, cmd
            ))
        }
    };
    let m = manifest::read(&toml)?;
    let (main, lib) = (m.root.join("main.lume"), m.root.join("lib.lume"));
    let pick = match cmd {
        // a library is imported, not run
        "run" | "build" => main.is_file().then_some(main),
        _ => [main, lib].into_iter().find(|f| f.is_file()),
    };
    match pick {
        Some(f) => Ok((f, m.root.clone())),
        None if cmd == "fmt" || cmd == "clean" => Ok((m.root.join("main.lume"), m.root.clone())),
        None if cmd == "run" || cmd == "build" => Err(format!(
            "error: package `{}` has no `main.lume` to {}\n  help: a library is imported by other packages, not run; `lume test` runs its tests\n",
            m.name, cmd
        )),
        None => Err(format!("error: package `{}` has neither `main.lume` nor `lib.lume`\n  help: a program starts at `main.lume`, a library at `lib.lume`, both next to `lume.toml`\n", m.name)),
    }
}

/// `lume new <name> [--lib]`: a folder with a `lume.toml` and a first file.
fn new_package(args: &[String]) -> Result<(), String> {
    let mut name = None;
    let mut lib = false;
    for a in args {
        match a.as_str() {
            "--lib" => lib = true,
            _ if name.is_none() && !a.starts_with('-') => name = Some(a.clone()),
            _ => return Err("usage: lume new <name> [--lib]\n".into()),
        }
    }
    let path = PathBuf::from(name.ok_or("usage: lume new <name> [--lib]\n")?);
    let pkg = path.file_name().map(|n| n.to_string_lossy().to_string()).unwrap_or_default();
    if let Some(m) = manifest::check_name(&pkg) {
        return Err(format!("error: {}\n", m));
    }
    if path.exists() {
        return Err(format!("error: `{}` already exists\n", path.display()));
    }
    fs::create_dir_all(&path).map_err(|e| format!("error: cannot create `{}`: {}\n", path.display(), e))?;
    let toml = format!("[package]\nname = \"{}\"\nversion = \"0.1.0\"\n\n[dependencies]\n", pkg);
    let (file, body) = if lib {
        ("lib.lume", format!("# The package `{p}`: what is `pub` here, other packages can use.\n\npub def greeting(name: Str) -> Str = \"Hello, #{{name}}!\"\n\ntest \"greeting\":\n  assert greeting(\"{p}\") == \"Hello, {p}!\"\n", p = pkg))
    } else {
        ("main.lume", "def main:\n  puts \"Hello, world!\"\n".to_string())
    };
    for (f, text) in [(manifest::FILE, toml), (file, body)] {
        fs::write(path.join(f), text).map_err(|e| format!("error: cannot write `{}`: {}\n", path.join(f).display(), e))?;
    }
    eprintln!("made {} `{}` in {}", if lib { "library" } else { "program" }, pkg, path.display());
    Ok(())
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }
    // The two flags everyone types first. Asking for help is not a mistake,
    // so it goes to stdout and exits 0.
    match args[0].as_str() {
        "--version" | "-V" | "version" => {
            println!("lume {}", env!("CARGO_PKG_VERSION"));
            process::exit(0);
        }
        "--help" | "-h" | "help" => {
            println!("{}", help_text());
            process::exit(0);
        }
        _ => {}
    }
    let cmd = args[0].as_str();
    if cmd == "new" {
        if let Err(e) = new_package(&args[1..]) {
            eprint!("{}", e);
            process::exit(1);
        }
        return;
    }
    // A file, or — with none — the package the current folder is in.
    let named = args.get(1).filter(|f| !f.starts_with('-') && !(cmd == "crate" && !f.ends_with(".lume") && args.len() == 2));
    let (file, pkg_root, args): (PathBuf, Option<PathBuf>, Vec<String>) = match named {
        Some(f) => {
            let file = PathBuf::from(f);
            if file.extension().map(|e| e != "lume").unwrap_or(true) {
                eprintln!("error: `{}` is not a .lume file", file.display());
                process::exit(2);
            }
            (file, None, args.clone())
        }
        None => match package_entry(cmd) {
            Ok((file, root)) => {
                // the commands below read their options from `args[2..]`
                let mut a = vec![args[0].clone(), file.display().to_string()];
                a.extend(args[1..].iter().cloned());
                (file, Some(root), a)
            }
            Err(e) => {
                eprint!("{}", e);
                process::exit(2);
            }
        },
    };

    match cmd {
        "emit" => match compile_to_rust(&file, false) {
            Ok(c) => print!("{}", c.rust),
            Err(e) => {
                eprint!("{}", e);
                process::exit(1);
            }
        },
        // `check` compiles the tests too, so they are checked along with the program
        "check" => match compile_to_rust(&file, true) {
            Ok(_) => eprintln!("ok: {}", file.display()),
            Err(e) => {
                eprint!("{}", e);
                process::exit(1);
            }
        },
        "build" => {
            let mut out = None;
            let mut i = 2;
            while i < args.len() {
                if args[i] == "-o" {
                    out = args.get(i + 1).map(PathBuf::from);
                    i += 2;
                } else {
                    usage();
                }
            }
            if let Err(e) = build(&file, out, false, false, Mode::Ship) {
                eprint!("{}", e);
                process::exit(1);
            }
        }
        "run" => {
            let prog_args: Vec<&String> = {
                let rest = &args[2..];
                match rest.iter().position(|a| a == "--") {
                    Some(p) => rest[p + 1..].iter().collect(),
                    None => rest.iter().collect(),
                }
            };
            let bin = match build(&file, None, true, false, Mode::Iterate) {
                Ok(b) => b,
                Err(e) => {
                    eprint!("{}", e);
                    process::exit(1);
                }
            };
            let status = Command::new(&bin)
                .args(prog_args)
                .status()
                .unwrap_or_else(|e| {
                    eprintln!("error: cannot run `{}`: {}", bin.display(), e);
                    process::exit(1);
                });
            process::exit(status.code().unwrap_or(1));
        }
        "fmt" if pkg_root.is_some() => {
            let mode = args.get(2).map(|s| s.as_str()).unwrap_or("");
            if !matches!(mode, "" | "--check") {
                if mode == "--stdout" {
                    eprintln!("error: `--stdout` formats one file: name it, as in `lume fmt main.lume --stdout`");
                    process::exit(2);
                }
                usage();
            }
            let mut files = Vec::new();
            package_files(pkg_root.as_deref().unwrap(), &mut files);
            files.sort();
            let mut failed = false;
            for f in &files {
                failed |= !fmt_file(f, mode);
            }
            if failed {
                process::exit(1);
            }
        }
        "fmt" => {
            let mode = args.get(2).map(|s| s.as_str()).unwrap_or("");
            if !matches!(mode, "" | "--check" | "--stdout") {
                usage();
            }
            if !fmt_file(&file, mode) {
                process::exit(1);
            }
        }
        "clean" => {
            let dir = file.parent().map(|p| p.to_path_buf()).unwrap_or_else(|| PathBuf::from(".")).join(".lume");
            if dir.exists() {
                if let Err(e) = fs::remove_dir_all(&dir) {
                    eprintln!("error: cannot remove `{}`: {}", dir.display(), e);
                    process::exit(1);
                }
                eprintln!("removed {}", dir.display());
            }
            if args.get(2).map(|a| a == "--cache").unwrap_or(false) {
                let cache = shared_target_dir();
                if cache.exists() {
                    if let Err(e) = fs::remove_dir_all(&cache) {
                        eprintln!("error: cannot remove `{}`: {}", cache.display(), e);
                        process::exit(1);
                    }
                    eprintln!("removed {}", cache.display());
                }
            }
        }
        // `lume crate <file.lume> <crate>`: what Lume can call in a crate the file imports
        "crate" => {
            let krate = match args.get(2) {
                Some(k) => k.clone(),
                None => usage(),
            };
            // make sure the project (and so the crate) exists, then describe it
            let deps: Vec<(String, String)> = match loader::load(&file).and_then(|l| crate_requirements(&l.modules, &l.packages)) {
                Ok(c) => c.into_iter().map(|(k, v, _)| (k, v)).collect(),
                Err(e) => {
                    eprint!("{}", e);
                    process::exit(1);
                }
            };
            if !deps.iter().any(|(k, _)| *k == krate) {
                eprintln!("error: `{}` does not import crate `{}`", file.display(), krate);
                if pkg_root.is_some() {
                    eprintln!("  help: add `{} = \"<version>\"` under `[rust]` in lume.toml, and `import rust.{}` to a file", krate, krate);
                } else {
                    eprintln!("  help: add `import rust.{} = \"<version>\"` to the file first", krate);
                }
                process::exit(1);
            }
            let proj = match write_cargo_project(&file, &deps) {
                Ok(p) => p,
                Err(e) => {
                    eprint!("{}", e);
                    process::exit(1);
                }
            };
            match bridge::load_crate(&proj, &krate, &krate, &shared_target_dir()) {
                Ok(info) => print!("{}", info.describe(&krate)),
                Err(e) => {
                    eprint!("{}", e);
                    process::exit(1);
                }
            }
        }
        "test" => {
            let bin = match build(&file, None, true, true, Mode::Iterate) {
                Ok(b) => b,
                Err(e) => {
                    eprint!("{}", e);
                    process::exit(1);
                }
            };
            let status = Command::new(&bin).status().unwrap_or_else(|e| {
                eprintln!("error: cannot run `{}`: {}", bin.display(), e);
                process::exit(1);
            });
            process::exit(status.code().unwrap_or(1));
        }
        _ => usage(),
    }
}

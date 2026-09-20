//! `lume` command-line tool.
//!
//!   lume build <file.lume> [-o <binary>]   compile to a native binary
//!   lume run   <file.lume> [-- args...]    compile and run
//!   lume emit  <file.lume>                 print the generated Rust
//!   lume check <file.lume>                 parse and check, emit nothing
//!   lume test  <file.lume>                 build and run the `test` blocks
//!   lume fmt   <file.lume> [--check|--stdout]  rewrite in the canonical layout

mod ast;
mod codegen;
mod error;
mod fmt;
mod lexer;
mod loader;
mod parser;

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command, Stdio};
use std::time::Instant;

fn usage() -> ! {
    eprintln!(
        "lume {}\n\nusage:\n  lume build <file.lume> [-o <binary>]\n  lume run   <file.lume> [-- <args>...]\n  lume test  <file.lume>\n  lume fmt   <file.lume> [--check | --stdout]\n  lume emit  <file.lume>\n  lume check <file.lume>",
        env!("CARGO_PKG_VERSION")
    );
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
    let modules = loader::load(path)?;
    let mut exports: std::collections::HashMap<String, codegen::Exports> = std::collections::HashMap::new();
    let mut rust = String::new();
    let mut deps: Vec<(String, String)> = Vec::new();
    let mut has_rust_blocks = false;
    let n = modules.len();
    for (i, m) in modules.iter().enumerate() {
        let is_entry = i + 1 == n;
        let file = m.path.display().to_string();
        let dep_list: Vec<codegen::Dep> = m
            .imports
            .iter()
            .map(|r| match r {
                loader::Resolved::Module { alias, id, line, col } => codegen::Dep::Module { alias: alias.clone(), id: id.clone(), exports: &exports[id], line: *line, col: *col },
                loader::Resolved::Single { local, id, item, line, col } => codegen::Dep::Single { local: local.clone(), id: id.clone(), item: item.clone(), exports: &exports[id], line: *line, col: *col },
            })
            .collect();
        let rust_mod = if is_entry { None } else { Some(m.rust_mod()) };
        let (out, ex) = codegen::generate_module(&m.items, rust_mod.as_deref(), &dep_list, test_mode, &m.src).map_err(|e| e.render(&file, &m.src))?;
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
        for d in out.deps {
            if !deps.contains(&d) {
                deps.push(d);
            }
        }
        has_rust_blocks |= out.has_rust_blocks;
        exports.insert(m.id.clone(), ex);
    }
    if test_mode {
        rust.push_str(&test_runner(&modules));
    }
    Ok(Compiled { rust, deps, has_rust_blocks })
}

/// `fn main` for `lume test`: runs every `test` block of every module,
/// catching failures so the rest still run, and reports a summary.
fn test_runner(modules: &[loader::Module]) -> String {
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

/// Builds through cargo when the program imports Rust crates.
fn build_with_cargo(stem: &str, build_dir: &Path, rust: &str, deps: &[(String, String)], bin: &Path, has_rust_blocks: bool) -> Result<(), String> {
    let proj = build_dir.join(format!("cargo-{}", stem));
    let src_dir = proj.join("src");
    fs::create_dir_all(&src_dir).map_err(|e| format!("error: cannot create `{}`: {}", src_dir.display(), e))?;
    let pkg = stem.replace(|c: char| !c.is_alphanumeric() && c != '_', "_");
    let mut toml = format!("[package]\nname = \"{}\"\nversion = \"0.0.0\"\nedition = \"2021\"\n\n[dependencies]\n", pkg);
    for (k, v) in deps {
        toml.push_str(&format!("{} = \"{}\"\n", k, v));
    }
    toml.push_str("\n[profile.release]\nopt-level = 3\ndebug = false\n");
    fs::write(proj.join("Cargo.toml"), toml).map_err(|e| format!("error: cannot write Cargo.toml: {}", e))?;
    fs::write(src_dir.join("main.rs"), rust).map_err(|e| format!("error: cannot write main.rs: {}", e))?;
    let out = Command::new("cargo")
        .args(["build", "--release", "-q"])
        .current_dir(&proj)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .map_err(|e| format!("error: cannot run cargo: {}\n  help: install Rust from https://rustup.rs", e))?;
    if !out.status.success() {
        let err = String::from_utf8_lossy(&out.stderr);
        return Err(rustc_failed_banner(has_rust_blocks, &src_dir.join("main.rs"), "cargo", &err));
    }
    let built = proj.join("target").join("release").join(&pkg);
    fs::copy(&built, bin).map_err(|e| format!("error: cannot copy `{}` to `{}`: {}", built.display(), bin.display(), e))?;
    Ok(())
}

fn build(path: &Path, out: Option<PathBuf>, quiet: bool, test_mode: bool) -> Result<PathBuf, String> {
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
    if !compiled.deps.is_empty() {
        build_with_cargo(&stem, &build_dir, &rust, &compiled.deps, &bin, compiled.has_rust_blocks)?;
        if !quiet {
            eprintln!("compiled {} -> {} in {:.2}s (cargo, {} crate{})", path.display(), bin.display(), t0.elapsed().as_secs_f64(), compiled.deps.len(), if compiled.deps.len() == 1 { "" } else { "s" });
        }
        return Ok(bin);
    }

    let status = Command::new("rustc")
        .args(["--edition", "2021", "-O", "-C", "debuginfo=0"])
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
    if !quiet {
        eprintln!("compiled {} -> {} in {:.2}s", path.display(), bin.display(), t0.elapsed().as_secs_f64());
    }
    Ok(bin)
}

fn main() {
    let args: Vec<String> = env::args().skip(1).collect();
    if args.is_empty() {
        usage();
    }
    let cmd = args[0].as_str();
    let file = match args.get(1) {
        Some(f) if !f.starts_with('-') => PathBuf::from(f),
        _ => usage(),
    };
    if file.extension().map(|e| e != "lume").unwrap_or(true) {
        eprintln!("error: `{}` is not a .lume file", file.display());
        process::exit(2);
    }

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
            if let Err(e) = build(&file, out, false, false) {
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
            let bin = match build(&file, None, true, false) {
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
        "fmt" => {
            let mode = args.get(2).map(|s| s.as_str()).unwrap_or("");
            if !matches!(mode, "" | "--check" | "--stdout") {
                usage();
            }
            let src = match fs::read_to_string(&file) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!("error: cannot read `{}`: {}", file.display(), e);
                    process::exit(1);
                }
            };
            let formatted = match fmt::format_source(&src) {
                Ok(f) => f,
                Err(e) => {
                    eprint!("{}", e.render(&file.display().to_string(), &src));
                    process::exit(1);
                }
            };
            match mode {
                "--stdout" => print!("{}", formatted),
                "--check" => {
                    if formatted != src {
                        eprintln!("would reformat {}", file.display());
                        process::exit(1);
                    }
                }
                _ => {
                    if formatted != src {
                        if let Err(e) = fs::write(&file, &formatted) {
                            eprintln!("error: cannot write `{}`: {}", file.display(), e);
                            process::exit(1);
                        }
                        eprintln!("formatted {}", file.display());
                    }
                }
            }
        }
        "test" => {
            let bin = match build(&file, None, true, true) {
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

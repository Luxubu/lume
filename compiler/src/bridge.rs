//! The Rust bridge, phase 2: crate signatures.
//!
//! `import rust.regex` makes the crate's public functions and types callable
//! from Lume with no `rust:` block: `regex.Regex.new(p)`, `re.is_match(s)`.
//! The signatures come from rustdoc's JSON output for the crate, produced
//! once per crate version and cached next to the cargo project. rustdoc JSON
//! is still an unstable rustdoc feature; `RUSTC_BOOTSTRAP=1` lets the stable
//! toolchain produce it, which is what cargo-semver-checks and friends do.
//!
//! What crosses the bridge, and how (Rust type -> Lume type):
//!   &str, String, Cow<str>            -> Str
//!   bool                              -> Bool
//!   i8..i64, u8..u64, isize, usize    -> Int    (cast at the boundary)
//!   f32, f64                          -> Float
//!   ()                                -> ()
//!   Option<T>                         -> T?
//!   Result<T, E>                      -> T or Error   (E shown with Display)
//!   Vec<T>, &[T]                      -> [T]
//!   (A, B, ...)                       -> tuples
//!   a pub struct or enum of the crate -> an opaque named type, `regex.Regex`,
//!                                        with the crate's inherent methods
//!   a struct that is an Iterator      -> a lazy chain over its items
//!   a generic `T: AsRef<str>`, `T: Into<String>`, or any bound the crate
//!   implements for `&str`/`String`     -> Str
//! Anything else makes that one function unusable from Lume, with the reason
//! kept for the error message. `rust:` blocks remain for those cases.

use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};

use serde_json::Value;

use crate::ast::{SelfKind, Type};

/// A function or method of a crate as Lume sees it.
#[derive(Debug, Clone)]
pub struct ForeignFn {
    /// `regex::Regex::new`, or the method name for methods
    pub rust_path: String,
    pub params: Vec<ForeignParam>,
    /// Lume type of the result (an iterator struct becomes `Type::Iter`)
    pub ret: Type,
    /// How to turn the Rust result into the Lume value
    pub ret_conv: Conv,
    /// `None` for a free or associated function
    pub self_kind: Option<SelfKind>,
    /// Why Lume cannot call it, when it cannot
    pub unsupported: Option<String>,
}

#[derive(Debug, Clone)]
pub struct ForeignParam {
    pub name: String,
    pub ty: Type,
    pub pass: Pass,
}

/// How a Lume argument is handed to Rust.
#[derive(Debug, Clone, PartialEq)]
pub enum Pass {
    /// `&str`, `&[T]`, `&Struct`: borrow (Lume passes by reference anyway)
    Borrow,
    /// `String`, `Vec<T>`, `Struct`: an owned value
    Owned,
    /// an integer of another width: `(x) as usize`
    IntCast(&'static str),
    /// `f32`
    F32,
    /// `Option<T>`: the payload converted with `Pass`
    Option(Box<Pass>),
    /// `&mut T`: Lume needs a `var`
    MutBorrow,
}

/// How a Rust result becomes a Lume value.
#[derive(Debug, Clone, PartialEq)]
pub enum Conv {
    None,
    /// `&str` or `String` -> String
    ToString,
    /// `Cow<str>` -> String
    IntoOwned,
    /// integer widths -> i64, f32 -> f64
    Cast(&'static str),
    /// `&[T]` -> Vec<T>
    ToVec,
    /// `&T` -> T (clone)
    Clone,
    Option(Box<Conv>),
    /// the error side goes through Display into Lume's `Error`
    Result(Box<Conv>),
    Vec(Box<Conv>),
    Tuple(Vec<Conv>),
}

impl Conv {
    pub fn is_none(&self) -> bool {
        match self {
            Conv::None => true,
            Conv::Option(c) | Conv::Vec(c) => c.is_none(),
            Conv::Tuple(cs) => cs.iter().all(|c| c.is_none()),
            _ => false,
        }
    }

    /// Wraps the Rust expression `e` so that it has the Lume value.
    pub fn apply(&self, e: &str) -> String {
        match self {
            Conv::None => e.to_string(),
            Conv::ToString => format!("({}).to_string()", e),
            Conv::IntoOwned => format!("({}).into_owned()", e),
            Conv::Cast(t) => format!("(({}) as {})", e, t),
            Conv::ToVec => format!("({}).to_vec()", e),
            Conv::Clone => format!("({}).clone()", e),
            Conv::Option(c) => {
                if c.is_none() { e.to_string() } else { format!("({}).map(|lume_x| {})", e, c.apply("lume_x")) }
            }
            Conv::Result(c) => {
                let ok = if c.is_none() { String::new() } else { format!(".map(|lume_x| {})", c.apply("lume_x")) };
                format!("({}){}.map_err(|lume_e| Error {{ message: lume_e.to_string() }})", e, ok)
            }
            Conv::Vec(c) => {
                if c.is_none() { e.to_string() } else { format!("({}).into_iter().map(|lume_x| {}).collect::<Vec<_>>()", e, c.apply("lume_x")) }
            }
            Conv::Tuple(cs) => {
                if cs.iter().all(|c| c.is_none()) {
                    return e.to_string();
                }
                let names: Vec<String> = (0..cs.len()).map(|i| format!("lume_t{}", i)).collect();
                let parts: Vec<String> = cs.iter().zip(&names).map(|(c, n)| c.apply(n)).collect();
                format!("{{ let ({}) = {}; ({}) }}", names.join(", "), e, parts.join(", "))
            }
        }
    }
}

/// A crate's `pub struct` or `pub enum`, opaque to Lume.
#[derive(Debug, Clone, Default)]
pub struct ForeignType {
    /// `regex::Regex`
    pub rust_path: String,
    pub lifetimes: usize,
    /// inherent methods, by name
    pub methods: HashMap<String, ForeignFn>,
    /// associated functions (`Regex::new`), by name
    pub assoc: HashMap<String, ForeignFn>,
    /// `impl Iterator for ...`: the Lume item type
    pub iter_item: Option<Type>,
    pub display: bool,
    pub clone: bool,
    pub partial_eq: bool,
}

/// The names a module of the crate offers.
#[derive(Debug, Clone, Default)]
pub struct Namespace {
    pub fns: HashMap<String, ForeignFn>,
    /// type name -> key in `CrateInfo::types`
    pub types: HashMap<String, String>,
    pub modules: HashMap<String, Namespace>,
}

#[derive(Debug, Clone, Default)]
pub struct CrateInfo {
    pub name: String,
    pub root: Namespace,
    /// keyed by the Lume-side name, `<alias>.Regex`, `<alias>.bytes.Regex`
    pub types: HashMap<String, ForeignType>,
}

impl CrateInfo {
    /// A readable listing: what Lume can call in this crate, and what it
    /// cannot yet (with the reason). `lume crate <file.lume> <crate>`.
    pub fn describe(&self, alias: &str) -> String {
        fn sig(f: &ForeignFn) -> String {
            let ps: Vec<String> = f.params.iter().map(|p| format!("{}: {}", p.name, crate::codegen::type_name(&p.ty))).collect();
            let ret = if f.ret == Type::Unit { String::new() } else { format!(" -> {}", crate::codegen::type_name(&f.ret)) };
            match &f.unsupported {
                Some(why) => format!("({}){}   [not from Lume yet: {}]", ps.join(", "), ret, why),
                None => format!("({}){}", ps.join(", "), ret),
            }
        }
        fn walk(out: &mut String, ns: &Namespace, prefix: &str, types: &HashMap<String, ForeignType>) {
            let mut fns: Vec<_> = ns.fns.iter().collect();
            fns.sort_by(|a, b| a.0.cmp(b.0));
            for (n, f) in fns {
                out.push_str(&format!("def {}.{}{}\n", prefix, n, sig(f)));
            }
            let mut tys: Vec<_> = ns.types.iter().collect();
            tys.sort_by(|a, b| a.0.cmp(b.0));
            for (n, key) in tys {
                let ft = &types[key];
                let mut tags = Vec::new();
                if let Some(it) = &ft.iter_item {
                    tags.push(format!("iterates {}", crate::codegen::type_name(it)));
                }
                if ft.display {
                    tags.push("printable".into());
                }
                out.push_str(&format!("type {}.{}{}\n", prefix, n, if tags.is_empty() { String::new() } else { format!("  ({})", tags.join(", ")) }));
                let mut assoc: Vec<_> = ft.assoc.iter().collect();
                assoc.sort_by(|a, b| a.0.cmp(b.0));
                for (m, f) in assoc {
                    out.push_str(&format!("  def {}.{}.{}{}\n", prefix, n, m, sig(f)));
                }
                let mut ms: Vec<_> = ft.methods.iter().collect();
                ms.sort_by(|a, b| a.0.cmp(b.0));
                for (m, f) in ms {
                    let var = if f.self_kind == Some(SelfKind::Mutate) { "var " } else { "" };
                    out.push_str(&format!("  def {}.{}{}\n", var, m, sig(f)));
                }
            }
            let mut mods: Vec<_> = ns.modules.iter().collect();
            mods.sort_by(|a, b| a.0.cmp(b.0));
            for (n, sub) in mods {
                walk(out, sub, &format!("{}.{}", prefix, n), types);
            }
        }
        let mut out = String::new();
        walk(&mut out, &self.root, alias, &self.types);
        out
    }
}

/// Produces (or reuses) the rustdoc JSON for `krate` inside the cargo
/// project at `proj`, and reads it.
pub fn load_crate(proj: &Path, krate: &str, alias: &str, target_dir: &Path) -> Result<CrateInfo, String> {
    let lib_name = krate.replace('-', "_");
    let json_path: PathBuf = proj.join(format!("signatures-{}.json", lib_name));
    let toml = proj.join("Cargo.toml");
    let fresh = match (std::fs::metadata(&json_path), std::fs::metadata(&toml)) {
        (Ok(j), Ok(t)) => j.modified().ok() >= t.modified().ok(),
        _ => false,
    };
    if !fresh {
        {
            use std::io::IsTerminal;
            if std::io::stderr().is_terminal() {
                eprintln!("reading the signatures of crate `{}`...", krate);
            }
        }
        let out = Command::new("cargo")
            .args(["rustdoc", "-q", "-p", krate, "--", "-Z", "unstable-options", "--output-format", "json"])
            .env("RUSTC_BOOTSTRAP", "1")
            .env("CARGO_TARGET_DIR", target_dir)
            .current_dir(proj)
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .output()
            .map_err(|e| format!("error: cannot run cargo: {}\n  help: install Rust from https://rustup.rs", e))?;
        if !out.status.success() {
            let err = String::from_utf8_lossy(&out.stderr);
            return Err(format!("error: could not read the signatures of crate `{}`\n\ncargo said:\n{}", krate, err));
        }
        // keep our own copy: the shared doc directory is overwritten by other programs' versions
        let produced = target_dir.join("doc").join(format!("{}.json", lib_name));
        std::fs::copy(&produced, &json_path).map_err(|e| format!("error: cannot copy `{}`: {}", produced.display(), e))?;
    }
    let text = std::fs::read_to_string(&json_path).map_err(|e| format!("error: cannot read `{}`: {}", json_path.display(), e))?;
    let doc: Value = serde_json::from_str(&text).map_err(|e| format!("error: bad rustdoc JSON for `{}`: {}", krate, e))?;
    let mut b = Builder {
        doc: &doc,
        lib_name: lib_name.clone(),
        info: CrateInfo { name: krate.to_string(), ..Default::default() },
        visiting: Vec::new(),
        ids: HashMap::new(),
        pending: Vec::new(),
        current_self_key: None,
    };
    let root_id = doc["root"].to_string();
    // pass 1: names and types; pass 2: the methods, once every type has a name
    let root = b.module(&root_id, alias, &lib_name);
    b.info.root = root;
    let pending = std::mem::take(&mut b.pending);
    // 2a: what each type is (iterator item, Display, Clone) — method
    // signatures that mention the type need this first
    for (id, key, kind) in &pending {
        let it = match b.item(id) {
            Some(it) => it.clone(),
            None => continue,
        };
        b.current_self_key = Some(key.clone());
        let flags = b.type_flags(&it, kind, id);
        b.current_self_key = None;
        let ft = b.info.types.get_mut(key).unwrap();
        ft.lifetimes = flags.0;
        ft.iter_item = flags.1;
        ft.display = flags.2;
        ft.clone = flags.3;
        ft.partial_eq = flags.4;
    }
    // 2b: the methods
    for (id, key, kind) in &pending {
        let it = match b.item(id) {
            Some(it) => it.clone(),
            None => continue,
        };
        let rust_path = b.info.types[key].rust_path.clone();
        b.current_self_key = Some(key.clone());
        let (methods, assoc) = b.type_methods(&it, kind, id, &rust_path);
        b.current_self_key = None;
        let ft = b.info.types.get_mut(key).unwrap();
        ft.methods = methods;
        ft.assoc = assoc;
    }
    Ok(b.info)
}

struct Builder<'a> {
    doc: &'a Value,
    lib_name: String,
    info: CrateInfo,
    /// module ids being expanded (glob re-exports can be circular)
    visiting: Vec<String>,
    /// item id -> Lume key, for every exported struct/enum
    ids: HashMap<String, String>,
    /// (id, key, "struct"|"enum") awaiting their methods
    pending: Vec<(String, String, String)>,
    /// the type whose methods are being read (`Self`)
    current_self_key: Option<String>,
}

/// A Rust type as rustdoc describes it, reduced to what the bridge handles.
#[derive(Debug, Clone)]
enum RTy {
    Str,
    String,
    Bool,
    Int(&'static str),
    Float(&'static str),
    Unit,
    Ref(Box<RTy>, bool),
    Slice(Box<RTy>),
    Vec(Box<RTy>),
    Option(Box<RTy>),
    Result(Box<RTy>, Box<RTy>),
    Cow(Box<RTy>),
    Tuple(Vec<RTy>),
    /// a struct/enum of this crate: (item id, display path)
    Local(String, String),
    /// a type parameter: name
    Generic(String),
    SelfTy,
    Other(String),
}

impl<'a> Builder<'a> {
    fn item(&self, id: &str) -> Option<&'a Value> {
        let it = &self.doc["index"][id];
        if it.is_null() { None } else { Some(it) }
    }

    fn item_kind(it: &Value) -> Option<(&str, &Value)> {
        it["inner"].as_object().and_then(|o| o.iter().next()).map(|(k, v)| (k.as_str(), v))
    }

    fn is_public(it: &Value) -> bool {
        it["visibility"] == "public" || it["visibility"] == "default"
    }

    /// The rustdoc path of a local item, `regex::regex::string::Regex`.
    fn path_of(&self, id: &str) -> Option<Vec<String>> {
        let p = &self.doc["paths"][id];
        if p.is_null() {
            return None;
        }
        Some(p["path"].as_array()?.iter().filter_map(|s| s.as_str().map(|s| s.to_string())).collect())
    }

    /// Builds the namespace of module `id`, whose Lume-side prefix is
    /// `lume_prefix` (`regex`, `regex.bytes`) and Rust path `rust_prefix`.
    fn module(&mut self, id: &str, lume_prefix: &str, rust_prefix: &str) -> Namespace {
        let mut ns = Namespace::default();
        if self.visiting.contains(&id.to_string()) {
            return ns;
        }
        self.visiting.push(id.to_string());
        let items: Vec<String> = self
            .item(id)
            .and_then(|it| it["inner"]["module"]["items"].as_array())
            .map(|a| a.iter().map(|v| v.to_string()).collect())
            .unwrap_or_default();
        for iid in items {
            self.add_item(&mut ns, &iid, None, lume_prefix, rust_prefix);
        }
        self.visiting.pop();
        ns
    }

    fn add_item(&mut self, ns: &mut Namespace, iid: &str, rename: Option<&str>, lume_prefix: &str, rust_prefix: &str) {
        let it = match self.item(iid) {
            Some(it) => it,
            None => return,
        };
        if !Self::is_public(it) {
            return;
        }
        let (kind, inner) = match Self::item_kind(it) {
            Some(k) => k,
            None => return,
        };
        let name = rename.map(|s| s.to_string()).or_else(|| it["name"].as_str().map(|s| s.to_string()));
        match kind {
            "use" => {
                let target = inner["id"].to_string();
                if inner["is_glob"].as_bool().unwrap_or(false) {
                    // everything the target module offers, under this module's names
                    let sub = self.module(&target, lume_prefix, rust_prefix);
                    for (k, v) in sub.fns {
                        ns.fns.entry(k).or_insert(v);
                    }
                    for (k, v) in sub.types {
                        ns.types.entry(k).or_insert(v);
                    }
                    for (k, v) in sub.modules {
                        ns.modules.entry(k).or_insert(v);
                    }
                } else if let Some(n) = inner["name"].as_str() {
                    if self.item(&target).is_some() {
                        self.add_item(ns, &target, Some(n), lume_prefix, rust_prefix);
                    }
                }
            }
            "module" => {
                if let Some(n) = name {
                    let sub = self.module(iid, &format!("{}.{}", lume_prefix, n), &format!("{}::{}", rust_prefix, n));
                    ns.modules.insert(n, sub);
                }
            }
            "function" => {
                if let Some(n) = name {
                    let f = self.function(inner, &format!("{}::{}", rust_prefix, n), None);
                    ns.fns.insert(n, f);
                }
            }
            "struct" | "enum" => {
                // a type with type parameters (`Encoded<S>`) stays on the Rust side
                let has_type_params = inner["generics"]["params"]
                    .as_array()
                    .map(|ps| ps.iter().any(|p| p["kind"].get("lifetime").is_none()))
                    .unwrap_or(false);
                if has_type_params {
                    return;
                }
                if let Some(n) = name {
                    if let Some(existing) = self.ids.get(iid) {
                        // the same type reached through another re-export
                        ns.types.insert(n, existing.clone());
                        return;
                    }
                    let key = format!("{}.{}", lume_prefix, n);
                    self.info.types.insert(key.clone(), ForeignType { rust_path: format!("{}::{}", rust_prefix, n), ..Default::default() });
                    self.ids.insert(iid.to_string(), key.clone());
                    self.pending.push((iid.to_string(), key.clone(), kind.to_string()));
                    ns.types.insert(n, key);
                }
            }
            _ => {}
        }
    }

    fn impl_ids(it: &Value, kind: &str) -> Vec<String> {
        it["inner"][kind]["impls"].as_array().map(|a| a.iter().map(|v| v.to_string()).collect()).unwrap_or_default()
    }

    /// (lifetimes, iterator item, Display, Clone, PartialEq)
    fn type_flags(&mut self, it: &Value, kind: &str, id: &str) -> (usize, Option<Type>, bool, bool, bool) {
        let lifetimes = it["inner"][kind]["generics"]["params"]
            .as_array()
            .map(|ps| ps.iter().filter(|p| p["kind"].get("lifetime").is_some()).count())
            .unwrap_or(0);
        let self_id = id.to_string();
        let (mut item, mut display, mut clone, mut peq) = (None, false, false, false);
        for imp_id in Self::impl_ids(it, kind) {
            let imp = match self.item(&imp_id) {
                Some(i) => i["inner"]["impl"].clone(),
                None => continue,
            };
            match imp["trait"]["path"].as_str() {
                Some("Iterator") => {
                    let items: Vec<String> = imp["items"].as_array().map(|a| a.iter().map(|v| v.to_string()).collect()).unwrap_or_default();
                    for aid in items {
                        if let Some(ait) = self.item(&aid) {
                            if ait["name"] == "Item" {
                                if let Some(("assoc_type", ai)) = Self::item_kind(ait) {
                                    let rt = self.rty(&ai["type"], Some(&self_id));
                                    item = self.lume_type(&rt);
                                }
                            }
                        }
                    }
                }
                Some("Display") => display = true,
                Some("Clone") => clone = true,
                Some("PartialEq") => peq = true,
                _ => {}
            }
        }
        (lifetimes, item, display, clone, peq)
    }

    fn type_methods(&mut self, it: &Value, kind: &str, id: &str, rust_path: &str) -> (HashMap<String, ForeignFn>, HashMap<String, ForeignFn>) {
        let self_id = id.to_string();
        let mut methods = HashMap::new();
        let mut assoc = HashMap::new();
        for imp_id in Self::impl_ids(it, kind) {
            let imp = match self.item(&imp_id) {
                Some(i) => i["inner"]["impl"].clone(),
                None => continue,
            };
            if imp["trait"].is_null() {
                let items: Vec<String> = imp["items"].as_array().map(|a| a.iter().map(|v| v.to_string()).collect()).unwrap_or_default();
                for fid in items {
                    let fit = match self.item(&fid) {
                        Some(f) => f,
                        None => continue,
                    };
                    if !Self::is_public(fit) {
                        continue;
                    }
                    if let Some(("function", finner)) = Self::item_kind(fit) {
                        let fname = fit["name"].as_str().unwrap_or("").to_string();
                        let f = self.function(finner, &format!("{}::{}", rust_path, fname), Some(&self_id));
                        if f.self_kind.is_some() {
                            methods.insert(fname, f);
                        } else {
                            assoc.insert(fname, f);
                        }
                    }
                }
            }
        }
        (methods, assoc)
    }

    /// Reads a function item into a `ForeignFn`.
    fn function(&mut self, f: &Value, rust_path: &str, self_id: Option<&String>) -> ForeignFn {
        let mut out = ForeignFn { rust_path: rust_path.to_string(), params: Vec::new(), ret: Type::Unit, ret_conv: Conv::None, self_kind: None, unsupported: None };
        // generics: type parameters are only allowed when a bound pins them to a string
        let mut generic_as_str: Vec<String> = Vec::new();
        if let Some(ps) = f["generics"]["params"].as_array() {
            for p in ps {
                if p["kind"].get("lifetime").is_some() {
                    continue;
                }
                let name = p["name"].as_str().unwrap_or("").to_string();
                if let Some(tk) = p["kind"].get("type") {
                    if self.bounds_accept_str(&tk["bounds"]) {
                        generic_as_str.push(name);
                        continue;
                    }
                }
                out.unsupported = Some(format!("it is generic over `{}`", name));
            }
        }
        if let Some(wp) = f["generics"]["where_predicates"].as_array() {
            if !wp.is_empty() && out.unsupported.is_none() {
                out.unsupported = Some("it has a `where` clause".into());
            }
        }
        if f["header"]["is_unsafe"].as_bool().unwrap_or(false) {
            out.unsupported = Some("it is `unsafe`".into());
        }
        if f["header"]["is_async"].as_bool().unwrap_or(false) {
            out.unsupported = Some("it is `async` (async crate calls come later)".into());
        }
        let inputs = f["sig"]["inputs"].as_array().cloned().unwrap_or_default();
        for (i, inp) in inputs.iter().enumerate() {
            let pname = inp[0].as_str().unwrap_or("").to_string();
            let mut rt = self.rty(&inp[1], self_id);
            if let RTy::Generic(g) = &rt {
                if generic_as_str.contains(g) {
                    rt = RTy::Str;
                }
            }
            if i == 0 && pname == "self" {
                out.self_kind = Some(match &rt {
                    RTy::Ref(_, true) => SelfKind::Mutate,
                    RTy::Ref(_, false) => SelfKind::Read,
                    _ => {
                        if out.unsupported.is_none() {
                            out.unsupported = Some("it takes `self` by value (consumes the value)".into());
                        }
                        SelfKind::Read
                    }
                });
                continue;
            }
            match self.param(&rt) {
                Some((ty, pass)) => out.params.push(ForeignParam { name: pname, ty, pass }),
                None => {
                    if out.unsupported.is_none() {
                        out.unsupported = Some(format!("parameter `{}` has type `{}`", pname, self.describe(&rt)));
                    }
                    out.params.push(ForeignParam { name: pname, ty: Type::Unknown, pass: Pass::Owned });
                }
            }
        }
        let ret = if f["sig"]["output"].is_null() { RTy::Unit } else { self.rty(&f["sig"]["output"], self_id) };
        match self.result(&ret) {
            Some((ty, conv)) => {
                out.ret = ty;
                out.ret_conv = conv;
            }
            None => {
                if out.unsupported.is_none() {
                    out.unsupported = Some(format!("it returns `{}`", self.describe(&ret)));
                }
            }
        }
        out
    }

    /// Does a generic parameter's bound list mean "any string will do"?
    fn bounds_accept_str(&self, bounds: &Value) -> bool {
        let bs = match bounds.as_array() {
            Some(b) => b,
            None => return false,
        };
        if bs.is_empty() {
            return false;
        }
        bs.iter().all(|b| {
            let tb = &b["trait_bound"]["trait"];
            let path = tb["path"].as_str().unwrap_or("");
            let short = path.rsplit("::").next().unwrap_or(path);
            match short {
                "AsRef" | "Into" | "Borrow" => {
                    // AsRef<str>, AsRef<[u8]>, Into<String>
                    let arg = &tb["args"]["angle_bracketed"]["args"][0]["type"];
                    arg["primitive"] == "str"
                        || arg["slice"]["primitive"] == "u8"
                        || arg["resolved_path"]["path"].as_str().map(|p| p.ends_with("String")).unwrap_or(false)
                }
                "Sized" | "Send" | "Sync" => true,
                _ => {
                    // a crate trait implemented for `&str` or `String` (regex's Replacer)
                    let tid = tb["id"].to_string();
                    self.trait_impl_for_str(&tid)
                }
            }
        })
    }

    fn trait_impl_for_str(&self, trait_id: &str) -> bool {
        let tr = match self.item(trait_id) {
            Some(t) => t,
            None => return false,
        };
        let impls = match tr["inner"]["trait"]["implementations"].as_array() {
            Some(i) => i,
            None => return false,
        };
        impls.iter().any(|i| {
            let f = &self.doc["index"][i.to_string()]["inner"]["impl"]["for"];
            f["borrowed_ref"]["type"]["primitive"] == "str" || f["resolved_path"]["path"].as_str().map(|p| p.ends_with("String")).unwrap_or(false)
        })
    }

    fn rty(&self, t: &Value, self_id: Option<&String>) -> RTy {
        if let Some(p) = t.get("primitive").and_then(|p| p.as_str()) {
            return match p {
                "str" => RTy::Str,
                "bool" => RTy::Bool,
                "i64" => RTy::Int("i64"),
                "i32" => RTy::Int("i32"),
                "i16" => RTy::Int("i16"),
                "i8" => RTy::Int("i8"),
                "isize" => RTy::Int("isize"),
                "u64" => RTy::Int("u64"),
                "u32" => RTy::Int("u32"),
                "u16" => RTy::Int("u16"),
                "u8" => RTy::Int("u8"),
                "usize" => RTy::Int("usize"),
                "f64" => RTy::Float("f64"),
                "f32" => RTy::Float("f32"),
                "char" => RTy::Other("char".into()),
                other => RTy::Other(other.into()),
            };
        }
        if let Some(r) = t.get("borrowed_ref") {
            return RTy::Ref(Box::new(self.rty(&r["type"], self_id)), r["is_mutable"].as_bool().unwrap_or(false));
        }
        if let Some(s) = t.get("slice") {
            return RTy::Slice(Box::new(self.rty(s, self_id)));
        }
        if let Some(items) = t.get("tuple").and_then(|x| x.as_array()) {
            if items.is_empty() {
                return RTy::Unit;
            }
            return RTy::Tuple(items.iter().map(|i| self.rty(i, self_id)).collect());
        }
        if let Some(g) = t.get("generic").and_then(|g| g.as_str()) {
            return if g == "Self" { RTy::SelfTy } else { RTy::Generic(g.to_string()) };
        }
        if let Some(rp) = t.get("resolved_path") {
            let path = rp["path"].as_str().unwrap_or("");
            let id = rp["id"].to_string();
            let short = path.rsplit("::").next().unwrap_or(path);
            let args: Vec<RTy> = rp["args"]["angle_bracketed"]["args"]
                .as_array()
                .map(|a| a.iter().filter(|x| x.get("type").is_some()).map(|x| self.rty(&x["type"], self_id)).collect())
                .unwrap_or_default();
            let full = self.path_of(&id);
            let is_std = full.as_ref().map(|p| matches!(p.first().map(|s| s.as_str()), Some("core" | "alloc" | "std"))).unwrap_or(false);
            if is_std || full.is_none() {
                return match (short, args.len()) {
                    ("String", 0) => RTy::String,
                    ("Option", 1) => RTy::Option(Box::new(args[0].clone())),
                    ("Result", 2) => RTy::Result(Box::new(args[0].clone()), Box::new(args[1].clone())),
                    ("Result", 1) => RTy::Result(Box::new(args[0].clone()), Box::new(RTy::Other("error".into()))),
                    ("Vec", 1) => RTy::Vec(Box::new(args[0].clone())),
                    ("Cow", 1) => RTy::Cow(Box::new(args[0].clone())),
                    ("Box", 1) => args[0].clone(),
                    _ => RTy::Other(path.to_string()),
                };
            }
            // a type of this crate
            let full = full.unwrap();
            if full.first().map(|s| s.as_str()) == Some(self.lib_name.as_str()) {
                if Some(&id) == self_id {
                    return RTy::SelfTy;
                }
                return RTy::Local(id, short.to_string());
            }
            return RTy::Other(path.to_string());
        }
        if let Some(q) = t.get("qualified_path") {
            return RTy::Other(format!("<..>::{}", q["name"].as_str().unwrap_or("?")));
        }
        if t.get("impl_trait").is_some() {
            if self.bounds_accept_str(&t["impl_trait"]) {
                return RTy::Str;
            }
            return RTy::Other("impl Trait".into());
        }
        if t.get("dyn_trait").is_some() {
            return RTy::Other("dyn Trait".into());
        }
        if let Some(a) = t.get("array") {
            return RTy::Other(format!("[{}; N]", self.describe(&self.rty(&a["type"], self_id))));
        }
        RTy::Other("?".into())
    }

    fn describe(&self, t: &RTy) -> String {
        match t {
            RTy::Str => "&str".into(),
            RTy::String => "String".into(),
            RTy::Bool => "bool".into(),
            RTy::Int(n) | RTy::Float(n) => n.to_string(),
            RTy::Unit => "()".into(),
            RTy::Ref(i, m) => format!("&{}{}", if *m { "mut " } else { "" }, self.describe(i)),
            RTy::Slice(i) => format!("[{}]", self.describe(i)),
            RTy::Vec(i) => format!("Vec<{}>", self.describe(i)),
            RTy::Option(i) => format!("Option<{}>", self.describe(i)),
            RTy::Result(a, b) => format!("Result<{}, {}>", self.describe(a), self.describe(b)),
            RTy::Cow(i) => format!("Cow<{}>", self.describe(i)),
            RTy::Tuple(ts) => format!("({})", ts.iter().map(|t| self.describe(t)).collect::<Vec<_>>().join(", ")),
            RTy::Local(_, n) => n.clone(),
            RTy::Generic(g) => g.clone(),
            RTy::SelfTy => "Self".into(),
            RTy::Other(s) => s.clone(),
        }
    }

    fn key_for_id(&self, id: &str) -> Option<String> {
        self.ids.get(id).cloned()
    }

    /// Lume type of a Rust type in a position that needs one.
    fn lume_type(&self, t: &RTy) -> Option<Type> {
        Some(match t {
            RTy::Str | RTy::String => Type::Str,
            RTy::Bool => Type::Bool,
            RTy::Int(_) => Type::Int,
            RTy::Float(_) => Type::Float,
            RTy::Unit => Type::Unit,
            RTy::Ref(i, _) => return self.lume_type(i),
            RTy::Slice(i) | RTy::Vec(i) => Type::List(Box::new(self.lume_type(i)?)),
            RTy::Option(i) => Type::Option(Box::new(self.lume_type(i)?)),
            RTy::Result(a, _) => Type::Result(Box::new(self.lume_type(a)?), Box::new(Type::Named("Error".into()))),
            RTy::Cow(i) => return self.lume_type(i),
            RTy::Tuple(ts) => Type::Tuple(ts.iter().map(|t| self.lume_type(t)).collect::<Option<Vec<_>>>()?),
            RTy::Local(id, _) => {
                let key = self.key_for_id(id)?;
                match self.info.types.get(&key).and_then(|ft| ft.iter_item.clone()) {
                    Some(item) => Type::Iter(Box::new(item), false),
                    None => Type::Named(key),
                }
            }
            RTy::SelfTy => Type::Named(self.current_self_key.clone()?),
            RTy::Generic(_) | RTy::Other(_) => return None,
        })
    }

    /// How a Lume value is passed for a parameter of this Rust type.
    fn param(&self, t: &RTy) -> Option<(Type, Pass)> {
        Some(match t {
            RTy::Str => (Type::Str, Pass::Borrow),
            RTy::String => (Type::Str, Pass::Owned),
            RTy::Bool => (Type::Bool, Pass::Owned),
            RTy::Int("i64") => (Type::Int, Pass::Owned),
            RTy::Int(w) => (Type::Int, Pass::IntCast(w)),
            RTy::Float("f64") => (Type::Float, Pass::Owned),
            RTy::Float(_) => (Type::Float, Pass::F32),
            RTy::Ref(inner, false) => match &**inner {
                RTy::Str => (Type::Str, Pass::Borrow),
                RTy::String => (Type::Str, Pass::Borrow),
                RTy::Slice(e) | RTy::Vec(e) => (Type::List(Box::new(self.lume_type(e)?)), Pass::Borrow),
                RTy::Local(..) | RTy::SelfTy => (self.lume_type(inner)?, Pass::Borrow),
                RTy::Int(_) | RTy::Float(_) | RTy::Bool => return None,
                _ => return None,
            },
            RTy::Ref(inner, true) => match &**inner {
                RTy::Local(..) | RTy::SelfTy | RTy::Vec(_) | RTy::String => (self.lume_type(inner)?, Pass::MutBorrow),
                _ => return None,
            },
            RTy::Vec(e) => (Type::List(Box::new(self.lume_type(e)?)), Pass::Owned),
            RTy::Option(inner) => {
                let (t, p) = self.param(inner)?;
                if p == Pass::Borrow {
                    return None;
                }
                (Type::Option(Box::new(t)), Pass::Option(Box::new(p)))
            }
            RTy::Local(..) => (self.lume_type(t)?, Pass::Owned),
            _ => return None,
        })
    }

    /// Lume type and conversion of a returned Rust value.
    fn result(&self, t: &RTy) -> Option<(Type, Conv)> {
        Some(match t {
            RTy::Str => (Type::Str, Conv::ToString),
            RTy::String => (Type::Str, Conv::None),
            RTy::Bool => (Type::Bool, Conv::None),
            RTy::Unit => (Type::Unit, Conv::None),
            RTy::Int("i64") => (Type::Int, Conv::None),
            RTy::Int(_) => (Type::Int, Conv::Cast("i64")),
            RTy::Float("f64") => (Type::Float, Conv::None),
            RTy::Float(_) => (Type::Float, Conv::Cast("f64")),
            RTy::Cow(inner) => match &**inner {
                RTy::Str => (Type::Str, Conv::IntoOwned),
                _ => return None,
            },
            RTy::Ref(inner, _) => match &**inner {
                RTy::Str | RTy::String => (Type::Str, Conv::ToString),
                RTy::Slice(e) | RTy::Vec(e) => {
                    let (et, ec) = self.result(e)?;
                    (Type::List(Box::new(et)), if ec.is_none() { Conv::ToVec } else { Conv::Vec(Box::new(ec)) })
                }
                RTy::Local(..) | RTy::SelfTy => (self.lume_type(inner)?, Conv::Clone),
                RTy::Int(_) | RTy::Float(_) | RTy::Bool => {
                    let (t2, c2) = self.result(inner)?;
                    (t2, if c2.is_none() { Conv::Clone } else { c2 })
                }
                _ => return None,
            },
            RTy::Vec(e) => {
                let (et, ec) = self.result(e)?;
                (Type::List(Box::new(et)), Conv::Vec(Box::new(ec)))
            }
            RTy::Option(inner) => {
                let (it, ic) = self.result(inner)?;
                (Type::Option(Box::new(it)), Conv::Option(Box::new(ic)))
            }
            RTy::Result(ok, _err) => {
                let (ot, oc) = self.result(ok)?;
                (Type::Result(Box::new(ot), Box::new(Type::Named("Error".into()))), Conv::Result(Box::new(oc)))
            }
            RTy::Tuple(ts) => {
                let parts: Vec<(Type, Conv)> = ts.iter().map(|t| self.result(t)).collect::<Option<Vec<_>>>()?;
                (Type::Tuple(parts.iter().map(|(t, _)| t.clone()).collect()), Conv::Tuple(parts.into_iter().map(|(_, c)| c).collect()))
            }
            RTy::Local(..) | RTy::SelfTy => (self.lume_type(t)?, Conv::None),
            _ => return None,
        })
    }
}

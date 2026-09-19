//! Code generator: Lume AST -> Rust source.
//!
//! Holds the type table (structs, enums, signatures), infers expression
//! types, decides borrowing, and emits Rust. The checks that must produce
//! *Lume* errors (rather than rustc errors) live here: unknown names,
//! assignment to an immutable binding, changing a field without `var self`,
//! wrong argument counts and keywords, `if` used as a value without `else`,
//! non-exhaustive `match`, `?` in a function that cannot return `None`.

use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::error::{LumeError, Result};

pub const PRELUDE: &str = r#"#![allow(unused, non_snake_case, non_camel_case_types, unused_parens, unused_mut, clippy::all)]
// ---- Lume prelude ----
trait LumePow { fn lume_pow(self, e: Self) -> Self; }
impl LumePow for i64 { fn lume_pow(self, e: Self) -> Self { self.pow(e as u32) } }
impl LumePow for f64 { fn lume_pow(self, e: Self) -> Self { self.powf(e) } }
trait LumeLen { fn lume_len(&self) -> i64; }
impl<T> LumeLen for Vec<T> { fn lume_len(&self) -> i64 { self.len() as i64 } }
impl<T> LumeLen for [T] { fn lume_len(&self) -> i64 { self.len() as i64 } }
impl LumeLen for String { fn lume_len(&self) -> i64 { self.chars().count() as i64 } }
impl LumeLen for str { fn lume_len(&self) -> i64 { self.chars().count() as i64 } }
impl<T: LumeLen + ?Sized> LumeLen for &T { fn lume_len(&self) -> i64 { (**self).lume_len() } }
trait LumeEmpty { fn lume_empty(&self) -> bool; }
impl<T> LumeEmpty for Vec<T> { fn lume_empty(&self) -> bool { self.is_empty() } }
impl<T> LumeEmpty for [T] { fn lume_empty(&self) -> bool { self.is_empty() } }
impl LumeEmpty for String { fn lume_empty(&self) -> bool { self.is_empty() } }
impl LumeEmpty for str { fn lume_empty(&self) -> bool { self.is_empty() } }
impl<T: LumeEmpty + ?Sized> LumeEmpty for &T { fn lume_empty(&self) -> bool { (**self).lume_empty() } }
trait LumeSum<T> { fn lume_sum(&self) -> T; }
impl LumeSum<i64> for Vec<i64> { fn lume_sum(&self) -> i64 { self.iter().sum() } }
impl LumeSum<f64> for Vec<f64> { fn lume_sum(&self) -> f64 { self.iter().sum() } }
impl<T, U: LumeSum<T>> LumeSum<T> for &U { fn lume_sum(&self) -> T { (**self).lume_sum() } }
trait LumeShow { fn lume_str(&self) -> String; }
impl LumeShow for i64 { fn lume_str(&self) -> String { self.to_string() } }
impl LumeShow for f64 { fn lume_str(&self) -> String { format!("{:?}", if *self == 0.0 { 0.0 } else { *self }) } }
impl LumeShow for bool { fn lume_str(&self) -> String { self.to_string() } }
impl LumeShow for String { fn lume_str(&self) -> String { self.clone() } }
impl LumeShow for str { fn lume_str(&self) -> String { self.to_string() } }
impl LumeShow for () { fn lume_str(&self) -> String { String::from("()") } }
impl<T: LumeShow> LumeShow for Vec<T> {
    fn lume_str(&self) -> String { format!("[{}]", self.iter().map(|x| x.lume_str()).collect::<Vec<_>>().join(", ")) }
}
impl<T: LumeShow> LumeShow for [T] {
    fn lume_str(&self) -> String { format!("[{}]", self.iter().map(|x| x.lume_str()).collect::<Vec<_>>().join(", ")) }
}
impl<T: LumeShow> LumeShow for Option<T> {
    fn lume_str(&self) -> String { match self { Some(x) => format!("Some({})", x.lume_str()), None => String::from("None") } }
}
impl<A: LumeShow, B: LumeShow> LumeShow for (A, B) {
    fn lume_str(&self) -> String { format!("({}, {})", self.0.lume_str(), self.1.lume_str()) }
}
impl<A: LumeShow, B: LumeShow, C: LumeShow> LumeShow for (A, B, C) {
    fn lume_str(&self) -> String { format!("({}, {}, {})", self.0.lume_str(), self.1.lume_str(), self.2.lume_str()) }
}
impl<T: LumeShow + ?Sized> LumeShow for &T { fn lume_str(&self) -> String { (**self).lume_str() } }
impl<T: LumeShow + ?Sized> LumeShow for Box<T> { fn lume_str(&self) -> String { (**self).lume_str() } }
#[derive(Debug, Clone, PartialEq)]
struct Error { message: String }
impl LumeShow for Error { fn lume_str(&self) -> String { format!("Error({})", self.message) } }
impl<T: LumeShow, E: LumeShow> LumeShow for Result<T, E> {
    fn lume_str(&self) -> String { match self { Ok(x) => format!("Ok({})", x.lume_str()), Err(e) => e.lume_str() } }
}
fn lume_to_int(s: &str) -> Result<i64, Error> {
    s.trim().parse::<i64>().map_err(|_| Error { message: format!("`{}` is not an integer", s) })
}
fn lume_to_float(s: &str) -> Result<f64, Error> {
    s.trim().parse::<f64>().map_err(|_| Error { message: format!("`{}` is not a number", s) })
}
fn lume_read_file(path: &str) -> Result<String, Error> {
    std::fs::read_to_string(path).map_err(|e| Error { message: format!("cannot read `{}`: {}", path, e) })
}
fn lume_write_file(path: &str, text: &str) -> Result<(), Error> {
    std::fs::write(path, text).map_err(|e| Error { message: format!("cannot write `{}`: {}", path, e) })
}
fn lume_args() -> Vec<String> { std::env::args().skip(1).collect() }
/// Lume's map: insertion-ordered (as in Ruby), hash lookups, one copy of each key.
/// Entries live in a Vec; a hash -> positions index finds them. Removal leaves a
/// tombstone; the Vec is compacted when tombstones outnumber live entries.
#[derive(Clone, Debug)]
struct LumeMap<K, V> {
    entries: Vec<Option<(K, V)>>,
    index: std::collections::HashMap<u64, Vec<usize>>,
    live: usize,
}
impl<K: std::hash::Hash + Eq + Clone, V: Clone> LumeMap<K, V> {
    fn new() -> Self { LumeMap { entries: Vec::new(), index: std::collections::HashMap::new(), live: 0 } }
    fn hash_of<Q: std::hash::Hash + ?Sized>(k: &Q) -> u64 {
        use std::hash::Hasher;
        let mut h = std::collections::hash_map::DefaultHasher::new();
        k.hash(&mut h);
        h.finish()
    }
    fn position<Q>(&self, k: &Q) -> Option<usize> where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized {
        let h = Self::hash_of(k);
        self.index.get(&h)?.iter().copied().find(|&i| matches!(&self.entries[i], Some((ek, _)) if ek.borrow() == k))
    }
    fn get<Q>(&self, k: &Q) -> Option<&V> where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized {
        self.position(k).and_then(|i| self.entries[i].as_ref().map(|(_, v)| v))
    }
    fn get_mut<Q>(&mut self, k: &Q) -> Option<&mut V> where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized {
        let i = self.position(k)?;
        self.entries[i].as_mut().map(|(_, v)| v)
    }
    fn contains_key<Q>(&self, k: &Q) -> bool where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized { self.position(k).is_some() }
    fn insert(&mut self, k: K, v: V) -> Option<V> {
        if let Some(i) = self.position(&k) {
            return self.entries[i].as_mut().map(|e| std::mem::replace(&mut e.1, v));
        }
        let h = Self::hash_of(&k);
        self.index.entry(h).or_default().push(self.entries.len());
        self.entries.push(Some((k, v)));
        self.live += 1;
        None
    }
    /// The value for `k`, inserting `default` first if absent.
    fn entry_or_insert(&mut self, k: K, default: V) -> &mut V {
        let i = match self.position(&k) {
            Some(i) => i,
            None => {
                let h = Self::hash_of(&k);
                self.index.entry(h).or_default().push(self.entries.len());
                self.entries.push(Some((k, default)));
                self.live += 1;
                self.entries.len() - 1
            }
        };
        &mut self.entries[i].as_mut().unwrap().1
    }
    fn remove<Q>(&mut self, k: &Q) -> Option<V> where K: std::borrow::Borrow<Q>, Q: std::hash::Hash + Eq + ?Sized {
        let i = self.position(k)?;
        let h = Self::hash_of(k);
        if let Some(v) = self.index.get_mut(&h) { v.retain(|&j| j != i); }
        let (_, v) = self.entries[i].take()?;
        self.live -= 1;
        if self.entries.len() > 8 && self.live * 2 < self.entries.len() { self.compact(); }
        Some(v)
    }
    fn compact(&mut self) {
        let old = std::mem::take(&mut self.entries);
        self.index.clear();
        self.live = 0;
        for e in old.into_iter().flatten() { self.insert(e.0, e.1); }
    }
    fn len(&self) -> usize { self.live }
    fn is_empty(&self) -> bool { self.live == 0 }
    fn iter(&self) -> impl Iterator<Item = (&K, &V)> { self.entries.iter().flatten().map(|(k, v)| (k, v)) }
    fn keys(&self) -> impl Iterator<Item = &K> { self.iter().map(|(k, _)| k) }
    fn values(&self) -> impl Iterator<Item = &V> { self.iter().map(|(_, v)| v) }
    fn from<const N: usize>(pairs: [(K, V); N]) -> Self {
        let mut m = Self::new();
        for (k, v) in pairs { m.insert(k, v); }
        m
    }
}
impl<K: std::hash::Hash + Eq + Clone, V: Clone + PartialEq> PartialEq for LumeMap<K, V> {
    fn eq(&self, other: &Self) -> bool {
        self.len() == other.len() && self.iter().all(|(k, v)| other.get(k) == Some(v))
    }
}
impl<K: LumeShow + std::hash::Hash + Eq + Clone, V: LumeShow + Clone> LumeShow for LumeMap<K, V> {
    fn lume_str(&self) -> String {
        format!("{{{}}}", self.iter().map(|(k, v)| format!("{}: {}", k.lume_str(), v.lume_str())).collect::<Vec<_>>().join(", "))
    }
}
impl<K: std::hash::Hash + Eq + Clone, V: Clone> LumeLen for LumeMap<K, V> { fn lume_len(&self) -> i64 { self.len() as i64 } }
impl<K: std::hash::Hash + Eq + Clone, V: Clone> LumeEmpty for LumeMap<K, V> { fn lume_empty(&self) -> bool { self.is_empty() } }
/// `split` that drops trailing empty pieces (Ruby), without collecting.
struct LumeSplit<'a> { inner: std::str::Split<'a, &'a str>, pending_empty: usize, buffered: Option<&'a str>, done: bool }
impl<'a> LumeSplit<'a> {
    fn new(inner: std::str::Split<'a, &'a str>) -> Self { LumeSplit { inner, pending_empty: 0, buffered: None, done: false } }
}
impl<'a> Iterator for LumeSplit<'a> {
    type Item = &'a str;
    fn next(&mut self) -> Option<&'a str> {
        if self.pending_empty > 0 { self.pending_empty -= 1; return Some(""); }
        if let Some(b) = self.buffered.take() { return Some(b); }
        if self.done { return None; }
        loop {
            match self.inner.next() {
                None => { self.done = true; self.pending_empty = 0; return None; }
                Some("") => { self.pending_empty += 1; }
                Some(piece) => {
                    if self.pending_empty > 0 { self.pending_empty -= 1; self.buffered = Some(piece); return Some(""); }
                    return Some(piece);
                }
            }
        }
    }
}
fn lume_pad<T: LumeShow>(x: T, width: i64) -> String { format!("{:>w$}", x.lume_str(), w = width.max(0) as usize) }
fn lume_now() -> i64 {
    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_secs() as i64).unwrap_or(0)
}
fn lume_split(s: &str, sep: &str) -> Vec<String> {
    let mut v: Vec<String> = s.split(sep).map(|x| x.to_string()).collect();
    while v.last().map(|x| x.is_empty()).unwrap_or(false) { v.pop(); }
    v
}
// ---- end prelude ----
"#;

const RUST_RESERVED: &[&str] = &[
    "as", "async", "await", "box", "crate", "dyn", "extern", "fn", "impl", "let", "loop", "macro",
    "mod", "move", "mut", "ref", "static", "super", "trait", "type", "union", "unsafe", "use",
    "yield", "try", "Self", "self", "abstract", "become", "final", "override", "priv", "typeof",
    "unsized", "virtual", "where", "continue", "match", "enum", "struct", "pub", "const", "if",
    "else", "while", "for", "in", "return", "break", "true", "false", "String", "Vec", "Option",
    "Result", "Box", "Some", "None", "Ok", "Err",
];

#[derive(Clone)]
struct Binding {
    mutable: bool,
    borrowed: bool,
    ty: Type,
    line: usize,
}

#[derive(Clone)]
struct Sig {
    params: Vec<(String, Type)>,
    /// Which parameters are `var` (passed as `&mut`).
    var_params: Vec<bool>,
    ret: Type,
    self_kind: SelfKind,
}

#[derive(Clone)]
struct StructInfo {
    fields: Vec<(String, Type)>,
    methods: HashMap<String, Sig>,
}

#[derive(Clone)]
struct EnumInfo {
    variants: Vec<(String, Vec<(String, Type)>)>,
    methods: HashMap<String, Sig>,
}

#[derive(Clone)]
struct IfaceInfo {
    methods: HashMap<String, Sig>,
    required: Vec<String>,
    /// Methods with a body, emitted inside the trait.
    defaults: Vec<FnDef>,
    /// Which module owns the interface (for the emit-once rule).
    local: bool,
}

/// Why a type does or does not satisfy an interface.
#[derive(Clone)]
enum Conformance {
    Yes,
    Missing(Vec<String>),
    Mismatch { method: String, expected: String, actual: String },
}

/// A compiled pattern: the Rust pattern text, extra guard conditions (for
/// string and float literals, which Rust cannot match directly), and the
/// names it binds with their types.
struct CompiledPat {
    text: String,
    guards: Vec<String>,
    binds: Vec<(String, Type, BindKind)>,
}

#[derive(Clone, Copy, PartialEq)]
enum BindKind {
    /// Copy value reached through a reference: rebind with `let x = *x;`
    Deref,
    /// Reference into the scrutinee
    Ref,
    /// Slice from `..rest`: rebind with `let rest = rest.to_vec();`
    Slice,
    /// Owned value (scrutinee matched by value)
    Owned,
}

/// What a module offers to the files that import it.
#[derive(Clone, Default)]
pub struct Exports {
    pub rust_mod: String,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    fns: HashMap<String, Sig>,
    interfaces: HashMap<String, IfaceInfo>,
    /// Methods types gained through `extend` in that module: type key -> methods.
    ext_methods: HashMap<String, HashMap<String, Sig>>,
    private: Vec<String>,
}

/// One import of this module, already resolved to a file by the loader.
pub enum Dep<'a> {
    /// `import users.model [as m]`: the whole module under an alias
    Module { alias: String, id: String, exports: &'a Exports, line: usize, col: usize },
    /// `import users.model.User [as U]`: one public item under a local name
    Single { local: String, id: String, item: String, exports: &'a Exports, line: usize, col: usize },
}

pub struct Output {
    pub rust: String,
    pub warnings: Vec<LumeError>,
    /// Cargo dependencies from `import rust.<crate>`: (crate, version).
    pub deps: Vec<(String, String)>,
    /// The program contains `rust:` blocks, so rustc errors may be the user's.
    pub has_rust_blocks: bool,
}

pub struct Gen {
    out: String,
    indent: usize,
    scopes: Vec<HashMap<String, Binding>>,
    fns: HashMap<String, Sig>,
    structs: HashMap<String, StructInfo>,
    enums: HashMap<String, EnumInfo>,
    interfaces: HashMap<String, IfaceInfo>,
    /// Methods a type gained through `extend`: type key -> name -> signature.
    ext_methods: HashMap<String, HashMap<String, Sig>>,
    /// `extend` targets that are built-in types: key ("Str") -> the type.
    ext_targets: HashMap<String, Type>,
    /// Local types (struct/enum names) — interfaces are emitted for these.
    local_types: HashSet<String>,
    /// The struct or enum whose method is being generated, if any.
    current_type: Option<String>,
    /// The type of `self` when it is a built-in (inside `extend Str with ...`).
    current_self_ty: Option<Type>,
    /// Set while emitting methods that live inside a trait or trait impl (no `pub`).
    in_trait_impl: bool,
    current_self: SelfKind,
    current_ret: Type,
    current_fn: String,
    loop_depth: usize,
    in_block: bool,
    /// True while emitting statements whose value is the function's result
    /// (so `T or E` returns get their implied `Ok`/`Err`).
    tail_of_fn: bool,
    /// Set just before emitting an if/match that sits in tail position.
    at_tail: bool,
    tmp: usize,
    pub warnings: Vec<LumeError>,
    has_rust_blocks: bool,
    /// Statements that follow the one being emitted, in the same block.
    rest_of_block: Vec<Stmt>,
    /// Rust paths of imported items, keyed by canonical id ("users.model.User").
    paths: HashMap<String, String>,
    /// How this module spells imported items -> canonical id
    /// ("model.User" and a single-imported "User" -> "users.model.User").
    canon: HashMap<String, String>,
    /// Module aliases in scope (`model` for `import users.model`).
    module_aliases: HashMap<String, String>,
    /// Private names of imported modules, for good error messages: alias -> names.
    module_private: HashMap<String, Vec<String>>,
    /// True for the file that holds `main`.
    is_entry: bool,
}

pub fn generate(program: &[Item]) -> Result<Output> {
    generate_module(program, None, &[]).map(|(o, _)| o)
}

/// Compiles one module. `rust_mod` is `Some(name)` for an imported file,
/// which is emitted as `mod name { ... }`; `None` for the entry file.
pub fn generate_module(program: &[Item], rust_mod: Option<&str>, deps: &[Dep]) -> Result<(Output, Exports)> {
    let mut g = Gen {
        out: String::new(),
        indent: 0,
        scopes: Vec::new(),
        fns: HashMap::new(),
        structs: HashMap::new(),
        enums: HashMap::new(),
        interfaces: HashMap::new(),
        ext_methods: HashMap::new(),
        ext_targets: HashMap::new(),
        local_types: HashSet::new(),
        current_type: None,
        current_self_ty: None,
        in_trait_impl: false,
        current_self: SelfKind::Read,
        current_ret: Type::Unit,
        current_fn: String::new(),
        loop_depth: 0,
        in_block: false,
        tail_of_fn: false,
        at_tail: false,
        tmp: 0,
        warnings: Vec::new(),
        has_rust_blocks: false,
        rest_of_block: Vec::new(),
        paths: HashMap::new(),
        canon: HashMap::new(),
        module_aliases: HashMap::new(),
        module_private: HashMap::new(),
        is_entry: true,
    };
    // The built-in Error type: a struct with one field, defined in the prelude.
    g.structs.insert("Error".into(), StructInfo { fields: vec![("message".into(), Type::Str)], methods: HashMap::new() });
    g.is_entry = rust_mod.is_none();
    g.register_deps(deps)?;
    g.program(program)?;
    let mut rust_deps = Vec::new();
    for item in program {
        if let Item::Import(imp) = item {
            if imp.is_rust {
                rust_deps.push((imp.krate().to_string(), imp.version.clone().unwrap_or_else(|| "*".into())));
            }
        }
    }
    let exports = g.exports(program, rust_mod.unwrap_or("main"));
    let rust = match rust_mod {
        Some(m) => {
            let body = g.out.lines().map(|l| if l.is_empty() { String::new() } else { format!("    {}", l) }).collect::<Vec<_>>().join("\n");
            format!("pub mod {} {{\n    use super::*;\n{}\n}}\n", m, body)
        }
        None => g.out,
    };
    Ok((Output { rust, warnings: g.warnings, deps: rust_deps, has_rust_blocks: g.has_rust_blocks }, exports))
}

/// Rewrites a type from module `id` so its named types are keyed the way the
/// importing module sees them (`User` -> `users.model.User`).
fn qualify_type(t: &Type, id: &str, ex: &Exports) -> Type {
    match t {
        Type::Named(n) if n != "Error" && (ex.structs.contains_key(n) || ex.enums.contains_key(n)) => Type::Named(format!("{}.{}", id, n)),
        Type::List(i) => Type::List(Box::new(qualify_type(i, id, ex))),
        Type::Option(i) => Type::Option(Box::new(qualify_type(i, id, ex))),
        Type::Iter(i, b) => Type::Iter(Box::new(qualify_type(i, id, ex)), *b),
        Type::Tuple(ts) => Type::Tuple(ts.iter().map(|x| qualify_type(x, id, ex)).collect()),
        Type::Result(a, b) => Type::Result(Box::new(qualify_type(a, id, ex)), Box::new(qualify_type(b, id, ex))),
        Type::Map(a, b) => Type::Map(Box::new(qualify_type(a, id, ex)), Box::new(qualify_type(b, id, ex))),
        other => other.clone(),
    }
}

fn qualify_sig(sig: &Sig, id: &str, ex: &Exports) -> Sig {
    Sig {
        params: sig.params.iter().map(|(n, t)| (n.clone(), qualify_type(t, id, ex))).collect(),
        var_params: sig.var_params.clone(),
        ret: qualify_type(&sig.ret, id, ex),
        self_kind: sig.self_kind,
    }
}

fn rust_name(name: &str) -> String {
    if name == "_" {
        return "lume_it".into();
    }
    if let Some(op) = op_method_name(name) {
        return op.into();
    }
    let base = name.replace('?', "_q");
    if RUST_RESERVED.contains(&base.as_str()) {
        if base == "self" || base == "Self" || base == "crate" || base == "super" {
            format!("lume_{}", base)
        } else {
            format!("r#{}", base)
        }
    } else {
        base
    }
}

pub fn rust_type(t: &Type) -> String {
    match t {
        Type::Int => "i64".into(),
        Type::Float => "f64".into(),
        Type::Bool => "bool".into(),
        Type::Str => "String".into(),
        Type::Unit => "()".into(),
        Type::List(inner) => format!("Vec<{}>", rust_type(inner)),
        Type::Named(n) => n.clone(),
        Type::Option(inner) => format!("Option<{}>", rust_type(inner)),
        Type::Tuple(ts) => format!("({})", ts.iter().map(rust_type).collect::<Vec<_>>().join(", ")),
        Type::Result(t, e) => format!("Result<{}, {}>", rust_type(t), rust_type(e)),
        Type::Map(k, v) => format!("LumeMap<{}, {}>", rust_type(k), rust_type(v)),
        Type::Iter(inner, _) => format!("Vec<{}>", rust_type(inner)),
        Type::Unknown => "_".into(),
    }
}

pub fn type_name(t: &Type) -> String {
    match t {
        Type::Int => "Int".into(),
        Type::Float => "Float".into(),
        Type::Bool => "Bool".into(),
        Type::Str => "Str".into(),
        Type::Unit => "()".into(),
        Type::List(i) => format!("[{}]", type_name(i)),
        Type::Named(n) => n.clone(),
        Type::Option(i) => format!("{}?", type_name(i)),
        Type::Tuple(ts) => format!("({})", ts.iter().map(type_name).collect::<Vec<_>>().join(", ")),
        Type::Result(t, e) => format!("{} or {}", type_name(t), type_name(e)),
        Type::Map(k, v) => format!("{{{}: {}}}", type_name(k), type_name(v)),
        Type::Iter(i, _) => format!("[{}]", type_name(i)),
        Type::Unknown => "?".into(),
    }
}

fn escape_rust_str(s: &str, for_format: bool) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            '{' if for_format => out.push_str("{{"),
            '}' if for_format => out.push_str("}}"),
            other => out.push(other),
        }
    }
    out
}

fn plural(n: usize, one: &str, many: &str) -> String {
    if n == 1 { one.to_string() } else { many.to_string() }
}

impl Gen {
    /// Rust spelling of a type; names of imported items go through their module path.
    fn rt(&self, t: &Type) -> String {
        match t {
            Type::Named(_) if self.is_interface(t) => format!("Box<dyn {}>", self.path_of(match t { Type::Named(n) => n, _ => unreachable!() })),
            Type::Named(n) => self.path_of(n),
            Type::List(inner) => format!("Vec<{}>", self.rt(inner)),
            Type::Option(inner) => format!("Option<{}>", self.rt(inner)),
            Type::Tuple(ts) => format!("({})", ts.iter().map(|x| self.rt(x)).collect::<Vec<_>>().join(", ")),
            Type::Result(a, b) => format!("Result<{}, {}>", self.rt(a), self.rt(b)),
            Type::Map(k, v) => format!("LumeMap<{}, {}>", self.rt(k), self.rt(v)),
            Type::Iter(inner, _) => format!("Vec<{}>", self.rt(inner)),
            other => rust_type(other),
        }
    }

    /// The Rust path of a struct/enum/function key as seen from this module.
    fn path_of(&self, key: &str) -> String {
        let key = self.canon(key);
        match self.paths.get(&key) {
            Some(p) => p.clone(),
            None => key,
        }
    }

    /// Canonical id of a name as written in this module.
    fn canon(&self, name: &str) -> String {
        self.canon.get(name).cloned().unwrap_or_else(|| name.to_string())
    }

    /// A type with its named parts canonicalised.
    fn ct(&self, t: &Type) -> Type {
        match t {
            Type::Named(n) => Type::Named(self.canon(n)),
            Type::List(i) => Type::List(Box::new(self.ct(i))),
            Type::Option(i) => Type::Option(Box::new(self.ct(i))),
            Type::Iter(i, b) => Type::Iter(Box::new(self.ct(i)), *b),
            Type::Tuple(ts) => Type::Tuple(ts.iter().map(|x| self.ct(x)).collect()),
            Type::Result(a, b) => Type::Result(Box::new(self.ct(a)), Box::new(self.ct(b))),
            Type::Map(a, b) => Type::Map(Box::new(self.ct(a)), Box::new(self.ct(b))),
            other => other.clone(),
        }
    }

    fn sig_of(&self, f: &FnDef) -> Sig {
        Sig {
            params: f.params.iter().map(|p| (p.name.clone(), self.ct(&p.ty))).collect(),
            var_params: f.params.iter().map(|p| p.mutable).collect(),
            ret: f.ret.as_ref().map(|t| self.ct(t)).unwrap_or(Type::Unknown),
            self_kind: f.self_kind,
        }
    }

    // ----- output helpers -------------------------------------------------

    fn line(&mut self, s: &str) {
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
        self.out.push_str(s);
        self.out.push('\n');
    }

    fn fresh(&mut self, base: &str) -> String {
        self.tmp += 1;
        format!("__{}{}", base, self.tmp)
    }

    // ----- scopes -----------------------------------------------------------

    fn push_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    fn pop_scope(&mut self) {
        self.scopes.pop();
    }

    fn lookup(&self, name: &str) -> Option<&Binding> {
        for s in self.scopes.iter().rev() {
            if let Some(b) = s.get(name) {
                return Some(b);
            }
        }
        None
    }

    fn declare(&mut self, name: &str, mutable: bool, borrowed: bool, ty: Type, line: usize) {
        self.scopes.last_mut().unwrap().insert(name.to_string(), Binding { mutable, borrowed, ty, line });
    }

    fn field_type(&self, name: &str) -> Option<Type> {
        let s = self.current_type.as_ref()?;
        let info = self.structs.get(s)?;
        info.fields.iter().find(|(n, _)| n == name).map(|(_, t)| t.clone())
    }

    /// A zero-argument method of the current type, callable bare inside
    /// another method (`total` for `def total`).
    fn bare_method(&self, name: &str) -> Option<Sig> {
        let t = self.current_type.as_ref()?;
        let ms = self.methods_of(t)?;
        let m = ms.get(name)?;
        if m.params.is_empty() { Some(m.clone()) } else { None }
    }

    /// Methods callable on a type: inherent ones plus those from `extend`;
    /// for an interface, its methods; for a built-in, only `extend` methods.
    fn methods_of(&self, tname: &str) -> Option<HashMap<String, Sig>> {
        let mut m: HashMap<String, Sig> = if let Some(s) = self.structs.get(tname) {
            s.methods.clone()
        } else if let Some(e) = self.enums.get(tname) {
            e.methods.clone()
        } else if let Some(i) = self.interfaces.get(tname) {
            i.methods.clone()
        } else if self.ext_methods.contains_key(tname) {
            HashMap::new()
        } else {
            return None;
        };
        if let Some(ext) = self.ext_methods.get(tname) {
            for (k, v) in ext {
                m.entry(k.clone()).or_insert_with(|| v.clone());
            }
        }
        Some(m)
    }

    fn is_type(&self, name: &str) -> bool {
        self.structs.contains_key(name) || self.enums.contains_key(name) || self.interfaces.contains_key(name)
    }

    fn is_interface(&self, t: &Type) -> bool {
        matches!(t, Type::Named(n) if self.interfaces.contains_key(&self.canon(n)))
    }

    /// The key under which `extend` methods and conformance are recorded
    /// for a type: its name for structs/enums, its spelling for built-ins.
    fn type_key(&self, t: &Type) -> String {
        match t {
            Type::Named(n) => self.canon(n),
            other => type_name(other),
        }
    }

    /// Does `t` satisfy interface `iface`? Structural: every required
    /// method exists with the same parameters and result; defaults that the
    /// type also defines must match too.
    fn conformance(&self, t: &Type, iface: &str) -> Conformance {
        let iface = self.canon(iface);
        let info = match self.interfaces.get(&iface) {
            Some(i) => i,
            None => return Conformance::Missing(vec![]),
        };
        if self.is_interface(t) {
            return if self.type_key(t) == iface { Conformance::Yes } else { Conformance::Missing(info.required.clone()) };
        }
        let have = self.methods_of(&self.type_key(t)).unwrap_or_default();
        let mut missing = Vec::new();
        for (name, want) in &info.methods {
            match have.get(name) {
                None => {
                    if info.required.contains(name) {
                        missing.push(name.clone());
                    }
                }
                Some(got) => {
                    let same = got.params.len() == want.params.len()
                        && got.params.iter().zip(&want.params).all(|((_, a), (_, b))| a == b || *a == Type::Unknown || *b == Type::Unknown)
                        && (got.ret == want.ret || got.ret == Type::Unknown || want.ret == Type::Unknown);
                    if !same {
                        return Conformance::Mismatch { method: name.clone(), expected: self.describe_sig(want), actual: self.describe_sig(got) };
                    }
                }
            }
        }
        missing.sort();
        if missing.is_empty() { Conformance::Yes } else { Conformance::Missing(missing) }
    }

    fn describe_sig(&self, s: &Sig) -> String {
        let ps: Vec<String> = s.params.iter().map(|(n, t)| format!("{}: {}", n, type_name(t))).collect();
        if ps.is_empty() { format!("-> {}", type_name(&s.ret)) } else { format!("({}) -> {}", ps.join(", "), type_name(&s.ret)) }
    }

    /// Error for a value of type `t` used where interface `iface` is needed.
    fn require_conforms(&self, t: &Type, iface: &str, line: usize, col: usize) -> Result<()> {
        match self.conformance(t, iface) {
            Conformance::Yes => Ok(()),
            Conformance::Missing(m) => {
                let info = &self.interfaces[&self.canon(iface)];
                let needs: Vec<String> = m.iter().filter_map(|n| info.methods.get(n).map(|s| format!("def {} {}", n, self.describe_sig(s)))).collect();
                Err(LumeError::new(line, col, format!("`{}` is used as a `{}` here, but it has no `{}` method", type_name(t), iface, m.join("`, `")))
                    .with_help(format!("`{}` needs: {}. Add {} to `{}`, or `extend {} with {}:`", iface, needs.join("; "), if m.len() == 1 { "it" } else { "them" }, type_name(t), type_name(t), iface)))
            }
            Conformance::Mismatch { method, expected, actual } => Err(LumeError::new(
                line,
                col,
                format!("`{}` is used as a `{}`, but its `{}` is `{}` and `{}` needs `{}`", type_name(t), iface, method, actual, iface, expected),
            )),
        }
    }

    /// Boxes a concrete value where an interface value is stored.
    fn coerce(&self, text: String, from: &Type, to: &Type, line: usize, col: usize) -> Result<String> {
        if let Type::Named(iface) = to {
            if self.is_interface(to) && !self.is_interface(from) && *from != Type::Unknown {
                self.require_conforms(from, iface, line, col)?;
                return Ok(format!("(Box::new({}) as Box<dyn {}>)", text, self.path_of(iface)));
            }
        }
        Ok(text)
    }

    /// Emits the one-time note for a declaration whose type holds interface
    /// values behind a pointer.
    fn note_dynamic(&mut self, name: &str, t: &Type, line: usize, col: usize) {
        fn boxed_iface<'a>(g: &'a Gen, t: &'a Type) -> Option<&'a str> {
            match t {
                Type::List(i) | Type::Option(i) | Type::Iter(i, _) => match &**i {
                    Type::Named(n) if g.is_interface(i) => Some(n.as_str()),
                    other => boxed_iface(g, other),
                },
                Type::Map(_, v) => boxed_iface(g, v),
                Type::Tuple(ts) => ts.iter().find_map(|x| boxed_iface(g, x)),
                _ => None,
            }
        }
        if let Some(iface) = boxed_iface(self, t) {
            self.warnings.push(
                LumeError::new(line, col, format!("`{}` holds values of different types behind `{}`, so calls on its items go through a pointer", name, iface))
                    .with_help("fine for most code; use a single concrete type or an enum if this is a hot loop"),
            );
        }
    }

    /// Enums that have a variant with this name.
    fn enums_with_variant(&self, v: &str) -> Vec<String> {
        let mut out: Vec<String> = self.enums.iter().filter(|(_, e)| e.variants.iter().any(|(n, _)| n == v)).map(|(n, _)| n.clone()).collect();
        out.sort_by_key(|k| (k.matches('.').count(), k.clone()));
        let mut seen = HashSet::new();
        out.retain(|k| seen.insert(self.path_of(k)));
        out
    }

    /// Resolves a bare variant name: the current enum's variant first, then
    /// any enum that has it uniquely.
    fn resolve_variant(&self, v: &str, line: usize, col: usize) -> Result<Option<String>> {
        if let Some(t) = &self.current_type {
            if let Some(e) = self.enums.get(t) {
                if e.variants.iter().any(|(n, _)| n == v) {
                    return Ok(Some(t.clone()));
                }
            }
        }
        let owners = self.enums_with_variant(v);
        match owners.len() {
            0 => Ok(None),
            1 => Ok(Some(owners[0].clone())),
            _ => Err(LumeError::new(line, col, format!("`{}` is a variant of more than one enum: {}", v, owners.join(", ")))
                .with_help(format!("write `{}.{}`", owners[0], v))),
        }
    }

    fn all_names(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for s in &self.scopes {
            v.extend(s.keys().cloned());
        }
        v.extend(self.fns.keys().cloned());
        v.extend(self.structs.keys().cloned());
        v.extend(self.enums.keys().cloned());
        if let Some(s) = &self.current_type {
            if let Some(info) = self.structs.get(s) {
                v.extend(info.fields.iter().map(|(n, _)| n.clone()));
            }
            if let Some(m) = self.methods_of(s) {
                v.extend(m.keys().cloned());
            }
        }
        v
    }

    fn suggest_from(&self, name: &str, cands: impl Iterator<Item = String>) -> Option<String> {
        let mut best: Option<(usize, String)> = None;
        for cand in cands {
            let d = edit_distance(name, &cand);
            if d <= 2 && d < name.len() && best.as_ref().map(|b| d < b.0).unwrap_or(true) {
                best = Some((d, cand));
            }
        }
        best.map(|b| b.1)
    }

    fn suggest(&self, name: &str) -> Option<String> {
        self.suggest_from(name, self.all_names().into_iter())
    }

    /// Makes an imported module's public items visible under `key_prefix`
    /// ("model" for an alias, "users.model" for the full id).
    fn register_module_items(&mut self, key_prefix: &str, id: &str, ex: &Exports) {
        for (n, info) in &ex.structs {
            let key = format!("{}.{}", key_prefix, n);
            let qualified = StructInfo {
                fields: info.fields.iter().map(|(f, t)| (f.clone(), qualify_type(t, id, ex))).collect(),
                methods: info.methods.iter().map(|(m, sg)| (m.clone(), qualify_sig(sg, id, ex))).collect(),
            };
            self.structs.insert(key.clone(), qualified);
            self.paths.insert(key, format!("{}::{}", ex.rust_mod, n));
        }
        for (n, info) in &ex.enums {
            let key = format!("{}.{}", key_prefix, n);
            let qualified = EnumInfo {
                variants: info.variants.iter().map(|(v, fs)| (v.clone(), fs.iter().map(|(f, t)| (f.clone(), qualify_type(t, id, ex))).collect())).collect(),
                methods: info.methods.iter().map(|(m, sg)| (m.clone(), qualify_sig(sg, id, ex))).collect(),
            };
            self.enums.insert(key.clone(), qualified);
            self.paths.insert(key, format!("{}::{}", ex.rust_mod, n));
        }
        for (n, sig) in &ex.fns {
            let key = format!("{}.{}", key_prefix, n);
            self.fns.insert(key.clone(), qualify_sig(sig, id, ex));
            self.paths.insert(key, format!("{}::{}", ex.rust_mod, n));
        }
        for (n, info) in &ex.interfaces {
            let key = format!("{}.{}", key_prefix, n);
            let qualified = IfaceInfo {
                methods: info.methods.iter().map(|(m, sg)| (m.clone(), qualify_sig(sg, id, ex))).collect(),
                required: info.required.clone(),
                defaults: Vec::new(),
                local: false,
            };
            self.interfaces.insert(key.clone(), qualified);
            self.paths.insert(key, format!("{}::{}", ex.rust_mod, n));
        }
        for (tkey, ms) in &ex.ext_methods {
            let tkey = if ex.structs.contains_key(tkey) || ex.enums.contains_key(tkey) { format!("{}.{}", key_prefix, tkey) } else { tkey.clone() };
            let entry = self.ext_methods.entry(tkey).or_default();
            for (m, sg) in ms {
                entry.insert(m.clone(), qualify_sig(sg, id, ex));
            }
        }
    }

    fn register_deps(&mut self, deps: &[Dep]) -> Result<()> {
        for d in deps {
            match d {
                Dep::Module { alias, id, exports, line, col } => {
                    if self.module_aliases.contains_key(alias) {
                        return Err(LumeError::new(*line, *col, format!("`{}` is imported twice", alias)));
                    }
                    self.module_aliases.insert(alias.clone(), id.clone());
                    self.module_private.insert(alias.clone(), exports.private.clone());
                    self.register_module_items(id, id, exports);
                    for n in exports.structs.keys().chain(exports.enums.keys()).chain(exports.fns.keys()).chain(exports.interfaces.keys()) {
                        self.canon.insert(format!("{}.{}", alias, n), format!("{}.{}", id, n));
                    }
                }
                Dep::Single { local, id, item, exports, line, col } => {
                    // the module's types must resolve for signatures that mention them
                    self.register_module_items(id, id, exports);
                    let full = format!("{}.{}", id, item);
                    if self.structs.contains_key(&full) || self.enums.contains_key(&full) || self.fns.contains_key(&full) || self.interfaces.contains_key(&full) {
                        self.canon.insert(local.clone(), full.clone());
                    } else if exports.private.contains(item) {
                        return Err(LumeError::new(*line, *col, format!("`{}` exists in module `{}` but is not `pub`", item, id))
                            .with_help(format!("add `pub` in front of its definition in {}.lume", id.replace('.', "/"))));
                    } else {
                        let e = LumeError::new(*line, *col, format!("module `{}` has no `{}`", id, item));
                        let names: Vec<String> = exports.structs.keys().chain(exports.enums.keys()).chain(exports.fns.keys()).cloned().collect();
                        return Err(match self.suggest_from(item, names.iter().cloned()) {
                            Some(sug) => e.with_help(format!("did you mean `{}`?", sug)),
                            None => e.with_help(format!("its public items are: {}", names.join(", "))),
                        });
                    }
                }
            }
        }
        Ok(())
    }

    /// This module's public items, for files that import it.
    fn exports(&self, program: &[Item], rust_mod: &str) -> Exports {
        let mut ex = Exports { rust_mod: rust_mod.to_string(), ..Default::default() };
        for item in program {
            match item {
                Item::Fn(f) if f.public => {
                    ex.fns.insert(f.name.clone(), self.fns[&f.name].clone());
                }
                Item::Fn(f) => ex.private.push(f.name.clone()),
                Item::Struct(st) if st.public => {
                    ex.structs.insert(st.name.clone(), self.structs[&st.name].clone());
                }
                Item::Struct(st) => ex.private.push(st.name.clone()),
                Item::Enum(en) if en.public => {
                    ex.enums.insert(en.name.clone(), self.enums[&en.name].clone());
                }
                Item::Enum(en) => ex.private.push(en.name.clone()),
                Item::Interface(i) if i.public => {
                    ex.interfaces.insert(i.name.clone(), self.interfaces[&i.name].clone());
                }
                Item::Interface(i) => ex.private.push(i.name.clone()),
                Item::Import(_) | Item::Extend(_) => {}
            }
        }
        ex.ext_methods = self.ext_methods.clone();
        ex
    }

    /// A `module.name` reference: the item key if `module` is an imported
    /// module alias, plus a good error when the name is private or missing.
    fn module_item(&self, alias: &str, name: &str, line: usize, col: usize) -> Result<Option<String>> {
        let id = match self.module_aliases.get(alias) {
            Some(id) => id.clone(),
            None => return Ok(None),
        };
        let key = self.canon(&format!("{}.{}", alias, name));
        if self.structs.contains_key(&key) || self.enums.contains_key(&key) || self.fns.contains_key(&key) || self.interfaces.contains_key(&key) {
            return Ok(Some(key));
        }
        if self.module_private.get(alias).map(|p| p.contains(&name.to_string())).unwrap_or(false) {
            return Err(LumeError::new(line, col, format!("`{}` exists in module `{}` but is not `pub`", name, id))
                .with_help(format!("add `pub` in front of its definition in {}.lume", id.replace('.', "/"))));
        }
        let e = LumeError::new(line, col, format!("module `{}` has no `{}`", id, name));
        let prefix = format!("{}.", id);
        let names: Vec<String> = self
            .structs
            .keys()
            .chain(self.enums.keys())
            .chain(self.fns.keys())
            .filter_map(|k| k.strip_prefix(&prefix).map(|s| s.to_string()))
            .filter(|s| !s.contains('.'))
            .collect();
        Err(match self.suggest_from(name, names.iter().cloned()) {
            Some(sug) => e.with_help(format!("did you mean `{}`?", sug)),
            None => e.with_help(format!("its public items are: {}", names.join(", "))),
        })
    }

    // ----- program ----------------------------------------------------------

    fn program(&mut self, program: &[Item]) -> Result<()> {
        // Pass 1: collect types and signatures.
        let mut seen = HashSet::new();
        let mut imported: HashSet<String> = HashSet::new();
        for item in program {
            match item {
                Item::Import(imp) if imp.is_rust => {
                    let key = imp.alias.clone().unwrap_or_else(|| imp.krate().to_string());
                    if !imported.insert(key.clone()) {
                        return Err(LumeError::new(imp.line, imp.col, format!("`{}` is imported twice", key)));
                    }
                }
                Item::Import(_) => {}
                Item::Fn(f) => {
                    if !seen.insert(f.name.clone()) {
                        return Err(LumeError::new(f.line, f.col, format!("function `{}` is defined twice", f.name)));
                    }
                    let sg = self.sig_of(f);
                    self.fns.insert(f.name.clone(), sg);
                }
                Item::Struct(s) => {
                    if s.name == "Error" {
                        return Err(LumeError::new(s.line, s.col, "`Error` is the built-in error type").with_help("name yours differently, or define an `enum` of error kinds and return `T or MyError`"));
                    }
                    if !seen.insert(s.name.clone()) {
                        return Err(LumeError::new(s.line, s.col, format!("`{}` is defined twice", s.name)));
                    }
                    let mut fnames = HashSet::new();
                    for fld in &s.fields {
                        if !fnames.insert(fld.name.clone()) {
                            return Err(LumeError::new(fld.line, fld.col, format!("field `{}` is listed twice in `{}`", fld.name, s.name)));
                        }
                    }
                    let methods = self.collect_methods(&s.methods, &s.name, &fnames)?;
                    let fields = s.fields.iter().map(|p| (p.name.clone(), self.ct(&p.ty))).collect();
                    self.structs.insert(s.name.clone(), StructInfo { fields, methods });
                }
                Item::Enum(e) => {
                    if !seen.insert(e.name.clone()) {
                        return Err(LumeError::new(e.line, e.col, format!("`{}` is defined twice", e.name)));
                    }
                    let methods = self.collect_methods(&e.methods, &e.name, &HashSet::new())?;
                    let variants = e
                        .variants
                        .iter()
                        .map(|v| (v.name.clone(), v.fields.iter().map(|p| (p.name.clone(), self.ct(&p.ty))).collect()))
                        .collect();
                    self.enums.insert(e.name.clone(), EnumInfo { variants, methods });
                    self.local_types.insert(e.name.clone());
                }
                Item::Interface(i) => {
                    if !seen.insert(i.name.clone()) {
                        return Err(LumeError::new(i.line, i.col, format!("`{}` is defined twice", i.name)));
                    }
                    let mut methods = HashMap::new();
                    for m in i.required.iter().chain(&i.defaults) {
                        if m.self_kind == SelfKind::Mutate {
                            return Err(LumeError::new(m.line, m.col, format!("interface method `{}` cannot take `var self`", m.name))
                                .with_help("interfaces describe reading behaviour; mutation stays on the concrete type"));
                        }
                        if methods.insert(m.name.clone(), self.sig_of(m)).is_some() {
                            return Err(LumeError::new(m.line, m.col, format!("method `{}` is listed twice in `{}`", m.name, i.name)));
                        }
                    }
                    self.interfaces.insert(
                        i.name.clone(),
                        IfaceInfo { methods, required: i.required.iter().map(|m| m.name.clone()).collect(), defaults: i.defaults.clone(), local: true },
                    );
                }
                Item::Extend(_) => {}
            }
        }
        for item in program {
            if let Item::Struct(s) = item {
                self.local_types.insert(s.name.clone());
            }
        }
        // `extend Type with Iface:` adds methods to the type.
        for item in program {
            if let Item::Extend(x) = item {
                let target = self.ct(&x.target);
                self.check_type(&target, x.line, x.col)?;
                let iface = self.canon(&x.iface);
                if !self.interfaces.contains_key(&iface) {
                    return Err(LumeError::new(x.line, x.col, format!("unknown interface `{}`", x.iface)));
                }
                if self.is_interface(&target) {
                    return Err(LumeError::new(x.line, x.col, "an interface cannot be extended with another; extend the concrete types"));
                }
                let key = self.type_key(&target);
                if !matches!(target, Type::Named(_)) {
                    self.ext_targets.insert(key.clone(), target.clone());
                }
                let existing = self.methods_of(&key).unwrap_or_default();
                for m in &x.methods {
                    if m.self_kind == SelfKind::Mutate {
                        return Err(LumeError::new(m.line, m.col, "methods in an `extend` block cannot take `var self`"));
                    }
                    if !self.interfaces[&iface].methods.contains_key(&m.name) {
                        return Err(LumeError::new(m.line, m.col, format!("`{}` is not a method of `{}`", m.name, x.iface))
                            .with_help(format!("`{}` has: {}", x.iface, self.interfaces[&iface].methods.keys().cloned().collect::<Vec<_>>().join(", "))));
                    }
                    if existing.contains_key(&m.name) {
                        return Err(LumeError::new(m.line, m.col, format!("`{}` already has a `{}` method", type_name(&target), m.name)));
                    }
                    let sg = self.sig_of(m);
                    self.ext_methods.entry(key.clone()).or_default().insert(m.name.clone(), sg);
                }
            }
        }
        // Check that every named type exists.
        for item in program {
            match item {
                Item::Import(_) => {}
                Item::Fn(f) => self.check_sig_types(f)?,
                Item::Struct(s) => {
                    for fld in &s.fields {
                        self.check_type(&fld.ty, fld.line, fld.col)?;
                    }
                    for m in &s.methods {
                        self.check_sig_types(m)?;
                    }
                }
                Item::Enum(e) => {
                    for v in &e.variants {
                        for fld in &v.fields {
                            self.check_type(&fld.ty, fld.line, fld.col)?;
                        }
                    }
                    for m in &e.methods {
                        self.check_sig_types(m)?;
                    }
                }
                Item::Interface(i) => {
                    for m in i.required.iter().chain(&i.defaults) {
                        self.check_sig_types(m)?;
                        if m.params.iter().any(|p| self.is_interface(&self.ct(&p.ty))) || m.ret.as_ref().map(|r| self.is_interface(&self.ct(r))).unwrap_or(false) {
                            return Err(LumeError::new(m.line, m.col, format!("interface method `{}` mentions an interface in its signature, which is not supported yet", m.name))
                                .with_help("use concrete types in interface signatures for now"));
                        }
                    }
                }
                Item::Extend(x) => {
                    for m in &x.methods {
                        self.check_sig_types(m)?;
                    }
                }
            }
        }
        // Pass 2: infer missing return types (a few rounds so dependencies settle).
        for _round in 0..4 {
            let mut progressed = false;
            for item in program {
                let ext_key;
                let (fns, owner): (Vec<&FnDef>, Option<&String>) = match item {
                    Item::Fn(f) => (vec![f], None),
                    Item::Struct(s) => (s.methods.iter().collect(), Some(&s.name)),
                    Item::Enum(e) => (e.methods.iter().collect(), Some(&e.name)),
                    Item::Interface(i) => (i.defaults.iter().collect(), Some(&i.name)),
                    Item::Extend(x) => {
                        ext_key = self.type_key(&self.ct(&x.target));
                        (x.methods.iter().collect(), Some(&ext_key))
                    }
                    Item::Import(_) => (vec![], None),
                };
                for f in fns {
                    if f.ret.is_some() || self.sig_ret(&f.name, owner) != Type::Unknown {
                        continue;
                    }
                    let t = self.infer_ret(f, owner);
                    if t != Type::Unknown {
                        self.set_sig_ret(&f.name, owner, t);
                        progressed = true;
                    }
                }
            }
            if !progressed {
                break;
            }
        }
        for item in program {
            let ext_key;
            let (fns, owner): (Vec<&FnDef>, Option<&String>) = match item {
                Item::Fn(f) => (vec![f], None),
                Item::Struct(s) => (s.methods.iter().collect(), Some(&s.name)),
                Item::Enum(e) => (e.methods.iter().collect(), Some(&e.name)),
                Item::Interface(i) => (i.defaults.iter().collect(), Some(&i.name)),
                Item::Extend(x) => {
                    ext_key = self.type_key(&self.ct(&x.target));
                    (x.methods.iter().collect(), Some(&ext_key))
                }
                Item::Import(_) => (vec![], None),
            };
            for f in fns {
                if self.sig_ret(&f.name, owner) == Type::Unknown {
                    return Err(LumeError::new(f.line, f.col, format!("cannot work out what `{}` returns", f.name))
                        .with_help(format!("add the return type to the signature: `def {}(...) -> Type`", f.name)));
                }
            }
        }

        // Pass 3: emit. The prelude is shared: only the entry file carries it.
        if self.is_entry {
            self.out.push_str(PRELUDE);
            self.out.push('\n');
        }
        for item in program {
            if let Item::Import(imp) = item {
                if !imp.is_rust {
                    continue;
                }
                match &imp.alias {
                    Some(a) => self.line(&format!("use {} as {};", imp.krate(), a)),
                    None => self.line(&format!("use {};", imp.krate())),
                }
            }
        }
        self.out.push('\n');
        for item in program {
            match item {
                Item::Fn(f) => self.fn_def(f, None)?,
                Item::Struct(s) => self.struct_def(s)?,
                Item::Enum(e) => self.enum_def(e)?,
                Item::Interface(i) => self.interface_def(i)?,
                Item::Import(_) | Item::Extend(_) => continue,
            }
            self.out.push('\n');
        }
        self.emit_conformances(program)?;
        if self.is_entry && !self.fns.contains_key("main") {
            return Err(LumeError::new(1, 1, "no `main` function").with_help("a program starts at `def main:`"));
        }
        Ok(())
    }

    fn sig_ret(&self, name: &str, owner: Option<&String>) -> Type {
        match owner {
            None => self.fns[name].ret.clone(),
            Some(o) => self.methods_of(o).and_then(|m| m.get(name).map(|s| s.ret.clone())).unwrap_or(Type::Unknown),
        }
    }

    fn set_sig_ret(&mut self, name: &str, owner: Option<&String>, t: Type) {
        match owner {
            None => self.fns.get_mut(name).unwrap().ret = t,
            Some(o) => {
                if let Some(s) = self.structs.get_mut(o).and_then(|s| s.methods.get_mut(name)) {
                    s.ret = t;
                } else if let Some(s) = self.enums.get_mut(o).and_then(|e| e.methods.get_mut(name)) {
                    s.ret = t;
                } else if let Some(s) = self.interfaces.get_mut(o).and_then(|i| i.methods.get_mut(name)) {
                    s.ret = t;
                } else if let Some(s) = self.ext_methods.get_mut(o).and_then(|m| m.get_mut(name)) {
                    s.ret = t;
                }
            }
        }
    }

    fn check_type(&self, t: &Type, line: usize, col: usize) -> Result<()> {
        match t {
            Type::Named(n) if !self.is_type(&self.canon(n)) => {
                let e = LumeError::new(line, col, format!("unknown type `{}`", n));
                let cands = self
                    .structs
                    .keys()
                    .cloned()
                    .chain(self.enums.keys().cloned())
                    .chain(self.interfaces.keys().cloned())
                    .chain(["Int", "Float", "Bool", "Str"].iter().map(|s| s.to_string()));
                Err(match self.suggest_from(n, cands) {
                    Some(s) => e.with_help(format!("did you mean `{}`?", s)),
                    None => e.with_help("built-in types are Int, Float, Bool, Str, [T], T? and tuples; others must be a `struct` or `enum`"),
                })
            }
            Type::List(inner) | Type::Option(inner) => self.check_type(inner, line, col),
            Type::Tuple(ts) => ts.iter().try_for_each(|t| self.check_type(t, line, col)),
            Type::Result(t, e) | Type::Map(t, e) => {
                self.check_type(t, line, col)?;
                self.check_type(e, line, col)
            }
            _ => Ok(()),
        }
    }

    fn check_sig_types(&self, f: &FnDef) -> Result<()> {
        for p in &f.params {
            self.check_type(&p.ty, p.line, p.col)?;
        }
        if let Some(r) = &f.ret {
            self.check_type(r, f.line, f.col)?;
        }
        Ok(())
    }

    /// Type of the function's tail expression, evaluated in a scope holding
    /// only its parameters (and, for a method, its fields).
    fn infer_ret(&mut self, f: &FnDef, owner: Option<&String>) -> Type {
        if f.name == "main" && owner.is_none() {
            return Type::Unit;
        }
        self.push_scope();
        let saved = self.current_type.clone();
        let saved_self = self.current_self_ty.clone();
        self.current_type = owner.cloned();
        self.current_self_ty = owner.and_then(|o| self.ext_targets.get(o).cloned());
        for p in &f.params {
            let pty = self.ct(&p.ty);
            self.declare(&p.name, false, !pty.is_copy(), pty, p.line);
        }
        let t = self.tail_type(&f.body);
        self.current_type = saved;
        self.current_self_ty = saved_self;
        self.pop_scope();
        t
    }

    fn tail_type(&mut self, b: &Block) -> Type {
        self.push_scope();
        let mut t = Type::Unit;
        let n = b.stmts.len();
        for (i, s) in b.stmts.iter().enumerate() {
            let last = i + 1 == n;
            match s {
                Stmt::Bind { name, ty, value, line, .. } | Stmt::Var { name, ty, value, line, .. } => {
                    let vt = ty.as_ref().map(|t| self.ct(t)).unwrap_or_else(|| self.ty_of(value).materialized());
                    let mutable = matches!(s, Stmt::Var { .. });
                    if self.lookup(name).is_none() || mutable {
                        self.declare(name, mutable, false, vt, *line);
                    }
                    if last {
                        t = Type::Unit;
                    }
                }
                Stmt::Expr(e) if last => {
                    t = match &e.kind {
                        ExprKind::If { branches, else_block } => {
                            let mut bt = self.tail_type(&branches[0].1);
                            if bt == Type::Unknown {
                                if let Some(eb) = else_block {
                                    bt = self.tail_type(eb);
                                }
                            }
                            if else_block.is_none() { Type::Unit } else { bt }
                        }
                        ExprKind::Match { scrutinee, arms } => self.match_type(scrutinee, arms),
                        _ => self.ty_of(e).materialized(),
                    };
                }
                _ => {
                    if last {
                        t = Type::Unit;
                    }
                }
            }
        }
        self.pop_scope();
        t
    }

    /// Type of a `match`: the first arm whose body has a known type, with
    /// the arm's pattern bindings in scope.
    fn match_type(&mut self, scrutinee: &Expr, arms: &[MatchArm]) -> Type {
        let st = self.ty_of(scrutinee);
        for arm in arms {
            self.push_scope();
            let _ = self.declare_pattern_types(&arm.pat, &st);
            let t = self.tail_type(&arm.body);
            self.pop_scope();
            if t != Type::Unknown {
                return t;
            }
        }
        Type::Unknown
    }

    /// Declares the names a pattern binds, with their types, in the current
    /// scope (used for inference only; codegen does its own declaring).
    fn declare_pattern_types(&mut self, p: &Pattern, t: &Type) -> Result<()> {
        match &p.kind {
            PatKind::Bind(n) => {
                self.declare(n, false, !t.is_copy(), t.clone(), p.line);
            }
            PatKind::Variant { enum_name, name, args } => {
                match t {
                    Type::Option(inner) => {
                        if let Some(a) = args.first() {
                            self.declare_pattern_types(a, inner)?;
                        }
                    }
                    Type::Result(ok_t, err_t) => {
                        if let Some(a) = args.first() {
                            let inner = if name == "Ok" { ok_t } else { err_t };
                            self.declare_pattern_types(a, inner)?;
                        }
                    }
                    Type::Named(en) => {
                        let _ = enum_name;
                        if let Some(info) = self.enums.get(en).cloned() {
                            if let Some((_, fields)) = info.variants.iter().find(|(n, _)| n == name) {
                                for (a, (_, ft)) in args.iter().zip(fields) {
                                    self.declare_pattern_types(a, ft)?;
                                }
                            }
                        }
                    }
                    _ => {
                        for a in args {
                            self.declare_pattern_types(a, &Type::Unknown)?;
                        }
                    }
                }
            }
            PatKind::Tuple(items) => {
                if let Type::Tuple(ts) = t {
                    for (a, at) in items.iter().zip(ts) {
                        self.declare_pattern_types(a, at)?;
                    }
                } else {
                    for a in items {
                        self.declare_pattern_types(a, &Type::Unknown)?;
                    }
                }
            }
            PatKind::List { items, rest } => {
                let elem = match t {
                    Type::List(e) => (**e).clone(),
                    _ => Type::Unknown,
                };
                for a in items {
                    self.declare_pattern_types(a, &elem)?;
                }
                if let Some(Some(r)) = rest {
                    self.declare(r, false, false, Type::List(Box::new(elem)), p.line);
                }
            }
            _ => {}
        }
        Ok(())
    }

    /// Rewrites `model.f(x)` to a call of the key `model.f`, `model.User(..)`
    /// to that constructor, and `model.Shape.Circle(..)` to a variant of the
    /// key `model.Shape`, when `model` is an imported module alias.
    fn module_ref(&mut self, e: &Expr) -> Result<Option<Expr>> {
        let (recv, name, args) = match &e.kind {
            ExprKind::Method { recv, name, args } => (recv, name, args),
            _ => return Ok(None),
        };
        // model.Enum.Variant
        if let ExprKind::Method { recv: r2, name: tname, args: a2 } = &recv.kind {
            if a2.is_empty() {
                if let ExprKind::Ident(alias) = &r2.kind {
                    if self.lookup(alias).is_none() && self.module_aliases.contains_key(alias) {
                        let key = self.canon(&format!("{}.{}", alias, tname));
                        if self.enums.contains_key(&key) {
                            let en = Expr::new(ExprKind::Ident(key), recv.line, recv.col);
                            return Ok(Some(Expr::new(ExprKind::Method { recv: Box::new(en), name: name.clone(), args: args.clone() }, e.line, e.col)));
                        }
                    }
                }
            }
        }
        if let ExprKind::Ident(alias) = &recv.kind {
            if self.lookup(alias).is_none() && self.module_aliases.contains_key(alias) {
                let key = match self.module_item(alias, name, e.line, e.col)? {
                    Some(k) => k,
                    None => return Ok(None),
                };
                if self.enums.contains_key(&key) {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is an enum; pick a variant like `{}.{}`", key, key, self.enums[&key].variants[0].0)));
                }
                return Ok(Some(Expr::new(ExprKind::Call { name: key, args: args.clone() }, e.line, e.col)));
            }
        }
        Ok(None)
    }

    /// `xs.map(f)` with `f` a function name: the same as `xs.map { |x| f(x) }`.
    fn fn_ref_as_block(&self, e: &Expr) -> Option<Expr> {
        let (recv, name, args) = match &e.kind {
            ExprKind::Method { recv, name, args } => (recv, name, args),
            _ => return None,
        };
        const BLOCK_METHODS: &[&str] = &["map", "filter", "reject", "each", "sum", "count", "any?", "all?", "find", "take_while", "sort_by", "min_by", "max_by"];
        if !BLOCK_METHODS.contains(&name.as_str()) || args.len() != 1 || args[0].name.is_some() {
            return None;
        }
        let fname = match &args[0].value.kind {
            ExprKind::Ident(n) if self.lookup(n).is_none() && self.fns.contains_key(n) => n.clone(),
            _ => return None,
        };
        let (l, c) = (args[0].value.line, args[0].value.col);
        let call = Expr::new(ExprKind::Call { name: fname, args: vec![Arg { name: None, value: Expr::new(ExprKind::Ident("_".into()), l, c) }] }, l, c);
        let lam = Expr::new(ExprKind::Lambda { params: vec!["_".into()], body: Block { stmts: vec![Stmt::Expr(call)] } }, l, c);
        Some(Expr::new(ExprKind::Method { recv: recv.clone(), name: name.clone(), args: vec![Arg { name: None, value: lam }] }, e.line, e.col))
    }

    /// `x.name?` where no method `name?` exists but `name` does means
    /// `x.name` followed by `?` (propagation). Returns the rewritten
    /// expression in that case.
    fn split_trailing_try(&mut self, e: &Expr) -> Option<Expr> {
        let (recv, name, args) = match &e.kind {
            ExprKind::Method { recv, name, args } => (recv, name, args),
            _ => return None,
        };
        if !name.ends_with('?') || name.len() < 2 {
            return None;
        }
        let base = &name[..name.len() - 1];
        // built-in namespaces: File.read?
        if let ExprKind::Ident(tn) = &recv.kind {
            if self.lookup(tn).is_none() && builtin_namespace_type(tn, name).is_none() && builtin_namespace_type(tn, base).is_some() {
                let inner = Expr::new(ExprKind::Method { recv: recv.clone(), name: base.to_string(), args: args.clone() }, e.line, e.col);
                return Some(Expr::new(ExprKind::Try(Box::new(inner)), e.line, e.col));
            }
        }
        let rt = self.ty_of(recv);
        let exists = |g: &Self, n: &str| -> bool {
            match &rt {
                Type::Named(tn) => {
                    g.methods_of(tn).map(|m| m.contains_key(n)).unwrap_or(false)
                        || g.structs.get(tn).map(|s| s.fields.iter().any(|(f, _)| f == n)).unwrap_or(false)
                }
                Type::Unknown => false,
                other => is_builtin_name(n) || builtin_method_type(other, n) != Type::Unknown,
            }
        };
        if exists(self, name) || !exists(self, base) {
            return None;
        }
        let inner = Expr::new(ExprKind::Method { recv: recv.clone(), name: base.to_string(), args: args.clone() }, e.line, e.col);
        Some(Expr::new(ExprKind::Try(Box::new(inner)), e.line, e.col))
    }

    // ----- type inference ---------------------------------------------------

    fn ty_of(&mut self, e: &Expr) -> Type {
        if let ExprKind::Method { .. } = &e.kind {
            if let Some(ne) = self.split_trailing_try(e) {
                return self.ty_of(&ne);
            }
            if let Some(ne) = self.fn_ref_as_block(e) {
                return self.ty_of(&ne);
            }
            if let Ok(Some(ne)) = self.module_ref(e) {
                return self.ty_of(&ne);
            }
        }
        match &e.kind {
            ExprKind::Int(_) => Type::Int,
            ExprKind::Float(_) => Type::Float,
            ExprKind::Bool(_) => Type::Bool,
            ExprKind::Str(_) => Type::Str,
            ExprKind::Ident(n) => {
                if let Some(b) = self.lookup(n) {
                    b.ty.clone()
                } else if let Some(t) = self.field_type(n) {
                    t
                } else if let Some(m) = self.bare_method(n) {
                    m.ret
                } else if let Some(st) = self.current_self_ty.clone() {
                    if builtin_method_type(&st, n) != Type::Unknown || is_builtin_name(n) {
                        builtin_method_type(&st, n)
                    } else {
                        Type::Unknown
                    }
                } else if let Ok(Some(en)) = self.resolve_variant(n, e.line, e.col) {
                    Type::Named(en)
                } else {
                    Type::Unknown
                }
            }
            ExprKind::SelfRef => self.current_self_ty.clone().or_else(|| self.current_type.clone().map(Type::Named)).unwrap_or(Type::Unknown),
            ExprKind::List(items) => {
                let mut t = Type::Unknown;
                for i in items {
                    let it = self.ty_of(i).materialized();
                    if it != Type::Unknown {
                        t = it;
                        break;
                    }
                }
                Type::List(Box::new(t))
            }
            ExprKind::Range { .. } => Type::Iter(Box::new(Type::Int), false),
            ExprKind::Unary { op, expr } => match *op {
                "not" => Type::Bool,
                _ => self.ty_of(expr),
            },
            ExprKind::Binary { op, lhs, rhs } => {
                if let Type::Named(tn) = self.ty_of(lhs).materialized() {
                    if let Some(sg) = self.methods_of(&tn).and_then(|m| m.get(*op).cloned()) {
                        return sg.ret;
                    }
                }
                match *op {
                    "==" | "!=" | "<" | "<=" | ">" | ">=" | "and" | "or" => Type::Bool,
                    _ => {
                        let l = self.ty_of(lhs);
                        if l == Type::Unknown { self.ty_of(rhs) } else { l }
                    }
                }
            }
            ExprKind::Call { name: raw_name, .. } => {
                let cname = self.canon(raw_name);
                let name = &cname;
                if let Some(s) = self.fns.get(name) {
                    s.ret.clone()
                } else if let Some(m) = self.current_type.as_ref().and_then(|t| self.methods_of(t)).and_then(|m| m.get(name).cloned()) {
                    m.ret
                } else if self.structs.contains_key(name) {
                    Type::Named(name.clone())
                } else if let Ok(Some(en)) = self.resolve_variant(name, e.line, e.col) {
                    Type::Named(en)
                } else {
                    Type::Unknown
                }
            }
            ExprKind::Method { recv, name, args } => {
                // Enum variant constructor: Shape.Circle(...)
                if let ExprKind::Ident(tn) = &recv.kind {
                    let ctn = self.canon(tn);
                    if self.lookup(tn).is_none() && self.enums.contains_key(&ctn) {
                        return Type::Named(ctn);
                    }
                    if self.lookup(tn).is_none() {
                        if let Some(t) = builtin_namespace_type(tn, name) {
                            return t;
                        }
                    }
                }
                let rt = self.ty_of(recv);
                if let Some(Arg { value: lam, .. }) = args.last() {
                    if let ExprKind::Lambda { params, body } = &lam.kind {
                        let init = if args.len() > 1 { Some(self.ty_of(&args[0].value)) } else { None };
                        return self.block_method_type(&rt, name, params, body, init);
                    }
                }
                if let Type::Named(sn) = &rt {
                    if let Some(info) = self.structs.get(sn) {
                        if let Some((_, ft)) = info.fields.iter().find(|(n, _)| n == name) {
                            return ft.clone();
                        }
                    }
                    if let Some(m) = self.methods_of(sn).and_then(|m| m.get(name).cloned()) {
                        return m.ret;
                    }
                    if name == "to_s" || name == "to_str" {
                        return Type::Str;
                    }
                }
                if let Some(m) = self.ext_methods.get(&self.type_key(&rt)).and_then(|m| m.get(name)) {
                    return m.ret.clone();
                }
                builtin_method_type(&rt, name)
            }
            ExprKind::If { branches, else_block } => {
                if else_block.is_none() {
                    return Type::Unit;
                }
                for (_, b) in branches {
                    let t = self.tail_type(b);
                    if t != Type::Unknown {
                        return t;
                    }
                }
                if let Some(eb) = else_block {
                    return self.tail_type(eb);
                }
                Type::Unknown
            }
            ExprKind::Puts(_) => Type::Unit,
            ExprKind::Placeholder | ExprKind::Lambda { .. } => Type::Unknown,
            ExprKind::Match { scrutinee, arms } => self.match_type(scrutinee, arms),
            ExprKind::Tuple(items) => Type::Tuple(items.iter().map(|i| self.ty_of(i).materialized()).collect()),
            ExprKind::TupleIndex { recv, index } => match self.ty_of(recv) {
                Type::Tuple(ts) => ts.get(*index).cloned().unwrap_or(Type::Unknown),
                _ => Type::Unknown,
            },
            ExprKind::Some(x) => Type::Option(Box::new(self.ty_of(x).materialized())),
            ExprKind::None => Type::Option(Box::new(Type::Unknown)),
            ExprKind::Try(x) | ExprKind::Unwrap(x) => match self.ty_of(x) {
                Type::Option(inner) => *inner,
                Type::Result(t, _) => *t,
                _ => Type::Unknown,
            },
            ExprKind::Rust(_) => Type::Unknown,
            ExprKind::Index { recv, .. } => match self.ty_of(recv).materialized() {
                Type::List(e) => Type::Option(e),
                Type::Map(_, v) => Type::Option(v),
                Type::Str => Type::Unknown,
                _ => Type::Unknown,
            },
            ExprKind::MapLit(pairs) => {
                let mut k = Type::Unknown;
                let mut v = Type::Unknown;
                for (pk, pv) in pairs {
                    if k == Type::Unknown { k = self.ty_of(pk).materialized(); }
                    if v == Type::Unknown { v = self.ty_of(pv).materialized(); }
                }
                Type::Map(Box::new(k), Box::new(v))
            }
            ExprKind::Ok(x) => {
                let t = self.ty_of(x).materialized();
                match &self.current_ret {
                    Type::Result(_, e) => Type::Result(Box::new(t), e.clone()),
                    _ => Type::Result(Box::new(t), Box::new(Type::Named("Error".into()))),
                }
            }
        }
    }

    /// Element type and by-reference flag of a list, range or lazy chain.
    fn elem_of(&self, t: &Type) -> Option<(Type, bool)> {
        match t {
            Type::List(e) => Some(((**e).clone(), !e.is_copy())),
            Type::Iter(e, by_ref) => Some(((**e).clone(), *by_ref)),
            Type::Map(k, v) => Some((Type::Tuple(vec![(**k).clone(), (**v).clone()]), true)),
            _ => None,
        }
    }

    /// Declares block parameters for an item of type `elem`. Two parameters
    /// destructure a pair; with `acc` set, the first parameter is the fold
    /// accumulator.
    fn declare_block_params(&mut self, params: &[String], elem: &Type, by_ref: bool, acc: Option<&Type>, line: usize) {
        match (acc, params.len(), elem) {
            (Some(at), 2, _) => {
                self.declare(&params[0], false, false, at.clone(), line);
                self.declare(&params[1], false, by_ref && !elem.is_copy(), elem.clone(), line);
            }
            (None, 2, Type::Tuple(ts)) if ts.len() == 2 => {
                self.declare(&params[0], false, by_ref && !ts[0].is_copy(), ts[0].clone(), line);
                self.declare(&params[1], false, by_ref && !ts[1].is_copy(), ts[1].clone(), line);
            }
            _ => {
                if let Some(p) = params.first() {
                    self.declare(p, false, by_ref && !elem.is_copy(), elem.clone(), line);
                }
            }
        }
    }

    fn lambda_body_type(&mut self, params: &[String], elem: &Type, by_ref: bool, acc: Option<&Type>, body: &Block) -> Type {
        self.push_scope();
        self.declare_block_params(params, elem, by_ref, acc, 0);
        let t = self.tail_type(body);
        self.pop_scope();
        t
    }

    fn block_method_type(&mut self, recv: &Type, name: &str, params: &[String], body: &Block, init: Option<Type>) -> Type {
        let (elem, by_ref) = match self.elem_of(recv) {
            Some(x) => x,
            None => return Type::Unknown,
        };
        match name {
            "map" => {
                let bt = self.lambda_body_type(params, &elem, by_ref, None, body).materialized();
                Type::Iter(Box::new(bt), false)
            }
            "filter" | "reject" | "take_while" => Type::Iter(Box::new(elem), by_ref),
            "each" => Type::Unit,
            "sum" => self.lambda_body_type(params, &elem, by_ref, None, body),
            "count" => Type::Int,
            "any?" | "all?" => Type::Bool,
            "sort_by" => Type::List(Box::new(elem)),
            "find" => Type::Option(Box::new(elem)),
            "min_by" | "max_by" => Type::Option(Box::new(elem)),
            "fold" => init.map(|t| t.materialized()).unwrap_or(Type::Unknown),
            _ => Type::Unknown,
        }
    }

    // ----- items ------------------------------------------------------------

    fn interface_def(&mut self, i: &InterfaceDef) -> Result<()> {
        self.line(&format!("pub trait {}: LumeShow {{", i.name));
        self.indent += 1;
        let info = self.interfaces[&i.name].clone();
        for m in &i.required {
            let sg = &info.methods[&m.name];
            self.line(&format!("fn {}({}) -> {};", rust_name(&m.name), self.sig_params_rust(sg, true), self.rt(&sg.ret)));
        }
        self.in_trait_impl = true;
        for m in &i.defaults {
            self.fn_def(m, Some(&i.name))?;
        }
        self.in_trait_impl = false;
        self.indent -= 1;
        self.line("}");
        // Boxed values forward to the value inside.
        self.line(&format!("impl<T: {} + ?Sized> {} for Box<T> {{", i.name, i.name));
        self.indent += 1;
        for (name, sg) in &info.methods {
            let args: Vec<String> = sg.params.iter().map(|(n, _)| rust_name(n)).collect();
            self.line(&format!(
                "fn {}({}) -> {} {{ (**self).{}({}) }}",
                rust_name(name),
                self.sig_params_rust(sg, true),
                self.rt(&sg.ret),
                rust_name(name),
                args.join(", ")
            ));
        }
        self.indent -= 1;
        self.line("}");
        Ok(())
    }

    /// `&self, a: &A, b: i64` for a method signature.
    fn sig_params_rust(&self, sg: &Sig, with_self: bool) -> String {
        let mut parts: Vec<String> = Vec::new();
        if with_self {
            parts.push("&self".into());
        }
        for (i, (n, t)) in sg.params.iter().enumerate() {
            let rt = self.rt(t);
            let rt = if sg.var_params.get(i).copied().unwrap_or(false) { format!("&mut {}", rt) } else if t.is_copy() { rt } else if *t == Type::Str { "&str".into() } else { format!("&{}", rt) };
            parts.push(format!("{}: {}", rust_name(n), rt));
        }
        parts.join(", ")
    }

    /// For every (type, interface) pair this module is responsible for, emit
    /// the trait impl: inherent methods delegate, `extend` methods bring
    /// their bodies, defaults are left to the trait.
    fn emit_conformances(&mut self, program: &[Item]) -> Result<()> {
        // extend blocks by (type key, iface)
        let mut ext_bodies: HashMap<(String, String), Vec<FnDef>> = HashMap::new();
        for item in program {
            if let Item::Extend(x) = item {
                let key = self.type_key(&self.ct(&x.target));
                ext_bodies.entry((key, self.canon(&x.iface))).or_default().extend(x.methods.iter().cloned());
            }
        }
        // candidate types: local structs/enums, extend targets, imported types (when the interface is local)
        let mut types: Vec<(String, Type)> = Vec::new();
        for n in &self.local_types {
            types.push((n.clone(), Type::Named(n.clone())));
        }
        for (k, t) in &self.ext_targets {
            types.push((k.clone(), t.clone()));
        }
        for n in self.structs.keys().chain(self.enums.keys()) {
            if n != "Error" && !self.local_types.contains(n) && n.contains('.') {
                types.push((n.clone(), Type::Named(n.clone())));
            }
        }
        let ifaces: Vec<(String, IfaceInfo)> = self.interfaces.iter().map(|(k, v)| (k.clone(), v.clone())).collect();
        let mut done: HashSet<(String, String)> = HashSet::new();
        for (tkey, t) in &types {
            for (iname, info) in &ifaces {
                let type_local = self.local_types.contains(tkey) || self.ext_targets.contains_key(tkey);
                let owns = info.local || (type_local && !tkey.contains('.'));
                let has_extend = ext_bodies.contains_key(&(tkey.clone(), iname.clone()));
                if !(owns || has_extend) {
                    continue;
                }
                if !done.insert((tkey.clone(), iname.clone())) {
                    continue;
                }
                if !matches!(self.conformance(t, iname), Conformance::Yes) {
                    if let Some(x) = program.iter().find_map(|it| match it {
                        Item::Extend(x) if self.type_key(&self.ct(&x.target)) == *tkey && self.canon(&x.iface) == *iname => Some(x),
                        _ => None,
                    }) {
                        // an extend that still leaves methods missing is an error here
                        if let Conformance::Missing(m) = self.conformance(t, iname) {
                            let info = &self.interfaces[iname];
                            let needs: Vec<String> = m.iter().filter_map(|n| info.methods.get(n).map(|s| format!("def {} {}", n, self.describe_sig(s)))).collect();
                            return Err(LumeError::new(x.line, x.col, format!("`extend {} with {}` is missing `{}`", type_name(t), x.iface, m.join("`, `")))
                                .with_help(format!("`{}` needs: {}. Add {} inside this `extend` block", x.iface, needs.join("; "), if m.len() == 1 { "it" } else { "them" })));
                        }
                        self.require_conforms(t, iname, x.line, x.col)?;
                    }
                    continue;
                }
                let bodies = ext_bodies.get(&(tkey.clone(), iname.clone())).cloned().unwrap_or_default();
                let targets: Vec<String> = match t {
                    Type::Str => vec!["str".into(), "String".into()],
                    other => vec![self.rt(other)],
                };
                for target in targets {
                    self.line(&format!("impl {} for {} {{", self.path_of(iname), target));
                    self.indent += 1;
                    let inherent = self.methods_of(tkey).unwrap_or_default();
                    for (mname, sg) in &info.methods {
                        if let Some(body) = bodies.iter().find(|b| b.name == *mname) {
                            if target == "String" {
                                // delegate to the str impl
                                let args: Vec<String> = sg.params.iter().map(|(n, _)| rust_name(n)).collect();
                                self.line(&format!("fn {}({}) -> {} {{ self.as_str().{}({}) }}", rust_name(mname), self.sig_params_rust(sg, true), self.rt(&sg.ret), rust_name(mname), args.join(", ")));
                            } else {
                                self.in_trait_impl = true;
                                self.fn_def(body, Some(tkey))?;
                                self.in_trait_impl = false;
                            }
                        } else if inherent.contains_key(mname) && !self.ext_methods.get(tkey).map(|m| m.contains_key(mname)).unwrap_or(false) {
                            let args: Vec<String> = sg.params.iter().map(|(n, _)| rust_name(n)).collect();
                            self.line(&format!(
                                "fn {}({}) -> {} {{ {}::{}(self{}{}) }}",
                                rust_name(mname),
                                self.sig_params_rust(sg, true),
                                self.rt(&sg.ret),
                                self.rt(t),
                                rust_name(mname),
                                if args.is_empty() { "" } else { ", " },
                                args.join(", ")
                            ));
                        }
                    }
                    self.indent -= 1;
                    self.line("}");
                }
            }
        }
        Ok(())
    }

    /// `#[derive(...)]` for a user type: `PartialEq` is derived unless the
    /// type defines `==` itself.
    fn derive_line(&mut self, methods: &[FnDef]) {
        if methods.iter().any(|m| m.name == "==") {
            self.line("#[derive(Debug, Clone)]");
        } else {
            self.line("#[derive(Debug, Clone, PartialEq)]");
        }
    }

    /// `def ==` becomes `PartialEq`, `def <` becomes `PartialOrd`, so the
    /// type works with `contains?`, `sort`, `max`, `min` and `==` on lists.
    fn op_impls(&mut self, name: &str, methods: &[FnDef]) {
        if methods.iter().any(|m| m.name == "==") {
            self.line(&format!("impl PartialEq for {} {{ fn eq(&self, o: &Self) -> bool {{ self.op_eq(o) }} }}", name));
        }
        if methods.iter().any(|m| m.name == "<") {
            self.line(&format!(
                "impl PartialOrd for {} {{ fn partial_cmp(&self, o: &Self) -> Option<std::cmp::Ordering> {{ Some(if self.op_lt(o) {{ std::cmp::Ordering::Less }} else if o.op_lt(self) {{ std::cmp::Ordering::Greater }} else {{ std::cmp::Ordering::Equal }}) }} }}",
                name
            ));
        }
    }

    fn struct_def(&mut self, s: &StructDef) -> Result<()> {
        self.derive_line(&s.methods);
        self.line(&format!("pub struct {} {{", s.name));
        self.indent += 1;
        for f in &s.fields {
            let ft = self.rt(&f.ty);
            self.line(&format!("pub {}: {},", rust_name(&f.name), ft));
        }
        self.indent -= 1;
        self.line("}");
        self.line(&format!("impl LumeShow for {} {{", s.name));
        self.indent += 1;
        let fmt: Vec<String> = s.fields.iter().map(|f| format!("{}: {{}}", f.name)).collect();
        let args: Vec<String> = s.fields.iter().map(|f| format!("self.{}.lume_str()", rust_name(&f.name))).collect();
        self.line(&format!("fn lume_str(&self) -> String {{ format!(\"{}({})\", {}) }}", s.name, fmt.join(", "), args.join(", ")));
        self.indent -= 1;
        self.line("}");
        self.op_impls(&s.name, &s.methods);
        if !s.methods.is_empty() {
            self.line(&format!("impl {} {{", s.name));
            self.indent += 1;
            for m in &s.methods {
                self.fn_def(m, Some(&s.name))?;
            }
            self.indent -= 1;
            self.line("}");
        }
        Ok(())
    }

    fn enum_def(&mut self, e: &EnumDef) -> Result<()> {
        self.derive_line(&e.methods);
        self.line(&format!("pub enum {} {{", e.name));
        self.indent += 1;
        for v in &e.variants {
            if v.fields.is_empty() {
                self.line(&format!("{},", v.name));
            } else {
                let fs: Vec<String> = v.fields.iter().map(|f| format!("{}: {}", rust_name(&f.name), self.rt(&f.ty))).collect();
                self.line(&format!("{} {{ {} }},", v.name, fs.join(", ")));
            }
        }
        self.indent -= 1;
        self.line("}");
        self.line(&format!("impl LumeShow for {} {{", e.name));
        self.indent += 1;
        self.line("fn lume_str(&self) -> String {");
        self.indent += 1;
        self.line("match self {");
        self.indent += 1;
        for v in &e.variants {
            if v.fields.is_empty() {
                self.line(&format!("{}::{} => String::from(\"{}\"),", e.name, v.name, v.name));
            } else {
                let names: Vec<String> = v.fields.iter().map(|f| rust_name(&f.name)).collect();
                let fmt: Vec<String> = v.fields.iter().map(|f| format!("{}: {{}}", f.name)).collect();
                let args: Vec<String> = names.iter().map(|n| format!("{}.lume_str()", n)).collect();
                self.line(&format!(
                    "{}::{} {{ {} }} => format!(\"{}({})\", {}),",
                    e.name,
                    v.name,
                    names.join(", "),
                    v.name,
                    fmt.join(", "),
                    args.join(", ")
                ));
            }
        }
        self.indent -= 1;
        self.line("}");
        self.indent -= 1;
        self.line("}");
        self.indent -= 1;
        self.line("}");
        self.op_impls(&e.name, &e.methods);
        if !e.methods.is_empty() {
            self.line(&format!("impl {} {{", e.name));
            self.indent += 1;
            for m in &e.methods {
                self.fn_def(m, Some(&e.name))?;
            }
            self.indent -= 1;
            self.line("}");
        }
        Ok(())
    }

    fn fn_def(&mut self, f: &FnDef, owner: Option<&String>) -> Result<()> {
        let sig = match owner {
            None => self.fns[&f.name].clone(),
            Some(s) => self.methods_of(s).unwrap()[&f.name].clone(),
        };
        let is_main = f.name == "main" && owner.is_none() && self.is_entry;
        let main_result = is_main && matches!(sig.ret, Type::Result(..));
        // `-> () or E`: the body ends with statements, and finishes with Ok(()).
        let unit_result = matches!(&sig.ret, Type::Result(t, _) if **t == Type::Unit);
        if is_main {
            let ok = f.params.is_empty()
                && match &f.ret {
                    None => true,
                    Some(Type::Unit) => true,
                    Some(Type::Result(t, _)) => **t == Type::Unit,
                    _ => false,
                };
            if !ok {
                return Err(LumeError::new(f.line, f.col, "`main` takes no parameters and returns nothing, or `() or Error`")
                    .with_help("write `def main:` or `def main -> () or Error:`"));
            }
        }
        let fn_name = if main_result { "lume_main".to_string() } else { rust_name(&f.name) };
        let mut parts: Vec<String> = Vec::new();
        let mut generics: Vec<String> = Vec::new();
        if owner.is_some() {
            parts.push(match f.self_kind {
                SelfKind::Read => "&self".into(),
                SelfKind::Mutate => "&mut self".into(),
            });
        }
        let mut names = HashSet::new();
        for p in &f.params {
            if !names.insert(p.name.clone()) {
                return Err(LumeError::new(p.line, p.col, format!("parameter `{}` is listed twice", p.name)));
            }
            let pty = self.ct(&p.ty);
            if self.is_interface(&pty) && !self.in_trait_impl {
                // `s: Shape` — one generic parameter per interface-typed parameter (static dispatch)
                let g = format!("T{}", generics.len());
                let iface = match &pty { Type::Named(n) => self.path_of(n), _ => unreachable!() };
                generics.push(format!("{}: {}", g, iface));
                parts.push(format!("{}: &{}", rust_name(&p.name), g));
                continue;
            }
            let rt = self.rt(&pty);
            let rt = if p.mutable { format!("&mut {}", rt) } else if pty.is_copy() { rt } else if pty == Type::Str { "&str".to_string() } else { format!("&{}", rt) };
            parts.push(format!("{}: {}", rust_name(&p.name), rt));
        }
        let vis = if self.in_trait_impl { "" } else { "pub " };
        let gen = if generics.is_empty() { String::new() } else { format!("<{}>", generics.join(", ")) };
        let mut header = format!("{}fn {}{}({})", vis, fn_name, gen, parts.join(", "));
        if sig.ret != Type::Unit {
            header.push_str(&format!(" -> {}", self.rt(&sig.ret)));
        }
        header.push_str(" {");
        self.line(&header);
        self.indent += 1;
        self.push_scope();
        for p in &f.params {
            let pty = self.ct(&p.ty);
            self.declare(&p.name, p.mutable, !pty.is_copy(), pty, p.line);
        }
        self.current_ret = sig.ret.clone();
        self.current_type = owner.cloned();
        self.current_self_ty = owner.and_then(|o| self.ext_targets.get(o).cloned());
        self.current_self = f.self_kind;
        self.current_fn = f.name.clone();
        let want_value = sig.ret != Type::Unit;
        self.tail_of_fn = true;
        if unit_result {
            self.block_body(&f.body, false)?;
            self.line("Ok(())");
        } else {
            self.block_body(&f.body, want_value)?;
        }
        self.tail_of_fn = false;
        self.pop_scope();
        self.current_type = None;
        self.current_self_ty = None;
        self.indent -= 1;
        self.line("}");
        if main_result {
            self.line("fn main() {");
            self.line("    if let Err(e) = lume_main() { eprintln!(\"error: {}\", e.message); std::process::exit(1); }");
            self.line("}");
        }
        Ok(())
    }

    /// In a function returning `T or E`, a result value of type `T` is
    /// wrapped in `Ok`, and one of type `E` in `Err`.
    fn coerce_result(&mut self, text: String, e: &Expr) -> Result<String> {
        if !self.tail_of_fn || self.in_block {
            return Ok(text);
        }
        if matches!(e.kind, ExprKind::Rust(_)) {
            return Ok(text);
        }
        let ret = self.current_ret.clone();
        if self.is_interface(&ret) {
            let et = self.ty_of(e).materialized();
            return self.coerce(text, &et, &ret, e.line, e.col);
        }
        let (ok_t, err_t) = match &ret {
            Type::Result(t, e) => ((**t).clone(), (**e).clone()),
            _ => return Ok(text),
        };
        let et = self.ty_of(e).materialized();
        if matches!(et, Type::Result(..)) {
            return Ok(text);
        }
        if et == err_t && et != ok_t {
            return Ok(format!("Err({})", text));
        }
        Ok(format!("Ok({})", text))
    }

    fn block_body(&mut self, b: &Block, want_value: bool) -> Result<()> {
        let n = b.stmts.len();
        if want_value && n == 0 {
            return Err(LumeError::new(1, 1, "a block that produces a value cannot be empty"));
        }
        for (i, s) in b.stmts.iter().enumerate() {
            let last = i + 1 == n;
            let saved = std::mem::replace(&mut self.rest_of_block, b.stmts[i + 1..].to_vec());
            let r = self.stmt(s, want_value && last);
            self.rest_of_block = saved;
            r?;
        }
        Ok(())
    }

    /// True when a name bound in the current block is not used by any later
    /// statement of that block, so its value may be moved instead of cloned.
    fn dead_after_this(&self, name: &str) -> bool {
        if !self.scopes.last().map(|sc| sc.contains_key(name)).unwrap_or(false) {
            return false;
        }
        !self.rest_of_block.iter().any(|st| stmt_mentions(st, name))
    }

    fn nested_block(&mut self, b: &Block, want_value: bool) -> Result<()> {
        self.indent += 1;
        self.push_scope();
        self.at_tail = false;
        self.block_body(b, want_value)?;
        self.pop_scope();
        self.indent -= 1;
        Ok(())
    }

    // ----- statements -------------------------------------------------------

    fn stmt(&mut self, s: &Stmt, is_tail: bool) -> Result<()> {
        match s {
            Stmt::Var { name, ty, value, line, col } => {
                if let Some(t) = ty {
                    self.check_type(t, *line, *col)?;
                }
                let inferred = self.ty_of(value).materialized();
                let vt = ty.as_ref().map(|t| self.ct(t)).unwrap_or_else(|| inferred.clone());
                if ty.is_none() && !type_is_known(&inferred) {
                    return Err(LumeError::new(*line, *col, format!("cannot tell the type of `{}` from `{}` alone", name, describe_value(value)))
                        .with_help(format!("add the type: `var {}: {} = ...`", name, suggest_type(&inferred))));
                }
                let v = self.expr_owned_as(value, &vt)?;
                if ty.is_some() {
                    self.note_dynamic(name, &vt, *line, *col);
                }
                if self.scopes.last().unwrap().contains_key(name) {
                    return Err(LumeError::new(*line, *col, format!("`{}` is already declared in this block", name))
                        .with_help(format!("to change it, write `{} = ...`", name)));
                }
                self.declare(name, true, false, vt.clone(), *line);
                let ann = if ty.is_some() { format!(": {}", self.rt(&vt)) } else { String::new() };
                self.line(&format!("let mut {}{} = {};", rust_name(name), ann, v));
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::Bind { name, ty, value, line, col } => {
                if let Some(t) = ty {
                    self.check_type(t, *line, *col)?;
                    if self.lookup(name).is_some() {
                        return Err(LumeError::new(*line, *col, format!("`{}` already exists; a type goes only on a new binding", name)));
                    }
                }
                let inferred = self.ty_of(value).materialized();
                let vt = ty.as_ref().map(|t| self.ct(t)).unwrap_or_else(|| inferred.clone());
                if ty.is_none() && self.lookup(name).is_none() && self.field_type(name).is_none() && !type_is_known(&inferred) {
                    return Err(LumeError::new(*line, *col, format!("cannot tell the type of `{}` from `{}` alone", name, describe_value(value)))
                        .with_help(format!("add the type: `{}: {} = ...`", name, suggest_type(&inferred))));
                }
                let ann = if ty.is_some() { format!(": {}", self.rt(&vt)) } else { String::new() };
                match self.lookup(name).cloned() {
                    Some(b) if b.mutable => {
                        let v = self.expr_owned(value)?;
                        self.line(&format!("{} = {};", rust_name(name), v));
                    }
                    Some(b) => {
                        if leftmost_ident(value) == Some(name.as_str()) {
                            let v = self.expr_owned(value)?;
                            self.declare(name, false, false, vt, *line);
                            self.line(&format!("let {} = {};", rust_name(name), v));
                        } else {
                            return Err(LumeError::new(*line, *col, format!("`{}` is immutable and cannot be reassigned", name))
                                .with_help(format!(
                                    "declare it with `var {} = ...` on line {} if it needs to change; `{} = {}.something` is allowed as a transform of the same value",
                                    name, b.line, name, name
                                )));
                        }
                    }
                    None => {
                        if self.field_type(name).is_some() {
                            self.require_var_self(name, *line, *col)?;
                            let v = self.expr_owned(value)?;
                            self.line(&format!("self.{} = {};", rust_name(name), v));
                        } else {
                            let v = self.expr_owned_as(value, &vt)?;
                            if ty.is_some() {
                                self.note_dynamic(name, &vt, *line, *col);
                            }
                            self.declare(name, false, false, vt, *line);
                            self.line(&format!("let {}{} = {};", rust_name(name), ann, v));
                        }
                    }
                }
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::OpAssign { name, op, value, line, col } => {
                let v = self.expr(value)?;
                match self.lookup(name).cloned() {
                    Some(b) if b.mutable => {
                        self.line(&format!("{} {} {};", rust_name(name), op, v));
                    }
                    Some(b) => {
                        return Err(LumeError::new(*line, *col, format!("`{}` is immutable and cannot be changed with `{}`", name, op))
                            .with_help(format!("declare it with `var {} = ...` on line {}", name, b.line)));
                    }
                    None => {
                        if self.field_type(name).is_some() {
                            self.require_var_self(name, *line, *col)?;
                            self.line(&format!("self.{} {} {};", rust_name(name), op, v));
                        } else {
                            return Err(self.unknown_name(name, *line, *col));
                        }
                    }
                }
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::IndexAssign { recv, index, op, value, line, col } => {
                let rt = self.ty_of(recv).materialized();
                match &rt {
                    Type::List(_) | Type::Map(..) => {}
                    Type::Unknown => {}
                    other => return Err(LumeError::new(*line, *col, format!("`{}` values cannot be indexed", type_name(other)))),
                }
                let target = Expr::new(ExprKind::Index { recv: Box::new(recv.clone()), index: Box::new(index.clone()) }, *line, *col);
                if let (Type::Map(k_ty, _), None) = (&rt, op) {
                    // `m[k] = m[k].or(d) op x`: one lookup through the entry API.
                    if let ExprKind::Binary { op: bop, lhs, rhs } = &value.kind {
                        if matches!(*bop, "+" | "-" | "*" | "/" | "%") {
                            if let ExprKind::Method { recv: orecv, name: oname, args: oargs } = &lhs.kind {
                                if oname == "or" && oargs.len() == 1 {
                                    if let ExprKind::Index { recv: irecv, index: iidx } = &orecv.kind {
                                        if same_expr(irecv, recv) && same_expr(iidx, index) {
                                            let place = self.mutable_place(recv, "this map", *line, *col)?;
                                            let k = self.expr_val(index)?;
                                            let movable = matches!(&index.kind, ExprKind::Ident(n) if self.dead_after_this(n) && !self.is_borrowed_ident(index));
                                            let k = if k_ty.is_copy() || !matches!(index.kind, ExprKind::Ident(_)) || movable { k } else if **k_ty == Type::Str { format!("({}).to_string()", k) } else { format!("({}).clone()", k) };
                                            let k = if k_ty.is_copy() && self.is_borrowed_ident(index) { format!("*{}", k) } else { k };
                                            let d = self.expr_owned(&oargs[0].value)?;
                                            let x = self.expr_val(rhs)?;
                                            let tmp = self.fresh("e");
                                            self.line(&format!("{{ let {} = {}.entry_or_insert({}, {}); *{} = *{} {} {}; }}", tmp, place, k, d, tmp, tmp, bop, x));
                                            if is_tail {
                                                return self.tail_unit(*line, *col);
                                            }
                                            return Ok(());
                                        }
                                    }
                                }
                            }
                        }
                    }
                    // insert or replace: the value is computed first (it may read the
                    // same key), and a non-Copy key is cloned so it stays usable after.
                    let place = self.mutable_place(recv, "this map", *line, *col)?;
                    let k = self.expr_val(index)?;
                    let movable = matches!(&index.kind, ExprKind::Ident(n) if self.dead_after_this(n) && !self.is_borrowed_ident(index));
                    let k = if k_ty.is_copy() || matches!(index.kind, ExprKind::Str(_) | ExprKind::Int(_)) || movable { k } else if **k_ty == Type::Str { format!("({}).to_string()", k) } else { format!("({}).clone()", k) };
                    let v = self.expr_owned(value)?;
                    let tmp = self.fresh("v");
                    self.line(&format!("{{ let {} = {}; {}.insert({}, {}); }}", tmp, v, place, k, tmp));
                } else {
                    let place = self.mutable_place(&target, "this position", *line, *col)?;
                    let v = if op.is_some() { self.expr(value)? } else { self.expr_owned(value)? };
                    self.line(&format!("{} {} {};", place, op.unwrap_or("="), v));
                }
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::FieldAssign { recv, field, op, value, line, col } if matches!(recv.kind, ExprKind::Index { .. }) => {
                let rt = self.ty_of(recv).materialized();
                let inner = match &rt {
                    Type::Option(i) => (**i).clone(),
                    other => other.clone(),
                };
                let sname = match &inner {
                    Type::Named(s) if self.structs.contains_key(s) => s.clone(),
                    other => return Err(LumeError::new(*line, *col, format!("`{}` values have no fields to assign", type_name(other)))),
                };
                if !self.structs[&sname].fields.iter().any(|(n, _)| n == field) {
                    return Err(self.no_such_member(&sname, field, *line, *col));
                }
                let place = self.mutable_place(recv, &format!("the field `{}`", field), *line, *col)?;
                let v = if op.is_some() { self.expr(value)? } else { self.expr_owned(value)? };
                self.line(&format!("{}.{} {} {};", place, rust_name(field), op.unwrap_or("="), v));
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::FieldAssign { recv, field, op, value, line, col } => {
                let rt = self.ty_of(recv);
                let sname = match &rt {
                    Type::Named(s) if self.structs.contains_key(s) => s.clone(),
                    Type::Named(s) => return Err(LumeError::new(*line, *col, format!("`{}` is an enum; its values have no assignable fields", s))),
                    Type::Unknown => return Err(LumeError::new(*line, *col, format!("cannot tell what `.{}` belongs to here", field))),
                    other => return Err(LumeError::new(*line, *col, format!("`{}` values have no fields to assign", type_name(other)))),
                };
                let info = self.structs[&sname].clone();
                if !info.fields.iter().any(|(n, _)| n == field) {
                    return Err(self.no_such_member(&sname, field, *line, *col));
                }
                match &recv.kind {
                    ExprKind::SelfRef => self.require_var_self(field, *line, *col)?,
                    ExprKind::Ident(n) => match self.lookup(n).cloned() {
                        Some(b) if b.mutable => {}
                        Some(b) => {
                            return Err(LumeError::new(*line, *col, format!("`{}` is immutable, so its field `{}` cannot be changed", n, field))
                                .with_help(format!("declare it with `var {} = ...` on line {}", n, b.line)));
                        }
                        None => {
                            if self.field_type(n).is_some() {
                                self.require_var_self(n, *line, *col)?;
                            } else {
                                return Err(self.unknown_name(n, *line, *col));
                            }
                        }
                    },
                    _ => {
                        return Err(LumeError::new(*line, *col, "only a named value's field can be assigned").with_help("bind the value to a `var` first"));
                    }
                }
                let r = self.expr(recv)?;
                let v = if op.is_some() { self.expr(value)? } else { self.expr_owned(value)? };
                self.line(&format!("{}.{} {} {};", r, rust_name(field), op.unwrap_or("="), v));
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::Expr(e) => {
                if is_tail {
                    if let ExprKind::If { else_block: None, .. } = &e.kind {
                        return Err(LumeError::new(e.line, e.col, "this `if` is the function's result but has no `else`")
                            .with_help("add an `else` branch, or add `return` before it if it is not the result"));
                    }
                    let v = match &e.kind {
                        ExprKind::If { .. } | ExprKind::Match { .. } => {
                            self.at_tail = self.tail_of_fn;
                            self.expr_owned(e)?
                        }
                        _ => {
                            let v = self.expr_owned(e)?;
                            self.coerce_result(v, e)?
                        }
                    };
                    self.line(&v);
                } else {
                    // In a `() or E` function a bare error value is an early return.
                    if !self.in_block {
                        if let Type::Result(t, err_t) = self.current_ret.clone() {
                            if *t == Type::Unit {
                                let et = self.ty_of(e).materialized();
                                if et == *err_t && et != Type::Unknown {
                                    let v = self.expr_owned(e)?;
                                    self.line(&format!("return Err({});", v));
                                    return Ok(());
                                }
                            }
                        }
                    }
                    let v = self.expr_stmt(e)?;
                    self.line(&format!("{};", v));
                }
            }
            Stmt::Return { value, line, col } => {
                if self.in_block {
                    self.warnings.push(
                        LumeError::new(*line, *col, "`return` inside a block ends this item's block, not the function")
                            .with_help("the last expression is already the block's value; to stop early, use `find`, `take_while` or a `for` loop"),
                    );
                    match value {
                        Some(e) => {
                            let v = self.expr_owned(e)?;
                            self.line(&format!("return {};", v));
                        }
                        None => self.line("return;"),
                    }
                    return Ok(());
                }
                match value {
                    Some(e) => {
                        if self.current_ret == Type::Unit {
                            return Err(LumeError::new(*line, *col, "this function returns nothing, but `return` has a value")
                                .with_help("add `-> Type` to the function signature"));
                        }
                        let v = self.expr_owned(e)?;
                        let saved = self.tail_of_fn;
                        self.tail_of_fn = true;
                        let v = self.coerce_result(v, e);
                        self.tail_of_fn = saved;
                        let v = v?;
                        self.line(&format!("return {};", v));
                    }
                    None => {
                        match &self.current_ret {
                            Type::Unit => self.line("return;"),
                            Type::Result(t, _) if **t == Type::Unit => self.line("return Ok(());"),
                            other => {
                                return Err(LumeError::new(*line, *col, format!("this function returns `{}`, so `return` needs a value", type_name(other))));
                            }
                        }
                    }
                }
            }
            Stmt::While { cond, body } => {
                let c = self.expr(cond)?;
                self.line(&format!("while {} {{", c));
                self.loop_depth += 1;
                self.nested_block(body, false)?;
                self.loop_depth -= 1;
                self.line("}");
            }
            Stmt::For { vars, iter, filter, body, line, col } => {
                let it_ty = self.ty_of(iter);
                // Iterating a collection reached through `self` while the body may
                // change `self` would be two borrows at once; iterate a copy instead.
                let self_rooted = self.current_self == SelfKind::Mutate
                    && matches!(self.place_root(iter).map(|r| &r.kind), Some(ExprKind::SelfRef))
                    || (self.current_self == SelfKind::Mutate
                        && matches!(self.place_root(iter).map(|r| &r.kind), Some(ExprKind::Ident(n)) if self.lookup(n).is_none() && self.field_type(n).is_some()));
                let (it, elem_ty, borrowed) = match &it_ty {
                    Type::Iter(elem, by_ref) => (self.expr(iter)?, (**elem).clone(), *by_ref && !elem.is_copy()),
                    Type::List(elem) | Type::Map(_, elem) if self_rooted => {
                        let ex = self.expr(iter)?;
                        let (et, is_map) = match &it_ty {
                            Type::Map(k, v) => (Type::Tuple(vec![(**k).clone(), (**v).clone()]), true),
                            _ => ((**elem).clone(), false),
                        };
                        let _ = is_map;
                        (format!("({}).clone().into_iter()", ex), et, false)
                    }
                    Type::List(elem) if elem.is_copy() || **elem == Type::Unknown => (format!("({}).iter().cloned()", self.expr(iter)?), (**elem).clone(), false),
                    Type::List(elem) => (format!("({}).iter()", self.expr(iter)?), (**elem).clone(), true),
                    Type::Map(k, v) => (format!("({}).iter()", self.expr(iter)?), Type::Tuple(vec![(**k).clone(), (**v).clone()]), true),
                    Type::Unknown => (format!("({}).iter().cloned()", self.expr(iter)?), Type::Unknown, false),
                    other => {
                        return Err(LumeError::new(iter.line, iter.col, format!("cannot loop over a `{}`", type_name(other))).with_help("`for` needs a list or a range"));
                    }
                };
                let pattern = if vars.len() == 1 {
                    rust_name(&vars[0])
                } else {
                    match &elem_ty {
                        Type::Tuple(ts) if ts.len() == vars.len() => {
                            format!("({})", vars.iter().map(|v| rust_name(v)).collect::<Vec<_>>().join(", "))
                        }
                        Type::Tuple(ts) => {
                            return Err(LumeError::new(*line, *col, format!("`for {}` names {} variables, but each item has {} parts", vars.join(", "), vars.len(), ts.len())));
                        }
                        _ => {
                            return Err(LumeError::new(*line, *col, format!("`for {}` names two variables, but the items are not pairs", vars.join(", ")))
                                .with_help("use `xs.enumerate` for (index, item) pairs, or a single variable"));
                        }
                    }
                };
                self.line(&format!("for {} in {} {{", pattern, it));
                self.indent += 1;
                self.push_scope();
                if vars.len() == 1 {
                    self.declare(&vars[0], false, borrowed, elem_ty.clone(), *line);
                } else if let Type::Tuple(ts) = &elem_ty {
                    for (v, t) in vars.iter().zip(ts) {
                        self.declare(v, false, borrowed && !t.is_copy(), t.clone(), *line);
                    }
                }
                if let Some(f) = filter {
                    let fc = self.expr(f)?;
                    self.line(&format!("if !({}) {{ continue; }}", fc));
                }
                self.loop_depth += 1;
                self.block_body(body, false)?;
                self.loop_depth -= 1;
                self.pop_scope();
                self.indent -= 1;
                self.line("}");
            }
            Stmt::Break { line, col } => {
                self.loop_control("break", "break;", *line, *col)?;
            }
            Stmt::Next { line, col } => {
                self.loop_control("next", "continue;", *line, *col)?;
            }
        }
        Ok(())
    }

    fn loop_control(&mut self, kw: &str, rs: &str, line: usize, col: usize) -> Result<()> {
        if self.loop_depth == 0 {
            let e = LumeError::new(line, col, format!("`{}` outside of a loop", kw));
            return Err(if self.in_block {
                e.with_help(format!(
                    "a block runs once per item and cannot `{}`; stop early with `find`, `take_while` or `filter` before `each`, or use a `for` loop",
                    kw
                ))
            } else {
                e
            });
        }
        self.line(rs);
        Ok(())
    }

    /// The root name of a place expression (`a.b[c].d` -> `a`), if any.
    fn place_root<'a>(&self, e: &'a Expr) -> Option<&'a Expr> {
        match &e.kind {
            ExprKind::Ident(_) | ExprKind::SelfRef => Some(e),
            ExprKind::Method { recv, args, .. } if args.is_empty() => self.place_root(recv),
            ExprKind::Index { recv, .. } | ExprKind::TupleIndex { recv, .. } => self.place_root(recv),
            _ => None,
        }
    }

    /// Checks that a place can be written through, and returns the Rust
    /// text of the place as an lvalue.
    fn mutable_place(&mut self, e: &Expr, what: &str, line: usize, col: usize) -> Result<String> {
        let root = match self.place_root(e) {
            Some(r) => r.clone(),
            None => {
                return Err(LumeError::new(line, col, format!("only a named value can be changed, not {}", what)).with_help("bind the value to a `var` first"));
            }
        };
        match &root.kind {
            ExprKind::SelfRef => self.require_var_self(what, line, col)?,
            ExprKind::Ident(n) => match self.lookup(n).cloned() {
                Some(b) if b.mutable => {}
                Some(b) => {
                    return Err(LumeError::new(line, col, format!("`{}` is immutable, so {} cannot be changed", n, what))
                        .with_help(format!("declare it with `var {} = ...` on line {}", n, b.line)));
                }
                None if self.field_type(n).is_some() => self.require_var_self(n, line, col)?,
                None => return Err(self.unknown_name(n, line, col)),
            },
            _ => unreachable!(),
        }
        self.lvalue(e)
    }

    /// Rust lvalue text for a place: indexes become `[i as usize]` on lists
    /// and `get_mut` on maps.
    fn lvalue(&mut self, e: &Expr) -> Result<String> {
        match &e.kind {
            ExprKind::Index { recv, index } => {
                let rt = self.ty_of(recv).materialized();
                let r = self.lvalue(recv)?;
                match rt {
                    Type::List(_) | Type::Unknown => Ok(format!("{}[({}) as usize]", r, self.expr(index)?)),
                    Type::Map(..) => {
                        let k = self.expr_val(index)?;
                        let k = map_key(&rt, &k);
                        Ok(format!("(*{}.get_mut({}).expect(\"no such key in map\"))", r, k))
                    }
                    other => Err(LumeError::new(e.line, e.col, format!("`{}` values cannot be indexed", type_name(&other)))),
                }
            }
            ExprKind::Method { recv, name, args } if args.is_empty() => {
                let r = self.lvalue(recv)?;
                Ok(format!("{}.{}", r, rust_name(name)))
            }
            ExprKind::TupleIndex { recv, index } => Ok(format!("{}.{}", self.lvalue(recv)?, index)),
            _ => self.expr(e),
        }
    }

    fn require_var_self(&self, field: &str, line: usize, col: usize) -> Result<()> {
        if self.current_self != SelfKind::Mutate {
            return Err(LumeError::new(line, col, format!("`{}` changes the field `{}`, but its `self` is read-only", self.current_fn, field))
                .with_help(format!("write `def {}(var self, ...)` to allow it", self.current_fn)));
        }
        Ok(())
    }

    fn tail_unit(&mut self, line: usize, col: usize) -> Result<()> {
        Err(LumeError::new(line, col, format!("the last line of a function returning `{}` must be a value, not a binding", type_name(&self.current_ret)))
            .with_help("put the value on its own line after the binding"))
    }

    fn expr_stmt(&mut self, e: &Expr) -> Result<String> {
        match &e.kind {
            ExprKind::If { branches, else_block } => self.if_chain(branches, else_block.as_ref(), false),
            ExprKind::Match { scrutinee, arms } => self.match_expr(scrutinee, arms, false, e),
            _ => self.expr(e),
        }
    }

    // ----- expressions ------------------------------------------------------

    fn unknown_name(&self, name: &str, line: usize, col: usize) -> LumeError {
        let e = LumeError::new(line, col, format!("unknown name `{}`", name));
        match self.suggest(name) {
            Some(s) => e.with_help(format!("did you mean `{}`?", s)),
            None => e.with_help("names must be bound with `name = value` or `var name = value` before use"),
        }
    }

    fn no_such_member(&self, tname: &str, name: &str, line: usize, col: usize) -> LumeError {
        let e = LumeError::new(line, col, format!("`{}` has no field or method named `{}`", tname, name));
        let fields: Vec<String> = self.structs.get(tname).map(|s| s.fields.iter().map(|(n, _)| n.clone()).collect()).unwrap_or_default();
        let methods: Vec<String> = self.methods_of(tname).map(|m| m.keys().cloned().collect()).unwrap_or_default();
        match self.suggest_from(name, fields.iter().cloned().chain(methods.iter().cloned())) {
            Some(s) => e.with_help(format!("did you mean `{}`?", s)),
            None if !fields.is_empty() => e.with_help(format!("`{}` has fields {}", tname, fields.join(", "))),
            None => e,
        }
    }

    /// True when the expression names a place we cannot move out of: a
    /// borrowed parameter, `self`, or a field reached through either.
    fn is_borrowed_place(&mut self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Ident(n) => match self.lookup(n) {
                Some(b) => b.borrowed,
                None => self.field_type(n).is_some(),
            },
            ExprKind::SelfRef => true,
            ExprKind::Method { recv, name, args } if args.is_empty() => {
                if let Type::Named(s) = self.ty_of(recv) {
                    if let Some(info) = self.structs.get(&s) {
                        return info.fields.iter().any(|(n, _)| n == name);
                    }
                }
                false
            }
            ExprKind::TupleIndex { recv, .. } => self.is_borrowed_place(recv) || self.is_borrowed_ident(recv),
            _ => false,
        }
    }

    fn is_borrowed_ident(&self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Ident(n) => self.lookup(n).map(|b| b.borrowed).unwrap_or(false),
            ExprKind::SelfRef => true,
            _ => false,
        }
    }

    /// Materialise a lazy chain into a Vec of owned values.
    fn collect_iter(&self, it: &str, by_ref: bool) -> String {
        self.collect_iter_t(it, by_ref, &Type::Unknown)
    }

    /// Like `collect_iter`, but knows the item type: borrowed strings may be
    /// `&str` (from `split`/`lines`), which `to_string` handles and `cloned` does not.
    fn collect_iter_t(&self, it: &str, by_ref: bool, elem: &Type) -> String {
        if by_ref && *elem == Type::Str {
            format!("({}).map(|s| s.to_string()).collect::<Vec<String>>()", it)
        } else if by_ref {
            format!("({}).cloned().collect::<Vec<_>>()", it)
        } else {
            format!("({}).collect::<Vec<_>>()", it)
        }
    }

    /// An expression wherever a plain value is needed (printing, comparing,
    /// interpolating): lazy chains are collected, everything else passes.
    fn expr_val(&mut self, e: &Expr) -> Result<String> {
        if let Type::Iter(elem, by_ref) = self.ty_of(e) {
            let s = self.expr(e)?;
            return Ok(self.collect_iter_t(&s, by_ref, &elem));
        }
        self.expr(e)
    }

    /// An expression in a position that needs an owned value (binding,
    /// return, constructor argument, list item). Places that are borrowed
    /// get cloned; owned locals move.
    fn expr_owned(&mut self, e: &Expr) -> Result<String> {
        let t = self.ty_of(e);
        if let Type::Iter(elem, by_ref) = &t {
            let (by_ref, elem) = (*by_ref, (**elem).clone());
            let s = self.expr(e)?;
            return Ok(self.collect_iter_t(&s, by_ref, &elem));
        }
        let s = self.expr(e)?;
        if !t.is_copy() && self.is_borrowed_place(e) {
            // a borrowed string may be a `&str`: to_string covers both
            if t == Type::Str { Ok(format!("{}.to_string()", s)) } else { Ok(format!("{}.clone()", s)) }
        } else {
            Ok(s)
        }
    }

    /// `expr_owned` for a position whose type is known: boxes a concrete
    /// value stored as an interface, element by element for list literals.
    fn expr_owned_as(&mut self, e: &Expr, expected: &Type) -> Result<String> {
        match (&e.kind, expected) {
            (ExprKind::List(items), Type::List(elem)) if self.is_interface(elem) => {
                let mut parts = Vec::new();
                for i in items {
                    let it = self.ty_of(i).materialized();
                    let v = self.expr_owned(i)?;
                    parts.push(self.coerce(v, &it, elem, i.line, i.col)?);
                }
                Ok(format!("vec![{}]", parts.join(", ")))
            }
            (ExprKind::Some(inner), Type::Option(t)) if self.is_interface(t) => {
                let it = self.ty_of(inner).materialized();
                let v = self.expr_owned(inner)?;
                Ok(format!("Some({})", self.coerce(v, &it, t, inner.line, inner.col)?))
            }
            _ => {
                let from = self.ty_of(e).materialized();
                let v = self.expr_owned(e)?;
                self.coerce(v, &from, expected, e.line, e.col)
            }
        }
    }

    /// An argument for a parameter of type `t`: Copy types by value,
    /// everything else by reference.
    fn expr_arg(&mut self, e: &Expr, t: &Type) -> Result<String> {
        if self.is_interface(t) {
            // static dispatch through a generic parameter; the value must conform
            let at = self.ty_of(e).materialized();
            if let Type::Named(iface) = t {
                if at != Type::Unknown && !self.is_interface(&at) {
                    self.require_conforms(&at, iface, e.line, e.col)?;
                }
            }
        }
        if let Type::Iter(elem, by_ref) = self.ty_of(e) {
            let s = self.expr(e)?;
            let c = self.collect_iter_t(&s, by_ref, &elem);
            return Ok(format!("&{}", c));
        }
        let s = self.expr(e)?;
        if t.is_copy() {
            return Ok(if self.is_borrowed_ident(e) && self.ty_of(e).is_copy() { format!("(*{})", s) } else { s });
        }
        if self.is_borrowed_ident(e) {
            Ok(s)
        } else {
            Ok(format!("&{}", s))
        }
    }

    /// An argument for a `var` parameter: must be a `var` binding (or a `var`
    /// parameter, or a field of a `var self`), passed as `&mut`.
    fn expr_var_arg(&mut self, e: &Expr, pname: &str, callee: &str) -> Result<String> {
        match &e.kind {
            ExprKind::Ident(n) => match self.lookup(n).cloned() {
                Some(b) if b.mutable && b.borrowed => Ok(rust_name(n)), // a var parameter: reborrow
                Some(b) if b.mutable => Ok(format!("&mut {}", rust_name(n))),
                Some(b) => Err(LumeError::new(e.line, e.col, format!("`{}` may change `{}`, but `{}` is immutable", callee, pname, n))
                    .with_help(format!("declare it with `var {} = ...` on line {}", n, b.line))),
                None if self.field_type(n).is_some() => {
                    self.require_var_self(n, e.line, e.col)?;
                    Ok(format!("&mut self.{}", rust_name(n)))
                }
                None => Err(self.unknown_name(n, e.line, e.col)),
            },
            _ => Err(LumeError::new(e.line, e.col, format!("`{}` changes its `{}` argument, so it needs a named `var` value, not an expression", callee, pname))
                .with_help("bind the value to a `var` first")),
        }
    }

    /// The iterator a list, range or chain yields: (rust, elem type, by_ref).
    fn iter_base(&mut self, recv: &Expr, e: &Expr) -> Result<(String, Type, bool)> {
        let rt = self.ty_of(recv);
        let r = self.expr(recv)?;
        match rt {
            Type::List(elem) => {
                if elem.is_copy() {
                    Ok((format!("({}).iter().cloned()", r), *elem, false))
                } else {
                    Ok((format!("({}).iter()", r), *elem, true))
                }
            }
            Type::Iter(elem, by_ref) => Ok((r, *elem, by_ref)),
            Type::Map(k, v) => Ok((format!("({}).iter().map(|(k, v)| (k, v))", r), Type::Tuple(vec![*k, *v]), true)),
            Type::Unknown => Err(LumeError::new(e.line, e.col, "cannot tell what kind of value this block is applied to")
                .with_help("add a type to the binding or parameter it comes from")),
            other => Err(LumeError::new(e.line, e.col, format!("`{}` values cannot take a block; only lists and ranges can", type_name(&other)))),
        }
    }

    /// Emits a Rust closure for a Lume block over items of type `elem`.
    /// `pattern_ref` is set for predicates (Rust passes `&Item`). `acc` is
    /// the accumulator type for `fold`.
    fn gen_lambda(&mut self, params: &[String], body: &Block, elem: &Type, by_ref: bool, pattern_ref: bool, want_value: bool, acc: Option<&Type>, at: &Expr) -> Result<String> {
        self.gen_lambda_ex(params, body, elem, by_ref, pattern_ref, want_value, acc, false, at)
    }

    fn gen_lambda_ex(&mut self, params: &[String], body: &Block, elem: &Type, by_ref: bool, pattern_ref: bool, want_value: bool, acc: Option<&Type>, negate: bool, at: &Expr) -> Result<String> {
        let expected = if acc.is_some() { 2 } else if matches!(elem, Type::Tuple(ts) if ts.len() == 2) && params.len() == 2 { 2 } else { 1 };
        if params.len() != expected {
            let msg = if acc.is_some() {
                "`fold` takes a block with two arguments: the accumulator and the item".to_string()
            } else if params.len() == 2 {
                "this block names two arguments, but the items are not pairs".to_string()
            } else {
                format!("this block takes one argument, but {} were named", params.len())
            };
            return Err(LumeError::new(at.line, at.col, msg));
        }
        let names: Vec<String> = params.iter().map(|p| rust_name(p)).collect();
        let item_pat = if acc.is_none() && params.len() == 2 {
            format!("({}, {})", names[0], names[1])
        } else if acc.is_some() {
            names[1].clone()
        } else {
            names[0].clone()
        };
        let item_pat = if pattern_ref && elem.is_copy() { format!("&{}", item_pat) } else { item_pat };
        let pattern = if acc.is_some() { format!("{}, {}", names[0], item_pat) } else { item_pat };
        let item_borrowed = if pattern_ref { !elem.is_copy() } else { by_ref && !elem.is_copy() };
        self.push_scope();
        self.declare_block_params(params, elem, item_borrowed, acc, at.line);
        let saved_loop = self.loop_depth;
        let saved_in_block = self.in_block;
        let saved_tail = self.tail_of_fn;
        self.loop_depth = 0;
        self.in_block = true;
        self.tail_of_fn = false;
        let saved_out = std::mem::take(&mut self.out);
        let base = self.indent;
        let inline = body.stmts.len() == 1 && matches!(body.stmts[0], Stmt::Expr(ref x) if !matches!(x.kind, ExprKind::If { .. } | ExprKind::Match { .. }));
        let text = if inline {
            if let Stmt::Expr(x) = &body.stmts[0] {
                let v = if want_value { self.expr_owned(x)? } else { self.expr_stmt(x)? };
                if negate { format!("|{}| !({})", pattern, v) } else { format!("|{}| {}", pattern, v) }
            } else {
                unreachable!()
            }
        } else {
            self.out.push_str(&format!("|{}| {}{{\n", pattern, if negate { "!" } else { "" }));
            self.nested_block(body, want_value)?;
            self.out.push_str(&"    ".repeat(base));
            self.out.push('}');
            std::mem::take(&mut self.out)
        };
        self.out = saved_out;
        self.loop_depth = saved_loop;
        self.in_block = saved_in_block;
        self.tail_of_fn = saved_tail;
        self.pop_scope();
        Ok(text)
    }

    /// Collection methods that take a block.
    fn block_method(&mut self, recv: &Expr, name: &str, args: &[Arg], lam: &Expr, e: &Expr) -> Result<String> {
        let (params, body) = match &lam.kind {
            ExprKind::Lambda { params, body } => (params, body),
            _ => unreachable!(),
        };
        if name != "fold" && !args.is_empty() {
            return Err(LumeError::new(e.line, e.col, format!("`{}` takes only a block", name)));
        }
        let (it, elem, by_ref) = self.iter_base(recv, e)?;
        Ok(match name {
            "map" => {
                let f = self.gen_lambda(params, body, &elem, by_ref, false, true, None, lam)?;
                format!("{}.map({})", it, f)
            }
            "filter" | "reject" | "take_while" => {
                let f = self.gen_lambda_ex(params, body, &elem, by_ref, true, true, None, name == "reject", lam)?;
                format!("{}.{}({})", it, if name == "reject" { "filter" } else { name }, f)
            }
            "each" => {
                let f = self.gen_lambda(params, body, &elem, by_ref, false, false, None, lam)?;
                format!("{}.for_each({})", it, f)
            }
            "sum" => {
                let bt = self.lambda_body_type(params, &elem, by_ref, None, body);
                let f = self.gen_lambda(params, body, &elem, by_ref, false, true, None, lam)?;
                let rt = if bt == Type::Float { "f64" } else { "i64" };
                format!("{}.map({}).sum::<{}>()", it, f, rt)
            }
            "count" => {
                let f = self.gen_lambda(params, body, &elem, by_ref, true, true, None, lam)?;
                format!("({}.filter({}).count() as i64)", it, f)
            }
            "any?" | "all?" => {
                let f = self.gen_lambda(params, body, &elem, by_ref, false, true, None, lam)?;
                format!("{}.{}({})", it, if name == "any?" { "any" } else { "all" }, f)
            }
            "find" => {
                let f = self.gen_lambda(params, body, &elem, by_ref, true, true, None, lam)?;
                if by_ref {
                    format!("{}.find({}).cloned()", it, f)
                } else {
                    format!("{}.find({})", it, f)
                }
            }
            "sort_by" | "min_by" | "max_by" => {
                let f = self.gen_lambda(params, body, &elem, by_ref, false, true, None, lam)?;
                let owned = self.collect_iter_t(&it, by_ref, &elem);
                // annotate the closure's whole parameter pattern with the item type
                let key = match f.strip_prefix('|').and_then(|rest| rest.find('|').map(|i| (&rest[..i], &rest[i + 1..]))) {
                    Some((pat, body)) => format!("|{}: &{}|{}", pat, self.rt(&elem), body),
                    None => f.clone(),
                };
                match name {
                    "sort_by" => format!("{{ let mut v = {}; let key = {}; v.sort_by(|a, b| key(a).partial_cmp(&key(b)).unwrap()); v }}", owned, key),
                    "min_by" => format!("{{ let key = {}; {}.into_iter().min_by(|a, b| key(a).partial_cmp(&key(b)).unwrap()) }}", key, owned),
                    _ => format!("{{ let key = {}; {}.into_iter().max_by(|a, b| key(a).partial_cmp(&key(b)).unwrap()) }}", key, owned),
                }
            }
            "fold" => {
                if args.len() != 1 {
                    return Err(LumeError::new(e.line, e.col, "`fold` takes the starting value and a block: `xs.fold(0) { |acc, x| acc + x }`"));
                }
                let acc_ty = self.ty_of(&args[0].value).materialized();
                let init = self.expr_owned(&args[0].value)?;
                let f = self.gen_lambda(params, body, &elem, by_ref, false, true, Some(&acc_ty), lam)?;
                format!("{}.fold({}, {})", it, init, f)
            }
            _ => {
                return Err(LumeError::new(e.line, e.col, format!("`{}` does not take a block", name))
                    .with_help("block methods are: map, filter, reject, each, sum, count, any?, all?, find, take_while, sort_by, min_by, max_by, fold"));
            }
        })
    }

    fn if_chain(&mut self, branches: &[(Expr, Block)], else_block: Option<&Block>, want_value: bool) -> Result<String> {
        let saved_tail = self.tail_of_fn;
        if !self.at_tail {
            self.tail_of_fn = false;
        }
        self.at_tail = false;
        let r = self.if_chain_inner(branches, else_block, want_value);
        self.tail_of_fn = saved_tail;
        r
    }

    fn if_chain_inner(&mut self, branches: &[(Expr, Block)], else_block: Option<&Block>, want_value: bool) -> Result<String> {
        let saved = std::mem::take(&mut self.out);
        let base = self.indent;
        let mut first = true;
        for (cond, body) in branches {
            let c = self.expr(cond)?;
            if first {
                self.out.push_str(&format!("if {} {{\n", c));
                first = false;
            } else {
                self.line(&format!("}} else if {} {{", c));
            }
            self.nested_block(body, want_value)?;
        }
        if let Some(eb) = else_block {
            self.line("} else {");
            self.nested_block(eb, want_value)?;
        }
        self.out.push_str(&"    ".repeat(base));
        self.out.push('}');
        Ok(std::mem::replace(&mut self.out, saved))
    }

    // ----- match --------------------------------------------------------------

    fn match_expr(&mut self, scrutinee: &Expr, arms: &[MatchArm], want_value: bool, e: &Expr) -> Result<String> {
        let saved_tail = self.tail_of_fn;
        if !self.at_tail {
            self.tail_of_fn = false;
        }
        self.at_tail = false;
        let r = self.match_expr_inner(scrutinee, arms, want_value, e);
        self.tail_of_fn = saved_tail;
        r
    }

    fn match_expr_inner(&mut self, scrutinee: &Expr, arms: &[MatchArm], want_value: bool, e: &Expr) -> Result<String> {
        let st = self.ty_of(scrutinee).materialized();
        if st == Type::Unknown {
            return Err(LumeError::new(scrutinee.line, scrutinee.col, "cannot tell the type of the value being matched")
                .with_help("bind it to a name with a known type first"));
        }
        let uses_list_pat = arms.iter().any(|a| matches!(a.pat.kind, PatKind::List { .. }));
        let s = self.expr_val(scrutinee)?;
        // How the scrutinee is matched: by value for Copy types, otherwise by reference.
        let by_ref = !st.is_copy();
        let scrut = if uses_list_pat {
            if !matches!(st, Type::List(_)) {
                return Err(LumeError::new(e.line, e.col, format!("a list pattern cannot match a `{}`", type_name(&st))));
            }
            format!("({}).as_slice()", s)
        } else if by_ref && !self.is_borrowed_ident(scrutinee) {
            format!("&({})", s)
        } else {
            format!("({})", s)
        };
        self.check_exhaustive(&st, arms, e)?;

        let saved = std::mem::take(&mut self.out);
        let base = self.indent;
        self.out.push_str(&format!("match {} {{\n", scrut));
        self.indent += 1;
        for arm in arms {
            let cp = self.compile_pattern(&arm.pat, &st, by_ref)?;
            self.push_scope();
            let mut rebinds: Vec<String> = Vec::new();
            for (name, ty, kind) in &cp.binds {
                match kind {
                    BindKind::Deref => {
                        rebinds.push(format!("let {} = *{};", rust_name(name), rust_name(name)));
                        self.declare(name, false, false, ty.clone(), arm.line);
                    }
                    BindKind::Slice => {
                        rebinds.push(format!("let {} = {}.to_vec();", rust_name(name), rust_name(name)));
                        self.declare(name, false, false, ty.clone(), arm.line);
                    }
                    BindKind::Ref => self.declare(name, false, true, ty.clone(), arm.line),
                    BindKind::Owned => self.declare(name, false, false, ty.clone(), arm.line),
                }
            }
            let mut guards = cp.guards.clone();
            if let Some(g) = &arm.guard {
                // In the guard the rebinds have not run yet: Copy bindings are still references.
                for (name, ty, kind) in &cp.binds {
                    if *kind == BindKind::Deref {
                        self.declare(name, false, true, ty.clone(), arm.line);
                    }
                }
                guards.push(self.expr(g)?);
                for (name, ty, kind) in &cp.binds {
                    if *kind == BindKind::Deref {
                        self.declare(name, false, false, ty.clone(), arm.line);
                    }
                }
            }
            let guard_text = if guards.is_empty() { String::new() } else { format!(" if {}", guards.join(" && ")) };
            self.line(&format!("{}{} => {{", cp.text, guard_text));
            self.indent += 1;
            for r in rebinds {
                self.line(&r);
            }
            self.block_body(&arm.body, want_value)?;
            self.indent -= 1;
            self.line("}");
            self.pop_scope();
        }
        self.indent -= 1;
        self.out.push_str(&"    ".repeat(base));
        self.out.push('}');
        let _ = base;
        Ok(std::mem::replace(&mut self.out, saved))
    }

    fn compile_pattern(&mut self, p: &Pattern, t: &Type, by_ref: bool) -> Result<CompiledPat> {
        let mut cp = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new() };
        self.compile_pat_into(p, t, by_ref, &mut cp)?;
        Ok(cp)
    }

    fn compile_pat_into(&mut self, p: &Pattern, t: &Type, by_ref: bool, cp: &mut CompiledPat) -> Result<()> {
        let text = match &p.kind {
            PatKind::Wild => "_".to_string(),
            PatKind::Bind(n) => {
                let kind = if !by_ref {
                    BindKind::Owned
                } else if t.is_copy() {
                    BindKind::Deref
                } else {
                    BindKind::Ref
                };
                cp.binds.push((n.clone(), t.clone(), kind));
                rust_name(n)
            }
            PatKind::Int(v) => {
                self.expect_pat_type(t, &Type::Int, p, "an integer")?;
                format!("{}", v)
            }
            PatKind::Bool(b) => {
                self.expect_pat_type(t, &Type::Bool, p, "a boolean")?;
                b.to_string()
            }
            PatKind::Range { lo, hi, inclusive } => {
                self.expect_pat_type(t, &Type::Int, p, "a range")?;
                if *inclusive { format!("{}..={}", lo, hi) } else { format!("{}..{}", lo, hi) }
            }
            PatKind::Float(v) => {
                self.expect_pat_type(t, &Type::Float, p, "a float")?;
                let tmp = self.fresh("f");
                let deref = if by_ref { "*" } else { "" };
                cp.guards.push(format!("{}{} == {:?}", deref, tmp, v));
                tmp
            }
            PatKind::Str(s) => {
                self.expect_pat_type(t, &Type::Str, p, "a string")?;
                let tmp = self.fresh("s");
                cp.guards.push(format!("{} == \"{}\"", tmp, escape_rust_str(s, false)));
                tmp
            }
            PatKind::Tuple(items) => {
                let ts = match t {
                    Type::Tuple(ts) => ts.clone(),
                    Type::Unknown => vec![Type::Unknown; items.len()],
                    other => return Err(LumeError::new(p.line, p.col, format!("a tuple pattern cannot match a `{}`", type_name(other)))),
                };
                if ts.len() != items.len() {
                    return Err(LumeError::new(p.line, p.col, format!("this tuple has {} parts, but the pattern names {}", ts.len(), items.len())));
                }
                let mut parts = Vec::new();
                for (it, ty) in items.iter().zip(&ts) {
                    let mut sub = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new() };
                    self.compile_pat_into(it, ty, by_ref, &mut sub)?;
                    parts.push(sub.text);
                    cp.guards.extend(sub.guards);
                    cp.binds.extend(sub.binds);
                }
                format!("({})", parts.join(", "))
            }
            PatKind::List { items, rest } => {
                let elem = match t {
                    Type::List(e) => (**e).clone(),
                    other => return Err(LumeError::new(p.line, p.col, format!("a list pattern cannot match a `{}`", type_name(other)))),
                };
                let mut parts = Vec::new();
                for it in items {
                    let mut sub = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new() };
                    // slice elements are always reached by reference
                    self.compile_pat_into(it, &elem, true, &mut sub)?;
                    parts.push(sub.text);
                    cp.guards.extend(sub.guards);
                    cp.binds.extend(sub.binds);
                }
                match rest {
                    Some(Some(r)) => {
                        cp.binds.push((r.clone(), Type::List(Box::new(elem)), BindKind::Slice));
                        parts.push(format!("{} @ ..", rust_name(r)));
                    }
                    Some(None) => parts.push("..".into()),
                    None => {}
                }
                format!("[{}]", parts.join(", "))
            }
            PatKind::Variant { enum_name, name, args } => {
                // Result
                if (name == "Ok" || name == "Error") && matches!(t, Type::Result(..)) {
                    let (ok_t, err_t) = match t {
                        Type::Result(a, b) => ((**a).clone(), (**b).clone()),
                        _ => unreachable!(),
                    };
                    if args.len() != 1 {
                        return Err(LumeError::new(p.line, p.col, format!("`{}` takes exactly one pattern, like `{}(x)`", name, name)));
                    }
                    let inner_t = if name == "Ok" { ok_t } else { err_t };
                    let mut sub = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new() };
                    self.compile_pat_into(&args[0], &inner_t, by_ref, &mut sub)?;
                    cp.guards.extend(sub.guards);
                    cp.binds.extend(sub.binds);
                    cp.text = format!("{}({})", if name == "Ok" { "Ok" } else { "Err" }, sub.text);
                    return Ok(());
                }
                let enum_has_it = matches!(t, Type::Named(n) if self.enums.get(n).map(|e| e.variants.iter().any(|(v, _)| v == name)).unwrap_or(false));
                if (name == "Ok" || name == "Error") && *t != Type::Unknown && !enum_has_it && !matches!(t, Type::Named(n) if n == "Error") {
                    return Err(LumeError::new(p.line, p.col, format!("`{}` matches a `T or E` value, but this is a `{}`", name, type_name(t))));
                }
                // Option
                if name == "Some" || name == "None" {
                    let inner = match t {
                        Type::Option(i) => (**i).clone(),
                        other => {
                            return Err(LumeError::new(p.line, p.col, format!("`{}` matches an optional value, but this is a `{}`", name, type_name(other))));
                        }
                    };
                    if name == "None" {
                        if !args.is_empty() {
                            return Err(LumeError::new(p.line, p.col, "`None` carries no value"));
                        }
                        "None".to_string()
                    } else {
                        if args.len() != 1 {
                            return Err(LumeError::new(p.line, p.col, "`Some` takes exactly one pattern, like `Some(x)`"));
                        }
                        let mut sub = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new() };
                        self.compile_pat_into(&args[0], &inner, by_ref, &mut sub)?;
                        cp.guards.extend(sub.guards);
                        cp.binds.extend(sub.binds);
                        format!("Some({})", sub.text)
                    }
                } else {
                    let en = match (enum_name, t) {
                        (Some(en), _) => self.canon(en),
                        (None, Type::Named(en)) if self.enums.contains_key(en) => en.clone(),
                        (None, _) => match self.resolve_variant(name, p.line, p.col)? {
                            Some(en) => en,
                            None => {
                                return Err(LumeError::new(p.line, p.col, format!("unknown variant `{}`", name))
                                    .with_help("variants start with a capital letter and belong to an `enum`; lowercase names are bindings"));
                            }
                        },
                    };
                    if let Type::Named(actual) = t {
                        if actual != &en {
                            return Err(LumeError::new(p.line, p.col, format!("`{}` is a variant of `{}`, but the value is a `{}`", name, en, actual)));
                        }
                        // A bare name that is not a variant of this enum but is one of another
                        if enum_name.is_none() && !self.enums[actual].variants.iter().any(|(n, _)| n == name) {
                            let owners = self.enums_with_variant(name);
                            if !owners.is_empty() {
                                return Err(LumeError::new(p.line, p.col, format!("`{}` is a variant of `{}`, but the value is a `{}`", name, owners.join("` and `"), actual))
                                    .with_help(format!("the variants of `{}` are: {}", actual, self.enums[actual].variants.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", "))));
                            }
                        }
                    } else if *t != Type::Unknown {
                        return Err(LumeError::new(p.line, p.col, format!("`{}` is a variant of `{}`, but the value is a `{}`", name, en, type_name(t))));
                    }
                    let info = self.enums[&en].clone();
                    let (_, fields) = match info.variants.iter().find(|(n, _)| n == name) {
                        Some(v) => v.clone(),
                        None => {
                            let e = LumeError::new(p.line, p.col, format!("`{}` has no variant `{}`", en, name));
                            let names: Vec<String> = info.variants.iter().map(|(n, _)| n.clone()).collect();
                            return Err(match self.suggest_from(name, names.iter().cloned()) {
                                Some(s) => e.with_help(format!("did you mean `{}`?", s)),
                                None => e.with_help(format!("the variants are: {}", names.join(", "))),
                            });
                        }
                    };
                    let epath = self.path_of(&en);
                    if fields.is_empty() {
                        if !args.is_empty() {
                            return Err(LumeError::new(p.line, p.col, format!("`{}` carries no values; write it without parentheses", name)));
                        }
                        format!("{}::{}", epath, name)
                    } else {
                        if args.len() != fields.len() {
                            return Err(LumeError::new(p.line, p.col, format!(
                                "`{}` carries {} {}, but the pattern names {}",
                                name,
                                fields.len(),
                                plural(fields.len(), "value", "values"),
                                args.len()
                            ))
                            .with_help(format!("write `{}({})`; use `_` for values you do not need", name, fields.iter().map(|(n, _)| n.as_str()).collect::<Vec<_>>().join(", "))));
                        }
                        let mut parts = Vec::new();
                        for (a, (fname, fty)) in args.iter().zip(&fields) {
                            let mut sub = CompiledPat { text: String::new(), guards: Vec::new(), binds: Vec::new() };
                            self.compile_pat_into(a, fty, by_ref, &mut sub)?;
                            parts.push(format!("{}: {}", rust_name(fname), sub.text));
                            cp.guards.extend(sub.guards);
                            cp.binds.extend(sub.binds);
                        }
                        format!("{}::{} {{ {} }}", epath, name, parts.join(", "))
                    }
                }
            }
        };
        cp.text = text;
        Ok(())
    }

    fn expect_pat_type(&self, actual: &Type, want: &Type, p: &Pattern, what: &str) -> Result<()> {
        if actual == want || *actual == Type::Unknown {
            Ok(())
        } else {
            Err(LumeError::new(p.line, p.col, format!("this pattern is {}, but the value is a `{}`", what, type_name(actual))))
        }
    }

    /// A pattern that matches every value of its type, with no guard.
    fn irrefutable(&self, p: &Pattern, t: &Type) -> bool {
        match &p.kind {
            PatKind::Wild | PatKind::Bind(_) => true,
            PatKind::Tuple(items) => match t {
                Type::Tuple(ts) => items.iter().zip(ts).all(|(i, it)| self.irrefutable(i, it)),
                _ => items.iter().all(|i| self.irrefutable(i, &Type::Unknown)),
            },
            PatKind::List { items, rest } => items.is_empty() && rest.is_some(),
            _ => false,
        }
    }

    fn check_exhaustive(&self, t: &Type, arms: &[MatchArm], e: &Expr) -> Result<()> {
        let catch_all = arms.iter().any(|a| a.guard.is_none() && self.irrefutable(&a.pat, t));
        if catch_all {
            return Ok(());
        }
        let missing: Vec<String> = match t {
            Type::Named(en) if self.enums.contains_key(en) => {
                let info = &self.enums[en];
                info.variants
                    .iter()
                    .filter(|(vname, fields)| {
                        !arms.iter().any(|a| {
                            a.guard.is_none()
                                && matches!(&a.pat.kind, PatKind::Variant { name, args, .. } if name == vname
                                    && args.iter().zip(fields).all(|(ap, (_, ft))| self.irrefutable(ap, ft)))
                        })
                    })
                    .map(|(n, _)| {
                        let mentioned = arms.iter().any(|a| matches!(&a.pat.kind, PatKind::Variant { name, .. } if name == n));
                        if mentioned { format!("{} (its arms only match some values: a guard or a nested pattern)", n) } else { n.clone() }
                    })
                    .collect()
            }
            Type::Option(inner) => {
                let mut m = Vec::new();
                let has_none = arms.iter().any(|a| a.guard.is_none() && matches!(&a.pat.kind, PatKind::Variant { name, .. } if name == "None"));
                let has_some = arms.iter().any(|a| {
                    a.guard.is_none() && matches!(&a.pat.kind, PatKind::Variant { name, args, .. } if name == "Some" && args.len() == 1 && self.irrefutable(&args[0], inner))
                });
                if !has_some {
                    m.push("Some(_)".to_string());
                }
                if !has_none {
                    m.push("None".to_string());
                }
                m
            }
            Type::Result(ok_t, err_t) => {
                let mut m = Vec::new();
                let has = |want: &str, inner: &Type| {
                    arms.iter().any(|a| {
                        a.guard.is_none() && matches!(&a.pat.kind, PatKind::Variant { name, args, .. } if name == want && args.len() == 1 && self.irrefutable(&args[0], inner))
                    })
                };
                if !has("Ok", ok_t) {
                    m.push("Ok(_)".to_string());
                }
                if !has("Error", err_t) {
                    m.push("Error(_)".to_string());
                }
                m
            }
            Type::Bool => {
                let mut m = Vec::new();
                for b in [true, false] {
                    if !arms.iter().any(|a| a.guard.is_none() && matches!(a.pat.kind, PatKind::Bool(x) if x == b)) {
                        m.push(b.to_string());
                    }
                }
                m
            }
            Type::List(_) => {
                let empty = arms.iter().any(|a| a.guard.is_none() && matches!(&a.pat.kind, PatKind::List { items, rest: None } if items.is_empty()));
                let one_plus = arms.iter().any(|a| {
                    a.guard.is_none() && matches!(&a.pat.kind, PatKind::List { items, rest: Some(_) } if items.len() == 1 && self.irrefutable(&items[0], &Type::Unknown))
                });
                if empty && one_plus {
                    return Ok(());
                }
                let mut m = Vec::new();
                if !empty {
                    m.push("[]".into());
                }
                if !one_plus {
                    m.push("[x, ..rest]".into());
                }
                m
            }
            _ => vec!["_".into()],
        };
        if missing.is_empty() {
            return Ok(());
        }
        let err = LumeError::new(e.line, e.col, format!("this `match` on `{}` does not cover: {}", type_name(t), missing.join(", ")));
        let plain: Vec<String> = missing.iter().map(|m| m.split(" (").next().unwrap_or(m).to_string()).collect();
        Err(if missing == ["_"] {
            err.with_help("values of this type have too many cases to list; add a final `_ ->` arm")
        } else {
            err.with_help(format!("add {} for {}, or a final `_ ->` arm", plural(plain.len(), "an arm", "arms"), plain.join(" and ")))
        })
    }

    // ----- calls ----------------------------------------------------------------

    /// Reorders positional + keyword arguments to match a parameter list.
    fn bind_args<'a>(&self, what: &str, params: &[(String, Type)], args: &'a [Arg], line: usize, col: usize) -> Result<Vec<&'a Expr>> {
        let mut slots: Vec<Option<&Expr>> = vec![None; params.len()];
        let mut pos = 0;
        for a in args {
            match &a.name {
                None => {
                    if pos >= params.len() {
                        return Err(LumeError::new(line, col, format!(
                            "{} takes {} {}, but {} {} given",
                            what,
                            params.len(),
                            plural(params.len(), "argument", "arguments"),
                            args.len(),
                            plural(args.len(), "was", "were")
                        )));
                    }
                    slots[pos] = Some(&a.value);
                    pos += 1;
                }
                Some(n) => match params.iter().position(|(p, _)| p == n) {
                    Some(i) => {
                        if slots[i].is_some() {
                            return Err(LumeError::new(a.value.line, a.value.col, format!("`{}` is given twice", n)));
                        }
                        slots[i] = Some(&a.value);
                    }
                    None => {
                        let e = LumeError::new(a.value.line, a.value.col, format!("{} has no parameter named `{}`", what, n));
                        let names: Vec<String> = params.iter().map(|(p, _)| p.clone()).collect();
                        return Err(match self.suggest_from(n, names.iter().cloned()) {
                            Some(s) => e.with_help(format!("did you mean `{}`?", s)),
                            None => e.with_help(format!("the parameters are: {}", names.join(", "))),
                        });
                    }
                },
            }
        }
        let missing: Vec<String> = params.iter().zip(&slots).filter(|(_, s)| s.is_none()).map(|((n, _), _)| n.clone()).collect();
        if !missing.is_empty() {
            return Err(LumeError::new(line, col, format!("{} is missing {}: {}", what, plural(missing.len(), "an argument", "arguments"), missing.join(", "))));
        }
        Ok(slots.into_iter().map(|s| s.unwrap()).collect())
    }

    /// `Enum.Variant(args)` or bare `Variant(args)`.
    fn variant_ctor(&mut self, en: &str, vname: &str, args: &[Arg], e: &Expr) -> Result<String> {
        let info = self.enums[en].clone();
        let (_, fields) = match info.variants.iter().find(|(n, _)| n == vname) {
            Some(v) => v.clone(),
            None => {
                let err = LumeError::new(e.line, e.col, format!("`{}` has no variant `{}`", en, vname));
                let names: Vec<String> = info.variants.iter().map(|(n, _)| n.clone()).collect();
                return Err(match self.suggest_from(vname, names.iter().cloned()) {
                    Some(s) => err.with_help(format!("did you mean `{}`?", s)),
                    None => err.with_help(format!("the variants are: {}", names.join(", "))),
                });
            }
        };
        let epath = self.path_of(en);
        if fields.is_empty() {
            if !args.is_empty() {
                return Err(LumeError::new(e.line, e.col, format!("`{}` carries no values; write `{}.{}` without parentheses", vname, en, vname)));
            }
            return Ok(format!("{}::{}", epath, vname));
        }
        let bound = self.bind_args(&format!("`{}.{}`", en, vname), &fields, args, e.line, e.col)?;
        let mut parts = Vec::new();
        for (a, (fname, _)) in bound.iter().zip(&fields) {
            parts.push(format!("{}: {}", rust_name(fname), self.expr_owned(a)?));
        }
        Ok(format!("{}::{} {{ {} }}", epath, vname, parts.join(", ")))
    }

    fn expr(&mut self, e: &Expr) -> Result<String> {
        Ok(match &e.kind {
            ExprKind::Int(v) => format!("{}i64", v),
            ExprKind::Float(v) => {
                let s = format!("{}", v);
                if s.contains('.') || s.contains('e') { format!("{}f64", s) } else { format!("{}.0f64", s) }
            }
            ExprKind::Bool(b) => b.to_string(),
            ExprKind::Str(pieces) => {
                let only_lit = pieces.iter().all(|p| matches!(p, StrPiece::Lit(_)));
                if only_lit {
                    let s: String = pieces
                        .iter()
                        .map(|p| match p {
                            StrPiece::Lit(s) => s.clone(),
                            _ => unreachable!(),
                        })
                        .collect();
                    format!("String::from(\"{}\")", escape_rust_str(&s, false))
                } else {
                    let mut fmt = String::new();
                    let mut args = Vec::new();
                    for p in pieces {
                        match p {
                            StrPiece::Lit(s) => fmt.push_str(&escape_rust_str(s, true)),
                            StrPiece::Expr(x) => {
                                fmt.push_str("{}");
                                args.push(format!("({}).lume_str()", self.expr_val(x)?));
                            }
                        }
                    }
                    format!("format!(\"{}\", {})", fmt, args.join(", "))
                }
            }
            ExprKind::Ident(name) => {
                if self.lookup(name).is_some() {
                    rust_name(name)
                } else if self.field_type(name).is_some() {
                    format!("self.{}", rust_name(name))
                } else if let Some(m) = self.bare_method(name) {
                    if m.self_kind == SelfKind::Mutate {
                        self.require_var_self(name, e.line, e.col)?;
                    }
                    format!("self.{}()", rust_name(name))
                } else if self.current_self_ty.as_ref().map(|st| builtin_method_type(st, name) != Type::Unknown || is_builtin_name(name)).unwrap_or(false) {
                    // inside `extend Str with ...`: a bare built-in method name is `self.name`
                    let call = Expr::new(ExprKind::Method { recv: Box::new(Expr::new(ExprKind::SelfRef, e.line, e.col)), name: name.clone(), args: vec![] }, e.line, e.col);
                    return self.expr(&call);
                } else if let Some(en) = self.resolve_variant(name, e.line, e.col)? {
                    self.variant_ctor(&en, name, &[], e)?
                } else if self.fns.contains_key(name) {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is a function; call it with `{}()`", name, name)));
                } else if self.structs.contains_key(name) {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is a type; construct one with `{}(...)`", name, name)));
                } else if self.enums.contains_key(name) {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is an enum; pick a variant like `{}.{}`", name, name, self.enums[name].variants[0].0)));
                } else {
                    return Err(self.unknown_name(name, e.line, e.col));
                }
            }
            ExprKind::SelfRef => {
                if self.current_type.is_none() {
                    return Err(LumeError::new(e.line, e.col, "`self` is only meaningful inside a method"));
                }
                "self".into()
            }
            ExprKind::List(items) => {
                let parts: Result<Vec<String>> = items.iter().map(|i| self.expr_owned(i)).collect();
                format!("vec![{}]", parts?.join(", "))
            }
            ExprKind::Tuple(items) => {
                let parts: Result<Vec<String>> = items.iter().map(|i| self.expr_owned(i)).collect();
                format!("({})", parts?.join(", "))
            }
            ExprKind::TupleIndex { recv, index } => {
                let rt = self.ty_of(recv);
                match &rt {
                    Type::Tuple(ts) => {
                        if *index >= ts.len() {
                            return Err(LumeError::new(e.line, e.col, format!("this tuple has {} parts, so `.{}` does not exist", ts.len(), index)));
                        }
                    }
                    Type::Unknown => {}
                    other => return Err(LumeError::new(e.line, e.col, format!("`.{}` needs a tuple, but this is a `{}`", index, type_name(other)))),
                }
                format!("{}.{}", self.expr(recv)?, index)
            }
            ExprKind::Some(x) => format!("Some({})", self.expr_owned(x)?),
            ExprKind::None => "None".into(),
            ExprKind::Try(x) => {
                let xt = self.ty_of(x);
                if self.in_block {
                    return Err(LumeError::new(e.line, e.col, "`?` cannot be used inside a block").with_help("handle the missing or failed value with `match` or `.or(default)` inside the block"));
                }
                match (&xt, &self.current_ret) {
                    (Type::Option(_), Type::Option(_)) | (Type::Result(..), Type::Result(..)) | (Type::Unknown, _) => {}
                    (Type::Option(_), Type::Result(..)) => {
                        return Err(LumeError::new(e.line, e.col, format!("`?` on an optional value returns `None`, but `{}` returns `{}`", self.current_fn, type_name(&self.current_ret)))
                            .with_help("turn the absence into an error first: `.or_error(\"what went wrong\")?`"));
                    }
                    (Type::Result(..), Type::Option(_)) => {
                        return Err(LumeError::new(e.line, e.col, format!("`?` on a `{}` returns the error, but `{}` returns `{}`", type_name(&xt), self.current_fn, type_name(&self.current_ret)))
                            .with_help("make the function return `T or Error`, or handle the error with `match` or `.or(default)`"));
                    }
                    (Type::Option(_), _) => {
                        return Err(LumeError::new(e.line, e.col, format!("`?` returns `None` early, but `{}` does not return an optional value", self.current_fn))
                            .with_help("make the function return `T?`, or handle the `None` case with `match` or `.or(default)`"));
                    }
                    (Type::Result(..), _) => {
                        return Err(LumeError::new(e.line, e.col, format!("`?` returns the error early, but `{}` does not return `T or E`", self.current_fn))
                            .with_help(format!("write `def {}(...) -> Type or Error`, or handle the error with `match` or `.or(default)`", self.current_fn)));
                    }
                    (other, _) => {
                        return Err(LumeError::new(e.line, e.col, format!("`?` needs an optional value or a `T or E`, but this is a `{}`", type_name(other))));
                    }
                }
                format!("({})?", self.expr(x)?)
            }
            ExprKind::Unwrap(x) => {
                let xt = self.ty_of(x);
                match xt {
                    Type::Option(_) | Type::Result(..) | Type::Unknown => {}
                    other => return Err(LumeError::new(e.line, e.col, format!("`!` unwraps an optional or a `T or E`, but this is a `{}`", type_name(&other)))),
                }
                self.warnings.push(
                    LumeError::new(e.line, e.col, "`!` stops the program if the value is missing or an error")
                        .with_help("fine in tests and quick scripts; elsewhere use `match`, `?` or `.or(default)`"),
                );
                let inner = self.expr(x)?;
                match xt {
                    Type::Result(..) => format!("({}).unwrap_or_else(|e| panic!(\"{{}}\", e.message))", inner),
                    _ => format!("({}).expect(\"expected a value, found None\")", inner),
                }
            }
            ExprKind::Ok(x) => format!("Ok({})", self.expr_owned(x)?),
            ExprKind::Index { recv, index } => {
                let rt = self.ty_of(recv).materialized();
                let r = self.expr_val(recv)?;
                match rt {
                    Type::List(_) => {
                        let i = self.expr(index)?;
                        let it = self.ty_of(index);
                        if it != Type::Int && it != Type::Unknown {
                            return Err(LumeError::new(index.line, index.col, format!("a list position is an `Int`, but this is a `{}`", type_name(&it))));
                        }
                        format!("({}).get(({}) as usize).cloned()", r, i)
                    }
                    Type::Map(ref k, _) => {
                        let key = self.expr_val(index)?;
                        let kt = self.ty_of(index);
                        if kt != **k && kt != Type::Unknown {
                            return Err(LumeError::new(index.line, index.col, format!("this map's keys are `{}`, but the key is a `{}`", type_name(k), type_name(&kt))));
                        }
                        let key = map_key(&rt, &key);
                        format!("({}).get({}).cloned()", r, key)
                    }
                    Type::Str => {
                        return Err(LumeError::new(e.line, e.col, "strings cannot be indexed by position")
                            .with_help("use `.chars`, `.first`, `.slice(from, len)` or `.split`"));
                    }
                    Type::Unknown => format!("({}).get(({}) as usize).cloned()", r, self.expr(index)?),
                    other => return Err(LumeError::new(e.line, e.col, format!("`{}` values cannot be indexed", type_name(&other)))),
                }
            }
            ExprKind::MapLit(pairs) => {
                if pairs.is_empty() {
                    "LumeMap::new()".to_string()
                } else {
                    let mut parts = Vec::new();
                    for (k, v) in pairs {
                        parts.push(format!("({}, {})", self.expr_owned(k)?, self.expr_owned(v)?));
                    }
                    format!("LumeMap::from([{}])", parts.join(", "))
                }
            }
            ExprKind::Rust(code) => {
                self.has_rust_blocks = true;
                if code.contains('\n') {
                    let pad = "    ".repeat(self.indent + 1);
                    let body = code.lines().map(|l| format!("{}{}", pad, l)).collect::<Vec<_>>().join("\n");
                    format!("{{\n{}\n{}}}", body, "    ".repeat(self.indent))
                } else {
                    format!("{{ {} }}", code)
                }
            }
            ExprKind::Range { lo, hi, inclusive } => {
                let l = self.expr(lo)?;
                let h = self.expr(hi)?;
                if *inclusive { format!("({}..={})", l, h) } else { format!("({}..{})", l, h) }
            }
            ExprKind::Unary { op, expr } => {
                let x = self.expr(expr)?;
                match *op {
                    "not" => format!("(!{})", x),
                    "-" => format!("(-{})", x),
                    _ => unreachable!(),
                }
            }
            ExprKind::Binary { op, lhs, rhs } => {
                // user-defined operators: `def +(other)` on the left operand's type
                let lt0 = self.ty_of(lhs).materialized();
                if let Type::Named(tn) = &lt0 {
                    let lookup = |g: &Self, o: &str| g.methods_of(tn).and_then(|m| m.get(o).cloned());
                    // `!=` follows from `==`; `>`, `<=`, `>=` follow from `<`
                    let (found, method, negate, swap) = match lookup(self, op) {
                        Some(sg) => (Some(sg), *op, false, false),
                        None if *op == "!=" => (lookup(self, "=="), "==", true, false),
                        None if *op == ">" => (lookup(self, "<"), "<", false, true),
                        None if *op == "<=" => (lookup(self, "<"), "<", true, true),
                        None if *op == ">=" => (lookup(self, "<"), "<", true, false),
                        None => (None, *op, false, false),
                    };
                    if let Some(sg) = found {
                        let call = if swap {
                            let l = self.expr_arg(lhs, &sg.params[0].1)?;
                            let r = self.expr(rhs)?;
                            format!("({}).{}({})", r, rust_name(method), l)
                        } else {
                            let l = self.expr(lhs)?;
                            let r = self.expr_arg(rhs, &sg.params[0].1)?;
                            format!("({}).{}({})", l, rust_name(method), r)
                        };
                        return Ok(if negate { format!("(!{})", call) } else { call });
                    }
                    let user_type = self.structs.contains_key(&self.canon(tn)) || self.enums.contains_key(&self.canon(tn));
                    if user_type && matches!(*op, "+" | "-" | "*" | "/" | "%" | "<" | "<=" | ">" | ">=") {
                        let (want, extra) = match *op {
                            "<" | "<=" | ">" | ">=" => ("<", "; `<=`, `>` and `>=` then follow"),
                            _ => (*op, ""),
                        };
                        return Err(LumeError::new(e.line, e.col, format!("`{}` has no `{}` operator", tn, op))
                            .with_help(format!("add `def {}(other: {}) -> {}:` to `{}`{}", want, tn, if want == "<" { "Bool" } else { tn }, tn, extra)));
                    }
                    if user_type && *op == "**" {
                        return Err(LumeError::new(e.line, e.col, format!("`{}` has no `**` operator", tn)).with_help("`**` is not overloadable; write a method such as `def pow(n: Int)`"));
                    }
                }
                if matches!(*op, "+" | "-" | "*" | "/" | "%" | "**" | "<" | "<=" | ">" | ">=") {
                    for side in [lhs, rhs] {
                        let st = self.ty_of(side);
                        let what = match &st {
                            Type::Option(_) => Some("may be absent"),
                            Type::Result(..) => Some("may be an error"),
                            _ => None,
                        };
                        if let Some(w) = what {
                            return Err(LumeError::new(side.line, side.col, format!("this value is a `{}` — it {}, so it cannot be used with `{}` directly", type_name(&st), w, op))
                                .with_help(if matches!(st, Type::Option(_)) { "unwrap it first: `match` on `Some(x)`/`None`, `?`, or `.or(default)`" } else { "unwrap it first: `?` to pass the error up, `match` on `Ok(x)`/`Error(e)`, or `.or(default)`" }));
                        }
                    }
                }
                let mut l = self.expr_val(lhs)?;
                let mut r = self.expr_val(rhs)?;
                let lt = self.ty_of(lhs);
                let lb = self.is_borrowed_ident(lhs);
                let rb = self.is_borrowed_ident(rhs);
                if lb && !rb {
                    l = format!("(*{})", l);
                }
                if rb && !lb {
                    r = format!("(*{})", r);
                }
                match *op {
                    "and" => format!("({} && {})", l, r),
                    "or" => format!("({} || {})", l, r),
                    "**" => format!("({}).lume_pow({})", l, r),
                    "+" if lt == Type::Str || self.ty_of(rhs) == Type::Str => {
                        format!("format!(\"{{}}{{}}\", ({}).lume_str(), ({}).lume_str())", l, r)
                    }
                    _ => format!("({} {} {})", l, op, r),
                }
            }
            ExprKind::Call { name: raw_name, args } => {
                let cname = self.canon(raw_name);
                let name = &cname;
                if let Some(sig) = self.fns.get(name).cloned() {
                    let bound = self.bind_args(&format!("`{}`", name), &sig.params, args, e.line, e.col)?;
                    let mut parts = Vec::new();
                    for (i, (a, (pname, t))) in bound.iter().zip(&sig.params).enumerate() {
                        if sig.var_params[i] {
                            parts.push(self.expr_var_arg(a, pname, name)?);
                        } else {
                            parts.push(self.expr_arg(a, t)?);
                        }
                    }
                    let callee = if self.paths.contains_key(name) { self.path_of(name) } else { rust_name(name) };
                    format!("{}({})", callee, parts.join(", "))
                } else if let Some(info) = self.structs.get(name).cloned() {
                    let bound = self.bind_args(&format!("`{}`", name), &info.fields, args, e.line, e.col)?;
                    let mut parts = Vec::new();
                    for (a, (fname, fty)) in bound.iter().zip(&info.fields) {
                        parts.push(format!("{}: {}", rust_name(fname), self.expr_owned_as(a, fty)?));
                    }
                    format!("{} {{ {} }}", self.path_of(name), parts.join(", "))
                } else if let Some(en) = self.resolve_variant(name, e.line, e.col)? {
                    self.variant_ctor(&en, name, args, e)?
                } else if self.lookup(name).is_some() {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is a value, not a function", name)));
                } else if let Some(tn) = self.current_type.clone() {
                    if self.methods_of(&tn).map(|m| m.contains_key(name)).unwrap_or(false) {
                        // a sibling method called bare: `deps_of(pkg)` is `self.deps_of(pkg)`
                        let call = Expr::new(
                            ExprKind::Method { recv: Box::new(Expr::new(ExprKind::SelfRef, e.line, e.col)), name: name.clone(), args: args.clone() },
                            e.line,
                            e.col,
                        );
                        return self.expr(&call);
                    }
                    return Err(self.unknown_fn(name, e.line, e.col));
                } else {
                    return Err(self.unknown_fn(name, e.line, e.col));
                }
            }
            ExprKind::Method { recv, name, args } => {
                if let Some(ne) = self.split_trailing_try(e) {
                    return self.expr(&ne);
                }
                if let Some(ne) = self.fn_ref_as_block(e) {
                    return self.expr(&ne);
                }
                if let Some(ne) = self.module_ref(e)? {
                    return self.expr(&ne);
                }
                // Enum.Variant(...) constructor
                if let ExprKind::Ident(tn) = &recv.kind {
                    let ctn = self.canon(tn);
                    if self.lookup(tn).is_none() && self.enums.contains_key(&ctn) {
                        return self.variant_ctor(&ctn, name, args, e);
                    }
                    if self.lookup(tn).is_none() && builtin_namespace_type(tn, name).is_some() {
                        let mut parts = Vec::new();
                        for a in args {
                            parts.push(self.expr_val(&a.value)?);
                        }
                        let need = |n: usize| -> Result<()> {
                            if parts.len() != n {
                                Err(LumeError::new(e.line, e.col, format!("`{}.{}` takes {} {}", tn, name, n, plural(n, "argument", "arguments"))))
                            } else {
                                Ok(())
                            }
                        };
                        return Ok(match (tn.as_str(), name.as_str()) {
                            ("File", "read") => { need(1)?; format!("lume_read_file(&{})", parts[0]) }
                            ("File", "write") => { need(2)?; format!("lume_write_file(&{}, &{})", parts[0], parts[1]) }
                            ("File", "exists?") => { need(1)?; format!("std::path::Path::new(&*{}).exists()", parts[0]) }
                            ("Env", "args") => { need(0)?; "lume_args()".to_string() }
                            ("Env", "get") => { need(1)?; format!("std::env::var(&*{}).ok()", parts[0]) }
                            ("Time", "now") => { need(0)?; "lume_now()".to_string() }
                            _ => unreachable!(),
                        });
                    }
                    if self.lookup(tn).is_none() && (tn == "File" || tn == "Env" || tn == "Time") {
                        return Err(LumeError::new(e.line, e.col, format!("`{}` has no `{}`", tn, name))
                            .with_help(match tn.as_str() { "File" => "File has read(path), write(path, text) and exists?(path)", "Env" => "Env has args and get(name)", _ => "Time has now" }));
                    }
                    if self.lookup(tn).is_none() && self.structs.contains_key(tn) {
                        return Err(LumeError::new(e.line, e.col, format!("`{}.{}` — static methods on a type are not supported yet", tn, name))
                            .with_help(format!("construct a value with `{}(...)` and call the method on it", tn)));
                    }
                }
                if let Some(last) = args.last() {
                    if matches!(last.value.kind, ExprKind::Lambda { .. }) {
                        return self.block_method(recv, name, &args[..args.len() - 1], &last.value, e);
                    }
                }
                let rt = self.ty_of(recv);
                // Lazy chains: a few methods stay lazy, the rest collect first.
                if let Type::Iter(elem, by_ref) = &rt {
                    let r = self.expr(recv)?;
                    match name.as_str() {
                        "take" | "skip" if args.len() == 1 => {
                            let n = self.expr(&args[0].value)?;
                            return Ok(format!("{}.{}(({}) as usize)", r, name, n));
                        }
                        "enumerate" if args.is_empty() => {
                            return Ok(format!("{}.enumerate().map(|(i, x)| (i as i64, x))", r));
                        }
                        "len" if args.is_empty() => return Ok(format!("({}.count() as i64)", r)),
                        "to_list" if args.is_empty() => return Ok(self.collect_iter_t(&r, *by_ref, elem)),
                        "sum" if args.is_empty() => {
                            let t = if **elem == Type::Float { "f64" } else { "i64" };
                            let r = if *by_ref { format!("{}.cloned()", r) } else { r };
                            return Ok(format!("{}.sum::<{}>()", r, t));
                        }
                        "max" | "min" if args.is_empty() && elem.is_copy() => {
                            if **elem == Type::Float {
                                let r = if *by_ref { format!("{}.cloned()", r) } else { r };
                                return Ok(format!("{}.{}_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))", r, name));
                            }
                            return Ok(format!("{}.{}()", r, name));
                        }
                        "first" if args.is_empty() => {
                            let r = if *by_ref { format!("{}.cloned()", r) } else { r };
                            return Ok(format!("{}.next()", r));
                        }
                        "empty?" | "any?" if args.is_empty() => {
                            let neg = if name == "empty?" { "" } else { "!" };
                            return Ok(format!("({}{}.next().is_none())", neg, r));
                        }
                        _ => {
                            let collected = self.collect_iter_t(&r, *by_ref, elem);
                            let mut parts = Vec::new();
                            for a in args {
                                parts.push(self.expr_val(&a.value)?);
                            }
                            return self.method(&collected, name, &parts, &rt.materialized(), e);
                        }
                    }
                }
                let r = self.expr(recv)?;
                if let Type::Named(tname) = &rt {
                    if let Some(info) = self.structs.get(tname).cloned() {
                        if info.fields.iter().any(|(n, _)| n == name) {
                            if !args.is_empty() {
                                return Err(LumeError::new(e.line, e.col, format!("`{}` is a field of `{}`, not a method; it takes no arguments", name, tname)));
                            }
                            return Ok(format!("{}.{}", r, rust_name(name)));
                        }
                    }
                    // universal methods on user types
                    if (name == "to_s" || name == "to_str") && args.is_empty() && self.methods_of(tname).map(|m| !m.contains_key(name)).unwrap_or(true) {
                        return Ok(format!("({}).lume_str()", r));
                    }
                    if let Some(m) = self.methods_of(tname).and_then(|m| m.get(name).cloned()) {
                        let bound = self.bind_args(&format!("`{}.{}`", tname, name), &m.params, args, e.line, e.col)?;
                        let mut parts = Vec::new();
                        for (i, (a, (pname, t))) in bound.iter().zip(&m.params).enumerate() {
                            if m.var_params[i] {
                                parts.push(self.expr_var_arg(a, pname, name)?);
                            } else {
                                parts.push(self.expr_arg(a, t)?);
                            }
                        }
                        if m.self_kind == SelfKind::Mutate {
                            self.check_receiver_mutable(recv, name, e.line, e.col)?;
                        }
                        return Ok(format!("{}.{}({})", r, rust_name(name), parts.join(", ")));
                    }
                    return Err(self.no_such_member(tname, name, e.line, e.col));
                }
                // methods a built-in type gained through `extend`
                if !matches!(rt, Type::Named(_)) {
                    let key = self.type_key(&rt);
                    if let Some(m) = self.ext_methods.get(&key).and_then(|m| m.get(name)).cloned() {
                        let bound = self.bind_args(&format!("`{}.{}`", type_name(&rt), name), &m.params, args, e.line, e.col)?;
                        let mut parts = Vec::new();
                        for (a, (_, t)) in bound.iter().zip(&m.params) {
                            parts.push(self.expr_arg(a, t)?);
                        }
                        return Ok(format!("({}).{}({})", r, rust_name(name), parts.join(", ")));
                    }
                }
                if let Type::Option(inner) = &rt {
                    if !matches!(name.as_str(), "or" | "some?" | "none?" | "or_error" | "to_s" | "to_str") {
                        let known = match &**inner {
                            Type::Named(tn) => self.methods_of(tn).map(|m| m.contains_key(name)).unwrap_or(false)
                                || self.structs.get(tn).map(|s| s.fields.iter().any(|(n, _)| n == name)).unwrap_or(false),
                            other => builtin_method_type(other, name) != Type::Unknown,
                        };
                        let err = LumeError::new(e.line, e.col, format!("this value is a `{}` — it may be absent, so `.{}` cannot be called on it directly", type_name(&rt), name));
                        return Err(if known {
                            err.with_help(format!("unwrap it first: `match` on `Some(x)`/`None`, or `.or(default).{}`", name))
                        } else {
                            err.with_help("unwrap it first with `match` on `Some(x)`/`None`, or `.or(default)`")
                        });
                    }
                }
                if let Type::Result(..) = &rt {
                    if !matches!(name.as_str(), "or" | "ok?" | "error?" | "error" | "to_s" | "to_str") {
                        return Err(LumeError::new(e.line, e.col, format!("this value is a `{}` — it may be an error, so `.{}` cannot be called on it directly", type_name(&rt), name))
                            .with_help("unwrap it first: `?` to pass the error up, `match` on `Ok(x)`/`Error(e)`, or `.or(default)`"));
                    }
                }
                if rt != Type::Unknown && builtin_method_type(&rt, name) == Type::Unknown && !is_builtin_name(name) {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` values have no method `{}`", type_name(&rt), name)));
                }
                if args.iter().any(|a| a.name.is_some()) {
                    return Err(LumeError::new(e.line, e.col, format!("built-in method `{}` does not take keyword arguments", name)));
                }
                let mut parts = Vec::new();
                for a in args {
                    // `.or(default)` and `.push(x)` store their argument: owned position
                    parts.push(match (name.as_str(), &rt) {
                        ("push", Type::List(elem)) => self.expr_owned_as(&a.value, &elem.clone())?,
                        ("or" | "push", _) => self.expr_owned(&a.value)?,
                        _ => self.expr_val(&a.value)?,
                    });
                }
                if let Type::List(elem) = &rt {
                    let base = if elem.is_copy() { format!("({}).iter().cloned()", r) } else { format!("({}).iter()", r) };
                    match name.as_str() {
                        "take" | "skip" if parts.len() == 1 => return Ok(format!("{}.{}(({}) as usize)", base, name, parts[0])),
                        "enumerate" if parts.is_empty() => return Ok(format!("{}.enumerate().map(|(i, x)| (i as i64, x))", base)),
                        "to_list" if parts.is_empty() => return Ok(format!("({}).clone()", r)),
                        _ => {}
                    }
                }
                self.method(&r, name, &parts, &rt, e)?
            }
            ExprKind::If { branches, else_block } => {
                if else_block.is_none() {
                    return Err(LumeError::new(e.line, e.col, "an `if` used as a value needs an `else`").with_help("every branch must produce the value"));
                }
                self.if_chain(branches, else_block.as_ref(), true)?
            }
            ExprKind::Match { scrutinee, arms } => self.match_expr(scrutinee, arms, true, e)?,
            ExprKind::Puts(arg) => {
                let a = self.expr_val(arg)?;
                format!("println!(\"{{}}\", ({}).lume_str())", a)
            }
            ExprKind::Placeholder => {
                return Err(LumeError::new(e.line, e.col, "`_` can only be used inside a method argument").with_help("write `xs.map(_.name)`; elsewhere give the value a name"));
            }
            ExprKind::Lambda { .. } => {
                return Err(LumeError::new(e.line, e.col, "a block must follow a method call").with_help("write `xs.each do |x|` or `xs.map { |x| ... }`"));
            }
        })
    }

    fn check_receiver_mutable(&self, recv: &Expr, method: &str, line: usize, col: usize) -> Result<()> {
        match &recv.kind {
            ExprKind::Ident(n) => match self.lookup(n) {
                Some(b) if b.mutable => Ok(()),
                Some(b) => Err(LumeError::new(line, col, format!("`{}` is immutable, but `{}` changes it", n, method))
                    .with_help(format!("declare it with `var {} = ...` on line {}", n, b.line))),
                None if self.field_type(n).is_some() => self.require_var_self(n, line, col),
                None => Ok(()),
            },
            ExprKind::SelfRef => self.require_var_self(method, line, col),
            _ => Ok(()),
        }
    }

    fn unknown_fn(&self, name: &str, line: usize, col: usize) -> LumeError {
        let err = LumeError::new(line, col, format!("unknown function `{}`", name));
        match self.suggest(name) {
            Some(s) => err.with_help(format!("did you mean `{}`?", s)),
            None => err,
        }
    }

    /// Maps built-in method names onto Rust. `rt` is the receiver's type.
    fn method(&mut self, recv: &str, name: &str, args: &[String], rt: &Type, e: &Expr) -> Result<String> {
        let need = |n: usize| -> Result<()> {
            if args.len() != n {
                Err(LumeError::new(e.line, e.col, format!(
                    "`.{}` takes {} {}, but {} {} given",
                    name, n, plural(n, "argument", "arguments"), args.len(), plural(args.len(), "was", "were")
                )))
            } else {
                Ok(())
            }
        };
        Ok(match name {
            "pad" => { need(1)?; format!("lume_pad({}, {})", recv, args[0]) }
            "keys" if matches!(rt, Type::Map(..)) => { need(0)?; format!("({}).keys().cloned().collect::<Vec<_>>()", recv) }
            "values" if matches!(rt, Type::Map(..)) => { need(0)?; format!("({}).values().cloned().collect::<Vec<_>>()", recv) }
            "to_list" if matches!(rt, Type::Map(..)) => { need(0)?; format!("({}).iter().map(|(k, v)| (k.clone(), v.clone())).collect::<Vec<_>>()", recv) }
            "contains?" if matches!(rt, Type::Map(..)) => { need(1)?; format!("({}).contains_key({})", recv, map_key(rt, &args[0])) }
            "remove" if matches!(rt, Type::Map(..)) => { need(1)?; format!("({}).remove({})", recv, map_key(rt, &args[0])) }
            "get" if matches!(rt, Type::Map(..)) => { need(1)?; format!("({}).get({}).cloned()", recv, map_key(rt, &args[0])) }
            "len" => { need(0)?; format!("({}).lume_len()", recv) }
            "empty?" => { need(0)?; format!("({}).lume_empty()", recv) }
            "any?" => { need(0)?; format!("(!({}).lume_empty())", recv) }
            "to_str" | "to_s" => { need(0)?; format!("({}).lume_str()", recv) }
            "to_float" => { need(0)?; if *rt == Type::Str { format!("lume_to_float(&{})", recv) } else { format!("(({}) as f64)", recv) } }
            "to_int" => { need(0)?; if *rt == Type::Str { format!("lume_to_int(&{})", recv) } else { format!("(({}) as i64)", recv) } }
            "upcase" => { need(0)?; format!("({}).to_uppercase()", recv) }
            "downcase" => { need(0)?; format!("({}).to_lowercase()", recv) }
            "trim" => { need(0)?; format!("({}).trim().to_string()", recv) }
            "sqrt" | "abs" | "floor" | "ceil" | "round" => { need(0)?; format!("({}).{}()", recv, name) }
            "sum" => { need(0)?; format!("({}).lume_sum()", recv) }
            "push" => { need(1)?; format!("({}).push({})", recv, args[0]) }
            "pop" => { need(0)?; format!("({}).pop()", recv) }
            "first" => { need(0)?; format!("({}).first().cloned()", recv) }
            "last" => { need(0)?; format!("({}).last().cloned()", recv) }
            "contains?" => {
                need(1)?;
                match rt {
                    Type::Str => format!("({}).contains(&*({}))", recv, args[0]),
                    Type::List(e) if **e == Type::Str => format!("({}).iter().any(|x| x.as_str() == &*({}))", recv, args[0]),
                    _ => format!("({}).contains(&({}))", recv, args[0]),
                }
            }
            "reverse" => { need(0)?; format!("{{ let mut v = ({}).clone(); v.reverse(); v }}", recv) }
            "sort" | "max" | "min" => {
                need(0)?;
                let partial = match rt {
                    Type::List(el) => {
                        if let Type::Named(tn) = &**el {
                            let key = self.canon(tn);
                            if (self.structs.contains_key(&key) || self.enums.contains_key(&key)) && self.methods_of(tn).map(|m| !m.contains_key("<")).unwrap_or(true) {
                                return Err(LumeError::new(e.line, e.col, format!("`.{}` needs to compare `{}` values, but `{}` has no `<` operator", name, tn, tn))
                                    .with_help(format!("add `def <(other: {}) -> Bool:` to `{}`, or use `.{}_by(_.field)`", tn, tn, if name == "sort" { "sort" } else { name })));
                            }
                            true
                        } else {
                            **el == Type::Float
                        }
                    }
                    _ => false,
                };
                match (name, partial) {
                    ("sort", true) => format!("{{ let mut v = ({}).clone(); v.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal)); v }}", recv),
                    ("sort", false) => format!("{{ let mut v = ({}).clone(); v.sort(); v }}", recv),
                    (_, true) => format!("({}).iter().cloned().{}_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))", recv, name),
                    (_, false) => format!("({}).iter().cloned().{}()", recv, name),
                }
            }
            "lines" => { need(0)?; format!("({}).lines()", recv) }
            "split" => { need(1)?; format!("LumeSplit::new(({}).split(&*({})))", recv, args[0]) }
            "join" => { need(1)?; format!("({}).join(&*{})", recv, args[0]) }
            "starts_with?" => { need(1)?; format!("({}).starts_with(&*{})", recv, args[0]) }
            "ends_with?" => { need(1)?; format!("({}).ends_with(&*{})", recv, args[0]) }
            "chars" => { need(0)?; format!("({}).chars().map(|c| c.to_string()).collect::<Vec<String>>()", recv) }
            // Option
            "or" => {
                need(1)?;
                if !matches!(rt, Type::Option(_) | Type::Result(..) | Type::Unknown) {
                    return Err(LumeError::new(e.line, e.col, format!("`.or` supplies a default for an optional value or a `T or E`, but this is a `{}`", type_name(rt))));
                }
                format!("({}).unwrap_or({})", recv, args[0])
            }
            "some?" => { need(0)?; format!("({}).is_some()", recv) }
            "none?" => { need(0)?; format!("({}).is_none()", recv) }
            "ok?" => { need(0)?; format!("({}).is_ok()", recv) }
            "error?" => { need(0)?; format!("({}).is_err()", recv) }
            "or_error" => {
                need(1)?;
                if !matches!(rt, Type::Option(_) | Type::Unknown) {
                    return Err(LumeError::new(e.line, e.col, format!("`.or_error` turns an optional value into a `T or Error`, but this is a `{}`", type_name(rt))));
                }
                format!("({}).ok_or_else(|| Error {{ message: ({}).lume_str() }})", recv, args[0])
            }
            "error" => {
                need(0)?;
                if !matches!(rt, Type::Result(..) | Type::Unknown) {
                    return Err(LumeError::new(e.line, e.col, format!("`.error` reads the error of a `T or E`, but this is a `{}`", type_name(rt))));
                }
                format!("({}).clone().err()", recv)
            }
            _ => format!("({}).{}({})", recv, rust_name(name), args.join(", ")),
        })
    }
}


impl Gen {
    fn collect_methods(&self, methods: &[FnDef], owner: &str, fields: &HashSet<String>) -> Result<HashMap<String, Sig>> {
        let mut out = HashMap::new();
        for m in methods {
            if fields.contains(&m.name) {
                return Err(LumeError::new(m.line, m.col, format!("`{}` is both a field and a method of `{}`", m.name, owner)));
            }
            if op_method_name(&m.name).is_some() {
                if m.params.len() != 1 {
                    return Err(LumeError::new(m.line, m.col, format!("`def {}` on `{}` must take exactly one argument, the right-hand side", m.name, owner))
                        .with_help(format!("write `def {}(other: {}) -> {}:`; the left-hand side is `self`", m.name, owner, if matches!(m.name.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") { "Bool" } else { owner })));
                }
                if matches!(m.name.as_str(), "==" | "!=" | "<" | "<=" | ">" | ">=") && m.ret.as_ref().map(|t| *t != Type::Bool).unwrap_or(false) {
                    return Err(LumeError::new(m.line, m.col, format!("`def {}` must return `Bool`", m.name)));
                }
                if m.self_kind == SelfKind::Mutate {
                    return Err(LumeError::new(m.line, m.col, format!("`def {}` cannot take `var self`: operators produce a new value", m.name)));
                }
            }
            if out.insert(m.name.clone(), self.sig_of(m)).is_some() {
                return Err(LumeError::new(m.line, m.col, format!("method `{}` is defined twice in `{}`", m.name, owner)));
            }
        }
        Ok(out)
    }
}

fn is_builtin_name(name: &str) -> bool {
    matches!(
        name,
        "len" | "empty?" | "any?" | "to_str" | "to_s" | "to_float" | "to_int" | "upcase" | "downcase" | "trim"
            | "sqrt" | "abs" | "floor" | "ceil" | "round" | "sum" | "push" | "pop" | "first" | "last"
            | "contains?" | "reverse" | "sort" | "max" | "min" | "lines" | "split" | "join"
            | "starts_with?" | "ends_with?" | "chars" | "take" | "skip" | "to_list" | "enumerate"
            | "or" | "some?" | "none?" | "ok?" | "error?" | "or_error" | "error"
            | "pad" | "keys" | "values" | "remove" | "get"
    )
}

/// Types of the built-in `File` and `Env` namespaces.
fn builtin_namespace_type(ns: &str, name: &str) -> Option<Type> {
    let err = || Box::new(Type::Named("Error".into()));
    Some(match (ns, name) {
        ("File", "read") => Type::Result(Box::new(Type::Str), err()),
        ("File", "write") => Type::Result(Box::new(Type::Unit), err()),
        ("File", "exists?") => Type::Bool,
        ("Env", "args") => Type::List(Box::new(Type::Str)),
        ("Env", "get") => Type::Option(Box::new(Type::Str)),
        ("Time", "now") => Type::Int,
        _ => return None,
    })
}

/// Result type of a built-in method on a value of type `recv`.
fn builtin_method_type(recv: &Type, name: &str) -> Type {
    let elem = match recv {
        Type::List(e) | Type::Iter(e, _) => Some((**e).clone()),
        _ => None,
    };
    if let Type::Map(k, v) = recv {
        return match name {
            "len" => Type::Int,
            "empty?" | "any?" | "contains?" => Type::Bool,
            "keys" => Type::List(k.clone()),
            "values" => Type::List(v.clone()),
            "to_list" => Type::List(Box::new(Type::Tuple(vec![(**k).clone(), (**v).clone()]))),
            "remove" | "get" => Type::Option(v.clone()),
            "to_s" | "to_str" => Type::Str,
            _ => Type::Unknown,
        };
    }
    match name {
        "pad" => Type::Str,
        "take" | "skip" => match recv {
            Type::List(e) => Type::Iter(e.clone(), !e.is_copy()),
            Type::Iter(..) => recv.clone(),
            _ => Type::Unknown,
        },
        "enumerate" => match recv {
            Type::List(e) => Type::Iter(Box::new(Type::Tuple(vec![Type::Int, (**e).clone()])), !e.is_copy()),
            Type::Iter(e, by_ref) => Type::Iter(Box::new(Type::Tuple(vec![Type::Int, (**e).clone()])), *by_ref),
            _ => Type::Unknown,
        },
        "to_list" => recv.materialized(),
        "len" => Type::Int,
        "to_int" => if *recv == Type::Str { Type::Result(Box::new(Type::Int), Box::new(Type::Named("Error".into()))) } else { Type::Int },
        "empty?" | "any?" | "contains?" | "starts_with?" | "ends_with?" | "some?" | "none?" | "ok?" | "error?" => Type::Bool,
        "or_error" => match recv {
            Type::Option(i) => Type::Result(i.clone(), Box::new(Type::Named("Error".into()))),
            _ => Type::Unknown,
        },
        "error" => match recv {
            Type::Result(_, e) => Type::Option(e.clone()),
            _ => Type::Unknown,
        },
        "to_str" | "to_s" | "upcase" | "downcase" | "trim" | "join" => Type::Str,
        "to_float" => if *recv == Type::Str { Type::Result(Box::new(Type::Float), Box::new(Type::Named("Error".into()))) } else { Type::Float },
        "sqrt" | "floor" | "ceil" | "round" => Type::Float,
        "abs" => recv.clone(),
        "sum" => elem.unwrap_or(Type::Unknown),
        "first" | "last" | "max" | "min" | "pop" => elem.map(|e| Type::Option(Box::new(e))).unwrap_or(Type::Unknown),
        "sort" | "reverse" => recv.materialized(),
        "push" => Type::Unit,
        "lines" | "split" => Type::Iter(Box::new(Type::Str), true),
        "chars" => Type::List(Box::new(Type::Str)),
        "or" => match recv {
            Type::Option(i) => (**i).clone(),
            Type::Result(t, _) => (**t).clone(),
            _ => Type::Unknown,
        },
        _ => Type::Unknown,
    }
}

/// The identifier an expression starts with, reading left to right:
/// `x.trim` -> x, `x + 1` -> x, `y + x` -> y, `3 + x` -> None.
fn leftmost_ident(e: &Expr) -> Option<&str> {
    match &e.kind {
        ExprKind::Ident(n) => Some(n.as_str()),
        ExprKind::Method { recv, .. } | ExprKind::TupleIndex { recv, .. } => leftmost_ident(recv),
        ExprKind::Binary { lhs, .. } => leftmost_ident(lhs),
        ExprKind::Range { lo, .. } => leftmost_ident(lo),
        ExprKind::Try(x) => leftmost_ident(x),
        _ => None,
    }
}

fn edit_distance(a: &str, b: &str) -> usize {
    let a: Vec<char> = a.chars().collect();
    let b: Vec<char> = b.chars().collect();
    let mut prev: Vec<usize> = (0..=b.len()).collect();
    for i in 1..=a.len() {
        let mut cur = vec![i; b.len() + 1];
        for j in 1..=b.len() {
            let cost = if a[i - 1] == b[j - 1] { 0 } else { 1 };
            cur[j] = (prev[j] + 1).min(cur[j - 1] + 1).min(prev[j - 1] + cost);
        }
        prev = cur;
    }
    prev[b.len()]
}

/// A type with no unknown parts.
fn type_is_known(t: &Type) -> bool {
    match t {
        Type::Unknown => false,
        Type::List(i) | Type::Option(i) | Type::Iter(i, _) => type_is_known(i),
        Type::Tuple(ts) => ts.iter().all(type_is_known),
        Type::Result(a, b) | Type::Map(a, b) => type_is_known(a) && type_is_known(b),
        _ => true,
    }
}

fn suggest_type(t: &Type) -> String {
    match t {
        Type::List(i) if **i == Type::Unknown => "[Int]".into(),
        Type::Option(i) if **i == Type::Unknown => "Int?".into(),
        Type::Map(k, v) if **k == Type::Unknown || **v == Type::Unknown => "{Str: Int}".into(),
        Type::Unknown => "Type".into(),
        other => type_name(other),
    }
}

fn describe_value(e: &Expr) -> String {
    match &e.kind {
        ExprKind::List(items) if items.is_empty() => "[]".into(),
        ExprKind::MapLit(pairs) if pairs.is_empty() => "{}".into(),
        ExprKind::None => "None".into(),
        _ => "this value".into(),
    }
}

/// A borrowed map key: `&*k` works whether `k` is owned or already a
/// reference (for `String` it yields `&str`, which `Borrow` accepts).
fn map_key(map_ty: &Type, k: &str) -> String {
    match map_ty {
        Type::Map(kt, _) if kt.is_copy() => format!("&({})", k),
        _ => format!("&*({})", k),
    }
}

/// Structural equality for the simple expressions that appear as map
/// receivers and keys (names, fields, literals, tuple indexes).
fn same_expr(a: &Expr, b: &Expr) -> bool {
    match (&a.kind, &b.kind) {
        (ExprKind::Ident(x), ExprKind::Ident(y)) => x == y,
        (ExprKind::SelfRef, ExprKind::SelfRef) => true,
        (ExprKind::Int(x), ExprKind::Int(y)) => x == y,
        (ExprKind::Str(x), ExprKind::Str(y)) => {
            x.len() == y.len() && x.iter().zip(y).all(|(p, q)| matches!((p, q), (StrPiece::Lit(s), StrPiece::Lit(t)) if s == t))
        }
        (ExprKind::Method { recv: r1, name: n1, args: a1 }, ExprKind::Method { recv: r2, name: n2, args: a2 }) => {
            n1 == n2 && a1.is_empty() && a2.is_empty() && same_expr(r1, r2)
        }
        (ExprKind::TupleIndex { recv: r1, index: i1 }, ExprKind::TupleIndex { recv: r2, index: i2 }) => i1 == i2 && same_expr(r1, r2),
        _ => false,
    }
}

fn stmt_mentions(s: &Stmt, name: &str) -> bool {
    let blk = |b: &Block| b.stmts.iter().any(|st| stmt_mentions(st, name));
    match s {
        Stmt::Bind { value, .. } | Stmt::Var { value, .. } | Stmt::OpAssign { value, .. } => expr_mentions(value, name),
        Stmt::FieldAssign { recv, value, .. } => expr_mentions(recv, name) || expr_mentions(value, name),
        Stmt::IndexAssign { recv, index, value, .. } => expr_mentions(recv, name) || expr_mentions(index, name) || expr_mentions(value, name),
        Stmt::Expr(e) => expr_mentions(e, name),
        Stmt::Return { value, .. } => value.as_ref().map(|e| expr_mentions(e, name)).unwrap_or(false),
        Stmt::While { cond, body } => expr_mentions(cond, name) || blk(body),
        Stmt::For { iter, filter, body, .. } => expr_mentions(iter, name) || filter.as_ref().map(|f| expr_mentions(f, name)).unwrap_or(false) || blk(body),
        Stmt::Break { .. } | Stmt::Next { .. } => false,
    }
}

fn expr_mentions(e: &Expr, name: &str) -> bool {
    let blk = |b: &Block| b.stmts.iter().any(|st| stmt_mentions(st, name));
    match &e.kind {
        ExprKind::Ident(n) => n == name,
        ExprKind::Rust(code) => code.contains(name),
        ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) | ExprKind::SelfRef | ExprKind::None | ExprKind::Placeholder => false,
        ExprKind::Str(pieces) => pieces.iter().any(|p| matches!(p, StrPiece::Expr(x) if expr_mentions(x, name))),
        ExprKind::List(items) | ExprKind::Tuple(items) => items.iter().any(|i| expr_mentions(i, name)),
        ExprKind::MapLit(pairs) => pairs.iter().any(|(k, v)| expr_mentions(k, name) || expr_mentions(v, name)),
        ExprKind::Range { lo, hi, .. } => expr_mentions(lo, name) || expr_mentions(hi, name),
        ExprKind::Unary { expr, .. } | ExprKind::Some(expr) | ExprKind::Ok(expr) | ExprKind::Try(expr) | ExprKind::Unwrap(expr) | ExprKind::Puts(expr) => expr_mentions(expr, name),
        ExprKind::TupleIndex { recv, .. } => expr_mentions(recv, name),
        ExprKind::Index { recv, index } => expr_mentions(recv, name) || expr_mentions(index, name),
        ExprKind::Binary { lhs, rhs, .. } => expr_mentions(lhs, name) || expr_mentions(rhs, name),
        ExprKind::Call { args, .. } => args.iter().any(|a| expr_mentions(&a.value, name)),
        ExprKind::Method { recv, args, .. } => expr_mentions(recv, name) || args.iter().any(|a| expr_mentions(&a.value, name)),
        ExprKind::If { branches, else_block } => branches.iter().any(|(c, b)| expr_mentions(c, name) || blk(b)) || else_block.as_ref().map(blk).unwrap_or(false),
        ExprKind::Match { scrutinee, arms } => expr_mentions(scrutinee, name) || arms.iter().any(|a| a.guard.as_ref().map(|g| expr_mentions(g, name)).unwrap_or(false) || blk(&a.body)),
        ExprKind::Lambda { body, .. } => blk(body),
    }
}

/// Rust method name for an operator method (`def +(o)` -> `op_add`).
fn op_method_name(op: &str) -> Option<&'static str> {
    Some(match op {
        "+" => "op_add",
        "-" => "op_sub",
        "*" => "op_mul",
        "/" => "op_div",
        "%" => "op_rem",
        "==" => "op_eq",
        "!=" => "op_ne",
        "<" => "op_lt",
        "<=" => "op_le",
        ">" => "op_gt",
        ">=" => "op_ge",
        _ => return None,
    })
}

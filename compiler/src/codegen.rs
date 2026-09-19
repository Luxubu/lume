//! Code generator: Lume AST -> Rust source.
//!
//! Milestone 2 adds a type table so the generator can tell fields from
//! methods, borrow non-Copy arguments instead of moving them, and infer the
//! return type of functions written without `-> Type`.
//!
//! The checks that must produce *Lume* errors (rather than rustc errors)
//! live here: unknown names, assignment to an immutable binding, changing a
//! field without `var self`, wrong argument counts and keywords, `if` used
//! as a value without `else`.

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
impl LumeLen for String { fn lume_len(&self) -> i64 { self.chars().count() as i64 } }
impl LumeLen for str { fn lume_len(&self) -> i64 { self.chars().count() as i64 } }
impl<T: LumeLen + ?Sized> LumeLen for &T { fn lume_len(&self) -> i64 { (**self).lume_len() } }
trait LumeEmpty { fn lume_empty(&self) -> bool; }
impl<T> LumeEmpty for Vec<T> { fn lume_empty(&self) -> bool { self.is_empty() } }
impl LumeEmpty for String { fn lume_empty(&self) -> bool { self.is_empty() } }
impl LumeEmpty for str { fn lume_empty(&self) -> bool { self.is_empty() } }
impl<T: LumeEmpty + ?Sized> LumeEmpty for &T { fn lume_empty(&self) -> bool { (**self).lume_empty() } }
trait LumeSum<T> { fn lume_sum(&self) -> T; }
impl LumeSum<i64> for Vec<i64> { fn lume_sum(&self) -> i64 { self.iter().sum() } }
impl LumeSum<f64> for Vec<f64> { fn lume_sum(&self) -> f64 { self.iter().sum() } }
impl<T, U: LumeSum<T>> LumeSum<T> for &U { fn lume_sum(&self) -> T { (**self).lume_sum() } }
trait LumeShow { fn lume_str(&self) -> String; }
impl LumeShow for i64 { fn lume_str(&self) -> String { self.to_string() } }
impl LumeShow for f64 { fn lume_str(&self) -> String { format!("{:?}", self) } }
impl LumeShow for bool { fn lume_str(&self) -> String { self.to_string() } }
impl LumeShow for String { fn lume_str(&self) -> String { self.clone() } }
impl LumeShow for str { fn lume_str(&self) -> String { self.to_string() } }
impl LumeShow for () { fn lume_str(&self) -> String { String::from("()") } }
impl<T: LumeShow> LumeShow for Vec<T> {
    fn lume_str(&self) -> String { format!("[{}]", self.iter().map(|x| x.lume_str()).collect::<Vec<_>>().join(", ")) }
}
impl<T: LumeShow + ?Sized> LumeShow for &T { fn lume_str(&self) -> String { (**self).lume_str() } }
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
    ret: Type,
    self_kind: SelfKind,
    line: usize,
    col: usize,
}

#[derive(Clone)]
struct StructInfo {
    fields: Vec<(String, Type)>,
    methods: HashMap<String, Sig>,
    line: usize,
}

pub struct Gen {
    out: String,
    indent: usize,
    scopes: Vec<HashMap<String, Binding>>,
    fns: HashMap<String, Sig>,
    structs: HashMap<String, StructInfo>,
    /// The struct whose method is being generated, if any.
    current_struct: Option<String>,
    current_self: SelfKind,
    current_ret: Type,
    current_fn: String,
    loop_depth: usize,
}

pub fn generate(program: &[Item]) -> Result<String> {
    let mut g = Gen {
        out: String::new(),
        indent: 0,
        scopes: Vec::new(),
        fns: HashMap::new(),
        structs: HashMap::new(),
        current_struct: None,
        current_self: SelfKind::Read,
        current_ret: Type::Unit,
        current_fn: String::new(),
        loop_depth: 0,
    };
    g.program(program)?;
    Ok(g.out)
}

fn rust_name(name: &str) -> String {
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

impl Gen {
    // ----- output helpers -------------------------------------------------

    fn line(&mut self, s: &str) {
        for _ in 0..self.indent {
            self.out.push_str("    ");
        }
        self.out.push_str(s);
        self.out.push('\n');
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
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), Binding { mutable, borrowed, ty, line });
    }

    fn field_type(&self, name: &str) -> Option<Type> {
        let s = self.current_struct.as_ref()?;
        let info = self.structs.get(s)?;
        info.fields.iter().find(|(n, _)| n == name).map(|(_, t)| t.clone())
    }

    fn all_names(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for s in &self.scopes {
            v.extend(s.keys().cloned());
        }
        v.extend(self.fns.keys().cloned());
        v.extend(self.structs.keys().cloned());
        if let Some(s) = &self.current_struct {
            if let Some(info) = self.structs.get(s) {
                v.extend(info.fields.iter().map(|(n, _)| n.clone()));
                v.extend(info.methods.keys().cloned());
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

    // ----- program ----------------------------------------------------------

    fn program(&mut self, program: &[Item]) -> Result<()> {
        // Pass 1: collect struct fields and every signature.
        let mut seen = HashSet::new();
        for item in program {
            match item {
                Item::Fn(f) => {
                    if !seen.insert(f.name.clone()) {
                        return Err(LumeError::new(f.line, f.col, format!("function `{}` is defined twice", f.name)));
                    }
                    self.fns.insert(f.name.clone(), sig_of(f));
                }
                Item::Struct(s) => {
                    if !seen.insert(s.name.clone()) {
                        return Err(LumeError::new(s.line, s.col, format!("`{}` is defined twice", s.name)));
                    }
                    let mut fnames = HashSet::new();
                    for fld in &s.fields {
                        if !fnames.insert(fld.name.clone()) {
                            return Err(LumeError::new(fld.line, fld.col, format!("field `{}` is listed twice in `{}`", fld.name, s.name)));
                        }
                    }
                    let mut methods = HashMap::new();
                    for m in &s.methods {
                        if fnames.contains(&m.name) {
                            return Err(LumeError::new(m.line, m.col, format!("`{}` is both a field and a method of `{}`", m.name, s.name)));
                        }
                        if methods.insert(m.name.clone(), sig_of(m)).is_some() {
                            return Err(LumeError::new(m.line, m.col, format!("method `{}` is defined twice in `{}`", m.name, s.name)));
                        }
                    }
                    self.structs.insert(
                        s.name.clone(),
                        StructInfo {
                            fields: s.fields.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
                            methods,
                            line: s.line,
                        },
                    );
                }
            }
        }
        // Check that every named type exists.
        for item in program {
            match item {
                Item::Fn(f) => self.check_sig_types(f)?,
                Item::Struct(s) => {
                    for fld in &s.fields {
                        self.check_type(&fld.ty, fld.line, fld.col)?;
                    }
                    for m in &s.methods {
                        self.check_sig_types(m)?;
                    }
                }
            }
        }
        // Pass 2: infer missing return types (a few rounds so dependencies settle).
        for _round in 0..4 {
            let mut progressed = false;
            for item in program {
                match item {
                    Item::Fn(f) => {
                        if f.ret.is_none() && self.fns[&f.name].ret == Type::Unknown {
                            let t = self.infer_ret(f, None);
                            if t != Type::Unknown {
                                self.fns.get_mut(&f.name).unwrap().ret = t;
                                progressed = true;
                            }
                        }
                    }
                    Item::Struct(s) => {
                        for m in &s.methods {
                            if m.ret.is_none() && self.structs[&s.name].methods[&m.name].ret == Type::Unknown {
                                let t = self.infer_ret(m, Some(&s.name));
                                if t != Type::Unknown {
                                    self.structs.get_mut(&s.name).unwrap().methods.get_mut(&m.name).unwrap().ret = t;
                                    progressed = true;
                                }
                            }
                        }
                    }
                }
            }
            if !progressed {
                break;
            }
        }
        for item in program {
            let (f, owner) = match item {
                Item::Fn(f) => (vec![f], None),
                Item::Struct(s) => (s.methods.iter().collect(), Some(&s.name)),
            };
            for f in f {
                let ret = match owner {
                    None => self.fns[&f.name].ret.clone(),
                    Some(s) => self.structs[s].methods[&f.name].ret.clone(),
                };
                if ret == Type::Unknown {
                    return Err(LumeError::new(f.line, f.col, format!("cannot work out what `{}` returns", f.name))
                        .with_help(format!("add the return type to the signature: `def {}(...) -> Type`", f.name)));
                }
            }
        }

        // Pass 3: emit.
        self.out.push_str(PRELUDE);
        self.out.push('\n');
        for item in program {
            match item {
                Item::Fn(f) => self.fn_def(f, None)?,
                Item::Struct(s) => self.struct_def(s)?,
            }
            self.out.push('\n');
        }
        if !self.fns.contains_key("main") {
            return Err(LumeError::new(1, 1, "no `main` function").with_help("a program starts at `def main:`"));
        }
        Ok(())
    }

    fn check_type(&self, t: &Type, line: usize, col: usize) -> Result<()> {
        match t {
            Type::Named(n) if !self.structs.contains_key(n) => {
                let e = LumeError::new(line, col, format!("unknown type `{}`", n));
                Err(match self.suggest_from(n, self.structs.keys().cloned().chain(["Int", "Float", "Bool", "Str"].iter().map(|s| s.to_string()))) {
                    Some(s) => e.with_help(format!("did you mean `{}`?", s)),
                    None => e.with_help("built-in types are Int, Float, Bool, Str and [T]; others must be a `struct`"),
                })
            }
            Type::List(inner) => self.check_type(inner, line, col),
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
        let saved_struct = self.current_struct.clone();
        self.current_struct = owner.cloned();
        for p in &f.params {
            self.declare(&p.name, false, !p.ty.is_copy(), p.ty.clone(), p.line);
        }
        let t = self.tail_type(&f.body);
        self.current_struct = saved_struct;
        self.pop_scope();
        t
    }

    fn tail_type(&mut self, b: &Block) -> Type {
        // Bindings inside the body matter for the tail's type, so walk them.
        self.push_scope();
        let mut t = Type::Unit;
        let n = b.stmts.len();
        for (i, s) in b.stmts.iter().enumerate() {
            let last = i + 1 == n;
            match s {
                Stmt::Bind { name, value, line, .. } | Stmt::Var { name, value, line, .. } => {
                    let vt = self.ty_of(value);
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
                        _ => self.ty_of(e),
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

    // ----- type inference ---------------------------------------------------

    fn ty_of(&self, e: &Expr) -> Type {
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
                } else {
                    Type::Unknown
                }
            }
            ExprKind::SelfRef => self.current_struct.clone().map(Type::Named).unwrap_or(Type::Unknown),
            ExprKind::List(items) => {
                let mut t = Type::Unknown;
                for i in items {
                    let it = self.ty_of(i);
                    if it != Type::Unknown {
                        t = it;
                        break;
                    }
                }
                Type::List(Box::new(t))
            }
            ExprKind::Range { .. } => Type::List(Box::new(Type::Int)),
            ExprKind::Unary { op, expr } => match *op {
                "not" => Type::Bool,
                _ => self.ty_of(expr),
            },
            ExprKind::Binary { op, lhs, rhs } => match *op {
                "==" | "!=" | "<" | "<=" | ">" | ">=" | "and" | "or" => Type::Bool,
                _ => {
                    let l = self.ty_of(lhs);
                    if l == Type::Unknown { self.ty_of(rhs) } else { l }
                }
            },
            ExprKind::Call { name, .. } => {
                if let Some(s) = self.fns.get(name) {
                    s.ret.clone()
                } else if self.structs.contains_key(name) {
                    Type::Named(name.clone())
                } else {
                    Type::Unknown
                }
            }
            ExprKind::Method { recv, name, .. } => {
                let rt = self.ty_of(recv);
                if let Type::Named(sn) = &rt {
                    if let Some(info) = self.structs.get(sn) {
                        if let Some((_, ft)) = info.fields.iter().find(|(n, _)| n == name) {
                            return ft.clone();
                        }
                        if let Some(m) = info.methods.get(name) {
                            return m.ret.clone();
                        }
                    }
                }
                builtin_method_type(&rt, name)
            }
            ExprKind::If { branches, else_block } => {
                if else_block.is_none() {
                    return Type::Unit;
                }
                // Type of the first branch's tail expression, without bindings.
                for (_, b) in branches {
                    if let Some(Stmt::Expr(last)) = b.stmts.last() {
                        let t = self.ty_of(last);
                        if t != Type::Unknown {
                            return t;
                        }
                    }
                }
                Type::Unknown
            }
            ExprKind::Puts(_) => Type::Unit,
        }
    }

    // ----- items ------------------------------------------------------------

    fn struct_def(&mut self, s: &StructDef) -> Result<()> {
        self.line("#[derive(Debug, Clone, PartialEq)]");
        self.line(&format!("struct {} {{", s.name));
        self.indent += 1;
        for f in &s.fields {
            self.line(&format!("{}: {},", rust_name(&f.name), rust_type(&f.ty)));
        }
        self.indent -= 1;
        self.line("}");
        // Printing: `Point(x: 1.0, y: 2.0)`
        self.line(&format!("impl LumeShow for {} {{", s.name));
        self.indent += 1;
        let fmt: Vec<String> = s.fields.iter().map(|f| format!("{}: {{}}", f.name)).collect();
        let args: Vec<String> = s.fields.iter().map(|f| format!("self.{}.lume_str()", rust_name(&f.name))).collect();
        self.line(&format!(
            "fn lume_str(&self) -> String {{ format!(\"{}({})\", {}) }}",
            s.name,
            fmt.join(", "),
            args.join(", ")
        ));
        self.indent -= 1;
        self.line("}");
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

    fn fn_def(&mut self, f: &FnDef, owner: Option<&String>) -> Result<()> {
        let sig = match owner {
            None => self.fns[&f.name].clone(),
            Some(s) => self.structs[s].methods[&f.name].clone(),
        };
        if f.name == "main" && owner.is_none() {
            if !f.params.is_empty() || matches!(f.ret, Some(ref r) if *r != Type::Unit) {
                return Err(LumeError::new(f.line, f.col, "`main` takes no parameters and returns nothing")
                    .with_help("write `def main:`"));
            }
        }
        let mut parts: Vec<String> = Vec::new();
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
            let rt = rust_type(&p.ty);
            let rt = if p.ty.is_copy() { rt } else { format!("&{}", rt) };
            parts.push(format!("{}: {}", rust_name(&p.name), rt));
        }
        let mut header = format!("fn {}({})", rust_name(&f.name), parts.join(", "));
        if sig.ret != Type::Unit {
            header.push_str(&format!(" -> {}", rust_type(&sig.ret)));
        }
        header.push_str(" {");
        self.line(&header);
        self.indent += 1;
        self.push_scope();
        for p in &f.params {
            self.declare(&p.name, false, !p.ty.is_copy(), p.ty.clone(), p.line);
        }
        self.current_ret = sig.ret.clone();
        self.current_struct = owner.cloned();
        self.current_self = f.self_kind;
        self.current_fn = f.name.clone();
        let want_value = sig.ret != Type::Unit;
        self.block_body(&f.body, want_value)?;
        self.pop_scope();
        self.current_struct = None;
        self.indent -= 1;
        self.line("}");
        Ok(())
    }

    fn block_body(&mut self, b: &Block, want_value: bool) -> Result<()> {
        let n = b.stmts.len();
        if want_value && n == 0 {
            return Err(LumeError::new(1, 1, "a block that produces a value cannot be empty"));
        }
        for (i, s) in b.stmts.iter().enumerate() {
            let last = i + 1 == n;
            self.stmt(s, want_value && last)?;
        }
        Ok(())
    }

    fn nested_block(&mut self, b: &Block, want_value: bool) -> Result<()> {
        self.indent += 1;
        self.push_scope();
        self.block_body(b, want_value)?;
        self.pop_scope();
        self.indent -= 1;
        Ok(())
    }

    // ----- statements -------------------------------------------------------

    fn stmt(&mut self, s: &Stmt, is_tail: bool) -> Result<()> {
        match s {
            Stmt::Var { name, value, line, col } => {
                let vt = self.ty_of(value);
                let v = self.expr_owned(value)?;
                if self.scopes.last().unwrap().contains_key(name) {
                    return Err(LumeError::new(*line, *col, format!("`{}` is already declared in this block", name))
                        .with_help(format!("to change it, write `{} = ...`", name)));
                }
                self.declare(name, true, false, vt, *line);
                self.line(&format!("let mut {} = {};", rust_name(name), v));
                if is_tail {
                    return self.tail_unit(*line, *col);
                }
            }
            Stmt::Bind { name, value, line, col } => {
                let vt = self.ty_of(value);
                match self.lookup(name).cloned() {
                    Some(b) if b.mutable => {
                        let v = self.expr_owned(value)?;
                        self.line(&format!("{} = {};", rust_name(name), v));
                    }
                    Some(b) => {
                        if leftmost_ident(value) == Some(name.as_str()) {
                            // self-transform shadowing: x = x.trim
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
                        if let Some(_ft) = self.field_type(name) {
                            // assignment to a field of self
                            self.require_var_self(name, *line, *col)?;
                            let v = self.expr_owned(value)?;
                            self.line(&format!("self.{} = {};", rust_name(name), v));
                        } else {
                            let v = self.expr_owned(value)?;
                            self.declare(name, false, false, vt, *line);
                            self.line(&format!("let {} = {};", rust_name(name), v));
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
            Stmt::FieldAssign { recv, field, op, value, line, col } => {
                let rt = self.ty_of(recv);
                let sname = match &rt {
                    Type::Named(s) => s.clone(),
                    Type::Unknown => return Err(LumeError::new(*line, *col, format!("cannot tell what `.{}` belongs to here", field))),
                    other => {
                        return Err(LumeError::new(*line, *col, format!("`{}` values have no fields to assign", type_name(other))))
                    }
                };
                let info = self.structs[&sname].clone();
                if !info.fields.iter().any(|(n, _)| n == field) {
                    return Err(self.no_such_member(&sname, field, *line, *col));
                }
                // Is the receiver mutable?
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
                        return Err(LumeError::new(*line, *col, "only a named value's field can be assigned")
                            .with_help("bind the value to a `var` first"));
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
                    let v = self.expr_owned(e)?;
                    self.line(&v);
                } else {
                    let v = self.expr_stmt(e)?;
                    self.line(&format!("{};", v));
                }
            }
            Stmt::Return { value, line, col } => match value {
                Some(e) => {
                    if self.current_ret == Type::Unit {
                        return Err(LumeError::new(*line, *col, "this function returns nothing, but `return` has a value")
                            .with_help("add `-> Type` to the function signature"));
                    }
                    let v = self.expr_owned(e)?;
                    self.line(&format!("return {};", v));
                }
                None => {
                    if self.current_ret != Type::Unit {
                        return Err(LumeError::new(*line, *col, format!(
                            "this function returns `{}`, so `return` needs a value",
                            type_name(&self.current_ret)
                        )));
                    }
                    self.line("return;");
                }
            },
            Stmt::While { cond, body } => {
                let c = self.expr(cond)?;
                self.line(&format!("while {} {{", c));
                self.loop_depth += 1;
                self.nested_block(body, false)?;
                self.loop_depth -= 1;
                self.line("}");
            }
            Stmt::For { var, iter, filter, body, line, .. } => {
                let it_ty = self.ty_of(iter);
                let (it, elem_ty, borrowed) = match (&iter.kind, &it_ty) {
                    (ExprKind::Range { .. }, _) => (self.expr(iter)?, Type::Int, false),
                    (_, Type::List(elem)) if elem.is_copy() || **elem == Type::Unknown => {
                        (format!("({}).iter().cloned()", self.expr(iter)?), (**elem).clone(), false)
                    }
                    (_, Type::List(elem)) => (format!("({}).iter()", self.expr(iter)?), (**elem).clone(), true),
                    (_, Type::Unknown) => (format!("({}).iter().cloned()", self.expr(iter)?), Type::Unknown, false),
                    (_, other) => {
                        return Err(LumeError::new(iter.line, iter.col, format!("cannot loop over a `{}`", type_name(other)))
                            .with_help("`for` needs a list or a range"));
                    }
                };
                self.line(&format!("for {} in {} {{", rust_name(var), it));
                self.indent += 1;
                self.push_scope();
                self.declare(var, false, borrowed, elem_ty, *line);
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
                if self.loop_depth == 0 {
                    return Err(LumeError::new(*line, *col, "`break` outside of a loop"));
                }
                self.line("break;");
            }
            Stmt::Next { line, col } => {
                if self.loop_depth == 0 {
                    return Err(LumeError::new(*line, *col, "`next` outside of a loop"));
                }
                self.line("continue;");
            }
        }
        Ok(())
    }

    fn require_var_self(&self, field: &str, line: usize, col: usize) -> Result<()> {
        if self.current_self != SelfKind::Mutate {
            return Err(LumeError::new(line, col, format!("`{}` changes the field `{}`, but its `self` is read-only", self.current_fn, field))
                .with_help(format!("write `def {}(var self, ...)` to allow it", self.current_fn)));
        }
        Ok(())
    }

    fn tail_unit(&mut self, line: usize, col: usize) -> Result<()> {
        Err(LumeError::new(line, col, format!(
            "the last line of a function returning `{}` must be a value, not a binding",
            type_name(&self.current_ret)
        ))
        .with_help("put the value on its own line after the binding"))
    }

    fn expr_stmt(&mut self, e: &Expr) -> Result<String> {
        if let ExprKind::If { branches, else_block } = &e.kind {
            return self.if_chain(branches, else_block.as_ref(), false);
        }
        self.expr(e)
    }

    // ----- expressions ------------------------------------------------------

    fn unknown_name(&self, name: &str, line: usize, col: usize) -> LumeError {
        let e = LumeError::new(line, col, format!("unknown name `{}`", name));
        match self.suggest(name) {
            Some(s) => e.with_help(format!("did you mean `{}`?", s)),
            None => e.with_help("names must be bound with `name = value` or `var name = value` before use"),
        }
    }

    fn no_such_member(&self, sname: &str, name: &str, line: usize, col: usize) -> LumeError {
        let info = &self.structs[sname];
        let e = LumeError::new(line, col, format!("`{}` has no field or method named `{}`", sname, name));
        let cands = info.fields.iter().map(|(n, _)| n.clone()).chain(info.methods.keys().cloned());
        match self.suggest_from(name, cands) {
            Some(s) => e.with_help(format!("did you mean `{}`?", s)),
            None => {
                let fields: Vec<String> = info.fields.iter().map(|(n, _)| n.clone()).collect();
                e.with_help(format!("`{}` has fields {}", sname, fields.join(", ")))
            }
        }
    }

    /// True when the expression names a place we cannot move out of: a
    /// borrowed parameter, `self`, or a field reached through either.
    fn is_borrowed_place(&self, e: &Expr) -> bool {
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
            _ => false,
        }
    }

    /// An expression in a position that needs an owned value (binding,
    /// return, constructor argument, list item). Places that are borrowed
    /// get cloned; owned locals move.
    fn expr_owned(&mut self, e: &Expr) -> Result<String> {
        let s = self.expr(e)?;
        let t = self.ty_of(e);
        if !t.is_copy() && self.is_borrowed_place(e) {
            Ok(format!("{}.clone()", s))
        } else {
            Ok(s)
        }
    }

    /// An argument for a parameter of type `t`: Copy types by value,
    /// everything else by reference.
    fn expr_arg(&mut self, e: &Expr, t: &Type) -> Result<String> {
        let s = self.expr(e)?;
        if t.is_copy() {
            return Ok(s);
        }
        let already_ref = match &e.kind {
            ExprKind::Ident(n) => self.lookup(n).map(|b| b.borrowed).unwrap_or(false),
            ExprKind::SelfRef => true,
            _ => false,
        };
        if already_ref {
            Ok(s)
        } else {
            Ok(format!("&{}", s))
        }
    }

    fn if_chain(&mut self, branches: &[(Expr, Block)], else_block: Option<&Block>, want_value: bool) -> Result<String> {
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

    /// Reorders positional + keyword arguments to match a parameter list.
    fn bind_args<'a>(&self, what: &str, params: &[(String, Type)], args: &'a [Arg], line: usize, col: usize) -> Result<Vec<&'a Expr>> {
        let mut slots: Vec<Option<&Expr>> = vec![None; params.len()];
        let mut pos = 0;
        for a in args {
            match &a.name {
                None => {
                    if pos >= params.len() {
                        return Err(LumeError::new(line, col, format!(
                            "{} takes {} argument{}, but {} {} given",
                            what,
                            params.len(),
                            if params.len() == 1 { "" } else { "s" },
                            args.len(),
                            if args.len() == 1 { "was" } else { "were" }
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
        let missing: Vec<String> = params
            .iter()
            .zip(&slots)
            .filter(|(_, s)| s.is_none())
            .map(|((n, _), _)| n.clone())
            .collect();
        if !missing.is_empty() {
            return Err(LumeError::new(line, col, format!("{} is missing {}: {}", what, if missing.len() == 1 { "an argument" } else { "arguments" }, missing.join(", "))));
        }
        Ok(slots.into_iter().map(|s| s.unwrap()).collect())
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
                                args.push(format!("({}).lume_str()", self.expr(x)?));
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
                } else if self.fns.contains_key(name) {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is a function; call it with `{}()`", name, name)));
                } else if self.structs.contains_key(name) {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is a type; construct one with `{}(...)`", name, name)));
                } else {
                    return Err(self.unknown_name(name, e.line, e.col));
                }
            }
            ExprKind::SelfRef => {
                if self.current_struct.is_none() {
                    return Err(LumeError::new(e.line, e.col, "`self` is only meaningful inside a struct method"));
                }
                "self".into()
            }
            ExprKind::List(items) => {
                let parts: Result<Vec<String>> = items.iter().map(|i| self.expr_owned(i)).collect();
                format!("vec![{}]", parts?.join(", "))
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
                let mut l = self.expr(lhs)?;
                let mut r = self.expr(rhs)?;
                let lt = self.ty_of(lhs);
                // Borrowed parameters compared or combined with owned values
                // need a deref so the types line up on both sides.
                let lb = self.is_borrowed_ident(lhs);
                let rb = self.is_borrowed_ident(rhs);
                if lb && !rb && !lt.is_copy() {
                    l = format!("(*{})", l);
                }
                if rb && !lb && !lt.is_copy() {
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
            ExprKind::Call { name, args } => {
                if let Some(sig) = self.fns.get(name).cloned() {
                    let bound = self.bind_args(&format!("`{}`", name), &sig.params, args, e.line, e.col)?;
                    let mut parts = Vec::new();
                    for (a, (_, t)) in bound.iter().zip(&sig.params) {
                        parts.push(self.expr_arg(a, t)?);
                    }
                    format!("{}({})", rust_name(name), parts.join(", "))
                } else if let Some(info) = self.structs.get(name).cloned() {
                    let bound = self.bind_args(&format!("`{}`", name), &info.fields, args, e.line, e.col)?;
                    let mut parts = Vec::new();
                    for (a, (fname, _)) in bound.iter().zip(&info.fields) {
                        parts.push(format!("{}: {}", rust_name(fname), self.expr_owned(a)?));
                    }
                    format!("{} {{ {} }}", name, parts.join(", "))
                } else if self.lookup(name).is_some() {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` is a value, not a function", name)));
                } else if let Some(sn) = self.current_struct.clone() {
                    if self.structs[&sn].methods.contains_key(name) {
                        return Err(LumeError::new(e.line, e.col, format!("`{}` is a method of `{}`; call it as `self.{}(...)`", name, sn, name)));
                    }
                    return Err(self.unknown_fn(name, e.line, e.col));
                } else {
                    return Err(self.unknown_fn(name, e.line, e.col));
                }
            }
            ExprKind::Method { recv, name, args } => {
                let rt = self.ty_of(recv);
                let r = self.expr(recv)?;
                if let Type::Named(sname) = &rt {
                    let info = self.structs[sname].clone();
                    if let Some((_, _ft)) = info.fields.iter().find(|(n, _)| n == name) {
                        if !args.is_empty() {
                            return Err(LumeError::new(e.line, e.col, format!("`{}` is a field of `{}`, not a method; it takes no arguments", name, sname)));
                        }
                        return Ok(format!("{}.{}", r, rust_name(name)));
                    }
                    if let Some(m) = info.methods.get(name) {
                        let bound = self.bind_args(&format!("`{}.{}`", sname, name), &m.params, args, e.line, e.col)?;
                        let mut parts = Vec::new();
                        for (a, (_, t)) in bound.iter().zip(&m.params) {
                            parts.push(self.expr_arg(a, t)?);
                        }
                        if m.self_kind == SelfKind::Mutate {
                            self.check_receiver_mutable(recv, name, e.line, e.col)?;
                        }
                        return Ok(format!("{}.{}({})", r, rust_name(name), parts.join(", ")));
                    }
                    return Err(self.no_such_member(sname, name, e.line, e.col));
                }
                if let Type::Named(_) | Type::Unknown = rt {
                } else if builtin_method_type(&rt, name) == Type::Unknown && !is_builtin_name(name) {
                    return Err(LumeError::new(e.line, e.col, format!("`{}` values have no method `{}`", type_name(&rt), name)));
                }
                if args.iter().any(|a| a.name.is_some()) {
                    return Err(LumeError::new(e.line, e.col, format!("built-in method `{}` does not take keyword arguments", name)));
                }
                let mut parts = Vec::new();
                for a in args {
                    parts.push(self.expr(&a.value)?);
                }
                self.method(&r, name, &parts, e)?
            }
            ExprKind::If { branches, else_block } => {
                if else_block.is_none() {
                    return Err(LumeError::new(e.line, e.col, "an `if` used as a value needs an `else`")
                        .with_help("every branch must produce the value"));
                }
                self.if_chain(branches, else_block.as_ref(), true)?
            }
            ExprKind::Puts(arg) => {
                let a = self.expr(arg)?;
                format!("println!(\"{{}}\", ({}).lume_str())", a)
            }
        })
    }

    fn is_borrowed_ident(&self, e: &Expr) -> bool {
        match &e.kind {
            ExprKind::Ident(n) => self.lookup(n).map(|b| b.borrowed).unwrap_or(false),
            ExprKind::SelfRef => true,
            _ => false,
        }
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

    /// Maps built-in method names onto Rust.
    fn method(&mut self, recv: &str, name: &str, args: &[String], e: &Expr) -> Result<String> {
        let need = |n: usize| -> Result<()> {
            if args.len() != n {
                Err(LumeError::new(e.line, e.col, format!(
                    "`.{}` takes {} argument{}, but {} {} given",
                    name, n, if n == 1 { "" } else { "s" }, args.len(), if args.len() == 1 { "was" } else { "were" }
                )))
            } else {
                Ok(())
            }
        };
        Ok(match name {
            "len" => { need(0)?; format!("({}).lume_len()", recv) }
            "empty?" => { need(0)?; format!("({}).lume_empty()", recv) }
            "any?" => { need(0)?; format!("(!({}).lume_empty())", recv) }
            "to_str" | "to_s" => { need(0)?; format!("({}).lume_str()", recv) }
            "to_float" => { need(0)?; format!("(({}) as f64)", recv) }
            "to_int" => { need(0)?; format!("(({}) as i64)", recv) }
            "upcase" => { need(0)?; format!("({}).to_uppercase()", recv) }
            "downcase" => { need(0)?; format!("({}).to_lowercase()", recv) }
            "trim" => { need(0)?; format!("({}).trim().to_string()", recv) }
            "sqrt" | "abs" | "floor" | "ceil" | "round" => { need(0)?; format!("({}).{}()", recv, name) }
            "sum" => { need(0)?; format!("({}).lume_sum()", recv) }
            "push" => { need(1)?; format!("({}).push({})", recv, args[0]) }
            "pop" => { need(0)?; format!("({}).pop().unwrap()", recv) }
            "first" => { need(0)?; format!("({}).first().cloned().unwrap()", recv) }
            "last" => { need(0)?; format!("({}).last().cloned().unwrap()", recv) }
            "contains?" => { need(1)?; format!("({}).contains(&{})", recv, args[0]) }
            "reverse" => { need(0)?; format!("{{ let mut v = ({}).clone(); v.reverse(); v }}", recv) }
            "sort" => { need(0)?; format!("{{ let mut v = ({}).clone(); v.sort(); v }}", recv) }
            "max" => { need(0)?; format!("({}).iter().cloned().max().unwrap()", recv) }
            "min" => { need(0)?; format!("({}).iter().cloned().min().unwrap()", recv) }
            "lines" => { need(0)?; format!("({}).lines().map(|s| s.to_string()).collect::<Vec<String>>()", recv) }
            "split" => { need(1)?; format!("({}).split(&*{}).map(|s| s.to_string()).collect::<Vec<String>>()", recv, args[0]) }
            "join" => { need(1)?; format!("({}).join(&*{})", recv, args[0]) }
            "starts_with?" => { need(1)?; format!("({}).starts_with(&*{})", recv, args[0]) }
            "ends_with?" => { need(1)?; format!("({}).ends_with(&*{})", recv, args[0]) }
            "chars" => { need(0)?; format!("({}).chars().map(|c| c.to_string()).collect::<Vec<String>>()", recv) }
            _ => format!("({}).{}({})", recv, rust_name(name), args.join(", ")),
        })
    }
}

fn sig_of(f: &FnDef) -> Sig {
    Sig {
        params: f.params.iter().map(|p| (p.name.clone(), p.ty.clone())).collect(),
        ret: f.ret.clone().unwrap_or(Type::Unknown),
        self_kind: f.self_kind,
        line: f.line,
        col: f.col,
    }
}

fn is_builtin_name(name: &str) -> bool {
    matches!(
        name,
        "len" | "empty?" | "any?" | "to_str" | "to_s" | "to_float" | "to_int" | "upcase" | "downcase" | "trim"
            | "sqrt" | "abs" | "floor" | "ceil" | "round" | "sum" | "push" | "pop" | "first" | "last"
            | "contains?" | "reverse" | "sort" | "max" | "min" | "lines" | "split" | "join"
            | "starts_with?" | "ends_with?" | "chars"
    )
}

/// Result type of a built-in method on a value of type `recv`.
fn builtin_method_type(recv: &Type, name: &str) -> Type {
    let elem = match recv {
        Type::List(e) => Some((**e).clone()),
        _ => None,
    };
    match name {
        "len" | "to_int" => Type::Int,
        "empty?" | "any?" | "contains?" | "starts_with?" | "ends_with?" => Type::Bool,
        "to_str" | "to_s" | "upcase" | "downcase" | "trim" | "join" => Type::Str,
        "to_float" | "sqrt" | "floor" | "ceil" | "round" => Type::Float,
        "abs" => recv.clone(),
        "sum" | "first" | "last" | "max" | "min" | "pop" => elem.unwrap_or(Type::Unknown),
        "sort" | "reverse" => recv.clone(),
        "push" => Type::Unit,
        "lines" | "split" | "chars" => Type::List(Box::new(Type::Str)),
        _ => Type::Unknown,
    }
}

/// The identifier an expression starts with, reading left to right:
/// `x.trim` -> x, `x + 1` -> x, `y + x` -> y, `3 + x` -> None.
fn leftmost_ident(e: &Expr) -> Option<&str> {
    match &e.kind {
        ExprKind::Ident(n) => Some(n.as_str()),
        ExprKind::Method { recv, .. } => leftmost_ident(recv),
        ExprKind::Binary { lhs, .. } => leftmost_ident(lhs),
        ExprKind::Range { lo, .. } => leftmost_ident(lo),
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

//! Code generator: Lume AST -> Rust source.
//!
//! Also performs the checks that must produce *Lume* errors rather than
//! rustc errors: unknown names, assignment to an immutable binding, calling a
//! function that does not exist, `if` used as a value without `else`.

use std::collections::{HashMap, HashSet};

use crate::ast::*;
use crate::error::{LumeError, Result};

pub const PRELUDE: &str = r#"#![allow(unused, non_snake_case, non_camel_case_types, unused_parens, unused_mut)]
// ---- Lume prelude ----
trait LumePow { fn lume_pow(self, e: Self) -> Self; }
impl LumePow for i64 { fn lume_pow(self, e: Self) -> Self { self.pow(e as u32) } }
impl LumePow for f64 { fn lume_pow(self, e: Self) -> Self { self.powf(e) } }
trait LumeLen { fn lume_len(&self) -> i64; }
impl<T> LumeLen for Vec<T> { fn lume_len(&self) -> i64 { self.len() as i64 } }
impl LumeLen for String { fn lume_len(&self) -> i64 { self.chars().count() as i64 } }
impl LumeLen for &str { fn lume_len(&self) -> i64 { self.chars().count() as i64 } }
trait LumeEmpty { fn lume_empty(&self) -> bool; }
impl<T> LumeEmpty for Vec<T> { fn lume_empty(&self) -> bool { self.is_empty() } }
impl LumeEmpty for String { fn lume_empty(&self) -> bool { self.is_empty() } }
trait LumeSum<T> { fn lume_sum(&self) -> T; }
impl LumeSum<i64> for Vec<i64> { fn lume_sum(&self) -> i64 { self.iter().sum() } }
impl LumeSum<f64> for Vec<f64> { fn lume_sum(&self) -> f64 { self.iter().sum() } }
// ---- end prelude ----
"#;

const RUST_RESERVED: &[&str] = &[
    "as", "async", "await", "box", "crate", "dyn", "extern", "fn", "impl", "let", "loop", "macro",
    "mod", "move", "mut", "ref", "static", "super", "trait", "type", "union", "unsafe", "use",
    "yield", "try", "Self", "self", "abstract", "become", "final", "override", "priv", "typeof",
    "unsized", "virtual", "where", "continue", "main_",
];

#[derive(Clone)]
struct Binding {
    mutable: bool,
    line: usize,
}

pub struct Gen {
    out: String,
    indent: usize,
    scopes: Vec<HashMap<String, Binding>>,
    fns: HashMap<String, (usize, Type)>, // name -> (arity, return type)
    current_ret: Type,
    loop_depth: usize,
}

pub fn generate(program: &[FnDef]) -> Result<String> {
    let mut g = Gen {
        out: String::new(),
        indent: 0,
        scopes: Vec::new(),
        fns: HashMap::new(),
        current_ret: Type::Unit,
        loop_depth: 0,
    };
    g.program(program)?;
    Ok(g.out)
}

fn rust_name(name: &str) -> String {
    // `adult?` -> `adult_q`, keywords -> raw identifiers
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
    }
}

fn escape_rust_str(s: &str) -> String {
    let mut out = String::new();
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\t' => out.push_str("\\t"),
            '\0' => out.push_str("\\0"),
            '{' => out.push_str("{{"),
            '}' => out.push_str("}}"),
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

    fn declare(&mut self, name: &str, mutable: bool, line: usize) {
        self.scopes
            .last_mut()
            .unwrap()
            .insert(name.to_string(), Binding { mutable, line });
    }

    fn all_names(&self) -> Vec<String> {
        let mut v: Vec<String> = Vec::new();
        for s in &self.scopes {
            v.extend(s.keys().cloned());
        }
        v.extend(self.fns.keys().cloned());
        v
    }

    fn suggest(&self, name: &str) -> Option<String> {
        let mut best: Option<(usize, String)> = None;
        for cand in self.all_names() {
            let d = edit_distance(name, &cand);
            if d <= 2 && d < name.len() {
                if best.as_ref().map(|b| d < b.0).unwrap_or(true) {
                    best = Some((d, cand));
                }
            }
        }
        best.map(|b| b.1)
    }

    // ----- program ----------------------------------------------------------

    fn program(&mut self, program: &[FnDef]) -> Result<()> {
        let mut seen = HashSet::new();
        for f in program {
            if !seen.insert(f.name.clone()) {
                return Err(LumeError::new(f.line, f.col, format!("function `{}` is defined twice", f.name)));
            }
            self.fns.insert(f.name.clone(), (f.params.len(), f.ret.clone()));
        }
        self.out.push_str(PRELUDE);
        self.out.push('\n');
        for f in program {
            self.fn_def(f)?;
            self.out.push('\n');
        }
        if !self.fns.contains_key("main") {
            let (l, c) = program.last().map(|f| (f.line, f.col)).unwrap_or((1, 1));
            return Err(LumeError::new(l, c, "no `main` function")
                .with_help("a program starts at `def main:`"));
        }
        Ok(())
    }

    fn fn_def(&mut self, f: &FnDef) -> Result<()> {
        if f.name == "main" && (!f.params.is_empty() || f.ret != Type::Unit) {
            return Err(LumeError::new(f.line, f.col, "`main` takes no parameters and returns nothing")
                .with_help("write `def main:`"));
        }
        let mut sig = format!("fn {}(", rust_name(&f.name));
        let mut names = HashSet::new();
        for (i, p) in f.params.iter().enumerate() {
            if !names.insert(p.name.clone()) {
                return Err(LumeError::new(p.line, p.col, format!("parameter `{}` is listed twice", p.name)));
            }
            if i > 0 {
                sig.push_str(", ");
            }
            sig.push_str(&format!("{}: {}", rust_name(&p.name), rust_type(&p.ty)));
        }
        sig.push(')');
        if f.ret != Type::Unit {
            sig.push_str(&format!(" -> {}", rust_type(&f.ret)));
        }
        sig.push_str(" {");
        self.line(&sig);
        self.indent += 1;
        self.push_scope();
        for p in &f.params {
            self.declare(&p.name, false, p.line);
        }
        self.current_ret = f.ret.clone();
        let want_value = f.ret != Type::Unit;
        self.block_body(&f.body, want_value)?;
        self.pop_scope();
        self.indent -= 1;
        self.line("}");
        Ok(())
    }

    /// Emits the statements of a block into the current Rust block. When
    /// `want_value` is set, the final expression statement is emitted without
    /// a semicolon so the block evaluates to it.
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
                let v = self.expr(value)?;
                if self.scopes.last().unwrap().contains_key(name) {
                    return Err(LumeError::new(*line, *col, format!("`{}` is already declared in this block", name))
                        .with_help(format!("to change it, write `{} = ...`", name)));
                }
                self.declare(name, true, *line);
                self.line(&format!("let mut {} = {};", rust_name(name), v));
                if is_tail {
                    self.tail_unit(*line, *col)?;
                }
            }
            Stmt::Bind { name, value, line, col } => {
                let v = self.expr(value)?;
                match self.lookup(name).cloned() {
                    Some(b) if b.mutable => {
                        self.line(&format!("{} = {};", rust_name(name), v));
                    }
                    Some(b) => {
                        // Self-transform shadowing: `x = x.trim`, `x = x + 1` rebinds an
                        // immutable x when the right side *starts with* x.
                        if leftmost_ident(value) == Some(name.as_str()) {
                            self.declare(name, false, *line);
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
                        self.declare(name, false, *line);
                        self.line(&format!("let {} = {};", rust_name(name), v));
                    }
                }
                if is_tail {
                    self.tail_unit(*line, *col)?;
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
                    None => return Err(self.unknown_name(name, *line, *col)),
                }
                if is_tail {
                    self.tail_unit(*line, *col)?;
                }
            }
            Stmt::Expr(e) => {
                if is_tail {
                    if let ExprKind::If { else_block: None, .. } = &e.kind {
                        return Err(LumeError::new(e.line, e.col, "this `if` is the function's result but has no `else`")
                            .with_help("add an `else` branch, or add `return` before it if it is not the result"));
                    }
                    let v = self.expr(e)?;
                    self.line(&v);
                } else {
                    let v = self.expr_stmt(e)?;
                    self.line(&format!("{};", v));
                }
            }
            Stmt::Return { value, line, col } => {
                match value {
                    Some(e) => {
                        if self.current_ret == Type::Unit {
                            return Err(LumeError::new(*line, *col, "this function returns nothing, but `return` has a value")
                                .with_help("add `-> Type` to the function signature"));
                        }
                        let v = self.expr(e)?;
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
            Stmt::For { var, iter, filter, body, line, .. } => {
                let it = match &iter.kind {
                    ExprKind::Range { .. } => self.expr(iter)?,
                    _ => format!("({}).iter().cloned()", self.expr(iter)?),
                };
                self.line(&format!("for {} in {} {{", rust_name(var), it));
                self.indent += 1;
                self.push_scope();
                self.declare(var, false, *line);
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

    fn tail_unit(&mut self, line: usize, col: usize) -> Result<()> {
        Err(LumeError::new(line, col, format!(
            "the last line of a function returning `{}` must be a value, not a binding",
            type_name(&self.current_ret)
        ))
        .with_help("put the value on its own line after the binding"))
    }

    /// An expression in statement position: an `if` here has unit branches.
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

    /// Emits an if/elif/else chain as a multi-line Rust expression. The first
    /// line carries no indentation (the caller places it); every later line
    /// carries its absolute indentation; the closing brace has no newline.
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
                    format!("String::from(\"{}\")", escape_rust_str(&s).replace("{{", "{").replace("}}", "}"))
                } else {
                    let mut fmt = String::new();
                    let mut args = Vec::new();
                    for p in pieces {
                        match p {
                            StrPiece::Lit(s) => fmt.push_str(&escape_rust_str(s)),
                            StrPiece::Expr(x) => {
                                fmt.push_str("{}");
                                args.push(self.expr(x)?);
                            }
                        }
                    }
                    format!("format!(\"{}\", {})", fmt, args.join(", "))
                }
            }
            ExprKind::Ident(name) => {
                if self.lookup(name).is_none() {
                    if self.fns.contains_key(name) {
                        return Err(LumeError::new(e.line, e.col, format!("`{}` is a function; call it with `{}()`", name, name)));
                    }
                    return Err(self.unknown_name(name, e.line, e.col));
                }
                rust_name(name)
            }
            ExprKind::List(items) => {
                let parts: Result<Vec<String>> = items.iter().map(|i| self.expr(i)).collect();
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
                let l = self.expr(lhs)?;
                let r = self.expr(rhs)?;
                match *op {
                    "and" => format!("({} && {})", l, r),
                    "or" => format!("({} || {})", l, r),
                    "**" => format!("({}).lume_pow({})", l, r),
                    "+" => {
                        // String + String is allowed in Lume; Rust wants String + &str.
                        let lhs_is_str = matches!(lhs.kind, ExprKind::Str(_));
                        let rhs_is_str = matches!(rhs.kind, ExprKind::Str(_));
                        if lhs_is_str || rhs_is_str {
                            format!("format!(\"{{}}{{}}\", {}, {})", l, r)
                        } else {
                            format!("({} + {})", l, r)
                        }
                    }
                    _ => format!("({} {} {})", l, op, r),
                }
            }
            ExprKind::Call { name, args } => {
                let (arity, _) = match self.fns.get(name) {
                    Some(f) => f.clone(),
                    None => {
                        if self.lookup(name).is_some() {
                            return Err(LumeError::new(e.line, e.col, format!("`{}` is a value, not a function", name)));
                        }
                        let err = LumeError::new(e.line, e.col, format!("unknown function `{}`", name));
                        return Err(match self.suggest(name) {
                            Some(s) => err.with_help(format!("did you mean `{}`?", s)),
                            None => err,
                        });
                    }
                };
                if args.len() != arity {
                    return Err(LumeError::new(e.line, e.col, format!(
                        "`{}` takes {} argument{}, but {} {} given",
                        name,
                        arity,
                        if arity == 1 { "" } else { "s" },
                        args.len(),
                        if args.len() == 1 { "was" } else { "were" }
                    )));
                }
                let parts: Result<Vec<String>> = args.iter().map(|a| self.expr(a)).collect();
                format!("{}({})", rust_name(name), parts?.join(", "))
            }
            ExprKind::Method { recv, name, args } => {
                let r = self.expr(recv)?;
                let parts: Result<Vec<String>> = args.iter().map(|a| self.expr(a)).collect();
                let a = parts?;
                self.method(&r, name, &a, e)?
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
                format!("println!(\"{{}}\", {})", a)
            }
        })
    }

    /// Maps Lume method names onto Rust. Unknown names pass through as
    /// ordinary method calls so user structs (milestone 2) will work.
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
            "to_str" | "to_s" => { need(0)?; format!("({}).to_string()", recv) }
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

/// The identifier an expression starts with, reading left to right:
/// `x.trim` -> x, `x + 1` -> x, `(x) * 2` -> x, `y + x` -> y, `3 + x` -> None.
fn leftmost_ident(e: &Expr) -> Option<&str> {
    match &e.kind {
        ExprKind::Ident(n) => Some(n.as_str()),
        ExprKind::Method { recv, .. } => leftmost_ident(recv),
        ExprKind::Binary { lhs, .. } => leftmost_ident(lhs),
        ExprKind::Range { lo, .. } => leftmost_ident(lo),
        _ => None,
    }
}

fn type_name(t: &Type) -> String {
    match t {
        Type::Int => "Int".into(),
        Type::Float => "Float".into(),
        Type::Bool => "Bool".into(),
        Type::Str => "Str".into(),
        Type::Unit => "()".into(),
        Type::List(i) => format!("[{}]", type_name(i)),
        Type::Named(n) => n.clone(),
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

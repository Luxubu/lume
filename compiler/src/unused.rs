//! Names given a value and never read, warned about as Rust's
//! `unused_variables` lint does: a local, a `for` variable, a parameter, a
//! block's parameter, a name a pattern binds. A name that begins with `_`
//! says it is meant, and is not warned about.
//!
//! The check reads the tree only, after the program has compiled: a name
//! counts as read if any later statement of its block — or of a block
//! around it — mentions it, which may keep quiet about a value written
//! over before it is read, and never warns about one that is read.

use crate::ast::*;
use crate::error::LumeError;
use std::collections::HashSet;

pub fn warnings(program: &[Item], src: &str) -> Vec<LumeError> {
    let mut w = Walker { out: Vec::new(), scopes: Vec::new(), lines: src.lines().map(|l| l.to_string()).collect() };
    let fields_of = |name: &str| -> Vec<String> {
        program
            .iter()
            .find_map(|i| match i {
                Item::Struct(s) if s.name == name => Some(s.fields.iter().map(|f| f.name.clone()).collect()),
                _ => None,
            })
            .unwrap_or_default()
    };
    for item in program {
        match item {
            Item::Fn(f) => w.function(f, &[]),
            Item::Struct(s) => {
                let fields: Vec<String> = s.fields.iter().map(|f| f.name.clone()).collect();
                for m in &s.methods {
                    w.function(m, &fields);
                }
            }
            Item::Enum(e) => {
                for m in &e.methods {
                    w.function(m, &[]);
                }
            }
            Item::Interface(i) => {
                for m in &i.defaults {
                    w.function(m, &[]);
                }
            }
            Item::Extend(x) => {
                let fields = match &x.target {
                    Type::Named(n) | Type::App(n, _) => fields_of(n),
                    _ => Vec::new(),
                };
                for m in &x.methods {
                    w.function(m, &fields);
                }
            }
            Item::Test(t) => {
                w.scopes.push(HashSet::new());
                w.block(&t.body.stmts, &[]);
                w.scopes.pop();
            }
            Item::Const(_) | Item::Import(_) => {}
        }
    }
    w.out.sort_by_key(|e| (e.line, e.col));
    w.out
}

struct Walker {
    out: Vec<LumeError>,
    /// The names in scope, innermost last: a `name = value` of one of them
    /// changes it, and binds nothing new.
    scopes: Vec<HashSet<String>>,
    /// The file's lines, to point at one name of `(a, b) = ...`.
    lines: Vec<String>,
}

fn quiet(name: &str) -> bool {
    name.starts_with('_') || name == "self" || name.chars().next().map(|c| c.is_uppercase()).unwrap_or(true)
}

/// A read of `name`: the name itself, or `name?` and `name!`, which the
/// lexer keeps as one word since a name may end in `?` (`empty?`).
fn reads_stmt(s: &Stmt, name: &str) -> bool {
    [name.to_string(), format!("{}?", name), format!("{}!", name)].iter().any(|n| crate::codegen::stmt_mentions_pub(s, n))
}

fn reads_expr(e: &Expr, name: &str) -> bool {
    [name.to_string(), format!("{}?", name), format!("{}!", name)].iter().any(|n| crate::codegen::expr_mentions_pub(e, n))
}

fn mentioned(name: &str, rest: &[Stmt], outer: &[&[Stmt]]) -> bool {
    rest.iter().any(|s| reads_stmt(s, name)) || outer.iter().any(|r| r.iter().any(|s| reads_stmt(s, name)))
}

fn block_mentions(b: &Block, name: &str) -> bool {
    b.stmts.iter().any(|s| reads_stmt(s, name))
}

impl Walker {
    fn bound(&self, name: &str) -> bool {
        self.scopes.iter().any(|s| s.contains(name))
    }

    fn bind(&mut self, name: &str) {
        if let Some(s) = self.scopes.last_mut() {
            s.insert(name.to_string());
        }
    }

    /// The column of `name` as a whole word on `line`, at or after `col`.
    fn name_col(&self, line: usize, col: usize, name: &str) -> Option<usize> {
        let text = self.lines.get(line.checked_sub(1)?)?;
        let chars: Vec<char> = text.chars().collect();
        let want: Vec<char> = name.chars().collect();
        let word = |c: char| c.is_alphanumeric() || c == '_' || c == '?';
        let mut i = col.saturating_sub(1);
        while i + want.len() <= chars.len() {
            let before = i == 0 || !word(chars[i - 1]);
            let after = i + want.len() == chars.len() || !word(chars[i + want.len()]);
            if before && after && chars[i..i + want.len()] == want[..] {
                return Some(i + 1);
            }
            i += 1;
        }
        None
    }

    fn function(&mut self, f: &FnDef, fields: &[String]) {
        let mut scope: HashSet<String> = fields.iter().cloned().collect();
        for p in &f.params {
            scope.insert(p.name.clone());
            // a `var` or `shared var` parameter changed in place is used: the
            // caller sees the change
            let changes_callers = p.mutable || matches!(p.ty, Type::Shared(..));
            if !quiet(&p.name) && !changes_callers && !block_mentions(&f.body, &p.name) {
                self.out.push(
                    LumeError::new(p.line, p.col, format!("parameter `{}` is never used", p.name))
                        .with_help(format!("name it `_{}` if it is there only to fit a signature, as an interface's method may be", p.name)),
                );
            }
        }
        self.scopes.push(scope);
        self.block(&f.body.stmts, &[]);
        self.scopes.pop();
    }

    /// The statements of one block; `outer` holds what follows this block in
    /// each block around it, which may read a name bound here too.
    fn block(&mut self, stmts: &[Stmt], outer: &[&[Stmt]]) {
        self.scopes.push(HashSet::new());
        for (i, s) in stmts.iter().enumerate() {
            let rest = &stmts[i + 1..];
            let mut deeper: Vec<&[Stmt]> = outer.to_vec();
            deeper.push(rest);
            self.stmt(s, rest, outer, &deeper);
        }
        self.scopes.pop();
    }

    fn new_name(&mut self, name: &str, line: usize, col: usize, rest: &[Stmt], outer: &[&[Stmt]]) {
        if self.bound(name) {
            return;
        }
        if !quiet(name) && !mentioned(name, rest, outer) {
            self.out.push(LumeError::new(line, col, format!("`{}` is given a value but never used", name)).with_help(format!("use it, or name it `_{}` to say it is not meant to be", name)));
        }
        self.bind(name);
    }

    fn stmt(&mut self, s: &Stmt, rest: &[Stmt], outer: &[&[Stmt]], deeper: &[&[Stmt]]) {
        match s {
            Stmt::Bind { name, value, line, col, .. } | Stmt::Var { name, value, line, col, .. } | Stmt::Shared { name, value, line, col, .. } => {
                self.expr(value, deeper);
                self.new_name(name, *line, *col, rest, outer);
            }
            Stmt::Destructure { names, value, line, col } => {
                self.expr(value, deeper);
                for n in names {
                    if n != "_" {
                        // the caret on the name itself, inside the parentheses
                        let at = self.name_col(*line, *col, n).unwrap_or(*col);
                        self.new_name(n, *line, at, rest, outer);
                    }
                }
            }
            Stmt::OpAssign { value, .. } | Stmt::Yield { value, .. } | Stmt::Assert { cond: value, .. } => self.expr(value, deeper),
            Stmt::FieldAssign { recv, value, .. } => {
                self.expr(recv, deeper);
                self.expr(value, deeper);
            }
            Stmt::IndexAssign { recv, index, value, .. } => {
                self.expr(recv, deeper);
                self.expr(index, deeper);
                self.expr(value, deeper);
            }
            Stmt::Expr(e) => self.expr(e, deeper),
            Stmt::Return { value, .. } => {
                if let Some(v) = value {
                    self.expr(v, deeper);
                }
            }
            Stmt::While { cond, body } => {
                self.expr(cond, deeper);
                self.block(&body.stmts, deeper);
            }
            Stmt::For { vars, iter, filter, body, line, col, .. } => {
                self.expr(iter, deeper);
                for v in vars {
                    let read = block_mentions(body, v) || filter.as_ref().map(|f| reads_expr(f, v)).unwrap_or(false);
                    if !quiet(v) && !read {
                        self.out.push(LumeError::new(*line, *col, format!("`{}` is never used in the loop", v)).with_help(format!("name it `_{}`, or write `for _ in ...` when only the count matters", v)));
                    }
                }
                self.scopes.push(vars.iter().cloned().collect());
                if let Some(f) = filter {
                    self.expr(f, deeper);
                }
                self.block(&body.stmts, deeper);
                self.scopes.pop();
            }
            Stmt::Break { .. } | Stmt::Next { .. } => {}
        }
    }

    fn expr(&mut self, e: &Expr, outer: &[&[Stmt]]) {
        match &e.kind {
            ExprKind::Lambda { params, body } => {
                for p in params {
                    if !quiet(p) && !block_mentions(body, p) {
                        self.out.push(LumeError::new(e.line, e.col, format!("block parameter `{}` is never used", p)).with_help(format!("name it `_{}`, or `_` alone", p)));
                    }
                }
                self.scopes.push(params.iter().cloned().collect());
                self.block(&body.stmts, outer);
                self.scopes.pop();
            }
            ExprKind::Match { scrutinee, arms } => {
                self.expr(scrutinee, outer);
                for a in arms {
                    let mut names = Vec::new();
                    pattern_names(&a.pat, &mut names);
                    for (n, line, col) in &names {
                        let read = block_mentions(&a.body, n) || a.guard.as_ref().map(|g| reads_expr(g, n)).unwrap_or(false);
                        if !quiet(n) && !read {
                            self.out.push(LumeError::new(*line, *col, format!("`{}` is bound by the pattern but never used", n)).with_help(format!("write `_` in its place, or name it `_{}`", n)));
                        }
                    }
                    self.scopes.push(names.into_iter().map(|(n, _, _)| n).collect());
                    if let Some(g) = &a.guard {
                        self.expr(g, outer);
                    }
                    self.block(&a.body.stmts, outer);
                    self.scopes.pop();
                }
            }
            ExprKind::If { branches, else_block } => {
                for (c, b) in branches {
                    self.expr(c, outer);
                    self.block(&b.stmts, outer);
                }
                if let Some(b) = else_block {
                    self.block(&b.stmts, outer);
                }
            }
            ExprKind::Spawn(b) => self.block(&b.stmts, outer),
            ExprKind::Str(pieces) => {
                for p in pieces {
                    if let StrPiece::Expr(x) = p {
                        self.expr(x, outer);
                    }
                }
            }
            ExprKind::List(items) | ExprKind::Tuple(items) | ExprKind::SetLit(items) => {
                for i in items {
                    self.expr(i, outer);
                }
            }
            ExprKind::MapLit(pairs) => {
                for (k, v) in pairs {
                    self.expr(k, outer);
                    self.expr(v, outer);
                }
            }
            ExprKind::Range { lo, hi, .. } | ExprKind::Binary { lhs: lo, rhs: hi, .. } | ExprKind::Index { recv: lo, index: hi } => {
                self.expr(lo, outer);
                self.expr(hi, outer);
            }
            ExprKind::Unary { expr: x, .. }
            | ExprKind::Some(x)
            | ExprKind::Ok(x)
            | ExprKind::Try(x)
            | ExprKind::Unwrap(x)
            | ExprKind::Puts(x)
            | ExprKind::Warn(x)
            | ExprKind::Await(x)
            | ExprKind::TupleIndex { recv: x, .. } => self.expr(x, outer),
            ExprKind::Call { args, .. } => {
                for a in args {
                    self.expr(&a.value, outer);
                }
            }
            ExprKind::Method { recv, args, .. } => {
                self.expr(recv, outer);
                for a in args {
                    self.expr(&a.value, outer);
                }
            }
            ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) | ExprKind::Ident(_) | ExprKind::SelfRef | ExprKind::Placeholder | ExprKind::None | ExprKind::Rust(_) => {}
        }
    }
}

/// The names a pattern binds, with where each is written.
fn pattern_names(p: &Pattern, out: &mut Vec<(String, usize, usize)>) {
    match &p.kind {
        PatKind::Bind(n) => {
            if !out.iter().any(|(m, _, _)| m == n) {
                out.push((n.clone(), p.line, p.col));
            }
        }
        PatKind::Variant { args, .. } => args.iter().for_each(|a| pattern_names(a, out)),
        PatKind::Tuple(items) => items.iter().for_each(|a| pattern_names(a, out)),
        PatKind::List { items, rest } => {
            items.iter().for_each(|a| pattern_names(a, out));
            if let Some(Some(r)) = rest {
                if !out.iter().any(|(m, _, _)| m == r) {
                    out.push((r.clone(), p.line, p.col));
                }
            }
        }
        // each alternative binds the same names: the first says where
        PatKind::Or(alts) => {
            if let Some(a) = alts.first() {
                pattern_names(a, out);
            }
        }
        PatKind::Wild | PatKind::Int(_) | PatKind::Float(_) | PatKind::Bool(_) | PatKind::Str(_) | PatKind::Range { .. } => {}
    }
}

//! Recursive-descent parser for Lume. Hand-written (no parser generator) so
//! every error message can be tuned to say what the programmer meant to do.

use crate::ast::*;
use crate::error::{LumeError, Result};
use crate::lexer::{self, StrPart, Tok, Token};

pub struct Parser {
    toks: Vec<Token>,
    pos: usize,
    /// Column offset applied to positions, used when parsing an interpolated
    /// expression inside a string so errors point at the right column.
    line_base: usize,
    col_base: usize,
    shape: Shape,
}

/// What the tree does not keep but the formatter must reproduce: which forms
/// the programmer chose where two spellings mean the same thing. Keyed by
/// source position.
#[derive(Debug, Default, Clone)]
pub struct Shape {
    /// `stmt if cond` / `stmt unless cond`, keyed by the `if` expression the
    /// parser builds; the value is true for `unless`.
    pub trailing: std::collections::HashMap<(usize, usize), bool>,
    /// Blocks written on the same line as their opener (`if c: x`,
    /// `pat -> x`, `{ |x| x }`, `def f = x`), keyed by the position of the
    /// block's single statement.
    pub inline: std::collections::HashSet<(usize, usize)>,
    /// `lhs |> stage`, keyed by the node the stage produced; the value is
    /// true when the `|>` began a new line.
    pub pipes: std::collections::HashMap<(usize, usize), bool>,
}

pub fn parse_program(toks: Vec<Token>) -> Result<Vec<Item>> {
    parse_program_shaped(toks).map(|(items, _)| items)
}

pub fn parse_program_shaped(toks: Vec<Token>) -> Result<(Vec<Item>, Shape)> {
    let mut p = Parser { toks, pos: 0, line_base: 0, col_base: 0, shape: Shape::default() };
    let items = p.program()?;
    Ok((items, p.shape))
}

/// Position of a statement, for the `inline` set.
pub fn stmt_pos(s: &Stmt) -> (usize, usize) {
    match s {
        Stmt::Bind { line, col, .. }
        | Stmt::Var { line, col, .. }
        | Stmt::OpAssign { line, col, .. }
        | Stmt::FieldAssign { line, col, .. }
        | Stmt::IndexAssign { line, col, .. }
        | Stmt::Return { line, col, .. }
        | Stmt::For { line, col, .. }
        | Stmt::Break { line, col }
        | Stmt::Next { line, col }
        | Stmt::Assert { line, col, .. }
        | Stmt::Shared { line, col, .. } => (*line, *col),
        Stmt::Expr(e) => (e.line, e.col),
        Stmt::While { cond, .. } => (cond.line, cond.col),
    }
}

impl Parser {
    // ----- token helpers ---------------------------------------------------

    fn peek(&self) -> &Tok {
        &self.toks[self.pos].tok
    }

    fn peek_at(&self, k: usize) -> &Tok {
        let i = (self.pos + k).min(self.toks.len() - 1);
        &self.toks[i].tok
    }

    fn here(&self) -> (usize, usize) {
        let t = &self.toks[self.pos];
        (t.line + self.line_base, t.col + self.col_base)
    }

    fn prev_tok(&self) -> Option<&Tok> {
        if self.pos == 0 {
            None
        } else {
            Some(&self.toks[self.pos - 1].tok)
        }
    }

    fn advance(&mut self) -> Token {
        let t = self.toks[self.pos].clone();
        if self.pos < self.toks.len() - 1 {
            self.pos += 1;
        }
        t
    }

    fn at_sym(&self, s: &str) -> bool {
        matches!(self.peek(), Tok::Sym(x) if *x == s)
    }

    fn at_kw(&self, s: &str) -> bool {
        matches!(self.peek(), Tok::Ident(x) if x == s)
    }

    fn eat_sym(&mut self, s: &str) -> bool {
        if self.at_sym(s) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn eat_kw(&mut self, s: &str) -> bool {
        if self.at_kw(s) {
            self.advance();
            true
        } else {
            false
        }
    }

    fn err(&self, msg: impl Into<String>) -> LumeError {
        let (l, c) = self.here();
        LumeError::new(l, c, msg)
    }

    fn describe(&self) -> String {
        match self.peek() {
            Tok::Ident(s) => format!("`{}`", s),
            Tok::Int(v) => format!("`{}`", v),
            Tok::Float(v) => format!("`{}`", v),
            Tok::Str(_) => "a string".into(),
            Tok::Sym(s) => format!("`{}`", s),
            Tok::Newline => "the end of the line".into(),
            Tok::Indent => "an indented block".into(),
            Tok::Dedent => "the end of the block".into(),
            Tok::Eof => "the end of the file".into(),
        }
    }

    fn expect_sym(&mut self, s: &str, what: &str) -> Result<()> {
        if self.eat_sym(s) {
            Ok(())
        } else {
            Err(self.err(format!("expected `{}` {}, found {}", s, what, self.describe())))
        }
    }

    fn ident(&mut self, what: &str) -> Result<(String, usize, usize)> {
        let (l, c) = self.here();
        match self.peek().clone() {
            Tok::Ident(s) if !lexer::is_keyword(&s) => {
                self.advance();
                Ok((s, l, c))
            }
            Tok::Ident(s) => Err(LumeError::new(
                l,
                c,
                format!("`{}` is a keyword and cannot be used as {}", s, what),
            )),
            _ => Err(self.err(format!("expected {}, found {}", what, self.describe()))),
        }
    }

    /// A field or method name: like `ident`, but a few keywords are allowed
    /// because they are natural member names (`next`, `match`, `in`) and are
    /// never ambiguous after a `.` or in a field list. Bare use inside a
    /// method still means the keyword, so such fields are read as `self.next`.
    fn member_name(&mut self, what: &str) -> Result<(String, usize, usize)> {
        let (l, c) = self.here();
        if let Tok::Ident(s) = self.peek().clone() {
            if matches!(s.as_str(), "next" | "match" | "in" | "where" | "test" | "import" | "pub" | "extend" | "interface" | "assert") {
                self.advance();
                return Ok((s, l, c));
            }
        }
        self.ident(what)
    }

    /// `name (` — a space between a name and `(` is always an error: it is
    /// unclear whether it is a call or a value in parentheses.
    fn reject_spaced_paren(&self, name: &str) -> Result<()> {
        if self.at_sym("(") && self.toks[self.pos].space_before {
            let (l, c) = self.here();
            return Err(LumeError::new(l, c, format!("a space between `{}` and `(` is ambiguous: a call, or a value in parentheses?", name))
                .with_help(format!("for a call write `{}(...)` with no space; for a value in parentheses, drop the outer parentheses or wrap the whole argument: `{}((a + b) * 2)`", name, name)));
        }
        Ok(())
    }

    fn mark_inline(&mut self, b: &Block) {
        if let Some(st) = b.stmts.first() {
            self.shape.inline.insert(stmt_pos(st));
        }
    }

    fn skip_newlines(&mut self) {
        while matches!(self.peek(), Tok::Newline) {
            self.advance();
        }
    }

    /// A statement ends at a newline, or right after a block that already
    /// consumed its own dedent.
    fn end_stmt(&mut self) -> Result<()> {
        match self.peek() {
            Tok::Newline => {
                self.advance();
                Ok(())
            }
            Tok::Dedent | Tok::Eof => Ok(()),
            _ if matches!(self.prev_tok(), Some(Tok::Dedent)) => Ok(()),
            _ => Err(self
                .err(format!("expected the end of the line, found {}", self.describe()))
                .with_help("only one statement per line; wrap long expressions in parentheses")),
        }
    }

    // ----- program / functions --------------------------------------------

    fn program(&mut self) -> Result<Vec<Item>> {
        let mut items = Vec::new();
        self.skip_newlines();
        while !matches!(self.peek(), Tok::Eof) {
            let public = self.at_kw("pub");
            if public {
                let (pl, pc) = self.here();
                self.advance();
                if !(self.at_kw("def") || self.at_kw("async") || self.at_kw("struct") || self.at_kw("enum") || self.at_kw("interface")) {
                    return Err(LumeError::new(pl, pc, "`pub` goes before `def`, `struct`, `enum` or `interface`"));
                }
            }
            if self.at_kw("interface") {
                let mut i = self.interface_def()?;
                i.public = public;
                items.push(Item::Interface(i));
                self.skip_newlines();
                continue;
            }
            if self.at_kw("extend") {
                items.push(Item::Extend(self.extend_def()?));
                self.skip_newlines();
                continue;
            }
            if self.at_kw("test") {
                items.push(Item::Test(self.test_def()?));
                self.skip_newlines();
                continue;
            }
            if self.at_kw("def") || self.at_kw("async") {
                let mut f = self.fn_def(false)?;
                f.public = public;
                items.push(Item::Fn(f));
            } else if self.at_kw("struct") {
                let mut s = self.struct_def()?;
                s.public = public;
                items.push(Item::Struct(s));
            } else if self.at_kw("enum") {
                let mut e = self.enum_def()?;
                e.public = public;
                items.push(Item::Enum(e));
            } else if self.at_kw("import") {
                items.push(Item::Import(self.import_def()?));
            } else if matches!(self.peek(), Tok::Indent) {
                return Err(self
                    .err("unexpected indentation at the top level")
                    .with_help("top-level code goes inside `def main:`"));
            } else {
                return Err(self
                    .err(format!("expected `def`, `struct`, `enum`, `interface`, `import` or `test`, found {}", self.describe()))
                    .with_help("a file is a list of `def` functions, `struct` and `enum` types; statements go inside `def main:`"));
            }
            self.skip_newlines();
        }
        Ok(items)
    }

    fn struct_def(&mut self) -> Result<StructDef> {
        let (line, col) = self.here();
        self.advance(); // struct
        let (name, nl, nc) = self.ident("a struct name")?;
        if !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            return Err(LumeError::new(nl, nc, format!("struct names start with a capital letter: `{}`", name))
                .with_help(format!("rename it `{}`", capitalize(&name))));
        }
        if !self.eat_sym(":") {
            return Err(self
                .err(format!("expected `:` after `struct {}`", name))
                .with_help("a struct's fields and methods go in an indented block"));
        }
        if !matches!(self.peek(), Tok::Newline) {
            return Err(self.err("expected the fields of the struct on the following lines"));
        }
        self.advance();
        if !matches!(self.peek(), Tok::Indent) {
            return Err(self
                .err(format!("struct `{}` has no fields", name))
                .with_help("indent at least one `name: Type` line under it"));
        }
        self.advance();
        let mut fields = Vec::new();
        let mut methods = Vec::new();
        while !matches!(self.peek(), Tok::Dedent | Tok::Eof) {
            if self.at_kw("def") || self.at_kw("async") {
                methods.push(self.fn_def(true)?);
            } else if let Tok::Ident(fname) = self.peek().clone() {
                if !methods.is_empty() {
                    return Err(self
                        .err(format!("field `{}` comes after a method", fname))
                        .with_help("list all fields first, then the methods"));
                }
                let (fname, fl, fc) = self.member_name("a field name")?;
                if !self.eat_sym(":") {
                    return Err(self
                        .err(format!("field `{}` needs a type", fname))
                        .with_help(format!("write `{}: Int`, `{}: Str`, and so on", fname, fname)));
                }
                let ty = self.parse_type()?;
                if self.at_sym("=") {
                    return Err(self.err("field defaults are not supported yet"));
                }
                fields.push(Param { name: fname, ty, mutable: false, line: fl, col: fc });
                self.end_stmt()?;
            } else {
                return Err(self.err(format!("expected a field or `def` inside `struct {}`, found {}", name, self.describe())));
            }
            self.skip_newlines();
        }
        if matches!(self.peek(), Tok::Dedent) {
            self.advance();
        }
        if fields.is_empty() {
            return Err(LumeError::new(line, col, format!("struct `{}` has no fields", name))
                .with_help("a struct needs at least one `name: Type` field"));
        }
        Ok(StructDef { name, public: false, fields, methods, line, col })
    }

    fn interface_def(&mut self) -> Result<InterfaceDef> {
        let (line, col) = self.here();
        self.advance(); // interface
        let (name, nl, nc) = self.ident("an interface name")?;
        if !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            return Err(LumeError::new(nl, nc, format!("interface names start with a capital letter: `{}`", name))
                .with_help(format!("rename it `{}`", capitalize(&name))));
        }
        if !self.eat_sym(":") {
            return Err(self.err(format!("expected `:` after `interface {}`", name)));
        }
        if !matches!(self.peek(), Tok::Newline) || !matches!(self.peek_at(1), Tok::Indent) {
            return Err(self.err(format!("interface `{}` has no methods", name)).with_help("indent at least one `def name -> Type` under it"));
        }
        self.advance();
        self.advance();
        let mut required = Vec::new();
        let mut defaults = Vec::new();
        while !matches!(self.peek(), Tok::Dedent | Tok::Eof) {
            if self.at_kw("async") {
                return Err(self.err("an interface method cannot be `async` yet").with_help("give the interface a plain method and do the awaiting in the caller"));
            }
            if !self.at_kw("def") {
                return Err(self.err(format!("expected `def` inside `interface {}`, found {}", name, self.describe())));
            }
            let (dl, dc) = self.here();
            let f = self.fn_signature_or_def()?;
            match f {
                (f, false) => {
                    if f.ret.is_none() {
                        return Err(LumeError::new(dl, dc, format!("interface method `{}` needs a return type", f.name))
                            .with_help("there is no body to infer it from: `def area -> Float`"));
                    }
                    required.push(f);
                }
                (f, true) => defaults.push(f),
            }
            self.skip_newlines();
        }
        if matches!(self.peek(), Tok::Dedent) {
            self.advance();
        }
        if required.is_empty() && defaults.is_empty() {
            return Err(LumeError::new(line, col, format!("interface `{}` has no methods", name)));
        }
        Ok(InterfaceDef { name, public: false, required, defaults, line, col })
    }

    /// `def name(params) -> T` alone (a required interface method) or with
    /// `:`/`=` and a body. Returns (def, has_body).
    fn fn_signature_or_def(&mut self) -> Result<(FnDef, bool)> {
        // Look ahead: after the signature, is there a `:` or `=`?
        let save = self.pos;
        let f = self.fn_def(true);
        match f {
            Ok(f) => Ok((f, true)),
            Err(e) => {
                // retry as a bare signature
                self.pos = save;
                let (line, col) = self.here();
                self.advance(); // def
                let (name, _, _) = self.ident("a method name")?;
                let mut params = Vec::new();
                if self.eat_sym("(") {
                    while !self.at_sym(")") {
                        let mutable = self.eat_kw("var");
                        let (pname, pl, pc) = self.ident("a parameter name")?;
                        if !self.eat_sym(":") {
                            return Err(self.err(format!("parameter `{}` needs a type", pname)));
                        }
                        let ty = self.parse_type()?;
                        params.push(Param { name: pname, ty, mutable, line: pl, col: pc });
                        if !self.eat_sym(",") {
                            break;
                        }
                    }
                    self.expect_sym(")", "to close the parameter list")?;
                }
                let ret = if self.eat_sym("->") { Some(self.parse_type()?) } else { None };
                if self.at_sym(":") || self.at_sym("=") {
                    return Err(e);
                }
                self.end_stmt()?;
                Ok((FnDef { name, public: true, is_async: false, params, ret, self_kind: SelfKind::Read, body: Block::default(), line, col }, false))
            }
        }
    }

    fn extend_def(&mut self) -> Result<ExtendDef> {
        let (line, col) = self.here();
        self.advance(); // extend
        let target = self.parse_type()?;
        if !self.eat_kw("with") && !matches!(self.peek(), Tok::Ident(s) if s == "with") {
            return Err(self.err("expected `with` and an interface name").with_help("write `extend Str with Shape:`"));
        }
        let (iface, _, _) = self.ident("an interface name")?;
        if !self.eat_sym(":") {
            return Err(self.err(format!("expected `:` after `extend ... with {}`", iface)));
        }
        if !matches!(self.peek(), Tok::Newline) || !matches!(self.peek_at(1), Tok::Indent) {
            return Err(self.err("expected the methods of the extension on the following lines, indented"));
        }
        self.advance();
        self.advance();
        let mut methods = Vec::new();
        while !matches!(self.peek(), Tok::Dedent | Tok::Eof) {
            if !self.at_kw("def") && !self.at_kw("async") {
                return Err(self.err(format!("expected `def` inside `extend`, found {}", self.describe())));
            }
            methods.push(self.fn_def(true)?);
            self.skip_newlines();
        }
        if matches!(self.peek(), Tok::Dedent) {
            self.advance();
        }
        Ok(ExtendDef { target, iface, methods, line, col })
    }

    fn import_def(&mut self) -> Result<Import> {
        let (line, col) = self.here();
        self.advance(); // import
        let (hl, hc) = self.here();
        let head = match self.peek().clone() {
            Tok::Ident(s) => {
                self.advance();
                s
            }
            _ => return Err(self.err(format!("expected a module name after `import`, found {}", self.describe()))),
        };
        let is_rust = head == "rust";
        let mut path: Vec<String> = Vec::new();
        if is_rust {
            if !self.eat_sym(".") {
                return Err(self.err("expected `.` and a crate name after `import rust`").with_help("for example `import rust.regex`"));
            }
            let (krate, _, _) = self.ident("a crate name")?;
            path.push(krate);
        } else {
            if lexer::is_keyword(&head) {
                return Err(LumeError::new(hl, hc, format!("`{}` is a keyword and cannot be a module name", head)));
            }
            path.push(head);
            while self.eat_sym(".") {
                let (seg, _, _) = self.member_name("a module or item name")?;
                path.push(seg);
            }
        }
        let version = if self.eat_sym("=") {
            if !is_rust {
                return Err(self.err("only `import rust.<crate>` takes a version"));
            }
            match self.peek().clone() {
                Tok::Str(parts) => {
                    self.advance();
                    let mut v = String::new();
                    for p in parts {
                        match p {
                            StrPart::Lit(t) => v.push_str(&t),
                            _ => return Err(self.err("a crate version is a plain string like \"1.10\"")),
                        }
                    }
                    Some(v)
                }
                _ => return Err(self.err("expected a version string after `=`, like `import rust.regex = \"1.10\"`")),
            }
        } else {
            None
        };
        let alias = if self.at_kw("as") || matches!(self.peek(), Tok::Ident(s) if s == "as") {
            self.advance();
            Some(self.ident("an alias")?.0)
        } else {
            None
        };
        self.end_stmt()?;
        Ok(Import { path, is_rust, version, alias, line, col })
    }

    fn enum_def(&mut self) -> Result<EnumDef> {
        let (line, col) = self.here();
        self.advance(); // enum
        let (name, nl, nc) = self.ident("an enum name")?;
        if !name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
            return Err(LumeError::new(nl, nc, format!("enum names start with a capital letter: `{}`", name))
                .with_help(format!("rename it `{}`", capitalize(&name))));
        }
        if !self.eat_sym(":") {
            return Err(self.err(format!("expected `:` after `enum {}`", name)));
        }
        if !matches!(self.peek(), Tok::Newline) || !matches!(self.peek_at(1), Tok::Indent) {
            return Err(self
                .err(format!("enum `{}` has no variants", name))
                .with_help("indent at least one variant under it, like `Circle(r: Float)` or `Empty`"));
        }
        self.advance();
        self.advance();
        let mut variants: Vec<Variant> = Vec::new();
        let mut methods = Vec::new();
        while !matches!(self.peek(), Tok::Dedent | Tok::Eof) {
            if self.at_kw("def") || self.at_kw("async") {
                methods.push(self.fn_def(true)?);
            } else if let Tok::Ident(vname) = self.peek().clone() {
                if !methods.is_empty() {
                    return Err(self.err(format!("variant `{}` comes after a method", vname)).with_help("list all variants first, then the methods"));
                }
                let (vname, vl, vc) = self.ident("a variant name")?;
                if !vname.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) {
                    return Err(LumeError::new(vl, vc, format!("variant names start with a capital letter: `{}`", vname))
                        .with_help(format!("rename it `{}`", capitalize(&vname))));
                }
                if variants.iter().any(|v| v.name == vname) {
                    return Err(LumeError::new(vl, vc, format!("variant `{}` is listed twice", vname)));
                }
                let mut fields = Vec::new();
                if self.at_sym("(") {
                    self.advance();
                    while !self.at_sym(")") {
                        let (fname, fl, fc) = self.ident("a field name")?;
                        if !self.eat_sym(":") {
                            return Err(self.err(format!("field `{}` of `{}` needs a type", fname, vname))
                                .with_help(format!("write `{}({}: Float)`", vname, fname)));
                        }
                        let ty = self.parse_type()?;
                        fields.push(Param { name: fname, ty, mutable: false, line: fl, col: fc });
                        if !self.eat_sym(",") {
                            break;
                        }
                    }
                    self.expect_sym(")", "to close the variant's fields")?;
                    if fields.is_empty() {
                        return Err(LumeError::new(vl, vc, format!("`{}()` has no fields; write `{}` without parentheses", vname, vname)));
                    }
                }
                variants.push(Variant { name: vname, fields, line: vl, col: vc });
                self.end_stmt()?;
            } else {
                return Err(self.err(format!("expected a variant or `def` inside `enum {}`, found {}", name, self.describe())));
            }
            self.skip_newlines();
        }
        if matches!(self.peek(), Tok::Dedent) {
            self.advance();
        }
        if variants.is_empty() {
            return Err(LumeError::new(line, col, format!("enum `{}` has no variants", name)));
        }
        Ok(EnumDef { name, public: false, variants, methods, line, col })
    }

    fn fn_def(&mut self, in_struct: bool) -> Result<FnDef> {
        let (line, col) = self.here();
        let is_async = self.eat_kw("async");
        if is_async && !self.at_kw("def") {
            return Err(self.err(format!("expected `def` after `async`, found {}", self.describe())).with_help("write `async def name(...)`"));
        }
        self.advance(); // def
        let name = match self.peek().clone() {
            Tok::Sym(op) if matches!(op, "+" | "-" | "*" | "/" | "%" | "==" | "!=" | "<" | "<=" | ">" | ">=") => {
                let (ol, oc) = self.here();
                if !in_struct {
                    return Err(LumeError::new(ol, oc, format!("`def {}` defines an operator, which belongs inside a struct, enum or extend", op)));
                }
                self.advance();
                op.to_string()
            }
            _ => self.ident("a function name")?.0,
        };
        let mut params = Vec::new();
        let mut self_kind = SelfKind::Read;
        if self.eat_sym("(") {
            let mut first = true;
            while !self.at_sym(")") {
                // `self` / `var self` as the first parameter of a method
                let is_var = self.at_kw("var");
                let self_next = if is_var { matches!(self.peek_at(1), Tok::Ident(s) if s == "self") } else { self.at_kw("self") };
                if self_next {
                    let (sl, sc) = self.here();
                    if !in_struct {
                        return Err(LumeError::new(sl, sc, "`self` is only allowed in a method inside a struct"));
                    }
                    if !first {
                        return Err(LumeError::new(sl, sc, "`self` must be the first parameter"));
                    }
                    if is_var {
                        self.advance();
                        self_kind = SelfKind::Mutate;
                    }
                    self.advance(); // self
                    if self.at_sym(":") {
                        return Err(self.err("`self` takes no type; it is always the struct itself"));
                    }
                    first = false;
                    if !self.eat_sym(",") {
                        break;
                    }
                    continue;
                }
                first = false;
                let mutable = self.eat_kw("var");
                let (pname, pl, pc) = self.ident("a parameter name")?;
                if !self.eat_sym(":") {
                    return Err(self
                        .err(format!("parameter `{}` needs a type", pname))
                        .with_help(format!("write `{}: Int`, `{}: Str`, and so on", pname, pname)));
                }
                let ty = self.parse_type()?;
                if mutable && ty.is_copy() {
                    return Err(LumeError::new(pl, pc, format!("`var {}` needs a list, map, string or struct; `{}` values are copied, so changing the copy would do nothing", pname, crate::codegen::type_name(&ty)))
                        .with_help("return the new value instead"));
                }
                params.push(Param { name: pname, ty, mutable, line: pl, col: pc });
                if !self.eat_sym(",") {
                    break;
                }
            }
            self.expect_sym(")", "to close the parameter list")?;
        }
        let ret = if self.eat_sym("->") { Some(self.parse_type()?) } else { None };
        let body = if self.eat_sym(":") {
            self.block()?
        } else if self.eat_sym("=") {
            // `def f = expr`, or `def f =` with the expression on the next indented line
            if matches!(self.peek(), Tok::Newline) && matches!(self.peek_at(1), Tok::Indent) {
                self.advance();
                self.advance();
                let e = self.expr()?;
                self.skip_newlines();
                if !matches!(self.peek(), Tok::Dedent) {
                    return Err(self
                        .err(format!("expected a single expression after `def {} =`, found {}", name, self.describe()))
                        .with_help("for several statements use `def ...:` and an indented body"));
                }
                self.advance();
                let b = Block { stmts: vec![Stmt::Expr(e)] };
                self.mark_inline(&b);
                b
            } else {
                let e = self.expr()?;
                self.end_stmt()?;
                let b = Block { stmts: vec![Stmt::Expr(e)] };
                self.mark_inline(&b);
                b
            }
        } else {
            return Err(self
                .err(format!("expected `:` or `=` after the signature of `{}`, found {}", name, self.describe()))
                .with_help("`def f(x: Int) -> Int:` starts a block; `def f(x: Int) -> Int = x * 2` is a one-liner"));
        };
        Ok(FnDef { name, public: false, is_async, params, ret, self_kind, body, line, col })
    }

    /// `test "name":` followed by an indented body.
    fn test_def(&mut self) -> Result<TestDef> {
        let (line, col) = self.here();
        self.advance(); // test
        let name = match self.peek().clone() {
            Tok::Str(parts) => {
                let (sl, sc) = self.here();
                self.advance();
                let mut text = String::new();
                for p in parts {
                    match p {
                        StrPart::Lit(s) => text.push_str(&s),
                        StrPart::Expr(..) => return Err(LumeError::new(sl, sc, "a test name is a plain string, without `#{}`")),
                    }
                }
                if text.trim().is_empty() {
                    return Err(LumeError::new(sl, sc, "a test needs a name"));
                }
                text
            }
            _ => {
                return Err(self
                    .err(format!("expected the test's name in quotes after `test`, found {}", self.describe()))
                    .with_help("write `test \"what it checks\":` and the body indented below"))
            }
        };
        if !self.eat_sym(":") {
            return Err(self.err(format!("expected `:` after the test name, found {}", self.describe())));
        }
        let body = self.block()?;
        Ok(TestDef { name, body, line, col })
    }

    fn parse_type(&mut self) -> Result<Type> {
        let t = self.parse_type_atom()?;
        if self.at_kw("or") {
            self.advance();
            let e = self.parse_type_atom()?;
            if self.at_kw("or") {
                return Err(self.err("`T or E` takes one error type").with_help("group alternatives in an `enum` if a function can fail in several ways"));
            }
            return Ok(Type::Result(Box::new(t), Box::new(e)));
        }
        Ok(t)
    }

    fn parse_type_atom(&mut self) -> Result<Type> {
        if self.eat_kw("shared") {
            let mutable = self.eat_kw("var");
            let inner = self.parse_type_atom()?;
            return Ok(Type::Shared(Box::new(inner), mutable));
        }
        if self.at_kw("Task") && matches!(self.peek_at(1), Tok::Sym("[")) {
            self.advance();
            self.advance();
            let inner = self.parse_type()?;
            self.expect_sym("]", "to close `Task[...]`")?;
            let t = Type::Task(Box::new(inner));
            return Ok(self.type_suffix(t));
        }
        if self.eat_sym("{") {
            let k = self.parse_type()?;
            if !self.eat_sym(":") {
                return Err(self.err("a map type is written `{Key: Value}`, like `{Str: Int}`"));
            }
            let v = self.parse_type()?;
            self.expect_sym("}", "to close the map type")?;
            return Ok(self.type_suffix(Type::Map(Box::new(k), Box::new(v))));
        }
        if self.eat_sym("[") {
            let inner = self.parse_type()?;
            self.expect_sym("]", "to close the list type")?;
            return Ok(self.type_suffix(Type::List(Box::new(inner))));
        }
        if self.eat_sym("(") {
            if self.eat_sym(")") {
                return Ok(Type::Unit);
            }
            let mut parts = vec![self.parse_type()?];
            while self.eat_sym(",") {
                parts.push(self.parse_type()?);
            }
            self.expect_sym(")", "to close the tuple type")?;
            if parts.len() == 1 {
                return Err(self.err("a tuple type needs at least two parts, like `(Int, Str)`"));
            }
            return Ok(self.type_suffix(Type::Tuple(parts)));
        }
        let (name, l, c) = match self.peek().clone() {
            Tok::Ident(s) => {
                let (l, c) = self.here();
                self.advance();
                (s, l, c)
            }
            _ => return Err(self.err(format!("expected a type, found {}", self.describe()))),
        };
        // qualified type from an imported module: `model.User`
        let mut name = name;
        while self.at_sym(".") && !self.toks[self.pos].space_before {
            if let Tok::Ident(seg) = self.peek_at(1).clone() {
                if seg.chars().next().map(|c| c.is_uppercase()).unwrap_or(false) || self.enum_or_module_like(&seg) {
                    self.advance();
                    self.advance();
                    name = format!("{}.{}", name, seg);
                    continue;
                }
            }
            break;
        }
        // The lexer folds a trailing `?` into identifiers (`empty?`); for a
        // type it means optional: `Int?`.
        let mut optional = 0;
        while name.ends_with('?') {
            name.pop();
            optional += 1;
        }
        let base = match name.as_str() {
            "Int" => Type::Int,
            "Float" => Type::Float,
            "Bool" => Type::Bool,
            "Str" => Type::Str,
            "int" | "float" | "bool" | "str" | "string" | "String" | "i64" | "f64" => {
                let fix = match name.as_str() {
                    "int" | "i64" => "Int",
                    "float" | "f64" => "Float",
                    "bool" => "Bool",
                    _ => "Str",
                };
                return Err(LumeError::new(l, c, format!("unknown type `{}`", name))
                    .with_help(format!("Lume spells it `{}`", fix)));
            }
            "Option" => {
                return Err(LumeError::new(l, c, "an optional value is written `T?`, not `Option[T]`")
                    .with_help("for example `Int?` or `User?`"));
            }
            _ => Type::Named(name),
        };
        let mut t = base;
        for _ in 0..optional {
            t = Type::Option(Box::new(t));
        }
        Ok(self.type_suffix(t))
    }

    fn enum_or_module_like(&self, seg: &str) -> bool {
        !lexer::is_keyword(seg) && seg.chars().all(|c| c.is_alphanumeric() || c == '_')
    }

    /// `T?` — the `?` binds to the type directly before it.
    fn type_suffix(&mut self, t: Type) -> Type {
        let mut t = t;
        while self.at_sym("?") && !self.toks[self.pos].space_before {
            self.advance();
            t = Type::Option(Box::new(t));
        }
        t
    }

    // ----- blocks and statements ------------------------------------------

    /// Parses `NEWLINE INDENT stmt* DEDENT`.
    fn block(&mut self) -> Result<Block> {
        if !matches!(self.peek(), Tok::Newline) {
            return Err(self
                .err(format!("expected an indented block on the next line, found {}", self.describe())));
        }
        self.advance();
        if !matches!(self.peek(), Tok::Indent) {
            return Err(self
                .err("expected an indented block")
                .with_help("the body must be indented more than the line above it"));
        }
        self.advance();
        let mut stmts = Vec::new();
        while !matches!(self.peek(), Tok::Dedent | Tok::Eof) {
            stmts.push(self.stmt()?);
            self.skip_newlines();
        }
        if matches!(self.peek(), Tok::Dedent) {
            self.advance();
        }
        Ok(Block { stmts })
    }

    fn stmt(&mut self) -> Result<Stmt> {
        let (line, col) = self.here();
        if matches!(self.peek(), Tok::Indent) {
            return Err(self
                .err("this line is indented more than the one above it, but nothing opened a block")
                .with_help("only a line ending in `:` opens a block; otherwise indent it the same as the line above"));
        }
        if self.eat_kw("var") {
            let (name, _, _) = self.ident("a variable name")?;
            let ty = if self.eat_sym(":") { Some(self.parse_type()?) } else { None };
            if !self.eat_sym("=") {
                return Err(self
                    .err(format!("`var {}` needs an initial value", name))
                    .with_help(format!("write `var {} = ...`", name)));
            }
            let value = self.expr()?;
            let s = Stmt::Var { name, ty, value, line, col };
            let s = self.trailing_condition(s)?;
            self.end_stmt()?;
            return Ok(s);
        }
        if self.eat_kw("return") {
            let value = if matches!(self.peek(), Tok::Newline | Tok::Dedent | Tok::Eof)
                || self.at_kw("if")
                || self.at_kw("unless")
            {
                None
            } else {
                Some(self.expr()?)
            };
            let s = Stmt::Return { value, line, col };
            let s = self.trailing_condition(s)?;
            self.end_stmt()?;
            return Ok(s);
        }
        if self.eat_kw("break") {
            let s = self.trailing_condition(Stmt::Break { line, col })?;
            self.end_stmt()?;
            return Ok(s);
        }
        if self.eat_kw("next") {
            let s = self.trailing_condition(Stmt::Next { line, col })?;
            self.end_stmt()?;
            return Ok(s);
        }
        if self.eat_kw("shared") {
            let mutable = self.eat_kw("var");
            let (name, _, _) = self.ident("a variable name")?;
            let ty = if self.eat_sym(":") { Some(self.parse_type()?) } else { None };
            if !self.eat_sym("=") {
                return Err(self
                    .err(format!("`shared {}` needs an initial value", name))
                    .with_help(format!("write `shared var {} = ...` (many tasks may change it) or `shared {} = ...` (read-only)", name, name)));
            }
            let value = self.expr()?;
            self.end_stmt()?;
            return Ok(Stmt::Shared { name, mutable, ty, value, line, col });
        }
        if self.eat_kw("assert") {
            if matches!(self.peek(), Tok::Newline | Tok::Dedent | Tok::Eof) {
                return Err(self.err("`assert` needs a condition").with_help("write `assert x == 3`"));
            }
            let cond = self.expr()?;
            if self.eat_sym(",") {
                return Err(self.err("`assert` takes only a condition; a failing `assert` prints both sides itself"));
            }
            self.end_stmt()?;
            return Ok(Stmt::Assert { cond, line, col });
        }
        if self.eat_kw("while") {
            let cond = self.expr()?;
            self.expect_sym(":", "after the `while` condition")?;
            let body = self.block()?;
            return Ok(Stmt::While { cond, body });
        }
        if self.eat_kw("for") {
            let mutable = self.eat_kw("var");
            let (var, _, _) = self.ident("a loop variable")?;
            let mut vars = vec![var];
            if mutable && self.at_sym(",") {
                return Err(self.err("`for var` takes one loop variable").with_help("write `for var item in items:`; pairs from `.enumerate` cannot be changed in place"));
            }
            while self.eat_sym(",") {
                let (v, vl, vc) = self.ident("a loop variable")?;
                if vars.contains(&v) {
                    return Err(LumeError::new(vl, vc, format!("loop variable `{}` is listed twice", v)));
                }
                vars.push(v);
            }
            if !self.eat_kw("in") {
                return Err(self
                    .err(format!("expected `in` after `for {}`", vars.join(", ")))
                    .with_help(format!("write `for {} in <collection or range>:`", vars.join(", "))));
            }
            let iter = self.expr()?;
            let filter = if self.eat_kw("where") { Some(self.expr()?) } else { None };
            self.expect_sym(":", "after the `for` header")?;
            let body = self.block()?;
            return Ok(Stmt::For { vars, mutable, iter, filter, body, line, col });
        }
        // Binding / assignment: `name = ...`, `name += ...`
        if let Tok::Ident(name) = self.peek().clone() {
            if !lexer::is_keyword(&name) {
                if let Tok::Sym(op) = self.peek_at(1).clone() {
                    match op {
                        "=" => {
                            self.advance();
                            self.advance();
                            let value = self.expr()?;
                            let s = Stmt::Bind { name, ty: None, value, line, col };
                            let s = self.trailing_condition(s)?;
                            self.end_stmt()?;
                            return Ok(s);
                        }
                        ":" => {
                            // typed binding: `name: Type = value`
                            self.advance();
                            self.advance();
                            let ty = self.parse_type()?;
                            if !self.eat_sym("=") {
                                return Err(self.err(format!("`{}: Type` needs a value", name)).with_help(format!("write `{}: Type = ...`", name)));
                            }
                            let value = self.expr()?;
                            let s = Stmt::Bind { name, ty: Some(ty), value, line, col };
                            let s = self.trailing_condition(s)?;
                            self.end_stmt()?;
                            return Ok(s);
                        }
                        "+=" | "-=" | "*=" | "/=" | "%=" => {
                            self.advance();
                            self.advance();
                            let value = self.expr()?;
                            let s = Stmt::OpAssign { name, op, value, line, col };
                            let s = self.trailing_condition(s)?;
                            self.end_stmt()?;
                            return Ok(s);
                        }
                        _ => {}
                    }
                }
            }
        }
        let e = self.expr()?;
        // Field assignment: `recv.field = value` / `recv.field += value`
        if let Tok::Sym(op) = self.peek().clone() {
            if matches!(op, "=" | "+=" | "-=" | "*=" | "/=" | "%=") {
                if let ExprKind::Index { recv, index } = &e.kind {
                    let recv = (**recv).clone();
                    let index = (**index).clone();
                    self.advance();
                    let value = self.expr()?;
                    let op = if op == "=" { None } else { Some(op) };
                    let s = Stmt::IndexAssign { recv, index, op, value, line, col };
                    let s = self.trailing_condition(s)?;
                    self.end_stmt()?;
                    return Ok(s);
                }
                if let ExprKind::Method { recv, name, args } = &e.kind {
                    if args.is_empty() {
                        let recv = (**recv).clone();
                        let field = name.clone();
                        self.advance();
                        let value = self.expr()?;
                        let op = if op == "=" { None } else { Some(op) };
                        let s = Stmt::FieldAssign { recv, field, op, value, line, col };
                        let s = self.trailing_condition(s)?;
                        self.end_stmt()?;
                        return Ok(s);
                    }
                }
                return Err(self
                    .err("only a name or a field can be assigned to")
                    .with_help("the left side of `=` must be `name` or `value.field`"));
            }
        }
        let s = self.trailing_condition(Stmt::Expr(e))?;
        self.end_stmt()?;
        Ok(s)
    }

    /// `stmt if cond` / `stmt unless cond` — Ruby's trailing forms.
    fn trailing_condition(&mut self, stmt: Stmt) -> Result<Stmt> {
        let (line, col) = self.here();
        if self.eat_kw("if") {
            let cond = self.expr()?;
            self.shape.trailing.insert((line, col), false);
            return Ok(Stmt::Expr(Expr::new(
                ExprKind::If { branches: vec![(cond, Block { stmts: vec![stmt] })], else_block: None },
                line,
                col,
            )));
        }
        if self.eat_kw("unless") {
            self.shape.trailing.insert((line, col), true);
            let cond = self.expr()?;
            let (cl, cc) = (cond.line, cond.col);
            let neg = Expr::new(ExprKind::Unary { op: "not", expr: Box::new(cond) }, cl, cc);
            return Ok(Stmt::Expr(Expr::new(
                ExprKind::If { branches: vec![(neg, Block { stmts: vec![stmt] })], else_block: None },
                line,
                col,
            )));
        }
        Ok(stmt)
    }

    // ----- expressions ------------------------------------------------------

    pub fn expr(&mut self) -> Result<Expr> {
        if self.at_kw("if") {
            return self.if_expr();
        }
        if self.at_kw("match") {
            return self.match_expr();
        }
        if self.at_kw("unless") {
            return Err(self
                .err("`unless` cannot start an expression")
                .with_help("use `if not ...:` here, or the trailing form `stmt unless cond`"));
        }
        self.binary(0)
    }

    fn if_expr(&mut self) -> Result<Expr> {
        let (line, col) = self.here();
        self.advance(); // if
        let mut branches = Vec::new();
        let mut else_block = None;
        let mut extra_indent = false;

        let cond = self.expr()?;
        let body = self.branch_body()?;
        branches.push((cond, body));

        loop {
            // Continuation may sit on the next line, optionally indented deeper.
            let mut save = self.pos;
            let mut consumed_indent = false;
            if matches!(self.peek(), Tok::Newline) {
                let k = if matches!(self.peek_at(1), Tok::Indent) { 2 } else { 1 };
                let next_is_branch = matches!(self.peek_at(k), Tok::Ident(s) if s == "elif" || s == "else");
                if next_is_branch {
                    self.advance();
                    if k == 2 {
                        if extra_indent {
                            // second level of continuation indent: not allowed
                            return Err(self
                                .err("`elif`/`else` is indented deeper than the previous branch")
                                .with_help("line it up with the `if` or the previous `elif`"));
                        }
                        self.advance();
                        consumed_indent = true;
                    }
                } else {
                    save = self.pos;
                }
            }
            if self.eat_kw("elif") {
                if consumed_indent {
                    extra_indent = true;
                }
                let cond = self.expr()?;
                let body = self.branch_body()?;
                branches.push((cond, body));
                continue;
            }
            if self.eat_kw("else") {
                if consumed_indent {
                    extra_indent = true;
                }
                if self.at_kw("if") {
                    return Err(self
                        .err("`else if` is written `elif` in Lume")
                        .with_help("replace `else if` with `elif`"));
                }
                else_block = Some(self.branch_body()?);
                break;
            }
            self.pos = save;
            break;
        }
        if extra_indent {
            if matches!(self.peek(), Tok::Newline) {
                self.advance();
            }
            if matches!(self.peek(), Tok::Dedent) {
                self.advance();
            }
        }
        Ok(Expr::new(ExprKind::If { branches, else_block }, line, col))
    }

    fn match_expr(&mut self) -> Result<Expr> {
        let (line, col) = self.here();
        self.advance(); // match
        let scrutinee = self.expr()?;
        if !self.eat_sym(":") {
            return Err(self.err(format!("expected `:` after the value to match, found {}", self.describe())));
        }
        if !matches!(self.peek(), Tok::Newline) || !matches!(self.peek_at(1), Tok::Indent) {
            return Err(self.err("the arms of a `match` go on the following lines, indented").with_help("each arm is `pattern -> value`"));
        }
        self.advance();
        self.advance();
        let mut arms = Vec::new();
        while !matches!(self.peek(), Tok::Dedent | Tok::Eof) {
            let (al, ac) = self.here();
            let pat = self.pattern()?;
            let guard = if self.eat_kw("if") { Some(self.expr()?) } else { None };
            if !self.eat_sym("->") {
                return Err(self
                    .err(format!("expected `->` after the pattern, found {}", self.describe()))
                    .with_help("an arm is `pattern -> value`, or `pattern ->` with an indented body"));
            }
            let body = if matches!(self.peek(), Tok::Newline) {
                self.block()?
            } else {
                // an inline arm holds one statement: a value, or an assignment
                let b = Block { stmts: vec![self.stmt()?] };
                self.mark_inline(&b);
                b
            };
            arms.push(MatchArm { pat, guard, body, line: al, col: ac });
            self.skip_newlines();
        }
        if matches!(self.peek(), Tok::Dedent) {
            self.advance();
        }
        if arms.is_empty() {
            return Err(LumeError::new(line, col, "`match` has no arms"));
        }
        Ok(Expr::new(ExprKind::Match { scrutinee: Box::new(scrutinee), arms }, line, col))
    }

    fn pattern(&mut self) -> Result<Pattern> {
        let (line, col) = self.here();
        let mk = |k: PatKind| Pattern { kind: k, line, col };
        match self.peek().clone() {
            Tok::Int(v) => {
                self.advance();
                if self.at_sym("..") || self.at_sym("...") {
                    let inclusive = self.at_sym("..");
                    self.advance();
                    match self.peek().clone() {
                        Tok::Int(hi) => {
                            self.advance();
                            Ok(mk(PatKind::Range { lo: v, hi, inclusive }))
                        }
                        _ => Err(self.err("a range pattern needs a number after `..`")),
                    }
                } else {
                    Ok(mk(PatKind::Int(v)))
                }
            }
            Tok::Sym("-") => {
                self.advance();
                match self.peek().clone() {
                    Tok::Int(v) => {
                        self.advance();
                        Ok(mk(PatKind::Int(-v)))
                    }
                    Tok::Float(v) => {
                        self.advance();
                        Ok(mk(PatKind::Float(-v)))
                    }
                    _ => Err(self.err("expected a number after `-` in the pattern")),
                }
            }
            Tok::Float(v) => {
                self.advance();
                Ok(mk(PatKind::Float(v)))
            }
            Tok::Str(parts) => {
                self.advance();
                let mut text = String::new();
                for p in parts {
                    match p {
                        StrPart::Lit(s) => text.push_str(&s),
                        StrPart::Expr(..) => return Err(LumeError::new(line, col, "a string pattern cannot contain `#{}` interpolation")),
                    }
                }
                Ok(mk(PatKind::Str(text)))
            }
            Tok::Sym("(") => {
                self.advance();
                let mut items = Vec::new();
                while !self.at_sym(")") {
                    items.push(self.pattern()?);
                    if !self.eat_sym(",") {
                        break;
                    }
                }
                self.expect_sym(")", "to close the tuple pattern")?;
                if items.len() < 2 {
                    return Err(LumeError::new(line, col, "a tuple pattern needs at least two parts, like `(a, b)`"));
                }
                Ok(mk(PatKind::Tuple(items)))
            }
            Tok::Sym("[") => {
                self.advance();
                let mut items = Vec::new();
                let mut rest = None;
                while !self.at_sym("]") {
                    if self.eat_sym("..") {
                        rest = Some(match self.peek().clone() {
                            Tok::Ident(n) if !lexer::is_keyword(&n) && n != "_" => {
                                self.advance();
                                Some(n)
                            }
                            Tok::Ident(n) if n == "_" => {
                                self.advance();
                                None
                            }
                            _ => None,
                        });
                        if !self.at_sym("]") {
                            return Err(self.err("`..rest` must be the last thing in a list pattern"));
                        }
                        break;
                    }
                    items.push(self.pattern()?);
                    if !self.eat_sym(",") {
                        break;
                    }
                }
                self.expect_sym("]", "to close the list pattern")?;
                Ok(mk(PatKind::List { items, rest }))
            }
            Tok::Ident(name) => {
                self.advance();
                match name.as_str() {
                    "_" => return Ok(mk(PatKind::Wild)),
                    "true" => return Ok(mk(PatKind::Bool(true))),
                    "false" => return Ok(mk(PatKind::Bool(false))),
                    _ if lexer::is_keyword(&name) => {
                        return Err(LumeError::new(line, col, format!("`{}` is a keyword and cannot be a pattern", name)));
                    }
                    _ => {}
                }
                let is_upper = name.chars().next().map(|c| c.is_uppercase()).unwrap_or(false);
                if !is_upper && !self.at_sym(".") {
                    if self.at_sym("(") {
                        return Err(self.err(format!("`{}` is a binding, so it takes no arguments; variants start with a capital letter", name)));
                    }
                    return Ok(mk(PatKind::Bind(name)));
                }
                // Variant, possibly qualified: Shape.Circle(r), model.Status.Draft
                let mut segs = vec![name];
                while self.at_sym(".") {
                    self.advance();
                    let (v, _, _) = self.ident("a variant name")?;
                    segs.push(v);
                }
                let vname = segs.pop().unwrap();
                let enum_name = if segs.is_empty() { None } else { Some(segs.join(".")) };
                let mut args = Vec::new();
                if self.at_sym("(") {
                    self.advance();
                    while !self.at_sym(")") {
                        args.push(self.pattern()?);
                        if !self.eat_sym(",") {
                            break;
                        }
                    }
                    self.expect_sym(")", "to close the variant pattern")?;
                }
                Ok(mk(PatKind::Variant { enum_name, name: vname, args }))
            }
            _ => Err(self.err(format!("expected a pattern, found {}", self.describe()))),
        }
    }

    /// After `if cond` / `elif cond` / `else`: either `: <inline expr>` or
    /// `:` followed by an indented block.
    fn branch_body(&mut self) -> Result<Block> {
        if !self.eat_sym(":") {
            return Err(self
                .err(format!("expected `:` after the condition, found {}", self.describe()))
                .with_help("`if x > 0: ...` — the colon starts the branch"));
        }
        if matches!(self.peek(), Tok::Newline) {
            self.block()
        } else if self.at_kw("return") || self.at_kw("break") || self.at_kw("next") {
            // `if done: return x` / `if x < 0: next`
            let (l, c) = self.here();
            let kw = match self.advance().tok {
                Tok::Ident(k) => k,
                _ => unreachable!(),
            };
            let st = match kw.as_str() {
                "return" => {
                    let value = if matches!(self.peek(), Tok::Newline | Tok::Dedent | Tok::Eof) { None } else { Some(self.expr()?) };
                    Stmt::Return { value, line: l, col: c }
                }
                "break" => Stmt::Break { line: l, col: c },
                _ => Stmt::Next { line: l, col: c },
            };
            let b = Block { stmts: vec![st] };
            self.mark_inline(&b);
            Ok(b)
        } else {
            let e = self.expr()?;
            let b = Block { stmts: vec![Stmt::Expr(e)] };
            self.mark_inline(&b);
            Ok(b)
        }
    }

    fn binary(&mut self, min_prec: u8) -> Result<Expr> {
        let mut lhs = self.unary()?;
        loop {
            let (op, prec, right_assoc): (&'static str, u8, bool) = match self.peek() {
                Tok::Ident(s) if s == "or" => ("or", 1, false),
                Tok::Ident(s) if s == "and" => ("and", 2, false),
                Tok::Sym(s) => match *s {
                    "==" => ("==", 4, false),
                    "!=" => ("!=", 4, false),
                    "<" => ("<", 4, false),
                    "<=" => ("<=", 4, false),
                    ">" => (">", 4, false),
                    ">=" => (">=", 4, false),
                    ".." => ("..", 5, false),
                    "..." => ("...", 5, false),
                    "+" => ("+", 6, false),
                    "-" => ("-", 6, false),
                    "*" => ("*", 7, false),
                    "/" => ("/", 7, false),
                    "%" => ("%", 7, false),
                    "**" => ("**", 9, true),
                    "|>" => ("|>", 0, false),
                    _ => break,
                },
                _ => break,
            };
            if prec < min_prec {
                break;
            }
            let (line, col) = self.here();
            let pipe_newline = self.pos > 0 && self.toks[self.pos - 1].line < self.toks[self.pos].line;
            self.advance();
            if op == "|>" {
                let key = self.here();
                lhs = self.pipe_stage(lhs, line, col)?;
                self.shape.pipes.insert(key, pipe_newline);
                continue;
            }
            let next_min = if right_assoc { prec } else { prec + 1 };
            let rhs = self.binary(next_min)?;
            lhs = match op {
                ".." | "..." => Expr::new(
                    ExprKind::Range { lo: Box::new(lhs), hi: Box::new(rhs), inclusive: op == ".." },
                    line,
                    col,
                ),
                _ => Expr::new(ExprKind::Binary { op, lhs: Box::new(lhs), rhs: Box::new(rhs) }, line, col),
            };
        }
        Ok(lhs)
    }

    /// The right side of `|>`: `.method(args)` applies to the piped value;
    /// `f(args)` receives it as the first argument; `f` alone calls `f(x)`.
    fn pipe_stage(&mut self, lhs: Expr, line: usize, col: usize) -> Result<Expr> {
        if self.at_sym(".") {
            return self.postfix_from(lhs);
        }
        match self.peek().clone() {
            Tok::Ident(name) if !lexer::is_keyword(&name) || name == "puts" => {
                let (sl, sc) = self.here();
                self.advance();
                if name == "puts" {
                    return Ok(Expr::new(ExprKind::Puts(Box::new(lhs)), sl, sc));
                }
                self.reject_spaced_paren(&name)?;
                let mut args = if self.at_sym("(") { self.call_args()? } else { Vec::new() };
                args.insert(0, Arg { name: None, value: lhs });
                let call = Expr::new(ExprKind::Call { name, args }, sl, sc);
                self.postfix_from(call)
            }
            _ => Err(LumeError::new(line, col, format!("expected a method or function after `|>`, found {}", self.describe()))
                .with_help("write `x |> .method(...)` or `x |> function(...)`")),
        }
    }

    fn unary(&mut self) -> Result<Expr> {
        let (line, col) = self.here();
        if self.eat_kw("await") {
            let e = self.unary()?;
            return Ok(Expr::new(ExprKind::Await(Box::new(e)), line, col));
        }
        if self.eat_kw("not") {
            let e = self.binary(3)?; // binds looser than comparison: `not a == b`
            return Ok(Expr::new(ExprKind::Unary { op: "not", expr: Box::new(e) }, line, col));
        }
        if self.eat_sym("-") {
            let e = self.unary()?;
            return Ok(Expr::new(ExprKind::Unary { op: "-", expr: Box::new(e) }, line, col));
        }
        if self.eat_sym("!") {
            return Err(LumeError::new(line, col, "`!` is not the boolean not")
                .with_help("write `not x`"));
        }
        self.postfix()
    }

    fn postfix(&mut self) -> Result<Expr> {
        let e = self.primary()?;
        self.postfix_from(e)
    }

    fn postfix_from(&mut self, start: Expr) -> Result<Expr> {
        let mut e = start;
        loop {
            if self.at_sym(".") {
                let (line, col) = self.here();
                self.advance();
                let name = match self.peek().clone() {
                    Tok::Ident(s) => {
                        self.advance();
                        s
                    }
                    Tok::Int(i) if i >= 0 => {
                        self.advance();
                        e = Expr::new(ExprKind::TupleIndex { recv: Box::new(e), index: i as usize }, line, col);
                        continue;
                    }
                    _ => {
                        return Err(self
                            .err(format!("expected a method name after `.`, found {}", self.describe())))
                    }
                };
                self.reject_spaced_paren(&format!(".{}", name))?;
                let mut args = if self.at_sym("(") { self.call_args()? } else { Vec::new() };
                if let Some(block) = self.trailing_block()? {
                    if args.iter().any(|a| matches!(a.value.kind, ExprKind::Lambda { .. })) {
                        return Err(LumeError::new(block.line, block.col, format!("`{}` is given two blocks", name))
                            .with_help("use either `_` in the argument or a `{ |x| }` / `do |x|` block, not both"));
                    }
                    args.push(Arg { name: None, value: block });
                }
                e = Expr::new(ExprKind::Method { recv: Box::new(e), name, args }, line, col);
                continue;
            }
            if self.at_sym("?") && !self.toks[self.pos].space_before {
                let (line, col) = self.here();
                self.advance();
                e = Expr::new(ExprKind::Try(Box::new(e)), line, col);
                continue;
            }
            if self.at_sym("!") && !self.toks[self.pos].space_before {
                let (line, col) = self.here();
                self.advance();
                e = Expr::new(ExprKind::Unwrap(Box::new(e)), line, col);
                continue;
            }
            if self.at_sym("[") && !self.toks[self.pos].space_before {
                let (line, col) = self.here();
                self.advance();
                let index = self.expr()?;
                if self.eat_sym(",") {
                    return Err(self.err("indexing takes one key or position"));
                }
                self.expect_sym("]", "to close the index")?;
                e = Expr::new(ExprKind::Index { recv: Box::new(e), index: Box::new(index) }, line, col);
                continue;
            }
            break;
        }
        Ok(e)
    }

    fn call_args(&mut self) -> Result<Vec<Arg>> {
        self.expect_sym("(", "to start the argument list")?;
        let mut args: Vec<Arg> = Vec::new();
        while !self.at_sym(")") {
            // keyword argument: `name: value`
            let mut name = None;
            if let (Tok::Ident(n), Tok::Sym(":")) = (self.peek().clone(), self.peek_at(1).clone()) {
                if !lexer::is_keyword(&n) {
                    let (l, c) = self.here();
                    if args.iter().any(|a| a.name.as_deref() == Some(n.as_str())) {
                        return Err(LumeError::new(l, c, format!("argument `{}` is given twice", n)));
                    }
                    self.advance();
                    self.advance();
                    name = Some(n);
                }
            }
            if name.is_none() && args.iter().any(|a| a.name.is_some()) {
                return Err(self
                    .err("a positional argument cannot follow a keyword argument")
                    .with_help("put positional arguments first, or name this one too"));
            }
            let value = self.expr()?;
            let value = self.wrap_placeholder(value)?;
            args.push(Arg { name, value });
            if !self.eat_sym(",") {
                break;
            }
        }
        if !self.eat_sym(")") {
            return Err(self
                .err(format!("expected `,` or `)` in the argument list, found {}", self.describe())));
        }
        Ok(args)
    }

    /// An argument that mentions `_` becomes a one-parameter lambda. `_` may
    /// appear exactly once (principle: one obvious way; two uses get a name).
    fn wrap_placeholder(&self, value: Expr) -> Result<Expr> {
        let n = count_placeholders(&value);
        match n {
            0 => Ok(value),
            1 => {
                let (l, c) = (value.line, value.col);
                let mut value = value;
                replace_placeholders(&mut value);
                Ok(Expr::new(
                    ExprKind::Lambda { params: vec!["_".into()], body: Block { stmts: vec![Stmt::Expr(value)] } },
                    l,
                    c,
                ))
            }
            _ => Err(LumeError::new(value.line, value.col, "`_` may appear only once in a shorthand block")
                .with_help("name the argument instead: `{ |x| x.a + x.b }`")),
        }
    }

    /// `{ |a, b| expr }` on the same line, or `do |a, b|` followed by an
    /// indented body. Returns None when no block follows.
    fn trailing_block(&mut self) -> Result<Option<Expr>> {
        let (line, col) = self.here();
        if self.at_sym("{") {
            self.advance();
            let params = self.block_params()?;
            let body = self.expr()?;
            if self.at_sym(";") || matches!(self.peek(), Tok::Newline) {
                return Err(self
                    .err("an inline block holds a single expression")
                    .with_help("for several statements use `do |x|` and an indented body"));
            }
            if !self.eat_sym("}") {
                return Err(self.err(format!("expected `}}` to close the block, found {}", self.describe())));
            }
            let b = Block { stmts: vec![Stmt::Expr(body)] };
            self.mark_inline(&b);
            return Ok(Some(Expr::new(ExprKind::Lambda { params, body: b }, line, col)));
        }
        if self.at_kw("do") {
            self.advance();
            let params = self.block_params()?;
            let body = self.block()?;
            return Ok(Some(Expr::new(ExprKind::Lambda { params, body }, line, col)));
        }
        Ok(None)
    }

    fn block_params(&mut self) -> Result<Vec<String>> {
        if !self.eat_sym("|") {
            return Err(self
                .err(format!("expected `|x|` naming the block's argument, found {}", self.describe()))
                .with_help("write `{ |x| ... }` or `do |x|`; use `_` only in a bare argument like `.map(_.name)`"));
        }
        let mut params = Vec::new();
        loop {
            let (p, l, c) = self.ident("a block parameter")?;
            if params.contains(&p) {
                return Err(LumeError::new(l, c, format!("block parameter `{}` is listed twice", p)));
            }
            params.push(p);
            if !self.eat_sym(",") {
                break;
            }
        }
        if !self.eat_sym("|") {
            return Err(self.err(format!("expected `|` after the block parameters, found {}", self.describe())));
        }
        Ok(params)
    }

    fn primary(&mut self) -> Result<Expr> {
        let (line, col) = self.here();
        match self.peek().clone() {
            Tok::Int(v) => {
                self.advance();
                Ok(Expr::new(ExprKind::Int(v), line, col))
            }
            Tok::Float(v) => {
                self.advance();
                Ok(Expr::new(ExprKind::Float(v), line, col))
            }
            Tok::Str(parts) => {
                self.advance();
                let mut pieces = Vec::new();
                for p in parts {
                    match p {
                        StrPart::Lit(s) => pieces.push(StrPiece::Lit(s)),
                        StrPart::Expr(raw, ecol) => {
                            // `#{ x }` — spaces inside the braces are allowed
                            let lead = raw.len() - raw.trim_start().len();
                            let raw = raw.trim().to_string();
                            let ecol = ecol + lead;
                            let toks = lexer::lex(&raw).map_err(|e| {
                                LumeError::new(line, ecol + e.col - 1, format!("in interpolation: {}", e.msg))
                            })?;
                            let mut sub = Parser { toks, pos: 0, line_base: line - 1, col_base: ecol - 1, shape: Shape::default() };
                            let e = sub.expr()?;
                            sub.skip_newlines();
                            if !matches!(sub.peek(), Tok::Eof) {
                                return Err(sub.err("unexpected text after the interpolated expression"));
                            }
                            self.shape.trailing.extend(sub.shape.trailing);
                            self.shape.inline.extend(sub.shape.inline);
                            self.shape.pipes.extend(sub.shape.pipes);
                            pieces.push(StrPiece::Expr(e));
                        }
                    }
                }
                Ok(Expr::new(ExprKind::Str(pieces), line, col))
            }
            Tok::Sym("(") => {
                self.advance();
                let first = self.expr()?;
                if self.eat_sym(",") {
                    let mut items = vec![first];
                    while !self.at_sym(")") {
                        items.push(self.expr()?);
                        if !self.eat_sym(",") {
                            break;
                        }
                    }
                    self.expect_sym(")", "to close the tuple")?;
                    return Ok(Expr::new(ExprKind::Tuple(items), line, col));
                }
                self.expect_sym(")", "to close the parenthesis")?;
                Ok(first)
            }
            Tok::Sym("[") => {
                self.advance();
                let mut items = Vec::new();
                while !self.at_sym("]") {
                    items.push(self.expr()?);
                    if !self.eat_sym(",") {
                        break;
                    }
                }
                self.expect_sym("]", "to close the list")?;
                Ok(Expr::new(ExprKind::List(items), line, col))
            }
            Tok::Sym("{") => {
                if matches!(self.peek_at(1), Tok::Sym("|")) {
                    return Err(self.err("a `{ |x| ... }` block goes after a method call, like `xs.map { |x| x * 2 }`"));
                }
                self.advance();
                let mut pairs = Vec::new();
                while !self.at_sym("}") {
                    let k = self.expr()?;
                    if !self.eat_sym(":") {
                        return Err(self.err("expected `:` between a map key and its value").with_help("a map literal is `{key: value, ...}`; an empty map is `{}`"));
                    }
                    let v = self.expr()?;
                    pairs.push((k, v));
                    if !self.eat_sym(",") {
                        break;
                    }
                }
                self.expect_sym("}", "to close the map")?;
                Ok(Expr::new(ExprKind::MapLit(pairs), line, col))
            }
            Tok::Ident(s) if s == "_" => {
                self.advance();
                Ok(Expr::new(ExprKind::Placeholder, line, col))
            }
            Tok::Ident(s) => match s.as_str() {
                "true" => {
                    self.advance();
                    Ok(Expr::new(ExprKind::Bool(true), line, col))
                }
                "false" => {
                    self.advance();
                    Ok(Expr::new(ExprKind::Bool(false), line, col))
                }
                "puts" => {
                    self.advance();
                    self.reject_spaced_paren("puts")?;
                    let arg = if self.at_sym("(") {
                        let mut a = self.call_args()?;
                        if a.len() != 1 || a[0].name.is_some() {
                            return Err(LumeError::new(line, col, "`puts` takes exactly one value"));
                        }
                        a.remove(0).value
                    } else if matches!(self.peek(), Tok::Newline | Tok::Dedent | Tok::Eof) {
                        Expr::new(ExprKind::Str(vec![StrPiece::Lit(String::new())]), line, col)
                    } else {
                        self.expr()?
                    };
                    Ok(Expr::new(ExprKind::Puts(Box::new(arg)), line, col))
                }
                "if" => self.if_expr(),
                "nil" | "null" => Err(LumeError::new(line, col, format!("there is no `{}` in Lume", s))
                    .with_help("absence is `None`, and a value that may be absent has type `T?`")),
                "None" => {
                    self.advance();
                    if self.at_sym("(") {
                        return Err(self.err("`None` takes no value"));
                    }
                    Ok(Expr::new(ExprKind::None, line, col))
                }
                "rust" => {
                    self.advance();
                    if self.eat_sym(":") {
                        // the lexer captured the indented block as one string token
                        return match self.peek().clone() {
                            Tok::Str(parts) => {
                                self.advance();
                                let mut code = String::new();
                                for p in parts {
                                    if let StrPart::Lit(t) = p {
                                        code.push_str(&t);
                                    }
                                }
                                Ok(Expr::new(ExprKind::Rust(code), line, col))
                            }
                            _ => Err(self.err("`rust:` needs an indented block of Rust code on the following lines")),
                        };
                    }
                    self.reject_spaced_paren("rust")?;
                    if !self.eat_sym("(") {
                        return Err(self.err("expected `rust(\"...\")` or `rust:` followed by an indented block"));
                    }
                    let code = match self.peek().clone() {
                        Tok::Str(parts) => {
                            self.advance();
                            let mut code = String::new();
                            for p in parts {
                                match p {
                                    StrPart::Lit(t) => code.push_str(&t),
                                    StrPart::Expr(..) => return Err(self.err("Rust code cannot contain `#{}` interpolation; Lume names are visible inside it directly")),
                                }
                            }
                            code
                        }
                        _ => return Err(self.err("`rust(...)` takes one string of Rust code")),
                    };
                    self.expect_sym(")", "to close `rust(...)`")?;
                    Ok(Expr::new(ExprKind::Rust(code), line, col))
                }
                "Ok" => {
                    self.advance();
                    self.reject_spaced_paren("Ok")?;
                    let mut a = self.call_args()?;
                    if a.len() != 1 || a[0].name.is_some() {
                        return Err(LumeError::new(line, col, "`Ok` takes exactly one value"));
                    }
                    Ok(Expr::new(ExprKind::Ok(Box::new(a.remove(0).value)), line, col))
                }
                "Some" => {
                    self.advance();
                    self.reject_spaced_paren("Some")?;
                    let mut a = self.call_args()?;
                    if a.len() != 1 || a[0].name.is_some() {
                        return Err(LumeError::new(line, col, "`Some` takes exactly one value"));
                    }
                    Ok(Expr::new(ExprKind::Some(Box::new(a.remove(0).value)), line, col))
                }
                "match" => self.match_expr(),
                "self" => {
                    self.advance();
                    Ok(Expr::new(ExprKind::SelfRef, line, col))
                }
                "spawn" => {
                    self.advance();
                    if !self.eat_sym(":") {
                        return Err(self.err("expected `:` after `spawn`").with_help("write `t = spawn:` with the task's body indented below, or `spawn: expr`"));
                    }
                    let body = if matches!(self.peek(), Tok::Newline) {
                        self.block()?
                    } else {
                        let e = self.expr()?;
                        let b = Block { stmts: vec![Stmt::Expr(e)] };
                        self.mark_inline(&b);
                        b
                    };
                    Ok(Expr::new(ExprKind::Spawn(body), line, col))
                }
                "test" => Err(LumeError::new(line, col, "`test` blocks go at the top level of the file, not inside a function")),
                "assert" => Err(LumeError::new(line, col, "`assert` is a statement and starts its own line")),
                "interface" | "extend" | "import" => Err(LumeError::new(line, col, format!("`{}` goes at the top level of the file", s))),
                _ if lexer::is_keyword(&s) => {
                    Err(LumeError::new(line, col, format!("unexpected keyword `{}` here", s)))
                }
                _ => {
                    self.advance();
                    self.reject_spaced_paren(&s)?;
                    if self.at_sym("(") {
                        let args = self.call_args()?;
                        Ok(Expr::new(ExprKind::Call { name: s, args }, line, col))
                    } else {
                        Ok(Expr::new(ExprKind::Ident(s), line, col))
                    }
                }
            },
            Tok::Newline | Tok::Dedent | Tok::Eof => {
                Err(self.err("expected a value here, but the line ended"))
            }
            _ => Err(self.err(format!("expected a value, found {}", self.describe()))),
        }
    }
}

fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
        None => String::new(),
    }
}

/// Number of `_` placeholders in an expression, not descending into nested
/// lambdas (those own their own `_`).
fn count_placeholders(e: &Expr) -> usize {
    fn walk_block(b: &Block) -> usize {
        b.stmts
            .iter()
            .map(|s| match s {
                Stmt::Expr(e) => count_placeholders(e),
                Stmt::Bind { value, .. } | Stmt::Var { value, .. } | Stmt::OpAssign { value, .. } => count_placeholders(value),
                Stmt::Return { value: Some(e), .. } => count_placeholders(e),
                _ => 0,
            })
            .sum()
    }
    match &e.kind {
        ExprKind::Placeholder => 1,
        ExprKind::Lambda { .. } => 0,
        ExprKind::Int(_) | ExprKind::Float(_) | ExprKind::Bool(_) | ExprKind::Ident(_) | ExprKind::SelfRef => 0,
        ExprKind::Str(pieces) => pieces
            .iter()
            .map(|p| match p {
                StrPiece::Expr(e) => count_placeholders(e),
                _ => 0,
            })
            .sum(),
        ExprKind::List(items) => items.iter().map(count_placeholders).sum(),
        ExprKind::Range { lo, hi, .. } => count_placeholders(lo) + count_placeholders(hi),
        ExprKind::Unary { expr, .. } => count_placeholders(expr),
        ExprKind::Binary { lhs, rhs, .. } => count_placeholders(lhs) + count_placeholders(rhs),
        ExprKind::Call { args, .. } => args.iter().map(|a| count_placeholders(&a.value)).sum(),
        ExprKind::Method { recv, args, .. } => {
            count_placeholders(recv) + args.iter().map(|a| count_placeholders(&a.value)).sum::<usize>()
        }
        ExprKind::If { branches, else_block } => {
            branches.iter().map(|(c, b)| count_placeholders(c) + walk_block(b)).sum::<usize>()
                + else_block.as_ref().map(walk_block).unwrap_or(0)
        }
        ExprKind::Puts(e) => count_placeholders(e),
        ExprKind::Some(e) | ExprKind::Ok(e) | ExprKind::Try(e) | ExprKind::Unwrap(e) | ExprKind::TupleIndex { recv: e, .. } => count_placeholders(e),
        ExprKind::Tuple(items) => items.iter().map(count_placeholders).sum(),
        ExprKind::None | ExprKind::Rust(_) => 0,
        ExprKind::Index { recv, index } => count_placeholders(recv) + count_placeholders(index),
        ExprKind::MapLit(pairs) => pairs.iter().map(|(k, v)| count_placeholders(k) + count_placeholders(v)).sum(),
        ExprKind::Match { scrutinee, arms } => {
            count_placeholders(scrutinee)
                + arms.iter().map(|a| a.guard.as_ref().map(count_placeholders).unwrap_or(0) + walk_block(&a.body)).sum::<usize>()
        }
        ExprKind::Await(e) => count_placeholders(e),
        ExprKind::Spawn(b) => walk_block(b),
    }
}

/// Turns the single `_` into a reference to the lambda's parameter, also
/// named `_`, without descending into nested lambdas.
fn replace_placeholders(e: &mut Expr) {
    fn walk_block(b: &mut Block) {
        for s in &mut b.stmts {
            match s {
                Stmt::Expr(e) => replace_placeholders(e),
                Stmt::Bind { value, .. } | Stmt::Var { value, .. } | Stmt::OpAssign { value, .. } => replace_placeholders(value),
                Stmt::Return { value: Some(e), .. } => replace_placeholders(e),
                _ => {}
            }
        }
    }
    match &mut e.kind {
        ExprKind::Placeholder => e.kind = ExprKind::Ident("_".into()),
        ExprKind::Lambda { .. } => {}
        ExprKind::Str(pieces) => {
            for p in pieces {
                if let StrPiece::Expr(x) = p {
                    replace_placeholders(x);
                }
            }
        }
        ExprKind::List(items) => items.iter_mut().for_each(replace_placeholders),
        ExprKind::Range { lo, hi, .. } => {
            replace_placeholders(lo);
            replace_placeholders(hi);
        }
        ExprKind::Unary { expr, .. } => replace_placeholders(expr),
        ExprKind::Binary { lhs, rhs, .. } => {
            replace_placeholders(lhs);
            replace_placeholders(rhs);
        }
        ExprKind::Call { args, .. } => args.iter_mut().for_each(|a| replace_placeholders(&mut a.value)),
        ExprKind::Method { recv, args, .. } => {
            replace_placeholders(recv);
            args.iter_mut().for_each(|a| replace_placeholders(&mut a.value));
        }
        ExprKind::If { branches, else_block } => {
            for (c, b) in branches {
                replace_placeholders(c);
                walk_block(b);
            }
            if let Some(b) = else_block {
                walk_block(b);
            }
        }
        ExprKind::Puts(x) => replace_placeholders(x),
        ExprKind::Some(x) | ExprKind::Ok(x) | ExprKind::Try(x) | ExprKind::Unwrap(x) | ExprKind::TupleIndex { recv: x, .. } => replace_placeholders(x),
        ExprKind::Tuple(items) => items.iter_mut().for_each(replace_placeholders),
        ExprKind::Index { recv, index } => {
            replace_placeholders(recv);
            replace_placeholders(index);
        }
        ExprKind::MapLit(pairs) => {
            for (k, v) in pairs {
                replace_placeholders(k);
                replace_placeholders(v);
            }
        }
        ExprKind::Match { scrutinee, arms } => {
            replace_placeholders(scrutinee);
            for a in arms {
                if let Some(g) = &mut a.guard {
                    replace_placeholders(g);
                }
                walk_block(&mut a.body);
            }
        }
        ExprKind::Await(x) => replace_placeholders(x),
        ExprKind::Spawn(b) => walk_block(b),
        _ => {}
    }
}

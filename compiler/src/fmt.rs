//! `lume fmt`: prints the tree back out in the one canonical layout.
//!
//! The tree drops what does not matter to the compiler, so the formatter
//! leans on three extra sources: the `Shape` the parser records (which of
//! two equivalent spellings was written), the comments and blank lines the
//! lexer collects, and the source text itself for literals, which are copied
//! verbatim so escapes and digit separators survive.
//!
//! Rules: two-space indentation; one space around binary operators and after
//! commas and colons; no space inside brackets; `puts x` without parentheses;
//! one blank line between top-level items and between methods; other blank
//! lines kept where the source had at least one, never doubled; comments
//! stay on the line they were on; a list, map, tuple or argument list that
//! spanned several lines is printed one item per line with a trailing comma.

use std::collections::{BTreeSet, HashSet};

use crate::ast::*;
use crate::error::Result;
use crate::lexer::{self, Comment};
use crate::parser::{self, stmt_pos, Shape};

pub fn format_source(src: &str) -> Result<String> {
    let (toks, info) = lexer::lex_full(src)?;
    let code_lines: BTreeSet<usize> = toks.iter().filter(|t| !matches!(t.tok, lexer::Tok::Newline | lexer::Tok::Indent | lexer::Tok::Dedent | lexer::Tok::Eof)).map(|t| t.line).collect();
    let (items, shape) = parser::parse_program_shaped(toks)?;
    let mut f = Fmt {
        src_lines: src.lines().map(|l| l.chars().collect()).collect(),
        shape,
        comments: info.comments,
        ci: 0,
        blanks: info.blank_lines,
        code_lines,
        indent: 0,
        last_line: 0,
        suppress_blank: false,
    };
    let mut out = f.items_text(&items);
    // whatever comments remain sit after the last item
    out.push_str(&f.comments_before(usize::MAX));
    let mut out = align_trailing(&out);
    while out.ends_with("\n\n") {
        out.pop();
    }
    if !out.ends_with('\n') {
        out.push('\n');
    }
    Ok(out)
}

struct Fmt {
    src_lines: Vec<Vec<char>>,
    shape: Shape,
    comments: Vec<Comment>,
    ci: usize,
    blanks: HashSet<usize>,
    code_lines: BTreeSet<usize>,
    indent: usize,
    /// The last source line whose content has been printed.
    last_line: usize,
    /// True right after a block header: no blank line before the first statement.
    suppress_blank: bool,
}

fn ind(n: usize) -> String {
    "  ".repeat(n)
}

/// Placed before a trailing comment while formatting; `align_trailing`
/// turns runs of them into a column.
const MARK: char = '\u{1}';

/// Trailing comments on consecutive lines line up, two spaces past the
/// longest code in the run (a line without one ends the run).
fn align_trailing(text: &str) -> String {
    let lines: Vec<&str> = text.split('\n').collect();
    let mut out: Vec<String> = Vec::with_capacity(lines.len());
    let mut i = 0;
    while i < lines.len() {
        if !lines[i].contains(MARK) {
            out.push(lines[i].to_string());
            i += 1;
            continue;
        }
        let mut j = i;
        while j < lines.len() && lines[j].contains(MARK) {
            j += 1;
        }
        let width = lines[i..j].iter().map(|l| l.split(MARK).next().unwrap().chars().count()).max().unwrap_or(0);
        for l in &lines[i..j] {
            let mut parts = l.splitn(2, MARK);
            let code = parts.next().unwrap();
            let comment = parts.next().unwrap_or("");
            let pad = width - code.chars().count() + 2;
            out.push(format!("{}{}{}", code, " ".repeat(pad), comment));
        }
        i = j;
    }
    out.join("\n")
}

/// The same rule for arrows in a `match`: consecutive one-line arms line up.
fn align_arrows(arms: &mut [(String, Option<String>)]) {
    // arms: (head, Some(inline body)) or (head, None) for a block arm
    let mut i = 0;
    while i < arms.len() {
        if arms[i].1.is_none() {
            i += 1;
            continue;
        }
        let mut j = i;
        while j < arms.len() && arms[j].1.is_some() {
            j += 1;
        }
        // a guard makes a head long; it does not set the column for the others
        let plain = arms[i..j].iter().filter(|(h, _)| !h.contains(" if ")).map(|(h, _)| h.chars().count()).max();
        let width = plain.unwrap_or_else(|| arms[i..j].iter().map(|(h, _)| h.chars().count()).max().unwrap_or(0));
        for (h, _) in &mut arms[i..j] {
            let pad = width.saturating_sub(h.chars().count());
            h.push_str(&" ".repeat(pad));
        }
        i = j;
    }
}

impl Fmt {
    // ----- comments and blank lines ----------------------------------------

    fn blank_before(&mut self, line: usize) -> String {
        if self.suppress_blank {
            self.suppress_blank = false;
            return String::new();
        }
        if self.last_line > 0 && line > self.last_line + 1 && (self.last_line + 1..line).any(|l| self.blanks.contains(&l)) {
            "\n".into()
        } else {
            String::new()
        }
    }

    /// Comments that sit on their own lines before `line`, at the current indent.
    fn comments_before(&mut self, line: usize) -> String {
        let mut out = String::new();
        while self.ci < self.comments.len() && self.comments[self.ci].line < line {
            let c = self.comments[self.ci].clone();
            out.push_str(&self.blank_before(c.line));
            out.push_str(&format!("{}#{}\n", ind(self.indent), c.text));
            self.last_line = c.line;
            self.ci += 1;
        }
        out
    }

    /// The comment at the end of `line`, if any, ready to append.
    fn take_trailing(&mut self, line: usize) -> String {
        if self.ci < self.comments.len() && self.comments[self.ci].line == line {
            let c = &self.comments[self.ci];
            self.ci += 1;
            format!("{}#{}", MARK, c.text)
        } else {
            String::new()
        }
    }

    /// Comments left at the end of a block: those indented at least as far
    /// as the block's statements, before the next line of code.
    fn tail_comments(&mut self, body_col: usize) -> String {
        let next = self.code_lines.range(self.last_line + 1..).next().copied().unwrap_or(usize::MAX);
        let mut out = String::new();
        while self.ci < self.comments.len() && self.comments[self.ci].line < next && self.comments[self.ci].col >= body_col {
            let c = self.comments[self.ci].clone();
            out.push_str(&self.blank_before(c.line));
            out.push_str(&format!("{}#{}\n", ind(self.indent), c.text));
            self.last_line = c.line;
            self.ci += 1;
        }
        out
    }

    /// A statement or item line: indentation, the text (whose later lines
    /// are already indented), the trailing comment after its first line.
    fn line_with(&self, text: &str, trailing: &str) -> String {
        let mut out = ind(self.indent);
        match text.find('\n') {
            Some(i) => {
                out.push_str(&text[..i]);
                out.push_str(trailing);
                out.push_str(&text[i..]);
            }
            None => {
                out.push_str(text);
                out.push_str(trailing);
            }
        }
        out.push('\n');
        out
    }

    // ----- source text ------------------------------------------------------

    /// The literal at (line, col) as written: a number (with `_` and `.`), or
    /// a string with its escapes and interpolations intact.
    fn raw_number(&self, line: usize, col: usize) -> Option<String> {
        let l = self.src_lines.get(line.checked_sub(1)?)?;
        let mut i = col.checked_sub(1)?;
        if i >= l.len() || !l[i].is_ascii_digit() {
            return None;
        }
        let start = i;
        while i < l.len() {
            let c = l[i];
            if c.is_ascii_digit() || c == '_' {
                i += 1;
            } else if c == '.' && i + 1 < l.len() && l[i + 1].is_ascii_digit() && !(i > start && l[i - 1] == '.') {
                i += 1;
            } else if (c == 'e' || c == 'E') && i + 1 < l.len() && (l[i + 1].is_ascii_digit() || ((l[i + 1] == '+' || l[i + 1] == '-') && i + 2 < l.len() && l[i + 2].is_ascii_digit())) {
                // an exponent: `1e15`, `2.5e-3`
                i += if l[i + 1].is_ascii_digit() { 1 } else { 2 };
            } else {
                break;
            }
        }
        Some(l[start..i].iter().collect())
    }

    fn raw_string(&self, line: usize, col: usize) -> Option<String> {
        let l = self.src_lines.get(line.checked_sub(1)?)?;
        let mut i = col.checked_sub(1)?;
        // `r"..."` and `r"""..."""`: copied through as written
        if i < l.len() && l[i] == 'r' && i + 1 < l.len() && l[i + 1] == '"' {
            let triple = i + 3 < l.len() && l[i + 2] == '"' && l[i + 3] == '"';
            let quote: &str = if triple { "\"\"\"" } else { "\"" };
            let mut out: String = l[i..].iter().collect();
            let mut ln = line - 1;
            while out[1 + quote.len()..].find(quote).is_none() {
                ln += 1;
                let next = self.src_lines.get(ln)?;
                out.push('\n');
                out.push_str(&next.iter().collect::<String>());
            }
            let end = out[1 + quote.len()..].find(quote)? + 1 + 2 * quote.len();
            return Some(out[..end].to_string());
        }
        if i >= l.len() || l[i] != '"' {
            return None;
        }
        // a `"""` block is copied through to its closing quotes, lines and all
        if i + 2 < l.len() && l[i + 1] == '"' && l[i + 2] == '"' {
            let mut out: String = l[i..].iter().collect();
            let mut ln = line - 1;
            let closes = |s: &str| s[3..].contains("\"\"\"");
            if !closes(&out) {
                loop {
                    ln += 1;
                    let next = self.src_lines.get(ln)?;
                    out.push('\n');
                    out.push_str(&next.iter().collect::<String>());
                    if next.iter().collect::<String>().contains("\"\"\"") {
                        break;
                    }
                }
            }
            let end = out[3..].find("\"\"\"")? + 6;
            return Some(out[..end].to_string());
        }
        let start = i;
        i += 1;
        while i < l.len() {
            match l[i] {
                '\\' => i += 2,
                '"' => return Some(l[start..=i].iter().collect()),
                '#' if i + 1 < l.len() && l[i + 1] == '{' => {
                    let mut depth = 0;
                    while i < l.len() {
                        if l[i] == '{' {
                            depth += 1;
                        } else if l[i] == '}' {
                            depth -= 1;
                            if depth == 0 {
                                break;
                            }
                        }
                        i += 1;
                    }
                    i += 1;
                }
                _ => i += 1,
            }
        }
        None
    }

    fn escape(s: &str) -> String {
        let mut out = String::new();
        let chars: Vec<char> = s.chars().collect();
        for (i, c) in chars.iter().enumerate() {
            match c {
                '\\' => out.push_str("\\\\"),
                '"' => out.push_str("\\\""),
                '\n' => out.push_str("\\n"),
                '\t' => out.push_str("\\t"),
                '\r' => out.push_str("\\r"),
                '\0' => out.push_str("\\0"),
                '#' if chars.get(i + 1) == Some(&'{') => out.push_str("\\#"),
                c if (*c as u32) < 0x20 || *c == '\u{7f}' => out.push_str(&format!("\\u{{{:x}}}", *c as u32)),
                c => out.push(*c),
            }
        }
        out
    }

    // ----- items ----------------------------------------------------------------

    fn item_line(item: &Item) -> usize {
        match item {
            Item::Fn(f) => f.line,
            Item::Const(c) => c.line,
            Item::Struct(s) => s.line,
            Item::Enum(e) => e.line,
            Item::Import(i) => i.line,
            Item::Interface(i) => i.line,
            Item::Extend(x) => x.line,
            Item::Test(t) => t.line,
        }
    }

    fn items_text(&mut self, items: &[Item]) -> String {
        let mut out = String::new();
        let mut prev_multiline: Option<bool> = None;
        for item in items {
            let line = Self::item_line(item);
            // Render first so we know whether this item spans lines; comments
            // that introduce it are collected before that, in source order.
            let intro_at = out.len();
            out.push_str(&self.comments_before(line));
            let blank = self.blank_before(line);
            let trailing = self.take_trailing(line);
            let text = self.item_text(item);
            let multiline = text.contains('\n');
            // one blank line between items, unless both are one-liners (imports, short defs)
            let force = matches!(prev_multiline, Some(p) if p || multiline);
            if force {
                let intro = out.split_off(intro_at);
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(intro.trim_start_matches('\n'));
                // a blank the programmer left between the comment and the item stays
                if !intro.trim().is_empty() {
                    out.push_str(&blank);
                }
            } else {
                out.push_str(&blank);
            }
            out.push_str(&self.line_with(&text, &trailing));
            self.last_line = self.last_line.max(line);
            prev_multiline = Some(multiline);
        }
        out
    }

    fn item_text(&mut self, item: &Item) -> String {
        match item {
            Item::Const(c) => {
                let pubkw = if c.public { "pub " } else { "" };
                match &c.ty {
                    Some(t) => format!("{}{}: {} = {}", pubkw, c.name, type_str(t), self.expr(&c.value)),
                    None => format!("{}{} = {}", pubkw, c.name, self.expr(&c.value)),
                }
            }
            Item::Fn(f) => self.fn_text(f, false),
            Item::Struct(s) => {
                let mut out = format!("{}struct {}{}:", if s.public { "pub " } else { "" }, s.name, generics_str(&s.generics));
                self.indent += 1;
                out.push_str(&self.members_text(&s.fields, &[], &s.methods));
                self.indent -= 1;
                out
            }
            Item::Enum(e) => {
                let mut out = format!("{}enum {}{}:", if e.public { "pub " } else { "" }, e.name, generics_str(&e.generics));
                self.indent += 1;
                out.push_str(&self.members_text(&[], &e.variants, &e.methods));
                self.indent -= 1;
                out
            }
            Item::Interface(i) => {
                let mut out = format!("{}interface {}{}:", if i.public { "pub " } else { "" }, i.name, generics_str(&i.generics));
                self.indent += 1;
                let mut all: Vec<(&FnDef, bool)> = i.required.iter().map(|f| (f, true)).chain(i.defaults.iter().map(|f| (f, false))).collect();
                all.sort_by_key(|(f, _)| (f.line, f.col));
                out.push_str(&self.defs_text(&all, true));
                self.indent -= 1;
                out
            }
            Item::Extend(x) => {
                let mut out = format!("extend {} with {}:", type_str(&x.target), type_str(&x.iface));
                self.indent += 1;
                let all: Vec<(&FnDef, bool)> = x.methods.iter().map(|f| (f, false)).collect();
                out.push_str(&self.defs_text(&all, true));
                self.indent -= 1;
                out
            }
            Item::Import(imp) => {
                let mut out = String::from("import ");
                if imp.is_rust {
                    out.push_str("rust.");
                }
                out.push_str(&imp.path.join("."));
                if let Some(v) = &imp.version {
                    out.push_str(&format!(" = \"{}\"", Self::escape(v)));
                }
                if let Some(a) = &imp.alias {
                    out.push_str(&format!(" as {}", a));
                }
                out
            }
            Item::Test(t) => {
                let mut out = format!("test \"{}\":", Self::escape(&t.name));
                self.indent += 1;
                out.push_str(&self.block_text(&t.body));
                self.indent -= 1;
                out
            }
        }
    }

    /// Fields (or variants), then methods, inside a struct or enum body.
    fn members_text(&mut self, fields: &[Param], variants: &[Variant], methods: &[FnDef]) -> String {
        let mut out = String::new();
        self.suppress_blank = true;
        let mut first_col = None;
        for f in fields {
            first_col.get_or_insert(f.col);
            out.push_str(&self.comments_before(f.line));
            out.push_str(&self.blank_before(f.line));
            let trailing = self.take_trailing(f.line);
            out.push_str(&self.line_with(&format!("{}: {}", f.name, type_str(&f.ty)), &trailing));
            self.last_line = self.last_line.max(f.line);
        }
        for v in variants {
            first_col.get_or_insert(v.col);
            out.push_str(&self.comments_before(v.line));
            out.push_str(&self.blank_before(v.line));
            let trailing = self.take_trailing(v.line);
            let text = if v.fields.is_empty() {
                v.name.clone()
            } else {
                let fs: Vec<String> = v.fields.iter().map(|f| format!("{}: {}", f.name, type_str(&f.ty))).collect();
                format!("{}({})", v.name, fs.join(", "))
            };
            out.push_str(&self.line_with(&text, &trailing));
            self.last_line = self.last_line.max(v.line);
        }
        let all: Vec<(&FnDef, bool)> = methods.iter().map(|f| (f, false)).collect();
        if !all.is_empty() {
            out.push('\n');
            self.suppress_blank = true;
            out.push_str(&self.defs_text(&all, false));
        }
        let body_col = first_col.or_else(|| methods.first().map(|m| m.col)).unwrap_or(1);
        out.push_str(&self.tail_comments(body_col));
        format!("\n{}", out.trim_end_matches('\n'))
    }

    /// Methods of a struct/enum/interface/extend, one blank line apart.
    /// `(def, is_signature_only)`.
    fn defs_text(&mut self, defs: &[(&FnDef, bool)], leading: bool) -> String {
        let mut out = String::new();
        if leading {
            self.suppress_blank = true;
        }
        let mut prev_multiline: Option<bool> = None;
        for (f, sig_only) in defs.iter() {
            let intro_at = out.len();
            out.push_str(&self.comments_before(f.line));
            let blank = self.blank_before(f.line);
            let trailing = self.take_trailing(f.line);
            let text = self.fn_text(f, *sig_only);
            let multiline = text.contains('\n');
            let force = matches!(prev_multiline, Some(p) if p || multiline);
            if force {
                let intro = out.split_off(intro_at);
                if !out.is_empty() {
                    out.push('\n');
                }
                out.push_str(intro.trim_start_matches('\n'));
                // a blank the programmer left between the comment and the item stays
                if !intro.trim().is_empty() {
                    out.push_str(&blank);
                }
            } else {
                out.push_str(&blank);
            }
            out.push_str(&self.line_with(&text, &trailing));
            self.last_line = self.last_line.max(f.line);
            prev_multiline = Some(multiline);
        }
        if leading {
            let body_col = defs.first().map(|(f, _)| f.col).unwrap_or(1);
            out.push_str(&self.tail_comments(body_col));
            format!("\n{}", out.trim_end_matches('\n'))
        } else {
            out
        }
    }

    fn fn_text(&mut self, f: &FnDef, sig_only: bool) -> String {
        let mut out = String::new();
        if f.public && !sig_only {
            out.push_str("pub ");
        }
        if f.is_async {
            out.push_str("async ");
        }
        out.push_str("def ");
        out.push_str(&f.name);
        out.push_str(&generics_str(&f.generics));
        let mut params: Vec<String> = Vec::new();
        if f.self_kind == SelfKind::Mutate {
            params.push("var self".into());
        }
        for p in &f.params {
            params.push(format!("{}{}: {}", if p.mutable { "var " } else { "" }, p.name, type_str(&p.ty)));
        }
        if !params.is_empty() {
            out.push_str(&format!("({})", params.join(", ")));
        }
        if let Some(r) = &f.ret {
            out.push_str(&format!(" -> {}", type_str(r)));
        }
        if sig_only {
            return out;
        }
        if self.is_inline(&f.body) {
            if let Some(Stmt::Expr(e)) = f.body.stmts.first() {
                let v = self.expr(e);
                if v.contains('\n') || ind(self.indent).len() + out.len() + 3 + v.chars().count() <= 100 {
                    out.push_str(" = ");
                    out.push_str(&v);
                } else {
                    out.push_str(" =\n");
                    out.push_str(&ind(self.indent + 1));
                    out.push_str(&v);
                }
                return out;
            }
        }
        out.push(':');
        self.indent += 1;
        out.push_str(&self.block_text(&f.body));
        self.indent -= 1;
        out
    }

    fn is_inline(&self, b: &Block) -> bool {
        b.stmts.len() == 1 && self.shape.inline.contains(&stmt_pos(&b.stmts[0]))
    }

    // ----- blocks and statements ----------------------------------------------

    /// An indented block, starting with a newline; `self.indent` is already
    /// the block's depth. No trailing newline.
    fn block_text(&mut self, b: &Block) -> String {
        let mut out = String::from("\n");
        self.suppress_blank = true;
        for st in &b.stmts {
            let (line, _) = stmt_pos(st);
            out.push_str(&self.comments_before(line));
            out.push_str(&self.blank_before(line));
            let trailing = self.take_trailing(line);
            let text = self.stmt_text(st);
            out.push_str(&self.line_with(&text, &trailing));
            self.last_line = self.last_line.max(line);
        }
        if let Some(st) = b.stmts.first() {
            out.push_str(&self.tail_comments(stmt_pos(st).1));
        }
        out.trim_end_matches('\n').to_string()
    }

    fn stmt_text(&mut self, s: &Stmt) -> String {
        match s {
            Stmt::Destructure { names, value, .. } => format!("({}) = {}", names.join(", "), self.expr(value)),
            Stmt::Bind { name, ty, value, .. } => match ty {
                Some(t) => format!("{}: {} = {}", name, type_str(t), self.expr(value)),
                None => format!("{} = {}", name, self.expr(value)),
            },
            Stmt::Var { name, ty, value, .. } => match ty {
                Some(t) => format!("var {}: {} = {}", name, type_str(t), self.expr(value)),
                None => format!("var {} = {}", name, self.expr(value)),
            },
            Stmt::OpAssign { name, op, value, .. } => format!("{} {} {}", name, op, self.expr(value)),
            Stmt::FieldAssign { recv, field, op, value, .. } => {
                format!("{}.{} {} {}", self.expr_p(recv, 10), field, op.unwrap_or("="), self.expr(value))
            }
            Stmt::IndexAssign { recv, index, op, value, .. } => {
                format!("{}[{}] {} {}", self.expr_p(recv, 10), self.expr(index), op.unwrap_or("="), self.expr(value))
            }
            Stmt::Return { value, .. } => match value {
                Some(v) => format!("return {}", self.expr(v)),
                None => "return".into(),
            },
            Stmt::Break { .. } => "break".into(),
            Stmt::Next { .. } => "next".into(),
            Stmt::Assert { cond, .. } => format!("assert {}", self.expr(cond)),
            Stmt::Shared { name, mutable, ty, value, .. } => {
                let kw = if *mutable { "shared var" } else { "shared" };
                match ty {
                    Some(t) => format!("{} {}: {} = {}", kw, name, type_str(t), self.expr(value)),
                    None => format!("{} {} = {}", kw, name, self.expr(value)),
                }
            }
            Stmt::While { cond, body } => {
                let mut out = format!("while {}:", self.expr(cond));
                self.indent += 1;
                out.push_str(&self.block_text(body));
                self.indent -= 1;
                out
            }
            Stmt::For { vars, mutable, iter, filter, body, .. } => {
                let mut out = format!("for {}{} in {}", if *mutable { "var " } else { "" }, vars.join(", "), self.expr(iter));
                if let Some(f) = filter {
                    out.push_str(&format!(" where {}", self.expr(f)));
                }
                out.push(':');
                self.indent += 1;
                out.push_str(&self.block_text(body));
                self.indent -= 1;
                out
            }
            Stmt::Expr(e) => {
                if let ExprKind::If { branches, else_block: None } = &e.kind {
                    if let Some(unless) = self.shape.trailing.get(&(e.line, e.col)).copied() {
                        let (cond, body) = &branches[0];
                        let inner = self.stmt_text(&body.stmts[0]);
                        let c = if unless {
                            match &cond.kind {
                                ExprKind::Unary { op: "not", expr } => self.expr(expr),
                                _ => self.expr(cond),
                            }
                        } else {
                            self.expr(cond)
                        };
                        return format!("{} {} {}", inner, if unless { "unless" } else { "if" }, c);
                    }
                }
                self.expr(e)
            }
        }
    }

    // ----- expressions ----------------------------------------------------------

    fn is_pipe(&self, e: &Expr) -> bool {
        matches!(e.kind, ExprKind::Method { .. } | ExprKind::Call { .. } | ExprKind::Puts(_)) && self.shape.pipes.contains_key(&(e.line, e.col))
    }

    /// Binding strength, for deciding where parentheses are needed.
    fn prec(&self, e: &Expr) -> u8 {
        if self.is_pipe(e) {
            return 0;
        }
        match &e.kind {
            ExprKind::Binary { op, .. } => match *op {
                "or" => 1,
                "and" => 2,
                "==" | "!=" | "<" | "<=" | ">" | ">=" => 4,
                "+" | "-" => 6,
                "*" | "/" | "%" => 7,
                "**" => 9,
                _ => 6,
            },
            ExprKind::Range { .. } => 5,
            ExprKind::Unary { op, .. } => {
                if *op == "not" {
                    3
                } else {
                    8
                }
            }
            ExprKind::If { .. } | ExprKind::Match { .. } | ExprKind::Puts(_) | ExprKind::Warn(_) | ExprKind::Spawn(_) => 0,
            ExprKind::Await(_) => 8,
            _ => 10,
        }
    }

    fn expr_p(&mut self, e: &Expr, min: u8) -> String {
        let s = self.expr(e);
        if self.prec(e) < min {
            format!("({})", s)
        } else {
            s
        }
    }

    /// True when the items of a bracketed list were spread over several lines.
    fn spread(first: Option<&Expr>, last: Option<&Expr>) -> bool {
        match (first, last) {
            (Some(a), Some(b)) => a.line != b.line,
            _ => false,
        }
    }

    /// `[a, b]` on one line, or one item per line when the source did that.
    fn bracketed(&mut self, open: &str, close: &str, items: Vec<String>, multi: bool) -> String {
        self.bracketed_at(open, close, items.into_iter().map(|i| (i, 0)).collect(), multi)
    }

    /// The same, with each item's source line, so a comment written after an
    /// item stays with that item instead of drifting past the closing bracket.
    fn bracketed_at(&mut self, open: &str, close: &str, items: Vec<(String, usize)>, multi: bool) -> String {
        if !multi || items.is_empty() {
            return format!("{}{}{}", open, items.into_iter().map(|(i, _)| i).collect::<Vec<_>>().join(", "), close);
        }
        let inner = ind(self.indent + 1);
        let mut out = format!("{}\n", open);
        for (it, line) in items {
            let trailing = if line > 0 { self.take_trailing(line) } else { String::new() };
            out.push_str(&format!("{}{},{}\n", inner, it, trailing));
        }
        out.push_str(&format!("{}{}", ind(self.indent), close));
        out
    }

    fn args_text(&mut self, args: &[Arg]) -> String {
        // a trailing `do` block or `{ |x| }` block goes outside the parentheses
        let (inner, block): (&[Arg], Option<&Arg>) = match args.last() {
            Some(a) if a.name.is_none() && self.is_block_lambda(&a.value) => (&args[..args.len() - 1], Some(a)),
            _ => (args, None),
        };
        let multi = inner.len() >= 2 && Self::spread(inner.first().map(|a| &a.value), inner.last().map(|a| &a.value));
        let mut out = String::new();
        if !inner.is_empty() {
            if multi {
                self.indent += 1;
            }
            let texts: Vec<String> = inner
                .iter()
                .map(|a| {
                    let v = self.expr(&a.value);
                    match &a.name {
                        Some(n) => format!("{}: {}", n, v),
                        None => v,
                    }
                })
                .collect();
            if multi {
                self.indent -= 1;
            }
            out.push_str(&self.bracketed("(", ")", texts, multi));
        }
        if let Some(b) = block {
            out.push(' ');
            out.push_str(&self.lambda_text(&b.value));
        }
        out
    }

    /// A `{ |x| ... }` or `do |x|` block (not a `_` shorthand).
    fn is_block_lambda(&self, e: &Expr) -> bool {
        matches!(&e.kind, ExprKind::Lambda { params, .. } if params.as_slice() != ["_"])
    }

    fn lambda_text(&mut self, e: &Expr) -> String {
        if let ExprKind::Lambda { params, body } = &e.kind {
            if params.as_slice() == ["_"] {
                if let Some(Stmt::Expr(x)) = body.stmts.first() {
                    return self.expr(x);
                }
            }
            // a block that takes nothing has no `|...|` at all
            let pipes = if params.is_empty() { String::new() } else { format!("|{}| ", params.join(", ")) };
            if self.is_inline(body) {
                if let Some(st) = body.stmts.first() {
                    // `stmt_text` keeps a trailing `if`/`unless` on one line
                    let text = self.stmt_text(st);
                    return format!("{{ {}{} }}", pipes, text);
                }
            }
            let mut out = if params.is_empty() { "do".to_string() } else { format!("do |{}|", params.join(", ")) };
            self.indent += 1;
            out.push_str(&self.block_text(body));
            self.indent -= 1;
            return out;
        }
        unreachable!()
    }

    /// The value on the left of `|>`: a pipe itself, a name, a call, or
    /// anything else in parentheses (`(1..10) |> .map(...)`).
    fn pipe_lhs(&mut self, e: &Expr) -> String {
        if self.is_pipe(e) {
            self.expr(e)
        } else {
            self.expr_p(e, 10)
        }
    }

    fn pipe_prefix(&self, e: &Expr, lhs: String) -> String {
        let newline = self.shape.pipes.get(&(e.line, e.col)).copied().unwrap_or(false);
        if newline {
            format!("{}\n{}|> ", lhs, ind(self.indent + 1))
        } else {
            format!("{} |> ", lhs)
        }
    }

    fn expr(&mut self, e: &Expr) -> String {
        match &e.kind {
            ExprKind::Int(v) => self.raw_number(e.line, e.col).unwrap_or_else(|| v.to_string()),
            ExprKind::Float(v) => self.raw_number(e.line, e.col).unwrap_or_else(|| format!("{:?}", v)),
            ExprKind::Bool(b) => b.to_string(),
            ExprKind::Str(pieces) => {
                if let Some(raw) = self.raw_string(e.line, e.col) {
                    return raw;
                }
                let mut out = String::from("\"");
                for p in pieces {
                    match p {
                        StrPiece::Lit(s) => out.push_str(&Self::escape(s)),
                        StrPiece::Expr(x) => out.push_str(&format!("#{{{}}}", self.expr(x))),
                    }
                }
                out.push('"');
                out
            }
            ExprKind::Ident(n) => n.clone(),
            ExprKind::SelfRef => "self".into(),
            ExprKind::Placeholder => "_".into(),
            ExprKind::None => "None".into(),
            ExprKind::List(items) => {
                let multi = items.len() >= 2 && Self::spread(items.first(), items.last());
                if multi {
                    self.indent += 1;
                }
                let texts: Vec<(String, usize)> = items.iter().map(|i| (self.expr(i), i.line)).collect();
                if multi {
                    self.indent -= 1;
                }
                self.bracketed_at("[", "]", texts, multi)
            }
            ExprKind::Tuple(items) => {
                let texts: Vec<String> = items.iter().map(|i| self.expr(i)).collect();
                format!("({})", texts.join(", "))
            }
            ExprKind::SetLit(items) => {
                let multi = items.len() >= 2 && Self::spread(items.first(), items.last());
                if multi {
                    self.indent += 1;
                }
                let texts: Vec<(String, usize)> = items.iter().map(|i| (self.expr(i), i.line)).collect();
                if multi {
                    self.indent -= 1;
                }
                self.bracketed_at("{", "}", texts, multi)
            }
            ExprKind::MapLit(pairs) => {
                let multi = pairs.len() >= 2 && Self::spread(pairs.first().map(|p| &p.0), pairs.last().map(|p| &p.0));
                if multi {
                    self.indent += 1;
                }
                let texts: Vec<(String, usize)> = pairs.iter().map(|(k, v)| (format!("{}: {}", self.expr(k), self.expr(v)), k.line)).collect();
                if multi {
                    self.indent -= 1;
                }
                self.bracketed_at("{", "}", texts, multi)
            }
            ExprKind::Range { lo, hi, inclusive } => {
                format!("{}{}{}", self.expr_p(lo, 6), if *inclusive { ".." } else { "..." }, self.expr_p(hi, 6))
            }
            ExprKind::Unary { op, expr } => {
                if *op == "not" {
                    format!("not {}", self.expr_p(expr, 3))
                } else {
                    format!("-{}", self.expr_p(expr, 8))
                }
            }
            ExprKind::Binary { op, lhs, rhs } => {
                let p = self.prec(e);
                let right_assoc = *op == "**";
                let l = self.expr_p(lhs, if right_assoc { p + 1 } else { p });
                let r = self.expr_p(rhs, if right_assoc { p } else { p + 1 });
                format!("{} {} {}", l, op, r)
            }
            ExprKind::Call { name, args } => {
                if self.is_pipe(e) {
                    let lhs = self.pipe_lhs(&args[0].value);
                    let rest = &args[1..];
                    let tail = if rest.is_empty() { String::new() } else { self.args_text(rest) };
                    return format!("{}{}{}", self.pipe_prefix(e, lhs), name, tail);
                }
                format!("{}{}", name, if args.is_empty() { "()".to_string() } else { self.args_text(args) })
            }
            ExprKind::Method { recv, name, args } => {
                let head = if self.is_pipe(e) {
                    let lhs = self.pipe_lhs(recv);
                    format!("{}.{}", self.pipe_prefix(e, lhs), name)
                } else {
                    format!("{}.{}", self.expr_p(recv, 10), name)
                };
                format!("{}{}", head, self.args_text(args))
            }
            ExprKind::Warn(x) => {
                let v = self.expr(x);
                if v.starts_with('(') { format!("warn({})", v) } else { format!("warn {}", v) }
            }
            ExprKind::Puts(x) => {
                if self.is_pipe(e) {
                    let lhs = self.pipe_lhs(x);
                    return format!("{}puts", self.pipe_prefix(e, lhs));
                }
                if matches!(&x.kind, ExprKind::Str(p) if p.len() == 1 && matches!(&p[0], StrPiece::Lit(s) if s.is_empty())) {
                    return "puts".into();
                }
                let v = self.expr(x);
                // `puts (a + b) * 2` would be ambiguous; the strict-parens rule wants `puts((a + b) * 2)`
                if v.starts_with('(') {
                    format!("puts({})", v)
                } else {
                    format!("puts {}", v)
                }
            }
            ExprKind::Lambda { .. } => self.lambda_text(e),
            ExprKind::TupleIndex { recv, index } => format!("{}.{}", self.expr_p(recv, 10), index),
            ExprKind::Some(x) => format!("Some({})", self.expr(x)),
            ExprKind::Ok(x) => format!("Ok({})", self.expr(x)),
            ExprKind::Try(x) => match &x.kind {
                // `p.next()?`, not `p.next?`: the latter reads as a predicate
                ExprKind::Method { recv, name, args } if args.is_empty() && !name.ends_with('?') && !self.is_pipe(x) => {
                    format!("{}.{}()?", self.expr_p(recv, 10), name)
                }
                _ => format!("{}?", self.expr_p(x, 10)),
            },
            ExprKind::Unwrap(x) => format!("{}!", self.expr_p(x, 10)),
            ExprKind::Index { recv, index } => format!("{}[{}]", self.expr_p(recv, 10), self.expr(index)),
            ExprKind::Rust(code) => {
                let was_inline = self.shape.rust_inline.contains(&(e.line, e.col));
                if was_inline && !code.contains('\n') && !code.contains('"') && !code.contains('\\') {
                    return format!("rust(\"{}\")", code);
                }
                let inner = ind(self.indent + 1);
                let mut out = String::from("rust:");
                for l in code.lines() {
                    if l.trim().is_empty() {
                        out.push('\n');
                    } else {
                        out.push_str(&format!("\n{}{}", inner, l));
                    }
                }
                out
            }
            ExprKind::Await(x) => format!("await {}", self.expr_p(x, 10)),
            ExprKind::Spawn(body) => {
                if self.is_inline(body) {
                    if let Some(Stmt::Expr(x)) = body.stmts.first() {
                        return format!("spawn: {}", self.expr(x));
                    }
                }
                let mut out = String::from("spawn:");
                self.indent += 1;
                out.push_str(&self.block_text(body));
                self.indent -= 1;
                out
            }
            ExprKind::If { branches, else_block } => self.if_text(branches, else_block.as_ref()),
            ExprKind::Match { scrutinee, arms } => {
                let mut out = format!("match {}:", self.expr(scrutinee));
                self.indent += 1;
                self.suppress_blank = true;
                // (intro comments + blank, head, inline body or block text, trailing)
                let mut rows: Vec<(String, (String, Option<String>), String, String)> = Vec::new();
                for arm in arms {
                    let mut intro = self.comments_before(arm.line);
                    intro.push_str(&self.blank_before(arm.line));
                    let trailing = self.take_trailing(arm.line);
                    let mut head = pattern_str(&arm.pat);
                    if let Some(g) = &arm.guard {
                        head.push_str(&format!(" if {}", self.expr(g)));
                    }
                    let (inline, block) = if self.is_inline(&arm.body) {
                        (Some(self.stmt_text(&arm.body.stmts[0])), String::new())
                    } else {
                        self.indent += 1;
                        let b = self.block_text(&arm.body);
                        self.indent -= 1;
                        (None, b)
                    };
                    rows.push((intro, (head, inline), block, trailing));
                    self.last_line = self.last_line.max(arm.line);
                }
                let mut heads: Vec<(String, Option<String>)> = rows.iter().map(|r| r.1.clone()).collect();
                align_arrows(&mut heads);
                let mut body = String::new();
                for (row, (head, inline)) in rows.iter().zip(heads) {
                    body.push_str(&row.0);
                    let text = match inline {
                        Some(v) => format!("{} -> {}", head, v),
                        None => format!("{} ->{}", head, row.2),
                    };
                    body.push_str(&self.line_with(&text, &row.3));
                }
                if let Some(a) = arms.first() {
                    body.push_str(&self.tail_comments(a.col));
                }
                self.indent -= 1;
                out.push('\n');
                out.push_str(body.trim_end_matches('\n'));
                out
            }
        }
    }

    fn if_text(&mut self, branches: &[(Expr, Block)], else_block: Option<&Block>) -> String {
        let mut out = String::new();
        // the source line the previous branch ended on, when it was inline
        let mut prev_inline_line: Option<usize> = None;
        for (i, (cond, body)) in branches.iter().enumerate() {
            let kw = if i == 0 { "if" } else { "elif" };
            let mut trailing = String::new();
            if i > 0 {
                let same_line = prev_inline_line.map(|l| l == cond.line).unwrap_or(false);
                if same_line {
                    out.push(' ');
                } else {
                    out.push('\n');
                    out.push_str(&self.comments_before(cond.line));
                    out.push_str(&ind(self.indent));
                    trailing = self.take_trailing(cond.line);
                }
            }
            out.push_str(&format!("{} {}:", kw, self.expr(cond)));
            if self.is_inline(body) {
                out.push(' ');
                out.push_str(&self.stmt_text(&body.stmts[0]));
                out.push_str(&trailing);
                prev_inline_line = Some(stmt_pos(&body.stmts[0]).0);
            } else {
                out.push_str(&trailing);
                self.indent += 1;
                out.push_str(&self.block_text(body));
                self.indent -= 1;
                prev_inline_line = None;
            }
            self.last_line = self.last_line.max(cond.line);
        }
        if let Some(eb) = else_block {
            let first = eb.stmts.first().map(stmt_pos).map(|p| p.0);
            let same_line = self.is_inline(eb) && prev_inline_line.is_some() && first == prev_inline_line;
            if same_line {
                out.push_str(" else: ");
                out.push_str(&self.stmt_text(&eb.stmts[0]));
            } else {
                out.push('\n');
                // the `else` line: the first statement's line when inline, else the line before the body
                let else_line = match first {
                    Some(l) if self.is_inline(eb) => l,
                    Some(l) => self.code_lines.range(..l).next_back().copied().unwrap_or(l),
                    None => self.last_line + 1,
                };
                out.push_str(&self.comments_before(else_line));
                out.push_str(&ind(self.indent));
                out.push_str("else:");
                let trailing = self.take_trailing(else_line);
                if self.is_inline(eb) {
                    out.push(' ');
                    out.push_str(&self.stmt_text(&eb.stmts[0]));
                    out.push_str(&trailing);
                } else {
                    out.push_str(&trailing);
                    self.indent += 1;
                    out.push_str(&self.block_text(eb));
                    self.indent -= 1;
                }
                self.last_line = self.last_line.max(else_line);
            }
        }
        out
    }
}

/// `[T]` / `[K, V: Ordered]` after a definition's name.
fn generics_str(gs: &[TypeParam]) -> String {
    if gs.is_empty() {
        return String::new();
    }
    let parts: Vec<String> = gs.iter().map(|p| match &p.bound {
        Some(b) => format!("{}: {}", p.name, type_str(b)),
        None => p.name.clone(),
    }).collect();
    format!("[{}]", parts.join(", "))
}

pub fn type_str(t: &Type) -> String {
    match t {
        Type::Int => "Int".into(),
        Type::Float => "Float".into(),
        Type::Bool => "Bool".into(),
        Type::Str => "Str".into(),
        Type::Unit => "()".into(),
        Type::List(e) => format!("[{}]", type_str(e)),
        Type::Named(n) => n.clone(),
        Type::Option(e) => match **e {
            Type::Result(..) => format!("({})?", type_str(e)),
            _ => format!("{}?", type_str(e)),
        },
        Type::Tuple(ts) => format!("({})", ts.iter().map(type_str).collect::<Vec<_>>().join(", ")),
        Type::Result(a, b) => format!("{} or {}", type_str(a), type_str(b)),
        Type::Map(k, v) => format!("{{{}: {}}}", type_str(k), type_str(v)),
        Type::Set(t) => format!("{{{}}}", type_str(t)),
        Type::Char => "Char".into(),
        Type::Iter(e, _) => format!("[{}]", type_str(e)),
        Type::Task(e) => format!("Task[{}]", type_str(e)),
        Type::Future(e) => format!("async {}", type_str(e)),
        Type::App(n, args) => format!("{}[{}]", n, args.iter().map(type_str).collect::<Vec<_>>().join(", ")),
        Type::Fn(ps, r) => format!("({}) -> {}", ps.iter().map(type_str).collect::<Vec<_>>().join(", "), type_str(r)),
        Type::Var(n) => n.clone(),
        Type::Shared(e, true) => format!("shared var {}", type_str(e)),
        Type::Shared(e, false) => format!("shared {}", type_str(e)),
        Type::Unknown => "_".into(),
    }
}

fn pattern_str(p: &Pattern) -> String {
    match &p.kind {
        PatKind::Wild => "_".into(),
        PatKind::Bind(n) => n.clone(),
        PatKind::Int(v) => v.to_string(),
        PatKind::Float(v) => format!("{:?}", v),
        PatKind::Bool(b) => b.to_string(),
        PatKind::Str(s) => format!("\"{}\"", Fmt::escape(s)),
        PatKind::Range { lo, hi, inclusive } => format!("{}{}{}", lo, if *inclusive { ".." } else { "..." }, hi),
        PatKind::Variant { enum_name, name, args, rest } => {
            let head = match enum_name {
                Some(e) => format!("{}.{}", e, name),
                None => name.clone(),
            };
            let mut parts: Vec<String> = args.iter().map(pattern_str).collect();
            if *rest {
                parts.push("..".into());
            }
            if parts.is_empty() {
                head
            } else {
                format!("{}({})", head, parts.join(", "))
            }
        }
        PatKind::Tuple(items) => format!("({})", items.iter().map(pattern_str).collect::<Vec<_>>().join(", ")),
        PatKind::Or(alts) => alts.iter().map(pattern_str).collect::<Vec<_>>().join(" | "),
        PatKind::List { items, rest } => {
            let mut parts: Vec<String> = items.iter().map(pattern_str).collect();
            if let Some(r) = rest {
                parts.push(match r {
                    Some(n) => format!("..{}", n),
                    None => "..".into(),
                });
            }
            format!("[{}]", parts.join(", "))
        }
    }
}

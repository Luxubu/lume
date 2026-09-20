//! Lexer: turns Lume source into tokens, including the INDENT / DEDENT /
//! NEWLINE tokens that give the language its Python-style block structure.
//!
//! Rules implemented here:
//! - Indentation is spaces only; tabs are an error.
//! - Blank lines and comment-only lines never affect indentation.
//! - Newlines inside ( ) [ ] { } are ignored, so calls and lists can wrap.
//! - A `?` directly after an identifier is part of the name (`empty?`).
//! - Strings support `#{expr}` interpolation; the expression source is kept
//!   raw and parsed later by the parser.

use crate::error::{LumeError, Result};

#[derive(Debug, Clone, PartialEq)]
pub enum StrPart {
    Lit(String),
    /// Raw expression source plus the column where it starts.
    Expr(String, usize),
}

#[derive(Debug, Clone, PartialEq)]
pub enum Tok {
    Ident(String),
    Int(i64),
    Float(f64),
    Str(Vec<StrPart>),
    Sym(&'static str),
    Newline,
    Indent,
    Dedent,
    Eof,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub tok: Tok,
    pub line: usize,
    pub col: usize,
    /// True when whitespace (or the line start) precedes this token.
    pub space_before: bool,
}

pub const KEYWORDS: &[&str] = &[
    "def", "var", "const", "if", "elif", "else", "unless", "while", "for", "in", "where",
    "return", "break", "next", "true", "false", "and", "or", "not", "puts", "struct", "enum",
    "match", "interface", "extend", "import", "pub", "test", "assert", "rust",
    "async", "await", "spawn", "shared",
];

pub fn is_keyword(s: &str) -> bool {
    KEYWORDS.contains(&s)
}

// Longest symbols first so `...` wins over `..` and `..` over `.`.
const SYMBOLS: &[&str] = &[
    "...", "**=", "->", "**", "==", "!=", "<=", ">=", "+=", "-=", "*=", "/=", "%=", "..", "|>",
    "(", ")", "[", "]", "{", "}", ",", ":", "=", "<", ">", "+", "-", "*", "/", "%", ".", "?",
    "!", "|",
];

/// A `#` comment, kept for the formatter (the parser never sees it).
#[derive(Debug, Clone)]
pub struct Comment {
    pub line: usize,
    pub col: usize,
    /// The text after `#`, untrimmed on the right.
    pub text: String,
}

/// What the formatter needs beyond the tokens.
#[derive(Debug, Default)]
pub struct LexInfo {
    pub comments: Vec<Comment>,
    /// Lines that hold nothing but whitespace.
    pub blank_lines: std::collections::HashSet<usize>,
}

pub fn lex(src: &str) -> Result<Vec<Token>> {
    lex_full(src).map(|(t, _)| t)
}

pub fn lex_full(src: &str) -> Result<(Vec<Token>, LexInfo)> {
    let mut info = LexInfo::default();
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut toks: Vec<Token> = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    let mut col = 1usize;
    let mut indent_stack: Vec<usize> = vec![0];
    let mut at_line_start = true;
    let mut depth: i32 = 0;

    let mut space_before = true;
    let push = |toks: &mut Vec<Token>, tok: Tok, line: usize, col: usize, space_before: &mut bool| {
        toks.push(Token { tok, line, col, space_before: *space_before });
        *space_before = false;
    };

    while i < n {
        if at_line_start && depth == 0 {
            let mut spaces = 0usize;
            let mut j = i;
            while j < n && (chars[j] == ' ' || chars[j] == '\t') {
                if chars[j] == '\t' {
                    return Err(LumeError::new(
                        line,
                        spaces + 1,
                        "tabs are not allowed for indentation",
                    )
                    .with_help("use spaces; two per level is the convention"));
                }
                spaces += 1;
                j += 1;
            }
            // Blank or comment-only line: skip it entirely.
            if j >= n || chars[j] == '\n' || chars[j] == '\r' || chars[j] == '#' {
                if j < n && chars[j] == '#' {
                    let start = j;
                    while j < n && chars[j] != '\n' {
                        j += 1;
                    }
                    let text: String = chars[start + 1..j].iter().collect();
                    info.comments.push(Comment { line, col: spaces + 1, text: text.trim_end().to_string() });
                } else {
                    info.blank_lines.insert(line);
                }
                while j < n && chars[j] != '\n' {
                    j += 1;
                }
                if j < n {
                    j += 1;
                    line += 1;
                }
                i = j;
                col = 1;
                continue;
            }
            // A line starting with `|>` continues the previous expression.
            if j + 1 < n && chars[j] == '|' && chars[j + 1] == '>' {
                if matches!(toks.last().map(|t| &t.tok), Some(Tok::Newline)) {
                    toks.pop();
                }
                i = j;
                col = spaces + 1;
                at_line_start = false;
                space_before = true;
                continue;
            }
            let top = *indent_stack.last().unwrap();
            if spaces > top {
                indent_stack.push(spaces);
                push(&mut toks, Tok::Indent, line, 1, &mut space_before);
            } else if spaces < top {
                while spaces < *indent_stack.last().unwrap() {
                    indent_stack.pop();
                    push(&mut toks, Tok::Dedent, line, 1, &mut space_before);
                }
                if spaces != *indent_stack.last().unwrap() {
                    return Err(LumeError::new(
                        line,
                        1,
                        "this line's indentation matches no enclosing block",
                    )
                    .with_help("indent it to line up with the block it belongs to"));
                }
            }
            i = j;
            col = spaces + 1;
            at_line_start = false;
            space_before = true;
        }

        let c = chars[i];
        match c {
            '\n' => {
                // `rust:` followed by an indented block: capture the block verbatim.
                let is_rust_block = depth == 0
                    && toks.len() >= 2
                    && matches!(toks[toks.len() - 1].tok, Tok::Sym(":"))
                    && matches!(&toks[toks.len() - 2].tok, Tok::Ident(s) if s == "rust");
                if is_rust_block {
                    let base = *indent_stack.last().unwrap();
                    let start_line = line;
                    i += 1;
                    line += 1;
                    let mut raw_lines: Vec<(usize, String)> = Vec::new();
                    loop {
                        if i >= n {
                            break;
                        }
                        let mut k = i;
                        let mut sp = 0;
                        while k < n && chars[k] == ' ' {
                            sp += 1;
                            k += 1;
                        }
                        let mut end = k;
                        while end < n && chars[end] != '\n' {
                            end += 1;
                        }
                        let text: String = chars[k..end].iter().collect();
                        let blank = text.trim().is_empty();
                        if !blank && sp <= base {
                            break;
                        }
                        raw_lines.push((sp, text));
                        i = if end < n { end + 1 } else { end };
                        line += 1;
                    }
                    while raw_lines.last().map(|(_, t)| t.trim().is_empty()).unwrap_or(false) {
                        raw_lines.pop();
                    }
                    if raw_lines.is_empty() {
                        return Err(LumeError::new(start_line, col, "`rust:` needs an indented block of Rust code on the following lines"));
                    }
                    let min_indent = raw_lines.iter().filter(|(_, t)| !t.trim().is_empty()).map(|(sp, _)| *sp).min().unwrap_or(0);
                    let code = raw_lines
                        .iter()
                        .map(|(sp, t)| format!("{}{}", " ".repeat(sp.saturating_sub(min_indent)), t))
                        .collect::<Vec<_>>()
                        .join("\n");
                    push(&mut toks, Tok::Str(vec![StrPart::Lit(code)]), start_line, col, &mut space_before);
                    push(&mut toks, Tok::Newline, line, 1, &mut space_before);
                    at_line_start = true;
                    col = 1;
                    continue;
                }
                if depth == 0 {
                    let last_is_break = matches!(
                        toks.last().map(|t| &t.tok),
                        Some(Tok::Newline) | Some(Tok::Indent) | None
                    );
                    if !last_is_break {
                        push(&mut toks, Tok::Newline, line, col, &mut space_before);
                    }
                    at_line_start = true;
                }
                i += 1;
                line += 1;
                col = 1;
            }
            ' ' | '\r' => {
                i += 1;
                col += 1;
                space_before = true;
            }
            '#' => {
                let start = i;
                let ccol = col;
                while i < n && chars[i] != '\n' {
                    i += 1;
                }
                let text: String = chars[start + 1..i].iter().collect();
                info.comments.push(Comment { line, col: ccol, text: text.trim_end().to_string() });
                col += i - start;
            }
            '0'..='9' => {
                let start_col = col;
                let mut s = String::new();
                let mut is_float = false;
                while i < n && (chars[i].is_ascii_digit() || chars[i] == '_') {
                    if chars[i] != '_' {
                        s.push(chars[i]);
                    }
                    i += 1;
                    col += 1;
                }
                if i + 1 < n && chars[i] == '.' && chars[i + 1].is_ascii_digit() {
                    is_float = true;
                    s.push('.');
                    i += 1;
                    col += 1;
                    while i < n && (chars[i].is_ascii_digit() || chars[i] == '_') {
                        if chars[i] != '_' {
                            s.push(chars[i]);
                        }
                        i += 1;
                        col += 1;
                    }
                }
                let tok = if is_float {
                    Tok::Float(s.parse().map_err(|_| {
                        LumeError::new(line, start_col, format!("bad number literal `{}`", s))
                    })?)
                } else {
                    Tok::Int(s.parse().map_err(|_| {
                        LumeError::new(line, start_col, format!("integer `{}` is too large", s))
                            .with_help("Int is 64-bit; the largest value is 9223372036854775807")
                    })?)
                };
                push(&mut toks, tok, line, start_col, &mut space_before);
            }
            '"' => {
                let start_col = col;
                i += 1;
                col += 1;
                let mut parts: Vec<StrPart> = Vec::new();
                let mut lit = String::new();
                let mut closed = false;
                while i < n {
                    let ch = chars[i];
                    if ch == '"' {
                        i += 1;
                        col += 1;
                        closed = true;
                        break;
                    }
                    if ch == '\n' {
                        return Err(LumeError::new(
                            line,
                            start_col,
                            "string literal is not closed before the end of the line",
                        )
                        .with_help("add the closing `\"`"));
                    }
                    if ch == '\\' && i + 1 < n && chars[i + 1] == 'u' {
                        // \u{1F600}
                        if i + 2 >= n || chars[i + 2] != '{' {
                            return Err(LumeError::new(line, col, "a unicode escape is written `\\u{...}` with the code point in hex")
                                .with_help("for example `\\u{1F600}`"));
                        }
                        let mut j = i + 3;
                        let mut hex = String::new();
                        while j < n && chars[j] != '}' && chars[j] != '"' && chars[j] != '\n' {
                            hex.push(chars[j]);
                            j += 1;
                        }
                        let cp = if j < n && chars[j] == '}' && !hex.is_empty() && hex.len() <= 6 { u32::from_str_radix(&hex, 16).ok() } else { None };
                        match cp.and_then(char::from_u32) {
                            Some(c) => lit.push(c),
                            None => {
                                return Err(LumeError::new(line, col, format!("`\\u{{{}}}` is not a valid unicode escape", hex))
                                    .with_help("write the code point in hex inside the braces, like `\\u{e9}` or `\\u{1F600}`"));
                            }
                        }
                        col += j + 1 - i;
                        i = j + 1;
                        continue;
                    }
                    if ch == '\\' && i + 1 < n {
                        let e = chars[i + 1];
                        lit.push(match e {
                            'n' => '\n',
                            't' => '\t',
                            '0' => '\0',
                            '"' => '"',
                            '\\' => '\\',
                            '#' => '#',
                            other => {
                                return Err(LumeError::new(
                                    line,
                                    col,
                                    format!("unknown escape `\\{}` in string", other),
                                )
                                .with_help("valid escapes are \\n \\t \\0 \\\" \\\\ \\# and \\u{...}"));
                            }
                        });
                        i += 2;
                        col += 2;
                        continue;
                    }
                    if ch == '#' && i + 1 < n && chars[i + 1] == '{' {
                        if !lit.is_empty() {
                            parts.push(StrPart::Lit(std::mem::take(&mut lit)));
                        }
                        i += 2;
                        col += 2;
                        let expr_col = col;
                        let mut braces = 1;
                        let mut raw = String::new();
                        while i < n {
                            let e = chars[i];
                            if e == '{' {
                                braces += 1;
                            } else if e == '}' {
                                braces -= 1;
                                if braces == 0 {
                                    break;
                                }
                            } else if e == '\n' {
                                return Err(LumeError::new(
                                    line,
                                    expr_col,
                                    "interpolation `#{` is not closed",
                                )
                                .with_help("add the closing `}`"));
                            }
                            raw.push(e);
                            i += 1;
                            col += 1;
                        }
                        if i >= n {
                            return Err(LumeError::new(
                                line,
                                expr_col,
                                "interpolation `#{` is not closed",
                            ));
                        }
                        i += 1; // closing }
                        col += 1;
                        if raw.trim().is_empty() {
                            return Err(LumeError::new(line, expr_col, "empty interpolation `#{}`"));
                        }
                        parts.push(StrPart::Expr(raw, expr_col));
                        continue;
                    }
                    lit.push(ch);
                    i += 1;
                    col += 1;
                }
                if !closed {
                    return Err(LumeError::new(
                        line,
                        start_col,
                        "string literal is not closed",
                    ));
                }
                if !lit.is_empty() || parts.is_empty() {
                    parts.push(StrPart::Lit(lit));
                }
                push(&mut toks, Tok::Str(parts), line, start_col, &mut space_before);
            }
            c if c.is_alphabetic() || c == '_' => {
                let start_col = col;
                let mut s = String::new();
                while i < n && (chars[i].is_alphanumeric() || chars[i] == '_') {
                    s.push(chars[i]);
                    i += 1;
                    col += 1;
                }
                // `name?` is a predicate name; `?` is only propagation after a call/value.
                if i < n && chars[i] == '?' {
                    s.push('?');
                    i += 1;
                    col += 1;
                }
                push(&mut toks, Tok::Ident(s), line, start_col, &mut space_before);
            }
            _ => {
                let mut matched = None;
                for sym in SYMBOLS {
                    let sc: Vec<char> = sym.chars().collect();
                    if i + sc.len() <= n && chars[i..i + sc.len()] == sc[..] {
                        matched = Some(*sym);
                        break;
                    }
                }
                match matched {
                    Some(sym) => {
                        match sym {
                            "(" | "[" | "{" => depth += 1,
                            ")" | "]" | "}" => depth -= 1,
                            _ => {}
                        }
                        push(&mut toks, Tok::Sym(sym), line, col, &mut space_before);
                        i += sym.len();
                        col += sym.len();
                    }
                    None => {
                        let e = LumeError::new(line, col, format!("unexpected character `{}`", c));
                        return Err(match c {
                            '\'' => e.with_help("strings use double quotes: \"like this\""),
                            ';' => e.with_help("Lume ends a statement at the end of the line; no `;` is needed"),
                            '&' => e.with_help("`and` joins conditions in Lume"),
                            '$' | '@' => e.with_help("names are plain words in Lume, with no sigil"),
                            _ => e,
                        });
                    }
                }
            }
        }
    }

    let last_is_break = matches!(
        toks.last().map(|t| &t.tok),
        Some(Tok::Newline) | Some(Tok::Dedent) | None
    );
    if !last_is_break {
        push(&mut toks, Tok::Newline, line, col, &mut space_before);
    }
    while indent_stack.len() > 1 {
        indent_stack.pop();
        push(&mut toks, Tok::Dedent, line, 1, &mut space_before);
    }
    push(&mut toks, Tok::Eof, line, col, &mut space_before);
    Ok((toks, info))
}

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
}

pub const KEYWORDS: &[&str] = &[
    "def", "var", "const", "if", "elif", "else", "unless", "while", "for", "in", "where",
    "return", "break", "next", "true", "false", "and", "or", "not", "puts", "struct", "enum",
    "match", "interface", "extend", "import", "pub", "test", "assert",
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

pub fn lex(src: &str) -> Result<Vec<Token>> {
    let chars: Vec<char> = src.chars().collect();
    let n = chars.len();
    let mut toks: Vec<Token> = Vec::new();
    let mut i = 0usize;
    let mut line = 1usize;
    let mut col = 1usize;
    let mut indent_stack: Vec<usize> = vec![0];
    let mut at_line_start = true;
    let mut depth: i32 = 0;

    let push = |toks: &mut Vec<Token>, tok: Tok, line: usize, col: usize| {
        toks.push(Token { tok, line, col });
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
            let top = *indent_stack.last().unwrap();
            if spaces > top {
                indent_stack.push(spaces);
                push(&mut toks, Tok::Indent, line, 1);
            } else if spaces < top {
                while spaces < *indent_stack.last().unwrap() {
                    indent_stack.pop();
                    push(&mut toks, Tok::Dedent, line, 1);
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
        }

        let c = chars[i];
        match c {
            '\n' => {
                if depth == 0 {
                    let last_is_break = matches!(
                        toks.last().map(|t| &t.tok),
                        Some(Tok::Newline) | Some(Tok::Indent) | None
                    );
                    if !last_is_break {
                        push(&mut toks, Tok::Newline, line, col);
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
            }
            '#' => {
                while i < n && chars[i] != '\n' {
                    i += 1;
                }
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
                push(&mut toks, tok, line, start_col);
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
                                .with_help("valid escapes are \\n \\t \\0 \\\" \\\\ \\#"));
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
                push(&mut toks, Tok::Str(parts), line, start_col);
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
                push(&mut toks, Tok::Ident(s), line, start_col);
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
                        push(&mut toks, Tok::Sym(sym), line, col);
                        i += sym.len();
                        col += sym.len();
                    }
                    None => {
                        return Err(LumeError::new(
                            line,
                            col,
                            format!("unexpected character `{}`", c),
                        ));
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
        push(&mut toks, Tok::Newline, line, col);
    }
    while indent_stack.len() > 1 {
        indent_stack.pop();
        push(&mut toks, Tok::Dedent, line, 1);
    }
    push(&mut toks, Tok::Eof, line, col);
    Ok(toks)
}

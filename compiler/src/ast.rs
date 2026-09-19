//! Abstract syntax tree for the milestone-1 subset of Lume:
//! functions, bindings, if/elif/else as expressions, while, for..in..where,
//! calls, method calls, string interpolation, lists and ranges.

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    Unit,
    List(Box<Type>),
    Named(String),
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<Param>,
    pub ret: Type,
    pub body: Block,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Default)]
pub struct Block {
    pub stmts: Vec<Stmt>,
}

#[derive(Debug, Clone)]
pub enum Stmt {
    /// `name = value` — a new immutable binding, or an assignment if `name`
    /// already exists in scope (the code generator decides, and rejects
    /// assignment to an immutable binding).
    Bind { name: String, value: Expr, line: usize, col: usize },
    /// `var name = value`
    Var { name: String, value: Expr, line: usize, col: usize },
    /// `name += value` and friends
    OpAssign { name: String, op: &'static str, value: Expr, line: usize, col: usize },
    Expr(Expr),
    Return { value: Option<Expr>, line: usize, col: usize },
    While { cond: Expr, body: Block },
    For { var: String, iter: Expr, filter: Option<Expr>, body: Block, line: usize, col: usize },
    Break { line: usize, col: usize },
    Next { line: usize, col: usize },
}

#[derive(Debug, Clone)]
pub enum StrPiece {
    Lit(String),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct Expr {
    pub kind: ExprKind,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(Vec<StrPiece>),
    Ident(String),
    List(Vec<Expr>),
    Range { lo: Box<Expr>, hi: Box<Expr>, inclusive: bool },
    Unary { op: &'static str, expr: Box<Expr> },
    Binary { op: &'static str, lhs: Box<Expr>, rhs: Box<Expr> },
    Call { name: String, args: Vec<Expr> },
    Method { recv: Box<Expr>, name: String, args: Vec<Expr> },
    If { branches: Vec<(Expr, Block)>, else_block: Option<Block> },
    Puts(Box<Expr>),
}

impl Expr {
    pub fn new(kind: ExprKind, line: usize, col: usize) -> Self {
        Expr { kind, line, col }
    }
}

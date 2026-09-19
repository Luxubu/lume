//! Abstract syntax tree for the milestone-2 subset of Lume:
//! functions, structs with methods, bindings, if/elif/else as expressions,
//! while, for..in..where, calls with keyword arguments, method calls,
//! string interpolation, lists and ranges.

#[derive(Debug, Clone, PartialEq)]
pub enum Type {
    Int,
    Float,
    Bool,
    Str,
    Unit,
    List(Box<Type>),
    Named(String),
    /// `T?`
    Option(Box<Type>),
    /// `(A, B, ...)`
    Tuple(Vec<Type>),
    /// `T or E`
    Result(Box<Type>, Box<Type>),
    /// A lazy chain (`xs.filter(...).map(...)`) not yet collected. The bool
    /// says whether items are references into the source list. Internal:
    /// becomes `[T]` wherever a value is needed, never reaches a signature.
    Iter(Box<Type>, bool),
    /// Not yet inferred. Never reaches generated Rust.
    Unknown,
}

impl Type {
    /// Types that Rust copies bitwise; everything else is borrowed when
    /// passed to a function.
    pub fn is_copy(&self) -> bool {
        match self {
            Type::Int | Type::Float | Type::Bool | Type::Unit => true,
            Type::Option(t) => t.is_copy(),
            Type::Tuple(ts) => ts.iter().all(|t| t.is_copy()),
            _ => false,
        }
    }

    /// The type a value of this type has once materialised.
    pub fn materialized(&self) -> Type {
        match self {
            Type::Iter(e, _) => Type::List(e.clone()),
            other => other.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SelfKind {
    /// A free function, or a method that only reads fields (`&self`).
    Read,
    /// `def f(var self, ...)` — may change fields (`&mut self`).
    Mutate,
}

#[derive(Debug, Clone)]
pub struct FnDef {
    pub name: String,
    pub params: Vec<Param>,
    /// `None` when the signature has no `-> Type`; inferred from the body.
    pub ret: Option<Type>,
    pub self_kind: SelfKind,
    pub body: Block,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct StructDef {
    pub name: String,
    pub fields: Vec<Param>,
    pub methods: Vec<FnDef>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Variant {
    pub name: String,
    pub fields: Vec<Param>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct EnumDef {
    pub name: String,
    pub variants: Vec<Variant>,
    pub methods: Vec<FnDef>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Import {
    /// `import rust.regex` -> "regex"
    pub krate: String,
    pub version: Option<String>,
    pub alias: Option<String>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum Item {
    Fn(FnDef),
    Struct(StructDef),
    Enum(EnumDef),
    Import(Import),
}

#[derive(Debug, Clone)]
pub struct Pattern {
    pub kind: PatKind,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum PatKind {
    Wild,
    Bind(String),
    Int(i64),
    Float(f64),
    Bool(bool),
    Str(String),
    Range { lo: i64, hi: i64, inclusive: bool },
    /// `Circle(r)`, `Some(x)`, `None`, `Shape.Circle(r)`
    Variant { enum_name: Option<String>, name: String, args: Vec<Pattern> },
    Tuple(Vec<Pattern>),
    /// `[a, b]`, `[]`, `[first, ..rest]`
    List { items: Vec<Pattern>, rest: Option<Option<String>> },
}

#[derive(Debug, Clone)]
pub struct MatchArm {
    pub pat: Pattern,
    pub guard: Option<Expr>,
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
    /// `name = value` — a new immutable binding, an assignment to a `var`,
    /// a self-transform rebind, or a field assignment inside a method.
    Bind { name: String, ty: Option<Type>, value: Expr, line: usize, col: usize },
    /// `var name = value` / `var name: Type = value`
    Var { name: String, ty: Option<Type>, value: Expr, line: usize, col: usize },
    /// `name += value` and friends
    OpAssign { name: String, op: &'static str, value: Expr, line: usize, col: usize },
    /// `recv.field = value` / `recv.field += value`
    FieldAssign { recv: Expr, field: String, op: Option<&'static str>, value: Expr, line: usize, col: usize },
    Expr(Expr),
    Return { value: Option<Expr>, line: usize, col: usize },
    While { cond: Expr, body: Block },
    /// `for x in xs` or `for i, x in xs.enumerate` (vars.len() == 2 destructures a tuple)
    For { vars: Vec<String>, iter: Expr, filter: Option<Expr>, body: Block, line: usize, col: usize },
    Break { line: usize, col: usize },
    Next { line: usize, col: usize },
}

#[derive(Debug, Clone)]
pub enum StrPiece {
    Lit(String),
    Expr(Expr),
}

#[derive(Debug, Clone)]
pub struct Arg {
    pub name: Option<String>,
    pub value: Expr,
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
    SelfRef,
    List(Vec<Expr>),
    Range { lo: Box<Expr>, hi: Box<Expr>, inclusive: bool },
    Unary { op: &'static str, expr: Box<Expr> },
    Binary { op: &'static str, lhs: Box<Expr>, rhs: Box<Expr> },
    /// `name(args)` — a function call or a struct constructor.
    Call { name: String, args: Vec<Arg> },
    /// `recv.name` / `recv.name(args)` — a field read or a method call;
    /// the code generator decides from the receiver's type.
    Method { recv: Box<Expr>, name: String, args: Vec<Arg> },
    If { branches: Vec<(Expr, Block)>, else_block: Option<Block> },
    Puts(Box<Expr>),
    /// `_` inside a method argument; the parser turns the argument into a
    /// one-parameter `Lambda` whose parameter is named `_`.
    Placeholder,
    /// `{ |x| expr }`, `do |x| ... end-of-block`, or a wrapped `_` argument.
    Lambda { params: Vec<String>, body: Block },
    Match { scrutinee: Box<Expr>, arms: Vec<MatchArm> },
    Tuple(Vec<Expr>),
    /// `t.0`, `t.1`
    TupleIndex { recv: Box<Expr>, index: usize },
    /// `Some(x)`
    Some(Box<Expr>),
    None,
    /// `expr?` — early return on None or Error
    Try(Box<Expr>),
    /// `Ok(x)` — explicit success value (usually implied)
    Ok(Box<Expr>),
    /// `expr!` — unwrap or panic; a warning outside tests
    Unwrap(Box<Expr>),
    /// `rust("...")` or `rust:` + indented block — Rust code emitted verbatim
    Rust(String),
}

impl Expr {
    pub fn new(kind: ExprKind, line: usize, col: usize) -> Self {
        Expr { kind, line, col }
    }
}

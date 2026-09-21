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
    /// One character; becomes a `Str` wherever one is wanted.
    Char,
    Unit,
    List(Box<Type>),
    Named(String),
    /// `T?`
    Option(Box<Type>),
    /// `(A, B, ...)`
    Tuple(Vec<Type>),
    /// `T or E`
    Result(Box<Type>, Box<Type>),
    /// `{K: V}`
    Map(Box<Type>, Box<Type>),
    /// `{T}` — a set: each value at most once, insertion order kept
    Set(Box<Type>),
    /// `Stack[Int]` — a user generic type with its arguments filled in.
    App(String, Vec<Type>),
    /// `T` inside a definition that declares it: a type parameter.
    Var(String),
    /// `(A, B) -> C` — behaviour a caller hands in: a block, a `_`
    /// shorthand, or the name of a function.
    Fn(Vec<Type>, Box<Type>),
    /// A lazy chain (`xs.filter(...).map(...)`) not yet collected. The bool
    /// says whether items are references into the source list. Internal:
    /// becomes `[T]` wherever a value is needed, never reaches a signature.
    Iter(Box<Type>, bool),
    /// `Task[T]` — a spawned task that will produce a `T`
    Task(Box<Type>),
    /// The value of calling an `async def` before `await`. Internal.
    Future(Box<Type>),
    /// `shared T` (one value, many handles) / `shared var T` (behind a lock)
    Shared(Box<Type>, bool),
    /// Not yet inferred. Never reaches generated Rust.
    Unknown,
}

impl Type {
    /// Types that Rust copies bitwise; everything else is borrowed when
    /// passed to a function.
    pub fn is_copy(&self) -> bool {
        match self {
            Type::Int | Type::Float | Type::Bool | Type::Char | Type::Unit => true,
            // a type parameter stands for anything, so it is always borrowed
            Type::Var(_) => false,
            // behaviour is moved into the call, never lent
            Type::Fn(..) => true,
            Type::Option(t) => t.is_copy(),
            // a handle is cheap to clone, and cloning is how it is shared
            Type::Shared(..) => true,
            Type::Tuple(ts) => ts.iter().all(|t| t.is_copy()),
            _ => false,
        }
    }

    /// The type a value of this type has once materialised.
    pub fn materialized(&self) -> Type {
        match self {
            Type::Iter(e, _) => Type::List(e.clone()),
            // a shared value behaves as the value it holds
            Type::Shared(inner, _) => inner.materialized(),
            other => other.clone(),
        }
    }
}

/// `[T]` / `[T: Ordered]` after a `def`, `struct` or `enum` name.
#[derive(Debug, Clone, PartialEq)]
pub struct TypeParam {
    pub name: String,
    /// What every argument must satisfy: a built-in bound (`Ordered`,
    /// `Hashable`) or an interface, which may carry arguments of its own
    /// (`Comparable[T]`).
    pub bound: Option<Type>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Param {
    pub name: String,
    pub ty: Type,
    /// `var name: T` — the callee may change it in place
    pub mutable: bool,
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
    /// `pub def` — visible to importing modules
    pub public: bool,
    /// `async def` — callers `await` the result
    pub is_async: bool,
    /// `def first[T](...)` — type parameters this function declares.
    pub generics: Vec<TypeParam>,
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
    pub public: bool,
    pub generics: Vec<TypeParam>,
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
    pub public: bool,
    pub generics: Vec<TypeParam>,
    pub variants: Vec<Variant>,
    pub methods: Vec<FnDef>,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub struct Import {
    /// `import rust.regex` -> ["regex"]; `import users.model` -> ["users", "model"]
    pub path: Vec<String>,
    /// `import rust.<crate>`
    pub is_rust: bool,
    pub version: Option<String>,
    pub alias: Option<String>,
    pub line: usize,
    pub col: usize,
}

impl Import {
    pub fn krate(&self) -> &str {
        &self.path[0]
    }
}

/// `interface Name:` — required methods (no body) and defaults (with a body).
#[derive(Debug, Clone)]
pub struct InterfaceDef {
    pub name: String,
    pub public: bool,
    /// `interface Comparable[T]:` — type parameters the interface declares.
    pub generics: Vec<TypeParam>,
    /// Required: signature only. `body` is None.
    pub required: Vec<FnDef>,
    pub defaults: Vec<FnDef>,
    pub line: usize,
    pub col: usize,
}

/// `extend Type with Iface:` — methods that make `Type` conform to `Iface`.
#[derive(Debug, Clone)]
pub struct ExtendDef {
    pub target: Type,
    /// `extend Version with Comparable[Version]:` — the interface, with its
    /// arguments when it takes any.
    pub iface: Type,
    pub methods: Vec<FnDef>,
    pub line: usize,
    pub col: usize,
}

/// `test "name":` — compiled only by `lume test`.
#[derive(Debug, Clone)]
pub struct TestDef {
    pub name: String,
    pub body: Block,
    pub line: usize,
    pub col: usize,
}

#[derive(Debug, Clone)]
pub enum Item {
    Fn(FnDef),
    /// `NAME = value` at the top level: one value, computed once.
    Const(ConstDef),
    Struct(StructDef),
    Enum(EnumDef),
    Import(Import),
    Interface(InterfaceDef),
    Extend(ExtendDef),
    Test(TestDef),
}

#[derive(Debug, Clone)]
pub struct ConstDef {
    pub name: String,
    pub ty: Option<Type>,
    pub value: Expr,
    pub public: bool,
    pub line: usize,
    pub col: usize,
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
    Variant { enum_name: Option<String>, name: String, args: Vec<Pattern>, rest: bool },
    Tuple(Vec<Pattern>),
    /// `[a, b]`, `[]`, `[first, ..rest]`
    List { items: Vec<Pattern>, rest: Option<Option<String>> },
    /// `A | B`, `1 | 2 | 3`, `Some(0) | None`
    Or(Vec<Pattern>),
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
    /// `(a, b) = pair` — new immutable bindings, one per part; `_` skips a part
    Destructure { names: Vec<String>, value: Expr, line: usize, col: usize },
    /// `var name = value` / `var name: Type = value`
    Var { name: String, ty: Option<Type>, value: Expr, line: usize, col: usize },
    /// `name += value` and friends
    OpAssign { name: String, op: &'static str, value: Expr, line: usize, col: usize },
    /// `recv.field = value` / `recv.field += value`; `recv` may be `xs[i]`
    FieldAssign { recv: Expr, field: String, op: Option<&'static str>, value: Expr, line: usize, col: usize },
    /// `xs[i] = value` / `m[k] = value` / `xs[i] += value`
    IndexAssign { recv: Expr, index: Expr, op: Option<&'static str>, value: Expr, line: usize, col: usize },
    Expr(Expr),
    Return { value: Option<Expr>, line: usize, col: usize },
    While { cond: Expr, body: Block },
    /// `for x in xs` or `for i, x in xs.enumerate` (vars.len() == 2 destructures a tuple);
    /// `for var x in xs` lets the body change each element in place
    For { vars: Vec<String>, mutable: bool, iter: Expr, filter: Option<Expr>, body: Block, line: usize, col: usize },
    Break { line: usize, col: usize },
    Next { line: usize, col: usize },
    /// `assert cond` — stops the test (or program) with both sides printed
    Assert { cond: Expr, line: usize, col: usize },
    /// `shared x = v` / `shared var x = v` — a handle other tasks can hold
    Shared { name: String, mutable: bool, ty: Option<Type>, value: Expr, line: usize, col: usize },
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
    /// `{1, 2, 3}`
    SetLit(Vec<Expr>),
    Unary { op: &'static str, expr: Box<Expr> },
    Binary { op: &'static str, lhs: Box<Expr>, rhs: Box<Expr> },
    /// `name(args)` — a function call or a struct constructor.
    Call { name: String, args: Vec<Arg> },
    /// `recv.name` / `recv.name(args)` — a field read or a method call;
    /// the code generator decides from the receiver's type.
    Method { recv: Box<Expr>, name: String, args: Vec<Arg> },
    If { branches: Vec<(Expr, Block)>, else_block: Option<Block> },
    Puts(Box<Expr>),
    /// `warn x`: like `puts`, but to the error stream.
    Warn(Box<Expr>),
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
    /// `xs[i]` / `m[k]` — reads give `T?`
    Index { recv: Box<Expr>, index: Box<Expr> },
    /// `{k: v, ...}` / `{}`
    MapLit(Vec<(Expr, Expr)>),
    /// `await expr` — wait for an async call, a task, or a list of tasks
    Await(Box<Expr>),
    /// `spawn:` + block — run the block as its own task; the value is a `Task[T]`
    Spawn(Block),
}

impl Expr {
    pub fn new(kind: ExprKind, line: usize, col: usize) -> Self {
        Expr { kind, line, col }
    }
}

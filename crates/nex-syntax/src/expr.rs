//! expression ast.

use crate::node::{AstNode, HasSpan, Ident, NodeId, NodeInfo, Spanned};
use crate::pattern::Pattern;
use crate::stmt::Stmt;
use nex_lexer::Span;

/// unary operators
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    /// `-x`
    Neg,
    /// `!x`
    Not,
}

/// binary operators
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    // logical
    Or,
    And,
    // comparison
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    // bitwise
    BitOr,
    BitXor,
    BitAnd,
    Shl,
    Shr,
    // arithmetic
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

/// a single expression node
#[derive(Clone, Debug, PartialEq)]
pub struct Expr {
    pub info: NodeInfo,
    pub kind: ExprKind,
}

impl Expr {
    pub fn new(kind: ExprKind, info: NodeInfo) -> Self {
        Expr { info, kind }
    }
}

impl HasSpan for Expr {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Expr {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

/// every expression form in the language sketch
#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    /// `5`, `1.5`, `"hi"`, `true`, `()`
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Unit,
    /// a bare name
    Ident(Ident),
    /// `-x`, `!x`
    Unary {
        op: Spanned<UnaryOp>,
        operand: Box<Expr>,
    },
    /// `a + b`
    Binary {
        op: Spanned<BinaryOp>,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    /// `f(x, y)`
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    /// `p.x`
    Field {
        base: Box<Expr>,
        field: Ident,
    },
    /// `xs[0]`
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    /// `Point { x: 1.0, y: 2.0 }`
    StructLit {
        name: Ident,
        fields: Vec<FieldInit>,
    },
    /// `if c { a } else { b }`; `else` optional
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Option<Box<Expr>>,
    },
    /// `{ stmts }`
    Block(Block),
    /// `match x { pat => body, ... }`
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    /// `a..b`, `a..=b`
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
}

/// one `name: value` entry in a struct literal
#[derive(Clone, Debug, PartialEq)]
pub struct FieldInit {
    pub info: NodeInfo,
    pub name: Ident,
    pub value: Expr,
}

impl FieldInit {
    pub fn new(name: Ident, value: Expr, info: NodeInfo) -> Self {
        FieldInit { info, name, value }
    }
}

impl HasSpan for FieldInit {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for FieldInit {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

/// a `{ ... }` block. its value is the last expression statement
#[derive(Clone, Debug, PartialEq)]
pub struct Block {
    pub info: NodeInfo,
    pub stmts: Vec<Stmt>,
}

impl Block {
    pub fn new(stmts: Vec<Stmt>, info: NodeInfo) -> Self {
        Block { info, stmts }
    }
}

impl HasSpan for Block {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Block {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

/// one `pattern => body` arm of a match
#[derive(Clone, Debug, PartialEq)]
pub struct MatchArm {
    pub info: NodeInfo,
    pub pattern: Pattern,
    pub body: Expr,
}

impl MatchArm {
    pub fn new(pattern: Pattern, body: Expr, info: NodeInfo) -> Self {
        MatchArm {
            info,
            pattern,
            body,
        }
    }
}

impl HasSpan for MatchArm {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for MatchArm {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

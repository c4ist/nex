//! expression ast.

use crate::item::Param;
use crate::node::{AstNode, HasSpan, Ident, NodeId, NodeInfo, Spanned};
use crate::pattern::Pattern;
use crate::stmt::Stmt;
use nex_lexer::Span;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum UnaryOp {
    Neg,
    Not,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BinaryOp {
    Or,
    And,
    Eq,
    Ne,
    Lt,
    Le,
    Gt,
    Ge,
    BitOr,
    BitXor,
    BitAnd,
    Shl,
    Shr,
    Add,
    Sub,
    Mul,
    Div,
    Rem,
}

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

#[derive(Clone, Debug, PartialEq)]
pub enum ExprKind {
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    Unit,
    Ident(Ident),
    Unary {
        op: Spanned<UnaryOp>,
        operand: Box<Expr>,
    },
    Binary {
        op: Spanned<BinaryOp>,
        lhs: Box<Expr>,
        rhs: Box<Expr>,
    },
    Call {
        callee: Box<Expr>,
        args: Vec<Expr>,
    },
    Field {
        base: Box<Expr>,
        field: Ident,
    },
    Index {
        base: Box<Expr>,
        index: Box<Expr>,
    },
    StructLit {
        name: Ident,
        fields: Vec<FieldInit>,
    },
    If {
        cond: Box<Expr>,
        then: Box<Expr>,
        else_: Option<Box<Expr>>,
    },
    Block(Block),
    Match {
        scrutinee: Box<Expr>,
        arms: Vec<MatchArm>,
    },
    Range {
        start: Box<Expr>,
        end: Box<Expr>,
        inclusive: bool,
    },
    /// `|x| x + 1`; the body is one expression, which can be a block
    Closure {
        params: Vec<Param>,
        body: Box<Expr>,
    },
}

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

/// its value is the last expression statement
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

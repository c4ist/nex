//! statement ast.

use crate::expr::{Block, Expr};
use crate::node::{AstNode, HasSpan, Ident, NodeId, NodeInfo};
use nex_lexer::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Stmt {
    pub info: NodeInfo,
    pub kind: StmtKind,
}

impl Stmt {
    pub fn new(kind: StmtKind, info: NodeInfo) -> Self {
        Stmt { info, kind }
    }
}

impl HasSpan for Stmt {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Stmt {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum StmtKind {
    Expr(Expr),
    Let {
        mutable: bool,
        name: Ident,
        value: Expr,
    },
    Return(Option<Expr>),
    While {
        cond: Expr,
        body: Block,
    },
    ForIn {
        binding: Ident,
        iter: Expr,
        body: Block,
    },
    Break,
    Continue,
}

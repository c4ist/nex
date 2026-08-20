//! statement ast. just what blocks need for now; the rest lands in step 2.3.

use crate::expr::Expr;
use crate::node::{AstNode, HasSpan, NodeId, NodeInfo};
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
}

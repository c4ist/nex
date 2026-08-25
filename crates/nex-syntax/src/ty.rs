//! type ast. just named types for now; the rest lands in step 2.5.

use crate::node::{AstNode, HasSpan, Ident, NodeId, NodeInfo};
use nex_lexer::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Type {
    pub info: NodeInfo,
    pub kind: TypeKind,
}

impl Type {
    pub fn new(kind: TypeKind, info: NodeInfo) -> Self {
        Type { info, kind }
    }
}

impl HasSpan for Type {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Type {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum TypeKind {
    Named(Ident),
}

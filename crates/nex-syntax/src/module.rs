//! the ast root: one parsed source file.

use crate::item::Item;
use crate::node::{AstNode, HasSpan, NodeId, NodeInfo};
use nex_lexer::Span;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    pub info: NodeInfo,
    pub items: Vec<Item>,
}

impl Module {
    pub fn new(items: Vec<Item>, info: NodeInfo) -> Self {
        Module { info, items }
    }
}

impl HasSpan for Module {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Module {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

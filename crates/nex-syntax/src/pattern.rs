//! pattern ast.
//!
//! just what match needs for now; literal/enum-variant/struct/tuple patterns
//! land in step 2.6.

use crate::node::{AstNode, HasSpan, Ident, NodeId, NodeInfo};
use nex_lexer::Span;

/// a single pattern node
#[derive(Clone, Debug, PartialEq)]
pub struct Pattern {
    pub info: NodeInfo,
    pub kind: PatternKind,
}

impl Pattern {
    pub fn new(kind: PatternKind, info: NodeInfo) -> Self {
        Pattern { info, kind }
    }
}

impl HasSpan for Pattern {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for Pattern {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

/// every pattern form
#[derive(Clone, Debug, PartialEq)]
pub enum PatternKind {
    /// `_`
    Wildcard,
    /// `v` — binds the matched value
    Binding(Ident),
}

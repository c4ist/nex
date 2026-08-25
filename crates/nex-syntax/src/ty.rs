//! type ast.

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
    /// `i32`, `Point`, `Option<T>`, `Vec<i32>` - `args` is empty for a
    /// non-generic name
    Named { name: Ident, args: Vec<Type> },
    /// `[T]`
    Array(Box<Type>),
    /// `&T`
    Ref(Box<Type>),
    /// `fn(A, B) -> C`; a bare `fn(A, B)` has `return_type: None`
    Fn {
        params: Vec<Type>,
        return_type: Option<Box<Type>>,
    },
}

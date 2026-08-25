//! pattern ast for match arms.

use crate::node::{AstNode, HasSpan, Ident, NodeId, NodeInfo};
use nex_lexer::Span;

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

#[derive(Clone, Debug, PartialEq)]
pub enum PatternKind {
    Wildcard,
    Binding(Ident),
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    /// `Option::Some(v)`; `fields` is empty for a unit variant like
    /// `Option::None`
    EnumVariant {
        path: Vec<Ident>,
        fields: Vec<Pattern>,
    },
    Struct {
        name: Ident,
        fields: Vec<FieldPattern>,
    },
    Tuple(Vec<Pattern>),
}

/// one `name: pattern` inside a struct pattern, e.g. `x: px` in
/// `Point { x: px, y: py }`
#[derive(Clone, Debug, PartialEq)]
pub struct FieldPattern {
    pub info: NodeInfo,
    pub name: Ident,
    pub pattern: Pattern,
}

impl FieldPattern {
    pub fn new(name: Ident, pattern: Pattern, info: NodeInfo) -> Self {
        FieldPattern {
            info,
            name,
            pattern,
        }
    }
}

impl HasSpan for FieldPattern {
    fn span(&self) -> Span {
        self.info.span
    }
}

impl AstNode for FieldPattern {
    fn id(&self) -> NodeId {
        self.info.id
    }
}

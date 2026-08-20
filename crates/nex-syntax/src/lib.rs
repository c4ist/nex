//! the nex ast.
//!
//! will hold the ast and the parser. so far it's just the bit every node needs:
//! identity ([`NodeId`]) and location ([`Span`]).
//!
//! ```
//! use nex_syntax::{NodeIdGen, NodeInfo, Span};
//!
//! let mut ids = NodeIdGen::new();
//! let node = NodeInfo::new(ids.fresh(), Span::new(0, 3));
//! assert_eq!(format!("{node:?}"), "#0@0..3");
//! ```

mod expr;
mod node;
mod pattern;
mod stmt;

pub use expr::{BinaryOp, Block, Expr, ExprKind, FieldInit, MatchArm, UnaryOp};
pub use node::{spanning, AstNode, HasSpan, Ident, NodeId, NodeIdGen, NodeInfo, Spanned};
pub use pattern::{Pattern, PatternKind};
pub use stmt::{Stmt, StmtKind};

/// re-exported so downstream crates don't need a `nex-lexer` dep just to name
/// a source location
pub use nex_lexer::Span;

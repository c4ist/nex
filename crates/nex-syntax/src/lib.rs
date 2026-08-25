//! the nex ast.
//!
//! ```
//! use nex_syntax::{NodeIdGen, NodeInfo, Span};
//!
//! let mut ids = NodeIdGen::new();
//! let node = NodeInfo::new(ids.fresh(), Span::new(0, 3));
//! assert_eq!(format!("{node:?}"), "#0@0..3");
//! ```

mod expr;
mod item;
mod module;
mod node;
mod parser;
mod pattern;
mod pretty;
mod stmt;
mod ty;

pub use expr::{BinaryOp, Block, Expr, ExprKind, FieldInit, MatchArm, UnaryOp};
pub use item::{Enum, FieldDef, Fn, Impl, Item, ItemKind, Mod, Param, Struct, Use, Variant};
pub use module::Module;
pub use node::{spanning, AstNode, HasSpan, Ident, NodeId, NodeIdGen, NodeInfo, Spanned};
pub use parser::{parse_module, ParseError, Parser};
pub use pattern::{FieldPattern, Pattern, PatternKind};
pub use pretty::{print_expr, print_item, print_pattern, print_stmt, print_type};
pub use stmt::{Stmt, StmtKind};
pub use ty::{Type, TypeKind};

/// re-exported so downstream crates don't need a `nex-lexer` dep just to name
/// a source location
pub use nex_lexer::Span;

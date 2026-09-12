//! how a statement finishes.
//!
//! `break`, `continue` and `return` all unwind past whatever is running, so
//! executing a statement reports back which of them happened rather than
//! just returning a value.

use nex_lexer::Span;

use crate::error::{Result, RuntimeError};
use crate::value::Value;

/// jumps carry the span of the keyword that caused them, so a `break` with
/// no loop to leave can be reported where it was written
#[derive(Clone, Debug, PartialEq)]
pub enum Flow {
    /// ran to completion, leaving this value
    Normal(Value),
    Break(Span),
    Continue(Span),
    Return(Value, Span),
}

impl Flow {
    /// whether this has to keep unwinding rather than being the value of
    /// the statement that produced it
    pub fn is_jump(&self) -> bool {
        !matches!(self, Flow::Normal(_))
    }

    /// the value if it finished normally, and unit otherwise
    pub fn value(self) -> Value {
        match self {
            Flow::Normal(v) => v,
            _ => Value::Unit,
        }
    }

    /// unwraps at a boundary a jump can't cross. a loop catches `break` and
    /// `continue` before this, and a call catches `return`, so anything left
    /// here had nothing to jump out of.
    pub fn into_value(self) -> Result<Value> {
        match self {
            Flow::Normal(value) => Ok(value),
            Flow::Break(span) => Err(RuntimeError::new("`break` outside a loop", span)),
            Flow::Continue(span) => Err(RuntimeError::new("`continue` outside a loop", span)),
            Flow::Return(_, span) => Err(RuntimeError::new("`return` outside a function", span)),
        }
    }
}

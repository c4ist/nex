//! how a statement finishes.
//!
//! `break`, `continue` and `return` all unwind past whatever is running, so
//! executing a statement reports back which of them happened rather than
//! just returning a value.

use crate::value::Value;

#[derive(Clone, Debug, PartialEq)]
pub enum Flow {
    /// ran to completion, leaving this value
    Normal(Value),
    Break,
    Continue,
    Return(Value),
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
}

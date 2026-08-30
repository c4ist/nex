//! the nex tree-walking interpreter.
//!
//! ```
//! use nex_interp::Value;
//!
//! assert_eq!(Value::Int(1 + 2).to_string(), "3");
//! ```

mod env;
mod value;

pub use env::Scope;
pub use value::{Builtin, EnumValue, FnValue, StructValue, Value};

//! the nex tree-walking interpreter.
//!
//! ```
//! use nex_interp::{eval_str, Value};
//!
//! assert_eq!(eval_str("1 + 2 * 3").unwrap(), Value::Int(7));
//! ```

mod env;
mod error;
mod eval;
mod flow;
mod value;

pub use env::Scope;
pub use error::{Result, RuntimeError};
pub use eval::Interpreter;
pub use flow::Flow;
pub use value::{Builtin, EnumValue, FnValue, StructValue, Value};

/// lexes, parses and runs a sequence of statements, giving the value of
/// the last one. mainly for tests until `nex run` exists.
pub fn run_str(src: &str) -> Result<Value> {
    // wrapping in braces reuses the block parser, so `let` and loops work
    eval_str(&format!("{{ {src} }}"))
}

/// lexes, parses and evaluates one expression. mainly for tests and the
/// doc example; real programs go through the driver.
pub fn eval_str(src: &str) -> Result<Value> {
    let (tokens, lex_errors) = nex_lexer::tokenize(src);
    if let Some(error) = lex_errors.first() {
        return Err(RuntimeError::new(error.kind.to_string(), error.span));
    }

    let (expr, parse_errors) = nex_syntax::parse_expr(&tokens);
    if let Some(error) = parse_errors.first() {
        return Err(RuntimeError::new(error.message.clone(), error.span));
    }

    Interpreter::new().eval(&expr)
}

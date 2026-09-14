//! the nex tree-walking interpreter.
//!
//! ```
//! use nex_interp::{eval_str, Value};
//!
//! assert_eq!(eval_str("1 + 2 * 3").unwrap(), Value::Int(7));
//! ```

mod builtins;
mod env;
mod error;
mod eval;
mod flow;
mod value;

pub use env::Scope;
pub use error::{Result, RuntimeError};
pub use eval::{Interpreter, MAX_CALL_DEPTH};
pub use flow::Flow;
pub use value::{Builtin, EnumValue, FnBody, FnValue, StructValue, Value};

/// lexes, parses and runs a whole module, giving the value `main` returns.
/// a module without a `main` is an error.
pub fn run_module(src: &str) -> Result<Value> {
    let tokens = lex(src)?;
    let (module, parse_errors) = nex_syntax::parse_module(&tokens);
    if let Some(error) = parse_errors.first() {
        return Err(RuntimeError::new(error.message.clone(), error.span));
    }

    let mut interpreter = Interpreter::new();
    interpreter.load_module(&module)?;

    let main = interpreter
        .globals()
        .lookup("main")
        .ok_or_else(|| RuntimeError::new("no `main` function to run", module.info.span))?;

    match main {
        Value::Fn(func) => interpreter.call(&func, Vec::new(), module.info.span),
        other => Err(RuntimeError::new(
            format!("`main` is a {}, not a function", other.type_name()),
            module.info.span,
        )),
    }
}

/// lexes, parses and runs a sequence of statements, giving the value of
/// the last one. mainly for tests until `nex run` exists.
pub fn run_str(src: &str) -> Result<Value> {
    // wrapping in braces reuses the block parser, so `let` and loops work
    eval_str(&format!("{{ {src} }}"))
}

/// lexes, parses and evaluates one expression. mainly for tests and the
/// doc example; real programs go through the driver.
pub fn eval_str(src: &str) -> Result<Value> {
    let tokens = lex(src)?;
    let (expr, parse_errors) = nex_syntax::parse_expr(&tokens);
    if let Some(error) = parse_errors.first() {
        return Err(RuntimeError::new(error.message.clone(), error.span));
    }

    Interpreter::new().eval(&expr)
}

fn lex(src: &str) -> Result<Vec<nex_lexer::Token>> {
    let (tokens, errors) = nex_lexer::tokenize(src);
    match errors.first() {
        Some(error) => Err(RuntimeError::new(error.kind.to_string(), error.span)),
        None => Ok(tokens),
    }
}

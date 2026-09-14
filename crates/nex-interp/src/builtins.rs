//! functions implemented in rust rather than nex.

use nex_lexer::Span;

use crate::error::{Result, RuntimeError};
use crate::value::{Builtin, Value};

pub const ALL: &[Builtin] = &[Builtin {
    name: "len",
    arity: Some(1),
}];

pub fn call(builtin: Builtin, args: Vec<Value>, span: Span) -> Result<Value> {
    if let Some(arity) = builtin.arity {
        if args.len() != arity {
            return Err(RuntimeError::new(
                format!(
                    "`{}` takes {} argument{}, but {} were given",
                    builtin.name,
                    arity,
                    if arity == 1 { "" } else { "s" },
                    args.len()
                ),
                span,
            ));
        }
    }

    match builtin.name {
        "len" => len(&args[0], span),
        other => Err(RuntimeError::new(
            format!("`{other}` is not implemented"),
            span,
        )),
    }
}

fn len(value: &Value, span: Span) -> Result<Value> {
    match value {
        Value::Array(items) => Ok(Value::Int(items.borrow().len() as i64)),
        // a str's length is its byte count, matching how spans index source
        Value::Str(s) => Ok(Value::Int(s.len() as i64)),
        other => Err(RuntimeError::new(
            format!("`len` needs an array or a str, not {}", other.type_name()),
            span,
        )),
    }
}

//! evaluating literals, identifiers and operators.

use nex_interp::{eval_str, Value};

#[track_caller]
fn eval(src: &str) -> Value {
    eval_str(src).unwrap_or_else(|e| panic!("{src:?} failed: {e}"))
}

#[track_caller]
fn error(src: &str) -> String {
    match eval_str(src) {
        Ok(v) => panic!("{src:?} unexpectedly produced {v}"),
        Err(e) => e.message,
    }
}

// the plan's criterion for this step
#[test]
fn arithmetic_honours_precedence() {
    assert_eq!(eval("1 + 2 * 3"), Value::Int(7));
    assert_eq!(eval("(1 + 2) * 3"), Value::Int(9));
    assert_eq!(eval("10 - 2 - 3"), Value::Int(5));
}

#[test]
fn evaluates_literals() {
    assert_eq!(eval("42"), Value::Int(42));
    assert_eq!(eval("2.5"), Value::Float(2.5));
    assert_eq!(eval("true"), Value::Bool(true));
    assert_eq!(eval("\"hi\""), Value::str("hi"));
    assert_eq!(eval("()"), Value::Unit);
}

#[test]
fn evaluates_integer_arithmetic() {
    assert_eq!(eval("7 / 2"), Value::Int(3));
    assert_eq!(eval("7 % 2"), Value::Int(1));
    assert_eq!(eval("-5 + 2"), Value::Int(-3));
}

#[test]
fn evaluates_float_arithmetic() {
    assert_eq!(eval("1.5 + 2.5"), Value::Float(4.0));
    assert_eq!(eval("7.0 / 2.0"), Value::Float(3.5));
}

#[test]
fn evaluates_comparisons() {
    assert_eq!(eval("1 < 2"), Value::Bool(true));
    assert_eq!(eval("2 <= 2"), Value::Bool(true));
    assert_eq!(eval("3 > 4"), Value::Bool(false));
    assert_eq!(eval("1 == 1"), Value::Bool(true));
    assert_eq!(eval("1 != 1"), Value::Bool(false));
}

#[test]
fn evaluates_logic() {
    assert_eq!(eval("true && false"), Value::Bool(false));
    assert_eq!(eval("true || false"), Value::Bool(true));
    assert_eq!(eval("!true"), Value::Bool(false));
    assert_eq!(eval("!(1 > 2)"), Value::Bool(true));
}

// the right side of a short-circuit isn't evaluated, so an error there
// stays invisible when the left side already decides the answer
#[test]
fn logical_operators_short_circuit() {
    assert_eq!(eval("false && undefined_name"), Value::Bool(false));
    assert_eq!(eval("true || undefined_name"), Value::Bool(true));

    // ...but the right side does run when it has to
    assert_eq!(
        error("true && undefined_name"),
        "`undefined_name` is not defined"
    );
}

#[test]
fn evaluates_bitwise_operators() {
    assert_eq!(eval("6 & 3"), Value::Int(2));
    assert_eq!(eval("6 | 3"), Value::Int(7));
    assert_eq!(eval("6 ^ 3"), Value::Int(5));
    assert_eq!(eval("1 << 4"), Value::Int(16));
    assert_eq!(eval("16 >> 4"), Value::Int(1));
}

#[test]
fn concatenates_and_compares_strings() {
    assert_eq!(eval("\"a\" + \"b\""), Value::str("ab"));
    assert_eq!(eval("\"a\" < \"b\""), Value::Bool(true));
    assert_eq!(eval("\"a\" == \"a\""), Value::Bool(true));
}

#[test]
fn an_undefined_name_is_a_runtime_error() {
    assert_eq!(error("missing"), "`missing` is not defined");
}

// there are no implicit conversions, so mixing numeric types is an error
// rather than quietly coercing
#[test]
fn mixed_numeric_types_are_rejected() {
    assert_eq!(error("1 + 1.0"), "cannot apply `+` to i32 and f64");
    assert_eq!(error("1 == 1.0"), "cannot apply `==` to i32 and f64");
}

// only bool is truthy
#[test]
fn non_bools_are_not_conditions() {
    assert_eq!(error("1 && true"), "expected bool, found i32");
    assert_eq!(error("!1"), "cannot apply `!` to i32");
}

#[test]
fn division_by_zero_is_a_runtime_error() {
    assert_eq!(error("1 / 0"), "division by zero");
    assert_eq!(error("1 % 0"), "remainder by zero");
}

// arithmetic is checked, so overflow reports instead of wrapping silently
#[test]
fn integer_overflow_is_reported() {
    assert_eq!(error("9223372036854775807 + 1"), "integer overflow");
    assert_eq!(error("-9223372036854775807 - 2"), "integer overflow");
}

#[test]
fn out_of_range_shifts_are_reported() {
    assert_eq!(error("1 << 64"), "shift amount 64 is out of range");
    assert_eq!(error("1 << -1"), "shift amount -1 is out of range");
}

#[test]
fn float_division_by_zero_follows_ieee() {
    // unlike integers, floats have infinity for this
    assert_eq!(eval("1.0 / 0.0"), Value::Float(f64::INFINITY));
}

#[test]
fn unsupported_expressions_say_so() {
    assert_eq!(error("f(1)"), "calling a function is not supported yet");
    assert_eq!(error("{ 1 }"), "a block is not supported yet");
}

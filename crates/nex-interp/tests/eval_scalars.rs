//! floats, strings, and the comparison operators in detail.

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

// floats

#[test]
fn float_arithmetic_covers_every_operator() {
    assert_eq!(eval("1.5 + 2.25"), Value::Float(3.75));
    assert_eq!(eval("1.5 - 2.25"), Value::Float(-0.75));
    assert_eq!(eval("1.5 * 2.0"), Value::Float(3.0));
    assert_eq!(eval("3.0 / 2.0"), Value::Float(1.5));
    assert_eq!(eval("7.5 % 2.0"), Value::Float(1.5));
    assert_eq!(eval("-2.5"), Value::Float(-2.5));
}

#[test]
fn float_comparisons_cover_every_operator() {
    assert_eq!(eval("1.0 < 2.0"), Value::Bool(true));
    assert_eq!(eval("2.0 <= 2.0"), Value::Bool(true));
    assert_eq!(eval("3.0 > 2.0"), Value::Bool(true));
    assert_eq!(eval("2.0 >= 3.0"), Value::Bool(false));
    assert_eq!(eval("1.5 == 1.5"), Value::Bool(true));
    assert_eq!(eval("1.5 != 1.5"), Value::Bool(false));
}

#[test]
fn float_exponents_evaluate() {
    assert_eq!(eval("1e3"), Value::Float(1000.0));
    assert_eq!(eval("2.5e-3"), Value::Float(0.0025));
}

// division by zero is an error for ints but infinity for floats, which is
// what ieee says and what every other language with f64 does
#[test]
fn float_division_by_zero_gives_infinity_not_an_error() {
    assert_eq!(eval("1.0 / 0.0"), Value::Float(f64::INFINITY));
    assert_eq!(eval("-1.0 / 0.0"), Value::Float(f64::NEG_INFINITY));
}

// nan is not equal to itself, so `x == x` being false is correct here
#[test]
fn nan_is_not_equal_to_itself() {
    assert_eq!(eval("(0.0 / 0.0) == (0.0 / 0.0)"), Value::Bool(false));
    assert_eq!(eval("(0.0 / 0.0) != (0.0 / 0.0)"), Value::Bool(true));
    assert_eq!(eval("(0.0 / 0.0) < 1.0"), Value::Bool(false));
    assert_eq!(eval("(0.0 / 0.0) >= 1.0"), Value::Bool(false));
}

// bitwise operators are integer-only
#[test]
fn floats_reject_bitwise_operators() {
    assert_eq!(error("1.0 & 2.0"), "cannot apply `&` to f64 and f64");
    assert_eq!(error("1.0 << 2.0"), "cannot apply `<<` to f64 and f64");
}

// strings

#[test]
fn string_concatenation_chains() {
    assert_eq!(eval("\"a\" + \"b\" + \"c\""), Value::str("abc"));
    assert_eq!(eval("\"\" + \"x\""), Value::str("x"));
}

#[test]
fn string_concatenation_keeps_escapes_and_unicode() {
    assert_eq!(eval("\"a\\nb\""), Value::str("a\nb"));
    assert_eq!(eval("\"caf\" + \"é\""), Value::str("café"));
    assert_eq!(eval("\"\\t\" + \"x\""), Value::str("\tx"));
}

#[test]
fn strings_compare_lexicographically() {
    assert_eq!(eval("\"a\" < \"b\""), Value::Bool(true));
    assert_eq!(eval("\"b\" < \"a\""), Value::Bool(false));
    assert_eq!(eval("\"abc\" < \"abd\""), Value::Bool(true));
    assert_eq!(eval("\"a\" < \"ab\""), Value::Bool(true));
    assert_eq!(eval("\"a\" <= \"a\""), Value::Bool(true));
    assert_eq!(eval("\"b\" >= \"a\""), Value::Bool(true));
}

#[test]
fn string_equality_is_by_content() {
    assert_eq!(eval("\"abc\" == \"abc\""), Value::Bool(true));
    assert_eq!(eval("\"abc\" == \"abd\""), Value::Bool(false));
    assert_eq!(eval("\"\" == \"\""), Value::Bool(true));
}

// concatenation is the only arithmetic strings support
#[test]
fn strings_reject_the_other_arithmetic_operators() {
    assert_eq!(error("\"a\" - \"b\""), "cannot apply `-` to str and str");
    assert_eq!(error("\"a\" * \"b\""), "cannot apply `*` to str and str");
    assert_eq!(error("\"a\" / \"b\""), "cannot apply `/` to str and str");
}

// there's no `"a" + 1`; the types have to match
#[test]
fn strings_do_not_mix_with_other_types() {
    assert_eq!(error("\"a\" + 1"), "cannot apply `+` to str and i32");
    assert_eq!(error("1 + \"a\""), "cannot apply `+` to i32 and str");
    assert_eq!(error("\"a\" == 1"), "cannot apply `==` to str and i32");
}

// comparisons across the board

#[test]
fn bools_compare_for_equality() {
    assert_eq!(eval("true == true"), Value::Bool(true));
    assert_eq!(eval("true == false"), Value::Bool(false));
    assert_eq!(eval("true != false"), Value::Bool(true));
}

// ordering is for numbers and strings; `true < false` is meaningless
#[test]
fn bools_do_not_have_an_ordering() {
    assert_eq!(error("true < false"), "cannot apply `<` to bool and bool");
}

#[test]
fn unit_compares_only_with_unit() {
    assert_eq!(eval("() == ()"), Value::Bool(true));
    assert_eq!(eval("() != ()"), Value::Bool(false));
    assert_eq!(error("() == 1"), "cannot apply `==` to () and i32");
}

// comparison binds looser than arithmetic, so this is (1+1) == 2
#[test]
fn comparisons_compose_with_arithmetic() {
    assert_eq!(eval("1 + 1 == 2"), Value::Bool(true));
    assert_eq!(eval("2 * 3 > 5"), Value::Bool(true));
    assert_eq!(eval("1.0 + 0.5 == 1.5"), Value::Bool(true));
    assert_eq!(eval("\"a\" + \"b\" == \"ab\""), Value::Bool(true));
}

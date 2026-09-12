//! function definitions and calls.

use nex_interp::{run_module, Value};

#[track_caller]
fn run(src: &str) -> Value {
    run_module(src).unwrap_or_else(|e| panic!("failed: {e}\n--- source ---\n{src}"))
}

#[track_caller]
fn error(src: &str) -> String {
    match run_module(src) {
        Ok(v) => panic!("unexpectedly produced {v} for:\n{src}"),
        Err(e) => e.message,
    }
}

// the plan's criterion for this step
#[test]
fn computes_fibonacci_recursively() {
    let value = run(r#"
        fn fib(n: i32) -> i32 {
            if n < 2 { n } else { fib(n - 1) + fib(n - 2) }
        }

        fn main() -> i32 { fib(20) }
    "#);
    assert_eq!(value, Value::Int(6765));
}

#[test]
fn a_function_with_no_arguments_runs() {
    assert_eq!(run("fn main() -> i32 { 42 }"), Value::Int(42));
}

#[test]
fn arguments_bind_to_parameters_in_order() {
    let value = run(r#"
        fn sub(a: i32, b: i32) -> i32 { a - b }
        fn main() -> i32 { sub(10, 3) }
    "#);
    assert_eq!(value, Value::Int(7), "arguments must not be swapped");
}

// the body's last expression is the value, with no `return` needed
#[test]
fn the_last_expression_is_the_return_value() {
    let value = run(r#"
        fn double(n: i32) -> i32 { n * 2 }
        fn main() -> i32 { double(21) }
    "#);
    assert_eq!(value, Value::Int(42));
}

#[test]
fn an_explicit_return_ends_the_function_early() {
    let value = run(r#"
        fn first_even(limit: i32) -> i32 {
            for i in 0..limit {
                if i % 2 == 0 { return i; }
            }
            -1
        }
        fn main() -> i32 { first_even(5) }
    "#);
    assert_eq!(value, Value::Int(0));
}

#[test]
fn a_return_with_no_value_is_unit() {
    assert_eq!(run("fn main() { return; }"), Value::Unit);
}

#[test]
fn a_function_body_that_ends_in_a_statement_is_unit() {
    assert_eq!(run("fn main() { let x = 1; }"), Value::Unit);
}

// a call can be written before the definition it refers to
#[test]
fn functions_can_be_called_before_they_are_defined() {
    let value = run(r#"
        fn main() -> i32 { helper() }
        fn helper() -> i32 { 7 }
    "#);
    assert_eq!(value, Value::Int(7));
}

#[test]
fn functions_can_call_each_other() {
    let value = run(r#"
        fn is_even(n: i32) -> bool { n % 2 == 0 }
        fn count_evens(limit: i32) -> i32 {
            let mut total = 0;
            for i in 0..limit {
                if is_even(i) { total += 1; }
            }
            total
        }
        fn main() -> i32 { count_evens(10) }
    "#);
    assert_eq!(value, Value::Int(5));
}

#[test]
fn arguments_are_evaluated_before_the_call() {
    let value = run(r#"
        fn add(a: i32, b: i32) -> i32 { a + b }
        fn main() -> i32 { add(1 + 1, add(3, 4)) }
    "#);
    assert_eq!(value, Value::Int(9));
}

// each call gets its own frame, so recursion doesn't share a parameter
#[test]
fn each_call_gets_its_own_parameters() {
    let value = run(r#"
        fn countdown(n: i32) -> i32 {
            if n == 0 { 0 } else { n + countdown(n - 1) }
        }
        fn main() -> i32 { countdown(10) }
    "#);
    assert_eq!(value, Value::Int(55));
}

// parameters are copies of the argument, so writing to one is invisible
// to the caller
#[test]
fn a_parameter_can_be_reassigned_without_touching_the_caller() {
    let value = run(r#"
        fn consume(n: i32) -> i32 {
            n = 0;
            n
        }
        fn main() -> i32 {
            let x = 5;
            consume(x);
            x
        }
    "#);
    assert_eq!(value, Value::Int(5));
}

// scoping is lexical: a function sees globals, not its caller's locals
#[test]
fn a_function_cannot_see_its_callers_locals() {
    let src = r#"
        fn peek() -> i32 { hidden }
        fn main() -> i32 {
            let hidden = 1;
            peek()
        }
    "#;
    assert_eq!(error(src), "`hidden` is not defined");
}

#[test]
fn a_local_does_not_escape_the_function() {
    let src = r#"
        fn helper() -> i32 { let inner = 1; inner }
        fn main() -> i32 { helper(); inner }
    "#;
    assert_eq!(error(src), "`inner` is not defined");
}

#[test]
fn a_parameter_shadows_a_function_of_the_same_name() {
    let value = run(r#"
        fn shadow(helper: i32) -> i32 { helper }
        fn helper() -> i32 { 1 }
        fn main() -> i32 { shadow(99) }
    "#);
    assert_eq!(value, Value::Int(99));
}

// errors

#[test]
fn calling_with_the_wrong_number_of_arguments_is_an_error() {
    let too_few = r#"
        fn add(a: i32, b: i32) -> i32 { a + b }
        fn main() -> i32 { add(1) }
    "#;
    assert_eq!(error(too_few), "`add` takes 2 arguments, but 1 were given");

    let too_many = r#"
        fn one(a: i32) -> i32 { a }
        fn main() -> i32 { one(1, 2) }
    "#;
    assert_eq!(error(too_many), "`one` takes 1 argument, but 2 were given");
}

#[test]
fn calling_something_that_is_not_a_function_is_an_error() {
    let src = r#"
        fn main() -> i32 {
            let x = 1;
            x()
        }
    "#;
    assert_eq!(error(src), "i32 is not callable");
}

#[test]
fn calling_an_undefined_function_is_an_error() {
    assert_eq!(error("fn main() { nope(); }"), "`nope` is not defined");
}

#[test]
fn two_functions_cannot_share_a_name() {
    let src = r#"
        fn twice() -> i32 { 1 }
        fn twice() -> i32 { 2 }
        fn main() -> i32 { twice() }
    "#;
    assert_eq!(error(src), "`twice` is defined more than once");
}

#[test]
fn a_module_without_a_main_is_an_error() {
    assert_eq!(
        error("fn helper() -> i32 { 1 }"),
        "no `main` function to run"
    );
}

// runaway recursion stops cleanly rather than overflowing the rust stack
#[test]
fn endless_recursion_hits_a_depth_limit() {
    let src = r#"
        fn forever(n: i32) -> i32 { forever(n + 1) }
        fn main() -> i32 { forever(0) }
    "#;
    assert_eq!(error(src), "recursion went deeper than 256 calls");
}

// the limit is per nesting level, not a budget for the whole program
#[test]
fn the_depth_limit_resets_between_calls() {
    let value = run(r#"
        fn depth(n: i32) -> i32 {
            if n == 0 { 0 } else { depth(n - 1) }
        }
        fn main() -> i32 {
            let mut total = 0;
            for i in 0..10 {
                total += depth(100);
            }
            total
        }
    "#);
    assert_eq!(value, Value::Int(0));
}

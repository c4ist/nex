//! closures and the scope they capture.

use nex_interp::{run_module, run_str, Value};

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

// the plan's criterion for this step: a counter that keeps its own state
#[test]
fn a_closure_keeps_the_scope_it_captured() {
    let value = run(r#"
        fn make_counter() {
            let mut count = 0;
            || { count += 1; count }
        }

        fn main() -> i32 {
            let next = make_counter();
            next();
            next();
            next()
        }
    "#);
    assert_eq!(value, Value::Int(3), "the capture must outlive the call");
}

// each call to the maker gets a fresh frame, so the counters are separate
#[test]
fn two_closures_capture_separate_scopes() {
    let value = run(r#"
        fn make_counter() {
            let mut count = 0;
            || { count += 1; count }
        }

        fn main() -> i32 {
            let a = make_counter();
            let b = make_counter();
            a();
            a();
            a() * 10 + b()
        }
    "#);
    assert_eq!(value, Value::Int(31), "b must start from zero");
}

// calling

#[test]
fn a_closure_can_be_called() {
    assert_eq!(
        run("fn main() -> i32 { let f = |x| x + 1; f(1) }"),
        Value::Int(2)
    );
}

#[test]
fn a_closure_with_no_parameters_can_be_called() {
    assert_eq!(run("fn main() -> i32 { let f = || 7; f() }"), Value::Int(7));
}

#[test]
fn a_closure_takes_several_arguments() {
    let value = run("fn main() -> i32 { let f = |a, b, c| a * 100 + b * 10 + c; f(1, 2, 3) }");
    assert_eq!(value, Value::Int(123));
}

#[test]
fn a_closure_body_can_be_a_block() {
    let value = run(r#"
        fn main() -> i32 {
            let f = |n| {
                let doubled = n * 2;
                doubled + 1
            };
            f(5)
        }
    "#);
    assert_eq!(value, Value::Int(11));
}

#[test]
fn a_closure_can_be_called_immediately() {
    assert_eq!(run("fn main() -> i32 { (|x| x * 2)(21) }"), Value::Int(42));
}

// capturing

#[test]
fn a_closure_reads_the_scope_around_it() {
    let value = run(r#"
        fn main() -> i32 {
            let base = 10;
            let add_base = |n| n + base;
            add_base(5)
        }
    "#);
    assert_eq!(value, Value::Int(15));
}

// the capture is the scope itself, not a copy of its values
#[test]
fn a_closure_sees_changes_made_after_it_was_created() {
    let value = run(r#"
        fn main() -> i32 {
            let mut base = 1;
            let read = || base;
            base = 99;
            read()
        }
    "#);
    assert_eq!(value, Value::Int(99));
}

#[test]
fn a_closure_can_write_to_what_it_captured() {
    let value = run(r#"
        fn main() -> i32 {
            let mut total = 0;
            let add = |n| { total += n; };
            add(3);
            add(4);
            total
        }
    "#);
    assert_eq!(value, Value::Int(7));
}

#[test]
fn a_parameter_shadows_a_captured_binding() {
    let value = run(r#"
        fn main() -> i32 {
            let x = 1;
            let f = |x| x;
            f(99)
        }
    "#);
    assert_eq!(value, Value::Int(99));
}

// a closure's own locals don't leak into the scope it captured
#[test]
fn closure_locals_do_not_escape() {
    let src = r#"
        fn main() -> i32 {
            let f = || { let inner = 1; inner };
            f();
            inner
        }
    "#;
    assert_eq!(error(src), "`inner` is not defined");
}

#[test]
fn a_closure_captures_a_loop_binding() {
    let value = run(r#"
        fn main() -> i32 {
            let mut total = 0;
            for i in 0..4 {
                let read = || i;
                total += read();
            }
            total
        }
    "#);
    assert_eq!(value, Value::Int(6));
}

// closures nest, so the inner one reaches through the outer one's scope
#[test]
fn closures_nest() {
    let value = run(r#"
        fn main() -> i32 {
            let add = |a| |b| a + b;
            let add_two = add(2);
            add_two(40)
        }
    "#);
    assert_eq!(value, Value::Int(42));
}

// the binding exists by the time the closure runs, so it can call itself
#[test]
fn a_closure_can_recurse_through_its_own_binding() {
    let value = run(r#"
        fn main() -> i32 {
            let countdown = |n| if n == 0 { 0 } else { n + countdown(n - 1) };
            countdown(4)
        }
    "#);
    assert_eq!(value, Value::Int(10));
}

// passing and returning

#[test]
fn a_closure_can_be_passed_to_a_function() {
    let value = run(r#"
        fn apply_twice(f: fn(i32) -> i32, n: i32) -> i32 { f(f(n)) }
        fn main() -> i32 { apply_twice(|x| x * 3, 2) }
    "#);
    assert_eq!(value, Value::Int(18));
}

#[test]
fn a_named_function_can_be_passed_as_a_value() {
    let value = run(r#"
        fn double(n: i32) -> i32 { n * 2 }
        fn apply(f: fn(i32) -> i32, n: i32) -> i32 { f(n) }
        fn main() -> i32 { apply(double, 21) }
    "#);
    assert_eq!(value, Value::Int(42));
}

// a named function still can't see its caller's locals
#[test]
fn a_named_function_captures_the_globals_not_its_caller() {
    let src = r#"
        fn peek() -> i32 { hidden }
        fn main() -> i32 {
            let hidden = 1;
            peek()
        }
    "#;
    assert_eq!(error(src), "`hidden` is not defined");
}

// errors

#[test]
fn a_closure_reports_the_wrong_argument_count() {
    let src = "fn main() { let f = |a, b| a; f(1); }";
    assert_eq!(error(src), "closure takes 2 arguments, but 1 were given");
}

#[test]
fn a_closure_displays_as_a_closure() {
    assert_eq!(run_str("|x| x").unwrap().to_string(), "closure");
}

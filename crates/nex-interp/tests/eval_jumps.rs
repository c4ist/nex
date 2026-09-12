//! where `return`, `break` and `continue` stop unwinding.

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

// the plan's criterion for this step
#[test]
fn return_leaves_the_function_immediately() {
    let value = run(r#"
        fn find(limit: i32, target: i32) -> i32 {
            let mut checked = 0;
            for i in 0..limit {
                checked += 1;
                if i == target { return checked; }
            }
            -1
        }
        fn main() -> i32 { find(100, 3) }
    "#);
    assert_eq!(value, Value::Int(4), "the loop must stop at the target");
}

// return

#[test]
fn return_unwinds_out_of_nested_loops() {
    let value = run(r#"
        fn search() -> i32 {
            for i in 0..10 {
                for j in 0..10 {
                    if i * j == 12 { return i * 100 + j; }
                }
            }
            -1
        }
        fn main() -> i32 { search() }
    "#);
    assert_eq!(value, Value::Int(206), "i=2, j=6 is the first match");
}

#[test]
fn return_unwinds_out_of_a_block() {
    let value = run(r#"
        fn early() -> i32 {
            {
                {
                    return 1;
                }
            }
            2
        }
        fn main() -> i32 { early() }
    "#);
    assert_eq!(value, Value::Int(1));
}

// a return inside an expression abandons the expression
#[test]
fn return_unwinds_out_of_an_expression() {
    let value = run(r#"
        fn early() -> i32 {
            let x = 1 + { return 9; };
            x
        }
        fn main() -> i32 { early() }
    "#);
    assert_eq!(value, Value::Int(9));
}

#[test]
fn return_stops_at_the_function_it_is_in() {
    let value = run(r#"
        fn inner() -> i32 { return 1; }
        fn outer() -> i32 {
            inner();
            2
        }
        fn main() -> i32 { outer() }
    "#);
    assert_eq!(value, Value::Int(2), "inner's return must not exit outer");
}

#[test]
fn statements_after_a_return_do_not_run() {
    let value = run(r#"
        fn early() -> i32 {
            return 1;
            undefined_name
        }
        fn main() -> i32 { early() }
    "#);
    assert_eq!(value, Value::Int(1));
}

// break and continue

#[test]
fn break_stops_at_the_nearest_loop() {
    let value = run(r#"
        fn main() -> i32 {
            let mut total = 0;
            for i in 0..3 {
                for j in 0..10 {
                    if j == 2 { break; }
                    total += 1;
                }
                total += 100;
            }
            total
        }
    "#);
    assert_eq!(value, Value::Int(306), "the outer loop must keep running");
}

#[test]
fn break_unwinds_out_of_a_nested_block() {
    let value = run(r#"
        fn main() -> i32 {
            let mut i = 0;
            while true {
                i += 1;
                {
                    if i == 3 { break; }
                }
            }
            i
        }
    "#);
    assert_eq!(value, Value::Int(3));
}

// a jump in value position still reaches the loop
#[test]
fn break_unwinds_out_of_an_expression() {
    let value = run(r#"
        fn main() -> i32 {
            let mut i = 0;
            while i < 10 {
                i += 1;
                let ignored = if i == 3 { { break; } } else { 0 };
            }
            i
        }
    "#);
    assert_eq!(value, Value::Int(3));
}

#[test]
fn continue_stops_at_the_nearest_loop() {
    let value = run(r#"
        fn main() -> i32 {
            let mut total = 0;
            for i in 0..3 {
                for j in 0..4 {
                    if j % 2 == 1 { continue; }
                    total += 1;
                }
            }
            total
        }
    "#);
    assert_eq!(value, Value::Int(6));
}

// a jump with nothing to jump out of

#[test]
fn break_outside_a_loop_is_an_error() {
    assert_eq!(error("fn main() { break; }"), "`break` outside a loop");
    assert_eq!(
        run_str("break;").unwrap_err().message,
        "`break` outside a loop"
    );
}

#[test]
fn continue_outside_a_loop_is_an_error() {
    assert_eq!(
        error("fn main() { continue; }"),
        "`continue` outside a loop"
    );
    assert_eq!(
        run_str("continue;").unwrap_err().message,
        "`continue` outside a loop"
    );
}

// a loop body is a function boundary away, so this one has no loop either
#[test]
fn a_break_cannot_cross_a_function_boundary() {
    let src = r#"
        fn helper() { break; }
        fn main() {
            while true { helper(); }
        }
    "#;
    assert_eq!(error(src), "`break` outside a loop");
}

#[test]
fn return_outside_a_function_is_an_error() {
    assert_eq!(
        run_str("return 1;").unwrap_err().message,
        "`return` outside a function"
    );
}

// the error points at the keyword, not at whatever contained it
#[test]
fn a_stray_jump_is_reported_where_it_was_written() {
    let src = "fn main() {\n    let x = 1;\n    break;\n}";
    let error = run_module(src).unwrap_err();
    let start = error.span.start as usize;
    assert_eq!(&src[start..start + 5], "break");
}

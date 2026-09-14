//! array literals, indexing and `len`.

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
fn builds_indexes_and_measures_an_array() {
    let value = run(r#"
        fn sum(items: [i32]) -> i32 {
            let mut total = 0;
            for i in 0..len(items) {
                total += items[i];
            }
            total
        }

        fn main() -> i32 {
            let numbers = [1, 2, 3, 4, 5];
            sum(numbers)
        }
    "#);
    assert_eq!(value, Value::Int(15));
}

// literals

#[test]
fn an_array_literal_keeps_its_order() {
    let value = run("fn main() -> i32 { let a = [10, 20, 30]; a[0] * 100 + a[2] }");
    assert_eq!(value, Value::Int(1030));
}

#[test]
fn an_empty_array_has_no_elements() {
    assert_eq!(run("fn main() -> i32 { len([]) }"), Value::Int(0));
}

#[test]
fn array_elements_are_ordinary_expressions() {
    let value = run(r#"
        fn double(n: i32) -> i32 { n * 2 }
        fn main() -> i32 {
            let n = 5;
            let a = [n + 1, double(n), 1];
            a[0] * 100 + a[1]
        }
    "#);
    assert_eq!(value, Value::Int(610));
}

#[test]
fn arrays_hold_any_value() {
    assert_eq!(
        run("fn main() -> bool { [true, false][0] }"),
        Value::Bool(true)
    );
    assert_eq!(run("fn main() -> f64 { [1.5, 2.5][1] }"), Value::Float(2.5));
    assert_eq!(run("fn main() { [\"a\", \"b\"][1] }"), Value::str("b"));
}

#[test]
fn arrays_nest() {
    let value = run("fn main() -> i32 { let grid = [[1, 2], [3, 4]]; grid[1][0] }");
    assert_eq!(value, Value::Int(3));
}

#[test]
fn an_array_can_hold_structs() {
    let value = run(r#"
        struct Point { x: i32, y: i32 }
        fn main() -> i32 {
            let points = [Point { x: 1, y: 2 }, Point { x: 3, y: 4 }];
            points[1].x
        }
    "#);
    assert_eq!(value, Value::Int(3));
}

#[test]
fn an_array_displays_with_its_elements() {
    let value = run("fn main() { [1, 2, 3] }");
    assert_eq!(value.to_string(), "[1, 2, 3]");
}

// indexing

#[test]
fn indices_are_zero_based() {
    let value = run("fn main() -> i32 { let a = [7, 8, 9]; a[0] }");
    assert_eq!(value, Value::Int(7));
}

#[test]
fn an_index_is_an_ordinary_expression() {
    let value = run("fn main() -> i32 { let a = [7, 8, 9]; let i = 1; a[i + 1] }");
    assert_eq!(value, Value::Int(9));
}

#[test]
fn a_literal_can_be_indexed_directly() {
    assert_eq!(run("fn main() -> i32 { [1, 2, 3][1] }"), Value::Int(2));
}

// len

#[test]
fn len_counts_elements() {
    assert_eq!(run("fn main() -> i32 { len([1, 2, 3]) }"), Value::Int(3));
}

// a str's length is its byte count, the same unit spans use
#[test]
fn len_measures_a_str_in_bytes() {
    assert_eq!(run("fn main() -> i32 { len(\"abc\") }"), Value::Int(3));
    assert_eq!(run("fn main() -> i32 { len(\"café\") }"), Value::Int(5));
    assert_eq!(run("fn main() -> i32 { len(\"\") }"), Value::Int(0));
}

#[test]
fn len_can_be_passed_around_like_any_function() {
    let value = run(r#"
        fn apply(f: fn([i32]) -> i32, a: [i32]) -> i32 { f(a) }
        fn main() -> i32 { apply(len, [1, 2]) }
    "#);
    assert_eq!(value, Value::Int(2));
}

// sharing

// arrays are shared rather than copied, so two bindings see one array
#[test]
fn an_array_binding_shares_the_same_array() {
    let value = run(r#"
        fn main() -> bool {
            let a = [1, 2];
            let b = a;
            a == b
        }
    "#);
    assert_eq!(value, Value::Bool(true));
}

// equality is by content, so two separately built arrays match
#[test]
fn arrays_compare_by_their_elements() {
    assert_eq!(
        run("fn main() -> bool { [1, 2] == [1, 2] }"),
        Value::Bool(true)
    );
    assert_eq!(
        run("fn main() -> bool { [1, 2] == [1, 3] }"),
        Value::Bool(false)
    );
    assert_eq!(
        run("fn main() -> bool { [1] == [1, 2] }"),
        Value::Bool(false)
    );
}

// errors

#[test]
fn reading_past_the_end_is_an_error() {
    let src = "fn main() -> i32 { let a = [1, 2]; a[2] }";
    assert_eq!(error(src), "index 2 is out of bounds for an array of 2");
}

#[test]
fn a_negative_index_is_an_error() {
    let src = "fn main() -> i32 { let a = [1, 2]; a[-1] }";
    assert_eq!(error(src), "index -1 is out of bounds for an array of 2");
}

#[test]
fn indexing_an_empty_array_is_an_error() {
    assert_eq!(
        error("fn main() -> i32 { [][0] }"),
        "index 0 is out of bounds for an array of 0"
    );
}

#[test]
fn an_index_has_to_be_an_integer() {
    let src = "fn main() -> i32 { let a = [1, 2]; a[1.0] }";
    assert_eq!(error(src), "expected i32, found f64");
}

#[test]
fn indexing_a_non_array_is_an_error() {
    assert_eq!(
        error("fn main() -> i32 { let n = 1; n[0] }"),
        "cannot index i32"
    );
    assert_eq!(error("fn main() { \"abc\"[0] }"), "cannot index str");
}

#[test]
fn len_rejects_values_it_cannot_measure() {
    assert_eq!(
        error("fn main() -> i32 { len(1) }"),
        "`len` needs an array or a str, not i32"
    );
}

#[test]
fn len_takes_exactly_one_argument() {
    assert_eq!(
        error("fn main() -> i32 { len() }"),
        "`len` takes 1 argument, but 0 were given"
    );
    assert_eq!(
        error("fn main() -> i32 { len([1], [2]) }"),
        "`len` takes 1 argument, but 2 were given"
    );
}

// mutation through an index arrives with field mutation
#[test]
fn assigning_to_an_index_is_not_supported_yet() {
    let src = "fn main() -> i32 { let a = [1, 2]; a[0] = 5; a[0] }";
    assert_eq!(error(src), "only variables can be assigned to for now");
}

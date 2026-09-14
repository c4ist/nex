//! struct literals and field access.

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

// the plan's criterion for this step. there's no sqrt until builtins land,
// so this is the squared distance
#[test]
fn computes_the_distance_between_two_points() {
    let value = run(r#"
        struct Point { x: f64, y: f64 }

        fn distance_squared(a: Point, b: Point) -> f64 {
            let dx = a.x - b.x;
            let dy = a.y - b.y;
            dx * dx + dy * dy
        }

        fn main() -> f64 {
            let origin = Point { x: 0.0, y: 0.0 };
            let p = Point { x: 3.0, y: 4.0 };
            distance_squared(origin, p)
        }
    "#);
    assert_eq!(value, Value::Float(25.0));
}

// literals

#[test]
fn builds_a_struct_and_reads_its_fields() {
    let value = run(r#"
        struct Point { x: i32, y: i32 }
        fn main() -> i32 {
            let p = Point { x: 1, y: 2 };
            p.x * 10 + p.y
        }
    "#);
    assert_eq!(value, Value::Int(12));
}

// field order in the literal doesn't have to match the declaration
#[test]
fn fields_can_be_given_in_any_order() {
    let value = run(r#"
        struct Point { x: i32, y: i32 }
        fn main() -> i32 {
            let p = Point { y: 2, x: 1 };
            p.x * 10 + p.y
        }
    "#);
    assert_eq!(value, Value::Int(12));
}

#[test]
fn a_struct_with_one_field_works() {
    let value = run(r#"
        struct Wrapper { value: i32 }
        fn main() -> i32 { Wrapper { value: 7 }.value }
    "#);
    assert_eq!(value, Value::Int(7));
}

#[test]
fn field_values_are_ordinary_expressions() {
    let value = run(r#"
        struct Point { x: i32, y: i32 }
        fn double(n: i32) -> i32 { n * 2 }
        fn main() -> i32 {
            let n = 3;
            let p = Point { x: n + 1, y: double(n) };
            p.x * 10 + p.y
        }
    "#);
    assert_eq!(value, Value::Int(46));
}

#[test]
fn structs_nest() {
    let value = run(r#"
        struct Point { x: i32, y: i32 }
        struct Line { from: Point, to: Point }
        fn main() -> i32 {
            let line = Line {
                from: Point { x: 1, y: 2 },
                to: Point { x: 3, y: 4 },
            };
            line.to.x * 10 + line.from.y
        }
    "#);
    assert_eq!(value, Value::Int(32));
}

// a struct can be declared after the function that uses it
#[test]
fn a_struct_can_be_used_before_it_is_declared() {
    let value = run(r#"
        fn main() -> i32 { Point { x: 5 }.x }
        struct Point { x: i32 }
    "#);
    assert_eq!(value, Value::Int(5));
}

// passing and returning

#[test]
fn a_struct_can_be_returned_from_a_function() {
    let value = run(r#"
        struct Point { x: i32, y: i32 }
        fn origin() -> Point { Point { x: 0, y: 0 } }
        fn main() -> i32 { origin().x }
    "#);
    assert_eq!(value, Value::Int(0));
}

#[test]
fn a_struct_can_be_captured_by_a_closure() {
    let value = run(r#"
        struct Point { x: i32, y: i32 }
        fn main() -> i32 {
            let p = Point { x: 4, y: 0 };
            let read_x = || p.x;
            read_x()
        }
    "#);
    assert_eq!(value, Value::Int(4));
}

// structs are shared rather than copied, so two bindings see one value
#[test]
fn a_struct_binding_shares_the_same_value() {
    let value = run(r#"
        struct Point { x: i32, y: i32 }
        fn main() -> bool {
            let a = Point { x: 1, y: 2 };
            let b = a;
            a == b
        }
    "#);
    assert_eq!(value, Value::Bool(true));
}

// equality is by content, so two separately built points match
#[test]
fn structs_compare_by_their_fields() {
    let value = run(r#"
        struct Point { x: i32, y: i32 }
        fn main() -> bool {
            Point { x: 1, y: 2 } == Point { x: 1, y: 2 }
        }
    "#);
    assert_eq!(value, Value::Bool(true));

    let differs = run(r#"
        struct Point { x: i32, y: i32 }
        fn main() -> bool {
            Point { x: 1, y: 2 } == Point { x: 1, y: 3 }
        }
    "#);
    assert_eq!(differs, Value::Bool(false));
}

#[test]
fn a_struct_displays_with_its_fields() {
    let value = run(r#"
        struct Point { x: i32, y: i32 }
        fn main() { Point { x: 1, y: 2 } }
    "#);
    assert_eq!(value.to_string(), "Point { x: 1, y: 2 }");
}

// errors

#[test]
fn an_undeclared_struct_is_an_error() {
    assert_eq!(
        error("fn main() { Nope { x: 1 } }"),
        "`Nope` is not a struct"
    );
}

#[test]
fn an_unknown_field_is_an_error() {
    let src = r#"
        struct Point { x: i32, y: i32 }
        fn main() { Point { x: 1, y: 2, z: 3 } }
    "#;
    assert_eq!(error(src), "`Point` has no field `z`");
}

#[test]
fn a_missing_field_is_an_error() {
    let src = r#"
        struct Point { x: i32, y: i32 }
        fn main() { Point { x: 1 } }
    "#;
    assert_eq!(error(src), "missing field `y` in `Point`");
}

#[test]
fn giving_a_field_twice_is_an_error() {
    let src = r#"
        struct Point { x: i32, y: i32 }
        fn main() { Point { x: 1, y: 2, x: 3 } }
    "#;
    assert_eq!(error(src), "field `x` is given twice");
}

#[test]
fn reading_an_unknown_field_is_an_error() {
    let src = r#"
        struct Point { x: i32, y: i32 }
        fn main() -> i32 { Point { x: 1, y: 2 }.z }
    "#;
    assert_eq!(error(src), "`Point` has no field `z`");
}

#[test]
fn reading_a_field_of_a_non_struct_is_an_error() {
    assert_eq!(
        error("fn main() -> i32 { let n = 1; n.x }"),
        "i32 has no fields"
    );
    assert_eq!(
        error("fn main() -> i32 { \"hi\".len }"),
        "str has no fields"
    );
}

#[test]
fn two_structs_cannot_share_a_name() {
    let src = r#"
        struct Point { x: i32 }
        struct Point { y: i32 }
        fn main() { }
    "#;
    assert_eq!(error(src), "`Point` is defined more than once");
}

// mutation through a field arrives in a later step
#[test]
fn assigning_to_a_field_is_not_supported_yet() {
    let src = r#"
        struct Point { x: i32, y: i32 }
        fn main() -> i32 {
            let p = Point { x: 1, y: 2 };
            p.x = 5;
            p.x
        }
    "#;
    assert_eq!(error(src), "only variables can be assigned to for now");
}

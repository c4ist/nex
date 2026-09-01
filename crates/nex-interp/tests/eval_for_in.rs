//! for-in loops over ranges.

use nex_interp::{run_str, Value};

#[track_caller]
fn run(src: &str) -> Value {
    run_str(src).unwrap_or_else(|e| panic!("failed: {e}\n--- source ---\n{src}"))
}

#[track_caller]
fn error(src: &str) -> String {
    match run_str(src) {
        Ok(v) => panic!("unexpectedly produced {v} for:\n{src}"),
        Err(e) => e.message,
    }
}

// the plan's criterion for this step
#[test]
fn sums_a_range() {
    let value = run(r#"
        let mut total = 0;
        for i in 0..100 { total += i; }
        total
    "#);
    assert_eq!(value, Value::Int(4950));
}

#[test]
fn an_exclusive_range_stops_before_its_end() {
    let value = run(r#"
        let mut last = -1;
        for i in 0..3 { last = i; }
        last
    "#);
    assert_eq!(value, Value::Int(2));
}

#[test]
fn an_inclusive_range_reaches_its_end() {
    let value = run(r#"
        let mut total = 0;
        for i in 1..=4 { total += i; }
        total
    "#);
    assert_eq!(value, Value::Int(10));
}

#[test]
fn an_empty_range_never_runs_the_body() {
    assert_eq!(
        run("let mut n = 0; for i in 5..5 { n += 1; } n"),
        Value::Int(0)
    );
    assert_eq!(
        run("let mut n = 0; for i in 5..2 { n += 1; } n"),
        Value::Int(0)
    );
}

#[test]
fn negative_bounds_work() {
    let value = run(r#"
        let mut total = 0;
        for i in -3..0 { total += i; }
        total
    "#);
    assert_eq!(value, Value::Int(-6));
}

// the bounds are ordinary expressions, evaluated once up front
#[test]
fn range_bounds_can_be_expressions() {
    let value = run(r#"
        let n = 3;
        let mut total = 0;
        for i in 0..n * 2 { total += i; }
        total
    "#);
    assert_eq!(value, Value::Int(15));
}

// changing the variable the bound came from doesn't lengthen the loop
#[test]
fn the_end_bound_is_not_re_read_each_iteration() {
    let value = run(r#"
        let mut limit = 3;
        let mut count = 0;
        for i in 0..limit {
            count += 1;
            limit = 100;
        }
        count
    "#);
    assert_eq!(value, Value::Int(3));
}

#[test]
fn the_binding_is_visible_inside_the_body() {
    assert_eq!(
        run("let mut x = 0; for i in 7..8 { x = i; } x"),
        Value::Int(7)
    );
}

// the loop variable belongs to the loop, so it's gone afterwards
#[test]
fn the_binding_does_not_escape_the_loop() {
    assert_eq!(error("for i in 0..3 { } i"), "`i` is not defined");
}

#[test]
fn the_binding_shadows_an_outer_name_without_disturbing_it() {
    let value = run(r#"
        let i = 99;
        for i in 0..3 { }
        i
    "#);
    assert_eq!(value, Value::Int(99));
}

#[test]
fn break_leaves_the_loop() {
    let value = run(r#"
        let mut total = 0;
        for i in 0..100 {
            if i == 5 { break; }
            total += i;
        }
        total
    "#);
    assert_eq!(value, Value::Int(10));
}

#[test]
fn continue_skips_to_the_next_iteration() {
    let value = run(r#"
        let mut total = 0;
        for i in 0..10 {
            if i % 2 == 1 { continue; }
            total += i;
        }
        total
    "#);
    assert_eq!(value, Value::Int(20));
}

#[test]
fn loops_nest() {
    let value = run(r#"
        let mut total = 0;
        for i in 0..3 {
            for j in 0..4 { total += 1; }
        }
        total
    "#);
    assert_eq!(value, Value::Int(12));
}

#[test]
fn break_only_leaves_the_inner_loop() {
    let value = run(r#"
        let mut total = 0;
        for i in 0..3 {
            for j in 0..10 {
                if j == 2 { break; }
                total += 1;
            }
        }
        total
    "#);
    assert_eq!(value, Value::Int(6));
}

// a for and a while nest the same way
#[test]
fn a_for_nests_inside_a_while() {
    let value = run(r#"
        let mut outer = 0;
        let mut total = 0;
        while outer < 2 {
            for i in 0..3 { total += i; }
            outer += 1;
        }
        total
    "#);
    assert_eq!(value, Value::Int(6));
}

// body bindings are fresh each time round
#[test]
fn loop_bodies_get_a_fresh_scope_each_iteration() {
    let value = run(r#"
        let mut total = 0;
        for i in 0..3 {
            let step = 10;
            total += step;
        }
        total
    "#);
    assert_eq!(value, Value::Int(30));
}

#[test]
fn a_for_loop_is_unit() {
    assert_eq!(run("for i in 0..3 { 1 }"), Value::Unit);
}

#[test]
fn range_bounds_must_be_integers() {
    assert_eq!(error("for i in 0.0..3.0 { }"), "expected i32, found f64");
    assert_eq!(error("for i in 0..true { }"), "expected i32, found bool");
}

// only ranges are iterable so far
#[test]
fn other_iterables_are_not_supported_yet() {
    assert_eq!(
        error("for i in 5 { }"),
        "for loops can only iterate over ranges for now"
    );
}

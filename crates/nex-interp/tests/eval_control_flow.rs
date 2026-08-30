//! blocks, if/else and while.

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
fn computes_factorial_iteratively() {
    let value = run(r#"
        let n = 5;
        let mut result = 1;
        let mut i = 1;
        while i <= n {
            result = result * i;
            i += 1;
        }
        result
    "#);
    assert_eq!(value, Value::Int(120));
}

// blocks

#[test]
fn a_block_evaluates_to_its_last_expression() {
    assert_eq!(run("{ 1; 2; 3 }"), Value::Int(3));
    assert_eq!(run("{ }"), Value::Unit);
}

#[test]
fn let_bindings_are_visible_to_later_statements() {
    assert_eq!(run("let x = 2; let y = x * 3; y"), Value::Int(6));
}

// a block gets its own scope, so a binding inside it doesn't leak out
#[test]
fn block_scopes_do_not_leak() {
    assert_eq!(error("{ let inner = 1; } inner"), "`inner` is not defined");
}

#[test]
fn an_inner_binding_shadows_an_outer_one() {
    let value = run(r#"
        let x = 1;
        { let x = 2; }
        x
    "#);
    assert_eq!(value, Value::Int(1), "the inner binding must not escape");
}

// assignment reaches out to the existing binding rather than shadowing
#[test]
fn assignment_updates_an_outer_binding() {
    let value = run(r#"
        let mut x = 1;
        { x = 2; }
        x
    "#);
    assert_eq!(value, Value::Int(2));
}

#[test]
fn assigning_an_undefined_name_is_an_error() {
    assert_eq!(error("nope = 1;"), "`nope` is not defined");
}

#[test]
fn compound_assignment_applies_the_operator() {
    assert_eq!(run("let mut x = 10; x += 5; x"), Value::Int(15));
    assert_eq!(run("let mut x = 10; x -= 5; x"), Value::Int(5));
    assert_eq!(run("let mut x = 10; x *= 3; x"), Value::Int(30));
    assert_eq!(run("let mut x = 10; x /= 3; x"), Value::Int(3));
}

// if/else

#[test]
fn if_takes_the_matching_branch() {
    assert_eq!(run("if true { 1 } else { 2 }"), Value::Int(1));
    assert_eq!(run("if false { 1 } else { 2 }"), Value::Int(2));
}

// with no else and a false condition there's nothing to produce
#[test]
fn an_if_without_an_else_is_unit_when_false() {
    assert_eq!(run("if false { 1 }"), Value::Unit);
    assert_eq!(run("if true { 1 }"), Value::Int(1));
}

#[test]
fn else_if_chains_pick_the_first_match() {
    let src = |n: i32| {
        format!(
            "let n = {n};
             if n < 0 {{ \"negative\" }}
             else if n == 0 {{ \"zero\" }}
             else {{ \"positive\" }}"
        )
    };
    assert_eq!(run(&src(-5)), Value::str("negative"));
    assert_eq!(run(&src(0)), Value::str("zero"));
    assert_eq!(run(&src(7)), Value::str("positive"));
}

#[test]
fn if_conditions_must_be_bool() {
    assert_eq!(error("if 1 { 2 }"), "expected bool, found i32");
}

// only the taken branch runs, so an error in the other one stays quiet
#[test]
fn the_untaken_branch_is_not_evaluated() {
    assert_eq!(run("if true { 1 } else { undefined_name }"), Value::Int(1));
    assert_eq!(run("if false { undefined_name } else { 2 }"), Value::Int(2));
}

// while

#[test]
fn a_while_loop_runs_until_its_condition_is_false() {
    let value = run(r#"
        let mut i = 0;
        while i < 5 { i += 1; }
        i
    "#);
    assert_eq!(value, Value::Int(5));
}

#[test]
fn a_while_whose_condition_starts_false_never_runs() {
    let value = run(r#"
        let mut count = 0;
        while false { count += 1; }
        count
    "#);
    assert_eq!(value, Value::Int(0));
}

#[test]
fn break_leaves_the_loop() {
    let value = run(r#"
        let mut i = 0;
        while true {
            i += 1;
            if i == 3 { break; }
        }
        i
    "#);
    assert_eq!(value, Value::Int(3));
}

// continue skips the rest of the iteration but keeps looping
#[test]
fn continue_skips_to_the_next_iteration() {
    let value = run(r#"
        let mut i = 0;
        let mut evens = 0;
        while i < 10 {
            i += 1;
            if i % 2 == 1 { continue; }
            evens += 1;
        }
        evens
    "#);
    assert_eq!(value, Value::Int(5));
}

#[test]
fn loops_nest_and_break_only_leaves_the_inner_one() {
    let value = run(r#"
        let mut outer = 0;
        let mut total = 0;
        while outer < 3 {
            outer += 1;
            let mut inner = 0;
            while true {
                inner += 1;
                if inner == 2 { break; }
            }
            total += inner;
        }
        total
    "#);
    assert_eq!(value, Value::Int(6), "each inner loop should reach 2");
}

#[test]
fn a_while_condition_must_be_bool() {
    assert_eq!(error("while 1 { }"), "expected bool, found i32");
}

// a loop body is a block, so its bindings are fresh each iteration
#[test]
fn loop_bodies_get_a_fresh_scope_each_iteration() {
    let value = run(r#"
        let mut i = 0;
        let mut total = 0;
        while i < 3 {
            let step = 10;
            total += step;
            i += 1;
        }
        total
    "#);
    assert_eq!(value, Value::Int(30));
}

// for loops parse but don't run yet; that's the next step
#[test]
fn for_loops_are_not_supported_yet() {
    assert_eq!(
        error("for i in 0..3 { }"),
        "for loops are not supported yet"
    );
}

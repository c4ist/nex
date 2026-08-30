//! lexical scoping: shadowing, assignment through scopes, and lifetimes.

use nex_interp::{Scope, Value};

#[test]
fn defines_and_looks_up_a_binding() {
    let scope = Scope::global();
    scope.define("x", Value::Int(1));
    assert_eq!(scope.lookup("x"), Some(Value::Int(1)));
}

#[test]
fn an_undefined_name_is_not_found() {
    let scope = Scope::global();
    assert_eq!(scope.lookup("nope"), None);
}

#[test]
fn a_child_sees_its_parents_bindings() {
    let outer = Scope::global();
    outer.define("x", Value::Int(1));

    let inner = Scope::child(&outer);
    assert_eq!(inner.lookup("x"), Some(Value::Int(1)));
}

// the inner binding wins while it's in scope, and the outer one is intact
// afterwards
#[test]
fn an_inner_binding_shadows_an_outer_one() {
    let outer = Scope::global();
    outer.define("x", Value::Int(1));

    let inner = Scope::child(&outer);
    inner.define("x", Value::Int(2));

    assert_eq!(inner.lookup("x"), Some(Value::Int(2)));
    assert_eq!(outer.lookup("x"), Some(Value::Int(1)));
}

#[test]
fn a_parent_cannot_see_its_childs_bindings() {
    let outer = Scope::global();
    let inner = Scope::child(&outer);
    inner.define("only_inner", Value::Int(1));

    assert_eq!(outer.lookup("only_inner"), None);
}

// assignment finds the binding wherever it lives, so `x = 2` in a block
// updates the outer `x` instead of making a new one
#[test]
fn assignment_reaches_through_to_an_outer_binding() {
    let outer = Scope::global();
    outer.define("x", Value::Int(1));

    let inner = Scope::child(&outer);
    assert!(inner.assign("x", Value::Int(2)));

    assert_eq!(outer.lookup("x"), Some(Value::Int(2)));
    assert!(!inner.defined_locally("x"), "assignment must not shadow");
}

// but it stops at the nearest one, so shadowing still works
#[test]
fn assignment_updates_the_nearest_binding() {
    let outer = Scope::global();
    outer.define("x", Value::Int(1));

    let inner = Scope::child(&outer);
    inner.define("x", Value::Int(2));
    assert!(inner.assign("x", Value::Int(3)));

    assert_eq!(inner.lookup("x"), Some(Value::Int(3)));
    assert_eq!(outer.lookup("x"), Some(Value::Int(1)));
}

// assignment doesn't create bindings; that's what `let` is for
#[test]
fn assigning_an_undefined_name_fails() {
    let scope = Scope::global();
    assert!(!scope.assign("nope", Value::Int(1)));
    assert_eq!(scope.lookup("nope"), None);
}

#[test]
fn scopes_nest_arbitrarily_deep() {
    let root = Scope::global();
    root.define("x", Value::Int(1));

    let mut current = root.clone();
    for _ in 0..32 {
        current = Scope::child(&current);
    }

    assert_eq!(current.lookup("x"), Some(Value::Int(1)));
    assert!(current.assign("x", Value::Int(2)));
    assert_eq!(root.lookup("x"), Some(Value::Int(2)));
}

// a child holds its parent alive, which is what lets a closure outlive the
// call that created it
#[test]
fn a_child_keeps_its_parent_alive() {
    let inner = {
        let outer = Scope::global();
        outer.define("captured", Value::str("still here"));
        Scope::child(&outer)
    };

    assert_eq!(inner.lookup("captured"), Some(Value::str("still here")));
}

#[test]
fn redefining_in_the_same_scope_replaces_the_value() {
    let scope = Scope::global();
    scope.define("x", Value::Int(1));
    scope.define("x", Value::Int(2));
    assert_eq!(scope.lookup("x"), Some(Value::Int(2)));
}

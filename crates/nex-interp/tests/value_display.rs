//! how values print, which is what `print` and error messages rely on.

use std::cell::RefCell;
use std::collections::BTreeMap;
use std::rc::Rc;

use nex_interp::{Builtin, EnumValue, StructValue, Value};

#[test]
fn primitives_display() {
    assert_eq!(Value::Unit.to_string(), "()");
    assert_eq!(Value::Int(42).to_string(), "42");
    assert_eq!(Value::Int(-7).to_string(), "-7");
    assert_eq!(Value::Bool(true).to_string(), "true");
    assert_eq!(Value::str("hi").to_string(), "hi");
}

// a whole float keeps its point, so `1.0` doesn't print as `1` and read
// like an int
#[test]
fn floats_keep_their_decimal_point() {
    assert_eq!(Value::Float(1.0).to_string(), "1.0");
    assert_eq!(Value::Float(2.5).to_string(), "2.5");
    assert_eq!(Value::Float(-0.25).to_string(), "-0.25");
}

// print writes a string bare; debug quotes it
#[test]
fn strings_are_bare_when_displayed_and_quoted_when_debugged() {
    let value = Value::str("hello, world");
    assert_eq!(value.to_string(), "hello, world");
    assert_eq!(format!("{value:?}"), "\"hello, world\"");
}

#[test]
fn arrays_display_their_elements() {
    let empty = Value::array(vec![]);
    assert_eq!(empty.to_string(), "[]");

    let nums = Value::array(vec![Value::Int(1), Value::Int(2), Value::Int(3)]);
    assert_eq!(nums.to_string(), "[1, 2, 3]");

    let nested = Value::array(vec![Value::array(vec![Value::Int(1)])]);
    assert_eq!(nested.to_string(), "[[1]]");
}

#[test]
fn structs_display_their_fields() {
    let mut fields = BTreeMap::new();
    fields.insert("x".to_string(), Value::Float(1.0));
    fields.insert("y".to_string(), Value::Float(2.0));
    let point = Value::Struct(Rc::new(StructValue {
        name: "Point".to_string(),
        fields: RefCell::new(fields),
    }));
    assert_eq!(point.to_string(), "Point { x: 1.0, y: 2.0 }");
}

#[test]
fn enums_display_with_and_without_a_payload() {
    let none = Value::Enum(Rc::new(EnumValue {
        path: "Option".to_string(),
        variant: "None".to_string(),
        payload: vec![],
    }));
    assert_eq!(none.to_string(), "Option::None");

    let some = Value::Enum(Rc::new(EnumValue {
        path: "Option".to_string(),
        variant: "Some".to_string(),
        payload: vec![Value::Int(5)],
    }));
    assert_eq!(some.to_string(), "Option::Some(5)");
}

#[test]
fn builtins_display_by_name() {
    let print = Value::Builtin(Builtin {
        name: "print",
        arity: Some(1),
    });
    assert_eq!(print.to_string(), "fn print");
}

#[test]
fn type_names_are_the_ones_used_in_errors() {
    assert_eq!(Value::Int(1).type_name(), "i32");
    assert_eq!(Value::Float(1.0).type_name(), "f64");
    assert_eq!(Value::Bool(true).type_name(), "bool");
    assert_eq!(Value::str("s").type_name(), "str");
    assert_eq!(Value::Unit.type_name(), "()");
    assert_eq!(Value::array(vec![]).type_name(), "array");
}

// only bool is truthy: nex has no implicit conversions
#[test]
fn only_bools_are_truthy() {
    assert_eq!(Value::Bool(true).as_bool(), Some(true));
    assert_eq!(Value::Bool(false).as_bool(), Some(false));
    assert_eq!(Value::Int(1).as_bool(), None);
    assert_eq!(Value::Int(0).as_bool(), None);
    assert_eq!(Value::str("").as_bool(), None);
    assert_eq!(Value::Unit.as_bool(), None);
}

#[test]
fn equality_is_structural_for_data() {
    assert_eq!(Value::Int(1), Value::Int(1));
    assert_ne!(Value::Int(1), Value::Int(2));
    // no cross-type equality, even for numbers
    assert_ne!(Value::Int(1), Value::Float(1.0));
    assert_eq!(Value::str("a"), Value::str("a"));

    let a = Value::array(vec![Value::Int(1)]);
    let b = Value::array(vec![Value::Int(1)]);
    assert_eq!(a, b, "separate arrays with equal contents are equal");
}

// arrays are shared, so mutating through one binding is visible from the
// other. this is what makes `Clone` on a Value cheap.
#[test]
fn arrays_are_shared_not_copied() {
    let original = Value::array(vec![Value::Int(1)]);
    let alias = original.clone();

    if let Value::Array(items) = &alias {
        items.borrow_mut().push(Value::Int(2));
    }

    assert_eq!(original.to_string(), "[1, 2]");
}

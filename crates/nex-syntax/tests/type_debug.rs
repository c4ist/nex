//! debug round-trip tests on hand-built type trees.

use nex_syntax::{NodeIdGen, NodeInfo, Span, Spanned, Type, TypeKind};

fn info(ids: &mut NodeIdGen, span: Span) -> NodeInfo {
    NodeInfo::new(ids.fresh(), span)
}

fn ident(name: &str, span: Span) -> nex_syntax::Ident {
    Spanned::new(name.to_string(), span)
}

fn named(ids: &mut NodeIdGen, name: &str, args: Vec<Type>, span: Span) -> Type {
    Type::new(
        TypeKind::Named {
            name: ident(name, span),
            args,
        },
        info(ids, span),
    )
}

// `i32`
#[test]
fn debug_round_trips_a_plain_named_type() {
    let mut ids = NodeIdGen::new();
    let ty = named(&mut ids, "i32", vec![], Span::new(0, 3));
    assert_eq!(
        format!("{ty:?}"),
        "Type { info: #0@0..3, kind: Named { name: \"i32\"@0..3, args: [] } }"
    );
}

// `Option<T>`
#[test]
fn debug_round_trips_a_generic_named_type() {
    let mut ids = NodeIdGen::new();
    let arg = named(&mut ids, "T", vec![], Span::new(7, 8));
    let ty = Type::new(
        TypeKind::Named {
            name: ident("Option", Span::new(0, 6)),
            args: vec![arg],
        },
        info(&mut ids, Span::new(0, 9)),
    );
    assert_eq!(
        format!("{ty:?}"),
        "Type { info: #1@0..9, kind: Named { name: \"Option\"@0..6, args: [Type { info: #0@7..8, kind: Named { name: \"T\"@7..8, args: [] } }] } }"
    );
}

// `[i32]`
#[test]
fn debug_round_trips_an_array_type() {
    let mut ids = NodeIdGen::new();
    let elem = named(&mut ids, "i32", vec![], Span::new(1, 4));
    let ty = Type::new(
        TypeKind::Array(Box::new(elem)),
        info(&mut ids, Span::new(0, 5)),
    );
    assert_eq!(
        format!("{ty:?}"),
        "Type { info: #1@0..5, kind: Array(Type { info: #0@1..4, kind: Named { name: \"i32\"@1..4, args: [] } }) }"
    );
}

// `&Point`
#[test]
fn debug_round_trips_a_reference_type() {
    let mut ids = NodeIdGen::new();
    let pointee = named(&mut ids, "Point", vec![], Span::new(1, 6));
    let ty = Type::new(
        TypeKind::Ref(Box::new(pointee)),
        info(&mut ids, Span::new(0, 6)),
    );
    assert_eq!(
        format!("{ty:?}"),
        "Type { info: #1@0..6, kind: Ref(Type { info: #0@1..6, kind: Named { name: \"Point\"@1..6, args: [] } }) }"
    );
}

// `fn(i32, i32) -> i32`
#[test]
fn debug_round_trips_a_function_type_with_return() {
    let mut ids = NodeIdGen::new();
    let a = named(&mut ids, "i32", vec![], Span::new(3, 6));
    let b = named(&mut ids, "i32", vec![], Span::new(8, 11));
    let ret = named(&mut ids, "i32", vec![], Span::new(16, 19));
    let ty = Type::new(
        TypeKind::Fn {
            params: vec![a, b],
            return_type: Some(Box::new(ret)),
        },
        info(&mut ids, Span::new(0, 19)),
    );
    insta::assert_debug_snapshot!(ty);
}

// `fn()` - no params, no return type
#[test]
fn debug_round_trips_a_bare_function_type() {
    let mut ids = NodeIdGen::new();
    let ty = Type::new(
        TypeKind::Fn {
            params: vec![],
            return_type: None,
        },
        info(&mut ids, Span::new(0, 4)),
    );
    assert_eq!(
        format!("{ty:?}"),
        "Type { info: #0@0..4, kind: Fn { params: [], return_type: None } }"
    );
}

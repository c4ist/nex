//! debug round-trip tests on hand-built pattern trees.

use nex_syntax::{FieldPattern, NodeIdGen, NodeInfo, Pattern, PatternKind, Span, Spanned};

fn info(ids: &mut NodeIdGen, span: Span) -> NodeInfo {
    NodeInfo::new(ids.fresh(), span)
}

fn ident(name: &str, span: Span) -> nex_syntax::Ident {
    Spanned::new(name.to_string(), span)
}

fn binding(ids: &mut NodeIdGen, name: &str, span: Span) -> Pattern {
    Pattern::new(PatternKind::Binding(ident(name, span)), info(ids, span))
}

// literal patterns: `5`, `2.5`, `"hi"`, `true`
#[test]
fn debug_round_trips_literal_patterns() {
    let mut ids = NodeIdGen::new();
    let int = Pattern::new(PatternKind::Int(5), info(&mut ids, Span::new(0, 1)));
    let float = Pattern::new(PatternKind::Float(2.5), info(&mut ids, Span::new(0, 3)));
    let str_ = Pattern::new(
        PatternKind::Str("hi".to_string()),
        info(&mut ids, Span::new(0, 4)),
    );
    let bool_ = Pattern::new(PatternKind::Bool(true), info(&mut ids, Span::new(0, 4)));

    assert_eq!(
        format!("{int:?}"),
        "Pattern { info: #0@0..1, kind: Int(5) }"
    );
    assert_eq!(
        format!("{float:?}"),
        "Pattern { info: #1@0..3, kind: Float(2.5) }"
    );
    assert_eq!(
        format!("{str_:?}"),
        "Pattern { info: #2@0..4, kind: Str(\"hi\") }"
    );
    assert_eq!(
        format!("{bool_:?}"),
        "Pattern { info: #3@0..4, kind: Bool(true) }"
    );
}

// `Option::Some(v)`
#[test]
fn debug_round_trips_an_enum_variant_pattern_with_a_payload() {
    let mut ids = NodeIdGen::new();
    let v = binding(&mut ids, "v", Span::new(13, 14));
    let pat = Pattern::new(
        PatternKind::EnumVariant {
            path: vec![
                ident("Option", Span::new(0, 6)),
                ident("Some", Span::new(8, 12)),
            ],
            fields: vec![v],
        },
        info(&mut ids, Span::new(0, 15)),
    );
    assert_eq!(
        format!("{pat:?}"),
        "Pattern { info: #1@0..15, kind: EnumVariant { path: [\"Option\"@0..6, \"Some\"@8..12], fields: [Pattern { info: #0@13..14, kind: Binding(\"v\"@13..14) }] } }"
    );
}

// `Option::None` - unit variant, no payload
#[test]
fn debug_round_trips_a_unit_enum_variant_pattern() {
    let mut ids = NodeIdGen::new();
    let pat = Pattern::new(
        PatternKind::EnumVariant {
            path: vec![
                ident("Option", Span::new(0, 6)),
                ident("None", Span::new(8, 12)),
            ],
            fields: vec![],
        },
        info(&mut ids, Span::new(0, 12)),
    );
    assert_eq!(
        format!("{pat:?}"),
        "Pattern { info: #0@0..12, kind: EnumVariant { path: [\"Option\"@0..6, \"None\"@8..12], fields: [] } }"
    );
}

// `Point { x: px, y: py }`
#[test]
fn debug_round_trips_a_struct_pattern() {
    let mut ids = NodeIdGen::new();
    let fields = vec![
        FieldPattern::new(
            ident("x", Span::new(9, 10)),
            binding(&mut ids, "px", Span::new(12, 14)),
            info(&mut ids, Span::new(9, 14)),
        ),
        FieldPattern::new(
            ident("y", Span::new(16, 17)),
            binding(&mut ids, "py", Span::new(19, 21)),
            info(&mut ids, Span::new(16, 21)),
        ),
    ];
    let pat = Pattern::new(
        PatternKind::Struct {
            name: ident("Point", Span::new(0, 5)),
            fields,
        },
        info(&mut ids, Span::new(0, 23)),
    );
    insta::assert_debug_snapshot!(pat);
}

// `(a, b)`
#[test]
fn debug_round_trips_a_tuple_pattern() {
    let mut ids = NodeIdGen::new();
    let a = binding(&mut ids, "a", Span::new(1, 2));
    let b = binding(&mut ids, "b", Span::new(4, 5));
    let pat = Pattern::new(
        PatternKind::Tuple(vec![a, b]),
        info(&mut ids, Span::new(0, 6)),
    );
    assert_eq!(
        format!("{pat:?}"),
        "Pattern { info: #2@0..6, kind: Tuple([Pattern { info: #0@1..2, kind: Binding(\"a\"@1..2) }, Pattern { info: #1@4..5, kind: Binding(\"b\"@4..5) }]) }"
    );
}

//! debug round-trip tests on hand-built item trees.

use nex_syntax::{
    Block, Enum, Expr, ExprKind, FieldDef, Fn, Impl, Item, ItemKind, Mod, NodeIdGen, NodeInfo,
    Param, Span, Spanned, Stmt, StmtKind, Struct, Type, TypeKind, Use, Variant,
};

fn info(ids: &mut NodeIdGen, span: Span) -> NodeInfo {
    NodeInfo::new(ids.fresh(), span)
}

fn ident(name: &str, span: Span) -> nex_syntax::Ident {
    Spanned::new(name.to_string(), span)
}

fn named_type(ids: &mut NodeIdGen, name: &str, span: Span) -> Type {
    Type::new(TypeKind::Named(ident(name, span)), info(ids, span))
}

// `fn add(a: i32, b: i32) -> i32 { return a + b; }`
#[test]
fn debug_round_trips_a_fn_item() {
    let mut ids = NodeIdGen::new();
    let params = vec![
        Param::new(
            ident("a", Span::new(7, 8)),
            named_type(&mut ids, "i32", Span::new(10, 13)),
            info(&mut ids, Span::new(7, 13)),
        ),
        Param::new(
            ident("b", Span::new(15, 16)),
            named_type(&mut ids, "i32", Span::new(18, 21)),
            info(&mut ids, Span::new(15, 21)),
        ),
    ];
    let body = Block::new(
        vec![Stmt::new(
            StmtKind::Return(Some(Expr::new(
                ExprKind::Ident(ident("a", Span::new(33, 34))),
                info(&mut ids, Span::new(33, 34)),
            ))),
            info(&mut ids, Span::new(26, 35)),
        )],
        info(&mut ids, Span::new(24, 37)),
    );
    let item = Item::new(
        ItemKind::Fn(Fn::new(
            ident("add", Span::new(3, 6)),
            vec![],
            params,
            Some(named_type(&mut ids, "i32", Span::new(26, 29))),
            body,
            info(&mut ids, Span::new(22, 37)),
        )),
        info(&mut ids, Span::new(0, 37)),
    );
    insta::assert_debug_snapshot!(item);
}

// `struct Point { x: f64, y: f64 }`
#[test]
fn debug_round_trips_a_struct_item() {
    let mut ids = NodeIdGen::new();
    let fields = vec![
        FieldDef::new(
            ident("x", Span::new(15, 16)),
            named_type(&mut ids, "f64", Span::new(18, 21)),
            info(&mut ids, Span::new(15, 21)),
        ),
        FieldDef::new(
            ident("y", Span::new(23, 24)),
            named_type(&mut ids, "f64", Span::new(26, 29)),
            info(&mut ids, Span::new(23, 29)),
        ),
    ];
    let item = Item::new(
        ItemKind::Struct(Struct::new(
            ident("Point", Span::new(7, 12)),
            vec![],
            fields,
            info(&mut ids, Span::new(13, 31)),
        )),
        info(&mut ids, Span::new(0, 31)),
    );
    insta::assert_debug_snapshot!(item);
}

// `enum Option<T> { Some(T), None }`
#[test]
fn debug_round_trips_a_generic_enum_item() {
    let mut ids = NodeIdGen::new();
    let variants = vec![
        Variant::new(
            ident("Some", Span::new(18, 22)),
            vec![named_type(&mut ids, "T", Span::new(23, 24))],
            info(&mut ids, Span::new(18, 25)),
        ),
        Variant::new(
            ident("None", Span::new(27, 31)),
            vec![],
            info(&mut ids, Span::new(27, 31)),
        ),
    ];
    let item = Item::new(
        ItemKind::Enum(Enum::new(
            ident("Option", Span::new(5, 11)),
            vec![ident("T", Span::new(12, 13))],
            variants,
            info(&mut ids, Span::new(16, 33)),
        )),
        info(&mut ids, Span::new(0, 33)),
    );
    insta::assert_debug_snapshot!(item);
}

// `use a::b::c;`
#[test]
fn debug_round_trips_a_use_item() {
    let mut ids = NodeIdGen::new();
    let item = Item::new(
        ItemKind::Use(Use::new(
            vec![
                ident("a", Span::new(4, 5)),
                ident("b", Span::new(7, 8)),
                ident("c", Span::new(10, 11)),
            ],
            info(&mut ids, Span::new(4, 11)),
        )),
        info(&mut ids, Span::new(0, 12)),
    );
    assert_eq!(
        format!("{item:?}"),
        "Item { info: #1@0..12, kind: Use(Use { info: #0@4..11, path: [\"a\"@4..5, \"b\"@7..8, \"c\"@10..11] }) }"
    );
}

// `mod foo;` (external) and `mod foo { }` (inline, empty)
#[test]
fn debug_round_trips_mod_items_external_and_inline() {
    let mut ids = NodeIdGen::new();
    let external = Item::new(
        ItemKind::Mod(Mod::new(
            ident("foo", Span::new(4, 7)),
            None,
            info(&mut ids, Span::new(4, 7)),
        )),
        info(&mut ids, Span::new(0, 8)),
    );
    assert_eq!(
        format!("{external:?}"),
        "Item { info: #1@0..8, kind: Mod(Mod { info: #0@4..7, name: \"foo\"@4..7, items: None }) }"
    );

    let inline = Item::new(
        ItemKind::Mod(Mod::new(
            ident("foo", Span::new(4, 7)),
            Some(vec![]),
            info(&mut ids, Span::new(4, 11)),
        )),
        info(&mut ids, Span::new(0, 11)),
    );
    assert_eq!(
        format!("{inline:?}"),
        "Item { info: #3@0..11, kind: Mod(Mod { info: #2@4..11, name: \"foo\"@4..7, items: Some([]) }) }"
    );
}

// `impl Point { }` - stub, just the target name
#[test]
fn debug_round_trips_an_impl_stub() {
    let mut ids = NodeIdGen::new();
    let item = Item::new(
        ItemKind::Impl(Impl::new(
            ident("Point", Span::new(5, 10)),
            info(&mut ids, Span::new(5, 10)),
        )),
        info(&mut ids, Span::new(0, 14)),
    );
    assert_eq!(
        format!("{item:?}"),
        "Item { info: #1@0..14, kind: Impl(Impl { info: #0@5..10, target: \"Point\"@5..10 }) }"
    );
}

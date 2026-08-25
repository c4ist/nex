//! snapshot tests for the ast -> s-expression pretty printer.

use nex_syntax::{
    print_item, print_pattern, print_type, BinaryOp, Block, Enum, Expr, ExprKind, Fn, Item,
    ItemKind, NodeIdGen, NodeInfo, Param, Pattern, PatternKind, Span, Spanned, Stmt, StmtKind,
    Type, TypeKind, Variant,
};

fn info(ids: &mut NodeIdGen, span: Span) -> NodeInfo {
    NodeInfo::new(ids.fresh(), span)
}

fn ident(name: &str, span: Span) -> nex_syntax::Ident {
    Spanned::new(name.to_string(), span)
}

fn named_type(ids: &mut NodeIdGen, name: &str, span: Span) -> Type {
    Type::new(
        TypeKind::Named {
            name: ident(name, span),
            args: vec![],
        },
        info(ids, span),
    )
}

// `fn add(a: i32, b: i32) -> i32 { return a + b; }`
#[test]
fn prints_fn_add_as_an_s_expression() {
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
                ExprKind::Binary {
                    op: Spanned::new(BinaryOp::Add, Span::new(35, 36)),
                    lhs: Box::new(Expr::new(
                        ExprKind::Ident(ident("a", Span::new(33, 34))),
                        info(&mut ids, Span::new(33, 34)),
                    )),
                    rhs: Box::new(Expr::new(
                        ExprKind::Ident(ident("b", Span::new(37, 38))),
                        info(&mut ids, Span::new(37, 38)),
                    )),
                },
                info(&mut ids, Span::new(33, 38)),
            ))),
            info(&mut ids, Span::new(26, 39)),
        )],
        info(&mut ids, Span::new(24, 41)),
    );
    let item = Item::new(
        ItemKind::Fn(Fn::new(
            ident("add", Span::new(3, 6)),
            vec![],
            params,
            Some(named_type(&mut ids, "i32", Span::new(26, 29))),
            body,
            info(&mut ids, Span::new(22, 41)),
        )),
        info(&mut ids, Span::new(0, 41)),
    );
    insta::assert_snapshot!(print_item(&item), @"(fn add () ((a i32) (b i32)) i32 (block (return (+ a b))))");
}

// `Option<T>` and `[&i32]`
#[test]
fn prints_types() {
    let mut ids = NodeIdGen::new();
    let arg = named_type(&mut ids, "T", Span::new(7, 8));
    let generic = Type::new(
        TypeKind::Named {
            name: ident("Option", Span::new(0, 6)),
            args: vec![arg],
        },
        info(&mut ids, Span::new(0, 9)),
    );
    assert_eq!(print_type(&generic), "(Option T)");

    let inner = named_type(&mut ids, "i32", Span::new(2, 5));
    let reffed = Type::new(
        TypeKind::Ref(Box::new(inner)),
        info(&mut ids, Span::new(1, 5)),
    );
    let array = Type::new(
        TypeKind::Array(Box::new(reffed)),
        info(&mut ids, Span::new(0, 6)),
    );
    assert_eq!(print_type(&array), "(array (ref i32))");
}

// `enum Option<T> { Some(T), None }`
#[test]
fn prints_a_generic_enum_item() {
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
    assert_eq!(print_item(&item), "(enum Option (T) ((Some T) (None)))");
}

// `Option::Some(v)` and a struct pattern
#[test]
fn prints_patterns() {
    let mut ids = NodeIdGen::new();
    let binding = Pattern::new(
        PatternKind::Binding(ident("v", Span::new(13, 14))),
        info(&mut ids, Span::new(13, 14)),
    );
    let variant = Pattern::new(
        PatternKind::EnumVariant {
            path: vec![
                ident("Option", Span::new(0, 6)),
                ident("Some", Span::new(8, 12)),
            ],
            fields: vec![binding],
        },
        info(&mut ids, Span::new(0, 15)),
    );
    assert_eq!(print_pattern(&variant), "(Option::Some v)");

    let wildcard = Pattern::new(PatternKind::Wildcard, info(&mut ids, Span::new(0, 1)));
    assert_eq!(print_pattern(&wildcard), "_");
}

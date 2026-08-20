//! debug round-trip tests on hand-built expression trees.

use nex_syntax::{
    BinaryOp, Block, Expr, ExprKind, FieldInit, MatchArm, NodeIdGen, NodeInfo, Pattern,
    PatternKind, Span, Spanned, Stmt, StmtKind, UnaryOp,
};

fn info(ids: &mut NodeIdGen, span: Span) -> NodeInfo {
    NodeInfo::new(ids.fresh(), span)
}

fn int(ids: &mut NodeIdGen, v: i64, span: Span) -> Expr {
    Expr::new(ExprKind::Int(v), info(ids, span))
}

fn ident(ids: &mut NodeIdGen, name: &str, span: Span) -> Expr {
    Expr::new(
        ExprKind::Ident(Spanned::new(name.to_string(), span)),
        info(ids, span),
    )
}

#[test]
fn debug_round_trips_a_simple_node() {
    let mut ids = NodeIdGen::new();
    let expr = int(&mut ids, 5, Span::new(0, 1));
    assert_eq!(format!("{expr:?}"), "Expr { info: #0@0..1, kind: Int(5) }");
}

#[test]
fn debug_round_trips_nested_nodes() {
    let mut ids = NodeIdGen::new();
    let expr = Expr::new(
        ExprKind::Unary {
            op: Spanned::new(UnaryOp::Neg, Span::new(0, 1)),
            operand: Box::new(int(&mut ids, 3, Span::new(1, 2))),
        },
        info(&mut ids, Span::new(0, 2)),
    );
    assert_eq!(
        format!("{expr:?}"),
        "Expr { info: #1@0..2, kind: Unary { op: Neg@0..1, operand: Expr { info: #0@1..2, kind: Int(3) } } }"
    );
}

#[test]
fn debug_round_trips_a_range() {
    let mut ids = NodeIdGen::new();
    let expr = Expr::new(
        ExprKind::Range {
            start: Box::new(int(&mut ids, 0, Span::new(0, 1))),
            end: Box::new(int(&mut ids, 10, Span::new(3, 5))),
            inclusive: true,
        },
        info(&mut ids, Span::new(0, 5)),
    );
    assert_eq!(
        format!("{expr:?}"),
        "Expr { info: #2@0..5, kind: Range { start: Expr { info: #0@0..1, kind: Int(0) }, end: Expr { info: #1@3..5, kind: Int(10) }, inclusive: true } }"
    );
}

#[test]
fn unit_literal_debug() {
    let mut ids = NodeIdGen::new();
    let expr = Expr::new(ExprKind::Unit, info(&mut ids, Span::new(0, 2)));
    assert_eq!(format!("{expr:?}"), "Expr { info: #0@0..2, kind: Unit }");
}

// `1 + p.dist(origin)`
#[test]
fn binary_call_field_round_trip() {
    let mut ids = NodeIdGen::new();
    let expr = Expr::new(
        ExprKind::Binary {
            op: Spanned::new(BinaryOp::Add, Span::new(1, 2)),
            lhs: Box::new(int(&mut ids, 1, Span::new(0, 1))),
            rhs: Box::new(Expr::new(
                ExprKind::Call {
                    callee: Box::new(Expr::new(
                        ExprKind::Field {
                            base: Box::new(ident(&mut ids, "p", Span::new(4, 5))),
                            field: Spanned::new("dist".to_string(), Span::new(6, 10)),
                        },
                        info(&mut ids, Span::new(4, 10)),
                    )),
                    args: vec![ident(&mut ids, "origin", Span::new(11, 17))],
                },
                info(&mut ids, Span::new(4, 18)),
            )),
        },
        info(&mut ids, Span::new(0, 18)),
    );
    insta::assert_debug_snapshot!(expr);
}

// `if x > 0 { Point { x: 1.0, y: 2.0 } } else { match xs[0] { _ => -1 } }`
#[test]
fn kitchen_sink_round_trip() {
    let mut ids = NodeIdGen::new();
    let expr = Expr::new(
        ExprKind::If {
            cond: Box::new(Expr::new(
                ExprKind::Binary {
                    op: Spanned::new(BinaryOp::Gt, Span::new(3, 4)),
                    lhs: Box::new(ident(&mut ids, "x", Span::new(0, 1))),
                    rhs: Box::new(int(&mut ids, 0, Span::new(5, 6))),
                },
                info(&mut ids, Span::new(0, 6)),
            )),
            then: Box::new(Expr::new(
                ExprKind::Block(Block::new(
                    vec![Stmt::new(
                        StmtKind::Expr(Expr::new(
                            ExprKind::StructLit {
                                name: Spanned::new("Point".to_string(), Span::new(12, 17)),
                                fields: vec![
                                    FieldInit::new(
                                        Spanned::new("x".to_string(), Span::new(19, 20)),
                                        Expr::new(
                                            ExprKind::Float(1.0),
                                            info(&mut ids, Span::new(22, 25)),
                                        ),
                                        info(&mut ids, Span::new(19, 25)),
                                    ),
                                    FieldInit::new(
                                        Spanned::new("y".to_string(), Span::new(27, 28)),
                                        Expr::new(
                                            ExprKind::Float(2.0),
                                            info(&mut ids, Span::new(30, 33)),
                                        ),
                                        info(&mut ids, Span::new(27, 33)),
                                    ),
                                ],
                            },
                            info(&mut ids, Span::new(12, 34)),
                        )),
                        info(&mut ids, Span::new(12, 34)),
                    )],
                    info(&mut ids, Span::new(10, 36)),
                )),
                info(&mut ids, Span::new(10, 36)),
            )),
            else_: Some(Box::new(Expr::new(
                ExprKind::Block(Block::new(
                    vec![Stmt::new(
                        StmtKind::Expr(Expr::new(
                            ExprKind::Match {
                                scrutinee: Box::new(Expr::new(
                                    ExprKind::Index {
                                        base: Box::new(ident(&mut ids, "xs", Span::new(46, 48))),
                                        index: Box::new(int(&mut ids, 0, Span::new(49, 50))),
                                    },
                                    info(&mut ids, Span::new(46, 51)),
                                )),
                                arms: vec![MatchArm::new(
                                    Pattern::new(
                                        PatternKind::Wildcard,
                                        info(&mut ids, Span::new(57, 58)),
                                    ),
                                    Expr::new(
                                        ExprKind::Unary {
                                            op: Spanned::new(UnaryOp::Neg, Span::new(62, 63)),
                                            operand: Box::new(int(&mut ids, 1, Span::new(63, 64))),
                                        },
                                        info(&mut ids, Span::new(62, 64)),
                                    ),
                                    info(&mut ids, Span::new(57, 64)),
                                )],
                            },
                            info(&mut ids, Span::new(52, 65)),
                        )),
                        info(&mut ids, Span::new(52, 65)),
                    )],
                    info(&mut ids, Span::new(44, 67)),
                )),
                info(&mut ids, Span::new(44, 67)),
            ))),
        },
        info(&mut ids, Span::new(0, 67)),
    );
    insta::assert_debug_snapshot!(expr);
}

// `match true { v => "hi" }`
#[test]
fn literals_and_binding_round_trip() {
    let mut ids = NodeIdGen::new();
    let expr = Expr::new(
        ExprKind::Match {
            scrutinee: Box::new(Expr::new(
                ExprKind::Bool(true),
                info(&mut ids, Span::new(0, 4)),
            )),
            arms: vec![MatchArm::new(
                Pattern::new(
                    PatternKind::Binding(Spanned::new("v".to_string(), Span::new(8, 9))),
                    info(&mut ids, Span::new(8, 9)),
                ),
                Expr::new(
                    ExprKind::Str("hi".to_string()),
                    info(&mut ids, Span::new(13, 17)),
                ),
                info(&mut ids, Span::new(8, 17)),
            )],
        },
        info(&mut ids, Span::new(0, 18)),
    );
    insta::assert_debug_snapshot!(expr);
}

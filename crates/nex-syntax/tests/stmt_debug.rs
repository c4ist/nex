//! debug round-trip tests on hand-built statement trees.

use nex_syntax::{Block, Expr, ExprKind, NodeIdGen, NodeInfo, Span, Spanned, Stmt, StmtKind};

fn info(ids: &mut NodeIdGen, span: Span) -> NodeInfo {
    NodeInfo::new(ids.fresh(), span)
}

fn int(ids: &mut NodeIdGen, v: i64, span: Span) -> Expr {
    Expr::new(ExprKind::Int(v), info(ids, span))
}

// `let x = 5;`
#[test]
fn debug_round_trips_a_let_statement() {
    let mut ids = NodeIdGen::new();
    let stmt = Stmt::new(
        StmtKind::Let {
            mutable: false,
            name: Spanned::new("x".to_string(), Span::new(4, 5)),
            value: int(&mut ids, 5, Span::new(8, 9)),
        },
        info(&mut ids, Span::new(0, 10)),
    );
    assert_eq!(
        format!("{stmt:?}"),
        "Stmt { info: #1@0..10, kind: Let { mutable: false, name: \"x\"@4..5, value: Expr { info: #0@8..9, kind: Int(5) } } }"
    );
}

// `let mut s = "hi";`
#[test]
fn debug_round_trips_a_mutable_let_statement() {
    let mut ids = NodeIdGen::new();
    let stmt = Stmt::new(
        StmtKind::Let {
            mutable: true,
            name: Spanned::new("s".to_string(), Span::new(8, 9)),
            value: Expr::new(
                ExprKind::Str("hi".to_string()),
                info(&mut ids, Span::new(12, 16)),
            ),
        },
        info(&mut ids, Span::new(0, 17)),
    );
    assert_eq!(
        format!("{stmt:?}"),
        "Stmt { info: #1@0..17, kind: Let { mutable: true, name: \"s\"@8..9, value: Expr { info: #0@12..16, kind: Str(\"hi\") } } }"
    );
}

// `return a + b;` vs bare `return;`
#[test]
fn debug_round_trips_return_with_and_without_a_value() {
    let mut ids = NodeIdGen::new();
    let with_value = Stmt::new(
        StmtKind::Return(Some(int(&mut ids, 1, Span::new(7, 8)))),
        info(&mut ids, Span::new(0, 9)),
    );
    assert_eq!(
        format!("{with_value:?}"),
        "Stmt { info: #1@0..9, kind: Return(Some(Expr { info: #0@7..8, kind: Int(1) })) }"
    );

    let bare = Stmt::new(StmtKind::Return(None), info(&mut ids, Span::new(0, 7)));
    assert_eq!(
        format!("{bare:?}"),
        "Stmt { info: #2@0..7, kind: Return(None) }"
    );
}

// `break;` and `continue;`
#[test]
fn debug_round_trips_break_and_continue() {
    let mut ids = NodeIdGen::new();
    let brk = Stmt::new(StmtKind::Break, info(&mut ids, Span::new(0, 6)));
    let cont = Stmt::new(StmtKind::Continue, info(&mut ids, Span::new(0, 9)));
    assert_eq!(format!("{brk:?}"), "Stmt { info: #0@0..6, kind: Break }");
    assert_eq!(
        format!("{cont:?}"),
        "Stmt { info: #1@0..9, kind: Continue }"
    );
}

// `while true { break; }`
#[test]
fn debug_round_trips_a_while_loop() {
    let mut ids = NodeIdGen::new();
    let stmt = Stmt::new(
        StmtKind::While {
            cond: Expr::new(ExprKind::Bool(true), info(&mut ids, Span::new(6, 10))),
            body: Block::new(
                vec![Stmt::new(
                    StmtKind::Break,
                    info(&mut ids, Span::new(13, 19)),
                )],
                info(&mut ids, Span::new(11, 21)),
            ),
        },
        info(&mut ids, Span::new(0, 21)),
    );
    assert_eq!(
        format!("{stmt:?}"),
        "Stmt { info: #3@0..21, kind: While { cond: Expr { info: #0@6..10, kind: Bool(true) }, body: Block { info: #2@11..21, stmts: [Stmt { info: #1@13..19, kind: Break }] } } }"
    );
}

// `for i in 0..10 { print(i); }`
#[test]
fn debug_round_trips_a_for_in_loop() {
    let mut ids = NodeIdGen::new();
    let call = Expr::new(
        ExprKind::Call {
            callee: Box::new(Expr::new(
                ExprKind::Ident(Spanned::new("print".to_string(), Span::new(18, 23))),
                info(&mut ids, Span::new(18, 23)),
            )),
            args: vec![Expr::new(
                ExprKind::Ident(Spanned::new("i".to_string(), Span::new(24, 25))),
                info(&mut ids, Span::new(24, 25)),
            )],
        },
        info(&mut ids, Span::new(18, 26)),
    );
    let stmt = Stmt::new(
        StmtKind::ForIn {
            binding: Spanned::new("i".to_string(), Span::new(4, 5)),
            iter: Expr::new(
                ExprKind::Range {
                    start: Box::new(int(&mut ids, 0, Span::new(9, 10))),
                    end: Box::new(int(&mut ids, 10, Span::new(12, 14))),
                    inclusive: false,
                },
                info(&mut ids, Span::new(9, 14)),
            ),
            body: Block::new(
                vec![Stmt::new(
                    StmtKind::Expr(call),
                    info(&mut ids, Span::new(18, 27)),
                )],
                info(&mut ids, Span::new(15, 29)),
            ),
        },
        info(&mut ids, Span::new(0, 29)),
    );
    insta::assert_debug_snapshot!(stmt);
}

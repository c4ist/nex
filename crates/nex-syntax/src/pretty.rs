//! ast -> s-expression pretty printer. exists for readable snapshot tests;
//! the format is not meant to be reparsed.

use crate::expr::{BinaryOp, Block, Expr, ExprKind, FieldInit, MatchArm, UnaryOp};
use crate::item::{
    Enum, FieldDef as ItemFieldDef, Fn, Impl, Item, ItemKind, Mod, Param, Struct, Use, Variant,
};
use crate::node::Ident;
use crate::pattern::{FieldPattern, Pattern, PatternKind};
use crate::stmt::{Stmt, StmtKind};
use crate::ty::{Type, TypeKind};

fn sexp(head: &str, parts: &[String]) -> String {
    if parts.is_empty() {
        format!("({head})")
    } else {
        format!("({head} {})", parts.join(" "))
    }
}

fn list(items: &[String]) -> String {
    format!("({})", items.join(" "))
}

fn ident_str(ident: &Ident) -> String {
    ident.value.clone()
}

fn path_str(path: &[Ident]) -> String {
    path.iter().map(ident_str).collect::<Vec<_>>().join("::")
}

fn unary_op_str(op: UnaryOp) -> &'static str {
    match op {
        UnaryOp::Neg => "-",
        UnaryOp::Not => "!",
    }
}

fn binary_op_str(op: BinaryOp) -> &'static str {
    use BinaryOp::*;
    match op {
        Or => "||",
        And => "&&",
        Eq => "==",
        Ne => "!=",
        Lt => "<",
        Le => "<=",
        Gt => ">",
        Ge => ">=",
        BitOr => "|",
        BitXor => "^",
        BitAnd => "&",
        Shl => "<<",
        Shr => ">>",
        Add => "+",
        Sub => "-",
        Mul => "*",
        Div => "/",
        Rem => "%",
    }
}

pub fn print_expr(expr: &Expr) -> String {
    match &expr.kind {
        ExprKind::Int(v) => v.to_string(),
        ExprKind::Float(v) => v.to_string(),
        ExprKind::Str(s) => format!("{s:?}"),
        ExprKind::Bool(b) => b.to_string(),
        ExprKind::Unit => "()".to_string(),
        ExprKind::Ident(name) => ident_str(name),
        ExprKind::Unary { op, operand } => sexp(unary_op_str(op.value), &[print_expr(operand)]),
        ExprKind::Binary { op, lhs, rhs } => {
            sexp(binary_op_str(op.value), &[print_expr(lhs), print_expr(rhs)])
        }
        ExprKind::Call { callee, args } => {
            let mut parts = vec![print_expr(callee)];
            parts.extend(args.iter().map(print_expr));
            sexp("call", &parts)
        }
        ExprKind::Field { base, field } => sexp("field", &[print_expr(base), ident_str(field)]),
        ExprKind::Index { base, index } => sexp("index", &[print_expr(base), print_expr(index)]),
        ExprKind::StructLit { name, fields } => {
            let mut parts = vec![ident_str(name)];
            parts.extend(fields.iter().map(print_field_init));
            sexp("struct-lit", &parts)
        }
        ExprKind::If { cond, then, else_ } => {
            let mut parts = vec![print_expr(cond), print_expr(then)];
            if let Some(else_) = else_ {
                parts.push(print_expr(else_));
            }
            sexp("if", &parts)
        }
        ExprKind::Block(block) => print_block(block),
        ExprKind::Match { scrutinee, arms } => {
            let mut parts = vec![print_expr(scrutinee)];
            parts.extend(arms.iter().map(print_match_arm));
            sexp("match", &parts)
        }
        ExprKind::Range {
            start,
            end,
            inclusive,
        } => {
            let head = if *inclusive { "range-incl" } else { "range" };
            sexp(head, &[print_expr(start), print_expr(end)])
        }
        ExprKind::Closure { params, body } => {
            sexp("closure", &[print_params(params), print_expr(body)])
        }
    }
}

fn print_field_init(field: &FieldInit) -> String {
    list(&[ident_str(&field.name), print_expr(&field.value)])
}

fn print_block(block: &Block) -> String {
    let parts: Vec<String> = block.stmts.iter().map(print_stmt).collect();
    sexp("block", &parts)
}

fn print_match_arm(arm: &MatchArm) -> String {
    list(&[print_pattern(&arm.pattern), print_expr(&arm.body)])
}

pub fn print_stmt(stmt: &Stmt) -> String {
    match &stmt.kind {
        StmtKind::Expr(e) => print_expr(e),
        StmtKind::Let {
            mutable,
            name,
            ty,
            value,
        } => {
            let head = if *mutable { "let-mut" } else { "let" };
            let mut parts = vec![ident_str(name)];
            if let Some(ty) = ty {
                parts.push(print_type(ty));
            }
            parts.push(print_expr(value));
            sexp(head, &parts)
        }
        StmtKind::Assign { target, op, value } => {
            let head = match op {
                Some(op) => format!("{}=", binary_op_str(*op)),
                None => "=".to_string(),
            };
            sexp(&head, &[print_expr(target), print_expr(value)])
        }
        StmtKind::Return(value) => match value {
            Some(e) => sexp("return", &[print_expr(e)]),
            None => "(return)".to_string(),
        },
        StmtKind::While { cond, body } => sexp("while", &[print_expr(cond), print_block(body)]),
        StmtKind::ForIn {
            binding,
            iter,
            body,
        } => sexp(
            "for-in",
            &[ident_str(binding), print_expr(iter), print_block(body)],
        ),
        StmtKind::Break => "(break)".to_string(),
        StmtKind::Continue => "(continue)".to_string(),
    }
}

pub fn print_type(ty: &Type) -> String {
    match &ty.kind {
        TypeKind::Named { name, args } => {
            if args.is_empty() {
                ident_str(name)
            } else {
                let mut parts = vec![ident_str(name)];
                parts.extend(args.iter().map(print_type));
                list(&parts)
            }
        }
        TypeKind::Array(elem) => sexp("array", &[print_type(elem)]),
        TypeKind::Ref(inner) => sexp("ref", &[print_type(inner)]),
        TypeKind::Fn {
            params,
            return_type,
        } => {
            let params_str = list(&params.iter().map(print_type).collect::<Vec<_>>());
            match return_type {
                Some(ret) => sexp("fn-type", &[params_str, print_type(ret)]),
                None => sexp("fn-type", &[params_str]),
            }
        }
        TypeKind::Infer => "_".to_string(),
    }
}

pub fn print_pattern(pattern: &Pattern) -> String {
    match &pattern.kind {
        PatternKind::Wildcard => "_".to_string(),
        PatternKind::Binding(name) => ident_str(name),
        PatternKind::Int(v) => v.to_string(),
        PatternKind::Float(v) => v.to_string(),
        PatternKind::Str(s) => format!("{s:?}"),
        PatternKind::Bool(b) => b.to_string(),
        PatternKind::EnumVariant { path, fields } => {
            if fields.is_empty() {
                path_str(path)
            } else {
                let mut parts = vec![path_str(path)];
                parts.extend(fields.iter().map(print_pattern));
                list(&parts)
            }
        }
        PatternKind::Struct { name, fields } => {
            let mut parts = vec![ident_str(name)];
            parts.extend(fields.iter().map(print_field_pattern));
            sexp("struct-pat", &parts)
        }
        PatternKind::Tuple(elems) => {
            let parts: Vec<String> = elems.iter().map(print_pattern).collect();
            sexp("tuple", &parts)
        }
    }
}

fn print_field_pattern(field: &FieldPattern) -> String {
    list(&[ident_str(&field.name), print_pattern(&field.pattern)])
}

pub fn print_item(item: &Item) -> String {
    let printed = print_item_kind(item);
    if item.is_pub {
        sexp("pub", &[printed])
    } else {
        printed
    }
}

fn print_item_kind(item: &Item) -> String {
    match &item.kind {
        ItemKind::Fn(f) => print_fn(f),
        ItemKind::Struct(s) => print_struct(s),
        ItemKind::Enum(e) => print_enum(e),
        ItemKind::Use(u) => print_use(u),
        ItemKind::Mod(m) => print_mod(m),
        ItemKind::Impl(i) => print_impl(i),
    }
}

fn print_generics(generics: &[Ident]) -> String {
    list(&generics.iter().map(ident_str).collect::<Vec<_>>())
}

fn print_params(params: &[Param]) -> String {
    list(
        &params
            .iter()
            .map(|p| list(&[ident_str(&p.name), print_type(&p.ty)]))
            .collect::<Vec<_>>(),
    )
}

fn print_fn(f: &Fn) -> String {
    let params = print_params(&f.params);
    let ret = match &f.return_type {
        Some(ty) => print_type(ty),
        None => "()".to_string(),
    };
    sexp(
        "fn",
        &[
            ident_str(&f.name),
            print_generics(&f.generics),
            params,
            ret,
            print_block(&f.body),
        ],
    )
}

fn print_field_def(field: &ItemFieldDef) -> String {
    list(&[ident_str(&field.name), print_type(&field.ty)])
}

fn print_struct(s: &Struct) -> String {
    let fields = list(&s.fields.iter().map(print_field_def).collect::<Vec<_>>());
    sexp(
        "struct",
        &[ident_str(&s.name), print_generics(&s.generics), fields],
    )
}

fn print_variant(v: &Variant) -> String {
    if v.fields.is_empty() {
        list(&[ident_str(&v.name)])
    } else {
        let mut parts = vec![ident_str(&v.name)];
        parts.extend(v.fields.iter().map(print_type));
        list(&parts)
    }
}

fn print_enum(e: &Enum) -> String {
    let variants = list(&e.variants.iter().map(print_variant).collect::<Vec<_>>());
    sexp(
        "enum",
        &[ident_str(&e.name), print_generics(&e.generics), variants],
    )
}

fn print_use(u: &Use) -> String {
    sexp("use", &[path_str(&u.path)])
}

fn print_mod(m: &Mod) -> String {
    match &m.items {
        None => sexp("mod", &[ident_str(&m.name)]),
        Some(items) => {
            let mut parts = vec![ident_str(&m.name)];
            parts.extend(items.iter().map(print_item));
            sexp("mod", &parts)
        }
    }
}

fn print_impl(i: &Impl) -> String {
    sexp("impl", &[ident_str(&i.target)])
}

---
title: AST Coverage
description: What the AST can represent today, and what it can't.
---

Every construct in the [language spec](/language-design/) and
`examples/tour.nex`, checked against what `crates/nex-syntax` can actually
build. Current as of step 3.11.

## Covered

| Area | Nodes |
| ---- | ----- |
| Literals | `Int`, `Float`, `Str`, `Bool`, `Unit` |
| Operators | every operator in the spec's table except `=` and `+=`/`-=`/`*=`/`/=` |
| Expressions | calls, field access, indexing, struct literals, `if`/`else`, blocks, `match`, ranges |
| Statements | `let`, `let mut`, expression statements, `return`, `while`, `for`-`in`, `break`, `continue` |
| Items | `fn` (params, return type, generics), `struct`, `enum`, `use` |
| Types | named with generic args, `[T]`, `&T`, `fn(..) -> T` |
| Patterns | wildcard, binding, literals, `Enum::Variant(..)`, `Struct { .. }`, tuple |

## Gaps

### No path expression

`ExprKind::Ident` holds one `Ident`, so a `::`-qualified name has nowhere to
go. The spec's own sample doesn't parse because of this:

```rust
match Option::Some(x) { ... }
```

Patterns handle `Option::Some(v)` fine, but the scrutinee is an expression and
fails with ``expected `{`, found `::` ``. `Field { base, field }` isn't a
substitute: `::` is a separate token from `.`, and `Option` on its own isn't an
expression. Fixing it means adding `ExprKind::Path(Vec<Ident>)`.

This is the one gap that blocks working nex code today.

### No assignment statement

`x = 5;` and `x += 1;` have no `StmtKind`. `let` only introduces a binding, so
there's currently no way to reassign one. Parser step 4.2 covers this.

### No `const` or `type` items

`examples/tour.nex` uses `const MAX: i32 = 100;`. Both keywords are reserved by
the lexer, but `ItemKind` has no `Const` or `TypeAlias` variant.

### No item visibility

`pub` is a reserved keyword and nothing in `ItemKind` records it.

### `impl` is a stub

`ItemKind::Impl` holds the target type's name and nothing else. Methods need
`Fn` items nested inside, which waits on the type system.

### No `()` type

`ExprKind::Unit` covers the value. There's no `TypeKind` for the type, so a
function can omit its return type but can't write `-> ()` explicitly.

### No tuple type or expression

`PatternKind::Tuple` exists, but there's nothing to match against: no
`ExprKind::Tuple`, no `TypeKind::Tuple`. Tuples aren't in the spec's type table
either, so the pattern may be the thing that's wrong here.

### `use` is a bare path

No `as` aliasing, no `{}` groups, no globs.

## Tracked elsewhere

Block comments and character literals are lexer gaps, not AST gaps, and are on
the [roadmap](/roadmap/). Memory management is a Phase 8 decision. Generic
bounds aren't in the spec at all, since v0.1 has no traits.

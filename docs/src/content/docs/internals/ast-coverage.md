---
title: AST Coverage
description: Every construct in language-design.md, checked against what the AST can actually represent today.
---

:::note[Status: phase 2 review pass]
This page is the step 2.8 review: walk every construct in
[Language Design](/language-design/) and `examples/tour.nex` and check it
against what `crates/nex-syntax` can actually build today. It is a snapshot
as of step 2.7 (expr/stmt/item/type/pattern all defined, plus the pretty
printer) - re-check it before Phase 2 is called done.
:::

## Fully covered

These map cleanly onto an existing `ExprKind` / `StmtKind` / `ItemKind` /
`TypeKind` / `PatternKind` variant:

- Literals: `Int`, `Float`, `Str`, `Bool`, `Unit` (`ExprKind`)
- Unary/binary operators, matching every operator in the spec's operator
  table except `=` and the compound-assignment forms (see gaps below)
- `call`, field access (`.`), indexing (`[]`), struct literals, `if`/`else`,
  blocks, `match`, ranges (`..`, `..=`)
- `let` / `let mut`, expression statements, `return`, `while`, `for`-`in`,
  `break`, `continue`
- `fn` (params, return type, generics), `struct`, `enum` (incl. generic
  variant payloads), `use` (bare path)
- Types: named (incl. generic args), `[T]` arrays, `&T` references, `fn(..)
  -> T` function types
- Patterns: wildcard, binding, literals, `Enum::Variant(..)`, `Struct { .. }`,
  tuple

## Gaps

Ranked by how much they'd surprise someone reading the spec.

### No path-qualified expression

`ExprKind::Ident` holds a single `Ident` — there is no multi-segment path
expression. **The spec's own sample can't be represented yet**: `match
Option::Some(x) { ... }` from `language-design.md` needs `Option::Some` as a
callable expression, and nothing in `ExprKind` can hold a `::`-qualified
name. `Field { base, field }` doesn't cover it either — `::` is a distinct
token from `.`, and `Option` isn't itself an expression. This needs either a
dedicated `ExprKind::Path(Vec<Ident>)` or for `Field`/a new variant to accept
`::` segments. Likely lands alongside expression parsing in Phase 3, but the
AST doesn't have a home for it yet.

### No assignment statement

`x = 5;` and `x += 1;` have no `StmtKind`. `let`/`let mut` only cover
*introducing* a binding, never reassigning one that `mut` already permits.
This is deliberate, not forgotten — parser step 4.2 is titled "Assignment and
compound assignment statements" — but the AST itself has zero shape for it
right now.

### No `const` item

`examples/tour.nex` has `const MAX: i32 = 100;`; `ItemKind` has no `Const`
variant. The `const` keyword is reserved by the lexer but nothing in
`nex-syntax` consumes it.

### No `type` alias item

The `type` keyword is reserved (`fn let mut if else for while return struct
enum match use mod pub true false in break continue const type impl self`)
but there's no `ItemKind::TypeAlias` for `type Foo = Bar;`.

### No item visibility

Nothing in `ItemKind` carries a `pub`/private flag, even though `pub` is a
reserved keyword. Every item is implicitly "however the parser decides to
treat it" until this lands.

### `impl` is a stub on purpose

`ItemKind::Impl` holds only the target type's `Ident` — no methods, no trait
name. Methods need `Fn` items nested inside plus more of the type system
wired through first; tracked in `progress.txt`, not a surprise.

### No `TypeKind::Unit`

The spec's Types table lists `()` as "unit, the empty type." `ExprKind::Unit`
exists (the *value* `()`), but `TypeKind` has no matching variant for the
*type* `()` — an explicit `-> ()` return annotation can't be written yet,
only omitted (`return_type: None`).

### No tuple type or tuple expression

`PatternKind::Tuple` exists (required by step 2.6), but there is no
`ExprKind::Tuple` or `TypeKind::Tuple` to match *against*. Worth noting:
tuples aren't in the spec's Types table at all, so this may be an
over-build in the pattern AST rather than an under-build elsewhere - revisit
whether tuple patterns are even needed for v0.1 once the parser reaches
patterns.

### `use` is a bare path

No `as` aliasing, no `{}` grouped imports (`use std::{io, fs};`), no glob
(`use foo::*;`). Just a flat `::`-separated `Vec<Ident>`.

## Out of scope for this review (already tracked elsewhere)

- Block comments (`/* */`) and character literals (`'a'`) are lexer-level
  gaps, not AST gaps — see the roadmap's "Deferred / pending" section.
- Memory management (refcounting vs. ownership) is a Phase 8 decision with
  no AST impact yet.
- Generic *bounds*/`where` clauses aren't in the spec (no traits in v0.1),
  so their absence from `generics: Vec<Ident>` isn't a gap.

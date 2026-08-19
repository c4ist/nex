# Nex language design (spec v0.0)

This is the normative sketch the implementation is written against. It will be
frozen as v0.1 at the end of Phase 7.

## Goals

- **Lightweight.** Small language, small runtime, fast compiles.
- **Statically typed.** Everything is checked before it runs.
- **Familiar.** C/Go/Rust-flavoured syntax; no surprises.
- **No ceremony.** Local type inference, no header files, no build boilerplate.

## Non-goals for v0.1

Ownership/borrow checking, lifetimes, traits with dynamic dispatch, async,
threads, macros, a central package registry.

## Sample

```nex
// comments are // only (block comments arrive in Phase 7)
fn add(a: i32, b: i32) -> i32 { return a + b; }

struct Point { x: f64, y: f64 }

enum Option<T> { Some(T), None }

fn main() {
    let x = 5;            // inferred i32
    let mut s = "hi";     // str
    if x > 3 { print("big"); } else { print("small"); }
    for i in 0..10 { print(i); }
    let p = Point { x: 1.0, y: 2.0 };
    match Option::Some(x) {
        Option::Some(v) => print(v),
        Option::None => print("none"),
    }
}
```

## Types

| Type   | Meaning                        |
| ------ | ------------------------------ |
| `i32`  | 32-bit signed integer          |
| `i64`  | 64-bit signed integer          |
| `f64`  | 64-bit float                   |
| `bool` | `true` / `false`               |
| `str`  | immutable UTF-8 string         |
| `()`   | unit, the empty type           |
| `[T]`  | array of `T`                   |
| `&T`   | reference to `T`               |

Generics are supported on functions, structs and enums, and are resolved by
monomorphisation in the backend.

## Lexical structure

- **Comments:** `// line`
- **Identifiers:** `_` or an alphabetic character, then alphanumerics and `_`
- **Keywords:** `fn let mut if else for while return struct enum match use mod
  pub true false in break continue const type impl self`
- **Integers:** decimal, `0x`, `0o`, `0b`; `_` allowed as a separator
- **Floats:** `1.0`, `1e10`, `2.5e-3`; a `.` only starts a fraction when a digit
  follows, so `0..10` is a range
- **Strings:** `"..."` with `\n \r \t \0 \\ \" \xNN` escapes
- **Operators:** `+ - * / % = == != < <= > >= && || ! & | ^ << >>
  += -= *= /= -> => .. ..= :: : ; , . ( ) { } [ ]`

## Memory

Decided in Phase 8. Current plan: automatic reference counting emitted by the
code generator for heap values (`str`, arrays, boxed enums). No ownership or
borrow checking in v0.1.

## Execution

1. **Phase 5** — tree-walking interpreter (`nex run`)
2. **Phase 8** — LLVM native code generation (`nex build`)
3. **Phase 9** — WebAssembly target (`nex build --target wasm32`)

All three must agree: the regression corpus is run under every available
backend and the outputs must be identical.

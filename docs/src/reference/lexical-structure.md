# Lexical Structure

> **Status: complete.** This page documents the lexer as implemented in
> `crates/nex-lexer` (Phase 1, done). Everything here is covered by tests.

The lexer is a hand-written scanner over a UTF-8 source string. It is
non-panicking and never stops early: recoverable problems are collected and
scanning continues, so one run reports every lexical error in the file. The
token stream always ends with exactly one `Eof` token.

## Trivia

- **Whitespace** (any Unicode whitespace) is skipped.
- **Comments** are `//` to end of line. Block comments (`/* */`) are *not*
  implemented yet — they arrive in Phase 7, so `/*` currently lexes as `Slash`
  then `Star`.
- Trivia produces no tokens.

## Identifiers

An identifier starts with `_` or any alphabetic character, and continues with
alphanumerics and `_`. Unicode letters and digits are allowed (e.g. `café`).
Keywords are recognised by spelling; a keyword prefix is an ordinary identifier
(`iff`, `letx`, `returns`, `Self`, `_` are all identifiers).

## Keywords

The 23 reserved words:

```
fn let mut if else for while return struct enum match
use mod pub true false in break continue const type impl self
```

`true` and `false` are keyword tokens (not identifiers).

## Literals

### Integers

- Decimal, plus `0x` (hex), `0o` (octal), `0b` (binary) prefixes.
- `_` is allowed anywhere among the digits as a separator: `1_000_000`,
  `0x_ff`.
- Values must fit in `i64`. Overflow reports `IntegerOverflow` and yields the
  value `0`. A prefix with no digits (`0x`) reports `MalformedNumber`.

### Floats

- `1.0`, `0.5`, with optional exponents: `1e10`, `2.5e-3`, `1E+5`.
- A `.` only starts a fraction when a digit follows, so `0..10` lexes as
  `Int(0) DotDot Int(10)` — the range is not a float.
- `1.foo` (dot before an identifier) is `MalformedNumber`.
- A dangling exponent marker is not consumed: `1e` is `Int(1)` followed by
  `Ident("e")`, not a broken float.

### Strings

- Double-quoted: `"..."`, with escapes `\n \r \t \0 \\ \"` and `\xNN` (exactly
  two hex digits).
- Unknown escapes (`\q`) report `InvalidEscape` but recover, keeping the
  character in the value.
- `\x` without two hex digits reports `InvalidHexEscape`.
- A string that runs to the end of the line or end of file reports
  `UnterminatedString` and recovers (scanning picks back up on the next line).

### Characters

Character literals (`'a'`) are **not** part of the language.

## Operators and punctuation

Matched longest-first:

| Token(s)                  | Spelling |
| ------------------------- | -------- |
| arithmetic                | `+ - * / %` |
| bitwise                   | `& \| ^ << >>` |
| logical                   | `&& \|\| !` |
| comparison                | `== != < <= > >=` |
| assignment                | `=` |
| compound assignment       | `+= -= *= /=` |
| arrows                    | `-> =>` |
| ranges                    | `.. ..=` |
| path separator            | `::` |
| delimiters                | `: ; , . ( ) { } [ ]` |

## Errors

All lexer errors are recoverable; the lexer reports every instance in one run.

| Error                  | Meaning                                         | Help shown |
| ---------------------- | ----------------------------------------------- | ---------- |
| `UnknownChar(c)`       | character that can't start any token            | —          |
| `UnterminatedString`   | string ran to end of line or end of file        | add a closing `"` |
| `InvalidEscape(c)`     | unknown escape sequence `\c`                    | lists valid escapes |
| `InvalidHexEscape`     | `\x` without exactly two hex digits             | —          |
| `MalformedNumber`      | `0x`/`0b`/`0o` with no digits, or `1.` followed by a non-digit | a digit must follow the prefix or decimal point |
| `IntegerOverflow`      | integer literal too large for `i64`             | maximum is 9223372036854775807 |
| `InvalidFloat`         | float literal that failed to parse              | —          |

Errors carry the same `Span` machinery as tokens, so diagnostics can point
exactly at the offending text.

## Spans

Every token and error carries a `Span`: a half-open byte range `[start, end)`
into the source file (`u32` offsets, so spans track multi-byte characters
correctly). The `Eof` token's span is the empty range at the end of the file.
`Span::text(src)` slices the original source back out of the span.

## The public API

`nex-lexer` exposes:

- `tokenize(src) -> (Vec<Token>, Vec<LexError>)` — scan everything
- `tokenize_kinds(src) -> Vec<TokenKind>` — drop the spans, for tests
- `dump_tokens(src) -> String` — one `Kind@start..end` line per token (plus an
  `-- errors --` section); the format used by `nex lex` and the snapshot tests
- `Lexer` — the scanner itself, usable as an `Iterator` over tokens

## Robustness guarantees

The test suite verifies that the lexer:

- never panics, even on a 200 000-character run of junk
- always terminates with a single `Eof` under a deterministic mutation fuzzer
- produces non-overlapping, in-bounds spans for any input

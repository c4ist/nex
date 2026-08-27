# Security

## Supported versions

None yet. Nex is pre-alpha and has no releases. Everything below applies to
`main`.

## Reporting a vulnerability

Use [GitHub's private vulnerability
reporting](https://github.com/c4ist/nex/security/advisories/new). It's only
visible to maintainers, so please use it instead of opening a public issue.

Include what you ran, what happened, and the input that caused it. A `.nex`
file that reproduces the problem is ideal.

Expect a reply within a week or so. This is a personal project, not a funded
one, so there's no formal response time.

## What counts

Nex is a compiler, so it reads untrusted source files. Things worth reporting:

- Input that panics or hangs the lexer, parser, or CLI. The lexer is
  fuzz-tested against this specifically, so a panic there is a real bug.
- Anything that reads or writes files outside what the command was asked to
  touch.
- Memory unsafety. There's no `unsafe` in the codebase at the moment, so this
  would most likely come from a dependency.

Things that aren't vulnerabilities right now:

- A compile error on valid code, or missing language features. Those are
  ordinary bugs; open an issue.
- Running a `.nex` program that does something destructive once the
  interpreter exists. Compiled code does what it says.

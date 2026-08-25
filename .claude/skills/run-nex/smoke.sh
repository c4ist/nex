#!/usr/bin/env bash
# Smoke-drives the `nex` CLI: builds it, runs the one real subcommand
# (`lex`) against the example programs, checks the stub subcommands fail
# the way they're supposed to, and runs the test suite.
#
# Run from the repo root: bash .claude/skills/run-nex/smoke.sh
set -euo pipefail
cd "$(git rev-parse --show-toplevel)"

pass() { echo "  ok: $1"; }
fail() { echo "  FAIL: $1"; exit 1; }

echo "==> cargo build --workspace"
cargo build --workspace

echo "==> nex lex examples/hello.nex"
out=$(cargo run -q -p nex-driver -- lex examples/hello.nex)
echo "$out" | grep -q '^Fn@' && pass "token stream starts with Fn" || fail "expected Fn token first"
echo "$out" | grep -q '^Eof@' && pass "token stream ends with Eof" || fail "expected trailing Eof token"

echo "==> nex lex examples/tour.nex (larger file, no crash)"
cargo run -q -p nex-driver -- lex examples/tour.nex > /dev/null
pass "tour.nex lexes without crashing"

echo "==> nex lex <missing file> (expect exit 1, stderr message)"
if cargo run -q -p nex-driver -- lex does-not-exist.nex 2>/tmp/nex-err; then
  fail "expected non-zero exit for missing file"
fi
grep -q "cannot read" /tmp/nex-err && pass "missing-file error message present"

echo "==> nex <stub subcommands> (expect exit 1, 'not implemented yet')"
for sub in "run examples/hello.nex" "build examples/hello.nex" "check examples/hello.nex" "fmt" "test"; do
  if cargo run -q -p nex-driver -- $sub 2>/tmp/nex-err; then
    fail "'$sub' should not succeed yet (still a stub) - update this driver once it's implemented"
  fi
  grep -q "not implemented yet" /tmp/nex-err && pass "'$sub' reports not-implemented" || fail "'$sub' gave unexpected error: $(cat /tmp/nex-err)"
done

echo "==> nex (no args, expect usage + exit 2)"
set +e
cargo run -q -p nex-driver --
code=$?
set -e
[ "$code" -eq 2 ] && pass "no-args exits 2 with usage" || fail "expected exit 2, got $code"

echo "==> cargo test --workspace"
cargo test --workspace

echo
echo "all smoke checks passed"

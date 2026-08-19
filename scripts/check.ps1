# same checks as ci. usage: .\scripts\check.ps1
$ErrorActionPreference = "Stop"

Write-Host "==> cargo fmt --check" -ForegroundColor Cyan
cargo fmt --all -- --check
if ($LASTEXITCODE -ne 0) { throw "formatting check failed; run 'cargo fmt --all'" }

Write-Host "==> cargo clippy" -ForegroundColor Cyan
cargo clippy --workspace --all-targets -- -D warnings
if ($LASTEXITCODE -ne 0) { throw "clippy failed" }

Write-Host "==> cargo test" -ForegroundColor Cyan
cargo test --workspace --all-targets
if ($LASTEXITCODE -ne 0) { throw "tests failed" }

Write-Host "all checks passed" -ForegroundColor Green

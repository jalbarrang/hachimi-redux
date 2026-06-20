<#
.SYNOPSIS
    Local quality gate script — mirrors CI pipeline.
    Run this before pushing to catch issues locally instead of waiting for CI.

.DESCRIPTION
    Executes the same checks as .github/workflows/ci.yml in order:
      1. rustfmt (formatting)
      2. cargo-deny (supply chain)
      3. cargo-machete (unused deps)
      4. clippy with -D warnings (zero-warning lint)
      5. cargo check (type verification)

    Exit code 0 = all gates passed. Non-zero = at least one gate failed.

.EXAMPLE
    .\scripts\quality-gates.ps1
    .\scripts\quality-gates.ps1 -SkipDeny   # skip cargo-deny (if not installed)
    .\scripts\quality-gates.ps1 -Quick       # only fmt + clippy (fastest feedback)
#>

param(
    [switch]$SkipDeny,
    [switch]$SkipMachete,
    [switch]$Quick
)

$ErrorActionPreference = "Stop"
$failed = @()

function Run-Gate {
    param([string]$Name, [scriptblock]$Command)

    Write-Host ""
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
    Write-Host "  $Name" -ForegroundColor Cyan
    Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

    try {
        & $Command
        if ($LASTEXITCODE -ne 0) {
            Write-Host "  ✗ FAILED: $Name" -ForegroundColor Red
            $script:failed += $Name
        } else {
            Write-Host "  ✓ PASSED: $Name" -ForegroundColor Green
        }
    } catch {
        Write-Host "  ✗ ERROR: $Name — $_" -ForegroundColor Red
        $script:failed += $Name
    }
}

$stopwatch = [System.Diagnostics.Stopwatch]::StartNew()

# ── Gate 1: Formatting ──────────────────────────────────────────
# Workspace-root gates cover every default member (core + the race-hud / debug-viewer
# plugins). Training Tracker is now an in-core module under apps/hachimi.
Run-Gate "Rustfmt" { cargo fmt --check }

# ── Gate 2: Supply chain ────────────────────────────────────────
if (-not $Quick -and -not $SkipDeny) {
    $denyCmd = Get-Command cargo-deny -ErrorAction SilentlyContinue
    if ($denyCmd) {
        Run-Gate "cargo-deny" { cargo deny check }
    } else {
        Write-Host ""
        Write-Host "  ⚠ SKIP: cargo-deny not installed (cargo install cargo-deny)" -ForegroundColor Yellow
    }
}

# ── Gate 3: Unused dependencies ─────────────────────────────────
if (-not $Quick -and -not $SkipMachete) {
    $macheteCmd = Get-Command cargo-machete -ErrorAction SilentlyContinue
    if ($macheteCmd) {
        Run-Gate "cargo-machete" { cargo machete }
    } else {
        Write-Host ""
        Write-Host "  ⚠ SKIP: cargo-machete not installed (cargo install cargo-machete)" -ForegroundColor Yellow
    }
}

# ── Gate 4: Clippy (zero warnings) ──────────────────────────────
Run-Gate "Clippy (zero warnings)" {
    cargo clippy --all-targets -- -D warnings
}

# ── Gate 5: Tests ─────────────────────────────────────────────────────
Run-Gate "Tests" { cargo test --lib }

# ── Gate 6: Type check ──────────────────────────────────────────
if (-not $Quick) {
    Run-Gate "Cargo check" { cargo check --all-targets }
}

# ── Summary ─────────────────────────────────────────────────────
$stopwatch.Stop()
Write-Host ""
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan
Write-Host "  Quality Gates Complete ($([math]::Round($stopwatch.Elapsed.TotalSeconds, 1))s)" -ForegroundColor Cyan
Write-Host "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━" -ForegroundColor Cyan

if ($failed.Count -gt 0) {
    Write-Host ""
    Write-Host "  ✗ $($failed.Count) gate(s) FAILED:" -ForegroundColor Red
    foreach ($f in $failed) {
        Write-Host "    - $f" -ForegroundColor Red
    }
    Write-Host ""
    exit 1
} else {
    Write-Host ""
    Write-Host "  ✓ All gates passed — safe to push." -ForegroundColor Green
    Write-Host ""
    exit 0
}

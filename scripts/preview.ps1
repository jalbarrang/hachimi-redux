<#
.SYNOPSIS
    Run a desktop UI preview (no game) for the host Control Center menu.

.DESCRIPTION
    The preview is an eframe window that renders the *real* UI draw code against
    mocked/default data behind a `dev-harness` Cargo feature — no game process,
    no IL2CPP, no D3D11. Fast (~1s) rebuilds for iterating on layout/styling.

    Target:
    - menu → cargo run -p hachimi --example menu_preview --features dev-harness
             (Control Center shell + General/Graphics/Gameplay/Hotkeys/
              Translations; Plugins/About are stubs.)

    (The former standalone Training Tracker overlay preview was retired when the
    tracker moved in-core; its UI is exercised through the host menu preview.)

.PARAMETER Release
    Build/run the preview in release mode (slower build, smoother window).

.EXAMPLE
    .\scripts\preview.ps1
    # Control Center menu preview.
#>
param(
    [switch]$Release
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Push-Location $RepoRoot
try {
    $argList = @("run", "-p", "hachimi", "--example", "menu_preview", "--features", "dev-harness")
    if ($Release) { $argList += "--release" }
    Write-Host "→ cargo $($argList -join ' ')" -ForegroundColor Cyan
    & cargo @argList
}
finally {
    Pop-Location
}

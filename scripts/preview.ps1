<#
.SYNOPSIS
    Run a desktop UI preview (no game) for the host Control Center menu and/or
    the Training Tracker overlay.

.DESCRIPTION
    Both previews are eframe windows that render the *real* UI draw code against
    mocked/default data behind a `dev-harness` Cargo feature — no game process,
    no IL2CPP, no D3D11. Fast (~1s) rebuilds for iterating on layout/styling.

    Targets:
    - menu    → cargo run -p hachimi --example menu_preview --features dev-harness
                (Control Center shell + General/Graphics/Gameplay/Hotkeys/
                 Translations; Plugins/About are stubs.)
    - tracker → cargo run -p hachimi-training-tracker --example overlay_preview --features dev-harness
                (Training Tracker overlay with a mocked late-game career.)

    The tracker preview renders real game sprites if it can find an icons dir;
    it checks $env:TT_PREVIEW_ICONS, $env:HONSE_ICONS_DIR, then
    <GameDir>\hachimi\icons. -GameDir / $env:HACHIMI_GAME_DIR set that fallback.

.PARAMETER Target
    Which preview to run: menu, tracker, or both. Default: menu.
    ("both" opens two windows.)

.PARAMETER Release
    Build/run the preview in release mode (slower build, smoother window).

.PARAMETER GameDir
    Game directory used only to locate the tracker's icons dir.
    Defaults to $env:HACHIMI_GAME_DIR or the standard Steam path.

.EXAMPLE
    .\scripts\preview.ps1
    # Control Center menu preview.

.EXAMPLE
    .\scripts\preview.ps1 -Target tracker

.EXAMPLE
    .\scripts\preview.ps1 -Target both
#>
param(
    [ValidateSet("menu", "tracker", "both")]
    [string]$Target = "menu",
    [switch]$Release,
    [string]$GameDir = $(if ($env:HACHIMI_GAME_DIR) { $env:HACHIMI_GAME_DIR } else {
        "${env:ProgramFiles(x86)}\Steam\steamapps\common\UmamusumePrettyDerby"
    })
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
Push-Location $RepoRoot
try {
    # Help the tracker harness find real sprites without forcing the user to set
    # env vars: fall back to the deployed icons dir under the game directory.
    if (-not $env:TT_PREVIEW_ICONS -and -not $env:HONSE_ICONS_DIR) {
        $env:HACHIMI_GAME_DIR = $GameDir
    }

    $relArgs = @()
    if ($Release) { $relArgs += "--release" }

    function Start-Preview {
        param(
            [string]$Package,
            [string]$Example,
            [switch]$Background
        )
        $argList = @("run", "-p", $Package, "--example", $Example, "--features", "dev-harness") + $relArgs
        Write-Host "→ cargo $($argList -join ' ')" -ForegroundColor Cyan
        if ($Background) {
            Start-Process -FilePath "cargo" -ArgumentList $argList -WorkingDirectory $RepoRoot | Out-Null
        }
        else {
            & cargo @argList
        }
    }

    switch ($Target) {
        "menu" {
            Start-Preview -Package "hachimi" -Example "menu_preview"
        }
        "tracker" {
            Start-Preview -Package "hachimi-training-tracker" -Example "overlay_preview"
        }
        "both" {
            # First window in the background, second in the foreground so the
            # script blocks until you close it.
            Start-Preview -Package "hachimi" -Example "menu_preview" -Background
            Start-Preview -Package "hachimi-training-tracker" -Example "overlay_preview"
        }
    }
}
finally {
    Pop-Location
}

<#
.SYNOPSIS
    Copy release-built Hachimi core and/or its cdylib plugins into the game directory.

.DESCRIPTION
    By default copies:
    - target\release\hachimi.dll → <GameDir>\cri_mana_vpx.dll (proxy; Training Tracker
      is compiled into this)
    - target\release\hachimi_race_hud.dll → <GameDir>\  (hot-swappable SDK dogfood)
    - target\release\hachimi_debug_viewer.dll → <GameDir>\  (dev-only diagnostics)

    With -PluginOnly, copies the race-hud plugin but skips the proxy.
    Never modifies cri_mana_vpx.dll.backup.

    -HotSwap only swaps the race-hud plugin via IPC; the debug-viewer plugin is
    deployed only in non-HotSwap runs (need a game restart).

    With -PluginOnly -HotSwap, unloads the plugin via Hachimi IPC (requires
    enable_ipc in config.json), copies the new DLL, then reloads it — no game restart.

.PARAMETER GameDir
    The Honse Game install folder (contains the game exe and cri_mana_vpx.dll).
    Defaults to $env:HACHIMI_GAME_DIR or the standard Steam path.

.PARAMETER Build
    Run `cargo build --release` before copying. Builds hachimi and the plugins by
    default; with -PluginOnly, builds only hachimi-race-hud.

.PARAMETER PluginOnly
    Deploy only hachimi_race_hud.dll. Skips the core proxy (cri_mana_vpx.dll).

.PARAMETER HotSwap
    Requires -PluginOnly. Unload the plugin via IPC, copy, reload via IPC. Requires
    the game to be running with enable_ipc: true in config.json.

.EXAMPLE
    .\scripts\deploy-windows.ps1
    .\scripts\deploy-windows.ps1 -Build
    .\scripts\deploy-windows.ps1 -GameDir "D:\Games\UmamusumePrettyDerby"

.EXAMPLE
    $env:HACHIMI_GAME_DIR = "C:\Program Files (x86)\Steam\steamapps\common\UmamusumePrettyDerby"
    .\scripts\deploy-windows.ps1 -Build

.EXAMPLE
    .\scripts\deploy-windows.ps1 -PluginOnly -Build

.EXAMPLE
    .\scripts\deploy-windows.ps1 -PluginOnly -HotSwap -Build
#>

param(
    [string]$GameDir = $(if ($env:HACHIMI_GAME_DIR) { $env:HACHIMI_GAME_DIR } else {
        "${env:ProgramFiles(x86)}\Steam\steamapps\common\UmamusumePrettyDerby"
    }),
    [switch]$Build,
    [switch]$PluginOnly,
    [switch]$HotSwap
)

$ErrorActionPreference = "Stop"

$RepoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$TargetDir = Join-Path $RepoRoot "target\release"
$HostDll = Join-Path $TargetDir "hachimi.dll"
# Training Tracker now ships inside hachimi.dll. The hot-swappable cdylib plugin is
# race-hud (the SDK dogfood); debug-viewer is the dev-only diagnostics plugin.
$PluginDll = Join-Path $TargetDir "hachimi_race_hud.dll"
$PluginFileName = "hachimi_race_hud.dll"
$DebugViewerDll = Join-Path $TargetDir "hachimi_debug_viewer.dll"
$DebugViewerFileName = "hachimi_debug_viewer.dll"
$ProxyName = "cri_mana_vpx.dll"
$BackupName = "cri_mana_vpx.dll.backup"
$IpcUrl = "http://127.0.0.1:50433"

function Require-File {
    param([string]$Path, [string]$Hint)
    if (-not (Test-Path -LiteralPath $Path)) {
        Write-Error "Missing: $Path`n$Hint"
    }
}

function Test-HachimiIpc {
    try {
        $null = Invoke-RestMethod -Uri $IpcUrl -Method Get -TimeoutSec 3
        return $true
    } catch {
        return $false
    }
}

function Invoke-HachimiIpcCommand {
    param(
        [Parameter(Mandatory)]
        [string]$Type,
        [hashtable]$Payload = @{}
    )
    $body = @{ type = $Type }
    foreach ($key in $Payload.Keys) {
        $body[$key] = $Payload[$key]
    }
    $json = $body | ConvertTo-Json -Compress
    try {
        $response = Invoke-RestMethod -Uri $IpcUrl -Method Post `
            -ContentType "application/json" -Body $json -TimeoutSec 30
    } catch {
        Write-Error @"
IPC request failed ($Type).
Ensure the game is running, Hachimi is loaded, and enable_ipc is true in config.json.
Details: $_
"@
    }
    if ($response.type -eq "Error") {
        $msg = if ($response.message) { $response.message } else { "(no message)" }
        Write-Error "IPC $Type failed: $msg"
    }
}

function Unload-PluginViaIpc {
    param([string]$Name)
    Write-Host "  Unloading $Name via IPC..." -ForegroundColor Cyan
    Invoke-HachimiIpcCommand -Type "UnloadPlugin" -Payload @{ name = $Name } | Out-Null
}

function Reload-PluginViaIpc {
    param([string]$Name)
    Write-Host "  Reloading $Name via IPC..." -ForegroundColor Cyan
    Invoke-HachimiIpcCommand -Type "ReloadPlugin" -Payload @{ name = $Name } | Out-Null
}

function Copy-PluginDll {
    param(
        [string]$Source,
        [string]$Dest,
        [switch]$HotSwap
    )
    if ($HotSwap) {
        if (-not (Test-HachimiIpc)) {
            Write-Error @"
-HotSwap requires Hachimi IPC (enable_ipc: true in config.json) with the game running.
IPC did not respond at $IpcUrl
"@
        }
        Unload-PluginViaIpc -Name $PluginFileName
        Start-Sleep -Milliseconds 250
    }

    $maxAttempts = if ($HotSwap) { 8 } else { 1 }
    for ($i = 1; $i -le $maxAttempts; $i++) {
        try {
            Copy-Item -LiteralPath $Source -Destination $Dest -Force
            return
        } catch {
            $locked = $_.Exception.Message -match "being used by another process"
            if (-not $locked -or $i -eq $maxAttempts) {
                if ($locked -and -not $HotSwap) {
                    Write-Error @"
Cannot overwrite $Dest - the plugin DLL is locked by the running game.

Use -HotSwap to unload the plugin via IPC, copy, and reload:
  .\scripts\deploy-windows.ps1 -PluginOnly -HotSwap -Build

Requires enable_ipc: true in config.json.
"@
                }
                throw
            }
            Start-Sleep -Milliseconds 200
        }
    }
}

if ($HotSwap -and -not $PluginOnly) {
    Write-Error "-HotSwap requires -PluginOnly"
}

if ($Build) {
    Write-Host "Building release artifacts..." -ForegroundColor Cyan
    Push-Location $RepoRoot
    try {
        if (-not $PluginOnly) {
            cargo build --release -p hachimi
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        }
        cargo build --release -p hachimi-race-hud
        if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        if (-not $PluginOnly) {
            cargo build --release -p hachimi-debug-viewer
            if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
        }
    }
    finally {
        Pop-Location
    }
}

if (-not $PluginOnly) {
    Require-File $HostDll "Run: cargo build --release -p hachimi`nOr pass -Build"
}
Require-File $PluginDll "Run: cargo build --release -p hachimi-race-hud`nOr pass -Build"
if (-not $PluginOnly) {
    Require-File $DebugViewerDll "Run: cargo build --release -p hachimi-debug-viewer`nOr pass -Build"
}

$GameDir = $GameDir.TrimEnd('\')
if (-not (Test-Path -LiteralPath $GameDir -PathType Container)) {
    Write-Error @"
Game directory not found: $GameDir

Set -GameDir or env:HACHIMI_GAME_DIR to your UmamusumePrettyDerby folder.
"@
}

$ProxyPath = Join-Path $GameDir $ProxyName
$BackupPath = Join-Path $GameDir $BackupName
$PluginDest = Join-Path $GameDir $PluginFileName
$DebugViewerDest = Join-Path $GameDir $DebugViewerFileName

if (-not $PluginOnly) {
    if (-not (Test-Path -LiteralPath $BackupPath)) {
        Write-Warning @"
$BackupName not found in the game folder.

Before the first proxy install, back up the stock DLL, e.g.:
  Copy-Item -LiteralPath '$ProxyPath' -Destination '$BackupPath'
(Only if '$ProxyName' is still the original game file.)

This script will NOT create or modify $BackupName.
"@
    }
}

Write-Host ""
if ($PluginOnly) {
    if ($HotSwap) {
        Write-Host "Hot-swapping plugin at: $GameDir" -ForegroundColor Green
    } else {
        Write-Host "Deploying plugin only to: $GameDir" -ForegroundColor Green
    }
} else {
    Write-Host "Deploying to: $GameDir" -ForegroundColor Green
}
Write-Host ""

if (-not $PluginOnly) {
    Copy-Item -LiteralPath $HostDll -Destination $ProxyPath -Force
    Write-Host "  hachimi.dll  ->  $ProxyName"
}

Copy-PluginDll -Source $PluginDll -Dest $PluginDest -HotSwap:$HotSwap
Write-Host "  hachimi_race_hud.dll  ->  hachimi_race_hud.dll"

if ($HotSwap) {
    Reload-PluginViaIpc -Name $PluginFileName
} else {
    Copy-PluginDll -Source $DebugViewerDll -Dest $DebugViewerDest
    Write-Host "  hachimi_debug_viewer.dll  ->  hachimi_debug_viewer.dll"
}

# Training-tracker data resources. At runtime the host downloads these into the
# game data dir (<GameDir>\hachimi, next to config.json) via the hosted_data sync,
# which is where the plugin's host_data_path lookup reads them. For local dev we
# drop them straight into that data dir so they're found without a download (and a
# local edit wins over the hosted copy).
$DataDir = Join-Path $GameDir 'hachimi'
if (-not (Test-Path -LiteralPath $DataDir -PathType Container)) {
  New-Item -ItemType Directory -Path $DataDir -Force | Out-Null
}

$SkillGradesSrc = Join-Path $PSScriptRoot "..\apps\hachimi\src\core\modules\training_tracker\assets\skill_grades.json"
if (Test-Path -LiteralPath $SkillGradesSrc) {
  Copy-Item -LiteralPath $SkillGradesSrc -Destination (Join-Path $DataDir "skill_grades.json") -Force
  Write-Host "  skill_grades.json  ->  hachimi\skill_grades.json"
} else {
  Write-Host "  (skill_grades.json missing; run: cargo run -p fetch-master-db; cargo run -p skill-grades)" -ForegroundColor Yellow
}

$CourseParamsSrc = Join-Path $PSScriptRoot "..\apps\hachimi\src\core\modules\training_tracker\assets\course_params.json"
if (Test-Path -LiteralPath $CourseParamsSrc) {
  Copy-Item -LiteralPath $CourseParamsSrc -Destination (Join-Path $DataDir "course_params.json") -Force
  Write-Host "  course_params.json  ->  hachimi\course_params.json"
} else {
  Write-Host "  (course_params.json missing; run: cargo run -p fetch-master-db; cargo run -p course-data)" -ForegroundColor Yellow
}

# Career-panel icon sprites (trainee portraits, rank sprites, stat/skill icons).
# These are the game UI sprites the honse-tracker dashboard already extracted; the
# overlay loads them on demand from <GameDir>\hachimi\icons via host_data_path.
# Source defaults to the sibling honse-tracker checkout; override with
# $env:HONSE_ICONS_DIR. ~16 MB; only the active frame's handful are loaded.
$IconsSrc = if ($env:HONSE_ICONS_DIR) { $env:HONSE_ICONS_DIR } else {
  Join-Path $PSScriptRoot "..\..\honse-tracker\apps\web\public\icons"
}
if (Test-Path -LiteralPath $IconsSrc -PathType Container) {
  $IconsDest = Join-Path $DataDir 'icons'
  if (-not (Test-Path -LiteralPath $IconsDest -PathType Container)) {
    New-Item -ItemType Directory -Path $IconsDest -Force | Out-Null
  }
  Copy-Item -Path (Join-Path $IconsSrc '*') -Destination $IconsDest -Recurse -Force
  $IconCount = (Get-ChildItem -LiteralPath $IconsDest -Recurse -File | Measure-Object).Count
  Write-Host "  icons ($IconCount files)  ->  hachimi\icons\"
} else {
  Write-Host "  (career icons missing; set `$env:HONSE_ICONS_DIR or clone honse-tracker beside this repo)" -ForegroundColor Yellow
}

Write-Host ""
Write-Host "Done. Ensure config.json lists the cdylib plugins under windows.load_libraries:" -ForegroundColor Cyan
Write-Host '  "load_libraries": ["hachimi_race_hud.dll", "hachimi_debug_viewer.dll"]'
Write-Host "(Training Tracker is built into hachimi.dll — no load_libraries entry needed.)"
if ($PluginOnly -and -not $HotSwap) {
    Write-Host ""
    Write-Host "If the game is already running, use -HotSwap or About -> Danger Zone -> Reload plugins." -ForegroundColor Cyan
} elseif ($HotSwap) {
    Write-Host ""
    Write-Host "Plugin hot-swapped via IPC - do not also click Reload plugins." -ForegroundColor Green
}
Write-Host ""
Write-Host "Launch the game yourself to verify (this script does not start the game)." -ForegroundColor DarkGray

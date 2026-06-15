#Requires -Version 5.1
<#
.SYNOPSIS
    SelfHost Helper — Full Build Pipeline (Windows)
.DESCRIPTION
    Cross-platform build script. On Windows, use this PowerShell script.
    On Linux/macOS, use build.sh instead.
.PARAMETER VersionName
    Version tag for the build output. Default: "local"
.PARAMETER Bundles
    Bundle types to build. Default: "nsis"
    Valid values: nsis
.PARAMETER OutputDir
    Custom output directory. Default: ..\SelfHost-Helper-Versions\<VersionName>
.PARAMETER SkipDeps
    Skip the npm install step.
.PARAMETER DryRun
    Show what would be built without building.
.EXAMPLE
    .\build.ps1
    .\build.ps1 -VersionName "v0.41.0"
    .\build.ps1 -VersionName "v2" -Bundles "nsis"
#>

[CmdletBinding()]
param(
    [string]$VersionName = "local",
    [string[]]$Bundles = @("nsis"),
    [string]$OutputDir = "",
    [switch]$SkipDeps,
    [switch]$DryRun,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

# ─── Helpers ───────────────────────────────────────────────────────
function Write-Step {
    param([string]$StepNum, [string]$Total, [string]$Message)
    Write-Host ""
    Write-Host "  === [$StepNum/$Total] $Message ===" -ForegroundColor Cyan
}

function Write-OK    { param([string]$Msg) Write-Host "  [OK] $Msg" -ForegroundColor Green }
function Write-Warn  { param([string]$Msg) Write-Host "  [!!] $Msg" -ForegroundColor Yellow }
function Write-Err   { param([string]$Msg) Write-Host "  [ERR] $Msg" -ForegroundColor Red }
function Write-Info  { param([string]$Msg) Write-Host "  |  $Msg" -ForegroundColor Gray }

$ScriptRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Set-Location $ScriptRoot

$Stopwatch = [System.Diagnostics.Stopwatch]::StartNew()
$TotalSteps = 5

# ─── Banner ────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  ========================================================" -ForegroundColor Magenta
Write-Host "       SelfHost Helper - Build Pipeline (Windows)" -ForegroundColor White
Write-Host "       Version: $VersionName" -ForegroundColor Gray
Write-Host "  ========================================================" -ForegroundColor Magenta
Write-Host ""

# ─── Platform ──────────────────────────────────────────────────────
$IsWindows = $true
$Arch = if ([Environment]::Is64BitOperatingSystem) { "x86_64" } else { "x86" }
Write-Info "Platform: Windows ($Arch)"
Write-Info "Root:     $ScriptRoot"

# ─── Help ──────────────────────────────────────────────────────────
if ($Help) {
    Get-Help $MyInvocation.MyCommand.Path -Detailed
    exit 0
}

# ─── Parse Bundles ────────────────────────────────────────────────
$ValidBundles = @("nsis")
foreach ($b in $Bundles) {
    if ($b -notin $ValidBundles) {
        Write-Err "Invalid bundle type '$b'. Valid: $($ValidBundles -join ', ')"
        exit 1
    }
}

Write-Info "Bundles:  $($Bundles -join ', ')"
Write-Host "  -------------------------------------------------------" -ForegroundColor DarkGray

if ($DryRun) {
    Write-Host ""
    Write-Info "Dry run - would execute:"
    Write-Info "  1. npm install --legacy-peer-deps"
    Write-Info "  2. npx vite build"
    Write-Info "  3. cargo tauri build --bundles $($Bundles -join ',')"
    Write-Info "  4. Copy artifacts to $OutputDir"
    Write-Info "  5. Print summary"
    Write-Host ""
    exit 0
}

# ─── Dependency Checks ────────────────────────────────────────────
$Missing = @()
foreach ($cmd in @("npm", "node", "cargo")) {
    if (-not (Get-Command $cmd -ErrorAction SilentlyContinue)) {
        $Missing += $cmd
    }
}
if ($Missing.Count -gt 0) {
    Write-Err "Missing required tools: $($Missing -join ', ')"
    exit 1
}

# ─── Step 1: Install Dependencies ────────────────────────────────
Write-Step 1 $TotalSteps "Installing frontend dependencies..."

if ($SkipDeps) {
    Write-Warn "Skipped (-SkipDeps)"
} else {
    if (Test-Path "package-lock.json") {
        Write-Info "Lockfile found - using npm ci"
        & npm ci --legacy-peer-deps 2>&1 | Select-Object -Last 5
    } else {
        & npm install --legacy-peer-deps 2>&1 | Select-Object -Last 5
    }
    if ($LASTEXITCODE -ne 0) { Write-Err "npm install failed"; exit 1 }
    Write-OK "Dependencies installed ($($Stopwatch.Elapsed))"
}

# ─── Step 2: Build Frontend ──────────────────────────────────────
Write-Step 2 $TotalSteps "Building frontend (Vite)..."

Write-Info "Running: npx vite build"
& npx vite build 2>&1 | Select-Object -Last 3
if ($LASTEXITCODE -ne 0) { Write-Err "Vite build failed"; exit 1 }

if (-not (Test-Path "dist") -or (Get-ChildItem "dist" -ErrorAction SilentlyContinue).Count -eq 0) {
    Write-Err "Vite build failed - dist/ is empty or missing"
    exit 1
}

$FileCount = (Get-ChildItem "dist" -Recurse -File).Count
$DirSize = "{0:N1} MB" -f ((Get-ChildItem "dist" -Recurse -File | Measure-Object -Property Length -Sum).Sum / 1MB)
Write-OK "Frontend built: $FileCount files, $DirSize ($($Stopwatch.Elapsed))"

# ─── Step 3: Build Tauri ────────────────────────────────────────
Write-Step 3 $TotalSteps "Building Tauri release binary..."

$BundleArgs = ($Bundles | ForEach-Object { "--bundles", $_ }) -join " "
Write-Info "Running: cargo tauri build $BundleArgs"
Write-Info "This may take a while on first build..."

$TauriStart = [System.Diagnostics.Stopwatch]::StartNew()
$TauriArgList = @("tauri", "build") + ($Bundles | ForEach-Object { "--bundles"; $_ })

& cargo @TauriArgList
if ($LASTEXITCODE -ne 0) {
    Write-Err "Tauri build failed!"
    Write-Host ""
    Write-Err "Common fixes:"
    Write-Err "  - Install Rust: https://rustup.rs"
    Write-Err "  - Install Tauri CLI: cargo install tauri-cli"
    Write-Err "  - Install Windows Build Tools: winget install Microsoft.VisualStudio.2022.BuildTools"
    Write-Err "  - Ensure WebView2 is installed (usually comes with Windows 10/11)"
    exit 1
}

$TauriStop = $TauriStart.Elapsed
Write-OK "Tauri build complete: $([int]$TauriStop.TotalMinutes)m $($TauriStop.Seconds)s ($($Stopwatch.Elapsed))"

# ─── Step 4: Collect Artifacts ───────────────────────────────────
Write-Step 4 $TotalSteps "Collecting build artifacts..."

if (-not $OutputDir) {
    $OutputDir = Join-Path (Split-Path $ScriptRoot -Parent) "SelfHost-Helper-Versions\$VersionName"
}
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

$ArtifactCount = 0

# Binary
$BinarySrc = "src-tauri\target\release\selfhost-helper.exe"
if (Test-Path $BinarySrc) {
    Copy-Item $BinarySrc "$OutputDir\"
    $ArtifactCount++
    $BinSize = "{0:N1} MB" -f ((Get-Item $BinarySrc).Length / 1MB)
    Write-OK "Binary: selfhost-helper.exe ($BinSize)"
} else {
    Write-Warn "Binary not found at $BinarySrc"
}

# Bundle
$BundleSrc = "src-tauri\target\release\bundle"
if (Test-Path $BundleSrc) {
    Copy-Item $BundleSrc "$OutputDir\bundle" -Recurse
    $ArtifactCount++

    $NsisFile = Get-ChildItem "$BundleSrc\nsis" -Filter "*.exe" -Recurse -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($NsisFile) {
        $NsisSize = "{0:N1} MB" -f ($NsisFile.Length / 1MB)
        Write-OK "NSIS: $($NsisFile.Name) ($NsisSize)"
    }
}

# Source snapshot
Write-Info "Copying source snapshot..."
Copy-Item "dist" "$OutputDir\dist" -Recurse
Copy-Item "src" "$OutputDir\src" -Recurse
Copy-Item "src-tauri" "$OutputDir\src-tauri" -Recurse

foreach ($f in @("package.json", "vite.config.js", "AGENTS.md", "MIGRATION_PLAN.md", "SECURITY_AUDIT_PLAN.md")) {
    if (Test-Path $f) { Copy-Item $f "$OutputDir\" -ErrorAction SilentlyContinue }
}

Write-Host "  -------------------------------------------------------" -ForegroundColor DarkGray
Write-OK "Artifacts collected: $ArtifactCount bundle(s) + source snapshot"

# ─── Step 5: Summary ──────────────────────────────────────────────
Write-Step 5 $TotalSteps "Build summary"

$TotalSize = "{0:N1} MB" -f ((Get-ChildItem $OutputDir -Recurse -File | Measure-Object -Property Length -Sum).Sum / 1MB)

Write-Host ""
Write-Host "  ========================================================" -ForegroundColor Magenta
Write-Host "    Build Complete!" -ForegroundColor Green
Write-Host ""
Write-Host "    Version:     $VersionName" -ForegroundColor White
Write-Host "    Platform:    Windows ($Arch)" -ForegroundColor White
Write-Host "    Bundles:     $($Bundles -join ', ')" -ForegroundColor White
Write-Host "    Duration:    $($Stopwatch.Elapsed)" -ForegroundColor White
Write-Host "    Output:      $OutputDir" -ForegroundColor White
Write-Host "    Total size:  $TotalSize" -ForegroundColor White
Write-Host ""
Write-Host "    Files:" -ForegroundColor White

if (Test-Path "$OutputDir\selfhost-helper.exe") {
    Write-Host "      -> Binary: $OutputDir\selfhost-helper.exe" -ForegroundColor Green
}
if (Test-Path "$OutputDir\bundle") {
    Write-Host "      -> Bundle: $OutputDir\bundle\" -ForegroundColor Green
}
Write-Host "      -> Source: $OutputDir\src\" -ForegroundColor Green
Write-Host "  ========================================================" -ForegroundColor Magenta
Write-Host ""

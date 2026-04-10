param(
    [string]$Version = "v0.1.0",
    [string]$TargetDir = "C:\Users\AKR\.codex\tmp\ccx-release-target"
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot
$distDir = Join-Path $repoRoot "dist"
$toolchain = "C:\Users\AKR\.rustup\toolchains\1.91.0-x86_64-pc-windows-msvc\bin"

if (-not (Test-Path -LiteralPath $toolchain)) {
    throw "Rust toolchain not found at $toolchain"
}

$env:PATH = "$toolchain;$env:PATH"
$env:CARGO_TARGET_DIR = $TargetDir

Push-Location $repoRoot
try {
    cargo.exe build --release --bin ccx

    New-Item -ItemType Directory -Force -Path $distDir | Out-Null

    $binaryPath = Join-Path $TargetDir "release\\ccx.exe"
    if (-not (Test-Path -LiteralPath $binaryPath)) {
        throw "Built binary not found at $binaryPath"
    }

    $artifactName = "ccx-windows-x86_64-$Version"
    $stagingDir = Join-Path $distDir $artifactName
    $zipPath = Join-Path $distDir "$artifactName.zip"
    $shaPath = Join-Path $distDir "$artifactName.sha256.txt"

    Remove-Item -LiteralPath $stagingDir -Recurse -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $zipPath -Force -ErrorAction SilentlyContinue
    Remove-Item -LiteralPath $shaPath -Force -ErrorAction SilentlyContinue

    New-Item -ItemType Directory -Force -Path $stagingDir | Out-Null
    Copy-Item -LiteralPath $binaryPath -Destination (Join-Path $stagingDir "ccx.exe")
    Copy-Item -LiteralPath (Join-Path $repoRoot "README.md") -Destination (Join-Path $stagingDir "README.md")
    Copy-Item -LiteralPath (Join-Path $repoRoot "LICENSE") -Destination (Join-Path $stagingDir "LICENSE")

    @"
Codex Continuity OS Windows release package.

Contents:
- ccx.exe
- README.md
- LICENSE

Quick start:
1. Run .\ccx.exe index
2. Run .\ccx.exe dashboard
3. Use ? inside the dashboard for first-run help
"@ | Set-Content -LiteralPath (Join-Path $stagingDir "QUICKSTART.txt")

    Compress-Archive -Path (Join-Path $stagingDir "*") -DestinationPath $zipPath

    $hash = Get-FileHash -Algorithm SHA256 -LiteralPath $zipPath
    "$($hash.Hash.ToLower())  $(Split-Path -Leaf $zipPath)" | Set-Content -LiteralPath $shaPath

    Write-Host "Created release package:"
    Write-Host "  $zipPath"
    Write-Host "Created checksum:"
    Write-Host "  $shaPath"
}
finally {
    Pop-Location
}

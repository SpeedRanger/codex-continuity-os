param(
    [string]$BinaryPath = "",
    [string]$InstallDir = "$env:USERPROFILE\.codex-continuity\bin",
    [switch]$AddToUserPath
)

$ErrorActionPreference = "Stop"

$repoRoot = Split-Path -Parent $PSScriptRoot

if ([string]::IsNullOrWhiteSpace($BinaryPath)) {
    $releaseBinary = Join-Path $repoRoot "target\release\ccx.exe"
    $debugBinary = Join-Path $repoRoot "target\debug\ccx.exe"

    if (Test-Path -LiteralPath $releaseBinary) {
        $BinaryPath = $releaseBinary
    }
    elseif (Test-Path -LiteralPath $debugBinary) {
        $BinaryPath = $debugBinary
    }
    else {
        throw "No ccx.exe found. Build first with: cargo build --release --bin ccx"
    }
}

$resolvedBinary = Resolve-Path -LiteralPath $BinaryPath
New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null

$targetBinary = Join-Path $InstallDir "ccx.exe"
Copy-Item -LiteralPath $resolvedBinary -Destination $targetBinary -Force

Write-Host "Installed ccx.exe:"
Write-Host "  $targetBinary"

if ($AddToUserPath) {
    $currentPath = [Environment]::GetEnvironmentVariable("Path", "User")
    $pathEntries = $currentPath -split ';' | Where-Object { -not [string]::IsNullOrWhiteSpace($_) }
    $alreadyPresent = $pathEntries | Where-Object { $_.TrimEnd('\') -ieq $InstallDir.TrimEnd('\') }

    if (-not $alreadyPresent) {
        $newPath = if ([string]::IsNullOrWhiteSpace($currentPath)) {
            $InstallDir
        }
        else {
            "$currentPath;$InstallDir"
        }
        [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
        Write-Host "Added install directory to the user PATH."
        Write-Host "Open a new terminal before running ccx by name."
    }
    else {
        Write-Host "Install directory is already present in the user PATH."
    }
}
else {
    Write-Host "PATH was not modified."
    Write-Host "Run with -AddToUserPath to add this install directory to your user PATH."
}

Write-Host ""
Write-Host "Next commands:"
Write-Host "  $targetBinary doctor"
Write-Host "  $targetBinary dashboard"

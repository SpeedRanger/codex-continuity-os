[CmdletBinding()]
param(
    [string]$Repo = 'D:\saas-workspace\products\roompilot-ai',
    [string]$CompareSessionA = '019d1f8d-698d-70d1-b07d-f099066d4d34',
    [string]$CompareSessionB = '019d30b1-1b6f-77a3-8c4b-cfcfe2d10973',
    [string]$Query = 'prompt profiles',
    [string]$Toolchain = 'C:\Users\AKR\.rustup\toolchains\1.91.0-x86_64-pc-windows-msvc\bin',
    [string]$CargoTargetDir = 'C:\Users\AKR\.codex\tmp\ccx-demo-target'
)

$ErrorActionPreference = 'Stop'

$repoRoot = Split-Path -Parent $PSScriptRoot
$env:PATH = "$Toolchain;$env:PATH"
$env:CARGO_TARGET_DIR = $CargoTargetDir
$binary = Join-Path $CargoTargetDir 'debug\ccx.exe'

function Invoke-DemoStep {
    param(
        [Parameter(Mandatory = $true)]
        [string]$Label,
        [Parameter(Mandatory = $true)]
        [scriptblock]$Action
    )

    Write-Host ''
    Write-Host "=== $Label ===" -ForegroundColor Cyan
    $sw = [System.Diagnostics.Stopwatch]::StartNew()
    & $Action
    $sw.Stop()
    Write-Host ("ELAPSED_MS={0}" -f $sw.ElapsedMilliseconds) -ForegroundColor DarkGray
}

Push-Location $repoRoot
try {
    Invoke-DemoStep -Label 'Build fresh isolated demo binary' -Action {
        cargo.exe build --bin ccx
    }

    if (-not (Test-Path $binary)) {
        throw "Expected demo binary at $binary after build."
    }

    Write-Host ''
    Write-Host "Binary: $binary" -ForegroundColor Yellow

    Invoke-DemoStep -Label 'Rebuild cache' -Action {
        & $binary index
    }

    Invoke-DemoStep -Label 'List projects' -Action {
        & $binary projects
    }

    Invoke-DemoStep -Label "Resume repo: $Repo" -Action {
        & $binary resume --repo $Repo
    }

    Invoke-DemoStep -Label "Find query: $Query" -Action {
        & $binary find $Query --repo $Repo
    }

    Invoke-DemoStep -Label "Compare sessions: $CompareSessionA vs $CompareSessionB" -Action {
        & $binary compare $CompareSessionA $CompareSessionB
    }

    Invoke-DemoStep -Label "Pack repo: $Repo" -Action {
        & $binary pack --repo $Repo
    }
}
finally {
    Pop-Location
}

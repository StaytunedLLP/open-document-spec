# ODS skill bootstrap for Windows (PowerShell 5.1+)
# Automates ODS binary download, installation, OS service registration, and health validation.
#
# Usage:
#   .\bootstrap.ps1                 # default: install -> check . -> ensure . (if compliant) -> doctor . -> validate
#   .\bootstrap.ps1 install
#   .\bootstrap.ps1 update
#   .\bootstrap.ps1 ensure [path]   # guarantee background service for a workspace
#   .\bootstrap.ps1 status [path]
#   .\bootstrap.ps1 doctor [path]
#   .\bootstrap.ps1 check [path]    # compliance check
#
# Env:
#   ODS_PREFIX    install dir for binary (default: %LOCALAPPDATA%\Programs\ods)
#   ODS_VERSION   pin a release tag (default: latest)
#   GH_TOKEN      required for private repos ($env:GH_TOKEN = (gh auth token))

[CmdletBinding()]
param(
    [string]$Command = "default",
    [string]$Path = "."
)
$ErrorActionPreference = "Stop"

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Definition
$SrcInstallScript = Join-Path $ScriptDir "..\..\src\scripts\install.ps1"

function Write-Step { Write-Host "==> $($args -join ' ')" }
function Write-Warn { Write-Warning $($args -join ' ') }
function Write-Fatal { Write-Error $($args -join ' '); exit 1 }

function Have-Ods {
    $cmd = Get-Command ods -ErrorAction SilentlyContinue
    if ($cmd) { return $true }
    $prefix = $env:ODS_PREFIX
    if (-not $prefix) { $prefix = Join-Path $env:LOCALAPPDATA "Programs\ods" }
    return (Test-Path (Join-Path $prefix "ods.exe"))
}

function Get-OdsVersion {
    $cmd = Get-Command ods -ErrorAction SilentlyContinue
    if ($cmd) {
        return (& $cmd.Source --version 2>$null)
    }
    $prefix = $env:ODS_PREFIX
    if (-not $prefix) { $prefix = Join-Path $env:LOCALAPPDATA "Programs\ods" }
    $candidate = Join-Path $prefix "ods.exe"
    if (Test-Path $candidate) {
        return (& $candidate --version 2>$null)
    }
    return "unknown"
}

function Invoke-Install {
    if (Test-Path $SrcInstallScript) {
        & $SrcInstallScript
    } else {
        $token = $env:GH_TOKEN
        if (-not $token -and $env:GITHUB_TOKEN) { $token = $env:GITHUB_TOKEN }
        if (-not $token) {
            try { $token = (gh auth token 2>$null) } catch {}
        }
        if ($token) { $env:GH_TOKEN = $token }
        irm -Headers @{ Authorization = "Bearer $env:GH_TOKEN" } `
            https://raw.githubusercontent.com/StaytunedLLP/open-document-spec/main/src/scripts/install.ps1 | iex
    }
}

function Find-WorkspaceRoot {
    param([string]$TargetDir)
    $dir = Resolve-Path $TargetDir -ErrorAction SilentlyContinue
    if (-not $dir) { return $null }
    $current = $dir.Path
    while ($current) {
        $indexPath = Join-Path $current "index.md"
        if (Test-Path $indexPath) {
            $content = Get-Content $indexPath -Raw -ErrorAction SilentlyContinue
            if ($content -match '(?m)^ods\s*:') {
                return $current
            }
        }
        $parent = Split-Path -Parent $current
        if ($parent -eq $current) { break }
        $current = $parent
    }
    return $null
}

function Invoke-Check {
    param([string]$TargetPath)
    $root = Find-WorkspaceRoot $TargetPath
    if ($root) {
        Write-Host "compliant=true root=$root"
    } else {
        Write-Host "compliant=false root="
        Write-Host "hint: not an ODS workspace (no index.md with `ods:`). Run: ods init $TargetPath"
    }
}

function Invoke-Ensure {
    param([string]$TargetPath)
    if (-not (Have-Ods)) { Write-Fatal "ods not installed. Run: .\bootstrap.ps1 install" }
    $root = Find-WorkspaceRoot $TargetPath
    if (-not $root) {
        Write-Warn "not an ODS workspace (no index.md with `ods:`). Run: ods init $TargetPath"
        return
    }
    Write-Step "starting ods service for $TargetPath"
    ods setup $TargetPath
}

switch ($Command.ToLower()) {
    "install" { Invoke-Install }
    "update"  {
        if (Have-Ods) { ods update } else { Invoke-Install }
    }
    "check"   { Invoke-Check $Path }
    "ensure"  { Invoke-Ensure $Path }
    "status"  {
        if (-not (Have-Ods)) { Write-Fatal "ods not installed" }
        ods --version
        ods start --status $Path
    }
    "doctor"  {
        if (-not (Have-Ods)) { Write-Fatal "ods not installed" }
        ods doctor $Path
    }
    "default" {
        Invoke-Install
        Invoke-Check $Path
        $root = Find-WorkspaceRoot $Path
        if ($root) {
            Invoke-Ensure $Path
            ods doctor $Path
        } else {
            Write-Warn "no ODS workspace at '$Path'; run 'ods init .' then '.\bootstrap.ps1 ensure .'"
        }
        $ver = Get-OdsVersion
        Write-Step "ODS is installed and running now in your machine!"
        Write-Step "Version: $ver"
    }
    default {
        Write-Fatal "unknown command '$Command'. Use: install, update, check, ensure, status, doctor"
    }
}

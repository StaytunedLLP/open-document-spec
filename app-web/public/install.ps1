# OpenDocify (odc) installer for Windows (PowerShell 5.1+)
#
# Supported platforms (auto-detected):
#   Windows x64   — windows-x86_64
#   Windows ARM64 — windows-arm64
#
# This repository is private. Set a GitHub token before running:
#   $env:GH_TOKEN = (gh auth token)   # or GITHUB_TOKEN with repo scope
#   irm -Headers @{ Authorization = "Bearer $env:GH_TOKEN" } `
#     https://raw.githubusercontent.com/StaytunedLLP/open-document-spec/main/src/scripts/install.ps1 | iex
#
# Note: the Authorization header on irm only fetches this script. The script
# itself also needs GH_TOKEN/GITHUB_TOKEN in the environment to download assets.
#
# Options via environment variables:
#   ODS_VERSION / ODC_VERSION — pin a release tag, e.g. "v0.1.0"  (default: latest)
#   ODS_PREFIX / ODC_PREFIX   — install dir (default: %LOCALAPPDATA%\Programs\odc)
#   ODS_NO_VERIFY — set to "1" to skip SHA256 checksum verification
#   GH_TOKEN / GITHUB_TOKEN — required while the repo is private
#
[CmdletBinding()]
param()
$ErrorActionPreference = "Stop"

$Repo = "StaytunedLLP/open-document-spec"
$Api  = "https://api.github.com/repos/$Repo"

function Write-Step { Write-Host "==> $($args -join ' ')" }
function Write-Warn { Write-Warning $($args -join ' ') }

function Get-GitHubToken {
    if ($env:GH_TOKEN) { return $env:GH_TOKEN }
    if ($env:GITHUB_TOKEN) { return $env:GITHUB_TOKEN }
    return $null
}

function Get-AuthHeaders {
    param([string]$Accept = "application/vnd.github+json")
    $token = Get-GitHubToken
    $h = @{
        "Accept"     = $Accept
        "User-Agent" = "odc-install"
    }
    if ($token) {
        $h["Authorization"] = "Bearer $token"
    }
    return $h
}

function Normalize-OdsVersion {
    param([string]$Version)
    if (-not $Version) { return "0.0.0" }
    return ($Version.Trim() -replace '^[vV]', '')
}

function Compare-OdsVersion {
    param(
        [Parameter(Mandatory)] [string] $Left,
        [Parameter(Mandatory)] [string] $Right
    )
    $l = (Normalize-OdsVersion $Left).Split('.')
    $r = (Normalize-OdsVersion $Right).Split('.')
    for ($i = 0; $i -lt 3; $i++) {
        $li = if ($i -lt $l.Length) { [int]($l[$i] -replace '[^0-9].*$', '') } else { 0 }
        $ri = if ($i -lt $r.Length) { [int]($r[$i] -replace '[^0-9].*$', '') } else { 0 }
        if ($li -gt $ri) { return 1 }
        if ($li -lt $ri) { return -1 }
    }
    return 0
}

function Get-InstalledOdsVersion {
    $cmd = Get-Command ods -ErrorAction SilentlyContinue
    if ($cmd) {
        $out = & $cmd.Source --version 2>$null
        if ($out -match '(?:odc|ods)\s+([^\s]+)') { return $Matches[1] }
    }
    $prefix = $env:ODS_PREFIX
    if (-not $prefix) { $prefix = Join-Path $env:LOCALAPPDATA "Programs\odc" }
    $candidate = Join-Path $prefix "ods.exe"
    if (Test-Path $candidate) {
        $out = & $candidate --version 2>$null
        if ($out -match '(?:odc|ods)\s+([^\s]+)') { return $Matches[1] }
    }
    return $null
}

function Show-PrivateRepoHint {
    Write-Host @"
This repository is private. Unauthenticated downloads return HTTP 404.

  `$env:GH_TOKEN = (gh auth token)   # or a PAT with repo scope
  irm -Headers @{ Authorization = "Bearer `$env:GH_TOKEN" } ``
    https://raw.githubusercontent.com/StaytunedLLP/open-document-spec/main/src/scripts/install.ps1 | iex

GH_TOKEN must be set in the environment that runs this script (not only on irm).
"@ -ForegroundColor Yellow
}

function Get-Release {
    param([string]$Tag)
    if ($Tag) {
        return Invoke-RestMethod "$Api/releases/tags/$Tag" -Headers (Get-AuthHeaders)
    }
    return Invoke-RestMethod "$Api/releases/latest" -Headers (Get-AuthHeaders)
}

function Download-ReleaseAsset {
    param(
        [Parameter(Mandatory)] $Release,
        [Parameter(Mandatory)] [string] $Name,
        [Parameter(Mandatory)] [string] $OutFile
    )
    $asset = $Release.assets | Where-Object { $_.name -eq $Name } | Select-Object -First 1
    if (-not $asset) {
        throw "Asset '$Name' not found on release $($Release.tag_name)"
    }
    # Private repos require the API asset endpoint + Accept: application/octet-stream
    Invoke-WebRequest -Uri "$Api/releases/assets/$($asset.id)" `
        -OutFile $OutFile `
        -Headers (Get-AuthHeaders -Accept "application/octet-stream") `
        -UseBasicParsing
}

# ── Architecture detection → short asset id ───────────────────────────────────
$ProcArch = [System.Runtime.InteropServices.RuntimeInformation]::ProcessArchitecture
$Asset = switch ($ProcArch.ToString()) {
    "X64"   { "windows-x86_64" }
    "Arm64" { "windows-arm64" }
    default { throw "Unsupported architecture: $ProcArch. Only x64 and ARM64 are supported." }
}

$Token = Get-GitHubToken
if (-not $Token) {
    Write-Warn "GH_TOKEN / GITHUB_TOKEN not set — downloads will fail if the repo is private"
}

# ── Version resolution ────────────────────────────────────────────────────────
$Version = $env:ODS_VERSION
try {
    if ($Version) {
        Write-Step "Fetching release $Version..."
        $Release = Get-Release -Tag $Version
    } else {
        Write-Step "Resolving latest ODS release..."
        $Release = Get-Release
        $Version = $Release.tag_name
    }
} catch {
    if (-not $Token) { Show-PrivateRepoHint }
    throw "Could not reach GitHub API. Check network and token. Error: $_"
}
if (-not $Version) { throw "Could not resolve latest release tag." }
Write-Step "Installing ODS $Version for $Asset"

$InstalledVersion = Get-InstalledOdsVersion
if ($InstalledVersion -and ((Compare-OdsVersion $InstalledVersion $Version) -ge 0)) {
    Write-Step "ods $InstalledVersion is up to date (latest $(Normalize-OdsVersion $Version))"
    $cmd = Get-Command ods -ErrorAction SilentlyContinue
    if ($cmd) {
        & $cmd.Source --version
    } else {
        $prefix = $env:ODS_PREFIX
        if (-not $prefix) { $prefix = Join-Path $env:LOCALAPPDATA "Programs\odc" }
        & (Join-Path $prefix "ods.exe") --version
    }
    return
}

# ── Filenames ─────────────────────────────────────────────────────────────────
$Filename = "ods-$Version-$Asset.zip"

# ── Temp workspace ────────────────────────────────────────────────────────────
$TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) "ods-install-$([System.Guid]::NewGuid().ToString('N'))"
New-Item -ItemType Directory -Path $TmpDir -Force | Out-Null
try {

# ── Download ──────────────────────────────────────────────────────────────────
Write-Step "Downloading $Filename..."
try {
    Download-ReleaseAsset -Release $Release -Name $Filename -OutFile "$TmpDir\$Filename"
} catch {
    if (-not $Token) { Show-PrivateRepoHint }
    throw "Download failed for $Filename on $Version`nhttps://github.com/$Repo/releases`nError: $_"
}

# ── Checksum verification ─────────────────────────────────────────────────────
if ($env:ODS_NO_VERIFY -ne "1") {
    Write-Step "Verifying checksum..."
    try {
        Download-ReleaseAsset -Release $Release -Name "SHA256SUMS" -OutFile "$TmpDir\SHA256SUMS"
    } catch {
        if (-not $Token) { Show-PrivateRepoHint }
        throw "Could not download SHA256SUMS for $Version"
    }
    $SumsContent = Get-Content "$TmpDir\SHA256SUMS"
    $ExpectedLine = $SumsContent | Where-Object { $_ -match " $([regex]::Escape($Filename))$" } | Select-Object -First 1
    if (-not $ExpectedLine) {
        throw "No checksum entry found for '$Filename' in SHA256SUMS."
    }
    $Expected = ($ExpectedLine -split '\s+')[0].ToLowerInvariant()
    $Actual   = (Get-FileHash "$TmpDir\$Filename" -Algorithm SHA256).Hash.ToLowerInvariant()
    if ($Expected -ne $Actual) {
        throw "Checksum mismatch!`n  Expected: $Expected`n  Got:      $Actual`nThe downloaded file may be corrupt or tampered with."
    }
    Write-Host "    Checksum OK"
}

# ── Extract ───────────────────────────────────────────────────────────────────
Write-Step "Extracting..."
Expand-Archive -Path "$TmpDir\$Filename" -DestinationPath $TmpDir -Force
$Extracted = "$TmpDir\odc-$Version-$Asset"
if (-not (Test-Path "$Extracted")) { $Extracted = "$TmpDir\ods-$Version-$Asset" }
$BinSrc = $null
if (Test-Path "$Extracted\odc.exe") { $BinSrc = "$Extracted\odc.exe" }
elseif (Test-Path "$Extracted\ods.exe") { $BinSrc = "$Extracted\ods.exe" }
else {
    $found = Get-ChildItem -Path $TmpDir -Recurse -Include "odc.exe","ods.exe" -ErrorAction SilentlyContinue | Select-Object -First 1
    if ($found) { $BinSrc = $found.FullName }
}
if (-not $BinSrc) { throw "odc.exe/ods.exe not found in archive" }

# ── Install ───────────────────────────────────────────────────────────────────
$Prefix = $env:ODC_PREFIX
if (-not $Prefix) { $Prefix = $env:ODS_PREFIX }
if (-not $Prefix) { $Prefix = Join-Path $env:LOCALAPPDATA "Programs\odc" }
New-Item -ItemType Directory -Force -Path $Prefix | Out-Null
Copy-Item $BinSrc (Join-Path $Prefix "odc.exe") -Force
Copy-Item $BinSrc (Join-Path $Prefix "ods.exe") -Force

Write-Host ""
Write-Host "==> Installed successfully:"
Write-Host "    $Prefix\odc.exe  (primary)"
Write-Host "    $Prefix\ods.exe  (legacy argv0)"

# ── PATH update ───────────────────────────────────────────────────────────────
$UserPath = [Environment]::GetEnvironmentVariable("PATH", "User")
if ($UserPath -notlike "*$Prefix*") {
    [Environment]::SetEnvironmentVariable("PATH", "$UserPath;$Prefix", "User")
    Write-Host ""
    Write-Host "  NOTE: '$Prefix' has been added to your user PATH."
    Write-Host "  Please restart your terminal (or run: `$env:PATH += ';$Prefix'`)"
} else {
    Write-Host "  '$Prefix' is already in your PATH."
}

} finally {
    Remove-Item -Recurse -Force $TmpDir -ErrorAction SilentlyContinue
}

# ── Next steps ────────────────────────────────────────────────────────────────
Write-Host ""
Write-Host "  Verify installation (in a new terminal):"
Write-Host "    odc --version"
Write-Host ""
Write-Host "  Get started:"
Write-Host "    odc ods init .              # make project ODS-compliant (creates root index.md)"
Write-Host "    odc setup               # set up machine background service & check workspace health"
Write-Host "    odc ods lint"
Write-Host "    odc ods export              # optional graph.md for AI"
Write-Host ""
Write-Host "  Keep tools current:"
Write-Host "    `$env:GH_TOKEN = (gh auth token)   # needed for private releases"
Write-Host "    odc update              # update binary & restart background service"
Write-Host "    (auto-check ~daily; disable with ODS_AUTO_UPDATE=0)"
Write-Host ""
Write-Host "  Guide: https://github.com/$Repo/blob/main/README.md"
Write-Host "  Changelog: https://github.com/$Repo/blob/main/CHANGELOG.md"

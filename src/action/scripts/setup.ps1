# GitHub Action setup & runner for OpenDocify CLI (odc) — Windows (PowerShell)
$ErrorActionPreference = 'Stop'

$Repo = if ($env:GITHUB_REPOSITORY) { $env:GITHUB_REPOSITORY } else { "StaytunedLLP/open-document-spec" }
$Api = "https://api.github.com/repos/$Repo"

$InputVersion = if ($env:INPUT_VERSION) { $env:INPUT_VERSION } else { "latest" }
$InputCommand = if ($env:INPUT_COMMAND) { $env:INPUT_COMMAND } else { "lint" }
$InputPath = if ($env:INPUT_PATH) { $env:INPUT_PATH } else { "." }
$InputLevel = if ($env:INPUT_LEVEL) { $env:INPUT_LEVEL } else { "3" }
$InputFormat = if ($env:INPUT_FORMAT) { $env:INPUT_FORMAT } else { "text" }
$InputToken = if ($env:INPUT_TOKEN) { $env:INPUT_TOKEN } elseif ($env:GH_TOKEN) { $env:GH_TOKEN } else { $env:GITHUB_TOKEN }
$InputAnnotate = if ($env:INPUT_ANNOTATE) { $env:INPUT_ANNOTATE } else { "true" }
$InputExtraArgs = if ($env:INPUT_EXTRA_ARGS) { $env:INPUT_EXTRA_ARGS } else { "" }

function Write-Info($msg) { Write-Host "==> $msg" }
function Write-Fatal($msg) { Write-Error "::error::$msg"; exit 1 }

$Arch = "x86_64"
if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { $Arch = "arm64" }
$Asset = "windows-$Arch"

function Get-Headers {
    $Headers = @{
        "User-Agent" = "odc-github-action-powershell"
    }
    if ($InputToken) {
        $Headers["Authorization"] = "Bearer $InputToken"
    }
    return $Headers
}

$Version = $InputVersion
if ($Version -eq "latest" -or [string]::IsNullOrWhiteSpace($Version)) {
    Write-Info "Resolving latest OpenDocify release..."
    try {
        $Release = Invoke-RestMethod -Uri "$Api/releases/latest" -Headers (Get-Headers) -UseBasicParsing
        $Version = $Release.tag_name
    } catch {
        Write-Info "No published GitHub release found yet via API."
    }
}

if ([string]::IsNullOrWhiteSpace($Version) -or $Version -eq "latest") {
    $Tag = "latest"
    $CleanVersion = "latest"
} else {
    $Tag = $Version
    if (-not $Tag.StartsWith("v")) { $Tag = "v$Version" }
    $CleanVersion = $Tag.TrimStart("v")
}

Write-Info "Target CLI version: $Tag ($Asset)"

$InstallDir = Join-Path $env:USERPROFILE ".local\bin"
if (-not (Test-Path $InstallDir)) { New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null }
$OdcBin = Join-Path $InstallDir "odc.exe"
$OdsBin = Join-Path $InstallDir "ods.exe"

$Installed = $false
if (Test-Path $OdcBin) {
    try {
        $CurVer = (& $OdcBin --version 2>$null | Select-Object -First 1)
        if ($CurVer -match '(?:odc|ods)\s+(\S+)') { $CurVer = $Matches[1] }
        if ($CurVer -eq $CleanVersion) {
            $Installed = $true
            Write-Info "CLI $CleanVersion is already installed."
        }
    } catch {}
}

if (-not $Installed) {
    $TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

    try {
        $Downloaded = $false
        foreach ($Prefix in @("odc", "ods")) {
            $Filename = "$Prefix-$Tag-$Asset.zip"
            Write-Info "Trying $Filename..."
            try {
                $Release = Invoke-RestMethod -Uri "$Api/releases/tags/$Tag" -Headers (Get-Headers) -UseBasicParsing
                $AssetObj = $Release.assets | Where-Object { $_.name -eq $Filename }
                if ($AssetObj) {
                    $AssetUrl = "$Api/releases/assets/$($AssetObj.id)"
                    $ZipPath = Join-Path $TmpDir $Filename
                    $DownloadHeaders = Get-Headers
                    $DownloadHeaders["Accept"] = "application/octet-stream"
                    Invoke-WebRequest -Uri $AssetUrl -Headers $DownloadHeaders -OutFile $ZipPath -UseBasicParsing
                    Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force
                    $ExtractedBin = Get-ChildItem -Path $TmpDir -Recurse -Include "odc.exe","ods.exe" | Select-Object -First 1
                    if ($ExtractedBin) {
                        Copy-Item -Path $ExtractedBin.FullName -Destination $OdcBin -Force
                        Copy-Item -Path $ExtractedBin.FullName -Destination $OdsBin -Force
                        Write-Info "Installed $OdcBin (+ ods.exe)"
                        $Downloaded = $true
                        break
                    }
                }
            } catch {}
        }

        if (-not $Downloaded) {
            if (Get-Command cargo -ErrorAction SilentlyContinue) {
                Write-Info "Release asset not available — compiling local Cargo fallback..."
                cargo build --release -p odc --bin odc --bin ods
                $ExtractedBin = Get-ChildItem -Path "." -Recurse -Filter "odc.exe" | Where-Object { $_.FullName -like "*release*" } | Select-Object -First 1
                if (-not $ExtractedBin) {
                    $ExtractedBin = Get-ChildItem -Path "." -Recurse -Filter "ods.exe" | Where-Object { $_.FullName -like "*release*" } | Select-Object -First 1
                }
                if ($ExtractedBin) {
                    Copy-Item -Path $ExtractedBin.FullName -Destination $OdcBin -Force
                    Copy-Item -Path $ExtractedBin.FullName -Destination $OdsBin -Force
                    Write-Info "Built and installed $OdcBin"
                } else {
                    Write-Fatal "Cargo build completed but odc.exe/ods.exe not found."
                }
            } else {
                Write-Fatal "Failed to download release asset and Cargo is not installed."
            }
        }
    } finally {
        Remove-Item -Path $TmpDir -Recurse -Force -ErrorAction SilentlyContinue
    }
}

if ($env:GITHUB_PATH) {
    Add-Content -Path $env:GITHUB_PATH -Value $InstallDir
}

if ($env:GITHUB_OUTPUT) {
    Add-Content -Path $env:GITHUB_OUTPUT -Value "ods-version=$CleanVersion"
    Add-Content -Path $env:GITHUB_OUTPUT -Value "ods-path=$OdcBin"
    Add-Content -Path $env:GITHUB_OUTPUT -Value "odc-version=$CleanVersion"
    Add-Content -Path $env:GITHUB_OUTPUT -Value "odc-path=$OdcBin"
}

if ($InputAnnotate -eq "true" -and $env:GITHUB_ACTION_PATH) {
    $MatcherPath = Join-Path $env:GITHUB_ACTION_PATH "src\action\problem-matcher.json"
    if (-not (Test-Path $MatcherPath)) {
        $MatcherPath = Join-Path $env:GITHUB_ACTION_PATH "problem-matcher.json"
    }
    if (Test-Path $MatcherPath) {
        Write-Host "::add-matcher::$MatcherPath"
        Write-Info "Registered problem matcher for inline annotations."
    }
}

if ([string]::IsNullOrWhiteSpace($InputCommand) -or $InputCommand -eq "none" -or $InputCommand -eq "setup") {
    Write-Info "Setup completed successfully (setup-only mode)."
    exit 0
}

$env:ODS_AUTO_UPDATE = "0"
$env:ODC_AUTO_UPDATE = "0"

Write-Info "Executing: odc ods $InputCommand $InputPath (namespaced ODS)"

$CmdArgs = @()
switch ($InputCommand) {
    "lint" { $CmdArgs = @("ods", "lint", $InputPath, "--level", $InputLevel, "--format", $InputFormat) }
    "index-check" { $CmdArgs = @("ods", "index", $InputPath, "--check") }
    "index_check" { $CmdArgs = @("ods", "index", $InputPath, "--check") }
    "doctor" { $CmdArgs = @("ods", "doctor", $InputPath) }
    "fmt-check" { $CmdArgs = @("ods", "fmt", $InputPath) }
    "bench" { $CmdArgs = @("ods", "bench", "stats", $InputPath) }
    "export" { $CmdArgs = @("ods", "export", $InputPath) }
    "okf-lint" { $CmdArgs = @("okf", "lint", $InputPath) }
    default { $CmdArgs = $InputCommand.Split(' ') + @($InputPath) }
}

if (-not [string]::IsNullOrWhiteSpace($InputExtraArgs)) {
    $CmdArgs += $InputExtraArgs.Split(' ')
}

& $OdcBin $CmdArgs
exit $LASTEXITCODE

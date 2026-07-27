# GitHub Action setup & runner for ODS (Open Document Specs) — Windows (PowerShell)
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

# Platform detection
$Arch = if ([IntPtr]::Size -eq 8) { "x86_64" } else { "x86_64" }
if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { $Arch = "arm64" }
$Asset = "windows-$Arch"

# Header helper
function Get-Headers {
    $Headers = @{
        "User-Agent" = "ods-github-action-powershell"
    }
    if ($InputToken) {
        $Headers["Authorization"] = "Bearer $InputToken"
    }
    return $Headers
}

# Resolve version
$Version = $InputVersion
if ($Version -eq "latest" -or [string]::IsNullOrWhiteSpace($Version)) {
    Write-Info "Resolving latest ODS release..."
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

Write-Info "Target ODS version: $Tag ($Asset)"

$InstallDir = Join-Path $env:USERPROFILE ".local\bin"
if (-not (Test-Path $InstallDir)) { New-Item -ItemType Directory -Force -Path $InstallDir | Out-Null }
$OdsBin = Join-Path $InstallDir "ods.exe"

$Installed = $false
if (Test-Path $OdsBin) {
    try {
        $CurVer = (& $OdsBin --version 2>$null | Select-Object -First 1) -replace '^ods\s+', ''
        if ($CurVer -eq $CleanVersion) {
            $Installed = $true
            Write-Info "ODS $CleanVersion is already installed."
        }
    } catch {}
}

if (-not $Installed) {
    $Filename = "ods-$Tag-$Asset.zip"
    $TmpDir = Join-Path ([System.IO.Path]::GetTempPath()) ([System.Guid]::NewGuid().ToString())
    New-Item -ItemType Directory -Force -Path $TmpDir | Out-Null

    try {
        Write-Info "Fetching release information for asset $Filename..."
        $Downloaded = $false
        try {
            $Release = Invoke-RestMethod -Uri "$Api/releases/tags/$Tag" -Headers (Get-Headers) -UseBasicParsing
            $AssetObj = $Release.assets | Where-Object { $_.name -eq $Filename }
            if ($AssetObj) {
                $AssetUrl = "$Api/releases/assets/$($AssetObj.id)"
                $ZipPath = Join-Path $TmpDir $Filename

                Write-Info "Downloading $Filename..."
                $DownloadHeaders = Get-Headers
                $DownloadHeaders["Accept"] = "application/octet-stream"
                Invoke-WebRequest -Uri $AssetUrl -Headers $DownloadHeaders -OutFile $ZipPath -UseBasicParsing

                Write-Info "Extracting $Filename..."
                Expand-Archive -Path $ZipPath -DestinationPath $TmpDir -Force

                $ExtractedBin = Get-ChildItem -Path $TmpDir -Recurse -Filter "ods.exe" | Select-Object -First 1
                if ($ExtractedBin) {
                    Copy-Item -Path $ExtractedBin.FullName -Destination $OdsBin -Force
                    Write-Info "ODS installed successfully to $OdsBin"
                    $Downloaded = $true
                }
            }
        } catch {}

        if (-not $Downloaded) {
            if (Get-Command cargo -ErrorAction SilentlyContinue) {
                Write-Info "Release asset $Filename not available — compiling local Cargo fallback binary..."
                cargo build --release --bin ods
                $ExtractedBin = Get-ChildItem -Path "." -Recurse -Filter "ods.exe" | Where-Object { $_.FullName -like "*release*" } | Select-Object -First 1
                if ($ExtractedBin) {
                    Copy-Item -Path $ExtractedBin.FullName -Destination $OdsBin -Force
                    Write-Info "ODS built and installed successfully to $OdsBin"
                } else {
                    Write-Fatal "Cargo build completed but ods.exe binary not found."
                }
            } else {
                Write-Fatal "Failed to download release asset $Filename and Cargo is not installed."
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
    Add-Content -Path $env:GITHUB_OUTPUT -Value "ods-path=$OdsBin"
}

if ($InputAnnotate -eq "true" -and $env:GITHUB_ACTION_PATH) {
    $MatcherPath = Join-Path $env:GITHUB_ACTION_PATH "src\action\problem-matcher.json"
    if (-not (Test-Path $MatcherPath)) {
        $MatcherPath = Join-Path $env:GITHUB_ACTION_PATH "problem-matcher.json"
    }
    if (Test-Path $MatcherPath) {
        Write-Host "::add-matcher::$MatcherPath"
        Write-Info "Registered ODS problem matcher for inline annotations."
    }
}

if ([string]::IsNullOrWhiteSpace($InputCommand) -or $InputCommand -eq "none" -or $InputCommand -eq "setup") {
    Write-Info "ODS setup completed successfully (setup-only mode)."
    exit 0
}

Write-Info "Executing ODS command: ods $InputCommand $InputPath"

$CmdArgs = @()
switch ($InputCommand) {
    "lint" { $CmdArgs = @("lint", $InputPath, "--level", $InputLevel, "--format", $InputFormat) }
    "index-check" { $CmdArgs = @("index", $InputPath, "--check") }
    "index_check" { $CmdArgs = @("index", $InputPath, "--check") }
    "doctor" { $CmdArgs = @("doctor", $InputPath) }
    "fmt-check" { $CmdArgs = @("fmt", $InputPath) }
    "bench" { $CmdArgs = @("bench", "stats", $InputPath) }
    "export" { $CmdArgs = @("export", $InputPath) }
    default { $CmdArgs = $InputCommand.Split(' ') + @($InputPath) }
}

if (-not [string]::IsNullOrWhiteSpace($InputExtraArgs)) {
    $CmdArgs += $InputExtraArgs.Split(' ')
}

& $OdsBin $CmdArgs
exit $LASTEXITCODE

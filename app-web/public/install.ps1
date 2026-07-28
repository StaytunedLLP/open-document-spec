# Open Document Spec (ODS) — Universal Windows PowerShell Installer
# Supported: Windows 10/11, Windows Server, PowerShell 5.1+, PowerShell 7+ (x64 / ARM64)
# Site: https://opendocify.com / https://prod-ods-260726.web.app

$ErrorActionPreference = "Stop"

Write-Host "--------------------------------------------------------" -ForegroundColor Cyan
Write-Host "  Installing Open Document Spec CLI (ods)..." -ForegroundColor Cyan
Write-Host "--------------------------------------------------------" -ForegroundColor Cyan

# Detect Architecture
$arch = if ([IntPtr]::Size -eq 8) { "x86_64" } else { "x86" }
if ($env:PROCESSOR_ARCHITECTURE -eq "ARM64") { $arch = "aarch64" }

Write-Host "Detected Architecture: $arch" -ForegroundColor Gray

$targetDir = "$env:USERPROFILE\.ods\bin"
if (-not (Test-Path $targetDir)) {
    New-Item -ItemType Directory -Force -Path $targetDir | Out-Null
}

$installed = $false

# Method A: Try Direct Binary Download
$downloadUrl = "https://github.com/StaytunedLLP/open-document-specs/releases/latest/download/ods-windows-$arch.zip"
$zipPath = "$env:TEMP\ods.zip"

try {
    Write-Host "Attempting binary download from GitHub Releases..." -ForegroundColor Gray
    [Net.ServicePointManager]::SecurityProtocol = [Net.SecurityProtocolType]::Tls12
    Invoke-WebRequest -Uri $downloadUrl -OutFile $zipPath -ErrorAction Stop
    Expand-Archive -Path $zipPath -DestinationPath $targetDir -Force
    Remove-Item $zipPath -ErrorAction SilentlyContinue
    $installed = $true
} catch {
    # Method B: Fallback to Rust Cargo Toolchain
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        Write-Host "Binary release unavailable. Installing via Rust Cargo..." -ForegroundColor Yellow
        cargo install ods-cli --force
        $installed = $true
        $targetDir = "$env:USERPROFILE\.cargo\bin"
    }
}

if (-not $installed) {
    Write-Host "`nCould not automatically install binary release for Windows." -ForegroundColor Red
    Write-Host "Please install Rust (https://rustup.rs) and run: cargo install ods-cli" -ForegroundColor Yellow
    exit 1
}

# Update User PATH Environment Variable
$userPath = [Environment]::GetEnvironmentVariable("Path", [EnvironmentVariableTarget]::User)
if ($userPath -notlike "*$targetDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$targetDir", [EnvironmentVariableTarget]::User)
    $env:Path += ";$targetDir"
    Write-Host "Added $targetDir to User PATH environment variable." -ForegroundColor Cyan
}

Write-Host "`n✓ Open Document Spec CLI (ods) installed successfully!" -ForegroundColor Green
Write-Host "Binary Location: $targetDir\ods.exe" -ForegroundColor Gray
Write-Host "Get Started: Run 'ods --help' or 'ods setup .'" -ForegroundColor Cyan

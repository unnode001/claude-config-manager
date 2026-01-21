# Build script for Claude Config Manager release (Windows)

param(
    [string]$Version = "local",
    [string]$OutputDir = ".\dist"
)

Write-Host "Building Claude Config Manager v$Version" -ForegroundColor Green

# Clean previous builds
Write-Host "Cleaning previous builds..." -ForegroundColor Yellow
cargo clean

# Create output directory
New-Item -ItemType Directory -Force -Path $OutputDir | Out-Null

# Build for current platform
Write-Host "Building release binary..." -ForegroundColor Yellow
cargo build --release --bin ccm

# Copy and package binary
$target = "windows-x86_64"
$source = "target\release\ccm.exe"
$dest = "$OutputDir\ccm-$target.exe"
$output = "$OutputDir\ccm-$target-$Version.zip"

Write-Host "Creating release archive..." -ForegroundColor Yellow
Copy-Item $source $dest

# Create ZIP archive
Compress-Archive -Path $dest -DestinationPath $output -Force

# Clean up temporary file
Remove-Item $dest

Write-Host ""
Write-Host "Build complete! Artifact: $output" -ForegroundColor Green

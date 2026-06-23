$ErrorActionPreference = "Stop"

. "$PSScriptRoot\ffmpeg-env.ps1"

$ProjectName = "scrcpy-mask"
$ProjectDir = Resolve-Path "$PSScriptRoot\.."
$OutputZip = Join-Path $ProjectDir "target\release\$ProjectName-$env:SCRCPY_MASK_OS.zip"
$BuildTarget = Join-Path $ProjectDir "target\release\$ProjectName.exe"
$AssetsDir = Join-Path $ProjectDir "assets"

& "$PSScriptRoot\prepare-adb.ps1"

Push-Location (Join-Path $ProjectDir "frontend")
pnpm build
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    exit $LASTEXITCODE
}
Pop-Location

Push-Location $ProjectDir
cargo build --release
if ($LASTEXITCODE -ne 0) {
    Pop-Location
    exit $LASTEXITCODE
}
Pop-Location

Write-Host "Build successful, creating zip package..."
Compress-Archive -Path @($BuildTarget, $AssetsDir) -DestinationPath $OutputZip -Force

Write-Host "Package created: $OutputZip"

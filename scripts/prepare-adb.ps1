$ErrorActionPreference = "Stop"

$ProjectDir = Resolve-Path "$PSScriptRoot\.."
$OutputDir = Join-Path $ProjectDir "assets\platform-tools"

$AdbCommand = Get-Command adb -CommandType Application -ErrorAction SilentlyContinue
if (-not $AdbCommand) {
    throw "adb not found in PATH. Install Android SDK Platform-Tools before building."
}

$AdbPath = Resolve-Path $AdbCommand.Source
$AdbDir = Split-Path -Parent $AdbPath

if (Test-Path $OutputDir) {
    Remove-Item $OutputDir -Recurse -Force
}
New-Item -ItemType Directory -Path $OutputDir | Out-Null

Copy-Item $AdbPath (Join-Path $OutputDir "adb.exe")

foreach ($File in @("AdbWinApi.dll", "AdbWinUsbApi.dll")) {
    $Source = Join-Path $AdbDir $File
    if (-not (Test-Path $Source)) {
        throw "$File not found next to adb.exe. Install official Android SDK Platform-Tools before building."
    }
    Copy-Item $Source $OutputDir
}

foreach ($File in @("NOTICE.txt", "source.properties")) {
    $Source = Join-Path $AdbDir $File
    if (Test-Path $Source) {
        Copy-Item $Source $OutputDir
    }
}

Write-Host "Bundled adb from $AdbPath"

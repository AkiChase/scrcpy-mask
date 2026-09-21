# UHID extension: read-only Android device registration check.
param([string]$Serial)
$ErrorActionPreference = 'Stop'
$adb = Join-Path $PSScriptRoot '..\assets\platform-tools\adb.exe'
if (-not (Test-Path -LiteralPath $adb)) { throw 'Bundled adb.exe is missing.' }
if (-not $Serial) {
    $devices = & $adb devices
    if ($LASTEXITCODE -ne 0) { throw 'adb devices failed.' }
    $ids = @($devices | ForEach-Object { if ($_ -match '^(\S+)\s+device$') { $Matches[1] } })
    if ($ids.Count -ne 1) { throw 'Connect exactly one authorized phone or pass -Serial.' }
    $Serial = $ids[0]
}
$inputState = & $adb -s $Serial shell dumpsys input
if ($LASTEXITCODE -ne 0) { throw 'Unable to read Android input devices.' }
$registered = $inputState | Select-String -SimpleMatch 'scrcpy-mask touch' -Context 1,6
if ($registered) {
    $registered
    Write-Host 'Device name found. This confirms registration only; coordinates and multi-touch still need testing.'
} else {
    Write-Host 'UHID touchscreen not found. Connect using UHID mode first; inspect app.log if it remains absent.'
    exit 2
}

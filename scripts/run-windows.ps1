$ErrorActionPreference = "Stop"

. "$PSScriptRoot\ffmpeg-env.ps1"

$ProjectDir = Resolve-Path "$PSScriptRoot\.."
$OldPath = $env:PATH
$env:PATH = "$ProjectDir\assets\lib\$env:SCRCPY_MASK_OS;$env:PATH"

Push-Location $ProjectDir
cargo run
$ExitCode = $LASTEXITCODE
Pop-Location

$env:PATH = $OldPath
exit $ExitCode

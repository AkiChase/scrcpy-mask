$ErrorActionPreference = "Stop"

. "$PSScriptRoot\ffmpeg-env.ps1"

$ProjectDir = Resolve-Path "$PSScriptRoot\.."
Push-Location $ProjectDir
cargo check
$ExitCode = $LASTEXITCODE
Pop-Location

exit $ExitCode

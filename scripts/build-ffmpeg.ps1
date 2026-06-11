$ProjectDir = Resolve-Path "$PSScriptRoot\.."
$Bash = if ($env:MSYS2_BASH) { $env:MSYS2_BASH } else { "C:\msys64\usr\bin\bash.exe" }

if (-not (Test-Path $Bash)) {
    $Bash = "bash"
}

$ProjectDirMsys = & $Bash -lc "cygpath -u '$ProjectDir'"
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

$env:SCRCPY_MASK_FFMPEG_TARGET = "windows-x64"
& $Bash -lc "cd '$ProjectDirMsys' && ./scripts/build-ffmpeg.sh"
exit $LASTEXITCODE

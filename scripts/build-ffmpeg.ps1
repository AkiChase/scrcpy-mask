$ErrorActionPreference = "Stop"

$ProjectDir = Resolve-Path "$PSScriptRoot\.."
$Bash = if ($env:MSYS2_BASH) { $env:MSYS2_BASH } else { "C:\msys64\usr\bin\bash.exe" }

function Import-MsvcEnvironment {
    $ProgramFilesX86 = [Environment]::GetEnvironmentVariable("ProgramFiles(x86)")
    if (-not $ProgramFilesX86) {
        return
    }

    $VsWhere = Join-Path $ProgramFilesX86 "Microsoft Visual Studio\Installer\vswhere.exe"
    if (-not (Test-Path $VsWhere)) {
        return
    }

    $InstallPath = & $VsWhere `
        -latest `
        -products "*" `
        -requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64 `
        -property installationPath
    if ($LASTEXITCODE -ne 0 -or -not $InstallPath) {
        return
    }

    $VsDevCmd = Join-Path $InstallPath "Common7\Tools\VsDevCmd.bat"
    if (-not (Test-Path $VsDevCmd)) {
        return
    }

    $EnvOutput = & cmd.exe /s /c "`"$VsDevCmd`" -arch=x64 -host_arch=x64 >nul && set"
    if ($LASTEXITCODE -ne 0) {
        return
    }

    foreach ($Line in $EnvOutput) {
        if ($Line -match "^([^=]+)=(.*)$") {
            [Environment]::SetEnvironmentVariable($Matches[1], $Matches[2], "Process")
        }
    }
}

if (-not (Test-Path $Bash)) {
    Write-Error @"
MSYS2 bash is required to run FFmpeg's build scripts on Windows.

Install MSYS2 or set MSYS2_BASH to the full path of bash.exe.
Default path: C:\msys64\usr\bin\bash.exe
Example:
  `$env:MSYS2_BASH = "C:\msys64\usr\bin\bash.exe"
"@
    exit 1
}

if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
    Import-MsvcEnvironment
}

if (-not (Get-Command cl.exe -ErrorAction SilentlyContinue)) {
    Write-Error @"
MSVC C++ Build Tools were not found.

Install Visual Studio Build Tools with the C++ build tools workload.
This provides cl.exe for FFmpeg's --toolchain=msvc build.

Then run:
  just build-ffmpeg
"@
    exit 1
}

$env:MSYS2_PATH_TYPE = "inherit"

$MsysCl = & $Bash -lc "command -v cl.exe"
if ($LASTEXITCODE -ne 0 -or -not $MsysCl) {
    Write-Error @"
MSVC was found in PowerShell, but MSYS2 bash cannot find cl.exe.

The FFmpeg MSVC toolchain runs configure through MSYS2, so MSYS2 must inherit
the Visual Studio Build Tools environment.
"@
    exit 1
}

$ProjectDirMsys = & $Bash -lc "cygpath -u '$ProjectDir'"
if ($LASTEXITCODE -ne 0) {
    exit $LASTEXITCODE
}

$env:SCRCPY_MASK_FFMPEG_TARGET = "windows-x64"
& $Bash -lc "cd '$ProjectDirMsys' && ./scripts/build-ffmpeg.sh"
exit $LASTEXITCODE

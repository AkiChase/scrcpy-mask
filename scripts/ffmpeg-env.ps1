$ProjectDir = Resolve-Path "$PSScriptRoot\.."
$FFMpegVersion = if ($env:FFMPEG_VERSION) { $env:FFMPEG_VERSION } else { "7.1.2" }
$ScrcpyMaskOS = "windows-x64"
$FFMpegDir = Join-Path $ProjectDir "ffmpeg-$FFMpegVersion\ffmpeg-$ScrcpyMaskOS"

if (-not (Test-Path $FFMpegDir)) {
    throw "FFmpeg not found at $FFMpegDir. Run .\scripts\build-ffmpeg.ps1 first."
}

$env:SCRCPY_MASK_OS = $ScrcpyMaskOS
$env:FFMPEG_DIR = $FFMpegDir
$env:PKG_CONFIG_PATH = Join-Path $FFMpegDir "lib\pkgconfig"

Write-Host "Building for Windows x64"
$ProjectName = "scrcpy-mask"
$Prefix = "ffmpeg-windows"
$OS = "windows-x64"
$FFMpeg = "ffmpeg-7.1.2"

$ScriptDir = Get-Location
$env:PKG_CONFIG_PATH = "$ScriptDir\$FFMpeg\$Prefix\lib\pkgconfig"
$env:FFMPEG_DIR = "$ScriptDir\$FFMpeg\$Prefix"

if ($args[0] -eq "run") {
    $OldPath = $env:PATH
    $env:PATH = "$ScriptDir\assets\lib\$OS;$env:PATH"
    cargo run
    $env:PATH = $OldPath
    exit 1
} elseif ($args[0] -eq "release") {
    cd .\frontend
    pnpm build
    if (-not $?) {
        Write-Host "Frontend build failed"
        exit 1
    }

    cd ..
    cargo build --release
    if (-not $?) {
        Write-Host "Project build failed"
        exit 1
    }

    Write-Host "Build successful, creating zip package..."
    $OutputZip = "$ScriptDir\target\release\$ProjectName-$OS.zip"
    $BuildTarget = "$ScriptDir\target\release\$ProjectName.exe"
    $AssetsDir = "$ScriptDir\assets"
    
    $TmpDir = "$ScriptDir\target\tmp"
    $TmpAssetsDir = "$TmpDir\assets"
    if (Test-Path $TmpDir) { Remove-Item -Recurse -Force $TmpDir }
    New-Item -ItemType Directory -Path $TmpAssetsDir | Out-Null

    Get-ChildItem -Path $AssetsDir -Exclude 'lib' | ForEach-Object {
        Copy-Item -Path $_.FullName -Destination $TmpAssetsDir -Recurse
    }

    $LibOSFolder = "$AssetsDir\lib\$OS"
    if (-not (Test-Path $LibOSFolder)) {
        Write-Error "Required folder not found: $LibOSFolder"
        exit 1
    }

    $LibOSItems = Get-ChildItem -Path $LibOSFolder
    $PathsToCompress = @($BuildTarget, $TmpAssetsDir) + $LibOSItems.FullName
    Compress-Archive -Path $PathsToCompress -DestinationPath $OutputZip -Force
    Remove-Item -Recurse -Force $TmpDir

    Write-Host "Package created: $OutputZip"
} else {
    Write-Host "Usage: .\script.ps1 {run|release}"
    exit 1
}

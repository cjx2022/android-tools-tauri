Add-Type -AssemblyName System.Drawing

$pngPath = Join-Path $PSScriptRoot "icon.png"
$icoPath = Join-Path $PSScriptRoot "icon.ico"

Write-Host "Converting: $pngPath -> $icoPath"

if (-not (Test-Path $pngPath)) {
    Write-Host "Error: PNG not found"
    exit 1
}

$bitmap = [System.Drawing.Image]::FromFile($pngPath)
$icon = [System.Drawing.Icon]::FromHandle(([System.Drawing.Bitmap]$bitmap).GetHicon())
$fs = [System.IO.File]::Create($icoPath)
$icon.Save($fs)
$fs.Close()
$bitmap.Dispose()

Write-Host "Icon converted successfully!"

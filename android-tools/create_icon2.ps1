$ErrorActionPreference = "Stop"

$iconDir = "d:\安卓工具开发\tauri_version\android-tools\icons"
$pngPath = "$iconDir\icon.png"
$icoPath = "$iconDir\icon.ico"

Add-Type -AssemblyName System.Drawing

$size = 256
$bmp = New-Object System.Drawing.Bitmap($size, $size)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.SmoothingMode = 'HighQuality'
$g.Clear([System.Drawing.Color]::FromArgb(33, 150, 243))

$font = New-Object System.Drawing.Font('Arial', 140, [System.Drawing.FontStyle]::Bold)
$brush = [System.Drawing.Brushes]::White
$sf = New-Object System.Drawing.StringFormat
$sf.Alignment = 'Center'
$sf.LineAlignment = 'Center'
$rect = New-Object System.Drawing.RectangleF(0, 0, $size, $size)
$g.DrawString('A', $font, $brush, $rect, $sf)
$g.Dispose()

$tempPng = [System.IO.Path]::GetTempFileName() + ".png"
$bmp.Save($tempPng, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()

Copy-Item $tempPng $pngPath -Force
Remove-Item $tempPng -Force

$icon = [System.Drawing.Icon]::ExtractAssociatedIcon((Get-Process -Id $PID).Path)
$fs = [System.IO.File]::Create($icoPath)
$icon.Save($fs)
$fs.Close()

Write-Host "Icons created successfully"

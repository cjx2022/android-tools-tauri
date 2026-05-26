Add-Type -AssemblyName System.Drawing

# Create 256x256 bitmap with transparent background
$bmp = New-Object System.Drawing.Bitmap(256, 256)
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.Clear([System.Drawing.Color]::FromArgb(18, 32, 44))
$g.SmoothingMode = [System.Drawing.Drawing2D.SmoothingMode]::AntiAlias

# Android green color
$androidGreen = [System.Drawing.Color]::FromArgb(0, 255, 136)
$brush = New-Object System.Drawing.SolidBrush($androidGreen)
$pen = New-Object System.Drawing.Pen($androidGreen, 4)

# Scale factor
$scale = 1.8
$offsetX = 128
$offsetY = 100

# Robot Body (rounded rectangle)
$bodyRect = New-Object System.Drawing.RectangleF($offsetX - 25*$scale, $offsetY + 10*$scale, 50*$scale, 40*$scale)
$g.FillRectangle($brush, $bodyRect)

# Robot Head (semicircle shape using path)
$headPath = New-Object System.Drawing.Drawing2D.GraphicsPath
$headRect = New-Object System.Drawing.RectangleF($offsetX - 25*$scale, $offsetY - 25*$scale, 50*$scale, 40*$scale)
$headPath.AddArc($headRect, 180, 180)
$headPath.AddLine($offsetX + 25*$scale, $offsetY - 5*$scale, $offsetX - 25*$scale, $offsetY - 5*$scale)
$g.FillPath($brush, $headPath)

# Left Antenna
$g.DrawLine($pen, $offsetX - 12*$scale, $offsetY - 15*$scale, $offsetX - 20*$scale, $offsetY - 35*$scale)

# Right Antenna
$g.DrawLine($pen, $offsetX + 12*$scale, $offsetY - 15*$scale, $offsetX + 20*$scale, $offsetY - 35*$scale)

# Eyes (black circles)
$eyeBrush = [System.Drawing.Brushes]::Black
$g.FillEllipse($eyeBrush, $offsetX - 15*$scale, $offsetY - 10*$scale, 8*$scale, 8*$scale)
$g.FillEllipse($eyeBrush, $offsetX + 7*$scale, $offsetY - 10*$scale, 8*$scale, 8*$scale)

# Left Arm
$armRect1 = New-Object System.Drawing.RectangleF($offsetX - 38*$scale, $offsetY + 10*$scale, 10*$scale, 30*$scale)
$g.FillRectangle($brush, $armRect1)

# Right Arm
$armRect2 = New-Object System.Drawing.RectangleF($offsetX + 28*$scale, $offsetY + 10*$scale, 10*$scale, 30*$scale)
$g.FillRectangle($brush, $armRect2)

# Left Leg
$legRect1 = New-Object System.Drawing.RectangleF($offsetX - 15*$scale, $offsetY + 45*$scale, 10*$scale, 25*$scale)
$g.FillRectangle($brush, $legRect1)

# Right Leg
$legRect2 = New-Object System.Drawing.RectangleF($offsetX + 5*$scale, $offsetY + 45*$scale, 10*$scale, 25*$scale)
$g.FillRectangle($brush, $legRect2)

# Dispose
$g.Dispose()
$brush.Dispose()
$pen.Dispose()

# Save as PNG
$bmp.Save('d:\安卓工具开发\tauri_version\android-tools\icons\icon.png', [System.Drawing.Imaging.ImageFormat]::Png)

# Convert to ICO using online tool or keep as PNG for Tauri
# Tauri can use PNG directly

$bmp.Dispose()
Write-Host 'Android robot icon created successfully!'

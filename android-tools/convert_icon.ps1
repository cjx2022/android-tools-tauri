Add-Type -AssemblyName System.Drawing

# Load PNG image
$pngPath = "d:\安卓工具开发\tauri_version\android-tools\icons\icon.png"
$icoPath = "d:\安卓工具开发\tauri_version\android-tools\icons\icon.ico"

# Check if PNG exists
if (-not (Test-Path $pngPath)) {
    Write-Host "PNG file not found: $pngPath"
    exit 1
}

# Load the PNG image
$bitmap = [System.Drawing.Image]::FromFile($pngPath)

# Create icon sizes
$sizes = @(16, 32, 48, 64, 128, 256)
$icons = @()

foreach ($size in $sizes) {
    $newBitmap = New-Object System.Drawing.Bitmap($bitmap, $size, $size)
    $icons += $newBitmap
}

# Create ICO file
$icoStream = [System.IO.File]::OpenWrite($icoPath)

# ICO Header
$icoStream.WriteByte(0)  # Reserved
$icoStream.WriteByte(0)
$icoStream.WriteByte(1)  # Type: Icon
$icoStream.WriteByte(0)
$icoStream.WriteByte($icons.Count)  # Count
$icoStream.WriteByte(0)

# Calculate offset for image data
$offset = 6 + ($icons.Count * 16)

# Write ICONDIRENTRY for each icon
for ($i = 0; $i -lt $icons.Count; $i++) {
    $icon = $icons[$i]
    $width = $icon.Width
    $height = $icon.Height
    
    if ($width -eq 256) { $width = 0 }
    if ($height -eq 256) { $height = 0 }
    
    $icoStream.WriteByte($width)
    $icoStream.WriteByte($height)
    $icoStream.WriteByte(0)  # Color palette
    $icoStream.WriteByte(0)  # Reserved
    $icoStream.WriteByte(1)  # Color planes
    $icoStream.WriteByte(0)
    $icoStream.WriteByte(32)  # Bits per pixel
    $icoStream.WriteByte(0)
    
    # Save icon to memory stream to get size
    $memStream = New-Object System.IO.MemoryStream
    $icon.Save($memStream, [System.Drawing.Imaging.ImageFormat]::Png)
    $size = $memStream.Length
    
    # Write size (4 bytes, little endian)
    $sizeBytes = [BitConverter]::GetBytes([int]$size)
    $icoStream.Write($sizeBytes, 0, 4)
    
    # Write offset (4 bytes, little endian)
    $offsetBytes = [BitConverter]::GetBytes([int]$offset)
    $icoStream.Write($offsetBytes, 0, 4)
    
    $offset += $size
}

# Write image data
for ($i = 0; $i -lt $icons.Count; $i++) {
    $icon = $icons[$i]
    $memStream = New-Object System.IO.MemoryStream
    $icon.Save($memStream, [System.Drawing.Imaging.ImageFormat]::Png)
    $data = $memStream.ToArray()
    $icoStream.Write($data, 0, $data.Length)
    $icon.Dispose()
    $memStream.Dispose()
}

$icoStream.Close()
$bitmap.Dispose()

Write-Host "Icon converted successfully: $icoPath"

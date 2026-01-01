Add-Type -AssemblyName System.Drawing
$img = [System.Drawing.Image]::FromFile('f:\projects\emiyashiro\assets\images\characters\hf_shirou_spritesheet_final_v2_1767279221195.png')
Write-Host "Width: $($img.Width) Height: $($img.Height)"
$img.Dispose()

Add-Type -AssemblyName System.Drawing,System.Windows.Forms
$b = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$bmp = New-Object System.Drawing.Bitmap $b.Width, $b.Height
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen($b.Left, $b.Top, 0, 0, $bmp.Size)
$bmp.Save($args[0])
$g.Dispose(); $bmp.Dispose()
Write-Output "saved $($b.Width)x$($b.Height)"

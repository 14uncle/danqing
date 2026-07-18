Add-Type -AssemblyName System.Drawing
$sig = @"
using System;
using System.Runtime.InteropServices;
public class PW {
    [DllImport("user32.dll")] public static extern bool PrintWindow(System.IntPtr h, System.IntPtr dc, uint flags);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(System.IntPtr h, out RECT r);
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
}
"@
Add-Type -TypeDefinition $sig
$h = [System.IntPtr][int64]$args[0]
$r = New-Object PW+RECT
[PW]::GetWindowRect($h, [ref]$r) | Out-Null
$w = $r.Right - $r.Left; $hh = $r.Bottom - $r.Top
$bmp = New-Object System.Drawing.Bitmap $w, $hh
$g = [System.Drawing.Graphics]::FromImage($bmp)
$dc = $g.GetHdc()
$ok = [PW]::PrintWindow($h, $dc, 2)  # PW_RENDERFULLCONTENT
$g.ReleaseHdc($dc)
$bmp.Save($args[1])
$g.Dispose(); $bmp.Dispose()
Write-Output "PrintWindow=$ok size=${w}x${hh}"

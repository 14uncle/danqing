Add-Type -AssemblyName System.Drawing,System.Windows.Forms
$sig = @"
using System;
using System.Runtime.InteropServices;
public class PW2 {
    [DllImport("user32.dll")] public static extern bool PrintWindow(System.IntPtr h, System.IntPtr dc, uint flags);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(System.IntPtr h, out RECT r);
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
}
"@
Add-Type -TypeDefinition $sig
$h = [System.IntPtr][int64]$args[0]
$cx = [int]$args[1]; $cy = [int]$args[2]
$out1 = $args[3]; $out2 = $args[4]

function Capture($path) {
    $r = New-Object PW2+RECT
    [PW2]::GetWindowRect($h, [ref]$r) | Out-Null
    $w = $r.Right - $r.Left; $hh = $r.Bottom - $r.Top
    $bmp = New-Object System.Drawing.Bitmap $w, $hh
    $g = [System.Drawing.Graphics]::FromImage($bmp)
    $dc = $g.GetHdc()
    [PW2]::PrintWindow($h, $dc, 2) | Out-Null
    $g.ReleaseHdc($dc)
    $bmp.Save($path)
    $g.Dispose(); $bmp.Dispose()
    return $r
}

# 基线:光标移到远处角落
[System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point(5, 5)
Start-Sleep -Milliseconds 250
$r = Capture $out1

# 悬停:光标移到客户区指定点(边框约 8px,标题栏约 31px)
[System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point(($r.Left + 8 + $cx), ($r.Top + 31 + $cy))
Start-Sleep -Milliseconds 250
Capture $out2
Write-Output "done"

Add-Type -AssemblyName System.Drawing,System.Windows.Forms
$sig = @"
using System;
using System.Runtime.InteropServices;
public class PW3 {
    [DllImport("user32.dll")] public static extern bool GetWindowRect(System.IntPtr h, out RECT r);
    [DllImport("user32.dll")] public static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
    public struct RECT { public int Left; public int Top; public int Right; public int Bottom; }
}
"@
Add-Type -TypeDefinition $sig
$h = [System.IntPtr][int64]$args[0]
$cx = [int]$args[1]; $cy = [int]$args[2]
$r = New-Object PW3+RECT
[PW3]::GetWindowRect($h, [ref]$r) | Out-Null
$sx = $r.Left + 8 + $cx; $sy = $r.Top + 31 + $cy
[System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point($sx, $sy)
Start-Sleep -Milliseconds 200
[PW3]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)  # LEFTDOWN
Start-Sleep -Milliseconds 120
Write-Output "clicked at screen ($sx, $sy), window rect: L=$($r.Left) T=$($r.Top) R=$($r.Right) B=$($r.Bottom)"

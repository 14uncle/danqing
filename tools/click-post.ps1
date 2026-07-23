$sig = @"
using System;
using System.Runtime.InteropServices;
public class Poster {
    [DllImport("user32.dll")] public static extern bool PostMessage(IntPtr h, uint msg, IntPtr w, IntPtr l);
    public static IntPtr LParam(int x, int y) { return (IntPtr)((y << 16) | (x & 0xFFFF)); }
}
"@
Add-Type -TypeDefinition $sig
$h = [System.IntPtr][int64]$args[0]
$cx = [int]$args[1]; $cy = [int]$args[2]
# Post synthetic mouse messages straight to the window queue.
# Immune to physical-mouse interference (no SetCursorPos / mouse_event).
# Order matters: the app tracks click position from the last WM_MOUSEMOVE.
$p = [Poster]::LParam($cx, $cy)
[Poster]::PostMessage($h, 0x0200, [IntPtr]::Zero, $p) | Out-Null   # WM_MOUSEMOVE
Start-Sleep -Milliseconds 60
[Poster]::PostMessage($h, 0x0201, [IntPtr]1, $p) | Out-Null        # WM_LBUTTONDOWN
Start-Sleep -Milliseconds 60
[Poster]::PostMessage($h, 0x0202, [IntPtr]::Zero, $p) | Out-Null   # WM_LBUTTONUP
Write-Output "posted click at client ($cx, $cy) to hwnd $h"

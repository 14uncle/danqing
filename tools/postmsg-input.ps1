$sig = @"
using System;
using System.Runtime.InteropServices;
public class PM {
    [DllImport("user32.dll")] public static extern bool PostMessage(System.IntPtr h, uint msg, UIntPtr w, IntPtr l);
}
"@
Add-Type -TypeDefinition $sig
$h = [System.IntPtr][int64]$args[0]
$x = [int]$args[1]; $y = [int]$args[2]
$action = $args[3]
$lp = ($y -shl 16) -bor ($x -band 0xFFFF)
if ($action -eq "move") {
    [PM]::PostMessage($h, 0x0200, [UIntPtr]::Zero, [IntPtr]$lp) | Out-Null  # WM_MOUSEMOVE
} elseif ($action -eq "down") {
    [PM]::PostMessage($h, 0x0200, [UIntPtr]::Zero, [IntPtr]$lp) | Out-Null
    [PM]::PostMessage($h, 0x0201, [UIntPtr]1, [IntPtr]$lp) | Out-Null      # WM_LBUTTONDOWN (MK_LBUTTON=1)
} elseif ($action -eq "up") {
    [PM]::PostMessage($h, 0x0200, [UIntPtr]::Zero, [IntPtr]$lp) | Out-Null
    [PM]::PostMessage($h, 0x0202, [UIntPtr]::Zero, [IntPtr]$lp) | Out-Null # WM_LBUTTONUP
}
Write-Output "posted $action at client ($x, $y)"

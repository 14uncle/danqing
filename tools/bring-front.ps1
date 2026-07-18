Add-Type -AssemblyName System.Drawing,System.Windows.Forms
$sig = @"
using System;
using System.Runtime.InteropServices;
public class FG {
    [DllImport("user32.dll")] public static extern System.IntPtr GetForegroundWindow();
    [DllImport("user32.dll")] public static extern bool SetForegroundWindow(System.IntPtr h);
    [DllImport("user32.dll")] public static extern bool BringWindowToTop(System.IntPtr h);
    [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a, uint b, bool attach);
    [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(System.IntPtr h, System.IntPtr pid);
    [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
}
"@
Add-Type -TypeDefinition $sig
$h = [System.IntPtr][int64]$args[0]
$fg = [FG]::GetForegroundWindow()
$tidFg = [FG]::GetWindowThreadProcessId($fg, [System.IntPtr]::Zero)
$tidMe = [FG]::GetCurrentThreadId()
[FG]::AttachThreadInput($tidMe, $tidFg, $true) | Out-Null
[FG]::BringWindowToTop($h) | Out-Null
[FG]::SetForegroundWindow($h) | Out-Null
[FG]::AttachThreadInput($tidMe, $tidFg, $false) | Out-Null
Start-Sleep -Milliseconds 400
Write-Output "foregrounded"

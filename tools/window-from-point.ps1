$sig = @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class WP {
    [DllImport("user32.dll")] public static extern System.IntPtr WindowFromPoint(System.Drawing.Point p);
    [DllImport("user32.dll")] public static extern int GetWindowText(System.IntPtr h, StringBuilder s, int n);
}
"@
Add-Type -TypeDefinition $sig -ReferencedAssemblies System.Drawing
$p = New-Object System.Drawing.Point([int]$args[0], [int]$args[1])
$h = [WP]::WindowFromPoint($p)
$sb = New-Object System.Text.StringBuilder 256
[WP]::GetWindowText($h, $sb, 256) | Out-Null
Write-Output ("hwnd=" + $h.ToInt64() + " title=[" + $sb.ToString() + "]")

$sig = @"
using System;
using System.Text;
using System.Runtime.InteropServices;
public class WinEnum {
    public delegate bool EnumProc(System.IntPtr h, System.IntPtr l);
    [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc cb, System.IntPtr l);
    [DllImport("user32.dll")] public static extern int GetWindowText(System.IntPtr h, StringBuilder s, int n);
    [DllImport("user32.dll")] public static extern bool IsWindowVisible(System.IntPtr h);
}
"@
Add-Type -TypeDefinition $sig
$script:found = @()
$cb = [WinEnum+EnumProc]{ param($h, $l)
    if ([WinEnum]::IsWindowVisible($h)) {
        $sb = New-Object System.Text.StringBuilder 256
        [WinEnum]::GetWindowText($h, $sb, 256) | Out-Null
        if ($sb.Length -gt 0) { $script:found += ("{0} [{1}]" -f $h, $sb.ToString()) }
    }
    return $true
}
[WinEnum]::EnumWindows($cb, [System.IntPtr]::Zero) | Out-Null
$script:found | Select-String -Pattern $args[0]

Add-Type -AssemblyName System.Drawing,System.Windows.Forms
$sig = @"
using System;
using System.Runtime.InteropServices;
public class Clicker {
    [DllImport("user32.dll")] static extern bool ClientToScreen(System.IntPtr h, ref POINT p);
    [DllImport("user32.dll")] static extern void mouse_event(uint flags, uint dx, uint dy, uint data, UIntPtr extra);
    struct POINT { public int X; public int Y; }
    public static void ClientOrigin(IntPtr h, out int x, out int y) {
        POINT p = new POINT { X = 0, Y = 0 };
        ClientToScreen(h, ref p);
        x = p.X; y = p.Y;
    }
    public static void Click() {
        mouse_event(0x0002, 0, 0, 0, UIntPtr.Zero);  // LEFTDOWN
        System.Threading.Thread.Sleep(120);
        mouse_event(0x0004, 0, 0, 0, UIntPtr.Zero);  // LEFTUP
    }
}
"@
Add-Type -TypeDefinition $sig -ReferencedAssemblies System.Drawing,System.Windows.Forms
$h = [System.IntPtr][int64]$args[0]
$cx = [int]$args[1]; $cy = [int]$args[2]
# Self-drawn borderless window: never guess fixed frame offsets like +8/+31.
# Use ClientToScreen for the true client-area origin in screen coordinates.
$ox = 0
$oy = 0
[Clicker]::ClientOrigin($h, [ref]$ox, [ref]$oy)
$sx = $ox + $cx
$sy = $oy + $cy
[System.Windows.Forms.Cursor]::Position = New-Object System.Drawing.Point($sx, $sy)
Start-Sleep -Milliseconds 200
[Clicker]::Click()
Start-Sleep -Milliseconds 120
Write-Output "clicked client ($cx, $cy) -> screen ($sx, $sy), client origin: X=$ox Y=$oy"

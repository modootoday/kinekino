# M6-5a: the player window on a Windows desktop session. Opens a clip, then
#  1. resizes the window from outside (SetWindowPos) and times until the right-anchored
#     caption corner is drawn at the new edge (resize-to-frame), and
#  2. drags the right edge with synthetic mouse input (the player hands the press to the
#     OS through BeginResize) and checks the window followed.
# Screenshots and results.json go to the output directory.
# Usage: windows-window-smoke.ps1 -Bin <kinekino.exe> -Clip <clip.mp4> -Out <dir>
param([string]$Bin, [string]$Clip, [string]$Out)
$ErrorActionPreference = 'Stop'
New-Item -ItemType Directory -Force -Path $Out | Out-Null
Add-Type -AssemblyName System.Drawing
Add-Type @"
using System; using System.Runtime.InteropServices; using System.Drawing;
public static class W {
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr h, out RECT r);
  [DllImport("user32.dll")] public static extern bool ClientToScreen(IntPtr h, ref POINT p);
  [StructLayout(LayoutKind.Sequential)] public struct POINT { public int X, Y; }
  [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr h, IntPtr a, int x, int y, int w, int hh, uint f);
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, int dx, int dy, uint d, UIntPtr e);
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern bool IsWindow(IntPtr h);
  public delegate bool EnumProc(IntPtr h, IntPtr l);
  [DllImport("user32.dll")] public static extern bool EnumWindows(EnumProc f, IntPtr l);
  [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h, out uint pid);
  [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr h);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetClassName(IntPtr h, System.Text.StringBuilder s, int n);
  // The player's own top-level window: visible, owned by pid, not a console window.
  public static IntPtr PlayerWindow(uint pid) {
    IntPtr found = IntPtr.Zero;
    EnumWindows((h, l) => {
      uint p; GetWindowThreadProcessId(h, out p);
      if (p != pid || !IsWindowVisible(h)) return true;
      var c = new System.Text.StringBuilder(256); GetClassName(h, c, 256);
      if (c.ToString() == "ConsoleWindowClass") return true;
      found = h; return false;
    }, IntPtr.Zero);
    return found;
  }
  public static Bitmap Grab(int x, int y, int w, int h) {
    var b = new Bitmap(w, h); using (var g = Graphics.FromImage(b)) g.CopyFromScreen(x, y, 0, 0, new Size(w, h)); return b;
  }
  public static double Diff(Bitmap a, Bitmap b) {
    double s = 0; int n = 0;
    for (int y = 0; y < a.Height; y += 2) for (int x = 0; x < a.Width; x += 2) {
      Color p = a.GetPixel(x, y), q = b.GetPixel(x, y);
      s += Math.Abs(p.R - q.R) + Math.Abs(p.G - q.G) + Math.Abs(p.B - q.B); n++;
    }
    return s / (3.0 * n);
  }
}
"@ -ReferencedAssemblies System.Drawing
[W]::SetProcessDPIAware() | Out-Null
$result = [ordered]@{ ok = $false; errors = @() }
function Rect($h) { $r = New-Object W+RECT; [W]::GetWindowRect($h, [ref]$r) | Out-Null; return $r }
function Client($h) {
  $c = New-Object W+RECT; [W]::GetClientRect($h, [ref]$c) | Out-Null
  $p = New-Object W+POINT; [W]::ClientToScreen($h, [ref]$p) | Out-Null
  return @{ x = $p.X; y = $p.Y; w = $c.R; h = $c.B }
}
function Shot($name) {
  $s = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
  $b = [W]::Grab($s.X, $s.Y, $s.Width, $s.Height); $b.Save((Join-Path $Out $name)); $b.Dispose()
}
Add-Type -AssemblyName System.Windows.Forms
$screen = [System.Windows.Forms.Screen]::PrimaryScreen.Bounds
$result.screen = "$($screen.Width)x$($screen.Height)"

$env:KINEKINO_TRACE_RESIZE = '1'
$p = Start-Process -FilePath $Bin -ArgumentList @('watch', $Clip) -PassThru -NoNewWindow `
  -RedirectStandardOutput (Join-Path $Out 'player.stdout') -RedirectStandardError (Join-Path $Out 'player.stderr')
$sw = [Diagnostics.Stopwatch]::StartNew()
$h = [IntPtr]::Zero
while ($sw.Elapsed.TotalSeconds -lt 60 -and $h -eq [IntPtr]::Zero) {
  Start-Sleep -Milliseconds 100; $p.Refresh()
  if ($p.HasExited) { break }
  $h = [W]::PlayerWindow([uint32]$p.Id)
}
if ($h -eq [IntPtr]::Zero) {
  $result.errors += "player window never appeared (exited: $($p.HasExited))"
  $result | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $Out 'results.json'); Shot 'no-window.png'; exit 1
}
$cls = New-Object System.Text.StringBuilder 256; [W]::GetClassName($h, $cls, 256) | Out-Null
$result.window = @{ class = $cls.ToString(); appeared_ms = [int]$sw.ElapsedMilliseconds }
Start-Sleep -Seconds 3
[W]::SetForegroundWindow($h) | Out-Null
[W]::SetCursorPos($screen.Width - 5, $screen.Height - 5) | Out-Null
$SWP = 0x0004 -bor 0x0010  # NOZORDER | NOACTIVATE
[W]::SetWindowPos($h, [IntPtr]::Zero, 40, 40, 720, 450, $SWP) | Out-Null
$c = Client $h
# Keep the pointer over the caption (left part, inside every tested size) so the player
# chrome stays shown during playback; the measured corner is at the right end.
[W]::SetCursorPos($c.x + 150, $c.y + 10) | Out-Null
Start-Sleep -Seconds 2
Shot 'start.png'
$c = Client $h
$result.start_client = $c
$pw = 64; $ph = 28
$ref = [W]::Grab($c.x + $c.w - $pw, $c.y, $pw, $ph)
$ref.Save((Join-Path $Out 'corner-ref.png'))

# 1. Outside resize -> the caption corner is drawn at the new right edge.
$sizes = @(@(900, 560), @(640, 400), @(820, 520), @(560, 350), @(960, 600), @(720, 450))
$rows = @()
foreach ($round in 0..4) {
  foreach ($s in $sizes) {
    $t = [Diagnostics.Stopwatch]::StartNew()
    [W]::SetWindowPos($h, [IntPtr]::Zero, 40, 40, $s[0], $s[1], $SWP -bor 0x4000) | Out-Null  # ASYNCWINDOWPOS
    $followed = $null; $best = 999.0; $sized = $null
    while ($t.Elapsed.TotalSeconds -lt 5) {
      $cc = Client $h
      if ($sized -eq $null -and $cc.w -gt 0 -and ((Rect $h).R - (Rect $h).L) -eq $s[0]) { $sized = $t.Elapsed.TotalMilliseconds }
      $g = [W]::Grab($cc.x + $cc.w - $pw, $cc.y, $pw, $ph); $d = [W]::Diff($ref, $g); $g.Dispose()
      if ($d -lt $best) { $best = $d }
      if ($sized -ne $null -and $d -lt 8) { $followed = $t.Elapsed.TotalMilliseconds; break }
    }
    $rows += [ordered]@{ round = $round; target = $s; rect_ms = $sized; followed_ms = $followed; best_diff = [math]::Round($best, 2) }
    if ($followed -eq $null) { $result.errors += "outside resize $($s[0])x$($s[1]): corner not drawn at the new edge" }
    if ($round -eq 0) { Start-Sleep -Milliseconds 300; Shot "resize-$($s[0])x$($s[1]).png" }
  }
}
$result.outside_resize = $rows

# 2. Edge drag with synthetic input: press in the right band, move, release.
$drag = [ordered]@{ steps = @() }
[W]::SetWindowPos($h, [IntPtr]::Zero, 40, 40, 720, 450, $SWP) | Out-Null
Start-Sleep -Seconds 1
$r = Rect $h; $c = Client $h
$x = $c.x + $c.w - 3; $y = $c.y + [int]($c.h / 2)
$startW = $r.R - $r.L
[W]::SetCursorPos($x, $y) | Out-Null; Start-Sleep -Milliseconds 300
[W]::mouse_event(0x0002, 0, 0, 0, [UIntPtr]::Zero)  # LEFTDOWN
Start-Sleep -Milliseconds 300
foreach ($step in 1..6) {
  $nx = $x + 30 * $step
  $t = [Diagnostics.Stopwatch]::StartNew()
  [W]::SetCursorPos($nx, $y) | Out-Null
  [W]::mouse_event(0x0001, 0, 0, 0, [UIntPtr]::Zero)  # MOVE
  $followed = $null
  while ($t.Elapsed.TotalSeconds -lt 3) {
    $rr = Rect $h
    if (($rr.R - $rr.L) -ge $startW + 30 * $step - 2) { $followed = $t.Elapsed.TotalMilliseconds; break }
    Start-Sleep -Milliseconds 1
  }
  $drag.steps += [ordered]@{ pointer_x = $nx; want_width = $startW + 30 * $step; width = ((Rect $h).R - (Rect $h).L); followed_ms = $followed }
}
[W]::mouse_event(0x0004, 0, 0, 0, [UIntPtr]::Zero)  # LEFTUP
Start-Sleep -Milliseconds 800
$r2 = Rect $h
$drag.start_width = $startW; $drag.final_width = $r2.R - $r2.L
$drag.resized = ($drag.final_width -gt $startW)
Shot 'after-drag.png'
$cc = Client $h
$g = [W]::Grab($cc.x + $cc.w - $pw, $cc.y, $pw, $ph); $drag.corner_diff_after = [math]::Round([W]::Diff($ref, $g), 2); $g.Dispose()
if (-not $drag.resized) { $result.errors += 'edge drag did not resize the window' }
$result.drag = $drag
$p.Refresh(); $result.player_running_after = -not $p.HasExited
if ($p.HasExited) { $result.errors += "player exited (code $($p.ExitCode))" }
$result.ok = ($result.errors.Count -eq 0)
$result | ConvertTo-Json -Depth 6 | Set-Content (Join-Path $Out 'results.json')
if (-not $p.HasExited) { Stop-Process -Id $p.Id -Force }
Get-Content (Join-Path $Out 'results.json')
if (-not $result.ok) { exit 1 }

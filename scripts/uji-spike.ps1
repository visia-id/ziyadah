# Uji spike otomatis Ziyadah (F1-01): fokus keyboard, klik-tembus, transparansi.
# Ziyadah harus sudah jalan dengan WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS=--remote-debugging-port=9334 dan murottal diputar.
# Jalankan: powershell -STA -File scripts/uji-spike.ps1. Keyboard dan mouse dipakai skrip; jangan disentuh selama uji.
param([int]$TypeSeconds = 40)
$ErrorActionPreference = "Stop"
$here = $PSScriptRoot
Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @"
using System; using System.Text; using System.Runtime.InteropServices;
public class U {
  [DllImport("user32.dll")] public static extern bool SetProcessDPIAware();
  [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
  [DllImport("user32.dll")] public static extern bool SetCursorPos(int x, int y);
  [DllImport("user32.dll")] public static extern void mouse_event(uint f, uint dx, uint dy, uint d, IntPtr e);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern int GetWindowText(IntPtr h, StringBuilder s, int n);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h, out RECT r);
  [DllImport("user32.dll", CharSet=CharSet.Unicode)] public static extern IntPtr FindWindow(string c, string t);
  public struct RECT { public int L, T, R, B; }
  public static string Title(IntPtr h) { var sb = new StringBuilder(256); GetWindowText(h, sb, 256); return sb.ToString(); }
}
"@
[void][U]::SetProcessDPIAware()
function Cdp($match, $expr) { (& node "$here\cdp.mjs" $match $expr) -join "" }

$panel = [U]::FindWindow([NullString]::Value, "Ziyadah Panel")
if ($panel -eq [IntPtr]::Zero) { throw "panel tidak ditemukan" }
$r = New-Object U+RECT; [void][U]::GetWindowRect($panel, [ref]$r)
"panel: x=$($r.L) y=$($r.T) w=$($r.R-$r.L) h=$($r.B-$r.T)"

# Jendela uji: magenta, menutupi area panel dan sekitarnya; kotak teks di kiri, di luar area panel.
$form = New-Object Windows.Forms.Form
$form.Text = "Uji Spike Ziyadah"; $form.StartPosition = "Manual"; $form.BackColor = [Drawing.Color]::Magenta
$form.FormBorderStyle = "FixedToolWindow"
$form.Bounds = New-Object Drawing.Rectangle(($r.L - 700), ($r.T - 300), ($r.R - $r.L + 760), ($r.B - $r.T + 340))
$box = New-Object Windows.Forms.TextBox; $box.Multiline = $true; $box.Location = New-Object Drawing.Point(10, 10); $box.Size = New-Object Drawing.Size(600, 200)
$form.Controls.Add($box)
$script:clicks = 0
$form.Add_MouseDown({ $script:clicks++ })
$form.Show(); [Windows.Forms.Application]::DoEvents(); Start-Sleep -Milliseconds 300
# Aktifkan dengan klik nyata di kotak teks: Windows memblokir SetForegroundWindow dari proses latar.
$bp = $box.PointToScreen((New-Object Drawing.Point(20, 20)))
[U]::SetCursorPos($bp.X, $bp.Y); Start-Sleep -Milliseconds 200
[U]::mouse_event(2,0,0,0,[IntPtr]::Zero); [U]::mouse_event(4,0,0,0,[IntPtr]::Zero)
Start-Sleep -Milliseconds 400; [Windows.Forms.Application]::DoEvents()
if ([U]::GetForegroundWindow() -ne $form.Handle) { $form.Close(); throw "jendela uji tidak aktif; uji dibatalkan, tidak ada yang diketik" }

# --- Uji 2: fokus keyboard selama ayat berganti ---
$expected = New-Object Text.StringBuilder
$lostFocus = 0; $samples = 0; $ayahs = @{}
$alphabet = "abcdefghijklmnopqrstuvwxyz"
$t0 = Get-Date; $i = 0
while (((Get-Date) - $t0).TotalSeconds -lt $TypeSeconds) {
  # Pengaman: jangan pernah mengetik ke jendela lain.
  $samples++
  if ([U]::GetForegroundWindow() -ne $form.Handle) { $lostFocus++; "fokus pindah ke: " + [U]::Title([U]::GetForegroundWindow()) + " (berhenti mengetik)"; break }
  $ch = $alphabet[$i % 26]; $i++
  [Windows.Forms.SendKeys]::SendWait([string]$ch); [void]$expected.Append($ch)
  [Windows.Forms.Application]::DoEvents()
  if ($i % 25 -eq 0) { $ayahs[(Cdp "panel.html" "document.querySelector('.meta')?.textContent")] = 1 }
  Start-Sleep -Milliseconds 60
}
[Windows.Forms.Application]::DoEvents()
$typedOk = $box.Text -eq $expected.ToString()
"UJI 2 fokus: $($expected.Length) huruf diketik, teks utuh=$typedOk, fokus hilang $lostFocus dari $samples sampel, ayat yang lewat: $(($ayahs.Keys | Where-Object { $_ }) -join ' | ')"

# --- Uji 1: transparansi (tangkap layar area panel di atas latar magenta) ---
Start-Sleep -Milliseconds 300
[void][U]::GetWindowRect($panel, [ref]$r)
$w = $r.R - $r.L; $h = $r.B - $r.T
$bmp = New-Object Drawing.Bitmap $w, $h
$g = [Drawing.Graphics]::FromImage($bmp); $g.CopyFromScreen($r.L, $r.T, 0, 0, $bmp.Size)
$bmp.Save("$env:TEMP\ziyadah-uji-panel.png")
$corner = $bmp.GetPixel(1, 1); $mid = $bmp.GetPixel([int]($w / 2), 3)
"UJI 1 transparan: sudut=$corner (harus magenta murni = tembus), tepi atas tengah=$mid (harus magenta yang digelapkan = semi transparan)"

# --- Uji 4: klik-tembus ---
$cx = $r.L + [int]($w / 2); $cy = $r.B - 6
function ClickAt { [U]::SetCursorPos($cx, $cy); Start-Sleep -Milliseconds 300; [U]::mouse_event(2,0,0,0,[IntPtr]::Zero); [U]::mouse_event(4,0,0,0,[IntPtr]::Zero); Start-Sleep -Milliseconds 400; [Windows.Forms.Application]::DoEvents() }
$script:clicks = 0; ClickAt; $offClicks = $script:clicks
Cdp "panel.html" "window.__TAURI_INTERNALS__.invoke('panel_set_click_through',{on:true}).then(()=>'on')" | Out-Null
Start-Sleep -Milliseconds 400
$script:clicks = 0; ClickAt; $onClicks = $script:clicks
Cdp "localhost/" "window.__TAURI_INTERNALS__.invoke('panel_set_click_through',{on:false}).then(()=>'off')" | Out-Null
"UJI 4 klik-tembus: mati -> klik sampai ke jendela bawah $offClicks kali (harus 0); aktif -> $onClicks kali (harus 1)"

$form.Close()

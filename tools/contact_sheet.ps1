Add-Type -AssemblyName System.Drawing
$dir = if ($args[0]) { $args[0] } else { 'F:\GTAVI-Build\gta5700\runs\XCL-BOOT2\RockstarGamesInc.28872443D20A7\logs' }
$frames = if ($args[1]) { $args[1] -split ',' | ForEach-Object { [int]$_ } } else { @(20,80,140,200,260,290,320,350,500,800,1100,1400,1700,2000,2300) }
$cols = 4; $tw = 320; $th = 180; $labelH = 20
$rows = [math]::Ceiling($frames.Count / $cols)
$bmp = New-Object System.Drawing.Bitmap ($cols * $tw), ($rows * ($th + $labelH))
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.Clear([System.Drawing.Color]::Black)
$font = New-Object System.Drawing.Font 'Arial', 11
$brush = New-Object System.Drawing.SolidBrush ([System.Drawing.Color]::Yellow)
for ($i = 0; $i -lt $frames.Count; $i++) {
  $f = $frames[$i]
  $p = Join-Path $dir "frame_$f.bmp"
  if (-not (Test-Path $p)) { $p = Join-Path $dir "frame-$f.png" }
  if (-not (Test-Path $p)) { $p = Join-Path $dir "frame_$f.png" }
  if (-not (Test-Path $p)) { continue }
  $src = [System.Drawing.Bitmap]::FromFile($p)
  $x = ($i % $cols) * $tw; $y = [math]::Floor($i / $cols) * ($th + $labelH)
  $g.DrawImage($src, (New-Object System.Drawing.Rectangle $x, $y, $tw, $th))
  $g.DrawString("frame $f", $font, $brush, $x + 4, $y + $th + 2)
  $src.Dispose()
}
$g.Dispose(); $font.Dispose(); $brush.Dispose()
$out = if ($args[2]) { $args[2] } else { 'F:\GTAVI-Build\gta5700\runs\XCL-BOOT2\contact.png' }
$bmp.Save($out, [System.Drawing.Imaging.ImageFormat]::Png)
$bmp.Dispose()
$out

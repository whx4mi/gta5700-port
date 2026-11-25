param([string]$Log, [int]$MaxVramMB = 7200, [int]$Seconds = 500)
$deadline = (Get-Date).AddSeconds($Seconds)
while ((Get-Date) -lt $deadline) {
  Start-Sleep -Seconds 3
  if (-not (Test-Path $Log)) { continue }
  $lines = Get-Content $Log -Tail 60 -ErrorAction SilentlyContinue
  if (-not $lines) { continue }
  $bad = $lines | Select-String -Pattern 'no video memory for more tiles|device removed|0x887a0006'
  $vram = 0
  foreach ($l in $lines) {
    if ($l -match 'video memory (\d+) MB local') { $vram = [int]$matches[1] }
  }
  if ($bad -or $vram -gt $MaxVramMB) {
    "WATCHDOG: trigger (vram=$vram MB, bad=$([bool]$bad))" | Out-File "$env:TEMP\opencode\watchdog.trigger" -Encoding utf8
    Get-Process -Name xcl -ErrorAction SilentlyContinue | Stop-Process -Force
    "WATCHDOG: killed xcl"
    exit 0
  }
}
"WATCHDOG: timeout"

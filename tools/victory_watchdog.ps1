param([int]$Seconds = 200)
$deadline = (Get-Date).AddSeconds($Seconds)
$crashLog = 'F:\GTAVI-Build\gta5700\runs\victory-data\crashes.log'
while ((Get-Date) -lt $deadline) {
  Start-Sleep -Seconds 3
  if (Test-Path $crashLog) {
    $tail = Get-Content $crashLog -Tail 20 -ErrorAction SilentlyContinue
    if ($tail -match 'device lost|device removed|GPU failure|0x887a0006') {
      "VWATCH: GPU failure in crashes.log" | Out-File "$env:TEMP\opencode\victory-watch.trigger" -Encoding utf8
      Get-Process -Name victory -ErrorAction SilentlyContinue | Stop-Process -Force
      exit 0
    }
  }
}
"VWATCH: timeout" | Out-File "$env:TEMP\opencode\victory-watch.trigger" -Encoding utf8
Get-Process -Name victory -ErrorAction SilentlyContinue | Stop-Process -Force

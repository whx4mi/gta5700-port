param(
  [int]$Segundos = 600,
  [string]$Keys = '',
  [switch]$Capturar,
  [switch]$Release
)

Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" |
  Where-Object { $_.CommandLine -like '*victory_watchdog.ps1*' -and $_.ProcessId -ne $PID } |
  ForEach-Object { Stop-Process -Id $_.ProcessId -Force }

Remove-Item "$env:TEMP\opencode\victory-watch.trigger" -ErrorAction SilentlyContinue

Set-Location 'E:\victory-fork'
if ($Capturar) {
  New-Item -ItemType Directory -Force -Path 'E:\victory-capture' | Out-Null
  $env:VICTORY_CAPTURE = 'E:\victory-capture'
  $env:VICTORY_CAPTURE_FRAMES = '20,100,400,600,800,1000,1200,1500,2000,2200,2500,3000,3500,4000,5000,6000,7000,8000'
}
if ($Keys) { $env:VICTORY_KEYS = $Keys }

Start-Process -FilePath powershell -ArgumentList @(
  '-ExecutionPolicy','Bypass','-File',
  'C:\Users\Whxami\AppData\Local\Temp\opencode\victory_watchdog.ps1',
  '-Seconds', ($Segundos + 15)
) -WindowStyle Hidden | Out-Null

$exe = if ($Release) { 'E:\victory-fork\target\release\victory.exe' } else { 'E:\victory-fork\target\diag\victory.exe' }
"Abrindo $exe por $Segundos s (watchdog ativo)."
"Input: setas = D-pad, Enter = A, Backspace = B, WASD = analógico."
& $exe run 'F:\GTA6\GTA6.Xbox.v1.0.2.1-CYBERLEEK\game_scarlett_master.exe' --execute

param(
  [string]$Nome = "fork",
  [int]$Segundos = 400,
  [switch]$Warp,
  [string]$Keys = '',
  [switch]$SemWatchdog,
  [string]$PressFile = ''
)

$me = $PID
Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" |
  Where-Object { $_.CommandLine -like '*victory_watchdog.ps1*' -and $_.ProcessId -ne $me } |
  ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
Get-Process -Name victory,xcl,game_scarlett_master -ErrorAction SilentlyContinue | Stop-Process -Force
Remove-Item "$env:TEMP\opencode\victory-watch.trigger" -ErrorAction SilentlyContinue

$cache = "F:\GTAVI-Build\gta5700\runs\victory-cache-$Nome"
$data = "F:\GTAVI-Build\gta5700\runs\victory-data-$Nome"
$capture = "F:\GTAVI-Build\gta5700\runs\victory-capture-$Nome"
New-Item -ItemType Directory -Force -Path $cache,$data,$capture | Out-Null

if (-not $SemWatchdog) {
  Start-Process -FilePath powershell -ArgumentList @(
    '-ExecutionPolicy','Bypass','-File','F:\GTAVI-Build\gta5700\tools\victory_watchdog.ps1',
    '-Seconds',($Segundos + 20)
  ) -WindowStyle Hidden | Out-Null
}

$env:VICTORY_CACHE_DIR = $cache
$env:VICTORY_DATA_DIR = $data
$env:VICTORY_CAPTURE = $capture
$env:VICTORY_CAPTURE_FRAMES = "20,100,200,400,600,800,1000,1200,1500,2000,2500,3000,4000,5000"
$env:VICTORY_TEXTURE_BUDGET_MB = "1024"
$env:VICTORY_CARD_BUFFERS_MIN_MB = "1"
$env:VICTORY_CARD_BUFFERS_MB = "256"
$env:VICTORY_RELAY = "2"
if ($Warp) { $env:VICTORY_D3D12_WARP = "1" }
if ($Keys) { $env:VICTORY_KEYS = $Keys }
if ($PressFile) { $env:VICTORY_PRESS_FILE = $PressFile; Set-Content -Path $PressFile -Value '' -NoNewline }

$exe = "F:\GTAVI-Build\gta5700\repos\victory-fork\target\diag\victory.exe"
$game = "F:\GTA6\GTA6.Xbox.v1.0.2.1-CYBERLEEK\game_scarlett_master.exe"

"Fork: $exe"
"Cache: $cache | Data: $data | Captura: $capture"
"WARP: $([bool]$Warp) | VICTORY_KEYS: $Keys | Watchdog: $(-not $SemWatchdog)"
& $exe run $game --execute

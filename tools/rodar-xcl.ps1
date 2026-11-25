param(
  [string]$Nome = "manual",
  [int]$Segundos = 300,
  [int]$FrameDump = 120,
  [switch]$SemWatchdog
)

$me = $PID
Get-CimInstance Win32_Process -Filter "Name='powershell.exe'" |
  Where-Object { $_.CommandLine -like '*_watchdog.ps1*' -and $_.ProcessId -ne $me } |
  ForEach-Object { Stop-Process -Id $_.ProcessId -Force }
Get-Process -Name xcl,game_scarlett_master -ErrorAction SilentlyContinue | Stop-Process -Force

$root = "F:\GTAVI-Build\gta5700\runs\XCL-$Nome-$(Get-Date -Format HHmm)"
New-Item -ItemType Directory -Force -Path $root | Out-Null
$log = "$root\RockstarGamesInc.28872443D20A7\logs\xcl.log"
Remove-Item "$env:TEMP\opencode\watchdog.trigger" -ErrorAction SilentlyContinue

if (-not $SemWatchdog) {
  Start-Process -FilePath powershell -ArgumentList @(
    '-ExecutionPolicy','Bypass','-File',
    'F:\GTAVI-Build\gta5700\tools\vram_watchdog.ps1',
    '-Log',$log,'-MaxVramMB','8400','-Seconds',($Segundos + 20)
  ) -WindowStyle Hidden | Out-Null
}

$env:XCL_NETWORK = "0"
$env:XCL_DXIL_CACHE = "F:\GTAVI-Build\gta5700\cache\xcl-dxil"
$env:XCL_DATA_ROOT = $root
$env:XCL_SHARED_TEXTURE_PAGES = "1"
$env:XCL_NO_FATAL = "1"
$env:XCL_CONTINUE_ASSERTS = "1"
$env:XCL_SAFE_READ = "1"
$env:XCL_PSO_WAIT_MS = "100"
$env:XCL_FRAME_DUMP = "$FrameDump"
$env:XCL_RUN_SECONDS = "$Segundos"

$exe = "F:\GTAVI-Build\gta5700\build\xcl-target\release\xcl.exe"
$game = "F:\GTA6\GTA6.Xbox.v1.0.2.1-CYBERLEEK"

"Root: $root"
"Log: $log"
"Controle: setas = D-pad, Enter = A, Backspace = B, WASD = analogico."
"Watchdog: $(if($SemWatchdog){'desligado'}else{'ativo'})"
& $exe run $game

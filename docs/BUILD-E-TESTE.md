# Build, testes e execução

## XCL (referência / correções de baixo nível)

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$env:CARGO_TARGET_DIR = "F:\GTAVI-Build\gta5700\build\xcl-target"
cd F:\GTAVI-Build\gta5700\repos\xcl
cargo build --release --workspace
cargo test -p xcl --lib -- --test-threads=4
```

Binário: `F:\GTAVI-Build\gta5700\build\xcl-target\release\xcl.exe`.
Baseline de testes: 695/4/21 (4 flaky por driver/GPU).

## victory-fork (base atual)

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
cd F:\GTAVI-Build\gta5700\repos\victory-fork
cargo build --profile diag -p victory --features diagnostics
cargo check -p victory-gpu --all-targets
cargo test -p victory-shader --lib
cargo test -p victory-gpu --lib shader_cache
```

Branch `mesh-fallback` contém o port; `main` intacto (`a803c87`). Sem commit/push.

## Scripts de execução

- `tools\rodar-xcl.ps1` — XCL com config validada, watchdog e data root por run.
  ```powershell
  powershell -ExecutionPolicy Bypass -File F:\GTAVI-Build\gta5700\tools\rodar-xcl.ps1 -Nome teste -Segundos 600 -FrameDump 60
  ```
- `tools\rodar-fork.ps1` — fork com caps de VRAM e watchdog.
  ```powershell
  powershell -ExecutionPolicy Bypass -File F:\GTAVI-Build\gta5700\tools\rodar-fork.ps1 -Nome teste -Segundos 600
  # -Warp para software; -Keys '20:Enter' para input automático
  ```

## Variáveis importantes

### XCL
`XCL_NETWORK=0`, `XCL_DXIL_CACHE=F:\GTAVI-Build\gta5700\cache\xcl-dxil`, `XCL_DATA_ROOT`, `XCL_SHARED_TEXTURE_PAGES=1`, `XCL_NO_FATAL=1`, `XCL_CONTINUE_ASSERTS=1`, `XCL_SAFE_READ=1`, `XCL_PSO_WAIT_MS=100`, `XCL_FRAME_DUMP`, `XCL_TEXTURE_RECHECK` (default OFF), `XCL_PROFILE=1`.

### Fork
`VICTORY_CACHE_DIR`, `VICTORY_DATA_DIR`, `VICTORY_CAPTURE`, `VICTORY_CAPTURE_FRAMES`, `VICTORY_TEXTURE_BUDGET_MB=1024`, `VICTORY_CARD_BUFFERS_MIN_MB=1`, `VICTORY_CARD_BUFFERS_MB=256`, `VICTORY_RELAY=2`, `VICTORY_MESH_FALLBACK=1`, `VICTORY_D3D12_WARP=1`, `VICTORY_KEYS`.

## Watchdog

`C:\Users\Whxami\AppData\Local\Temp\opencode\vram_watchdog.ps1` (XCL) e `victory_watchdog.ps1` (fork). Matam o processo se VRAM > limite, device removed ou timeout. Antes de cada run, matar watchdogs órfãos (excluir o PID atual) e processos `xcl`, `victory`, `game_scarlett_master`.

## Captura de frames

- XCL: `frame_<n>.bmp` em `<data root>\RockstarGamesInc.28872443D20A7\logs\`.
- Fork: `frame-<n>.png` em `VICTORY_CAPTURE`.
- Contact sheet: `%TEMP%\opencode\contact_sheet.ps1 <dir> "n1,n2,..." <out.png>`.

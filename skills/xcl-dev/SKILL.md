---
name: xcl-dev
description: XCL compatibility layer reference for the gta5700 project (GTA VI on RX 5700 RDNA1) - RE, aborts/asserts, write-watch/chunk hash, helper lanes, PSO stand-in, mesh fallback source. Use in F:\GTAVI-Build\gta5700\repos\xcl. XCL e referencia; a base do projeto e o victory-fork.
---

# xcl-dev (gta5700 / RX 5700 RDNA1)

XCL e a **referencia de compatibilidade** do projeto `F:\GTAVI-Build\gta5700` (GitHub `whx4mi/gta5700-port`). A base executavel e o victory-fork (`repos/victory-fork`); XCL fica em `repos/xcl` com 16 arquivos modificados (backup em `backups/`). Foi dele que vieram: fallback de mesh (portado para o fork), chunk hash, helper lanes, RE dos aborts/asserts, PSO stand-in, `XCL_PSO_WAIT_MS`, budget/eviccao de VRAM.

## Regras de ouro

- Nunca rodar XCL e victory/WARP ao mesmo tempo (ja gerou `device removed 0x887a0007`).
- Checar processos e watchdogs orfaos antes de cada run.
- Sempre `XCL_NETWORK=0`, `XCL_DXIL_CACHE=F:\GTAVI-Build\gta5700\cache\xcl-dxil`.
- Pagefile em E: -> commit ~96 GB (evita OOM).

## Build/teste

```powershell
$env:Path="$env:USERPROFILE\.cargo\bin;$env:Path"
$env:CARGO_TARGET_DIR="F:\GTAVI-Build\gta5700\build\xcl-target"
cd F:\GTAVI-Build\gta5700\repos\xcl
cargo build --release --workspace
cargo test -p xcl --lib -- --test-threads=4
```
Baseline: 695 passed / 4 failed / 21 ignored (4 falhas flaky por driver/GPU; ja foi 699/0/21 na mesma arvore).

## Correcoes validadas no XCL

| Sintoma | Correcao | Medicao |
|---|---|---|
| Intro da Rockstar nao renderizava | `XCL_TEXTURE_RECHECK` **desligado por padrao** (port do fork era prejudicial: A/B mostrou tela escura com ele ON) | OFF -> aguia/murais/logo R* aparecem |
| `chunk scan` 8270 ms em 401 s | `CHUNK` 64 -> 256 B em `sparse.rs` (hash dobrado; testes ajustados) | **8270 ms -> 86 ms** com 1,55 M paginas sujas |
| Linhas em pintura/vidro/pele | `IsHelperLane()` no `v_readfirstlane` (port do Belobug-AMD) | Build/testes OK |
| Buffer fixo de 16 K entradas no GetWriteWatch | buffer dimensionado para a maior faixa | Build/testes OK |

## Descobertas RE

- 7 aborts deliberados `mov dword [0], imm` (`XCL_NO_FATAL=1`); soft asserts int3 VEH (`XCL_CONTINUE_ASSERTS=1`); site benigno `0x2215c40`, site letal `0x2474606` (flag `0x14a943999` nunca setada -> burst de 32768 -> device removed).
- `XCL_PSO_WAIT_MS=100` destrava binds com stand-in (unset/0 = espera ilimitada).
- Primeiro-boot: `XCL_PRESS_AT` (gamepad HLE) nao avanca de forma confiavel a 2-6 fps; usar o fork com `VICTORY_PRESS_FILE`.
- Shader pack: 26.503 containers; precompile 22.333 + 13.238 cache, 19 erros (todos mesh sm6_6) = 99,9% DXIL. Traducao nao e o gargalo; RDNA1 sem SM6.6/mesh e.

## Executar XCL (referencia)

`powershell -ExecutionPolicy Bypass -File F:\GTAVI-Build\gta5700\tools\rodar-xcl.ps1 -Nome teste -Segundos 600`

## Documentacao

`F:\GTAVI-Build\gta5700\docs\`: STATUS (medicoes), REPOSITORIOS (o que foi reaproveitado de cada repo), BUILD-E-TESTE, ROADMAP, RETOMADA, INTRO-BINK. `context/roadmap-6h.md` nesta skill e o guia de 6 horas. Atualizar e commitar conforme avancarmos.

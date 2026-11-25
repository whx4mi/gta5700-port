---
name: victory-fork-dev
description: victory-fork GTA VI boot-to-prologue no gta5700 (RX 5700 RDNA1) - build diag, cache quente obrigatorio, Bink/intro travada, VICTORY_PRESS_FILE, fallback de mesh, watchdog. Use em F:\GTAVI-Build\gta5700\repos\victory-fork. Base preferida do gta5700; XCL e referencia.
---

# victory-fork-dev (gta5700 / RX 5700 RDNA1)

Projeto: `F:\GTAVI-Build\gta5700` -> GitHub `whx4mi/gta5700-port`. Repos de trabalho em `repos/`, codigo publico em `vendor/`, documentacao em `docs/`, scripts em `tools/`, caches em `runs/`. Base do projeto e o `victory-fork` (`repos/victory-fork`, branch `mesh-fallback`); XCL (`repos/xcl`) e referencia de compatibilidade.

## Regras de ouro

1. **Cache quente obrigatorio**: `VICTORY_CACHE_DIR=F:\GTAVI-Build\gta5700\runs\victory-cache` (0,61 GB). Cache frio -> intro escura e FPS instavel. Nunca criar cache novo para testar boot.
2. **Nunca dois processos de GPU ao mesmo tempo**: checar `Get-Process -Name xcl,victory,game_scarlett_master` e matar watchdogs orfaos antes de cada run (excluir o PID atual).
3. **Watchdog**: `tools\victory_watchdog.ps1` mata no timeout / device removed. Para navegacao manual longa, usar `Start-Process ... direto` sem watchdog.

## Build

```powershell
$env:Path="$env:USERPROFILE\.cargo\bin;$env:Path"
cd F:\GTAVI-Build\gta5700\repos\victory-fork
cargo build --profile diag -p victory --features diagnostics
cargo test -p victory-shader --lib
cargo test -p victory-gpu --lib shader_cache
```

## Executar

- Script: `powershell -ExecutionPolicy Bypass -File F:\GTAVI-Build\gta5700\tools\rodar-fork.ps1 -Nome teste -Segundos 600 [-Warp] [-SemWatchdog] [-PressFile <path>] [-Keys '20:Enter']`
- **VICTORY_PRESS_FILE** (portado do ScarlettPort): escrever `A`, `DOWN DOWN A` no arquivo pressiona botoes sequencialmente (150 ms, 1,5 s para sticks) e esvazia o arquivo. Resolve a navegacao dos termos.
- Sempre com cache quente e variaveis de caps de VRAM para estabilidade do mundo: `VICTORY_TEXTURE_BUDGET_MB=1024`, `VICTORY_CARD_BUFFERS_MIN_MB=1`, `VICTORY_CARD_BUFFERS_MB=256`, `VICTORY_RELAY=2`.

## Estado atual (25 Nov)

| Item | Situacao |
|---|---|
| Boot/menus | 30 fps estavel com cache quente, 0 pipelines recusadas nessa fase |
| Intro Rockstar (murais/aguia/estrelas) | **Travada com cortes** mesmo a 30 fps; causa documentada em `docs/INTRO-BINK.md` (texturas de filme pulam hash; ScarlettPort corrige com `film = linear && mips==1`) |
| Arte de loading com personagens | As vezes ausente; relacionada a texturas de filme / cache |
| Loading/mundo | FPS cai de 30 -> 14 -> 7 quando o carregamento comeca |
| Fallback de mesh | Portado (branch `mesh-fallback`); build/testes OK; corrigido VS pass-through (container do PS + outputs do mesh quando sem DXBC); **validacao em mundo pendente** |
| Estabilidade | Sem device removed com caps de VRAM; watchdog obrigatorio |

## Causa da intro travada (ScarlettPort)

`refs/ScarlettPort/runtime/victory/crates/victory-gpu/src/texture.rs:795-827`:
> the intro's movie: three pictures in turn that only changed every eighth frame, one old picture after another on the screen.

O refresh de textura pula o hash em 7 de cada 8 frames (`hash_every=8`); planos de video Bink sao lineares de 1 mip. A correcao: `let film = desc.swizzle.0 == 0 && desc.mips == 1;` e nunca pular o hash para filmes. Nossa base `a803c87` e mais antiga e nao tem `hash_every`; tentativa de reduzir `RECHECK_FRAMES` de `[4,12,40,120,400]` para `[1,2,4,12,40]` nao resolveu visualmente (ver `runs/cap-intro-hw`).

Numeros de shader: `victory shader-info shaders.rpf`: 26.503 containers, Vs 2.236, Ps 12.101, Ms 9.282, Cs 2.689. XCL report: CS 2.734, NGG 1.569, PS 3.297, VS 1.034. Precompile XCL: 22.333 compilados + 13.238 cache, 19 erros (todos mesh sm6_6) = ~99,9% DXIL. Traducao NAO e o gargalo; o problema e runtime/SM6.6-mesh na RDNA1.

## Repos de referencia

`refs/`: VIctory (upstream 70564ae), ScarlettPort (launcher + `victory warm`/`precompile-graphics`, `VICTORY_PRESS_FILE`, `VICTORY_NO_WATCHDOG`, patches), Belobug-AMD (quad reads/helper lanes, HOIST_CLAMPS, ALWAYS_COPY), five-star-recomp (mesh fallback), ztrx (so README), OpenScarlett, ScarlettX, Xbox2Windows.

## Documentacao do projeto

`F:\GTAVI-Build\gta5700\README.md`, `docs/STATUS.md`, `docs/REPOSITORIOS.md`, `docs/BUILD-E-TESTE.md`, `docs/ROADMAP.md`, `docs/RETOMADA.md`, `docs/INTRO-BINK.md`. Atualizar conforme avancarmos; commitar e dar push para `whx4mi/gta5700-port`.

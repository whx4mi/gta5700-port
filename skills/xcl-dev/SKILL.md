---
name: xcl-dev
description: XCL (Xbox Series X|S -> Windows) boot-to-world workflow for GTA VI - build, test, menu/cutscene/texture rendering, shader precompile cache, PSO stand-in, soft asserts int3, gamepad input, device-removed triage. Use when working in F:\GTAVI-Build\gta5700\repos\xcl or debugging GTA VI boot, menus, loading video, textures, subtitles, audio, PSO hangs or crashes after ~8 minutes.
---

# XCL development — objetivo: entrar no jogo

XCL executa código GDK de Xbox Series X|S nativamente no Windows: sem emulação de CPU, bibliotecas de console reimplementadas e programas RDNA2 recompilados para DXIL. O foco é **fazer o GTA VI bootar até o mundo**, com menu, legendas, cutscenes e texturas renderizando.

> **Base preferida agora: `victory-fork`.** O XCL não renderiza as ilustrações de personagens do loading e estoura a VRAM (~9 GB) no carregamento do mapa. O fork `fxceless/victory-fork` (`F:\GTAVI-Build\gta5700\repos\victory-fork`, skill `victory-fork-dev`) já renderiza a intro completa, as imagens de loading com personagens e o prólogo. **Use as duas skills**: `victory-fork-dev` para boot/cutscene/mundo; esta para RE, aborts/asserts e diagnóstico de baixo nível. Backup do XCL: `F:\GTAVI-Build\gta5700\backups\xcl-20261124-1836\`.
> **Limitação de hardware:** na RX 5700 (RDNA1, sem mesh shaders) o fork dá `device removed 0x887A0004/0x887A0001` ao entrar no mundo; a cutscene roda com texturas transparentes/bugadas e a intro da Rockstar fica tremula/com áudio lagado. WARP corrige a imagem, mas a ~1 fps. `VICTORY_DISPLAY_MODE=1280x720 --console series-s` em hardware dá tela preta. Upstream `xcl-dev/xcl` está em `20cf130` (fix MSAA/AMD, world-freeze 180 s) — repo local com mudanças não commitadas; não mesclar sem backup.
> **Decisão (24 Nov 2026): XCL é a base primária nesta máquina.** Empiricamente o XCL já renderizou intro/áudio e chegou a ~30 fps no mundo (`xcl-pso100`); o fork depende de SM6.6/mesh e falha na RX 5700. Manter `victory-fork` como referência (ideias de render/loading art, upstream Atualizado) e o port de fallback de mesh apenas como plano em `docs/fork-mesh-fallback.md`. Foco seguinte no XCL: arte de loading com personagens, OOM de VRAM no mapa e burst de `int3` em `0x2474606`.
> **Intro corrigida (A/B, 25 Nov)**: o port do recheck de texturas (`XCL_TEXTURE_RECHECK=1`) fazia a intro da Rockstar não renderizar (tela escura). Desligado por padrão, a intro volta **completa**, incluindo a águia com asas na bandeira dos EUA, murais e logo R*. Ver `context/roadmap-6h.md`.

- Repo local: `F:\GTAVI-Build\gta5700\repos\xcl` (GitHub `ivall/xcl`, GPL-2.0). HEAD local `47efa2e`; upstream mesclado em várias rodadas (85bdf21 → cb868b2 → 67a3735 → 47efa2e). Há mudanças locais **não commitadas** em 16 arquivos: fallback VS/AS para GPUs sem mesh shaders, stub `XMemIsToolsMemAvailable`, profiling `XCL_PROFILE`, budget/evicção de VRAM, stand-in 1×1 em OOM, `XCL_SAFE_READ`, `XCL_NO_FATAL` (7 aborts), `XCL_CONTINUE_ASSERTS` (int3 com VEH) e logging de código de erro em `0xc0000005`.
- GPU local: AMD RX 5700 (RDNA1, 8 GB VRAM, sem mesh shaders), 32 GB RAM, CPU Ryzen 5 5600G (6 núcleos).
- Jogo: `F:\GTA6\GTA6.Xbox.v1.0.2.1-CYBERLEEK` (v1.0.2.1, `game_scarlett_master.exe`, Scarlett).
- Cache DXIL concluído: `F:\GTAVI-Build\gta5700\cache\xcl-dxil`, 4,18 GB / 28.853 arquivos; log `%TEMP%\opencode\precompile-full.txt` com marcador `.done`. Resultado: `compiled 22333, already cached 13238, errors 19, skipped 390`.
- Projetos relacionados (referência, não base): VIctory em `C:\Users\Whxami\AppData\Local\Temp\opencode\VIctory` (checkout `97c78d1`); Xbox2Windows em `%TEMP%\opencode\Xbox2Windows`; AnyBox em `%TEMP%\opencode\AnyBox`; REA em `C:\Users\Whxami\OneDrive\Área de Trabalho\EngenhariaReversa\rea-main` (v6.7.0, sem motor Ghidra — falta instalar); skill VIctory em `F:\GTAVI-Build\.opencode\skills\victory-boot\SKILL.md`.

## Progresso real (atualizado 24 Nov 2026)

| Item | Estado |
|---|---|
| Boot/estabilidade | OK. Menus ~20–30 fps após otimizações do upstream (detile/hash in-place, write-watch em poucas chamadas, preload de PSOs). |
| Menu seleção de conta / avisos legais / calibração de brilho | Renderizam e são navegáveis (texto PT-BR nítido). |
| Vídeo Bink e arte de loading | Renderizam. Frames coloridos aparecem por janela curta (~47–65 s); press automático pode pulá-la. |
| Áudio | `xapu` conectado via MSOpusDecoder: 9003 packets, 0 erros, ~9,7 M amostras não-zero no run longo. `report.audio.engines=0` porque Bink importa XAudio2_9.dll diretamente, fora do HLE. |
| BCPack | Decoder do VIctory portado ao XCL upstream (`hle/bcpack/`): BC1/BC3/BC4/BC5/BC6H/BC7 "unpacked", sem E_DECOMPRESSION. |
| **Mundo** | **Destravado com `XCL_PSO_WAIT_MS=100`**: ~30 fps sustentados, 9600 frames em ~370 s, dumps com cores variadas (frame 5300: 518034/518400 amostras não-pretas). ~298 k draws com pipeline traduzida, 18 k dispatches. |
| Soft asserts | VEH `XCL_CONTINUE_ASSERTS=1` detecta e continua `int3`: RVA `0x2215c40` em 429 s e 477 s. RVA `0x2474606` entra em burst e derruba o device (ver Problemas). |
| VRAM | RX 5700 tem 8 GB; o jogo reserva 7,4 GB só em sparse. Sem VRAM compartilhada: `CreatePlacedResourceX` falha com E_OUTOFMEMORY → device removed. Com `XCL_SHARED_TEXTURE_PAGES=1`: evita OOM, mas houve crash de mirror stale. |
| Sessão longa | Usuário relata **sem crash até 530 s**. O log disponível (`xcl-pso100`) registra o fim em ~483 s (device removed + `0xc0000005`); confirmar em novo run com o mesmo build. |

Os 40 `d3d12.translation_errors` são pipelines MS de minimap com shader no slot AS. **Recusá-los é correto**: reativar fallback AS→MS causa tela preta (testado).

## Correções confirmadas (causa → efeito)

| Sintoma observado | Causa | Correção / estado |
|---|---|---|
| Tela escura com spinner, 0 fps no carregamento | `XCL_PSO_WAIT_MS` unset/0 = espera **ilimitada** por PSO em build; PSOs de compute de 200–560 KB levam 10–20 s cada | `XCL_PSO_WAIT_MS=100`: binds seguem com stand-in e o jogo renderiza. **Confirmado**. |
| `int3` mata o jogo no texture streaming | Soft assert do retail (~10/s) | `XCL_CONTINUE_ASSERTS=1`: VEH avança RIP (1 byte p/ `CC`, 2 p/ `CD 03`) e continua. **Confirmado** em `0x2215c40`. |
| `0xc0000005` com `write at address 0x0` | Um dos 7 aborts deliberados (`mov dword [0], imm`) | `XCL_NO_FATAL=1` neutra os 7 sites no startup. **Confirmado**. |
| `read at 0xb96a20000` (mirror stale) | memcpy em memória de GPU view sem mirror válido | `XCL_SAFE_READ=1` força `VirtualQuery`. **Confirmado**. |
| `CreatePlacedResourceX` E_OUTOFMEMORY / device removed | 8 GB insuficientes; sparse reserva 7,4 GB | `XCL_SHARED_TEXTURE_PAGES=1` + budget/evicção. **Parcial** — ver Problemas. |
| Tela preta ao aceitar pipelines MS no slot AS | 40 `translation_errors` de minimap | Recusar e usar stand-in. **Confirmado**. |
| Menu cai para 4–11 fps | hash/detile fora do caminho quente | detile/hash in-place do upstream + write-watch em poucas chamadas. **Confirmado**. |
| `chunk scan` de 8270 ms em 401 s (loading a 2 fps) | hash de 64 B por página, 4× mais chamadas que o necessário | `CHUNK` 64 → 256 B em `sparse.rs` (hash dobrado; testes ajustados). Medido: **8270 ms → 86 ms** com 1,55 M páginas sujas. **Validado (25 Nov)** |
| Linhas em pintura/vidro/pele (quad edge) | helper lanes no `rla` e `v_readfirstlane` | `v_readfirstlane` agora usa `IsHelperLane()` para helper ler seu próprio valor (port de `VIctory-Belobug-AMD`, `lower.rs:4411`). Build/testes OK (25 Nov) |

## Testes a serem feitos

1. **Build**: `cargo build --release --workspace` com `CARGO_TARGET_DIR` — OK no build atual (2 warnings não-bloqueantes).
2. **Unit tests**: `cargo test -p xcl --lib -- --test-threads=4` — baseline atual **699 passed, 0 failed, 21 ignored** (era 695/4/21; as 4 falhas de VRAM/FMA desapareceram).
3. **Smoke de boot**: config completa abaixo, `XCL_NETWORK=0`, clique do usuário nos avisos.
4. **Regressão do PSO wait**: comparar unset (trava) vs `100` (renderiza); checar `frames presented` e contador de "pipeline still being built".
5. **Estresse de assert**: run longo (>9 min) com `XCL_CONTINUE_ASSERTS=1`, observar se `0x2474606` volta a disparar em burst.
6. **Validação visual**: `XCL_FRAME_DUMP=120`, conferir frames do mundo, legendas e cutscene.
7. **VRAM/áudio/indirect**: ver detalhes em `context/next-steps.md`.

Matriz completa em `context/build-and-test.md`.

## Problemas a investigar

1. **Burst de int3 em `0x2474606`**: em 480,56 s o contador foi de 1 → 32768 em ~0,4 s (o VEH loga apenas potências de 2). RE mostrou `cmp byte [0x14a943999], 1 / jne int3`: a flag começa em 0 (`.data` zero-filled) e nunca é setada. Ach quem a escreve e sete no HLE; só depois considere lista negra/throttle no VEH. Não confundir com `0x2215c40`, que é benigno.
2. **Device removed `0x887a0006` em 481 s**: precedido de muitos `CreatePlacedResourceX ... a resource of its own (0x887a0005)` e do burst acima. Determinar se a causa é o burst, OOM de VRAM ou submissão inválida.
3. **Crash `0xc0000005` em `game_scarlett_master.exe+0x4dad047`** (read `0x17305b1a1b3`, bytes `4c 8b 41 18 ...`), com stack passando por `game+0x2213e3a`/`0x22145b1` (vizinhança do assert). Pode ser consequência do device removed, não causa raiz.
4. **Write-watch dominando o submit**: `90743 ms` acumulados em 2.789.282 páginas sujas (311 s), vs. lock 6 ms / scan 825 ms / upload 1248 ms. Entender por que o custo exploded no mundo.
5. **GPU queue.Wait timeout 5000 ms** com release no CPU ("prevent GPU hang") em 406 s — verificar se masks fences corretamente.
6. **`ExecuteIndirect` skipped**: 512 skips em 417 s (PSO ainda em build) e 4 pipes PC; confirmar se draws do mundo ficam faltando.
7. **Legendas/cutscenes longas e saída de áudio audível** — ver `context/next-steps.md`.

## Descobertas de engenharia reversa

- O jogo tem **7 sites de abort deliberado** no `.text`: padrão `C7 04 25 00 00 00 00 <imm32>` (`mov dword [0], imm`). RVAs: `0x2215c35`, `0x2215da9`, `0x23cf683` (`0x4d8bd421`), `0x258b000`, `0x29e0880`, `0x390e0c0`, `0x402ac70`. `XCL_NO_FATAL=1` neutramos todos no startup (após `load_main`).
- O jogo usa **soft asserts `int3`** (~10/s no texture streaming, confirmado pelo Xbox2Windows). `XCL_CONTINUE_ASSERTS=1` instala VEH que avança RIP e continua; log `soft assert (int3) ... continued`. RVAs vistos: `0x2215c40` (isolado, 2×) e `0x2474606` (burst). RE por capstone: `0x2215c40` é o `int3` após o abort em `0x2215c35` (o `XCL_NO_FATAL` NOPa 11 bytes e cai nele); `0x2474606` é `cmp byte [0x14a943999], 1 / jne int3` — flag em `.data` RVA `0xa943999`, zero-filled no load. São causas diferentes.
- Crash `0xc0000005` com `write at address 0x0` era o abort deliberado, não ponteiro aleatório. Um segundo crash (read em mirror stale `0xb96a20000`) é memcpy em memória de GPU view; `XCL_SAFE_READ=1` força `VirtualQuery` em todas as leituras.
- `XCL_PROFILE=1` mediu o caminho de submit: `refresh` 10–70 ms, replay 0,03–0,16 ms; dentro do batch de 12 MB: hash 3–8 ms (outlier 74 ms), detile 4–8 ms, gpu+wait 3–4 ms. Desligar o hash piora (detile 53 ms, FPS cai 11→4).
- `XCL_PSO_WAIT_MS` unset/0 = espera **ilimitada** por PSO em build; com PSOs de 20 s o jogo fica em 0 FPS. Valor `100` deixa seguir com stand-in — correção confirmada.

## Paths que importam

- Binário em desenvolvimento: `F:\GTAVI-Build\gta5700\build\xcl-target\release\xcl.exe` (use `CARGO_TARGET_DIR`).
- Binário antigo/pré-merge: `F:\GTAVI-Build\gta5700\repos\xcl\target\release\xcl.exe`.
- Dados/saves/logs por run: `C:\Users\Whxami\AppData\Local\Temp\opencode\xcl-<nome>\RockstarGamesInc.28872443D20A7\`.
- Run de referência do mundo: `...\opencode\xcl-pso100\` (log `xcl.log`, `report.json`, frames `frame_<n>.bmp`).
- Log: `...\logs\xcl.log`; relatório: `...\logs\report.json`; frames: `...\logs\frame_<n>.bmp` (converter com System.Drawing).
- Cache DXIL: `F:\GTAVI-Build\gta5700\cache\xcl-dxil` via `XCL_DXIL_CACHE`.

## Workflow

1. Leia `context/architecture.md` antes de mexer em runtime.
2. Siga `context/build-and-test.md` para comandos, baseline, matriz de testes e variáveis.
3. Use `context/next-steps.md` como plano atual (testes / causas e correções / investigar).
4. Sempre `XCL_NETWORK=0`: o crash reporter envia minidump.
5. `XCL_PRESS_AT` para input, `XCL_FRAME_DUMP` para frames (60/120 para vídeos curtos).
6. Triagem: `unhandled exception`, `translation_errors`, `STUB`, `raw texels`, `frames presented`, `soft assert`, `XCL_NO_FATAL`, `device removed`, `still being built`.
7. Input sintético (SendKeys/keybd_event) **não avança** os menus de avisos — pedir ao usuário para clicar.

## Executar (configuração validada no run do mundo)

```powershell
$root = "C:\Users\Whxami\AppData\Local\Temp\opencode\xcl-pso100b"
$exe  = "F:\GTAVI-Build\gta5700\build\xcl-target\release\xcl.exe"
$env:XCL_NETWORK="0"
$env:XCL_DXIL_CACHE="F:\GTAVI-Build\gta5700\cache\xcl-dxil"
$env:XCL_DATA_ROOT=$root
$env:XCL_SHARED_TEXTURE_PAGES="1"
$env:XCL_NO_FATAL="1"
$env:XCL_CONTINUE_ASSERTS="1"
$env:XCL_SAFE_READ="1"
$env:XCL_PSO_WAIT_MS="100"
$env:XCL_FRAME_DUMP="120"
$env:XCL_PRESS_AT="40:A,52:A,64:A,75:A,85:A,95:A,105:A,115:A,125:A,140:A"
& $exe run "F:\GTA6\GTA6.Xbox.v1.0.2.1-CYBERLEEK"
```

## Armadilhas

- **Nunca rodar XCL e victory-fork/WARP ao mesmo tempo**: um run XCL com WARP do fork ativo em paralelo deu `device removed 0x887a0007` ("outro aplicativo enviou comandos inválidos") aos 172 s; repetindo com a GPU exclusiva, 25 fps e sem device removed aos 162 s. Sempre conferir `Get-Process victory,xcl` antes de lançar.
- Não faça push para `ivall/xcl`; trabalho é local até o usuário pedir.
- `XCL_DXIL_CACHE` precisa apontar para o mesmo diretório da pré-compilação.
- Tela verde `0x008700` = GPU view da memória CPU sem mirror (corrigido no upstream).
- Não reativar AS→MS fallback: tela preta.
- O pré-compilador terminou; `target\release\xcl.exe` está livre, mas o desenvolvimento usa `CARGO_TARGET_DIR`.
- Há dois processos: host `xcl.exe` (sem janela) e `game_scarlett_master.exe` (com a janela "XCL - Grand Theft Auto VI"). Monitore o PID do jogo. **Para input real com foco, procurar a janela em `game_scarlett_master`, não em `xcl`** (`%TEMP%\opencode\auto_input.ps1`). `XCL_PRESS_AT` (gamepad do HLE) não avança as telas de primeiro boot; teste teclado real focado.
- `title_memory_mb=4096` faz o jogo **pular a cutscene inicial**; manter padrão (8192/"auto").
- Com VRAM compartilhada, o processo pode ficar com as 105 threads suspensas (crash handler escrevendo dump de 26 MB) — parecia deadlock, mas é dump em andamento.
- O contador de `soft assert ... continued` é logado só em potências de 2; um número como 32768 em 0,4 s indica **loop**, não 32768 asserts distintos.

## References

- `context/architecture.md` — runtime, memória/GPU views, BCPack, VRAM, PSO stand-in, abort/assert, lacunas.
- `context/build-and-test.md` — comandos, baseline 699/0/21, matriz de testes, env vars, cache.
- `context/next-steps.md` — testes a fazer, causas/correções conhecidas e problemas a investigar.
- Skill irmã `victory-fork-dev` — referência de correções (textura/pipeline) e estado do fork; `F:\GTAVI-Build\gta5700\repos\victory-fork\rodar.ps1` abre o jogo.
- `context/roadmap-6h.md` — guia de estudo/teste/melhorias das próximas 6 horas, com fontes de pesquisa e critérios de sucesso.

# Build, teste, execução e cache — estado atual (24 Nov 2026)

## Comandos

```powershell
$env:Path = "$env:USERPROFILE\.cargo\bin;$env:Path"
$env:CARGO_TARGET_DIR="F:\GTAVI-Build\gta5700\build\xcl-target"
cargo build --release --workspace
cargo test -p xcl --lib -- --test-threads=4
```

Rodar com `workdir` = `F:\GTAVI-Build\gta5700\repos\xcl` (o cwd padrão do opencode não tem `Cargo.toml`).

**Build atual**: OK (incremental ~1,2 s), 2 warnings não-bloqueantes:
- `Atomic::<u32>::fetch_update` deprecado em `hle/xgameruntime/mod.rs:547` → `try_update`.
- `const COOL` não usado em `hle/d3d12x/vram.rs:195`.

**Testes atuais**: **699 passed, 0 failed, 21 ignored** (19,33 s). Antes do merge eram 626/1/21 e, na revisão anterior desta skill, 695/4/21 — as 4 falhas (`gpu_nonfused_mad_and_mac_prevent_fma_contraction`, `title_textures::placed_title_textures_share_vram_and_preserve_pixels`, `title_textures::transitions_apply_when_their_list_is_submitted`, `vram::d32s8_targets_take_their_own_size_of_video_memory`) não se reproduzem mais com as mudanças locais atuais.

Se o binário estiver preso por processo em execução: parar `xcl.exe`/`game_scarlett_master.exe` antes de compilar.

## Cache de shaders

Concluído: `F:\GTAVI-Build\gta5700\cache\xcl-dxil`, 4,18 GB / 28.853 arquivos. Log `%TEMP%\opencode\precompile-full.txt`, marcador `.done`.
Resultado final: `compiled 22333, already cached 13238, errors 19, skipped (GS) 390, time 7879 s`.
Erros: 19 validações DXC de mesh shaders `ms_6_6` (DXC 1.7 da SDK 26100).
Upstream recomenda DXC 1.8+ via `XCL_DXC_DIR`; não instalado.
Driver PSO blobs: upstream salva em disco ("driver blobs kept on disk"), mas o log reporta `0 KB` nesta máquina — eficácia duvidosa.

## Matriz de testes a serem feitos

| # | Teste | Como validar | Estado |
|---|---|---|---|
| 1 | Build release | `cargo build --release --workspace` | OK |
| 2 | Unit tests | `cargo test -p xcl --lib -- --test-threads=4`, baseline 699/0/21 | OK |
| 3 | Boot + menus | Rodar config abaixo; `frames presented > 0`, texto PT-BR nos avisos, clique manual | OK em runs anteriores |
| 4 | **PSO wait** | `XCL_PSO_WAIT_MS` unset trava no spinner; `100` gera ~30 fps e 9600 frames. Checar `still being built (N so far)` | Confirmado em 24/11 |
| 5 | **Assert isolado** | `soft assert (int3) ... (RVA 0x2215c40)` continuado em 429 s e 477 s | Confirmado |
| 6 | **Assert em burst** | Run > 9 min; ver se `RVA 0x2474606` dispara 1→32768 em <1 s | **Falha conhecida** |
| 7 | Device removed / VRAM | Run longo; `CreatePlacedResourceX ... 0x887a0005`, `device removed 0x887a0006` | **Falha conhecida** |
| 8 | Visual do mundo | `XCL_FRAME_DUMP=120`; frame 5300 com 518034/518400 não-pretos, cores variadas | Parcial (imagem real, falta conferir legendas) |
| 9 | Indirect draws | `console code so far`: 512 `ExecuteIndirect` skipped, 4 PC pipelines | Investigar |
| 10 | Write-watch / perf | `sparse windows timing:`; 90743 ms em 2,8 M páginas sujas | Investigar |
| 11 | Áudio | `XCL_TRACE_AUDIO=1`; `xapu ... 0 errors`, `non-zero samples`, saída audível | Parcial (Bink fora do HLE) |
| 12 | Regressão de crash handler | Confirmar que `0xc0000005 ... code bytes:` aparece com o patch local de `eh.rs` | OK |

## Executar

Configuração validada no run do mundo (ver SKILL.md): `XCL_SHARED_TEXTURE_PAGES=1`, `XCL_NO_FATAL=1`, `XCL_CONTINUE_ASSERTS=1`, `XCL_SAFE_READ=1`, `XCL_PSO_WAIT_MS=100`, `XCL_FRAME_DUMP=120`, `XCL_PRESS_AT`, `XCL_NETWORK=0`, `XCL_DXIL_CACHE=F:\GTAVI-Build\gta5700\cache\xcl-dxil`.

Inputs sintéticos (`keybd_event`, SendKeys) **não avançam** os menus de avisos; pedir clique do usuário.

## Variáveis (incluindo locais novas)

| Variável | Uso |
|---|---|
| `XCL_NETWORK=0` | sempre; bloqueia crash reporter (db1ddb2 inclui hooks de sistema) |
| `XCL_DXIL_CACHE` | `F:\GTAVI-Build\gta5700\cache\xcl-dxil` |
| `XCL_DATA_ROOT` | raiz de saves/logs/overlay |
| `XCL_PRESS_AT` | `s:BUTTON,…` (A = Space/Enter) |
| `XCL_FRAME_DUMP` | dump a cada N presents (60–120 para vídeos curtos) |
| `XCL_TRACE_FRAME=1` | estado de draw/present/textura |
| `XCL_TRACE_AUDIO=1` | DRC/pico de amostra |
| `XCL_ACTIVITY=1` | contadores de submits/draws/tempos |
| `XCL_PROFILE=1` | tempo de submit: total/refresh/replay/tail e hash/detile/gpu-wait (local) |
| `XCL_NO_FATAL=1` | neutra 7 aborts deliberados (local) |
| `XCL_CONTINUE_ASSERTS=1` | VEH pula `int3` e continua; log amostrado em potências de 2 (local) |
| `XCL_SAFE_READ=1` | força `VirtualQuery` em leituras de textura (local) |
| `XCL_SHARED_TEXTURE_PAGES=1` | `place_pages` para texturas sampled (local; economiza VRAM) |
| `XCL_SPARSE_VRAM_MB` | cap de pools/arenas (local; não resolveu OOM) |
| `XCL_VRAM_IDLE_MS` | tempo de evicção de arenas/pools (local) |
| `XCL_PSO_WAIT_MS` | **100 = correção confirmada**; unset/0 = ilimitado (trava no carregamento) |
| `XCL_TEXTURE_RECHECK` | **default OFF (opt-in `=1`)**: port do fork `a803c87` que reexamina texturas enviadas. A/B mostrou que **com o recheck ligado a intro da Rockstar não renderiza** (tela escura); desligado, a águia/murais aparecem. Não usar até depurar. |
| `XCL_DXC_DIR` | DXC 1.8+ (não configurado) |

## Monitoramento

```powershell
$p = Get-Process -Id <PID_DO_JOGO>
$p.MainWindowTitle                       # FPS e contador de pipelines
Get-Content "$root\logs\xcl.log" -Tail 8
Select-String "$root\logs\xcl.log" -Pattern 'unhandled|soft assert|no video memory|0x887a0005|device removed|0x8007000e|frames presented|still being built|ExecuteIndirect.*skipped|sparse windows timing'
$j = Get-Content "$root\logs\report.json" -Raw | ConvertFrom-Json
$j.d3d12.presents; $j.d3d12.dxil_cache; $j.audio.xapu_packets_decoded
```

Há dois processos: host `xcl.exe` (sem janela) e jogo `game_scarlett_master.exe` (com janela). Monitore o PID do jogo.

Run de referência: `%TEMP%\opencode\xcl-pso100\RockstarGamesInc.28872443D20A7\logs\`.

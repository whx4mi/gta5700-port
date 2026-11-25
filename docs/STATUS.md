# Estado t�cnico e medi��es (25 Nov 2026)

## Corre��es validadas no XCL

| Sintoma | Causa | Corre��o | Medi��o |
|---|---|---|---|
| Intro da Rockstar n�o renderizava (tela escura) | Port do recheck de texturas do fork | `XCL_TEXTURE_RECHECK` **desligado por padr�o** (opt-in `=1`) | A/B: OFF ? �guia/murais/logo R* aparecem; ON ? tela escura |
| `chunk scan` 8270 ms em 401 s (loading a 2 fps) | Hash de 64 B por p�gina (4� mais chamadas) | `CHUNK` 64?256 B em `sparse.rs` | **8270 ms ? 86 ms** com 1,55 M p�ginas sujas |
| Linhas em pintura/vidro/pele | Helper lanes no `rla`/`v_readfirstlane` | `IsHelperLane()` ? helper l� seu pr�prio valor | Build/testes OK |
| Buffer fixo de 16384 entradas no `GetWriteWatch` | ~90 chamadas ao kernel por faixa com 1,5 M p�ginas | Buffer dimensionado para a maior faixa | Build/testes OK; medi��o em loading pendente |

Baseline de testes: **695 passed / 4 failed / 21 ignored**. As 4 falhas (FMA, 3 de VRAM) s�o flaky e dependentes do estado do driver/GPU; j� apareceram como 699/0/21 na mesma �rvore.

## Corre��es/estado no victory-fork

- Com `VICTORY_TEXTURE_BUDGET_MB=1024`, `VICTORY_CARD_BUFFERS_MIN_MB=1`, `VICTORY_CARD_BUFFERS_MB=256`, `VICTORY_RELAY=2`: **sem device removed**, VRAM 4984 MB de 7384 MB, commit 18,7 GB. Chegou ao mundo (frame ~8677) a 1,1 fps.
- Motivo da tela escura no mundo: pipelines mesh recusadas em massa na RX 5700.
- Fallback de mesh portado no branch `mesh-fallback`: build diag sem warnings; testes de shader e cache passam; first WARP run panicked em `draw.rs:906` (corrigido com `Option::flatten()`); run seguinte sem panic, 6�13 fps, mas preso nos Avisos Legais � fallback ainda n�o exercitado.
- WARP com resolu��o reduzida (`VICTORY_DISPLAY_MODE=1280x720`) e `VICTORY_SHADER_LOOP_LIMIT=64`: 30 fps, mas imagem preta (loops truncados). `4096` tamb�m preto. N�o usar.
- `--console series-s` em hardware: tela preta. N�o usar.

## Descoberta importante: cache frio vs cache quente (25 Nov)

- Capturas com caches novos (`victory-cache-mesh`, `-nocaps`, `-warpvalid`) mostraram **intro toda escura**.
- Com o cache aquecido (`runs\victory-cache`, 0,61 GB, gerado pelos primeiros runs) a intro e as ilustrações com personagens renderizam corretamente ( murais, flamingo, personagens).
- Ou seja: a "instabilidade da tela inicial" era **pipeline/shader cache frio**, não o fallback de mesh nem os caps de VRAM. Usar sempre `VICTORY_CACHE_DIR=runs\victory-cache`.
- `VICTORY_TEXTURE_BUDGET_MB=1024` e afins não afetam a intro; são só para estabilidade do mundo.

## Port de `press.txt` do ScarlettPort

- `VICTORY_PRESS_FILE=<path>`: escrever `A`, `DOWN DOWN A` etc. no arquivo pressiona as teclas em sequência (150 ms por botão, 1,5 s para sticks) e esvazia o arquivo. Portado de `refs\ScarlettPort\runtime\victory\crates\victory-hle\src\modules\gameinput.rs` para `repos\victory-fork`. Build diag OK.
- Uso: defina `VICTORY_PRESS_FILE` e escreva os botantes no arquivo enquanto o jogo roda — resolve a navegação dos Avisos Legais/Termos sem depender de timing cego.

## Falhas conhecidas

- `0x887a0004` / `0x887a0001` no fork sem caps de VRAM: SM6.6/mesh n�o suportado na RDNA1.
- `0x887a0007` numa execu��o XCL com WARP do fork simult�neo: dois processos de GPU.
- `device removed 0x887a0006` no XCL ao carregar o mapa com VRAM ~9 GB.
- Burst de soft assert `int3` em `0x2474606` (flag `0x14a943999` nunca setada) ? device removed em runs longos.
- `VICTORY_SHADER_LOOP_LIMIT` qualquer valor baixo apaga a imagem.
- `VICTORY_PIPELINE_WAIT_MS=24` descarta os draws da intro.
- OOM do OpenCode: resolvido com pagefile em E: (commit ~96 GB).

## Regra de ouro

Nunca rodar XCL e victory-fork/WARP ao mesmo tempo. Matar watchdogs �rf�os antes de cada run. Usar watchdog sempre.

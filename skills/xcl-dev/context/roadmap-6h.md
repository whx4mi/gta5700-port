# Guia de estudo, teste e melhorias � pr�ximas 6 horas (25 Nov 2026)

**Base:** XCL local (`F:\GTAVI-Build\gta5700\repos\xcl`) � � a �nica que roda nesta RX 5700 (RDNA1, sem mesh shaders / SM6.6). Os demais repos s�o refer�ncia de corre��es pontuais, n�o bases substitutas.

**Regra de ouro:** nunca rodar dois processos de GPU ao mesmo tempo (XCL + victory/WARP). Conferir `Get-Process -Name xcl,victory,game_scarlett_master` e matar watchdogs �rf�os antes de lan�ar. Watchdog: `%TEMP%\opencode\vram_watchdog.ps1`.

## Estado atual (ponto de partida)

- XCL local: 16 arquivos modificados. O reexame de texturas portado do fork (`a803c87`) est� em `title_textures.rs` mas **desligado por padr�o** (`XCL_TEXTURE_RECHECK=1` para opt-in). Build OK. Testes 695/4/21 � flaky por estado do driver/GPU (baseline 699/0/21 na mesma �rvore).
- **Bloco 2 conclu�do (A/B)**: com recheck ON (`F:\GTAVI-Build\gta5700\runs\XCL-RECHECK`) a intro **n�o renderizou** (tela escura, nenhum `changed after upload`); com `XCL_TEXTURE_RECHECK=0` (`F:\GTAVI-Build\gta5700\runs\XCL-AB-OFF`) a intro renderizou **perfeitamente**, incluindo a �guia com asas na bandeira dos EUA (frame 380), murais (500/620), logo R* (740). Causa do bug da intro do usu�rio: **o pr�prio recheck**; corrigido ao deslig�-lo.
- Fork `F:\GTAVI-Build\gta5700\repos\victory-fork` em `a803c87`; `ztrx` � s� fork dele + README; `OpenScarlett`, `ScarlettX`, `Xbox2Windows` clonados em `F:\GTAVI-Build\gta5700\refs\`.
- Pagefile em E: ativo ? commit limit 95,9 GB (OOM do OpenCode mitigado).
- Descoberta nova: `ThatRecompDev/five-star-recomp` tem mesh fallback maduro (`MeshComputeFallback`: compute grava records + triangle list + args ? `ExecuteIndirect DrawIndexed`), aceito em AMD iGPU sem mesh shaders; �til se o port de mesh voltar a ser prioridade.

## Bloco 0 � 0:00�0:30 � Consolida��o e seguran�a

1. Confirmar GPU saud�vel (`Get-PnpDevice -Class Display`) e nenhum processo de jogo/watchdog ativo.
2. Verificar resultado do run `F:\GTAVI-Build\gta5700\runs\XCL-RECHECK`: presen�a de `changed after upload`, frames e `frames presented`.
3. Backup do estado atual do XCL (repo + skills) em `F:\GTAVI-Build\gta5700\backups\`.
4. Registrar no skill o que mudou (recheck, pagefile, regra de processos).

## Bloco 1 � 0:30�1:30 � Pesquisa dirigida (fontes j� localizadas)

Objetivo: entender o que o jogo manda e onde otimizar, com artigos/papers:

- **Dynamic resources / bindless**: HLSL SM6.6 Dynamic Resources (Microsoft) e Clang HLSL Dynamic Resources. Explica por que a RX 5700 recusa root signatures 1.1 e como gerar uma variante SM5.1/descriptor-table se necess�rio.
- **Descritores**: "Descriptors are hard" (Jason Ekstrand) e "Bindless rendering setup" (Traverse Research).
- **Textura/streaming**: SamplerFeedbackStreaming (GameTechDev), DirectStorage optimizing load-time and streaming (AMD/GDC 2023), Tiled Resources (Microsoft). Foco: batching, tiles sob demanda, evic��o � guiar o custo de write-watch do XCL.
- **PSO/prewarm**: D3D12 pipeline state cache sample (Microsoft), Advanced API Performance: PSO (NVIDIA), Avoiding runtime shader compilation (Unreal). Comparar com `XCL_PSO_WAIT_MS` e o commit `d279e69` do fork.
- **Mesh fallback**: docs/shader-translation.md do five-star-recomp � contrato `msfb_cs/msfb_vs`, variantes wave32/wave64, `--mesh-fallbacks`.

Entrega: um par�grafo por fonte com o que aplicar ao XCL e em qual arquivo.

## Bloco 2 � 1:30�2:30 � Validar o recheck de textura

1. Se o log n�o mostrou `changed after upload`, baixar `RECHECK_SECONDS` inicial para 0.05 s e ampliar a janela de log; rodar 150 s com `XCL_FRAME_DUMP=60`.
2. Comparar frames da fase de loading (arte com personagens) com e sem `XCL_TEXTURE_RECHECK=0`.
3. Crit�rio: aparecer arte de loading colorida em vez de tela escura/spinner.
4. Se funcionar: manter default on e documentar. Se n�o: investigar se a arte � Bink (v�deo) em vez de textura e seguir para o Bloco 3.

## Bloco 3 � 2:30�3:30 � Otimizar submit e VRAM (peso)

1. Medir `sparse windows timing:` do run atual; meta: reduzir write-watch (antes 77 s em 133 s = 58%).
2. Testar `XCL_VRAM_IDLE_MS` padr�o (2 s) vs 500 ms � 500 ms travou antes; n�o usar sem monitor.
3. Manter `XCL_SHARED_TEXTURE_PAGES=1`; `XCL_SPARSE_VRAM_MB` sem cap (cap travou).
4. Preload: garantir `F:\GTAVI-Build\gta5700\cache\xcl-dxil` quente e driver blobs reaproveitados; medir tempo de PSO (`driver blobs kept`).
5. Crit�rio: >=24 fps sustentado no loading sem device removed.

## Bloco 4 � 3:30�4:30 � Portar o peda�o da cutscene do fork

1. O que o fork tem que o XCL n�o tem: pipeline prewarm (`d279e69`) e recheck (`a803c87`, j� portado). O XCL j� tem stand-in e `XCL_PSO_WAIT_MS`; portar s� o que faltar (ex.: registrar pipelines pendentes para a pr�xima launch).
2. Checar se a intro do XCL usa Bink com YUV->RGB e se h� drops de shader (`translation_errors`); corrigir apenas os que aparecem na intro.
3. Crit�rio: intro sem tremula��o e sem texturas transparentes no XCL.

## Bloco 5 � 4:30�5:15 � Revis�o peri�dica dos repos

Checklist: `xcl-dev/xcl`, `Marvisak/VIctory`, `fxceless/victory-fork`, `Zettrox/ztrx`, `OpenCommunityEdition/OpenScarlett`, `Freminet/ScarlettX`, `R0Y00/Xbox2Windows`, `ThatRecompDev/five-star-recomp`, `Belobug/VIctory-Belobug-AMD` (clonado em `F:\GTAVI-Build\gta5700\refs\`).
- **Belobug AMD** (`4cbe9d7`): mundo iluminado com texturas/pessoas/ve�culos/�gua/c�u a 3�5 fps em RX 9070 XT RDNA4 16 GB. Sem fallback RDNA1/SM6.5 � n�o substitui o XCL nesta m�quina. Ideias port�veis: `VICTORY_HOIST_CLAMPS` (clamps de indirect draw em lote: 225?80 ms de GPU/frame), quad reads com helper lanes no PS, `v_readfirstlane` de helper, dobro de slots de descritores, reuso de cache de shader entre builds (`VICTORY_SHADER_BUILD_OF`), isolamento de pipelines que travam o compilador AMD.
Para cada commit novo: classificar como (a) port�vel para XCL, (b) refer�ncia apenas, (c) irrelevante. Registrar no skill.

## Bloco 6 � 5:15�6:00 � Valida��o ponta a ponta e fechamento

1. Run final com data root com saves (pula primeiro-boot), config enxuta, watchdog, captura 120.
2. Validar: boot ? intro Rockstar ? arte de loading com personagens ? menu ? pr�logo.
3. Medir FPS, VRAM, `uploads left out`, `changed after upload`, `device removed`.
4. Atualizar skills `xcl-dev` e `victory-fork-dev` com resultados e pr�xima itera��o.

## Prioridades que podem mudar

- Bloco 2 fechado: recheck � prejudicial, desligado. Prioridade agora � o Bloco 6 (valida��o completa) e o Bloco 3 (peso/VRAM) para a arte de loading; Bloco 4 s� depois.
- **Bloqueio identificado (25 Nov)**: mesmo com a intro correta, o jogo n�o passa das telas de primeiro boot (acessibilidade/brilho/termos). Reutilizar o data root n�o pulou a configura��o inicial. `XCL_PRESS_AT` com A a cada 10�15 s n�o avan�a; A a cada 3 s (`F:\GTAVI-Build\gta5700\runs\XCL-INPUT-DENSE`) tamb�m n�o (250 s, 920 frames, preso na intro/primeiro-boot). Pr�xima via em teste: driver de **teclado real focado** ( `%TEMP%\opencode\auto_input.ps1`, Enter a cada 4 s com `SetForegroundWindow`), run `F:\GTAVI-Build\gta5700\runs\XCL-REALINPUT`. Hip�tese: essas telas leem teclado, n�o o gamepad do HLE. O driver em background morreu ap�s 1 tap quando a chamada que o lan�ou terminou; teste mantendo a chamada viva (`F:\GTAVI-Build\gta5700\runs\XCL-REALINPUT3` � ainda 1 tap, `SetForegroundWindow` travava); removido o `SetForegroundWindow` do driver (`F:\GTAVI-Build\gta5700\runs\XCL-REALINPUT4`: 14 taps, mas jogo a ~2 fps na intro � taps de 90 ms passam entre os polls). Teste atual: Enter com **hold de 1,2 s** a cada 6 s (`F:\GTAVI-Build\gta5700\runs\XCL-LONGINPUT`). Enter longo tamb�m n�o adiantou (intro a 2 fps). **S� A** a cada 15 s at� 410 s (`F:\GTAVI-Build\gta5700\runs\XCL-ALLA-LONG`) tamb�m n�o passou: 1030 frames em 430 s (~3 fps), terminou nas telas de acessibilidade. Conclus�o: com o jogo a 2�3 fps nessa fase, nenhum input automatizado (gamepad HLE ou teclado) avan�a de forma confi�vel. Pr�ximo passo exige o usu�rio presente: navegar manualmente (Enter/clique) enquanto capturamos frames.
- Write-watch no run `F:\GTAVI-Build\gta5700\runs\XCL-FULL-OK`: 63,7 s em 401 s (16%), mas com `chunk scan 8270 ms` � o hash de 64 B por p�gina dominava. **Otimiza��o aplicada**: `CHUNK` de 64 ? 256 B com hash dobrado (`sparse.rs`), 4� menos hashes; testes ajustados e passam (baseline 695/4/21). Medido em `F:\GTAVI-Build\gta5700\runs\XCL-CHUNK256`: **chunk scan 8270 ms ? 86 ms** (com 1,55 M p�ginas sujas) � confirma��o clara. Segundo ajuste testado (`WATCH_GAP` 64 ? 256 MiB) ficou inconclusivo � jogo parou de submeter ap�s ~15 s; revertido para 64 MiB. **Manter a otimiza��o de CHUNK=256.** Terceiro ajuste (port do journal do fork): redimensionar o buffer de resposta do `GetWriteWatch` para a maior faixa (1 chamada por faixa em vez de ~90 com buffer fixo de 16 K entradas), em `refresh_once`; build/testes OK, medindo em `F:\GTAVI-Build\gta5700\runs\XCL-WATCHBUF`.
- Usu�rio encerrou o run manual (`F:\GTAVI-Build\gta5700\runs\XCL-MANUAL`) por 2 fps na intro; navega��o manual avan�ou at� frame 3260. Pedido atual: trazer a estabilidade do fork para o XCL � primeira via foi o custo de chunk scan.
- Se aparecer `device removed`: parar e checar drivers/processos antes de qualquer teste.
- Se o OpenCode der OOM: aumentar pagefile/fechar apps; n�o culpar o jogo.
- Se um repo novo trouxer mesh fallback pronto para o XCL (five-star-recomp), reavaliar o Bloco 4.

- Script manual para o usu�rio: `F:\GTAVI-Build\gta5700\tools\rodar-xcl.ps1` (`-Nome`, `-Segundos`, `-FrameDump`, `-SemWatchdog`). Config j� validada; watchdog e root por chamada.
- **Reordena��o de relev�ncia (25 Nov)**: hoist clamps s� afeta indirect draws **no mundo**; antes disso � preciso chegar l�. Ordem real: (1) validar `CHUNK=256` + buffer do `GetWriteWatch` num run at� o mundo (run `F:\GTAVI-Build\gta5700\runs\XCL-VALIDA`, navega��o manual), (2) hoist clamps, (3) quad reads/descritores.

## Crit�rios de sucesso desta sessao

- [ ] Intro da Rockstar est�vel no XCL (sem tremula��o/transpar�ncia).
- [ ] Arte de loading com personagens vis�vel antes do menu.
- [ ] FPS >= 24 no carregamento, sem device removed.
- [ ] Guia e descobertas registrados nas skills.

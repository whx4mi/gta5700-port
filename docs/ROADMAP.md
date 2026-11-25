# Melhorias e testes prioritizados

Ordem revisada em 25 Nov: o hoist clamps e o quad-read só importam **depois** que o mundo renderiza. Antes disso, validar o fallback de mesh e o caminho até o loading.

## 1. Validar o fallback de mesh (mais relevante agora)

- Branch `mesh-fallback` do fork está pronto em código; build diag e testes passam.
- Obter um run que **chegue ao mundo** com `VICTORY_MESH_FALLBACK=1`:
  - WARP + watchdog primeiro (seguro), com navegação manual dos Avisos Legais/Termos (o `VICTORY_KEYS` automático não passa);
  - captura de frames no mundo para confirmar que as malhas aparecem;
  - só então RX 5700 com caps de VRAM.
- Atenção: linking VS↔PS, root params ≤ 64, argumentos mesh nos roots gráficos.

## 2. Fazer a arte de loading aparecer

- No fork/WARP as ilustrações com personagens aparecem; no XCL não.
- Comparar com `VICTORY_AMBIENT`, `VICTORY_MESH_TEST` e os diffs de textura do fork (`a803c87`).
- O recheck de textura portado para o XCL é prejudicial (apaga a intro) — manter desligado.

## 3. Navegação das telas de primeiro boot

- `XCL_PRESS_AT` (gamepad HLE), `VICTORY_KEYS` e teclado real não avançam de forma confiável a 2–6 fps.
- Scripts: `%TEMP%\opencode\auto_input.ps1`, `%TEMP%\opencode\send_keys.ps1`.
- Curto prazo: navegação manual pelo usuário com captura de frames.

## 4. Desempenho no mundo (após item 1)

- Portar `VICTORY_HOIST_CLAMPS` do fork Belobug (225→80 ms de GPU/frame).
- `VICTORY_CARD_REHASH_RUNS=1`, `VICTORY_JOIN_COPIES=1`, `VICTORY_SHADER_BUILD_OF`.
- Meta: >16 fps estáveis no carregamento; ideal ≥24.

## 5. Estabilidade em runs longos

- Burst de `int3` em `0x2474606`: achar quem seta a flag `0x14a943999`.
- VRAM no carregamento do mapa (device removed ~9 GB) — caps do fork já ajudam.
- Rodar >9 min com watchdog e checar `crashes.log`.

## Revisão periódica dos repos

`git fetch` semanal em: xcl-dev/xcl, Marvisak/VIctory, fxceless/victory-fork, Zettrox/ztrx, OpenCommunityEdition/OpenScarlett, Freminet/ScarlettX, R0Y00/Xbox2Windows, ThatRecompDev/five-star-recomp, Belobug/VIctory-Belobug-AMD. Classificar commits como portáveis / referência / irrelevantes e registrar em `REPOSITORIOS.md`.

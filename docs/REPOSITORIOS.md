# Repositórios analisados e o que foi reaproveitado

Todos os repos abaixo foram clonados/baixados e lidos. Nenhum código foi copiado sem entendimento; quando algo foi portado, foi reescrito para a arquitetura do projeto destino.

| Repo | Papel | O que veio dele para o gta5700 |
|---|---|---|
| `ivall/xcl` → `xcl-dev/xcl` | Base original, trabalho desde o início | Fallback de mesh para GPU sem mesh shaders (compute grava records → VS pass-through + `DrawInstanced`); RE dos 7 aborts e dos asserts `int3`; `XCL_NO_FATAL`, `XCL_CONTINUE_ASSERTS`, `XCL_SAFE_READ`; stand-in de PSO em build; `XCL_PSO_WAIT_MS`; budget/evicção de VRAM; BCPack; hash/detile in-place; diagnóstico `XCL_PROFILE`/`XCL_TRACE_FRAME`. Repo local: `F:\GTAVI-Build\gta5700\repos\xcl` (16 arquivos modificados, backup em `F:\GTAVI-Build\gta5700\backups\xcl-*`). |
| `Marvisak/VIctory` | Upstream do fork | Journal de write-watch com "ask-ahead" (inspiração para redimensionar o buffer de `GetWriteWatch` no XCL); correção de texturas por índice reexaminadas após upload (`a803c87`); twins de depth, prebuild de pipeline, `VICTORY_RELAY`/`LATE_WAIT` como referência de estabilidade. Upstream local: `F:\GTAVI-Build\gta5700\refs\VIctory` (atualizado para `70564ae`). |
| `fxceless/victory-fork` | **Base atual** | Fork AMD/geral do VIctory; correção de textura `a803c87`, prewarm de pipeline `d279e69`, switches de velocidade, captura `VICTORY_CAPTURE`, input `VICTORY_KEYS`, build diag. Local: `F:\GTAVI-Build\gta5700\repos\victory-fork`. |
| `Zettrox/ztrx` | Referência | README e launcher de play; **sem correções de engine** a mais que o fork — não vale merge. Clonado em `F:\GTAVI-Build\gta5700\refs\ztrx`. |
| `Belobug/VIctory-Belobug-AMD` | Referência AMD avançada | Mundo iluminado com texturas/pessoas/veículos/água/céu a 3–5 fps em RX 9070 XT (RDNA4). Ideias portadas: helper lane em `v_readfirstlane` (`IsHelperLane()` → próprio valor) **aplicada no XCL**; quad reads com helper lanes (já existia no `rla` do XCL); `VICTORY_HOIST_CLAMPS` (225→80 ms de GPU/frame), dobro de slots de descritores, reuso de cache entre builds — candidatos a port. Clonado em `F:\GTAVI-Build\gta5700\refs\VIctory-Belobug-AMD`. |
| `ThatRecompDev/five-star-recomp` | Referência de fallback de mesh | Contrato `MeshComputeFallback`: CS `u0` records, `u1` triangle list, `u2` argumentos `DrawIndexed`; VS `t0` lê records; `DispatchMesh` → `Dispatch` + barreira + `ExecuteIndirect`. Base do plano em `F:\GTAVI-Build\gta5700\repos\victory-fork\docs\fork-mesh-fallback.md`. |
| `OpenCommunityEdition/OpenScarlett` | Referência | Clonado em `F:\GTAVI-Build\gta5700\refs\OpenScarlett` para estudo de HLE/formato; nada portado ainda. |
| `Freminet/ScarlettX` | Referência | Clonado em `F:\GTAVI-Build\gta5700\refs\ScarlettX`; nada portado ainda. |
| `R0Y00/Xbox2Windows` | Referência | Clonado em `F:\GTAVI-Build\gta5700\refs\Xbox2Windows`; confirmou soft asserts `int3` e `-ContinueAsserts`. |

## O que foi portado de fato

1. **Do XCL para o fork**: fallback de mesh (branch `mesh-fallback` de `F:\GTAVI-Build\gta5700\repos\victory-fork`) — record layout, VS pass-through, writer compute, PSOs e caminho `Dispatch` → barreira → `DrawInstanced`. Código pronto; validação em mundo pendente.
2. **Do fork/Belobug para o XCL**: reexame de texturas (`XCL_TEXTURE_RECHECK`, desligado — prejudicial), redimensionamento do buffer de `GetWriteWatch`, `CHUNK` 64→256 B, helper lane em `v_readfirstlane`, journal/ask-ahead como inspiração.
3. **Do XCL upstream (20cf130)**: correções AMD/MSAA e world-freeze 180 s disponíveis para merge (não aplicadas; repo local tem mudanças não commitadas).

## Licenças

XCL GPL-2.0; VIctory e forks GPL-3.0. Todo o trabalho é local; nenhum push foi feito.

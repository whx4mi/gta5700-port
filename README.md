# gta5700 — GTA VI na AMD RX 5700

Projeto de engenharia reversa/compatibilidade para rodar o **GTA VI (build Xbox Series, 1.0.2.1)** nativamente num PC com **AMD Radeon RX 5700 (RDNA1, 8 GB, sem mesh shaders)**.

## Objetivo

Boot → intro da Rockstar → arte de loading com personagens → menu/termos → cutscene inicial → prólogo, com estabilidade e o melhor FPS que esta GPU permitir.

## Hardware e limitação central

- Ryzen 5 5600G (6 núcleos), 32 GB RAM, RX 5700 RDNA1 8 GB (sem mesh shaders, sem SM6.6 completo).
- Pagefile adicional em `E:\pagefile.sys` (32–64 GB) → commit limit ~96 GB (evita OOM do OpenCode/jogo).
- A RX 5700 já teve dois crashes de driver; **nunca rodar dois processos de GPU ao mesmo tempo** (XCL + victory/WARP).

## Decisão de arquitetura (atual)

**Base: `victory-fork`** com (1) caps de VRAM que eliminaram o `device removed`, e (2) o fallback de mesh portado do XCL, que é o que falta para o mundo renderizar nesta placa. **XCL permanece como referência e banco de correções de compatibilidade** — é onde as descobertas de baixo nível (aborts/asserts, write-watch, helper lanes, VRAM) foram feitas.

Motivo: o fork com caps de VRAM já chegou ao mundo (frame ~8677) **sem device removed**; o XCL tem a intro perfeita e 30 fps em estágios anteriores, mas ainda não passa das telas de primeiro boot de forma confiável e não mostra a arte de loading.

## Estado resumido

| Item | Situação |
|---|---|
| Intro Rockstar (águia, murais, logo R*) | OK no XCL; tremida/bugada no fork em hardware, correta no fork com WARP |
| Boot/menus/termos | Renderizam; navegação automatizada ainda não confiável (2–6 fps nessa fase) |
| Loading com personagens | Visível no fork em WARP/frames; ausente no XCL (telas escuras) |
| Mundo/prólogo | Fork chega lá com caps de VRAM, mas escuro até o fallback de mesh ser ativado/validado |
| Fallback de mesh | Portado para o fork (branch `mesh-fallback`); build/testes OK; falta validação em mundo real |
| Estabilidade | Sem `device removed` com caps de VRAM; watchdog obrigatório |

## Como rodar

- Fork (base): `powershell -ExecutionPolicy Bypass -File F:\GTAVI-Build\gta5700\tools\rodar-fork.ps1 -Nome teste -Segundos 600`
  - `-Warp` para software (seguro), `-Keys '20:Enter'` para input automático.
- XCL (referência): `powershell -ExecutionPolicy Bypass -File F:\GTAVI-Build\gta5700\tools\rodar-xcl.ps1 -Nome teste -Segundos 600`
- Antes de cada run: checar `Get-Process -Name xcl,victory,game_scarlett_master` e matar watchdogs órfãos.

## Estrutura

- `docs/REPOSITORIOS.md` — repos analisados e o que foi reaproveitado de cada um.
- `docs/STATUS.md` — medições, falhas conhecidas e correções validadas.
- `docs/BUILD-E-TESTE.md` — comandos de build/teste, variáveis e watchdogs.
- `docs/ROADMAP.md` — prioridades e próximos testes.
- `docs/RETOMADA.md` — o que pode ser apagado e como retomar.
- `tools/` — cópias dos scripts de execução.
- `.gitignore` — exclui `runs/`, `cache/`, `build/` e `backups/` do futuro repo público.

## Localização dos projetos

- XCL (base antiga + mudanças locais): `F:\GTAVI-Build\gta5700\repos\xcl` — backup em `F:\GTAVI-Build\gta5700\backups\xcl-*`.
- Fork (base atual): `F:\GTAVI-Build\gta5700\repos\victory-fork` — branch `mesh-fallback`; `main` em `a803c87`; backup em `F:\GTAVI-Build\gta5700\backups\victory-fork-*`.
- Repos de referência: `F:\GTAVI-Build\gta5700\refs\` (VIctory, ztrx, OpenScarlett, ScarlettX, Xbox2Windows, ztrx, five-star-recomp, Belobug).
- Skills: `~/.config/opencode/skills/xcl-dev` e `victory-fork-dev`.

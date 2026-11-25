# Retomada e limpeza

## Posso apagar os arquivos de teste?

**Parcialmente.** Se apagar tudo que foi criado durante os testes fora desta pasta, **não** dá para retomar exatamente onde estamos. O essencial já foi trazido para `F:\GTAVI-Build\gta5700`:

| item | local atual | por que manter |
|---|---|---|
| Repo XCL (16 arquivos modificados) | `repos\xcl` | contém todo o trabalho local não commitado |
| Repo victory-fork (branch `mesh-fallback`) | `repos\victory-fork` | fallback de mesh portado, sem commit |
| Cache DXIL do XCL (4,4 GB) | `cache\xcl-dxil` | levou horas para gerar; sem ele o boot fica lento/quebrado |
| Binários compilados | `build\xcl-target`, `repos\victory-fork\target` | evitam recompilar |
| Caches do fork | `runs\victory-cache*` | pipelines/shaders já aquecidos |
| Backups antigos | `backups\` | segurança |
| Logs/frames de referência | `runs\XCL-*`, `runs\victory-capture*` | evidência das medições |
| Skills | `skills\` e `~/.config/opencode/skills` | instruções carregadas pelo opencode |

## O que pode ser apagado com segurança

- Diretórios de runs que não sejam de referência (em `runs\`), se precisar de espaço.
- Backups antigos em `backups\`, **depois** de confirmar que os repos atuais estão íntegros (`git status` e `git branch`).
- Arquivos temporários em `%TEMP%\opencode\` (scripts antigos de input/watchdog) — as cópias canônicas estão em `tools\`.

## Como retomar

1. Abrir o opencode nesta pasta (`F:\GTAVI-Build\gta5700`) — sessão já migrada.
2. Checar GPU/processos (ver `docs\BUILD-E-TESTE.md`).
3. Validar os repos:
   ```powershell
   git -C repos\xcl status --short
   git -C repos\victory-fork branch --show-current   # mesh-fallback
   ```
4. Rebuild se necessário (comandos em `docs\BUILD-E-TESTE.md`).
5. Rodar com `tools\rodar-fork.ps1` (base atual) ou `tools\rodar-xcl.ps1` (referência).

## Nota para o repositório GitHub

Quando o projeto estiver estável, publicar apenas código/documentação/scripts. `runs\`, `cache\`, `build\` e `backups\` devem ficar fora do repo (ver `.gitignore`).

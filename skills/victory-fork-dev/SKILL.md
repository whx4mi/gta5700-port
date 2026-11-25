---
name: victory-fork-dev
description: fxceless/victory-fork (GTA VI Xbox -> Windows) boot-to-prologue workflow - build diag, configure victory.toml, capture frames, VICTORY_KEYS input, VRAM/GPU watchdog. Use when working in F:\GTAVI-Build\gta5700\repos\victory-fork or when XCL's boot/loading art fails; prefer this fork over XCL for boot-to-world.
---

# victory-fork � boot at� o pr�logo

Fork de VIctory (GPL-3.0) focado em fazer o GTA VI renderizar o mundo. Substitui o XCL como base: o XCL n�o renderiza as imagens de loading com personagens e estourava a VRAM no loading; este fork j� mostra a intro completa e as ilustra��es de personagens.

- Repo: `F:\GTAVI-Build\gta5700\repos\victory-fork` (GitHub `fxceless/victory-fork`, HEAD `a803c87`). Backup anterior: `F:\GTAVI-Build\gta5700\backups\victory-fork-20261124-2026\`. `a803c87` corrige texturas escolhidas por �ndice reexaminadas ap�s upload (pele/cabelo sem cor).
- Refer�ncias extras clonadas em `F:\GTAVI-Build\gta5700\refs\`: `OpenScarlett`, `ScarlettX`, `Xbox2Windows`, `ztrx`. **`ztrx` (Zettrox) � s� `victory-fork@2119b0e` + README/launcher** (`90b529f`): n�o traz corre��es de engine a mais, apesar do README dizer que o pr�logo � jog�vel em RTX 3050. N�o vale merge de c�digo.
- **Atualiza��es em 24 Nov**: fork recebeu `9e49b8a` + `2119b0e` (mudan�a de lane-mask #11 removida e recolocada � cancelam-se). Upstream `Marvisak/VIctory` est� em `70564ae`, 28 commits � frente (`13bbd6f` texturas/twins, `6d021fe` custo de frame, `c47b529` prebuild de pipeline); merge direto gera dezenas de conflitos (`lower.rs`, `draw.rs`, `host.rs` etc.) � abandonado, backup feito. `xcl-dev/xcl` est� em `20cf130` (fix MSAA/AMD), mas tem mudan�as locais n�o commitadas.
- **Controle de resolu��o**: `VICTORY_DISPLAY_MODE=WxH` (padr�o `3840x2160`, aceita 640�7680 � 360�4320) em `crates/victory-gpu/src/dxgi.rs:222-241`; `--console series-s` reduz or�amento de mem�ria. N�o h� outro switch de escala. **Na RX 5700, `VICTORY_DISPLAY_MODE=1280x720 --console series-s` produz tela preta em todos os frames capturados** � n�o usar em hardware; testar s� com WARP.
- Upstream VIctory `70564ae` (status 24/11): mundo agora � desenhado e apresentado em RX 6700 XT (RDNA2), 1�2 s/frame, com superf�cies sem luz e some geometry faltando. Confirma que RDNA2+ � o requisito pr�tico. Merge do upstream no fork gera dezenas de conflitos em `lower.rs`/`draw.rs`/`host.rs` � abandonado; backup em `F:\GTAVI-Build\gta5700\backups\victory-fork-20261124-2026\`. Cherry-pick isolado de `13bbd6f` (texturas/twins) tamb�m conflita em `capture.rs`, `descriptor/host.rs`, `draw.rs`, `host.rs`, `resource.rs`, `texture.rs`.

## Port do fallback de mesh (em andamento, branch `mesh-fallback`)

Branch `mesh-fallback` criada de `main a803c87`, **sem commit/push**; `main` intacto. Estado:
- **Feito**: `mesh_fallback.rs` integrado (record layout, `passthrough_vs` que gera DXIL direto, `MeshFallback` em `HostShader`, cache key/VSH2). Writer compute em `lower.rs`/`shader.rs`; pipelines compute+gr�ficas em `pipeline/host.rs`. Build diag OK; testes de shader e cache passam; DXIL gerado valida no `dxil.dll` do SDK.
- **Parte D/E conclu�da**: `draw.rs::dispatch_mesh` agora, com `VICTORY_MESH_FALLBACK=1`, aloca/zera scratch UAV, bind duplo (compute writer + VS/PS), `Dispatch` ? UAV barrier ? `DrawInstanced`, e restaura estado. Root slot 63 virou root descriptor; visibilidade ALL no modo fallback. Build diag **sem warnings**, `cargo check victory-gpu --all-targets` OK, testes de shader/cache passam.
- **Valida��o WARP (25 Nov)**: primeira execu��o panicked em `draw.rs:906` (`expect` em pipeline compute sem writer objects); corrigido com `Option::flatten()`. Rebuild diag OK. Run WARP seguinte: **sem panic e sem device removed**, 6�13 fps, mas preso nos Avisos Legais (input autom�tico n�o passa os termos) � fallback **ainda n�o exercitado**. Pr�ximo: navega��o manual dos termos para chegar ao mundo, s� ent�o testar RX 5700 com caps de VRAM.
- Aten��o a linking VS?PS, contagem de root params = 64, e argumentos mesh em roots gr�ficos. `VICTORY_MESH_TEST=keep:` pode precisar de nome espec�fico para fallback. Fora de escopo: per-primitive outputs, meshes n�o-tri�ngulo, wave64/LDS.

## Estabilidade obtida com caps de VRAM (25 Nov)

`F:\GTAVI-Build\gta5700\tools\rodar-fork.ps1 -Nome base1` com `VICTORY_TEXTURE_BUDGET_MB=1024`, `VICTORY_CARD_BUFFERS_MIN_MB=1`, `VICTORY_CARD_BUFFERS_MB=256`, `VICTORY_RELAY=2`: **sem device removed**, VRAM 4984 MB de 7384 MB de budget, commit 18,7 GB. Chegou ao mundo (frame ~8677 / 1,1 fps), mas a imagem do mundo � escura com spinner: pipelines mesh recusadas em massa (RX 5700 sem mesh shaders / SM6.6). Ou seja: **estabilidade resolvida; falta a compatibilidade de mesh do XCL**. Script do usu�rio: `F:\GTAVI-Build\gta5700\tools\rodar-fork.ps1`.

## Caminhos avaliados para a RX 5700 (RDNA1)

- **WARP** (`VICTORY_D3D12_WARP=1`, build diag): renderiza a intro corretamente (�guia/murais), sem device removed, mas ~1 fps mesmo a `VICTORY_DISPLAY_MODE=1280x720` � gargalo � software, n�o pixel fill. Serve para valida��o visual, n�o para jogar.
- **Portar o fallback de mesh do XCL**: o XCL faz NGG/GS em dois passos (compute grava records em UAV ? `vs_6_6` pass-through l� o record e `DrawInstanced`), ~250�350 linhas em `emit.rs`, `pso.rs`, `runner.rs`, `real.rs`, `bind.rs`. O fork **n�o tem** esse intermedi�rio compute/record � a tradu��o � `ms_6_6` direta (`emit/lower.rs:3997-4063`, `draw.rs:1801`). Portar exigiria criar record layout + escritor, emitter VS, segundo tipo de PSO, hook de `DispatchMesh` e relaxar a recusa em `pipeline/host.rs:359` � centenas a milhares de linhas. N�o � cherry-pick.
- **Switches que s� omitem geometria**: `VICTORY_MESH_TEST=skip|scissor|keep:`, `VICTORY_SKIP_PIPELINES` � n�o renderizam a cena de outro modo.
- **Fluidez da intro**: `VICTORY_PIPELINE_WAIT_MS=24` **n�o resolve** � os draws da intro s�o omitidos e o jogo vai direto para o seletor de conta (frame 1000 mostra "SELETOR DE CONTA Y / PLAYER A" em fundo escuro). N�o usar durante a intro.
- **Conclus�o**: switches n�o consertam a intro na RX 5700. Caminhos restantes: (a) WARP para valida��o visual (~1 fps), (b) portar a arquitetura de dois passos do XCL para o fork (projeto grande), (c) GPU RDNA2+.
- REA est� em `C:\Users\Whxami\OneDrive\�rea de Trabalho\EngenhariaReversa\rea-main`, mas `GHIDRA_INSTALL_DIR` n�o est� configurado e n�o h� Ghidra instalado (Java 21 presente). S� vale a pena para decodificar um shader espec�fico.
- Backup do projeto anterior: `F:\GTAVI-Build\gta5700\backups\xcl-20261124-1836\` (repo xcl com mudan�as locais, skill `xcl-dev`, `opencode.jsonc`). N�o mexer sem pedido.
- Config: `F:\GTAVI-Build\gta5700\repos\victory-fork\victory.toml` � `game_dir=F:\GTA6\GTA6.Xbox.v1.0.2.1-CYBERLEEK`, `cache_dir=F:\GTAVI-Build\gta5700\runs\victory-cache`, `data_dir=F:\GTAVI-Build\gta5700\runs\victory-data`. Paths precisam ser **fora** do repo.
- Jogo: `F:\GTA6\GTA6.Xbox.v1.0.2.1-CYBERLEEK\game_scarlett_master.exe` (title id `7952987A`).
- Hardware: Ryzen 5 5600G (6 n�cleos), RX 5700 RDNA1 8 GB (sem mesh shaders!), 32 GB RAM. O fork foi desenvolvido em i7-14700KF + RTX 4070 12 GB � diferen�a de GPU importa.
- Cuidado: a RX 5700 j� crashou duas vezes com o XCL. Sempre usar watchdog e parar se aparecer `device removed`/`device lost`.

## Build

```powershell
$env:Path="$env:USERPROFILE\.cargo\bin;$env:Path"
cargo build --release -p victory      # ~1m22s, OK
cargo build --profile diag -p victory --features diagnostics   # ~1m16s, OK
F:\GTAVI-Build\gta5700\repos\victory-fork\target\release\victory.exe config   # valida victory.toml (rodar com workdir F:\GTAVI-Build\gta5700\repos\victory-fork)
```

Rust instalado 1.100.0; `rust-toolchain.toml` pede 1.98.1 (cargo compilou com o stable dispon�vel).

## Executar (build diag, captura e input)

```powershell
$env:VICTORY_CAPTURE="F:\GTAVI-Build\gta5700\runs\victory-capture"
$env:VICTORY_CAPTURE_FRAMES="20,100,400,600,800,1000,1200,1500,2000,2500,3000,3500,4000"
$env:VICTORY_KEYS="20:Enter"
Start-Process -FilePath "F:\GTAVI-Build\gta5700\repos\victory-fork\target\diag\victory.exe" `
  -ArgumentList @('run','F:\GTA6\GTA6.Xbox.v1.0.2.1-CYBERLEEK\game_scarlett_master.exe','--execute') `
  -WorkingDirectory "F:\GTAVI-Build\gta5700\repos\victory-fork"
```

- `VICTORY_CAPTURE`/`VICTORY_CAPTURE_FRAMES`/`VICTORY_KEYS` **s� existem no build `diag`** (release ignora).
- `VICTORY_KEYS=<s>:<key>,�` � o rel�gio come�a na **primeira consulta de input**, n�o no boot; pin�ar o tempo � impreciso. `Enter` = A do gamepad; `Back` = B; setas = D-pad.
- Capacidade de captura: `frame-<n>.png` + `frame-<n>-ops.txt` em `F:\GTAVI-Build\gta5700\runs\victory-capture`. Use contact sheet para revisar (script `%TEMP%\opencode\contact_sheet.ps1`).
- Watchdog: `%TEMP%\opencode\victory_watchdog.ps1 -Seconds 210` (mata `victory.exe` no timeout e se `crashes.log` mostrar falha de GPU).

## Valida��o j� feita (24 Nov 2026)

| Est�gio | Resultado |
|---|---|
| Boot / menu / title "Americas" | OK, 28�29 fps, 5�11 draws/frame. |
| Intro Rockstar (murais, flamingo, frutas, logo R*) | **Renderiza** (frames 600�800). |
| Ilustra��es com personagens (rapaz ao carro, mo�a com bolsa, casario) | **Renderizam** (frames 1000�3000) � era exatamente o que faltava no XCL. |
| Input `VICTORY_KEYS=20:Enter` | N�o avan�ou na primeira tentativa (parece ter ficado no title); o jogo entrou no mundo de qualquer forma. |
| Mundo/pr�logo | 267 draws/frame, mas **1 fps / 1172 ms por frame** na RX 5700 (fork reporta 2�4 fps em RTX 4070). |
| Estabilidade | Sem `device removed` e sem `crashes.log`; watchdog encerrou no timeout. GPU saud�vel. |
| Switches de velocidade (`VICTORY_RELAY=2`, `VICTORY_LATE_WAIT=3`, `VICTORY_WIDE_BACK=1`, `VICTORY_CARD_BUFFERS_MIN_MB=1`, `VICTORY_CARD_BUFFERS_MB=1024`) | Sem crash; ~30 fps no menu, pacotes `G:\pack\core_restofworld\*` abertos (loading do mundo iniciado). Log confirma `VICTORY_RELAY=1`/`VICTORY_LATE_WAIT=2` internos (valores s�o mapeados). |
| Captura densa 2000�2200 | Confirma as ilustra��es de personagens com fade entre elas � � a arte de loading que o XCL n�o mostrava. |
| Teste manual do usu�rio (release) | Intro bugada, cutscene travada com texturas transparentes, �udio ok, depois `device removed 0x887A0004` (feature/interface n�o suportada) + `0x887A0001` (chamada inv�lida) ao submeter 1 lista. GPU voltou ao normal. Causa prov�vel: RX 5700 RDNA1 sem mesh shaders/SM6.6. |
| Valida��o WARP (`VICTORY_D3D12_WARP=1`, debug+DRED) | **Sem device removed**; intro da �guia com asas em bandeira dos EUA renderiza corretamente (frame 400), assim como os murais (frame 512). Por�m ~1,6 fps e v�rios `host pipeline refused 0x80070057` / `shader does not translate ... converter shader.dxi` � draws caem. Confirma que o bug visual � de hardware, n�o da l�gica do fork. |

## Pr�ximos passos

1. Repetir com input mais tardio/mais denso (`VICTORY_KEYS=40:Enter,60:Enter`) e captura frame 2500+ para ver a transi��o title ? cutscene ? loading do mapa.
2. Testar switches de velocidade do fork: `VICTORY_RELAY=2`, `VICTORY_LATE_WAIT=3`, `VICTORY_WIDE_BACK=1`, `VICTORY_CARD_BUFFERS_MIN_MB=1`, `VICTORY_CARD_BUFFERS_MB=1024` (reduzem o atraso da GPU; README diz 240�350 ms/frame em RTX 4070).
3. Considerar `VICTORY_SHADER_OPT=0` isoladamente para medir o custo das otimiza��es; `VICTORY_D3D12_WARP=1` como �ltimo recurso (software, protege o driver).
4. Confirmar depois da cutscene: HUD, pause menu, input do pad � o README do fork diz que funcionam.
5. Problema conhecido do fork: luz em sombra preta (ray tracing em progresso, desligado); interior do primeiro c�modo �s vezes n�o desenha.

## Armadilhas

- Sempre watchdog: a RX 5700 j� crashou o driver duas vezes em runs do XCL com VRAM alta.
- N�o rodar XCL e victory ao mesmo tempo.
- O game exige v�rios GiB de commit; `F:\GTAVI-Build\gta5700\runs\victory-cache\images\victory-image-...dll` tem 120 MB (shaders/image), cache em E: (80 GB livres).
- `ivall/xcl` no GitHub agora redireciona para `xcl-dev/xcl`; repo local est� 0 commits atr�s, mas est� em backup � n�o � mais a base.

## Refer�ncias

- README do fork: `F:\GTAVI-Build\gta5700\repos\victory-fork\README.md` (status 2026-11-25 e lista de switches).
- `docs/fork-ray-tracing.md`, `docs/architecture.md` no repo.
- Backup XCL: `F:\GTAVI-Build\gta5700\backups\xcl-20261124-1836\`.

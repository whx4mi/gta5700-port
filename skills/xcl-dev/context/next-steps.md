# Pr�ximos passos � foco: estabilizar o mundo (24 Nov 2026)

## J� feito (marcos)

1. Upstream mesclado at� `47efa2e`: BCPack do VIctory, detile/hash in-place, write-watch otimizado, preload de PSOs, ASLR sem high-entropy, pool/arena fixes.
2. Corre��es locais preservadas em 16 arquivos: fallback VS/AS, `XMemIsToolsMemAvailable`, diagnostics, `XCL_PROFILE`, budget/evic��o de VRAM, stand-in 1�1, `XCL_SAFE_READ`, `XCL_NO_FATAL` (7 aborts), `XCL_CONTINUE_ASSERTS`, code bytes em `0xc0000005`.
3. Boot, menus, calibra��o, avisos legais, v�deo Bink e arte de loading renderizam; �udio Opus decodifica com 0 erros.
4. FPS do menu estabilizado em ~20�30 (antes 4�11).
5. RE do abort: 7 sites `mov dword [0], imm`; soft asserts `int3` confirmados (`0x2215c40`, `0x2474606`).
6. **Bloqueio do mundo resolvido**: `XCL_PSO_WAIT_MS=100` ? ~30 fps, 9600 frames em ~370 s, frames com imagem real.
7. Build e testes verificado hoje: **OK**, `699 passed / 0 failed / 21 ignored` (antes 695/4/21).
8. Skill `victory-boot` criada em `F:\GTAVI-Build\.opencode\skills\` e registrada em `~/.config/opencode/opencode.jsonc`.

---

## A. Causas de problemas conhecidas e corre��es

| Sintoma | Causa | Corre��o | Status |
|---|---|---|---|
| Spinner eterno, 0 fps no carregamento | `XCL_PSO_WAIT_MS` unset = espera ilimitada por PSOs de 20 s | `XCL_PSO_WAIT_MS=100`; binds usam stand-in e o jogo segue renderizando | **Resolvido** |
| `int3` encerra o jogo no texture streaming | Soft assert do retail (~10/s) | `XCL_CONTINUE_ASSERTS=1` (VEH avan�a RIP) | **Resolvido** para `0x2215c40` |
| `0xc0000005 write at 0x0` | Abort deliberado (`mov dword [0], imm`) | `XCL_NO_FATAL=1` neutraliza 7 sites | Resolvido |
| `read at 0xb96a20000` | Mirror stale de GPU view | `XCL_SAFE_READ=1` (`VirtualQuery`) | Resolvido |
| `CreatePlacedResourceX` E_OUTOFMEMORY | 8 GB insuficientes; sparse reserva 7,4 GB | `XCL_SHARED_TEXTURE_PAGES=1` + budget/evic��o | Parcial |
| Tela preta com minimap | Pipelines MS no slot AS traduzidas errado | Recusar os 40 `translation_errors` | Resolvido (n�o reativar AS?MS) |
| Menu a 4�11 fps | hash/detile fora do caminho quente | detile/hash in-place do upstream | Resolvido |
| Burst de `int3` em `0x2474606` ? device removed | Continuar RIP n�o estabiliza o site; reentr�ncia | Ainda sem corre��o | **Em investiga��o** |
| `90743 ms` de write-watch no mundo | 2,8 M p�ginas sujas; custo dominante no submit | Ainda sem corre��o | **Em investiga��o** |

Descartadas como causa do antigo travamento: GPU (0,7% de uso), compila��o DXC (0 falhas com cache quente), thermal throttling (3,9 GHz), bin�rio antigo, `title_memory_mb` (4096 pula a cutscene; 8192 � o correto).

---

## B. Testes a serem feitos

1. **Novo run longo com o build atual** (`xcl-pso100b`), mesma config do SKILL.md, para confirmar o relato do usu�rio de **sem crash at� 530 s** � o log dispon�vel termina em ~483 s. Pedir clique manual nos avisos.
2. **Acompanhar assinaturas-chave**: `frames presented`, `soft assert`, `still being built`, `ExecuteIndirect ... skipped`, `device removed`, `unhandled`, `xapu`, `sparse windows timing`.
3. **Comparar com o run de refer�ncia**: `presents`, `dxil_cache`, `xapu_packets_decoded` no `report.json`; esperado ?9600 frames, ~30 fps, 9003+ packets sem erros.
4. **Teste de regress�o do PSO wait**: rodar uma vez sem `XCL_PSO_WAIT_MS` para reproduzir a trava e uma com `100`; documentar que a diferen�a � o stand-in.
5. **Teste de estresse de assert**: monitorar se `0x2474606` volta a estourar ap�s 8 min; bytes j� desmontados com capstone � pr�ximo passo � achar quem escreve a flag `0x14a943999`, n�o mexer no VEH �s cegas.
6. **Inspe��o visual**: `XCL_FRAME_DUMP=120` em frames 5300�9600; verificar mundo, logos, legendas e cutscene (converter BMP com System.Drawing).
7. **�udio**: `XCL_TRACE_AUDIO=1`, confirmar `xapu_nonzero_samples` e sa�da aud�vel; Bink importa `XAudio2_9.dll` direto � considerar proxy/HLE ou captura WASAPI.
8. **Unit tests ap�s qualquer mudan�a**: `cargo build --release --workspace` + `cargo test -p xcl --lib -- --test-threads=4` (baseline 699/0/21).
9. **N�o commitar/pushar** sem pedido expl�cito.

Checklist r�pido de valida��o de cada mudan�a: build ? testes ? run com `XCL_NETWORK=0` e cache ? checar `unhandled`/`soft assert`/`frames presented`/`raw texels`/`report.audio`/`dxil_cache`.

---

## C. Problemas a investigar (priorizados)

### 1. Burst de soft assert em `0x2474606` (mais urgente)
- Em 480,56 s o contador foi de 1 para 32768 em ~0,4 s (o log s� amostra pot�ncias de 2). Continuar o RIP gera loop reentrante.
- **RE feita hoje (capstone 5.0.7, bytes do exe em `F:\GTA6\...\game_scarlett_master.exe`)**: a fun��o em `0x142474500` testa
  `0x142474521: cmp byte ptr [0x14a943999], 1` / `0x142474528: jne 0x142474606` ? `int3`.
  Depois do `int3` vem `lea rcx,...; call 0x144d9dab0; ...; ret` (`0x14247461d`).
  Ou seja, � um **assert de flag n�o inicializada**, n�o um assert de texture streaming.
- A flag fica em `.data`, RVA `0xa943999` (endere�o `0x14a943999`), **al�m da parte raw da se��o** (`VirtualSize 0x43ec398` vs `SizeOfRawData 0x622e00`) ? � zero-filled no load. Algu�m precisa escrever `1` nela em runtime; n�o sabemos quem.
- Hip�tese: chamador em loop chama essa fun��o, a flag continua 0, cada chamada cai no `int3`, o VEH continua at� o `ret`, o chamador tenta de novo ? 32768 em 0,4 s. Corrigir a flag � melhor que continuar o int3.
- `0x2215c40` � diferente: � o byte **logo ap�s** o abort deliberado em `0x2215c35` (`mov dword [0],0`). Como o byte em `+11` � `0xCC` (n�o `0xC3`), `XCL_NO_FATAL` aplica 11 NOPs e a execu��o cai no `int3`. Portanto `XCL_CONTINUE_ASSERTS` � necess�rio **por causa** de `XCL_NO_FATAL` nesse site � � benigno (2 ocorr�ncias).
- Pr�ximo passo concreto: achar quem escreve `0x14a943999` (busca por refer�ncias ao endere�o/RVA no `.text` com capstone) e decidir entre (a) setar a flag no HLE, (b) lista negra de RVA com toler�ncia, (c) throttle por RVA/tempo. N�o generalizar "continuar todo int3".

### 2. Device removed `0x887a0006` e `CreatePlacedResourceX 0x887a0005`
- Em 480,5 s h� cascata de placed resources falhando antes do device removed.
- Separar causas: (a) burst de asserts acima corrompendo estado, (b) press�o de VRAM, (c) submiss�o inv�lida detectada pelo driver.
- Verificar se o crash `0xc0000005` em `game+0x4dad047` (read `0x17305b1a1b3`, stack em `game+0x2213e3a`) � s� consequ�ncia do device removed.

### 3. Custo de write-watch no mundo
- `90743 ms` acumulados em 2.789.282 p�ginas sujas vs. `lock wait 6 ms`, `chunk scan 825 ms`, `upload 1248 ms`.
- Investigar por que o n�mero de p�ginas sujas explodiu no mundo e se o `before_execute` pode limitar a faixa ou agrupar melhor.

### 4. Draws indirect faltando
- 512 `ExecuteIndirect` skipped por PSO em build e 4 pipelines PC em ~417 s.
- Confirmar se s�o draws do mundo e se precisam de replay ap�s o PSO terminar (hoje o skip � permanente para aquele command buffer).

### 5. Timeout de GPU wait
- `GPU queue.Wait ... 5000 ms ... released on CPU to prevent GPU hang` em 406 s, com fences `done=0x6/sig=0x7`.
- Verificar se o release no CPU deixa o estado do fence inconsistente com o que o jogo espera.

### 6. Render/�udio finais
- depth bounds, HDR/scaling, metadados de render target; legendas e cutscenes longas com `XCL_TRACE_FRAME`.
- Sa�da de �udio aud�vel (Bink fora do HLE), DRC do XMA.

---

## Refer�ncias cruzadas

- Xbox2Windows: soft asserts + `-ContinueAsserts` confirmam nossa dire��o; BCPack at�mico; indirect draws.
- VIctory: chega ao mundo mas com imagem preta a 1,5�3 s/frame; `VICTORY_INDIRECT=1` necess�rio.
- REA: `rea` 6.7.0 instalado, falta Ghidra 12.1.x (`GHIDRA_INSTALL_DIR`) para descompilar PE; `dumpbin.exe` do MSVC 14.44 dispon�vel; capstone 5.0.7 via pip.
- Run de refer�ncia: `%TEMP%\opencode\xcl-pso100\RockstarGamesInc.28872443D20A7\logs\xcl.log`.

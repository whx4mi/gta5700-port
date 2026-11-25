# Arquitetura XCL — estado atual (24 Nov 2026)

## Peças do runtime

- **Loader/PE**: mapeia exe, imports, TLS, exceções (`pe.rs`, `loader.rs`).
- **HLE**: `kernelx`, XGameRuntime, XUser/XPackage/saves/PLS, GameInput, XAudio2/xapu, DirectStorage+BCPack, `d3d12_xs` (`src/hle/`).
- **VFS/overlay**: `GAME:`, `G:`, `S:`, `T:` → pasta do jogo; escritas vão para overlay.
- **UMA**: janelas sparse VRAM (0–2) + coerente (3); ABI `u0=io`, `u1=res`, `u2..=mem`.
- **Recompiler** (`gpu/recomp/`): RDNA2 → HLSL → DXIL via DXC; Kernel/Vertex/Pixel/Compute/Mesh/Geometry; fallback VS/AS local para RX 5700 (sem mesh shaders).
- **D3D12.X proxy** (`hle/d3d12x/`): vtables adotadas, slots console, streams de PSO, `PresentX`, arenas/pools de VRAM, `Wait64BitValueX` → fence GPU.

## PSO em build e stand-in (correção do bloqueio do mundo)

- `XCL_PSO_WAIT_MS` unset/0 = espera **ilimitada**. PSOs de compute com DXIL de 200–560 KB levam 10–20 s cada (picos de 19–22 s no run longo) → o jogo fica preso no spinner com 0 fps.
- `XCL_PSO_WAIT_MS=100` faz o bind seguir com pipeline stand-in e o jogo continua renderizando. Log: `a pipeline still being built after 100 ms: its binds go on without it until it is built (N so far)` — N passou de 512 sem perder os ~30 fps. Correção confirmada.
- `ExecuteIndirect` com PSO ainda em build é **skipped** (`ExecuteIndirect with a pipeline that is still being built skipped`); 512 skips em ~417 s. A tradução em `bind.rs` só cobre PSOs no mapa `TRANSLATED_GRAPHICS`; PSOs two-pass caem no caminho original.
- Drivers blobs em disco: `4/8 driver blobs kept on disk ... (this one: 530 ms, 0 KB)` — eficácia duvidosa nesta máquina.
- PSO timeout de GPU wait: `GPU queue.Wait ... did not complete within 5000 ms ... released on CPU to prevent GPU hang` (406 s).

## Memória do título ↔ GPU

- `kernelx::mirrors` emparelha reservas GPU-only com a reserve CPU legível (`mirrors::source`).
- `title_textures::mirror_source` escolhe a CPU view quando nada está mapeado na GPU view. `XCL_SAFE_READ=1` força `VirtualQuery` em todas as leituras (evita memcpy de mirror stale, crash `read at 0xb96a20000`).
- `title_textures::before_execute` usa `GetWriteWatch`, `sparse::before_execute`, `vram::before_execute` e `upload_all` em lotes de 256 MiB. Upstream otimizou: detile e hash **in-place**, write-watch em poucas chamadas. No mundo, porém: `write-watch 90743 ms (2789282 dirty pages), lock wait 6 ms, chunk scan 825 ms, upload 1248 ms (325530 runs)` — write-watch é o custo dominante e merece investigação.
- Fast path de detile linear: upstream `1f282c7`/`7d60a48` (linhas row-copy e coluna/row pré-computadas) substituiu o patch local.
- `sparse.rs`: pools de 64 MiB em `D3D12_HEAP_TYPE_DEFAULT`, `UpdateTileMappings`, `decommit_tiles`, `retire_free_pools` (2 s). `XCL_VRAM_IDLE_MS` (local) reduz esse tempo.
- `vram.rs`: `Pages` (pools de 64 MiB para texturas), `Arenas` (heaps por faixa de endereço), `cooling`. `XCL_SHARED_TEXTURE_PAGES=1` (local) reativa `place_pages` para texturas sampled — economiza VRAM mas já produziu crash de mirror stale.
- `XCL_SPARSE_VRAM_MB` (local) limita pools/arenas; não evitou o OOM, que veio de recursos committed.
- Sob pressão no mundo: `CreatePlacedResourceX ... a resource of its own (0x887a0005)` em cascata → `D3D12 device removed: 0x887a0006` (~481 s). O jogo reserva 7,4 GB só em sparse.

## Descritores e texturas

- Console heap (window 0) guarda T#/V#; `console_heap::write` renumeram `w[0]`.
- PS da UI lê T# de `s4`; UAs da ABI ligados por `bind_abi_full`.
- `01a7446` upstream: CBV são linhas float4 (stride 16), não buffers raw.
- Formatos console 116–120 mapeados; `pc_format` local.
- `note_content` com `XCL_TRACE_FRAME` mostra bytes não-zero de cada textura.
- Run do mundo: frame 5300 (3840×2160, format 28) com 518034/518400 amostras não-pretas; cores dominantes `0xff48263d`, `0xff593452`, `0xff583351` — imagem real, não tela sólida.

## Geometria

- Programas NGG/GS rodam como compute + passada de rasterização. Sem mesh shaders: VS pass-through (`vs_6_6`) + `DrawInstanced` (fallback local).
- Pipelines MS no slot AS **recusados** (40 erros): reativar → tela preta.
- `ExecuteIndirect` traduzido em `bind.rs`; PSOs two-pass não entram no mapa `TRANSLATED_GRAPHICS`, então indirect com PSO two-pass cai no caminho original.

## Vídeo, legendas e áudio

- Bink2Scarlett.dll decodifica vídeo no CPU; planos YUV viram texturas via `title_textures`.
- `xapu` (upstream) decodifica Opus via MSOpusDecoder: 9003 packets, 0 erros, ~9,7 M amostras não-zero no run longo. `report.audio.engines=0` porque Bink importa `XAudio2_9.dll` direto.
- BCPack: módulo `hle/bcpack/` portado do VIctory; BC1/BC3/BC4/BC5/BC6H/BC7 decodificam, sem E_DECOMPRESSION.
- Legendas não identificadas em frames de loading; provavelmente otra textura + PS em cutscene com fala.

## Aborts e asserts do título (RE)

- 7 sites `C7 04 25 00 00 00 00 <imm32>` (`mov dword [0], imm`) em `.text`: RVAs `0x2215c35`, `0x2215da9`, `0x23cf683`, `0x258b000`, `0x29e0880`, `0x390e0c0`, `0x402ac70`. `XCL_NO_FATAL=1` varre `.text` após `load_main` e neutramos (C3/NOP).
- Soft asserts `int3`: ~10/s no texture streaming (Xbox2Windows). `XCL_CONTINUE_ASSERTS=1` instala VEH que incrementa RIP (1 byte para `CC`, 2 para `CD 03`) e retorna `EXCEPTION_CONTINUE_EXECUTION`. O log é amostrado em potências de 2 (`n <= 4 || n.is_power_of_two()`).
- RVAs observados no run longo: `0x2215c40` (429,0 s e 477,2 s — isolado, continuado com sucesso) e `0x2474606` (480,56 s — contador 1→32768 em ~0,4 s, burst reentrante).
- **RE dos dois sites (capstone 5.0.7)**: `0x2215c40` é o `int3` imediatamente depois do abort `mov dword [0],0` em `0x2215c35`; `XCL_NO_FATAL` NOPa 11 bytes (porque o byte em `+11` é `CC`, não `C3`) e a execução deságua no `int3` — daí a necessidade de `XCL_CONTINUE_ASSERTS`. Já `0x2474606` é um assert de flag: `0x142474521: cmp byte [0x14a943999], 1` / `jne 0x142474606`; a flag está em `.data` RVA `0xa943999` na parte zero-filled (além do `SizeOfRawData`), então começa em 0. O burst é um loop de chamador com a flag nunca setada, não um assert de textura.
- Após o burst em `0x2474606`: `device removed 0x887a0006`, `unhandled 0xc0000005` em `game+0x4dad047` (read `0x17305b1a1b3`, bytes `4c 8b 41 18 b0 01 49 29 d0 49 0f ba e0 37 72 08`), com stack em `game+0x2213e3a`/`0x22145b1` e depois crash no handler (`vcruntime140.dll+0x1d7b`).
- `0x80070003` com `mov dword [0], 0x4d8bd421` é abort deliberado — não bug de ponteiro.
- Crash handler do jogo suspende as 105 threads ao escrever dump de 26 MB — parece travamento mas é Breakpad.

## Lacunas priorizadas

1. **Burst de int3 em `0x2474606`** — continuar RIP não estabiliza; flag `0x14a943999` começa em 0, achar quem a seta antes de considerar cap/throttle no VEH.
2. **Device removed `0x887a0006` / `CreatePlacedResourceX 0x887a0005`** no mundo por volta de 480 s — separar OOM de submissão inválida.
3. **Write-watch** com 90 s acumulados e 2,8 M páginas sujas — principal custo de submit no mundo.
4. **Indirect draws**: 512 `ExecuteIndirect` skipped por PSO em build; 4 pipelines PC — confirmar o que falta renderizar.
5. **Render do mundo**: depth bounds, HDR/scaling, metadados de render target.
6. **Legendas/cutscenes longas**: validar com `XCL_TRACE_FRAME` + dumps densos.
7. **Áudio**: confirmar saída audível (Bink fora do HLE); DRC do XMA.

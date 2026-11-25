# Port of XCL's no-mesh fallback into victory-fork — design note

Date: 2026-11-24. Branch with the first prototype: `mesh-fallback` (uncommitted).

**Direction change (recorded here for the decision):** after empirical tests on the RX 5700
(RDNA1, no mesh shaders), XCL was chosen as the primary base — it already has the two-pass
fallback and reached ~30 fps in the world, while this fork refuses mesh pipelines on that GPU
(`crates/victory-gpu/src/pipeline/host.rs:359-360`, host failures `0x887A0004` / `0x887A0001`).
This note preserves the port map that was produced before the change. It is not a promise that
the port is the right path; it is the exact work estimate if it is ever resumed.

## 1. What XCL does, point by point

All line numbers below are from the local, uncommitted XCL tree at
`C:\Users\Whxami\source\repos\xcl` on 2026-11-24 (backup: `E:\BACKUPS\xcl-20261124-1836\`).

| Piece | XCL location | Role |
|---|---|---|
| Record layout | `crates/xcl/src/gpu/recomp/mod.rs:130-153` | `MsInterface { outputs, threads, max_vertices, max_primitives }`, `MESH_HEADER = 4`, `vert_dwords() = 4 * (1 + ranked(outputs, true).len())`, `record_dwords() = MESH_HEADER + max_vertices * vert_dwords + max_primitives` |
| Output order | `crates/xcl/src/gpu/recomp/mod.rs:284-293` | `ranked(sig, with_clip)`: non-system outputs by register, then clip/cull distances |
| Compute writer | `crates/xcl/src/gpu/recomp/emit.rs:1952,1969,1974` | `res.Store` of position / params / primitive word |
| Record addressing | `crates/xcl/src/gpu/recomp/emit.rs:4475-4477` | per group: `xcl_rec`, `xcl_vtx`, `xcl_prm`; counts at `4539` and `4618`; primitive packing at `4551` and `4630` |
| VS pass-through | `crates/xcl/src/gpu/recomp/emit.rs:4782,4895-4925` | `passthrough_elems_and_fill` + `passthrough_vs`: `SV_InstanceID` = record, `SV_VertexID/3` = primitive, bits `x = p & 0x3FF`, `y = (p >> 10) & 0x3FF`, `z = (p >> 20) & 0x3FF`, bit 31 = null; null/out-of-range selects vertex 0 (degenerate). ABI: u0 `io`, u1 `res`, u2 `mem`; draw `DrawInstanced(3 * max_primitives, records, 0, 0)` |
| PS-linked pass-through | `crates/xcl/src/gpu/recomp/emit.rs:4823-4830`, `pso.rs:544-546` | `mesh_passthrough_for(ms, ps_inputs)` and `passthrough_vs(ms, ps_inputs)` are generated from the *paired PS input signature*, so the VS output rows pack exactly like the PS inputs (XCL `tests.rs:2894,2949-2960`) |
| PSO bundle | `crates/xcl/src/gpu/recomp/pso.rs:520-548` | `TwoPass { cs, ms, vs, record, mesh_groups_per_record, samplers }`; `console_mesh()` at `553-562` compiles cs/ms/vs with DXC |
| Host feature check | `crates/xcl/src/hle/d3d12x/real.rs:459-469` | `mesh_supported()` queries `D3D12_OPTIONS7::MeshShaderTier` |
| Host PSO state | `crates/xcl/src/hle/d3d12x/real.rs:4158-4172,4273,4320,4409` | `TwoPassPso` gains `is_vs` and `max_primitives`; MS/GS PSO streams put VS + triangle topology + empty input layout when there are no mesh shaders |
| Dispatch hook | `crates/xcl/src/hle/d3d12x/real.rs:2447,4889-4932` | `DispatchMesh` override: scratch records in UMA, compute pass with the *graphics* arguments, UAV barrier, graphics bind, `DrawInstanced(3 * max_primitives, groups, 0, 0)` |
| Root/bind plumbing | `crates/xcl/src/hle/d3d12x/bind.rs:854-940` | `bind_abi_full(...)` separates `gfx_args` from `gfx_bind`, can bind the compute PSO while reading graphics state, and shares one `alloc_res()` scratch UAV between both passes |
| Reference runner | `crates/xcl/src/gpu/recomp/runner.rs:1048-1051,1093-1105,1109-1170,1254-1260` | `Raster::{Mesh,Vs}`; graphics PSO for the fallback; `DispatchMesh` vs `DrawInstanced(3 * mp, groups, 0, 0)` |

## 2. Where victory-fork is today

| Piece | victory-fork location | Current behaviour |
|---|---|---|
| Mesh translation | `crates/victory-shader/src/emit/lower.rs:1421-1429` (state), `3997-4063` (`mesh_prologue`: `setMeshOutputCounts`/`emitIndices`), `10110-10343` (`export`: `storeVertexOutput`/`storePrimitiveOutput`), `10348-10398` (`exp prim` bit decoding), `4067` (`mesh_thread`) | Direct `ms_6_6`; no record writer, no compute mode |
| Shader metadata | `crates/victory-shader/src/emit/shader.rs:100-106` (`Mesh`), `441+` (`finish`), `750-757` (program kind 13); `container.rs:8-25` (`SigElement`), `118-133` (`mesh_output_bytes`), `146-164` (`Psv`) | Can only finish a mesh or compute shader, never both from one translation |
| Host refusal | `crates/victory-gpu/src/pipeline/host.rs:327-361` (refusal at `359-360`), `549-579` (mesh pipeline path), `734-800` (`mesh_pipeline`), `651` (PSO library); `crates/victory-gpu/src/host.rs:2732+` (`mesh_shaders()`) | Mesh pipelines are dropped outright on the RX 5700 |
| Draw path | `crates/victory-gpu/src/draw.rs:1799-1826` (`dispatch_mesh`), `crates/victory-gpu/src/list.rs:1592-1619` (PC `DispatchMesh` / `DispatchMeshX` hooks) | Only `ID3D12GraphicsCommandList6::DispatchMesh(x,1,1)` |
| Translation + cache | `crates/victory-gpu/src/pipeline.rs:121-197`, `crates/victory-gpu/src/shader_cache.rs:40-100,156,193-356` | One `HostShader` per console stage, keyed by build + container; a two-pass translation needs two blobs and a new cache layout |
| Compiler availability | `crates/victory-shader/src/emit/validator.rs:111-214` (validator only), `crates/victory-shader/src/emit/diff_tests.rs:654-693` (external `dxc.exe` in tests only) | No runtime HLSL→DXIL compiler; emitted DXIL is built directly with `Module`/`op` |

## 3. State after the 2026-11-25 session (branch `mesh-fallback`, uncommitted)

The branch was recreated from `main` (`a803c87c`); `main` is untouched. No commits,
no pushes, no game runs. The earlier branch pointer was an older commit with no
unique work; the untracked `mesh_fallback.rs` and this note were preserved.

### Done in `victory-shader`

- `emit/mesh_fallback.rs` (now a real module, declared in `emit/mod.rs` and
  re-exported): `MESH_HEADER_DWORDS`, `RECORD_UAV_SLOT = 63`, `NULL_PRIMITIVE`,
  `PrimitiveIndices::decode/corner`, `FallbackOutput`, `ranked_indices`,
  `MeshRecordLayout`, `MeshFallback` (layout + ranked outputs, `slot_of()`),
  the original `passthrough_vs_hlsl()` test helper, and — the new part —
  **`passthrough_vs()`**, which emits the pass-through VS as DXIL directly with
  the crate's `Shader`/`Module` machinery (no runtime DXC needed). Its outputs
  are the paired PS's inputs with `SV_Position` added, null/out-of-range
  primitives degenerate onto vertex 0, and the record read from root UAV u63.
- `emit/lower.rs`: when `VICTORY_MESH_FALLBACK=1`, `mesh_prologue()` no longer
  emits `setMeshOutputCounts`/`emitIndices`; it stores the vertex/primitive
  counts and a culled primitive word per group into the record, and `export()`
  redirects vertex outputs (position + ranked params) into the same record via
  `rawBufferStore.i32`. `emit_indices()` writes the packed console word with
  bit 31. The console stage stays `HwStage::Ms`, but the resulting DXIL is a
  `cs_6_6` compute shader.
- `emit/shader.rs`: `Shader.fallback` makes `finish()` emit compute metadata
  (no PSG1, no mesh state, compute PSV/program kind) while keeping the thread
  group. `root_uav_handle()` now only sets `RAW_BUFFERS`; `USES_UAV` is set
  explicitly by the generated VS so existing translations are unaffected.
- `emit/mod.rs`: `HostShader.mesh_fallback: Option<MeshFallback>` (full layout
  and ranked outputs), populated by `emit_program` only for a triangle mesh
  program with the switch on; the field is threaded through all test literals.
- Tests: `mesh_fallback` module (3 tests, all pass, including DXIL validation
  of the generated VS and a HLSL/DXC reference compile) and
  `mesh_fallback_tests::mesh_program_lowers_to_a_compute_record_writer`
  (passes; validates the compute record writer's DXIL).

### Done in `victory-gpu`

- `shader_cache.rs`: the switch is in the build key, magic `VSH1` → `VSH2`, the
  layout and ranked outputs are encoded/decoded, and the round-trip test now
  carries a fallback descriptor (2 cache tests pass).
- `pipeline/host.rs`: `HostPipeline` gained `fallback`, `writer_pso` and
  `writer_root`. When the switch is set and the mesh shader carries the
  descriptor, `HostPipeline::new()` no longer refuses a mesh pipeline on a GPU
  without mesh shaders: it creates the compute writer PSO (pass A) and a
  VS+PS triangle graphics PSO (pass B, empty input layout, VS generated from
  the PS signature). Root parameters are visible to `ALL` stages in that mode
  (a compute PSO requires it, and the generated VS must reach u63), and slot
  63 is a root UAV descriptor rather than a one-view table.
- `draw.rs`: the two-pass dispatch is implemented (see section D). The
  `fallback`/`writer_pso`/`writer_root` fields are now read, not dead code.

Default behaviour is unchanged: no emitted DXIL, PSO or draw differs unless
`VICTORY_MESH_FALLBACK=1`. With the switch on, mesh dispatches now run the
two-pass fallback; no game run was made to verify it.

## 4. Remaining work for a complete port

Ordered by dependency. Estimates are development time for someone already familiar with the
two codebases, excluding game runs.

### A+B+C — compute writer, VS pass-through and PSO creation: done in this session

See section 3 for the exact locations. What is *not* finished inside these items:

- The compute writer still clamps vertex/primitive counts to the declared
  maxima (the old shortcut) rather than reading the console's real counts;
  unwritten vertices stay at whatever the scratch buffer held. Zero the
  scratch records (or write real counts) before a draw.
- Per-primitive outputs (`PSG1`) are refused in fallback mode
  (`lower.rs`, `export()` early return). The title's mesh programs seen so far
  do not export any; add a packed per-primitive slot if one appears.
- The record UAV is bound at root parameter `u63, space 1`, not XCL's `u1`.
  The host must reserve that slot in both root signatures (already true:
  `HostPipeline::new()` builds a graphics and a compute root signature from
  the same parameter list), and the draw path must actually set it.
- The graphics PSO for pass B is created directly with
  `CreateGraphicsPipelineState`, not through `pipeline/library.rs`, so it is
  not stored in the on-disk PSO library (safe, but recompiled each launch).
- Wave64/split-LDS paths in the compute writer were not exercised by a real
  title shader; only the synthetic primitive-export test ran.

### D+E — two-pass dispatch and root plumbing: done in this session

`draw.rs::State` gained `fallback_scratch`, `fallback_zero` and `fallback_bytes`.
`dispatch_mesh()` now checks the switch and the pipeline's descriptor: with
the fallback off, it calls `DispatchMesh` as before; with it on and a
descriptor present, it calls `dispatch_mesh_fallback()` after the usual
`prepare_for()`:

1. `fallback_buffers()` makes (or reuses) a DEFAULT-heap buffer with
   `ALLOW_UNORDERED_ACCESS`, sized `layout.layout.record_bytes() * x`, plus an
   UPLOAD-heap buffer of zeros. Both are kept by the list until it is reset.
2. Each dispatch copies zeros into the first `x` records, transitions the
   scratch buffer `COPY_DEST → UNORDERED_ACCESS`.
3. The compute pass swaps `State::compute` with a clone of `State::graphics`
   for the duration of `bind(list, p, true)`, so the title's mesh arguments
   (which the title set with graphics root calls) are replayed on the compute
   signature. `bind()` selects `writer_root`/`writer_pso` for that pass; u63
   is skipped there (and in the graphics pass) instead of looking for a title
   root UAV. The scratch GPU address is then set at u63 explicitly.
4. `Dispatch(x,1,1)` → UAV barrier → `bind(list, p, false)` (graphics PSO and
   root, same parameters) → set u63 on the graphics root →
   `IASetPrimitiveTopology(TRIANGLELIST)` →
   `DrawInstanced(3 * layout.layout.max_primitives, x, 0, 0)`.
5. The scratch buffer goes `UNORDERED_ACCESS → COPY_DEST` on every path, and
   the title's topology is restored with `set_topology()` if it had been set.
   The host list is left with the graphics root/PSO bound, as after any draw.
6. `list.rs` still rejects `y*z != 1` and indirect mesh dispatches before
   either path is reached (unchanged).

**Not verified on hardware.** The PSOs, root signatures, buffer states and
barriers are built exactly as described, but only compilation, unit tests and
DXIL validation ran; no WARP or RX 5700 run was made. What remains:

- A real title mesh pipeline must be checked for PSO creation (VS↔PS linking
  by semantic, root signature parameter count ≤ 64, compute root visibility).
- `VICTORY_MESH_TEST=keep:` filters on `p.name`, which for a fallback pipeline
  no longer contains `mesh <hash>` in the same form; the filter may need a
  separate name for fallback pipelines.
- If the title sets mesh arguments with **compute** root calls rather than
  graphics ones, the swap in step 3 picks the wrong vector; confirm from a
  frame listing before a guarded GPU run.
- `per-primitive outputs`, non-triangle mesh programs, exact console counts
  and wave64/LDS mesh programs remain out of scope (see item A+B+C above).

### F. Validation
WARP first (`VICTORY_D3D12_WARP=1`, diag build), then only short guarded runs on the RX 5700
with a watchdog; never run the full game unattended (the card has already been reset twice).
**Estimate: 2-4 days.**

**Total: ~1,500-2,700 LoC, roughly 16-30 development days, high risk.** Given the current
decision (XCL as primary base), this work is not scheduled.

## 5. Risks and observations

- The fork's check forbids mesh pipelines before any shader work (`pipeline/host.rs:359-360`);
  changing that check is necessary but not sufficient — the PSO form must change too.
- No direct HLSL compiler in the crate raises the largest integration risk; XCL's whole
  translation is HLSL text compiled by DXC, which the fork's bitcode emitter is not.
- The 32 KiB mesh output limit (`container.rs:118-133`) stops applying to records, but the UAV
  size and `DrawInstanced` vertex count must still be bounded by `max_primitives`.
- `cargo test -p victory-shader --lib` has two pre-existing failures in
  `emit/rect.rs` (`the_rectangle_stage_is_a_geometry_shader`, `rectangles_are_drawn_whole`):
  "Container part 'Pipeline State Validation' does not match expected for module". They
  were present before this work; the only `rect.rs` edit is the new `mesh_fallback: None`
  field. All other 216 tests pass, including the four fallback tests.
- The reference HLSL compiles with the Windows SDK DXC (`10.0.26100.0\x64\dxc.exe`).
  The crate now generates the VS DXIL itself (no runtime DXC) and validates it with
  `dxil.dll`; that proves the container is loadable, not that the two-pass pipeline
  works end to end.
- The compute writer uses root UAV `u63, space 1`. A real title shader that writes
  per-primitives, uses wave64 cross-lane ops with LDS, or relies on the console's
  exact vertex/primitive counts has NOT been validated — only a synthetic
  primitive-export program.

## 6. State of branch `mesh-fallback` (2026-11-25)

Recreated from `main` (`a803c87c48078e57b296e06e08146d7ceca44530`); `main` untouched,
no commits, no pushes. Untracked: `emit/mesh_fallback.rs`, this note, and the pre-existing
`rodar.ps1`. The draw-side two-pass dispatch was added in the 2026-11-25 follow-up
session (sections D+E). Validation done across both sessions:

- `cargo build --profile diag -p victory --features diagnostics` — PASS (no
  warnings after the draw path started reading the fallback PSO fields).
- `cargo check -p victory-shader --all-targets`, `cargo check -p victory-gpu --all-targets`
  — PASS.
- `cargo test -p victory-shader --lib mesh_fallback` (4 tests, including the
  compute writer test) — PASS; generated VS and compute DXIL both validate
  with the SDK validator.
- `cargo test -p victory-gpu --lib shader_cache` (2 tests) — PASS.
- Full `cargo test -p victory-shader --lib` — 216 pass, 3 ignored, the two
  pre-existing `rect.rs` failures above.
- The game was NOT run (GPU untouched). No commit, no push.

**Before any GPU run:** use WARP with the diag build and a watchdog; the
two-pass path has only been compiled and unit-tested, never executed.

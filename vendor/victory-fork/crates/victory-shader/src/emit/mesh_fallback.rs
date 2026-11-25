//! The record layout and vertex pass-through used to rasterise console mesh
//! programs on hosts without D3D12 mesh shaders (`VICTORY_MESH_FALLBACK=1`).
//!
//! This is the first, self-contained part of the two-pass port of XCL's NGG
//! fallback (see `docs/fork-mesh-fallback.md`). The console program is meant
//! to run as a compute pass that writes one record per group into a UAV:
//!
//! ```text
//! record = header(nv, np, spare) | vertices | primitive words
//! header[0] = vertex count   (clamped to max_vertices by the reader)
//! header[1] = primitive count
//! header[2..4] = spare
//! vertex  = position (4 dwords) + one 4-dword slot per ranked output
//! primitive word: x = p & 0x3FF, y = (p >> 10) & 0x3FF,
//!                 z = (p >> 20) & 0x3FF, bit 31 = null primitive
//! ```
//!
//! The second pass is a `vs_6_6` pass-through drawn with
//! `DrawInstanced(3 * max_primitives, groups, 0, 0)`: one instance is a
//! record, three vertices a primitive. [`passthrough_vs_hlsl`] generates it
//! with the same XCL shader ABI (u0 `io`, u1 `res`). Nothing here changes the
//! default translation: the descriptor is attached to [`HostShader`] only
//! when [`enabled`] is true.

use std::fmt::Write as _;
use std::sync::OnceLock;

use super::bitcode::{Attr, Bin, Cast, V, pred};
use super::container::SigElement;
use super::shader::Shader;
use crate::ShaderError;
use crate::dxbc::{Container, FourCC};

/// Dwords before the vertices of a record: vertex count, primitive count and
/// two spare words. The same constant as XCL's `MESH_HEADER`.
pub const MESH_HEADER_DWORDS: u32 = 4;

/// The root UAV (space 1) both passes use for the record buffer. It is ours,
/// not a console parameter: the host binds one scratch buffer at this slot
/// for the compute writer and the pass-through vertex shader alike.
pub const RECORD_UAV_SLOT: u8 = 63;

/// A primitive with bit 31 set is culled; the reader turns it into a
/// degenerate triangle.
pub const NULL_PRIMITIVE: u32 = 1 << 31;

const INDEX_BITS: u32 = 10;
const INDEX_MASK: u32 = (1 << INDEX_BITS) - 1;
const NULL_BIT: u32 = NULL_PRIMITIVE;

/// `VICTORY_MESH_FALLBACK=1` opts into the fallback. Read once, like the
/// other diagnostics switches; unset (or any other value) leaves the
/// translation as it was.
pub fn enabled() -> bool {
    static ON: OnceLock<bool> = OnceLock::new();
    *ON.get_or_init(|| std::env::var("VICTORY_MESH_FALLBACK").as_deref() == Ok("1"))
}

/// The three vertex indices a primitive word names. `None` is the console's
/// null primitive (bit 31), which the reader turns into a degenerate
/// triangle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PrimitiveIndices(pub [u32; 3]);

impl PrimitiveIndices {
    /// Decodes the console's packed primitive word, or `None` when bit 31
    /// culls it.
    pub fn decode(word: u32) -> Option<Self> {
        if word & NULL_BIT != 0 {
            return None;
        }
        Some(Self([
            word & INDEX_MASK,
            (word >> INDEX_BITS) & INDEX_MASK,
            (word >> (2 * INDEX_BITS)) & INDEX_MASK,
        ]))
    }

    /// The vertex a corner reads, with null and out-of-range primitives
    /// collapsed onto vertex 0 (zero area, no pixels rasterised).
    pub fn corner(&self, corner: u32, vertex_count: u32) -> u32 {
        let i = self.0.get(corner as usize).copied().unwrap_or(0);
        if i < vertex_count { i } else { 0 }
    }
}

/// One element of the per-vertex payload: enough to declare the pass-through
/// VS's output and to lay out the record. Translated from the mesh program's
/// `OSG1` by [`FallbackOutput::from_sig`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FallbackOutput {
    /// Semantic name, e.g. `TEXCOORD` or `SV_ClipDistance`.
    pub name: String,
    /// Semantic index.
    pub index: u32,
    /// DXIL semantic kind: 0 arbitrary, 6 clip distance, 7 cull distance.
    pub system_value: u8,
    /// DXIL component type: 4 i32, 5 u32, 9 f32.
    pub component_type: u8,
    /// Console output register, used to order the payload.
    pub row: u32,
    /// Components actually written.
    pub components: u8,
}

impl FallbackOutput {
    pub(crate) fn from_sig(e: &super::container::SigElement) -> Self {
        Self {
            name: e.name.clone(),
            index: e.index,
            system_value: e.kind,
            component_type: e.comp,
            row: e.row.unwrap_or(u32::MAX),
            components: e.cols.max(1),
        }
    }

    fn is_clip_or_cull(&self) -> bool {
        matches!(self.system_value, 6 | 7)
    }
}

/// Non-system outputs in register order, then clip and cull distances
/// (XCL's `ranked(..., true)`). This is the order the compute writer puts
/// them in the record, and the order the pass-through VS reads them.
fn ranked(outputs: &[FallbackOutput]) -> Vec<&FallbackOutput> {
    ranked_indices(outputs)
        .into_iter()
        .map(|i| &outputs[i])
        .collect()
}

/// Indices into `outputs` in record order: non-system outputs by register,
/// then clip and cull distances. Slot 0 of a vertex is its position, so the
/// rank `k` here is written to and read from dword `4 * (k + 1)`.
pub(crate) fn ranked_indices(outputs: &[FallbackOutput]) -> Vec<usize> {
    let mut elements: Vec<usize> = outputs
        .iter()
        .enumerate()
        .filter(|(_, e)| e.system_value == 0 && !e.name.starts_with("SV_"))
        .map(|(i, _)| i)
        .collect();
    elements.sort_by_key(|&i| outputs[i].row);
    let mut distances: Vec<usize> = outputs
        .iter()
        .enumerate()
        .filter(|(_, e)| e.is_clip_or_cull())
        .map(|(i, _)| i)
        .collect();
    distances.sort_by_key(|&i| outputs[i].row);
    elements.extend(distances);
    elements
}

/// The layout the compute writer and the pass-through VS share, attached to
/// a translated mesh shader only when `VICTORY_MESH_FALLBACK=1` is set.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MeshFallback {
    pub layout: MeshRecordLayout,
    /// The mesh program's outputs in record order (slot 1 onward). The
    /// position is always slot 0 and is not in this list.
    pub outputs: Vec<FallbackOutput>,
}

impl MeshFallback {
    pub(crate) fn new(outputs: &[FallbackOutput], mesh: super::shader::Mesh, threads: u32) -> Self {
        let list = ranked(outputs);
        Self {
            layout: MeshRecordLayout::for_outputs(
                outputs,
                mesh.max_vertices,
                mesh.max_primitives,
                threads,
            ),
            outputs: list.into_iter().cloned().collect(),
        }
    }

    /// Record slot of a signature element, or `None` when the paired pixel
    /// shader declares something the mesh program never exported.
    pub fn slot_of(&self, name: &str, index: u32, kind: u8) -> Option<u32> {
        if kind == 3 {
            return Some(0);
        }
        self.outputs
            .iter()
            .position(|e| e.name == name && e.index == index && e.system_value == kind)
            .map(|k| 1 + k as u32)
    }
}

/// A mesh record's size and offsets, mirroring XCL's `MsInterface` helpers.
///
/// The console limits vertices and primitives to 256 each, so all of these
/// fit in u32 and can be emitted as HLSL constants.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MeshRecordLayout {
    pub max_vertices: u32,
    pub max_primitives: u32,
    /// Console thread-group size (the writer's lane count).
    pub threads: u32,
    /// Dwords per vertex: position plus one slot per ranked output.
    pub vert_dwords: u32,
    /// Dwords per record: header, all vertices, all primitives.
    pub record_dwords: u32,
}

impl MeshRecordLayout {
    /// Derives the layout from the mesh program's outputs.
    pub fn for_outputs(
        outputs: &[FallbackOutput],
        max_vertices: u32,
        max_primitives: u32,
        threads: u32,
    ) -> Self {
        let vert_dwords = 4 * (1 + ranked(outputs).len() as u32);
        Self {
            max_vertices,
            max_primitives,
            threads,
            vert_dwords,
            record_dwords: MESH_HEADER_DWORDS + max_vertices * vert_dwords + max_primitives,
        }
    }

    /// Dword offset of a vertex inside a record.
    pub fn vertex_dword(&self, vertex: u32) -> u32 {
        MESH_HEADER_DWORDS + vertex * self.vert_dwords
    }

    /// Dword offset of a primitive word inside a record.
    pub fn primitive_dword(&self, primitive: u32) -> u32 {
        MESH_HEADER_DWORDS + self.max_vertices * self.vert_dwords + primitive
    }

    /// Bytes between consecutive records (the UAV stride).
    pub fn record_bytes(&self) -> u32 {
        self.record_dwords * 4
    }

    /// The vertex a draw corner must read, degenerating null and
    /// out-of-range primitives onto vertex 0.
    pub fn vertex_index(&self, word: u32, corner: u32, vertex_count: u32) -> u32 {
        PrimitiveIndices::decode(word)
            .map_or(0, |p| p.corner(corner, vertex_count.min(self.max_vertices)))
    }
}

fn hlsl_type(component: u8) -> &'static str {
    match component {
        4 => "int",
        5 => "uint",
        _ => "float",
    }
}

fn hlsl_cast(component: u8) -> &'static str {
    match component {
        4 => "asint",
        5 => "asuint",
        _ => "asfloat",
    }
}

fn semantic(e: &FallbackOutput) -> String {
    format!("{}{}", e.name, e.index)
}

/// HLSL of the second pass: one instance is one record, three vertices per
/// primitive. The primitive word's bit 31 and indices past the record's
/// vertex count all select vertex 0, i.e. a degenerate triangle.
///
/// The HLSL is compiled to `vs_6_6` by DXC (the validator found by
/// [`super::validator::Validator::default_path`] has a `dxc.exe` next to it).
/// It binds nothing but u0 `io` and u1 `res`, like XCL's `passthrough_vs`.
pub fn passthrough_vs_hlsl(layout: &MeshRecordLayout, outputs: &[FallbackOutput]) -> String {
    let elements = ranked(outputs);
    let mut h = String::new();
    h.push_str("// generated by VIctory: mesh fallback pass-through VS (no mesh shaders)\n");
    h.push_str("RWByteAddressBuffer io : register(u0);\n");
    h.push_str("RWByteAddressBuffer res : register(u1);\n");
    h.push_str("struct FallbackVert {\n    float4 pos : SV_Position;\n");
    for (k, e) in elements.iter().enumerate() {
        let n = if e.is_clip_or_cull() {
            e.components.clamp(1, 4) as usize
        } else {
            4
        };
        let ty = hlsl_type(e.component_type);
        let ty = if n == 1 { ty.to_owned() } else { format!("{ty}{n}") };
        let _ = writeln!(h, "    {ty} p{k} : {};", semantic(e));
    }
    h.push_str("};\n");
    let tri_base = layout.primitive_dword(0);
    h.push_str("FallbackVert main(uint vid : SV_VertexID, uint iid : SV_InstanceID) {\n");
    let _ = writeln!(h, "    uint rec = iid * {}u;", layout.record_dwords);
    let _ = writeln!(
        h,
        "    uint nv = min(res.Load(rec * 4u), {}u);",
        layout.max_vertices
    );
    h.push_str("    uint j = vid / 3u;\n    uint c = vid % 3u;\n");
    let _ = writeln!(
        h,
        "    uint p = res.Load((rec + {tri_base}u + j) * 4u);"
    );
    h.push_str(
        "    uint3 x = uint3(p & 0x3FFu, (p >> 10) & 0x3FFu, (p >> 20) & 0x3FFu);\n",
    );
    h.push_str("    bool bad = ((p >> 31) != 0u) || any(x >= nv);\n");
    let _ = writeln!(
        h,
        "    uint b = (rec + {}u + select(bad, 0u, x[c]) * {}u) * 4u;",
        MESH_HEADER_DWORDS, layout.vert_dwords
    );
    h.push_str("    FallbackVert o;\n");
    h.push_str("    o.pos = asfloat(res.Load4(b));\n");
    for (k, e) in elements.iter().enumerate() {
        let n = if e.is_clip_or_cull() {
            e.components.clamp(1, 4) as usize
        } else {
            4
        };
        let ty = hlsl_type(e.component_type);
        let cast = hlsl_cast(e.component_type);
        let _ = writeln!(
            h,
            "    {ty}4 v{k} = {cast}(res.Load4(b + {}u));",
            16 * (k + 1)
        );
        let value = if n == 4 {
            format!("v{k}")
        } else {
            format!("v{k}.{}", &"xyzw"[..n])
        };
        let _ = writeln!(h, "    o.p{k} = {value};");
    }
    h.push_str("    return o;\n}\n");
    h
}

/// The second pass's vertex shader as DXIL, generated with the crate's own
/// bitcode writer (the fork has no DXC at run time). Its outputs are the
/// paired pixel shader's inputs, so the two link; each reads its components
/// from a vertex of the record written by the compute pass.
///
/// `mesh` is the translated mesh shader carrying the layout; `ps` is the
/// pixel shader's DXIL container. The shader binds one root UAV,
/// [`RECORD_UAV_SLOT`], which the host fills with the scratch record buffer.
pub fn passthrough_vs(mesh: &super::HostShader, ps: &[u8]) -> Result<Vec<u8>, ShaderError> {
    let fallback = mesh.mesh_fallback.as_ref().ok_or(ShaderError::Unsupported {
        offset: 0,
        what: "a mesh shader without a fallback layout",
    })?;
    let container = Container::parse(ps)?;
    let mut outputs = match container.part(FourCC::ISG1) {
        Some(part) => super::signature(part.data)?,
        None => Vec::new(),
    };
    // A vertex shader must write a position, even when the pixel shader
    // declares none. Put it in row 0 and move the other rows up: D3D12 links
    // by semantic, not by row.
    if !outputs.iter().any(|e| e.kind == 3) {
        for e in &mut outputs {
            if let Some(row) = &mut e.row {
                *row += 1;
            }
        }
        outputs.insert(
            0,
            SigElement {
                name: "SV_Position".into(),
                index: 0,
                kind: 3,
                comp: 9,
                interp: 4,
                row: Some(0),
                col: 0,
                cols: 4,
                usage: 0xF,
                half: false,
            },
        );
    }
    // As `emit_program` declares a vertex shader's outputs.
    for e in &mut outputs {
        e.interp = match (e.kind, e.comp) {
            (3, _) => 4,
            (13, _) => 1,
            (28, _) => 2,
            (0 | 6 | 7, 9) => 2,
            (0, _) => 1,
            _ => 0,
        };
        if e.usage == 0 {
            e.usage = (1 << e.cols.min(8)) - 1;
        }
    }
    let mut s = Shader::new(false);
    // `SV_VertexID` (kind 1) and `SV_InstanceID` (kind 2) are rows 0 and 1.
    s.inputs = [
        ("SV_VertexID", 1u8),
        ("SV_InstanceID", 2u8),
    ]
    .into_iter()
    .enumerate()
    .map(|(row, (name, kind))| SigElement {
        name: name.into(),
        index: 0,
        kind,
        comp: 5,
        interp: 0,
        row: Some(row as u32),
        col: 0,
        cols: 1,
        usage: 1,
        half: false,
    })
    .collect();
    // A system value may only be in row 0..=1 in a VS input signature, and
    // our two inputs are there. Arbitrary outputs above must not be shifted
    // into row 1 by a caller that assumed a PS input signature: D3D12 links
    // by semantic, so their rows are ours to choose.
    for e in &mut outputs {
        if e.kind != 3 && let Some(row) = &mut e.row {
            *row = (*row).saturating_add(2);
        }
    }
    s.flags |= super::shader::flags::USES_UAV;
    s.outputs = outputs.clone();
    let b = s.entry;
    let (i32t, void, res_ret, f32t) = (s.t.i32, s.t.void, s.t.res_ret, s.t.f32);
    let iu = s.m.undef(i32t);
    let zero = s.m.i32(0);
    let four = s.m.i32(4);
    let col0_i8 = s.m.i8(0);
    let mask1 = s.m.i8(1);
    let mask4 = s.m.i8(0xF);
    let row0 = s.m.i32(0);
    let three = s.m.i32(3);
    let shifts = [zero, s.m.i32(INDEX_BITS), s.m.i32(2 * INDEX_BITS)];
    let index_mask = s.m.i32(INDEX_MASK);
    let null_bit = s.m.i32(NULL_PRIMITIVE);
    let load_input = |s: &mut Shader, id: u32, row: V| {
        let id = s.m.i32(id);
        // DXIL: `dx.op.loadInput(sigId, row, i8 col, undef axis)`.
        s.op(b, "loadInput.i32", Attr::ReadNone, i32t, 4, &[id, row, col0_i8, iu])
    };
    let vid = load_input(&mut s, 0, row0);
    let iid = load_input(&mut s, 1, row0);
    let h = s.root_uav_handle(RECORD_UAV_SLOT);
    let stride = s.m.i32(fallback.layout.record_dwords);
    let rec = s.m.bin(b, Bin::Mul, iid, stride);
    let max_vertices = s.m.i32(fallback.layout.max_vertices);
    let header_byte = s.m.bin(b, Bin::Mul, rec, four);
    let header = s.op(
        b,
        "rawBufferLoad.f32",
        Attr::ReadOnly,
        res_ret,
        139,
        &[h, header_byte, iu, mask1, four],
    );
    let header_bits = s.m.extract(b, header, 0);
    let count = s.m.cast(b, Cast::Bitcast, header_bits, i32t);
    let inside = s.m.cmp(b, pred::IULT, count, max_vertices);
    let nv = s.m.select(b, inside, count, max_vertices);
    let j = s.m.bin(b, Bin::UDiv, vid, three);
    let corner = s.m.bin(b, Bin::URem, vid, three);
    let prim_off = s.m.i32(fallback.layout.primitive_dword(0));
    let group_and_prim = s.m.bin(b, Bin::Add, rec, j);
    let prim = s.m.bin(b, Bin::Add, group_and_prim, prim_off);
    let prim_byte = s.m.bin(b, Bin::Mul, prim, four);
    let r = s.op(
        b,
        "rawBufferLoad.f32",
        Attr::ReadOnly,
        res_ret,
        139,
        &[h, prim_byte, iu, mask1, four],
    );
    let word_bits = s.m.extract(b, r, 0);
    let word = s.m.cast(b, Cast::Bitcast, word_bits, i32t);
    let mut indices = Vec::new();
    for shift in shifts {
        let i = s.m.bin(b, Bin::LShr, word, shift);
        indices.push(s.m.bin(b, Bin::And, i, index_mask));
    }
    let is_null = s.m.bin(b, Bin::And, word, null_bit);
    let mut bad = s.m.cmp(b, pred::NE, is_null, zero);
    for &i in &indices {
        let past = s.m.cmp(b, pred::IUGE, i, nv);
        bad = s.m.bin(b, Bin::Or, bad, past);
    }
    let mut index = indices[0];
    for (k, &i) in indices.iter().enumerate().skip(1) {
        let kth = s.m.i32(k as u32);
        let is_k = s.m.cmp(b, pred::EQ, corner, kth);
        index = s.m.select(b, is_k, i, index);
    }
    let index = s.m.select(b, bad, zero, index);
    let header_dwords = s.m.i32(MESH_HEADER_DWORDS);
    let vert_stride = s.m.i32(fallback.layout.vert_dwords);
    let in_vertex = s.m.bin(b, Bin::Mul, index, vert_stride);
    let vertex = s.m.bin(b, Bin::Add, header_dwords, in_vertex);
    let base = s.m.bin(b, Bin::Add, rec, vertex);
    // Every output reads the four dwords of its record slot. The bitcasts
    // make the bits the writer stored visible as the declared type.
    let mut payload: Vec<Vec<super::bitcode::V>> = Vec::new();
    for e in &outputs {
        let v = if let Some(slot) = fallback.slot_of(&e.name, e.index, e.kind) {
            let offset = s.m.i32(4 * (slot + u32::from(e.col)));
            let at = s.m.bin(b, Bin::Add, base, offset);
            let byte = s.m.bin(b, Bin::Mul, at, four);
            let r = s.op(
                b,
                "rawBufferLoad.f32",
                Attr::ReadOnly,
                res_ret,
                139,
                &[h, byte, iu, mask4, four],
            );
            let mut bits = Vec::new();
            for k in 0..4 {
                let f = s.m.extract(b, r, k);
                bits.push(s.m.cast(b, Cast::Bitcast, f, i32t));
            }
            bits
        } else {
            vec![zero; 4]
        };
        payload.push(v);
    }
    for (id, e) in outputs.iter().enumerate() {
        let name = if e.comp == 9 {
            "storeOutput.f32"
        } else {
            "storeOutput.i32"
        };
        let id_dword = s.m.i32(id as u32);
        let row = s.m.i32(0);
        for col in 0..usize::from(e.cols.min(4)) {
            let bits = payload[id][col];
            let value = if e.comp == 9 {
                s.m.cast(b, Cast::Bitcast, bits, f32t)
            } else {
                bits
            };
            let column = s.m.i8(e.col + col as u8);
            s.op(b, name, Attr::NoUnwind, void, 5, &[id_dword, row, column, value]);
        }
    }
    s.m.terminate(b, super::bitcode::Term::Ret);
    Ok(s.finish())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn output(name: &str, index: u32, kind: u8, comp: u8, row: u32) -> FallbackOutput {
        FallbackOutput {
            name: name.into(),
            index,
            system_value: kind,
            component_type: comp,
            row,
            components: 4,
        }
    }

    #[test]
    fn record_layout_and_primitive_words_match_the_console_abi() {
        let outputs = [
            output("TEXCOORD", 0, 0, 9, 0),
            output("COLOR", 0, 0, 5, 1),
            output("SV_ClipDistance", 0, 6, 9, 2),
        ];
        let layout = MeshRecordLayout::for_outputs(&outputs, 128, 64, 128);
        assert_eq!(layout.vert_dwords, 4 * (1 + 3));
        assert_eq!(layout.vertex_dword(0), MESH_HEADER_DWORDS);
        assert_eq!(layout.vertex_dword(1), MESH_HEADER_DWORDS + 16);
        assert_eq!(layout.primitive_dword(0), MESH_HEADER_DWORDS + 128 * 16);
        assert_eq!(layout.record_dwords, 4 + 128 * 16 + 64);
        assert_eq!(layout.record_bytes(), layout.record_dwords * 4);

        let p = PrimitiveIndices::decode(1 << 10 | 2 << 20).unwrap();
        assert_eq!(p.0, [0, 1, 2]);
        assert_eq!(p.corner(2, 128), 2);
        assert_eq!(p.corner(0, 0), 0, "out of range is degenerated");
        assert_eq!(
            PrimitiveIndices::decode(0x8000_0000 | 1 << 10),
            None,
            "bit 31 is the null primitive"
        );
        assert_eq!(layout.vertex_index(0x8000_0001, 0, 128), 0);
        assert_eq!(layout.vertex_index(3 << 20, 2, 2), 0);
    }

    #[test]
    #[cfg(windows)]
    fn passthrough_vs_compiles_and_validates() {
        let Some(dxil) = super::super::validator::Validator::default_path() else {
            eprintln!("no dxil.dll; skipped");
            return;
        };
        let dxc = dxil.with_file_name("dxc.exe");
        if !dxc.is_file() {
            eprintln!("no dxc.exe; skipped");
            return;
        }
        let outputs = [
            output("TEXCOORD", 0, 0, 9, 0),
            output("COLOR", 0, 0, 5, 1),
            output("SV_ClipDistance", 0, 6, 9, 2),
        ];
        let layout = MeshRecordLayout::for_outputs(&outputs, 128, 64, 128);
        let hlsl = passthrough_vs_hlsl(&layout, &outputs);
        assert!(hlsl.contains("SV_InstanceID"));
        assert!(hlsl.contains("select(bad, 0u, x[c])"));

        let dir = std::env::temp_dir().join(format!(
            "victory-mesh-fallback-{}-{}",
            std::process::id(),
            line!()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let src = dir.join("passthrough.hlsl");
        let bin = dir.join("passthrough.dxil");
        std::fs::write(&src, &hlsl).unwrap();
        let out = std::process::Command::new(&dxc)
            .args(["-T", "vs_6_6", "-E", "main", "-Fo"])
            .arg(&bin)
            .arg(&src)
            .output()
            .unwrap();
        assert!(
            out.status.success(),
            "DXC rejected the pass-through VS: {}",
            String::from_utf8_lossy(&out.stderr)
        );
        let mut bytes = std::fs::read(&bin).unwrap();
        let mut validator = super::super::validator::Validator::load(Some(&dxil)).unwrap();
        validator.validate(&mut bytes).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    #[cfg(windows)]
    fn generated_vs_dxil_is_a_linkable_vertex_shader() {
        let outputs = [
            output("TEXCOORD", 0, 0, 9, 0),
            output("COLOR", 0, 0, 5, 1),
            output("SV_ClipDistance", 0, 6, 9, 2),
        ];
        let fallback = MeshFallback::new(
            &outputs,
            super::super::shader::Mesh {
                max_vertices: 128,
                max_primitives: 64,
                topology: 2,
            },
            128,
        );
        assert_eq!(fallback.slot_of("TEXCOORD", 0, 0), Some(1));
        assert_eq!(fallback.slot_of("COLOR", 0, 0), Some(2));
        assert_eq!(fallback.slot_of("SV_ClipDistance", 0, 6), Some(3));
        assert_eq!(fallback.slot_of("SV_Position", 0, 3), Some(0));
        assert_eq!(fallback.slot_of("TEXCOORD1", 1, 0), None);

        // A pixel signature with no SV_Position: the VS must add one.
        let mut ps = super::super::shader::Shader::new(true);
        ps.inputs = vec![SigElement {
            name: "TEXCOORD".into(),
            index: 0,
            kind: 0,
            comp: 9,
            interp: 2,
            row: Some(0),
            col: 0,
            cols: 2,
            usage: 3,
            half: false,
        }];
        ps.m.terminate(ps.entry, super::super::bitcode::Term::Ret);
        let ps_dxil = ps.finish();
        let host = super::super::HostShader {
            stage: crate::HwStage::Ms,
            dxil: Vec::new(),
            user_data: Vec::new(),
            samplers: Vec::new(),
            static_samplers: Vec::new(),
            root_cbvs: Vec::new(),
            root_srvs: Vec::new(),
            root_uavs: Vec::new(),
            root_indexed_views: Vec::new(),
            mesh_fallback: Some(fallback),
        };
        let vs = passthrough_vs(&host, &ps_dxil).unwrap();
        let c = Container::parse(&vs).unwrap();
        let out = super::super::signature(c.part(FourCC::OSG1).unwrap().data).unwrap();
        assert_eq!(out[0].name, "SV_Position");
        assert_eq!(out[1].name, "TEXCOORD");
        assert_eq!(out[1].row, Some(3), "the other rows moved past the VS system inputs");
        let ins = super::super::signature(c.part(FourCC::ISG1).unwrap().data).unwrap();
        assert_eq!(ins[0].name, "SV_VertexID");
        assert_eq!(ins[1].name, "SV_InstanceID");
        // Validate with the SDK's DXIL validator, as the other emitted DXIL
        // tests do (a failure here means the container is not loadable).
        let Some(dxil) = super::super::validator::Validator::default_path() else {
            eprintln!("no dxil.dll; skipped validation");
            return;
        };
        let mut v = super::super::validator::Validator::load(Some(&dxil)).unwrap();
        v.validate(&mut vs.clone()).unwrap();
    }
}

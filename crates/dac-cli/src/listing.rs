//! `-O0` annotated listing renderer (B1.6).
//!
//! The `-O0` output is not a "real" backend — it is a faithful textual
//! view of the lifted IR. Each discovered function gets a header block
//! describing its address range, confidence, and contributing signals;
//! the body lists every decoded instruction with its address, encoded
//! bytes, raw mnemonic, and lifted IR classification.
//!
//! The output is deterministic by construction: functions are listed in
//! ascending address order (which is what [`dac_recovery::FunctionSet`]
//! already guarantees), instructions inside each function are emitted in
//! the order the decoder returns them, and every numeric field is
//! formatted with a fixed width. No floating-point formatting depends on
//! locale.
//!
//! Provenance link to the rest of the pipeline:
//!
//! - `dac-binfmt` supplies the [`BinaryModel`] (sections + symbols).
//! - `dac-arch` supplies the [`InstructionDecoder`] and
//!   [`InstructionLifter`] trait objects.
//! - `dac-recovery::functions` supplies the [`FunctionSet`].
//! - `dac-ir::instr::InstructionIr` is the lifted form rendered by
//!   [`format_ir`].

use std::fmt::Write as _;

use dac_arch::{InstructionDecoder, InstructionLifter};
use dac_binfmt::{BinaryModel, Section};
use dac_ir::instr::{InstructionIr, Operand, Operation, Target};
use dac_recovery::{FunctionSet, SourceMask};

/// Knobs for [`render_listing`]. All flags default to `true` so the
/// canonical listing carries the fullest provenance view; CLI callers
/// turn things off when they want a tighter output (e.g. for diffs).
#[derive(Debug, Clone, Copy)]
pub(crate) struct ListingOptions {
    /// Include the per-function header block (address range,
    /// confidence, signals).
    pub with_headers: bool,
    /// Include the encoded-bytes column on every instruction.
    pub with_bytes: bool,
    /// Include the lifted IR projection on every instruction.
    pub with_ir: bool,
}

impl Default for ListingOptions {
    fn default() -> Self {
        Self {
            with_headers: true,
            with_bytes: true,
            with_ir: true,
        }
    }
}

/// Render the `-O0` annotated listing for the given binary + function
/// set.
///
/// The `input_name` is the path or label the listing should display in
/// its preamble; it does not have to match a real filesystem path.
#[must_use]
pub(crate) fn render_listing(
    input_name: &str,
    model: &BinaryModel,
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
    lifter: &dyn InstructionLifter,
    functions: &FunctionSet,
    opts: &ListingOptions,
) -> String {
    let mut out = String::new();
    render_preamble(&mut out, input_name, model, functions);

    let exec_sections: Vec<&Section> = model
        .sections
        .iter()
        .filter(|s| s.perms.executable && s.file_offset.is_some() && s.size > 0)
        .collect();

    if functions.functions.is_empty() {
        out.push_str(";; (no functions discovered)\n");
        return out;
    }

    for fn_rec in &functions.functions {
        if opts.with_headers {
            render_function_header(&mut out, fn_rec);
        }
        render_function_body(
            &mut out,
            fn_rec,
            &exec_sections,
            bytes,
            decoder,
            lifter,
            opts,
        );
        out.push('\n');
    }
    out
}

fn render_preamble(out: &mut String, input_name: &str, model: &BinaryModel, fns: &FunctionSet) {
    let _ = writeln!(out, ";; dac -O0 annotated listing");
    let _ = writeln!(out, ";; input:     {input_name}");
    let _ = writeln!(out, ";; format:    {}", model.format.name());
    let _ = writeln!(out, ";; arch:      {}", model.architecture.name());
    let _ = writeln!(
        out,
        ";; entry:     {}",
        model
            .entry
            .map_or_else(|| "(none)".to_string(), |e| format!("{e:#018x}"))
    );
    let _ = writeln!(out, ";; size:      {} bytes", model.size);
    let _ = writeln!(out, ";; functions: {}", fns.functions.len());
    let _ = writeln!(
        out,
        ";; signals:   symbol={} entry={} call={} prologue={}",
        fns.stats.from_symbol, fns.stats.from_entry, fns.stats.from_call, fns.stats.from_prologue,
    );
    out.push('\n');
}

fn render_function_header(out: &mut String, f: &dac_recovery::Function) {
    let name = f.name.as_deref().unwrap_or("<unnamed>");
    let end = f
        .end
        .map_or_else(|| "????".to_string(), |e| format!("{e:#018x}"));
    let size = f
        .size()
        .map_or_else(|| "?".to_string(), |n| format!("{n} bytes"));
    let bar = ";; ============================================================";
    let _ = writeln!(out, "{bar}");
    let _ = writeln!(
        out,
        ";; function {name} [{:#018x}..{end}) ({size})",
        f.address,
    );
    let _ = writeln!(
        out,
        ";;   confidence: {} {:.3}",
        f.confidence.source().name(),
        f.confidence.value(),
    );
    let _ = writeln!(out, ";;   sources:   {}", format_sources(f.sources));
    let _ = writeln!(out, "{bar}");
}

fn format_sources(mask: SourceMask) -> String {
    let mut parts: Vec<&'static str> = Vec::new();
    if mask.contains(SourceMask::SYMBOL) {
        parts.push("symbol");
    }
    if mask.contains(SourceMask::ENTRY) {
        parts.push("entry");
    }
    if mask.contains(SourceMask::CALL) {
        parts.push("call");
    }
    if mask.contains(SourceMask::PROLOGUE) {
        parts.push("prologue");
    }
    if parts.is_empty() {
        "(none)".to_string()
    } else {
        parts.join(", ")
    }
}

fn render_function_body(
    out: &mut String,
    f: &dac_recovery::Function,
    exec_sections: &[&Section],
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
    lifter: &dyn InstructionLifter,
    opts: &ListingOptions,
) {
    let Some(end) = f.end else {
        out.push_str(";;   (unknown extent; no instructions rendered)\n");
        return;
    };
    let Some((slice, slice_addr)) = function_bytes(f.address, end, exec_sections, bytes) else {
        out.push_str(";;   (function body outside any loaded executable section)\n");
        return;
    };
    for inst in decoder.iter(slice, slice_addr) {
        let ir = lifter.lift(&inst.bytes, inst.address);
        let mut line = String::new();
        let _ = write!(&mut line, "{:#018x}  ", inst.address);
        if opts.with_bytes {
            let _ = write!(&mut line, "{:<28}  ", format_bytes(&inst.bytes));
        }
        if inst.operands.is_empty() {
            let _ = write!(&mut line, "{:<10}", inst.mnemonic);
        } else {
            let _ = write!(&mut line, "{:<10} {}", inst.mnemonic, inst.operands);
        }
        if opts.with_ir {
            let ir_text = format_ir(&ir);
            // Pad so the IR column lines up regardless of operand width.
            let pad = if line.len() < 60 { 60 - line.len() } else { 1 };
            for _ in 0..pad {
                line.push(' ');
            }
            let _ = write!(&mut line, "; {ir_text}");
        }
        line.push('\n');
        out.push_str(&line);
    }
}

fn function_bytes<'a>(
    start: u64,
    end: u64,
    exec_sections: &[&Section],
    bytes: &'a [u8],
) -> Option<(&'a [u8], u64)> {
    let sec = exec_sections.iter().find(|s| {
        let s_start = s.address;
        let s_end = s_start.saturating_add(s.size);
        start >= s_start && start < s_end
    })?;
    let s_start = sec.address;
    let s_end = s_start.saturating_add(sec.size);
    let clamped_end = end.min(s_end);
    if clamped_end <= start {
        return None;
    }
    let file_off = usize::try_from(sec.file_offset?).ok()?;
    let in_sec_off = usize::try_from(start - s_start).ok()?;
    let length = usize::try_from(clamped_end - start).ok()?;
    let begin = file_off.checked_add(in_sec_off)?;
    let finish = begin.checked_add(length)?;
    if finish > bytes.len() {
        return None;
    }
    Some((&bytes[begin..finish], start))
}

fn format_bytes(bytes: &[u8]) -> String {
    const MAX: usize = 8;
    let mut s = String::with_capacity(bytes.len().min(MAX) * 3 + 1);
    for (i, b) in bytes.iter().take(MAX).enumerate() {
        if i > 0 {
            s.push(' ');
        }
        let _ = write!(s, "{b:02x}");
    }
    if bytes.len() > MAX {
        s.push_str(" .");
    }
    s
}

/// Project an [`InstructionIr`] onto a short textual form for the
/// listing comment column. Lifted ops render to their semantic shape so
/// the reader can compare the disassembly text and the lifted view side
/// by side; [`Operation::Opaque`] keeps the mnemonic so the gap is
/// visible.
#[must_use]
pub(crate) fn format_ir(ir: &InstructionIr) -> String {
    match &ir.op {
        Operation::Move { dst, src } => format!("mov({}, {})", op(dst), op(src)),
        Operation::LoadAddress { dst, src } => format!("lea({}, {})", op(dst), op(src)),
        Operation::Add { dst, src } => format!("add({}, {})", op(dst), op(src)),
        Operation::Sub { dst, src } => format!("sub({}, {})", op(dst), op(src)),
        Operation::Mul { dst, src } => format!("mul({}, {})", op(dst), op(src)),
        Operation::Div { dst, src } => format!("div({}, {})", op(dst), op(src)),
        Operation::And { dst, src } => format!("and({}, {})", op(dst), op(src)),
        Operation::Or { dst, src } => format!("or({}, {})", op(dst), op(src)),
        Operation::Xor { dst, src } => format!("xor({}, {})", op(dst), op(src)),
        Operation::Shl { dst, src } => format!("shl({}, {})", op(dst), op(src)),
        Operation::Shr { dst, src } => format!("shr({}, {})", op(dst), op(src)),
        Operation::Sar { dst, src } => format!("sar({}, {})", op(dst), op(src)),
        Operation::Not { dst } => format!("not({})", op(dst)),
        Operation::Neg { dst } => format!("neg({})", op(dst)),
        Operation::Compare { lhs, rhs } => format!("cmp({}, {})", op(lhs), op(rhs)),
        Operation::Test { lhs, rhs } => format!("test({}, {})", op(lhs), op(rhs)),
        Operation::Push { src } => format!("push({})", op(src)),
        Operation::Pop { dst } => format!("pop({})", op(dst)),
        Operation::Jump { target, condition } => match condition {
            Some(c) => format!("jcc.{c}({})", target_str(target)),
            None => format!("jmp({})", target_str(target)),
        },
        Operation::Call { target } => format!("call({})", target_str(target)),
        Operation::Return => "ret".to_string(),
        Operation::Nop => "nop".to_string(),
        Operation::Interrupt { vector } => match vector {
            Some(v) => format!("int({v})"),
            None => "int".to_string(),
        },
        Operation::Syscall => "syscall".to_string(),
        Operation::Opaque { mnemonic } => format!("opaque({mnemonic})"),
    }
}

fn op(o: &Operand) -> String {
    match o {
        Operand::Register { name, size_bits } => format!("{name}:{size_bits}"),
        Operand::Immediate { value, size_bits } => format!("{value}#{size_bits}"),
        Operand::Memory {
            base,
            index,
            scale,
            displacement,
            size_bits,
            segment,
        } => {
            let mut s = String::new();
            if let Some(seg) = segment {
                let _ = write!(s, "{seg}:");
            }
            s.push('[');
            let mut wrote = false;
            if let Some(b) = base {
                let _ = write!(s, "{b}");
                wrote = true;
            }
            if let Some(i) = index {
                if wrote {
                    s.push('+');
                }
                let _ = write!(s, "{i}*{scale}");
                wrote = true;
            }
            if *displacement != 0 || !wrote {
                if wrote && *displacement >= 0 {
                    s.push('+');
                }
                let _ = write!(s, "{displacement}");
            }
            let _ = write!(s, "]:{size_bits}");
            s
        }
        Operand::Branch { target } => format!("@{target:#x}"),
    }
}

fn target_str(t: &Target) -> String {
    match t {
        Target::Direct(addr) => format!("@{addr:#x}"),
        Target::Indirect(o) => op(o),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_binfmt::{
        Architecture, BinaryFormat, BinaryModel, Bits, Endian, Permissions, Section, SectionKind,
    };
    use dac_core::EvidenceGraph;
    use dac_recovery::discover_functions;

    fn empty_model() -> BinaryModel {
        BinaryModel {
            format: BinaryFormat::Elf,
            architecture: Architecture::X86_64,
            endian: Endian::Little,
            bits: Bits::Bits64,
            entry: None,
            size: 0,
            sections: vec![Section {
                name: ".text".to_string(),
                address: 0x1000,
                size: 0x10,
                file_offset: Some(0),
                perms: Permissions {
                    readable: true,
                    writable: false,
                    executable: true,
                },
                kind: SectionKind::Text,
            }],
            segments: Vec::new(),
            symbols: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            relocations: Vec::new(),
            strings: Vec::new(),
            needed_libraries: Vec::new(),
        }
    }

    struct NullDecoder;
    impl InstructionDecoder for NullDecoder {
        fn decode_one(
            &self,
            _bytes: &[u8],
            _address: u64,
        ) -> Result<dac_arch::DecodedInstruction, dac_arch::DecodeError> {
            Err(dac_arch::DecodeError::Truncated { offset: 0 })
        }
        fn iter<'a>(
            &'a self,
            _bytes: &'a [u8],
            _address: u64,
        ) -> Box<dyn Iterator<Item = dac_arch::DecodedInstruction> + 'a> {
            Box::new(std::iter::empty())
        }
    }

    struct OpaqueLifter;
    impl InstructionLifter for OpaqueLifter {
        fn lift(&self, _bytes: &[u8], address: u64) -> InstructionIr {
            InstructionIr {
                address,
                length: 0,
                op: Operation::Opaque {
                    mnemonic: "(stub)".to_string(),
                },
            }
        }
    }

    #[test]
    fn empty_function_set_renders_a_no_functions_note() {
        let model = empty_model();
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[], &NullDecoder, &mut g);
        let s = render_listing(
            "test",
            &model,
            &[],
            &NullDecoder,
            &OpaqueLifter,
            &set,
            &ListingOptions::default(),
        );
        assert!(s.contains("(no functions discovered)"));
        assert!(s.contains(";; functions: 0"));
        assert!(s.contains(";; format:    ELF"));
        assert!(s.contains(";; arch:      x86-64"));
    }

    #[test]
    fn rendering_is_deterministic_across_runs() {
        let mut model = empty_model();
        model.entry = Some(0x1000);
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[0u8; 0x10], &NullDecoder, &mut g);
        let a = render_listing(
            "test",
            &model,
            &[0u8; 0x10],
            &NullDecoder,
            &OpaqueLifter,
            &set,
            &ListingOptions::default(),
        );
        let b = render_listing(
            "test",
            &model,
            &[0u8; 0x10],
            &NullDecoder,
            &OpaqueLifter,
            &set,
            &ListingOptions::default(),
        );
        assert_eq!(a, b);
    }

    #[test]
    fn format_ir_matches_operation_shape() {
        let mov = InstructionIr {
            address: 0,
            length: 3,
            op: Operation::Move {
                dst: Operand::Register {
                    name: "rax".to_string(),
                    size_bits: 64,
                },
                src: Operand::Register {
                    name: "rbx".to_string(),
                    size_bits: 64,
                },
            },
        };
        assert_eq!(format_ir(&mov), "mov(rax:64, rbx:64)");
        let ret = InstructionIr {
            address: 0,
            length: 1,
            op: Operation::Return,
        };
        assert_eq!(format_ir(&ret), "ret");
        let opaque = InstructionIr {
            address: 0,
            length: 4,
            op: Operation::Opaque {
                mnemonic: "vfmadd".to_string(),
            },
        };
        assert_eq!(format_ir(&opaque), "opaque(vfmadd)");
    }

    #[test]
    fn format_sources_lists_active_bits_in_canonical_order() {
        let mut m = SourceMask::empty();
        m.insert(SourceMask::CALL);
        m.insert(SourceMask::SYMBOL);
        assert_eq!(format_sources(m), "symbol, call");
    }
}

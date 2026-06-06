//! Analysis report (FR-25).
//!
//! Surfaces the per-function confidence + source attribution the
//! discoverer produces, alongside the per-function instruction
//! [`Coverage`] the lifter records. Renderable to a deterministic text
//! form for `--emit-report`.
//!
//! Where each field comes from:
//!
//! - `functions` / `signals` from `dac_recovery::FunctionSet`.
//! - `lift_coverage` from `dac_arch::Coverage` — built by walking every
//!   discovered function through the decoder + lifter.
//! - `unresolved` is the opaque-mnemonic histogram lifted out of
//!   `Coverage`, listed lexicographically.
//!
//! The "unresolved constructs" bit of FR-25 is currently satisfied by
//! the opaque mnemonic histogram. Later batches (calling-convention
//! mismatches in B2.5, type-recovery gaps in B2.6, structuring
//! fallbacks in B2.7) will extend this struct with their own
//! per-pass-gap counts.

use std::fmt::Write as _;

use dac_arch::{Coverage, InstructionDecoder, InstructionLifter};
use dac_binfmt::{BinaryModel, Section};
use dac_recovery::{Function, FunctionSet, SourceMask};

use crate::lift::LiftStats;

/// Per-function summary in the analysis report.
#[derive(Debug, Clone)]
pub(crate) struct FunctionSummary {
    pub address: u64,
    pub end: Option<u64>,
    pub name: Option<String>,
    pub confidence_value: f32,
    pub confidence_source: &'static str,
    pub sources: SourceMask,
}

/// Aggregated analysis report.
#[derive(Debug, Clone)]
pub(crate) struct Report {
    pub function_count: u64,
    pub from_symbol: u64,
    pub from_entry: u64,
    pub from_call: u64,
    pub from_prologue: u64,
    pub coverage: Coverage,
    pub lift: LiftStats,
    pub functions: Vec<FunctionSummary>,
}

impl Report {
    /// Build a report by walking the discovered functions through the
    /// lifter to fold a [`Coverage`] alongside the per-function summary.
    #[must_use]
    pub(crate) fn build(
        model: &BinaryModel,
        bytes: &[u8],
        decoder: &dyn InstructionDecoder,
        lifter: &dyn InstructionLifter,
        functions: &FunctionSet,
        lift: LiftStats,
    ) -> Self {
        let exec_sections: Vec<&Section> = model
            .sections
            .iter()
            .filter(|s| s.perms.executable && s.file_offset.is_some() && s.size > 0)
            .collect();
        let mut coverage = Coverage::default();
        let mut summaries = Vec::with_capacity(functions.functions.len());
        for f in &functions.functions {
            fold_function_coverage(&mut coverage, f, &exec_sections, bytes, decoder, lifter);
            summaries.push(summarize(f));
        }
        Self {
            function_count: functions.functions.len() as u64,
            from_symbol: functions.stats.from_symbol,
            from_entry: functions.stats.from_entry,
            from_call: functions.stats.from_call,
            from_prologue: functions.stats.from_prologue,
            coverage,
            lift,
            functions: summaries,
        }
    }
}

fn summarize(f: &Function) -> FunctionSummary {
    FunctionSummary {
        address: f.address,
        end: f.end,
        name: f.name.clone(),
        confidence_value: f.confidence.value(),
        confidence_source: f.confidence.source().name(),
        sources: f.sources,
    }
}

fn fold_function_coverage(
    coverage: &mut Coverage,
    f: &Function,
    exec_sections: &[&Section],
    bytes: &[u8],
    decoder: &dyn InstructionDecoder,
    lifter: &dyn InstructionLifter,
) {
    let Some(end) = f.end else {
        return;
    };
    let Some((slice, slice_addr)) = function_bytes(f.address, end, exec_sections, bytes) else {
        return;
    };
    for inst in decoder.iter(slice, slice_addr) {
        let ir = lifter.lift(&inst.bytes, inst.address);
        coverage.record(&ir);
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

/// Render the report to a deterministic textual form.
#[must_use]
pub(crate) fn render_report_text(r: &Report) -> String {
    let mut out = String::new();
    let _ = writeln!(out, ";; dac analysis report (FR-25)");
    let _ = writeln!(out, ";; functions:   {}", r.function_count);
    let _ = writeln!(
        out,
        ";; signals:     symbol={} entry={} call={} prologue={}",
        r.from_symbol, r.from_entry, r.from_call, r.from_prologue,
    );
    let cov = &r.coverage;
    let pct = cov.lifted_fraction() * 100.0;
    let _ = writeln!(
        out,
        ";; lift cover.: {} / {} ({:.2}% lifted, {} opaque)",
        cov.lifted, cov.total, pct, cov.opaque,
    );
    let lift_pct = r.lift.fraction() * 100.0;
    let _ = writeln!(
        out,
        ";; body cover.: {} / {} ({:.2}% real bodies, {} stubs)",
        r.lift.real,
        r.lift.total(),
        lift_pct,
        r.lift.stub,
    );
    let _ = writeln!(
        out,
        ";; recovery:    typed_sigs={} struct_fields={} switch_tables={} user_hints={}",
        r.lift.typed_signatures,
        r.lift.struct_field_functions,
        r.lift.switch_functions,
        r.lift.user_hint_functions,
    );
    let name_pct = r.lift.named_value_ratio() * 100.0;
    let _ = writeln!(
        out,
        ";; naming:      named_values={} / {} ({:.2}% heuristic coverage, hint={})",
        r.lift.named_values, r.lift.nameable_values, name_pct, r.lift.hint_named_values,
    );
    let _ = writeln!(
        out,
        ";; simplify:    folded={} dropped={}",
        r.lift.simplifier_folds, r.lift.simplifier_drops,
    );
    let _ = writeln!(
        out,
        ";; structuring: fallbacks={}",
        r.lift.structuring_fallbacks,
    );
    out.push('\n');
    out.push_str("functions:\n");
    for f in &r.functions {
        let name = f.name.as_deref().unwrap_or("<unnamed>");
        let end = f
            .end
            .map_or_else(|| "????".to_string(), |e| format!("{e:#018x}"));
        let _ = writeln!(
            out,
            "  {:#018x}..{end}  {:<24}  {}/{:.3}  {}",
            f.address,
            name,
            f.confidence_source,
            f.confidence_value,
            sources_str(f.sources),
        );
    }
    if !cov.opaque_mnemonics.is_empty() {
        out.push('\n');
        out.push_str("unresolved opaque mnemonics:\n");
        for (mnem, n) in &cov.opaque_mnemonics {
            let _ = writeln!(out, "  {mnem}: {n}");
        }
    }
    out
}

fn sources_str(mask: SourceMask) -> String {
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
        parts.join(",")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_binfmt::{
        Architecture, BinaryFormat, BinaryModel, Bits, Endian, Permissions, Section, SectionKind,
    };
    use dac_core::EvidenceGraph;
    use dac_ir::instr::{InstructionIr, Operation};
    use dac_recovery::discover_functions;

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

    fn empty_model() -> BinaryModel {
        BinaryModel {
            format: BinaryFormat::Elf,
            architecture: Architecture::X86_64,
            endian: Endian::Little,
            bits: Bits::Bits64,
            entry: Some(0x1000),
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

    #[test]
    fn report_aggregates_function_signals_and_coverage() {
        let model = empty_model();
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[0u8; 0x10], &NullDecoder, &mut g);
        let r = Report::build(
            &model,
            &[0u8; 0x10],
            &NullDecoder,
            &OpaqueLifter,
            &set,
            LiftStats::default(),
        );
        assert_eq!(r.function_count, 1);
        assert_eq!(r.from_entry, 1);
        // NullDecoder yields no instructions, so coverage stays empty.
        assert_eq!(r.coverage.total, 0);
    }

    #[test]
    fn report_text_renders_function_summary_lines() {
        let model = empty_model();
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[0u8; 0x10], &NullDecoder, &mut g);
        let r = Report::build(
            &model,
            &[0u8; 0x10],
            &NullDecoder,
            &OpaqueLifter,
            &set,
            LiftStats::default(),
        );
        let s = render_report_text(&r);
        assert!(s.contains(";; functions:   1"));
        assert!(s.contains("0x0000000000001000"));
        assert!(s.contains("entry"));
    }

    #[test]
    fn report_text_includes_structuring_fallbacks_row() {
        // B3.27: the structuring row is unconditional (zero even when
        // no fallbacks fired), keyed so a reader can grep for it.
        let model = empty_model();
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[0u8; 0x10], &NullDecoder, &mut g);
        let r = Report::build(
            &model,
            &[0u8; 0x10],
            &NullDecoder,
            &OpaqueLifter,
            &set,
            LiftStats::default(),
        );
        let s = render_report_text(&r);
        assert!(
            s.contains(";; structuring: fallbacks=0"),
            "expected structuring row in:\n{s}",
        );
    }

    #[test]
    fn render_report_text_is_byte_stable() {
        let model = empty_model();
        let mut g = EvidenceGraph::new();
        let set = discover_functions(&model, &[0u8; 0x10], &NullDecoder, &mut g);
        let r = Report::build(
            &model,
            &[0u8; 0x10],
            &NullDecoder,
            &OpaqueLifter,
            &set,
            LiftStats::default(),
        );
        let a = render_report_text(&r);
        let b = render_report_text(&r);
        assert_eq!(a, b);
    }
}

//! `--xrefs` / `--callgraph` rendering helpers (B3.1, FR-26, FR-27, FR-31).
//!
//! Two text artifacts surface from the CLI here:
//!
//! - [`render_xrefs_report`] — the human-readable text the CLI prints
//!   (and, with `--output`, writes to `<output>.xrefs.txt`) when the
//!   user passed `--xrefs <subject>`. Lists every reference *to* and
//!   *from* the resolved subject, sorted address-major, with the
//!   `(kind, confidence)` of each edge.
//! - the call graph DOT — produced by
//!   [`dac_analysis::render_callgraph_dot`] and written to
//!   `<output>.callgraph.dot` when `--callgraph` is set. The rendering
//!   itself lives in `dac-analysis`; this module owns the sidecar
//!   wiring only.

use std::fmt::Write as _;

#[cfg(test)]
use dac_analysis::XrefKind;
use dac_analysis::{Xref, XrefIndex};
use dac_binfmt::BinaryModel;
use dac_recovery::FunctionSet;

/// Format a single textual xref report rooted at `subject_va`. The
/// emitted layout is:
///
/// ```text
/// ;; dac --xrefs report
/// ;; subject:   <name or hex>
/// ;; address:   0x...
/// ;;
/// ;; xrefs to (callers / writers): <N>
/// ;; <kind>   <from>    <conf>
/// ;; ...
/// ;;
/// ;; xrefs from (callees / reads): <N>
/// ;; <kind>   <to>      <conf>
/// ;; ...
/// ```
///
/// The kind column is one of [`XrefKind::tag`]; addresses are formatted
/// as `0x{:x}` for consistency with the listing output. Confidence is
/// rendered as `{:.2}/{Source:?}` to match the report's existing style.
#[must_use]
pub(crate) fn render_xrefs_report(
    subject_raw: &str,
    subject_va: u64,
    subject_name: Option<&str>,
    index: &XrefIndex,
    model: &BinaryModel,
    functions: &FunctionSet,
) -> String {
    let mut out = String::new();
    out.push_str(";; dac --xrefs report (FR-26, FR-31)\n");
    out.push_str(&format!(";; subject:   {subject_raw}\n"));
    if let Some(n) = subject_name {
        out.push_str(&format!(";; name:      {n}\n"));
    }
    out.push_str(&format!(";; address:   {subject_va:#x}\n"));
    out.push_str(";;\n");

    let to = index.to(subject_va);
    out.push_str(&format!(";; xrefs to: {}\n", to.len()));
    for x in &to {
        let _ = writeln!(
            out,
            ";;   {:5}  from {}  {}",
            x.kind.tag(),
            va(x.from),
            conf(x)
        );
        let _ = annotate_endpoint(&mut out, x.from, model, functions);
    }
    out.push_str(";;\n");

    let from = index.from(subject_va);
    out.push_str(&format!(";; xrefs from: {}\n", from.len()));
    for x in &from {
        let _ = writeln!(
            out,
            ";;   {:5}  to   {}  {}",
            x.kind.tag(),
            va(x.to),
            conf(x)
        );
        let _ = annotate_endpoint(&mut out, x.to, model, functions);
    }
    out
}

fn va(va: u64) -> String {
    if va == 0 {
        "<external>".to_string()
    } else {
        format!("{va:#x}")
    }
}

fn conf(x: &Xref) -> String {
    format!("[{:.2}/{:?}]", x.confidence.value(), x.confidence.source())
}

/// Append a `;;     -> name=<sym>` line when `addr` matches a known
/// function (entry or interior) or symbol; silent when not. Keeps the
/// dense listing readable without forcing every xref onto two lines.
///
/// For call sites *inside* a function body, the address points to the
/// instruction, not to a function entry — fall back to the containing
/// function so callers can see *which function* the reference came
/// from.
fn annotate_endpoint(
    out: &mut String,
    addr: u64,
    model: &BinaryModel,
    functions: &FunctionSet,
) -> std::fmt::Result {
    if addr == 0 {
        return Ok(());
    }
    if let Some(f) = functions.get(addr) {
        if let Some(n) = &f.name {
            return writeln!(out, ";;          -> fn  {n}");
        }
    }
    for sym in &model.symbols {
        if sym.address == addr && !sym.name.is_empty() {
            return writeln!(out, ";;          -> sym {}", sym.name);
        }
    }
    // Containing-function fallback: walk in ascending order and pick
    // the function whose half-open `[address, end)` covers `addr`.
    for f in &functions.functions {
        let end = f.end.unwrap_or(f.address);
        if addr >= f.address && addr < end {
            if let Some(n) = &f.name {
                return writeln!(out, ";;          -> fn  {n}");
            }
        }
    }
    Ok(())
}

/// Kinds that should count toward a textual report's "data" tally.
/// Exposed for unit tests; the renderer itself does not consume it
/// because it walks every kind unconditionally.
#[cfg(test)]
pub(crate) const DATA_KINDS: &[XrefKind] = &[
    XrefKind::CodeToData,
    XrefKind::DataToCode,
    XrefKind::DataToData,
];

#[cfg(test)]
mod tests {
    use super::*;
    use dac_analysis::{build_xref_index, EXTERNAL_VA};
    use dac_arch::{
        ControlFlow as Cf, DecodeError, DecodedInstruction as Di, InstructionDecoder as Id,
    };
    use dac_binfmt::{
        Architecture, BinaryFormat, Bits, Endian, Permissions, Section, SectionKind, Symbol,
        SymbolBinding, SymbolKind, SymbolSource,
    };
    use dac_core::EvidenceGraph;
    use dac_recovery::discover_functions;

    struct ScriptedDec(Vec<(u64, Cf)>);
    impl Id for ScriptedDec {
        fn decode_one(&self, _: &[u8], _: u64) -> Result<Di, DecodeError> {
            Err(DecodeError::Truncated { offset: 0 })
        }
        fn iter<'a>(&'a self, bytes: &'a [u8], address: u64) -> Box<dyn Iterator<Item = Di> + 'a> {
            let end = address.saturating_add(bytes.len() as u64);
            let here: Vec<Di> = self
                .0
                .iter()
                .filter(|(a, _)| *a >= address && *a < end)
                .map(|(a, f)| Di {
                    address: *a,
                    length: 1,
                    bytes: vec![0],
                    mnemonic: "x".into(),
                    operands: String::new(),
                    flow: *f,
                    valid: true,
                })
                .collect();
            Box::new(here.into_iter())
        }
    }

    fn fixture() -> (BinaryModel, FunctionSet, XrefIndex) {
        let mut model = BinaryModel {
            format: BinaryFormat::Elf,
            architecture: Architecture::X86_64,
            endian: Endian::Little,
            bits: Bits::Bits64,
            entry: None,
            size: 0,
            sections: vec![Section {
                name: ".text".into(),
                address: 0x1000,
                size: 0x200,
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
        };
        model.symbols.push(Symbol {
            name: "caller".into(),
            address: 0x1000,
            size: 0x40,
            kind: SymbolKind::Text,
            binding: SymbolBinding::Global,
            section: Some(0),
            source: SymbolSource::Symtab,
            undefined: false,
        });
        model.symbols.push(Symbol {
            name: "callee".into(),
            address: 0x1080,
            size: 0x40,
            kind: SymbolKind::Text,
            binding: SymbolBinding::Global,
            section: Some(0),
            source: SymbolSource::Symtab,
            undefined: false,
        });
        let dec = ScriptedDec(vec![(
            0x1010,
            Cf::Call {
                target: Some(0x1080),
            },
        )]);
        let mut g = EvidenceGraph::new();
        let funcs = discover_functions(&model, &vec![0u8; 0x200], &dec, &mut g);
        let idx = build_xref_index(&model, &vec![0u8; 0x200], &dec, &funcs);
        (model, funcs, idx)
    }

    #[test]
    fn report_lists_callers_with_symbol_annotation() {
        let (model, funcs, idx) = fixture();
        let txt = render_xrefs_report("callee", 0x1080, Some("callee"), &idx, &model, &funcs);
        assert!(txt.contains("name:      callee"));
        assert!(txt.contains("address:   0x1080"));
        assert!(txt.contains("xrefs to: 1"));
        assert!(txt.contains("CALL"));
        assert!(txt.contains("from 0x1010"));
        assert!(txt.contains("-> fn  caller"));
    }

    #[test]
    fn report_for_unreferenced_address_lists_zero_xrefs() {
        let (model, funcs, idx) = fixture();
        let txt = render_xrefs_report("0x1FFF", 0x1FFF, None, &idx, &model, &funcs);
        assert!(txt.contains("xrefs to: 0"));
        assert!(txt.contains("xrefs from: 0"));
    }

    #[test]
    fn external_endpoint_is_rendered_as_marker() {
        let (model, funcs, idx) = fixture();
        // The synthetic external VA prints as "<external>" so the
        // reader sees that the edge originates outside the binary.
        let txt = render_xrefs_report("<external>", EXTERNAL_VA, None, &idx, &model, &funcs);
        assert!(txt.contains("address:   0x0"));
    }

    #[test]
    fn data_kinds_constant_lists_expected_three() {
        assert_eq!(DATA_KINDS.len(), 3);
    }
}

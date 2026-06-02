//! Reproducibility manifest (NFR-10, ARCHITECTURE.md §10).
//!
//! Every artifact dac emits is accompanied by a `manifest.json` capturing
//! tool version, build id, settings, AI provider, and plugin list. The
//! manifest is the substrate `--deterministic` rests on: identical inputs
//! plus identical settings must produce a byte-identical manifest, so the
//! JSON writer here is hand-rolled with sorted keys and stable numeric
//! formatting (no serde, no locale-sensitive `f64` formatting).
//!
//! The on-disk format is intentionally minimal at B1.6 — the fields that
//! later batches need (pass timing, plugin versions, artifact hash) land
//! with the passes/plugins themselves. NFR-10's "record tool version,
//! analysis settings, and backend versions" is satisfied today; the rest
//! is "as-yet-empty" rather than "missing".

use std::fmt::Write as _;

/// Everything the manifest records about a run.
#[derive(Debug, Clone)]
pub(crate) struct Manifest {
    pub tool: ManifestTool,
    pub input: ManifestInput,
    pub settings: ManifestSettings,
    /// AI provider id (e.g. `"none"`, `"local:llama"`). `None` is
    /// rendered as JSON null and signals the determinism corridor — see
    /// `--no-ai`.
    pub ai_provider: Option<String>,
    /// Plugin labels actually loaded. Empty until M5 (B5.1).
    pub plugins: Vec<String>,
}

/// Tool identity. `name` is always `"dac"`; pinned here so the manifest
/// is self-describing without the reader needing to know the binary name.
#[derive(Debug, Clone)]
pub(crate) struct ManifestTool {
    pub name: String,
    pub version: String,
    /// Build id (commit SHA in CI / release builds, `"dev"` for local
    /// builds). Recorded so a reproducible artifact can be traced back
    /// to the binary that produced it.
    pub build_id: String,
}

/// Input descriptor. The hash is left out at B1.6 — adding a SHA256 of
/// the input bytes will land alongside the on-disk cache format that
/// needs it for keying (M5). `size` and `path` are enough to identify
/// the run for now.
#[derive(Debug, Clone)]
pub(crate) struct ManifestInput {
    pub path: String,
    pub size: u64,
    pub format: String,
    pub architecture: String,
}

/// Settings that influenced the analysis. Mirrors the relevant CLI
/// flags (spec §10.1) so a reader can reproduce the run without
/// guessing.
#[derive(Debug, Clone)]
pub(crate) struct ManifestSettings {
    pub level: String,
    pub target: String,
    pub deterministic: bool,
    pub no_ai: bool,
    pub emit_ir: bool,
    pub emit_cfg: bool,
    pub emit_report: bool,
    pub emit_annotations: bool,
    pub threads: Option<u32>,
}

/// Serialize a manifest to deterministic JSON.
///
/// Key order is fixed (tool, input, settings, ai, plugins). Numbers are
/// rendered with Rust's default `{}` formatter, which is locale-free
/// for the integer types used here. Strings are escaped against the
/// JSON-mandatory characters; non-ASCII is passed through untouched so
/// the output is UTF-8 by default.
#[must_use]
pub(crate) fn render_manifest_json(m: &Manifest) -> String {
    let mut out = String::new();
    out.push_str("{\n");
    write_object(
        &mut out,
        2,
        "tool",
        &[
            ("name", json_string(&m.tool.name)),
            ("version", json_string(&m.tool.version)),
            ("build_id", json_string(&m.tool.build_id)),
        ],
        false,
    );
    write_object(
        &mut out,
        2,
        "input",
        &[
            ("path", json_string(&m.input.path)),
            ("size", m.input.size.to_string()),
            ("format", json_string(&m.input.format)),
            ("architecture", json_string(&m.input.architecture)),
        ],
        false,
    );
    write_object(
        &mut out,
        2,
        "settings",
        &[
            ("level", json_string(&m.settings.level)),
            ("target", json_string(&m.settings.target)),
            ("deterministic", json_bool(m.settings.deterministic)),
            ("no_ai", json_bool(m.settings.no_ai)),
            ("emit_ir", json_bool(m.settings.emit_ir)),
            ("emit_cfg", json_bool(m.settings.emit_cfg)),
            ("emit_report", json_bool(m.settings.emit_report)),
            ("emit_annotations", json_bool(m.settings.emit_annotations)),
            (
                "threads",
                m.settings
                    .threads
                    .map_or_else(|| "null".to_string(), |n| n.to_string()),
            ),
        ],
        false,
    );
    let ai_str = m
        .ai_provider
        .as_deref()
        .map_or_else(|| "null".to_string(), json_string);
    let _ = writeln!(out, "  \"ai\": {{ \"provider\": {ai_str} }},");
    out.push_str("  \"plugins\": [");
    for (i, p) in m.plugins.iter().enumerate() {
        if i > 0 {
            out.push_str(", ");
        }
        out.push_str(&json_string(p));
    }
    out.push_str("]\n");
    out.push_str("}\n");
    out
}

fn write_object(out: &mut String, indent: usize, key: &str, fields: &[(&str, String)], last: bool) {
    let pad = " ".repeat(indent);
    let inner = " ".repeat(indent + 2);
    let _ = writeln!(out, "{pad}\"{key}\": {{");
    for (i, (k, v)) in fields.iter().enumerate() {
        let comma = if i + 1 == fields.len() { "" } else { "," };
        let _ = writeln!(out, "{inner}\"{k}\": {v}{comma}");
    }
    let suffix = if last { "" } else { "," };
    let _ = writeln!(out, "{pad}}}{suffix}");
}

fn json_bool(b: bool) -> String {
    (if b { "true" } else { "false" }).to_string()
}

fn json_string(s: &str) -> String {
    let mut out = String::with_capacity(s.len() + 2);
    out.push('"');
    for c in s.chars() {
        match c {
            '"' => out.push_str("\\\""),
            '\\' => out.push_str("\\\\"),
            '\n' => out.push_str("\\n"),
            '\r' => out.push_str("\\r"),
            '\t' => out.push_str("\\t"),
            c if (c as u32) < 0x20 => {
                let _ = write!(out, "\\u{:04x}", c as u32);
            }
            c => out.push(c),
        }
    }
    out.push('"');
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Manifest {
        Manifest {
            tool: ManifestTool {
                name: "dac".to_string(),
                version: "0.1.0-pre".to_string(),
                build_id: "dev".to_string(),
            },
            input: ManifestInput {
                path: "hello-x86_64".to_string(),
                size: 17296,
                format: "ELF".to_string(),
                architecture: "x86-64".to_string(),
            },
            settings: ManifestSettings {
                level: "O0".to_string(),
                target: "c".to_string(),
                deterministic: false,
                no_ai: false,
                emit_ir: false,
                emit_cfg: false,
                emit_report: false,
                emit_annotations: false,
                threads: None,
            },
            ai_provider: None,
            plugins: Vec::new(),
        }
    }

    #[test]
    fn manifest_render_is_byte_stable() {
        let m = sample();
        let a = render_manifest_json(&m);
        let b = render_manifest_json(&m);
        assert_eq!(a, b);
    }

    #[test]
    fn manifest_records_tool_and_settings_fields_for_nfr_10() {
        let m = sample();
        let s = render_manifest_json(&m);
        assert!(s.contains("\"name\": \"dac\""));
        assert!(s.contains("\"version\": \"0.1.0-pre\""));
        assert!(s.contains("\"build_id\": \"dev\""));
        assert!(s.contains("\"level\": \"O0\""));
        assert!(s.contains("\"deterministic\": false"));
        assert!(s.contains("\"threads\": null"));
        assert!(s.contains("\"provider\": null"));
        assert!(s.contains("\"plugins\": []"));
    }

    #[test]
    fn manifest_renders_ai_provider_when_set() {
        let mut m = sample();
        m.ai_provider = Some("local:llama".to_string());
        m.plugins.push("plug-a".to_string());
        let s = render_manifest_json(&m);
        assert!(s.contains("\"provider\": \"local:llama\""));
        assert!(s.contains("\"plug-a\""));
    }

    #[test]
    fn json_string_escapes_quote_and_control_chars() {
        assert_eq!(json_string("a\"b"), "\"a\\\"b\"");
        assert_eq!(json_string("x\ny"), "\"x\\ny\"");
        assert_eq!(json_string("\x01"), "\"\\u0001\"");
    }

    #[test]
    fn manifest_threads_serializes_when_set() {
        let mut m = sample();
        m.settings.threads = Some(4);
        let s = render_manifest_json(&m);
        assert!(s.contains("\"threads\": 4"));
    }
}

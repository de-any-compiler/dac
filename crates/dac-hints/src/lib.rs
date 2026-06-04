//! `dac-hints` — user-supplied function-signature, struct, and
//! type-hint catalogue (B3.6, FR-20).
//!
//! The decompiler treats the binary as ground truth (I-1), but the
//! recovered name / type lattice is often improved by a reverse
//! engineer who already knows what a given function is or what a
//! given struct holds. This crate parses a tiny constrained TOML
//! dialect describing those hints and converts each entry into a
//! [`Hint`] that downstream passes can consult by function address
//! or recovered symbol name.
//!
//! Once a hint is loaded its [`HintId`] enters the
//! [`dac_core::EvidenceGraph`] as an [`dac_core::EvidenceNode::UserHint`]
//! node (I-2), and any retyping it drives carries `Source::UserHint`
//! confidence (I-3). The lift pipeline's `--emit-report` then exposes
//! the per-binary count, satisfying the B3.6 "reflected in the
//! confidence report" criterion.
//!
//! ## Schema (strict TOML subset)
//!
//! ```toml
//! # Per-function hint. Either `address` or `name` is required; both
//! # may appear and both must match.
//! [[function]]
//! address = "0x1040"
//! name = "main"
//! rename = "user_main"     # optional: replaces the emitted symbol
//! return = "int"           # optional: function's C return type
//! args = ["int", "char**"] # optional: positional argument types
//!
//! # Per-struct hint. Reserved at B3.6: parsed and recorded as a
//! # UserHint evidence node, but lowering only consumes function
//! # hints today. Struct hints will be applied once the C backend
//! # grows translation-unit-level struct typedefs (B3 follow-up).
//! [[struct]]
//! name = "Point"
//! fields = [
//!     { name = "x", offset = "0x0", ty = "int32" },
//!     { name = "y", offset = "0x4", ty = "int32" },
//! ]
//! ```
//!
//! The parser is intentionally narrow: only `[[table]]` array-of-
//! tables headers, `key = "string"`, `key = number`, `key = [array]`,
//! and inline `{ k = v, k = v }` tables. Bare hash comments. Keys
//! and table names are bare ASCII identifiers. Strings support
//! `\"`, `\\`, `\n`, `\t` escapes. This is enough for the schema
//! above without bringing a full TOML implementation into the
//! workspace.
//!
//! ## Type-string grammar
//!
//! [`HintType::parse`] accepts a small grammar that lines up with
//! the C surface the backend already prints:
//!
//! ```text
//! type := atom ('*' )*
//! atom := "void"
//!       | "char"
//!       | "int" | "long" | "short"
//!       | "int8" | "int16" | "int32" | "int64"
//!       | "uint8" | "uint16" | "uint32" | "uint64"
//! ```
//!
//! Pointer suffixes can be repeated (`int**` is `int**`). Whitespace
//! between the atom and the asterisks is permitted.
//!
//! ## Determinism (NFR-9)
//!
//! The loader walks the input top-to-bottom; the produced [`Hints`]
//! preserves source order so two runs on the same file yield
//! identical [`Hint::id`] assignments. The parser is pure and never
//! consults a clock, environment, or filesystem-iteration order.

#![forbid(unsafe_code)]

use std::fmt;
use std::path::Path;

use dac_core::EvidenceId;
use dac_ir::ty::{IntType, Signedness, Type as IrType};

mod parse;

#[cfg(test)]
mod tests;

pub use parse::parse_toml;

/// Identifier assigned to each parsed [`Hint`] in source order. The
/// CLI registers the same number as the payload of an
/// [`dac_core::EvidenceNode::UserHint`] node so the annotation channel
/// can cross-reference back to the hint that produced a retyping.
pub type HintId = u64;

/// One parsed hint file.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Hints {
    /// Per-function hints in the order they appeared in the source
    /// file.
    pub functions: Vec<FunctionHint>,
    /// Per-struct hints in the order they appeared in the source
    /// file. Parsed and counted but not applied at B3.6 (see
    /// `PLAN.md`'s B3 follow-up shelf — struct typedef surface).
    pub structs: Vec<StructHint>,
}

impl Hints {
    /// Empty hint set — convenient default for the CLI pipeline when
    /// the user did not pass `--hints`.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Total number of parsed hints. Surfaces in `--emit-report`'s
    /// recovery row to satisfy the B3.6 "reflected in the confidence
    /// report" criterion.
    #[must_use]
    pub fn len(&self) -> usize {
        self.functions.len() + self.structs.len()
    }

    /// True when no hints were parsed.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.functions.is_empty() && self.structs.is_empty()
    }

    /// Find a function hint matching the recovered function's
    /// address or name. An entry with both an `address` and a `name`
    /// matcher requires both to match the candidate; an entry with
    /// only one matcher uses that single field.
    #[must_use]
    pub fn find_function(&self, address: u64, name: Option<&str>) -> Option<&FunctionHint> {
        self.functions
            .iter()
            .find(|h| h.matcher.matches(address, name))
    }

    /// Load and parse a hint file from `path`. The full file is read
    /// into memory before parsing — hint files are small in practice.
    pub fn load_from_path(path: &Path) -> Result<Self, HintError> {
        let bytes = std::fs::read(path).map_err(|e| HintError::Io {
            path: path.display().to_string(),
            message: e.to_string(),
        })?;
        let text = std::str::from_utf8(&bytes).map_err(|_| HintError::NotUtf8 {
            path: path.display().to_string(),
        })?;
        parse_toml(text)
    }
}

/// Per-function hint. Each entry can rename the emitted symbol, pin
/// the function's return type, and pin its positional argument
/// types. Unset fields fall back to the deterministic recovery
/// passes' inferences.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FunctionHint {
    /// Stable identifier in source order. Mirrors the payload of
    /// the [`dac_core::EvidenceNode::UserHint`] node minted when the
    /// hint enters the evidence graph.
    pub id: HintId,
    /// Source-file line number where the `[[function]]` header
    /// appears. Used to seed `EvidenceNode::Bytes` spans and for
    /// error reporting.
    pub line: u32,
    /// Matching criterion: address, recovered symbol name, or both.
    pub matcher: HintMatcher,
    /// Optional override for the emitted C identifier. The CLI runs
    /// the override through its sanitiser before printing.
    pub rename: Option<String>,
    /// Optional override for the recovered return type.
    pub return_ty: Option<HintType>,
    /// Optional override for the positional argument types. The
    /// `i`-th entry pins the type of `int_args[i]`.
    pub args: Option<Vec<HintType>>,
    /// Evidence-graph node the CLI minted for this hint. Defaults
    /// to `None` until the orchestrator registers the hint; tests
    /// and the parser leave it `None`.
    pub evidence: Option<EvidenceId>,
}

/// Per-struct hint — parsed for completeness, applied later (B3
/// follow-up shelf entry: struct typedef surface).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructHint {
    pub id: HintId,
    pub line: u32,
    pub name: String,
    pub fields: Vec<StructFieldHint>,
    pub evidence: Option<EvidenceId>,
}

/// One field inside a [`StructHint`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructFieldHint {
    pub name: String,
    pub offset: u64,
    pub ty: HintType,
}

/// How a [`FunctionHint`] matches a recovered function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HintMatcher {
    /// Match by function virtual address only.
    Address(u64),
    /// Match by recovered symbol name only.
    Name(String),
    /// Both fields must match.
    Both { address: u64, name: String },
}

impl HintMatcher {
    /// Return `true` when `(address, name)` describes a function
    /// the matcher applies to. A `None` name never matches a
    /// `Name` / `Both` matcher.
    #[must_use]
    pub fn matches(&self, address: u64, name: Option<&str>) -> bool {
        match self {
            Self::Address(a) => *a == address,
            Self::Name(n) => name == Some(n.as_str()),
            Self::Both {
                address: a,
                name: n,
            } => *a == address && name == Some(n.as_str()),
        }
    }
}

/// One hint-side type. Maps onto [`IrType`] via [`HintType::to_ir`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HintType {
    /// `void` — used in `return` only; arg slots reject `void` at
    /// parse time.
    Void,
    /// Fixed-width integer. `signed: None` means the user wrote a
    /// signedness-agnostic atom (`int`, `int64`); `Some(true)`
    /// signed; `Some(false)` unsigned.
    Int {
        width_bits: u16,
        signed: Option<bool>,
    },
    /// Pointer to a pointee. `Box` keeps the enum small.
    Ptr(Box<HintType>),
}

impl HintType {
    /// Parse a type string written in the grammar above.
    pub fn parse(s: &str) -> Result<Self, HintError> {
        parse::parse_type(s)
    }

    /// Lower the hint type into the IR type lattice.
    #[must_use]
    pub fn to_ir(&self) -> IrType {
        match self {
            HintType::Void => IrType::Unknown,
            HintType::Int { width_bits, signed } => IrType::Int(IntType {
                width_bits: *width_bits,
                signedness: match signed {
                    None => Signedness::Unknown,
                    Some(true) => Signedness::Signed,
                    Some(false) => Signedness::Unsigned,
                },
            }),
            HintType::Ptr(p) => IrType::Ptr(Box::new(p.to_ir())),
        }
    }

    /// `true` for [`HintType::Void`].
    #[must_use]
    pub const fn is_void(&self) -> bool {
        matches!(self, HintType::Void)
    }
}

impl fmt::Display for HintType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HintType::Void => f.write_str("void"),
            HintType::Int { width_bits, signed } => match signed {
                None => write!(f, "int{width_bits}"),
                Some(true) => write!(f, "int{width_bits}_t"),
                Some(false) => write!(f, "uint{width_bits}_t"),
            },
            HintType::Ptr(p) => write!(f, "{p}*"),
        }
    }
}

/// Errors produced while loading or parsing a hint file.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HintError {
    Io {
        path: String,
        message: String,
    },
    NotUtf8 {
        path: String,
    },
    /// Syntactic problem encountered by the strict-TOML reader.
    Syntax {
        line: u32,
        message: String,
    },
    /// The hint refers to a type or matcher shape the loader does
    /// not yet support.
    Semantic {
        line: u32,
        message: String,
    },
}

impl HintError {
    /// One-line, human-readable summary. The CLI prints this to
    /// stderr and exits with the standard usage-error code so the
    /// reverse engineer sees what was wrong with their hint file.
    #[must_use]
    pub fn message(&self) -> String {
        match self {
            HintError::Io { path, message } => {
                format!("{path}: could not read hints file: {message}")
            }
            HintError::NotUtf8 { path } => format!("{path}: hints file is not valid UTF-8"),
            HintError::Syntax { line, message } => format!("hints:{line}: syntax error: {message}"),
            HintError::Semantic { line, message } => format!("hints:{line}: {message}"),
        }
    }
}

impl fmt::Display for HintError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message())
    }
}

impl std::error::Error for HintError {}

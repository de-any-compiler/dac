//! `dac-core` — orchestrator, pass manager, evidence graph, and confidence
//! lattice for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace root.
//!
//! Status: B0.3 lands the evidence graph and confidence lattice
//! (invariants I-2 and I-3). The pass manager lands with B0.4.

#![forbid(unsafe_code)]

mod confidence;
mod error;
mod evidence;
mod tracing_init;

pub use confidence::{Confidence, Source};
pub use error::{Error, Result};
pub use evidence::{Edge, EdgeKind, EvidenceGraph, EvidenceId, EvidenceNode, IrLayer};
pub use tracing_init::init_tracing;

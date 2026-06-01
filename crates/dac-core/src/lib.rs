//! `dac-core` — orchestrator, pass manager, evidence graph, and confidence
//! lattice for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace root.
//!
//! Status: B0.4 lands the pass manager skeleton on top of the B0.3
//! evidence graph and confidence lattice. The first real passes plug into
//! the manager from B1.x onward.

#![forbid(unsafe_code)]

mod confidence;
mod error;
mod evidence;
mod pass;
mod tracing_init;

pub use confidence::{Confidence, Source};
pub use error::{Error, Result};
pub use evidence::{Edge, EdgeKind, EvidenceGraph, EvidenceId, EvidenceNode, IrLayer};
pub use pass::{
    ArtifactKind, ArtifactStore, Determinism, Pass, PassContext, PassId, PassManager, PassOutcome,
    RunReport,
};
pub use tracing_init::init_tracing;

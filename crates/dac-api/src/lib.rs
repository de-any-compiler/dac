//! `dac-api` — public library API for embedding dac in other tools.
//!
//! This crate is the only stable, semver-respecting surface. Every other
//! crate is implementation detail and may change between 0.x releases.
//!
//! Status: B0.3 added the evidence + confidence types; B0.4 adds the pass
//! manager skeleton. The wider surface (binary models, real passes,
//! backends) lands batch-by-batch as those crates fill in.

#![forbid(unsafe_code)]

pub use dac_artifact::ArtifactCache;
pub use dac_core::{
    ArtifactKind, ArtifactStore, Confidence, Determinism, Edge, EdgeKind, Error, EvidenceGraph,
    EvidenceId, EvidenceNode, IrLayer, Pass, PassContext, PassId, PassManager, PassOutcome, Result,
    RunReport, Source,
};

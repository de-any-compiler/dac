//! `dac-api` — public library API for embedding dac in other tools.
//!
//! This crate is the only stable, semver-respecting surface. Every other
//! crate is implementation detail and may change between 0.x releases.
//!
//! Status: B0.3 re-exports the core types every embedder needs to talk
//! about provenance and confidence. The wider surface (binary models,
//! passes, backends) lands batch-by-batch as those crates fill in.

#![forbid(unsafe_code)]

pub use dac_core::{
    Confidence, Edge, EdgeKind, Error, EvidenceGraph, EvidenceId, EvidenceNode, IrLayer, Result,
    Source,
};

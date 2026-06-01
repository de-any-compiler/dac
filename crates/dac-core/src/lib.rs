//! `dac-core` — orchestrator, pass manager, evidence graph, and confidence
//! lattice for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` in the workspace root.
//!
//! Status: B0.2 adds the project [`Error`] enum, [`Result`] alias, and
//! [`init_tracing`]. Evidence graph and confidence lattice land with B0.3;
//! the pass manager lands with B0.4.

#![forbid(unsafe_code)]

mod error;
mod tracing_init;

pub use error::{Error, Result};
pub use tracing_init::init_tracing;

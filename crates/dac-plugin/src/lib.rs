//! `dac-plugin` — dynamic plugin loading and stable plugin ABI for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` §12 in the workspace
//! root. The dynamic ABI will require `unsafe` (FFI); when it lands, this
//! crate flips its `#![forbid(unsafe_code)]` to per-block allows.
//!
//! Status: stub. ABI freezes with `B5.1`.

#![forbid(unsafe_code)]

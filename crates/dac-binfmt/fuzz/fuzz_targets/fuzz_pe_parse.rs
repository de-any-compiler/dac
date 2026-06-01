//! `fuzz_pe_parse` — coverage-guided fuzz target for the PE path of
//! `dac_binfmt::load_from_bytes`.
//!
//! Invariant under test (NFR-4): the PE parser, like the ELF parser,
//! may return an error on any input but must never panic, never abort,
//! and never trigger UB.
//!
//! Run from `crates/dac-binfmt/`:
//!
//! ```text
//! cargo install cargo-fuzz
//! cargo +nightly fuzz run fuzz_pe_parse -- -max_total_time=300
//! ```
//!
//! The 5-minute total-time cap satisfies the B1.2 done-when. Use the
//! `tests/fixtures/*.exe` and `*.dll` files as starting seeds in
//! `corpus/fuzz_pe_parse/`.

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Both entry points are within scope — `detect_format` walks the
    // PE magic-byte check (MZ + DOS-stub-relative `PE\0\0`), and
    // `load_from_bytes` walks the full PE parser through `object`.
    let _ = dac_binfmt::detect_format(data);
    let _ = dac_binfmt::load_from_bytes(data);
});

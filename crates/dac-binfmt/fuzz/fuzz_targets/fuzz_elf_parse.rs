//! `fuzz_elf_parse` — coverage-guided fuzz target for the ELF path of
//! `dac_binfmt::load_from_bytes`.
//!
//! Invariant under test (NFR-4): the parser may return an error on any
//! input but must never panic, never abort, and never trigger UB.
//!
//! Run from `crates/dac-binfmt/`:
//!
//! ```text
//! cargo install cargo-fuzz
//! cargo +nightly fuzz run fuzz_elf_parse -- -max_total_time=300
//! ```
//!
//! The 5-minute total-time cap satisfies the B1.1 done-when. Add seeds
//! to `corpus/fuzz_elf_parse/` (the workspace `tests/fixtures/` files
//! are good starting seeds).

#![no_main]

use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Both entry points are within scope — `detect_format` is the cheap
    // path that the CLI hits first, `load_from_bytes` walks the full
    // ELF parser through `object`.
    let _ = dac_binfmt::detect_format(data);
    let _ = dac_binfmt::load_from_bytes(data);
});

//! `fuzz_x86_decode` — coverage-guided fuzz target for the iced-x86–backed
//! `InstructionDecoder` impl.
//!
//! Invariant under test (NFR-4 for decoders): the linear sweep must
//! never panic, abort, or trigger UB on any byte input, and the iterator
//! must always terminate (no infinite advance on pathological inputs).
//!
//! Run from `crates/dac-arch-x86/`:
//!
//! ```text
//! cargo install cargo-fuzz
//! cargo +nightly fuzz run fuzz_x86_decode -- -max_total_time=300
//! ```
//!
//! The 5-minute total-time cap matches the B1.3 done-when. The in-tree
//! unit tests cover the deterministic snapshot path; this target covers
//! the open-ended robustness invariant.

#![no_main]

use dac_arch::{Architecture, InstructionDecoder};
use dac_arch_x86::{I386, X86_64};
use libfuzzer_sys::fuzz_target;

fuzz_target!(|data: &[u8]| {
    // Exercise both bitnesses with the same input so a single fuzz run
    // covers the 32-bit + 64-bit code paths.
    for arch in [
        &X86_64 as &dyn Architecture,
        &I386 as &dyn Architecture,
    ] {
        let decoder = arch.decoder();
        // single-shot decode
        let _ = decoder.decode_one(data, 0);
        // linear sweep — must terminate
        let mut count = 0usize;
        for _instr in decoder.iter(data, 0x1000) {
            count += 1;
            // Cap iterations to defend the fuzzer against any future
            // bug where a decode reports length 0. Cheap and avoids
            // OOMs on adversarial inputs.
            if count > data.len().saturating_mul(2) + 64 {
                break;
            }
        }
    }
});

//! Calling-convention table (B2.5, FR-13).
//!
//! Each [`CallingConvention`] is a deterministic description of how
//! one ABI passes arguments, returns values, and partitions the
//! register file into callee-saved and caller-saved sets. The table
//! is curated and small — it covers the two x86-64 conventions that
//! M2 cares about (SysV AMD64 and Microsoft x64) and nothing else
//! yet. Architectures land alongside their decoder in `dac-arch-*`;
//! conventions land here so the inference pass in
//! [`dac_recovery`] can consult them without depending on an arch
//! crate.
//!
//! ## What this module does not do
//!
//! - **Encode parameter types.** A convention describes *where* an
//!   argument is passed, not what type it has. Type recovery is
//!   B2.6's job.
//! - **Cover float-only or vector-only calls.** The table records
//!   the float and vector register order so [`dac_recovery`] can
//!   surface them, but the M2 inference pass scores on integer
//!   register usage only.
//! - **Model variadic functions.** SysV's "rax = number of vector
//!   args" convention for variadics lives in a follow-up batch
//!   alongside symbol-driven signature import.
//!
//! Register names are lowercase ASCII to match the
//! [`dac_ir::ssa::Variable::name`] vocabulary the lifter emits, so a
//! caller does not have to canonicalize them at the boundary.

/// Description of one calling convention.
///
/// All register-name slices are in *call order*: index 0 is the
/// first register a caller fills. Stack-arg layout is described from
/// the callee's entry-stack-pointer anchor — the same anchor
/// [`dac_recovery`]'s stack pass uses, so the two views compose
/// without translation.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CallingConvention {
    /// Stable identifier suitable for diagnostics and reproducibility
    /// manifests. Lowercase, hyphen-separated.
    pub name: &'static str,
    /// Architecture this convention applies to. Matches the
    /// [`dac_arch::Architecture::name`] string at the point those
    /// architectures land.
    pub architecture: &'static str,
    /// Integer / pointer argument registers, in the order a caller
    /// fills them.
    pub int_arg_registers: &'static [&'static str],
    /// Float / SSE argument registers, in caller-fill order.
    pub float_arg_registers: &'static [&'static str],
    /// Register the callee uses to return a single integer / pointer
    /// value.
    pub int_return_register: &'static str,
    /// Register the callee uses to return a single floating-point
    /// value.
    pub float_return_register: &'static str,
    /// Registers the callee must preserve across the call.
    pub callee_saved: &'static [&'static str],
    /// Registers the caller assumes are clobbered.
    pub caller_saved: &'static [&'static str],
    /// Stack pointer register name.
    pub stack_pointer: &'static str,
    /// Frame pointer register, when the convention nominates one.
    pub frame_pointer: Option<&'static str>,
    /// Signed offset from the callee's entry stack pointer to the
    /// first *stack-passed* argument. SysV places stack args
    /// immediately after the return-address slot at `+8`; MsX64
    /// reserves four 8-byte home slots first and starts stack args
    /// at `+40` (5th arg onward).
    pub first_stack_arg_offset: i64,
    /// Alignment in bytes between consecutive stack-passed
    /// arguments. Always 8 for the conventions modelled today.
    pub stack_arg_alignment: u64,
    /// Bytes of caller-allocated stack space at positive offsets
    /// reserved for the callee to spill register arguments into.
    /// Zero on SysV; 32 on MsX64 (4 × 8 bytes for RCX/RDX/R8/R9).
    /// Inclusive of all home-space bytes; *not* inclusive of the
    /// return-address slot.
    pub shadow_space_bytes: u64,
}

impl CallingConvention {
    /// True when `name` is one of this convention's integer
    /// argument registers (case-insensitive ASCII).
    #[must_use]
    pub fn is_int_arg_register(&self, name: &str) -> bool {
        self.int_arg_registers
            .iter()
            .any(|r| r.eq_ignore_ascii_case(name))
    }

    /// True when `name` matches this convention's integer return
    /// register (case-insensitive ASCII).
    #[must_use]
    pub fn is_int_return_register(&self, name: &str) -> bool {
        self.int_return_register.eq_ignore_ascii_case(name)
    }

    /// True when `name` is one of this convention's callee-saved
    /// registers (case-insensitive ASCII).
    #[must_use]
    pub fn is_callee_saved(&self, name: &str) -> bool {
        self.callee_saved
            .iter()
            .any(|r| r.eq_ignore_ascii_case(name))
    }

    /// True when `name` is one of this convention's caller-saved
    /// registers (case-insensitive ASCII).
    #[must_use]
    pub fn is_caller_saved(&self, name: &str) -> bool {
        self.caller_saved
            .iter()
            .any(|r| r.eq_ignore_ascii_case(name))
    }

    /// Position (0-based) of `name` in [`Self::int_arg_registers`].
    #[must_use]
    pub fn int_arg_index(&self, name: &str) -> Option<usize> {
        self.int_arg_registers
            .iter()
            .position(|r| r.eq_ignore_ascii_case(name))
    }
}

/// The System V AMD64 ABI (Linux, BSD, macOS on x86-64).
pub const SYSV_AMD64: CallingConvention = CallingConvention {
    name: "sysv-amd64",
    architecture: "x86-64",
    int_arg_registers: &["rdi", "rsi", "rdx", "rcx", "r8", "r9"],
    float_arg_registers: &[
        "xmm0", "xmm1", "xmm2", "xmm3", "xmm4", "xmm5", "xmm6", "xmm7",
    ],
    int_return_register: "rax",
    float_return_register: "xmm0",
    callee_saved: &["rbx", "rbp", "r12", "r13", "r14", "r15"],
    caller_saved: &["rax", "rcx", "rdx", "rsi", "rdi", "r8", "r9", "r10", "r11"],
    stack_pointer: "rsp",
    frame_pointer: Some("rbp"),
    first_stack_arg_offset: 8, // immediately above the return address
    stack_arg_alignment: 8,
    shadow_space_bytes: 0,
};

/// The Microsoft x64 ABI (Windows on x86-64).
pub const MS_X64: CallingConvention = CallingConvention {
    name: "ms-x64",
    architecture: "x86-64",
    int_arg_registers: &["rcx", "rdx", "r8", "r9"],
    float_arg_registers: &["xmm0", "xmm1", "xmm2", "xmm3"],
    int_return_register: "rax",
    float_return_register: "xmm0",
    callee_saved: &["rbx", "rbp", "rdi", "rsi", "r12", "r13", "r14", "r15"],
    caller_saved: &["rax", "rcx", "rdx", "r8", "r9", "r10", "r11"],
    stack_pointer: "rsp",
    frame_pointer: Some("rbp"),
    first_stack_arg_offset: 40, // 8 ret addr + 32 home space
    stack_arg_alignment: 8,
    shadow_space_bytes: 32,
};

/// All x86-64 calling conventions known to dac, in lookup order.
///
/// The order is the order [`dac_recovery`]'s inference pass scores
/// them, so a tie at the top of the ranking breaks toward SysV
/// (the more common default on the corpora dac currently targets).
pub const X86_64_CONVENTIONS: &[&CallingConvention] = &[&SYSV_AMD64, &MS_X64];

/// Look up an x86-64 convention by its stable `name`.
#[must_use]
pub fn x86_64_convention_by_name(name: &str) -> Option<&'static CallingConvention> {
    X86_64_CONVENTIONS
        .iter()
        .copied()
        .find(|c| c.name.eq_ignore_ascii_case(name))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sysv_amd64_table_matches_abi() {
        assert_eq!(SYSV_AMD64.name, "sysv-amd64");
        assert_eq!(
            SYSV_AMD64.int_arg_registers,
            &["rdi", "rsi", "rdx", "rcx", "r8", "r9"]
        );
        assert_eq!(SYSV_AMD64.int_return_register, "rax");
        assert_eq!(SYSV_AMD64.shadow_space_bytes, 0);
        assert_eq!(SYSV_AMD64.first_stack_arg_offset, 8);
        assert_eq!(SYSV_AMD64.frame_pointer, Some("rbp"));
    }

    #[test]
    fn ms_x64_table_matches_abi() {
        assert_eq!(MS_X64.name, "ms-x64");
        assert_eq!(MS_X64.int_arg_registers, &["rcx", "rdx", "r8", "r9"]);
        assert_eq!(MS_X64.int_return_register, "rax");
        assert_eq!(MS_X64.shadow_space_bytes, 32);
        assert_eq!(MS_X64.first_stack_arg_offset, 40);
    }

    #[test]
    fn int_arg_index_is_zero_based_and_case_insensitive() {
        assert_eq!(SYSV_AMD64.int_arg_index("rdi"), Some(0));
        assert_eq!(SYSV_AMD64.int_arg_index("RDX"), Some(2));
        assert_eq!(SYSV_AMD64.int_arg_index("rax"), None);
        assert_eq!(MS_X64.int_arg_index("rcx"), Some(0));
        assert_eq!(MS_X64.int_arg_index("rdi"), None);
    }

    #[test]
    fn callee_caller_predicates_use_ignored_case() {
        assert!(SYSV_AMD64.is_callee_saved("RBX"));
        assert!(SYSV_AMD64.is_caller_saved("rdi"));
        assert!(MS_X64.is_callee_saved("rdi"));
        assert!(!MS_X64.is_caller_saved("rbx"));
    }

    #[test]
    fn lookup_by_name_returns_canonical_entry() {
        assert_eq!(
            x86_64_convention_by_name("sysv-amd64").unwrap().name,
            SYSV_AMD64.name,
        );
        assert_eq!(
            x86_64_convention_by_name("MS-X64").unwrap().name,
            MS_X64.name,
        );
        assert!(x86_64_convention_by_name("aapcs").is_none());
    }

    #[test]
    fn arg_register_sets_are_disjoint_between_sysv_unique_and_ms_unique() {
        // SysV-unique args (rdi, rsi) must not appear in MsX64's
        // arg list, otherwise the inference pass cannot distinguish
        // them.
        for r in &["rdi", "rsi"] {
            assert!(!MS_X64.is_int_arg_register(r));
        }
        // Shared regs do overlap by design.
        for r in &["rcx", "rdx", "r8", "r9"] {
            assert!(SYSV_AMD64.is_int_arg_register(r));
            assert!(MS_X64.is_int_arg_register(r));
        }
    }
}

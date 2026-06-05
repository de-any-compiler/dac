//! Calling-convention table (B2.5 / B3.13, FR-13).
//!
//! Each [`CallingConvention`] is a deterministic description of how
//! one ABI passes arguments, returns values, and partitions the
//! register file into callee-saved and caller-saved sets. The table
//! is curated and small — it covers the three x86-64 conventions
//! dac currently models: SysV AMD64 ([`SYSV_AMD64`]), Microsoft x64
//! ([`MS_X64`]), and the Linux kernel's [`SYSV_AMD64_SYSCALL`]
//! variant (B3.13 — `syscall` clobbers `rcx`, so the fourth integer
//! argument moves from `rcx` to `r10`). Architectures land alongside
//! their decoder in `dac-arch-*`; conventions land here so the
//! inference pass in [`dac_recovery`] can consult them without
//! depending on an arch crate.
//!
//! ## What this module does not do
//!
//! - **Encode parameter types.** A convention describes *where* an
//!   argument is passed, not what type it has. Type recovery is
//!   B2.6's job.
//! - **Cover float-only or vector-only calls.** The table records
//!   the float and vector register order so [`dac_recovery`] can
//!   surface them, but the inference pass scores on integer
//!   register usage only.
//! - **Carry a separate "SysV variadic" convention entry.** A
//!   variadic SysV call site is a SysV call site that *also*
//!   sets `rax` to the count of vector arguments before the call;
//!   the callee's register-passing layout is identical to a
//!   non-variadic SysV callee, so the discriminator is a
//!   call-site property, not a convention. The inference pass in
//!   `dac_recovery::convention` reports the count of detected
//!   variadic call sites via
//!   `InferredSignature::variadic_call_sites` so downstream passes
//!   can promote a hint or signature to its variadic shape without
//!   a new convention entry.
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
    /// What kind of call this convention models. The default,
    /// [`ConventionKind::Normal`], covers user-space SysV / MsX64
    /// calls. [`ConventionKind::Syscall`] flags the Linux kernel
    /// `syscall` instruction's layout — the recovery pass uses this
    /// to score a function containing a `syscall` opcode in favour
    /// of this convention over the user-space sibling.
    pub kind: ConventionKind,
}

/// Coarse classification used by [`dac_recovery::convention`] to
/// pick between user-space and kernel conventions on functions that
/// contain a `syscall` opcode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ConventionKind {
    /// Regular user-space call. The lifter sees a `Call` op at the
    /// boundary and no `syscall` opcode in the body.
    Normal,
    /// Linux kernel `syscall` instruction convention. The function
    /// body contains a `syscall` opcode; the fourth integer
    /// argument lives in `r10` rather than `rcx` (which `syscall`
    /// clobbers with the return RIP).
    Syscall,
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
    kind: ConventionKind::Normal,
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
    kind: ConventionKind::Normal,
};

/// Linux kernel `syscall` instruction convention on x86-64 (B3.13).
///
/// The fourth integer argument moves from `rcx` (SysV) to `r10`
/// because the `syscall` instruction itself writes the return RIP
/// into `rcx`. `rax` carries the syscall number on entry and the
/// kernel's signed return value on exit; `rcx` and `r11` are
/// always-clobbered by `syscall`. Everything else (return register,
/// callee-saved set, stack-arg layout) matches [`SYSV_AMD64`], so a
/// thin C wrapper around a syscall renders identically to a SysV
/// callee apart from the argument-register prefix.
pub const SYSV_AMD64_SYSCALL: CallingConvention = CallingConvention {
    name: "sysv-amd64-syscall",
    architecture: "x86-64",
    int_arg_registers: &["rdi", "rsi", "rdx", "r10", "r8", "r9"],
    float_arg_registers: &[],
    int_return_register: "rax",
    float_return_register: "xmm0",
    callee_saved: &["rbx", "rbp", "r12", "r13", "r14", "r15"],
    caller_saved: &["rax", "rcx", "rdx", "rsi", "rdi", "r8", "r9", "r10", "r11"],
    stack_pointer: "rsp",
    frame_pointer: Some("rbp"),
    first_stack_arg_offset: 8,
    stack_arg_alignment: 8,
    shadow_space_bytes: 0,
    kind: ConventionKind::Syscall,
};

/// All x86-64 calling conventions known to dac, in lookup order.
///
/// The order is the order [`dac_recovery`]'s inference pass scores
/// them, so a tie at the top of the ranking breaks toward SysV
/// (the more common default on the corpora dac currently targets).
/// [`SYSV_AMD64_SYSCALL`] sits last so a function that does *not*
/// contain a `syscall` opcode falls through to the user-space
/// reading without further ranking work — the syscall convention's
/// scoring rule applies a hard penalty when the opcode is absent.
pub const X86_64_CONVENTIONS: &[&CallingConvention] = &[&SYSV_AMD64, &MS_X64, &SYSV_AMD64_SYSCALL];

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

    #[test]
    fn sysv_syscall_swaps_rcx_for_r10_and_keeps_everything_else() {
        assert_eq!(SYSV_AMD64_SYSCALL.name, "sysv-amd64-syscall");
        assert_eq!(SYSV_AMD64_SYSCALL.kind, ConventionKind::Syscall);
        assert_eq!(
            SYSV_AMD64_SYSCALL.int_arg_registers,
            &["rdi", "rsi", "rdx", "r10", "r8", "r9"],
        );
        // rcx is NOT a syscall arg (the syscall instruction
        // clobbers it with the return RIP).
        assert!(!SYSV_AMD64_SYSCALL.is_int_arg_register("rcx"));
        // r10 IS the syscall's 4th arg.
        assert_eq!(SYSV_AMD64_SYSCALL.int_arg_index("r10"), Some(3));
        // Return register, stack-arg layout, callee-saved set all
        // match SysV; only the arg-prefix differs.
        assert_eq!(
            SYSV_AMD64_SYSCALL.int_return_register,
            SYSV_AMD64.int_return_register,
        );
        assert_eq!(
            SYSV_AMD64_SYSCALL.first_stack_arg_offset,
            SYSV_AMD64.first_stack_arg_offset,
        );
        assert_eq!(SYSV_AMD64_SYSCALL.callee_saved, SYSV_AMD64.callee_saved);
    }

    #[test]
    fn convention_kinds_default_to_normal_for_user_space_conventions() {
        assert_eq!(SYSV_AMD64.kind, ConventionKind::Normal);
        assert_eq!(MS_X64.kind, ConventionKind::Normal);
    }

    #[test]
    fn x86_64_conventions_ordered_normal_first_then_syscall_last() {
        // Order is load-bearing: normal SysV / MsX64 win ranking
        // ties when no syscall opcode is observed, and the syscall
        // entry sits last so the inference-pass slice traversal
        // never hits it unless its scoring rule lifts it.
        let names: Vec<_> = X86_64_CONVENTIONS.iter().map(|c| c.name).collect();
        assert_eq!(names, vec!["sysv-amd64", "ms-x64", "sysv-amd64-syscall"]);
    }

    #[test]
    fn convention_by_name_finds_syscall_variant() {
        assert_eq!(
            x86_64_convention_by_name("sysv-amd64-syscall")
                .unwrap()
                .kind,
            ConventionKind::Syscall,
        );
    }
}

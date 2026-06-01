//! Register files for i386 and x86-64.
//!
//! Both files cover the architectural general-purpose registers and
//! their narrower aliases (`eax` of `rax`, `ax` of `rax`, …) plus the
//! instruction pointer and flags register. Vector / FP registers come in
//! with the lifter once it actually models them.

use std::sync::OnceLock;

use dac_arch::{Register, RegisterClass, RegisterFile, RegisterId};

static X86_64_RF: OnceLock<RegisterFile> = OnceLock::new();
static I386_RF: OnceLock<RegisterFile> = OnceLock::new();

pub(crate) fn x86_64_register_file() -> &'static RegisterFile {
    X86_64_RF.get_or_init(build_x86_64)
}

pub(crate) fn i386_register_file() -> &'static RegisterFile {
    I386_RF.get_or_init(build_i386)
}

fn gp(idx: u32, name: &'static str, size_bits: u16, parent: Option<u32>) -> Register {
    Register {
        id: RegisterId(idx),
        name,
        class: RegisterClass::GeneralPurpose,
        size_bits,
        parent: parent.map(RegisterId),
    }
}

fn build_x86_64() -> RegisterFile {
    // 16 64-bit GPRs in System V AMD64 ABI canonical order, then their
    // 32 / 16 / 8-bit aliases. Sub-register parents point at the
    // 64-bit base.
    let gprs_64 = [
        "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp", "r8", "r9", "r10", "r11", "r12",
        "r13", "r14", "r15",
    ];
    let gprs_32 = [
        "eax", "ebx", "ecx", "edx", "esi", "edi", "ebp", "esp", "r8d", "r9d", "r10d", "r11d",
        "r12d", "r13d", "r14d", "r15d",
    ];
    let gprs_16 = [
        "ax", "bx", "cx", "dx", "si", "di", "bp", "sp", "r8w", "r9w", "r10w", "r11w", "r12w",
        "r13w", "r14w", "r15w",
    ];
    let gprs_8l = [
        "al", "bl", "cl", "dl", "sil", "dil", "bpl", "spl", "r8b", "r9b", "r10b", "r11b", "r12b",
        "r13b", "r14b", "r15b",
    ];

    let n = gprs_64.len() as u32;
    let mut regs: Vec<Register> = Vec::with_capacity((n as usize) * 4 + 2);

    for (i, name) in gprs_64.iter().enumerate() {
        regs.push(gp(i as u32, name, 64, None));
    }
    for (i, name) in gprs_32.iter().enumerate() {
        regs.push(gp(n + i as u32, name, 32, Some(i as u32)));
    }
    for (i, name) in gprs_16.iter().enumerate() {
        regs.push(gp(2 * n + i as u32, name, 16, Some(i as u32)));
    }
    for (i, name) in gprs_8l.iter().enumerate() {
        regs.push(gp(3 * n + i as u32, name, 8, Some(i as u32)));
    }

    let base = regs.len() as u32;
    regs.push(Register {
        id: RegisterId(base),
        name: "rip",
        class: RegisterClass::Special,
        size_bits: 64,
        parent: None,
    });
    regs.push(Register {
        id: RegisterId(base + 1),
        name: "rflags",
        class: RegisterClass::Flags,
        size_bits: 64,
        parent: None,
    });

    RegisterFile::new(regs)
}

fn build_i386() -> RegisterFile {
    // 8 32-bit GPRs plus their 16-bit, 8-bit-low, and 8-bit-high aliases
    // for the four registers that have them.
    let gprs_32 = ["eax", "ebx", "ecx", "edx", "esi", "edi", "ebp", "esp"];
    let gprs_16 = ["ax", "bx", "cx", "dx", "si", "di", "bp", "sp"];
    let gprs_8l = ["al", "bl", "cl", "dl"];
    let gprs_8h = ["ah", "bh", "ch", "dh"];

    let n = gprs_32.len() as u32;
    let mut regs: Vec<Register> =
        Vec::with_capacity((n as usize) * 2 + gprs_8l.len() + gprs_8h.len() + 2);

    for (i, name) in gprs_32.iter().enumerate() {
        regs.push(gp(i as u32, name, 32, None));
    }
    for (i, name) in gprs_16.iter().enumerate() {
        regs.push(gp(n + i as u32, name, 16, Some(i as u32)));
    }
    let mut next = 2 * n;
    for (i, name) in gprs_8l.iter().enumerate() {
        regs.push(gp(next + i as u32, name, 8, Some(i as u32)));
    }
    next += gprs_8l.len() as u32;
    for (i, name) in gprs_8h.iter().enumerate() {
        regs.push(gp(next + i as u32, name, 8, Some(i as u32)));
    }

    let base = regs.len() as u32;
    regs.push(Register {
        id: RegisterId(base),
        name: "eip",
        class: RegisterClass::Special,
        size_bits: 32,
        parent: None,
    });
    regs.push(Register {
        id: RegisterId(base + 1),
        name: "eflags",
        class: RegisterClass::Flags,
        size_bits: 32,
        parent: None,
    });

    RegisterFile::new(regs)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn x86_64_canonical_names_resolve() {
        let rf = x86_64_register_file();
        for n in [
            "rax", "rbx", "rcx", "rdx", "rsi", "rdi", "rbp", "rsp", "r8", "r15", "eax", "r15d",
            "ax", "r15w", "al", "r15b", "rip", "rflags",
        ] {
            assert!(rf.by_name(n).is_some(), "missing {n}");
        }
    }

    #[test]
    fn x86_64_aliases_point_at_parent() {
        let rf = x86_64_register_file();
        let rax = rf.by_name("rax").unwrap();
        let eax = rf.by_name("eax").unwrap();
        assert_eq!(eax.parent, Some(rax.id));
        assert_eq!(eax.size_bits, 32);
    }

    #[test]
    fn i386_does_not_advertise_64_bit_register_set() {
        let rf = i386_register_file();
        for n in ["rax", "r8", "rip", "rflags"] {
            assert!(rf.by_name(n).is_none(), "i386 must not expose {n}");
        }
        assert!(rf.by_name("eip").is_some());
    }

    #[test]
    fn lookup_is_case_insensitive() {
        let rf = x86_64_register_file();
        assert!(rf.by_name("RAX").is_some());
        assert!(rf.by_name("Rip").is_some());
    }

    #[test]
    fn ids_round_trip_through_register_file() {
        let rf = x86_64_register_file();
        let rax = rf.by_name("rax").unwrap();
        let by_id = rf.register(rax.id).unwrap();
        assert_eq!(by_id.name, "rax");
    }
}

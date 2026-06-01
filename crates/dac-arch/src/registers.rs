//! Architecture-neutral register metadata.
//!
//! [`RegisterFile`] is a flat catalogue of every register an architecture
//! exposes. Concrete backends populate it once (typically inside an
//! `OnceLock` initialiser) and hand the same reference back from
//! `Architecture::register_file`. The layout is intentionally simple:
//! the lifter (B1.4) and downstream passes look registers up by id; the
//! file does not embed encoding state, calling-convention roles, or
//! ABI hints, all of which live in their own crates.

/// Opaque numeric identifier for a register within a [`RegisterFile`].
///
/// The numeric value is stable for the lifetime of the register file but
/// is otherwise an opaque tag — passes look up [`Register`] metadata via
/// [`RegisterFile::register`] rather than reasoning about the integer.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct RegisterId(pub u32);

/// Coarse role classification used by structural analyses (liveness,
/// calling-convention inference, type recovery). Finer-grained roles
/// (e.g. "argument register #2 in SysV") live in `dac-knowledge`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegisterClass {
    GeneralPurpose,
    Vector,
    FloatingPoint,
    Flags,
    Segment,
    /// Instruction pointer, special-purpose status registers, MSRs, …
    /// — anything the lifter has to model but which is neither GP nor
    /// flag-shaped.
    Special,
}

#[derive(Debug, Clone)]
pub struct Register {
    pub id: RegisterId,
    pub name: &'static str,
    pub class: RegisterClass,
    pub size_bits: u16,
    /// `Some(parent)` if this is an alias / sub-register of another
    /// register (e.g. `eax` is a sub-register of `rax`, sharing the low
    /// 32 bits). `None` for full-width / standalone registers.
    pub parent: Option<RegisterId>,
}

/// Flat catalogue of the registers an architecture exposes. Built once
/// per architecture and returned by reference from
/// `Architecture::register_file`.
#[derive(Debug, Default)]
pub struct RegisterFile {
    registers: Vec<Register>,
}

impl RegisterFile {
    /// Construct a register file from a pre-built register list. Caller
    /// is responsible for assigning sequential ids matching the slice
    /// position; [`RegisterFile::register`] indexes by that position.
    #[must_use]
    pub fn new(registers: Vec<Register>) -> Self {
        Self { registers }
    }

    /// Look up a register by id. Returns `None` if the id is out of
    /// range for this file (foreign ids never panic).
    #[must_use]
    pub fn register(&self, id: RegisterId) -> Option<&Register> {
        self.registers.get(id.0 as usize)
    }

    /// All registers in declaration order.
    #[must_use]
    pub fn registers(&self) -> &[Register] {
        &self.registers
    }

    /// Case-insensitive lookup by canonical name. Useful for tests and
    /// for resolving user hints; passes should prefer ids.
    #[must_use]
    pub fn by_name(&self, name: &str) -> Option<&Register> {
        self.registers
            .iter()
            .find(|r| r.name.eq_ignore_ascii_case(name))
    }

    #[must_use]
    pub fn len(&self) -> usize {
        self.registers.len()
    }

    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.registers.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rf() -> RegisterFile {
        RegisterFile::new(vec![
            Register {
                id: RegisterId(0),
                name: "r0",
                class: RegisterClass::GeneralPurpose,
                size_bits: 64,
                parent: None,
            },
            Register {
                id: RegisterId(1),
                name: "r0w",
                class: RegisterClass::GeneralPurpose,
                size_bits: 32,
                parent: Some(RegisterId(0)),
            },
        ])
    }

    #[test]
    fn lookup_by_id_and_name() {
        let f = rf();
        assert_eq!(f.register(RegisterId(0)).unwrap().name, "r0");
        assert_eq!(f.by_name("R0W").unwrap().size_bits, 32);
    }

    #[test]
    fn out_of_range_id_returns_none() {
        let f = rf();
        assert!(f.register(RegisterId(99)).is_none());
        assert!(f.by_name("doesnotexist").is_none());
    }

    #[test]
    fn parent_alias_is_tracked() {
        let f = rf();
        let alias = f.by_name("r0w").unwrap();
        assert_eq!(alias.parent, Some(RegisterId(0)));
    }

    #[test]
    fn len_and_is_empty() {
        assert!(RegisterFile::default().is_empty());
        assert_eq!(rf().len(), 2);
    }
}

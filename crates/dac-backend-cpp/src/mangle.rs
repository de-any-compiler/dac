//! Tiny Itanium C++ mangled-name reader (FR-21, spec §6).
//!
//! The C++ backend keys class recovery off symbol names. This module
//! recognises the minimum subset of [Itanium ABI mangling][itanium] that
//! a small class hierarchy emits: nested-name member functions
//! (`_ZN<class>...<member>E...`), const-qualified members
//! (`_ZNK...`), ctor/dtor variants (`C[123]E.../D[012]E...`), free
//! functions (`_Z<name>...`), and the special data symbols every
//! polymorphic class produces — `_ZTV` (vtable), `_ZTI` (typeinfo),
//! `_ZTS` (typeinfo string), `_ZTT` (VTT).
//!
//! [itanium]: https://itanium-cxx-abi.github.io/cxx-abi/abi.html#mangling
//!
//! ## What's intentionally not parsed
//!
//! - **Argument types.** Everything after the closing `E` is consumed
//!   blindly. dac does not need the parameter list to group symbols by
//!   class, and a real demangler that handles substitution
//!   (`S_`/`S0_`/…), templates (`I…E`), and operator names would dwarf
//!   the rest of the backend in code volume. The [`MemberSignature`]
//!   carries the raw mangled suffix so callers can pass it through to a
//!   future fuller demangler.
//! - **Templates, operator overloads, lambdas.** A `_ZN…<unknown>E…`
//!   that the reader cannot classify returns `None`; class recovery
//!   degrades by leaving the symbol on the free-function pile rather
//!   than guessing.
//! - **Substitutions.** Substitution markers in the *suffix* are fine
//!   (we ignore them). Substitutions in the *nested name* would require
//!   a substitution table; we have not seen one in any of the small
//!   class hierarchies B3.5 targets, so we error out (return `None`).
//!
//! ## Determinism
//!
//! Pure function from `&str` to `Option<ItaniumSymbol>`. No allocations
//! beyond the owned strings in the returned variant.

/// A parsed Itanium-mangled symbol — only the variants dac-backend-cpp
/// needs at B3.5.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ItaniumSymbol {
    /// Member function (method, constructor, or destructor).
    Member {
        /// Nested-name chain from outermost to innermost. The class the
        /// member belongs to is `chain.last()`. For an unnested member
        /// the chain has length 1.
        chain: Vec<String>,
        /// Const-qualified member function (`_ZNK…`).
        is_const: bool,
        /// What kind of member: a named method, a ctor, or a dtor.
        kind: MemberKind,
        /// Untouched mangled suffix (everything after the closing `E`
        /// for methods, or after the variant digit for ctors / dtors).
        /// Carried so a future fuller demangler can render the signature.
        suffix: String,
    },
    /// Free function (top-level, not nested in a class). The chain is
    /// `[name]`; the suffix is the post-name part of the mangling.
    Free { name: String, suffix: String },
    /// `_ZTV<name>` — vtable for a class.
    Vtable { class_chain: Vec<String> },
    /// `_ZTI<name>` — typeinfo object for a class.
    TypeInfo { class_chain: Vec<String> },
    /// `_ZTS<name>` — typeinfo name string for a class.
    TypeInfoString { class_chain: Vec<String> },
    /// `_ZTT<name>` — virtual-table table (VTT) for a class.
    Vtt { class_chain: Vec<String> },
}

/// What kind of class member a [`ItaniumSymbol::Member`] entry is.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemberKind {
    /// Named method like `_ZN3Dog5speakEv` → `speak`.
    Method { name: String },
    /// Ctor variant — `C1` (complete object), `C2` (base object), `C3`
    /// (allocating). The digit is preserved so emit/lower can keep
    /// every variant as a distinct member function.
    Constructor { variant: u8 },
    /// Dtor variant — `D0` (deleting), `D1` (complete), `D2` (base).
    Destructor { variant: u8 },
}

impl MemberKind {
    /// Human-friendly label used by emit + annotations. Methods render
    /// as their name; ctors / dtors render as `<class>` / `~<class>`
    /// with a `_v<digit>` suffix for the variant.
    #[must_use]
    pub fn label(&self, class_name: &str) -> String {
        match self {
            Self::Method { name } => name.clone(),
            Self::Constructor { variant } => format!("{class_name}_ctor_v{variant}"),
            Self::Destructor { variant } => format!("{class_name}_dtor_v{variant}"),
        }
    }
}

/// Parse a single mangled symbol. Returns `None` for any name the
/// reader cannot recognise; callers must degrade gracefully (typically:
/// leave on the free-function pile, or skip entirely).
#[must_use]
pub fn parse(symbol: &str) -> Option<ItaniumSymbol> {
    let body = symbol.strip_prefix("_Z")?;

    if let Some(rest) = body.strip_prefix("TV") {
        return parse_class_only(rest).map(|chain| ItaniumSymbol::Vtable { class_chain: chain });
    }
    if let Some(rest) = body.strip_prefix("TI") {
        return parse_class_only(rest).map(|chain| ItaniumSymbol::TypeInfo { class_chain: chain });
    }
    if let Some(rest) = body.strip_prefix("TS") {
        return parse_class_only(rest)
            .map(|chain| ItaniumSymbol::TypeInfoString { class_chain: chain });
    }
    if let Some(rest) = body.strip_prefix("TT") {
        return parse_class_only(rest).map(|chain| ItaniumSymbol::Vtt { class_chain: chain });
    }

    if let Some(rest) = body.strip_prefix('N') {
        let (rest, is_const) = match rest.strip_prefix('K') {
            Some(after_k) => (after_k, true),
            None => (rest, false),
        };
        let (chain, kind, suffix) = parse_nested(rest)?;
        if chain.is_empty() {
            return None;
        }
        return Some(ItaniumSymbol::Member {
            chain,
            is_const,
            kind,
            suffix,
        });
    }

    // Free function: `_Z<len><name><suffix>`.
    let (name, suffix) = parse_length_name(body)?;
    Some(ItaniumSymbol::Free {
        name,
        suffix: suffix.to_string(),
    })
}

/// For `_ZTV*` / `_ZTI*` / `_ZTS*` / `_ZTT*`: parse the class chain
/// that follows. The chain may be a single `<len><name>` segment or a
/// nested `N…E` sequence.
fn parse_class_only(rest: &str) -> Option<Vec<String>> {
    if let Some(rest) = rest.strip_prefix('N') {
        let mut chain = Vec::new();
        let mut cursor = rest;
        loop {
            let (name, next) = parse_length_name(cursor)?;
            chain.push(name);
            cursor = next;
            if let Some(_after_e) = cursor.strip_prefix('E') {
                // We accept trailing material; vtable / typeinfo names
                // do not normally carry it, but a tolerant reader is
                // better than a brittle one.
                return Some(chain);
            }
        }
    }
    let (name, _rest) = parse_length_name(rest)?;
    Some(vec![name])
}

/// Parse a nested-name body — what follows the `N`(`K`?) prefix and
/// runs up to (and including) the closing `E`.
///
/// At each step the reader tries the ctor / dtor short-form before
/// reading another length-prefixed segment. The ctor / dtor form must
/// follow at least one class segment, so the very first segment is
/// always read as a name.
fn parse_nested(body: &str) -> Option<(Vec<String>, MemberKind, String)> {
    let mut chain: Vec<String> = Vec::new();
    let mut rest = body;
    loop {
        // Ctor / dtor short-forms only make sense once at least one
        // class name is on the chain — `_ZNC1E…` is not a real symbol.
        if !chain.is_empty() {
            if let Some(after) = rest.strip_prefix('C') {
                let variant = after.chars().next()?;
                if matches!(variant, '1' | '2' | '3') {
                    let after_digit = &after[1..];
                    let suffix = after_digit.strip_prefix('E').unwrap_or(after_digit);
                    return Some((
                        chain,
                        MemberKind::Constructor {
                            variant: variant as u8 - b'0',
                        },
                        suffix.to_string(),
                    ));
                }
            }
            if let Some(after) = rest.strip_prefix('D') {
                let variant = after.chars().next()?;
                if matches!(variant, '0' | '1' | '2') {
                    let after_digit = &after[1..];
                    let suffix = after_digit.strip_prefix('E').unwrap_or(after_digit);
                    return Some((
                        chain,
                        MemberKind::Destructor {
                            variant: variant as u8 - b'0',
                        },
                        suffix.to_string(),
                    ));
                }
            }
        }
        // Otherwise read another `<len><name>` segment.
        let (name, next) = parse_length_name(rest)?;
        // If `E` immediately follows, the segment we just read is the
        // method name and the prior segments are the class chain.
        if let Some(after_e) = next.strip_prefix('E') {
            return Some((chain, MemberKind::Method { name }, after_e.to_string()));
        }
        chain.push(name);
        rest = next;
    }
}

/// Read a `<len><name>` source-name segment. Returns `(name, rest)`
/// where `name` is the next `len` bytes of `body` and `rest` is what
/// follows.
fn parse_length_name(body: &str) -> Option<(String, &str)> {
    let mut digits_end = 0;
    for (i, c) in body.char_indices() {
        if !c.is_ascii_digit() {
            digits_end = i;
            break;
        }
    }
    if digits_end == 0 {
        return None;
    }
    let len: usize = body[..digits_end].parse().ok()?;
    if len == 0 || body.len() < digits_end + len {
        return None;
    }
    let name = &body[digits_end..digits_end + len];
    let rest = &body[digits_end + len..];
    // The name must be ASCII identifier-like. We do not enforce the
    // full identifier grammar — Itanium allows almost anything in a
    // source-name — but reject empty / non-printable bodies so a
    // garbled input fails closed instead of producing a bogus class.
    if name.is_empty() || name.chars().any(|c| c.is_control()) {
        return None;
    }
    Some((name.to_string(), rest))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nonprefixed_returns_none() {
        assert!(parse("main").is_none());
        assert!(parse("_Y3Dog").is_none());
        assert!(parse("").is_none());
    }

    #[test]
    fn vtable_typeinfo_typeinfostring_round_trip() {
        let v = parse("_ZTV3Dog").unwrap();
        assert_eq!(
            v,
            ItaniumSymbol::Vtable {
                class_chain: vec!["Dog".into()]
            }
        );
        let ti = parse("_ZTI6Animal").unwrap();
        assert_eq!(
            ti,
            ItaniumSymbol::TypeInfo {
                class_chain: vec!["Animal".into()]
            }
        );
        let ts = parse("_ZTS3Cat").unwrap();
        assert_eq!(
            ts,
            ItaniumSymbol::TypeInfoString {
                class_chain: vec!["Cat".into()]
            }
        );
    }

    #[test]
    fn nested_vtable_resolves_chain() {
        let v = parse("_ZTVN3Foo3BarE").unwrap();
        assert_eq!(
            v,
            ItaniumSymbol::Vtable {
                class_chain: vec!["Foo".into(), "Bar".into()]
            }
        );
    }

    #[test]
    fn ctor_variant_classified() {
        let s = parse("_ZN3DogC1Ev").unwrap();
        let ItaniumSymbol::Member {
            chain,
            is_const,
            kind,
            suffix,
        } = s
        else {
            panic!("expected member");
        };
        assert_eq!(chain, vec!["Dog".to_string()]);
        assert!(!is_const);
        assert_eq!(kind, MemberKind::Constructor { variant: 1 });
        assert_eq!(suffix, "v");
    }

    #[test]
    fn dtor_variants_all_three() {
        for (mangled, variant) in [("_ZN3DogD0Ev", 0u8), ("_ZN3DogD1Ev", 1), ("_ZN3DogD2Ev", 2)] {
            let s = parse(mangled).unwrap();
            let ItaniumSymbol::Member { kind, .. } = s else {
                panic!("expected member for {mangled}");
            };
            assert_eq!(kind, MemberKind::Destructor { variant });
        }
    }

    #[test]
    fn const_method_sets_is_const() {
        let s = parse("_ZNK3Dog5speakEv").unwrap();
        let ItaniumSymbol::Member {
            chain,
            is_const,
            kind,
            ..
        } = s
        else {
            panic!("expected member");
        };
        assert_eq!(chain, vec!["Dog".to_string()]);
        assert!(is_const);
        assert_eq!(
            kind,
            MemberKind::Method {
                name: "speak".into()
            }
        );
    }

    #[test]
    fn nested_method_keeps_full_chain() {
        let s = parse("_ZN3Foo3Bar4funcEv").unwrap();
        let ItaniumSymbol::Member { chain, kind, .. } = s else {
            panic!("expected member");
        };
        assert_eq!(chain, vec!["Foo".to_string(), "Bar".to_string()]);
        assert_eq!(
            kind,
            MemberKind::Method {
                name: "func".into()
            }
        );
    }

    #[test]
    fn free_function_keeps_name_and_suffix() {
        let s = parse("_Z6chorusPK6AnimalS1_").unwrap();
        let ItaniumSymbol::Free { name, suffix } = s else {
            panic!("expected free");
        };
        assert_eq!(name, "chorus");
        assert_eq!(suffix, "PK6AnimalS1_");
    }

    #[test]
    fn length_name_rejects_truncated_input() {
        // `9foo` claims a 9-byte name in a 3-byte buffer.
        assert!(parse_length_name("9foo").is_none());
    }

    #[test]
    fn nested_returns_none_without_class_segment() {
        // `_ZNC1Ev` would claim a ctor with no class — we refuse.
        assert!(parse("_ZNC1Ev").is_none());
    }

    #[test]
    fn label_renders_ctor_dtor_method() {
        assert_eq!(
            MemberKind::Method {
                name: "speak".into()
            }
            .label("Dog"),
            "speak"
        );
        assert_eq!(
            MemberKind::Constructor { variant: 1 }.label("Dog"),
            "Dog_ctor_v1"
        );
        assert_eq!(
            MemberKind::Destructor { variant: 0 }.label("Dog"),
            "Dog_dtor_v0"
        );
    }
}

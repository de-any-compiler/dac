//! Symbol-driven class recovery (B3.5, FR-21).
//!
//! Walks a [`BinaryModel`]'s symbol table together with the recovered
//! [`FunctionSet`] and groups Itanium-mangled symbols by class. The
//! output [`RecoveredClasses`] is everything [`crate::lower`] needs to
//! emit `class <Name> { … };` shapes — a flat list of classes (each
//! carrying its members), plus a residual list of free functions whose
//! mangled names did not belong to a class.
//!
//! ## Why symbol-driven (do less)
//!
//! PLAN.md's B3.5 calls for "class recovery from vtables". Itanium's
//! vtables are named symbols (`_ZTV<class>`), and the member-function
//! mangling already encodes the class chain — so a symbol-table walk
//! reconstructs the hierarchy directly without reading `.rodata` bytes.
//! Byte-level vtable scanning lands when the binary is stripped of
//! these symbols and the only remaining signal is a relocation pattern
//! in `.data.rel.ro`; B3.5 stays scoped to symbol-driven recovery so
//! the unhappy-path (stripped C++ binary) is an explicit deferral, not
//! a quiet failure.
//!
//! ## Evidence wiring (I-2)
//!
//! Each recovered class lands in the [`EvidenceGraph`] as a
//! [`EvidenceNode::IrNode { layer: Source, id: <class_index> }`].
//! Every member function is linked to that node via a
//! [`EdgeKind::Supports`] edge from the function's own evidence handle
//! (the `Cfg`-layer node minted by [`dac_recovery::discover_functions`])
//! to the class node. When the class also has a `_ZTV*` symbol the
//! recovery records a [`EvidenceNode::KnowledgeFact`] node holding the
//! `class_name_hash` and links it as supporting evidence so a reader
//! can ask "why do we think this class is polymorphic?" and follow the
//! edge back to the vtable symbol's name hash.
//!
//! ## Confidence
//!
//! Symbol-derived classes are [`Source::Observed`] at value `1.0` —
//! the mangled name is in the binary itself. The `has_vtable` flag is
//! [`Source::Observed`] when a `_ZTV*` symbol exists, otherwise
//! [`Source::Derived`] at `0.0` (we have no evidence, but absence is
//! not strong evidence). The numeric values combine through
//! [`Confidence::join`] when a class is supported by both a member-
//! function mangling and a vtable symbol.
//!
//! ## Determinism
//!
//! Pure function. Symbols are iterated in `BinaryModel` order, then
//! the output is sorted lexicographically by class chain, then by
//! `(MemberSortKey, address, mangled)` within a class. The free-
//! function list is sorted by address then mangled name.

use std::collections::BTreeMap;

use dac_binfmt::{BinaryModel, RelocationKind, SymbolKind};
use dac_core::{Confidence, EdgeKind, EvidenceGraph, EvidenceId, EvidenceNode, IrLayer, Source};
use dac_recovery::FunctionSet;

use crate::ast::AccessSpec;
use crate::mangle::{self, ItaniumSymbol, MemberKind};

/// Default confidence value for a class observed via its mangled
/// member-function or vtable symbol. Mirrors
/// [`dac_recovery::SYMBOL_CONFIDENCE`].
pub const CLASS_SYMBOL_CONFIDENCE: f32 = 1.0;

/// Output of [`recover_classes`].
#[derive(Debug, Clone, Default, PartialEq)]
pub struct RecoveredClasses {
    /// Classes recovered from the binary, sorted lexicographically by
    /// `scope_chain` then class name. Stable across re-runs.
    pub classes: Vec<RecoveredClass>,
    /// Recovered free functions — Itanium-mangled top-level functions
    /// (`_Z<name>…`) plus C-style functions whose name is not mangled.
    /// Sorted by `(address, mangled)`.
    pub free_functions: Vec<RecoveredFreeFunction>,
    /// Per-source counts for the manifest / debug output.
    pub stats: ClassRecoveryStats,
}

/// One recovered class.
#[derive(Debug, Clone, PartialEq)]
pub struct RecoveredClass {
    /// Class name (the innermost segment of the nested-name chain).
    pub name: String,
    /// Outer scope chain — everything but the final segment. Empty for
    /// a top-level class.
    pub scope_chain: Vec<String>,
    /// `true` when a `_ZTV<class>` symbol exists. Implies the class is
    /// polymorphic; lower / emit promote its member functions to
    /// `virtual` and add a `virtual ~Class();` declaration so the
    /// emitted unit reflects the binary's runtime shape.
    pub has_vtable: bool,
    /// `true` when a `_ZTI<class>` (typeinfo) symbol exists. Polymorphic
    /// classes with RTTI on always have this.
    pub has_typeinfo: bool,
    /// Member functions discovered for the class, sorted by
    /// `(MemberSortKey, address, mangled)`.
    pub members: Vec<RecoveredMember>,
    /// Base classes recovered from Itanium typeinfo relocations
    /// (`__si_class_type_info` and `__vmi_class_type_info` shapes
    /// in `.data.rel.ro`). Empty when the class has no typeinfo
    /// symbol, when the typeinfo is `__class_type_info` (no
    /// bases), or when the relocation table did not carry parent
    /// pointers for it. Sorted in the order the typeinfo records
    /// them so a binary's declared inheritance order is preserved.
    pub bases: Vec<RecoveredBase>,
    /// Joined confidence across the contributing signals — member-
    /// function mangling, vtable symbol, typeinfo symbol.
    pub confidence: Confidence,
    /// Handle to the class's `IrNode { layer: Source }` node in the
    /// evidence graph.
    pub evidence: EvidenceId,
}

/// One recovered base class.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveredBase {
    /// Fully qualified parent class name (e.g. `Animal` or
    /// `Foo::Bar`), demangled from the parent `_ZTI<chain>` symbol.
    pub qualified_name: String,
    /// Access specifier. B3.11 always emits [`AccessSpec::Public`]:
    /// the lower 8 bits of `__base_class_type_info::offset_flags`
    /// would carry public / private / virtual flags, but reading
    /// those bytes requires plumbing the input buffer through the
    /// recovery pass; the field is here so a future batch can
    /// refine the access spelling without a schema change.
    pub access: AccessSpec,
}

impl RecoveredClass {
    /// `Scope::Inner::Name` qualified spelling used by emit / debug.
    #[must_use]
    pub fn qualified_name(&self) -> String {
        let mut s = String::new();
        for seg in &self.scope_chain {
            s.push_str(seg);
            s.push_str("::");
        }
        s.push_str(&self.name);
        s
    }
}

/// One recovered member function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveredMember {
    /// Member name. For ctor/dtor variants this is the
    /// label-renderable name from [`MemberKind::label`] applied to the
    /// class name (e.g. `Dog_ctor_v1`, `~Dog_dtor_v2`) so emit/lower
    /// can pick the spelling without re-running the mangler.
    pub name: String,
    /// Original mangled symbol — kept so the annotation channel can
    /// cite it.
    pub mangled: String,
    /// Address of the function in the binary.
    pub address: u64,
    /// What kind of member: method, ctor, dtor.
    pub kind: MemberCategory,
    /// `true` for `_ZNK…` const-qualified member functions.
    pub is_const: bool,
    /// `true` when the owning class has `has_vtable = true`. emit/lower
    /// promote the C++ declaration to `virtual`.
    pub is_virtual: bool,
    /// Handle to the function's evidence node (the same handle the
    /// `dac_recovery::Function` carries). `None` only when the symbol
    /// is recovered but no matching entry exists in the function set —
    /// rare, but possible for weak/duplicated symbols.
    pub evidence: Option<EvidenceId>,
}

/// Coarse classification of a recovered member function. Mirrors
/// [`MemberKind`] but flattens the variant digits behind a
/// `MemberCategory::CtorVariant` / `MemberCategory::DtorVariant` so
/// downstream sorting and rendering does not have to peel
/// `MemberKind`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum MemberCategory {
    /// Named method.
    Method,
    /// Ctor variant (1 = complete object, 2 = base object, 3 =
    /// allocating).
    CtorVariant(u8),
    /// Dtor variant (0 = deleting, 1 = complete, 2 = base).
    DtorVariant(u8),
}

/// One recovered free function.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveredFreeFunction {
    /// Demangled name (when the symbol parsed as a `_Z<name>` form),
    /// otherwise the raw mangled / unmangled symbol.
    pub name: String,
    /// Original mangled symbol exactly as it appears in the binary.
    pub mangled: String,
    /// Address of the function.
    pub address: u64,
    /// Handle to the function's evidence node.
    pub evidence: Option<EvidenceId>,
}

/// Counts surfaced through the manifest / `--debug` channel.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ClassRecoveryStats {
    /// Number of classes recovered.
    pub classes: u32,
    /// Number of classes with a `_ZTV*` vtable symbol.
    pub polymorphic_classes: u32,
    /// Number of member functions recovered across all classes.
    pub member_functions: u32,
    /// Number of free functions, Itanium-mangled or C-style.
    pub free_functions: u32,
}

/// Run class recovery against a binary.
///
/// `model` provides the symbol table and architecture metadata.
/// `functions` provides the recovered function entries (so each
/// member function can carry the same `EvidenceId` as its
/// `dac_recovery::Function`). `graph` is mutated with one
/// `IrNode { layer: Source, id }` per class and one
/// `KnowledgeFact` node per `_ZTV*` symbol when present.
#[must_use]
pub fn recover_classes(
    model: &BinaryModel,
    functions: &FunctionSet,
    graph: &mut EvidenceGraph,
) -> RecoveredClasses {
    // Index function evidence handles by address for quick lookup.
    let mut func_evidence: BTreeMap<u64, EvidenceId> = BTreeMap::new();
    for f in &functions.functions {
        func_evidence.insert(f.address, f.evidence);
    }

    // Working state: class chain → in-progress descriptor.
    let mut working: BTreeMap<ChainKey, ClassBuilder> = BTreeMap::new();
    let mut free: Vec<RecoveredFreeFunction> = Vec::new();
    let mut text_symbol_addresses: BTreeMap<u64, &str> = BTreeMap::new();

    // First pass: collect text-kind symbols into the working bag.
    for sym in &model.symbols {
        if !matches!(sym.kind, SymbolKind::Text | SymbolKind::Label) {
            continue;
        }
        if sym.undefined {
            continue;
        }
        // Track addresses we've seen as text symbols so a later parse
        // pass over data symbols (vtables / typeinfo) does not
        // misclassify a code address.
        text_symbol_addresses.insert(sym.address, sym.name.as_str());

        match mangle::parse(&sym.name) {
            Some(ItaniumSymbol::Member {
                chain,
                is_const,
                kind,
                ..
            }) => {
                let class_name = chain
                    .last()
                    .cloned()
                    .expect("parse_nested guarantees non-empty chain");
                let scope_chain = chain[..chain.len() - 1].to_vec();
                let key = ChainKey {
                    scope: scope_chain.clone(),
                    name: class_name.clone(),
                };
                let builder = working.entry(key).or_insert_with(|| ClassBuilder {
                    name: class_name.clone(),
                    scope_chain,
                    has_vtable: false,
                    has_typeinfo: false,
                    members: Vec::new(),
                    typeinfo_address: None,
                    bases: Vec::new(),
                });
                let (member_label, category) = match &kind {
                    MemberKind::Method { name } => (name.clone(), MemberCategory::Method),
                    MemberKind::Constructor { variant } => (
                        MemberKind::Constructor { variant: *variant }.label(&class_name),
                        MemberCategory::CtorVariant(*variant),
                    ),
                    MemberKind::Destructor { variant } => (
                        MemberKind::Destructor { variant: *variant }.label(&class_name),
                        MemberCategory::DtorVariant(*variant),
                    ),
                };
                builder.members.push(RecoveredMember {
                    name: member_label,
                    mangled: sym.name.clone(),
                    address: sym.address,
                    kind: category,
                    is_const,
                    is_virtual: false,
                    evidence: func_evidence.get(&sym.address).copied(),
                });
            }
            Some(ItaniumSymbol::Free { name, .. }) => {
                free.push(RecoveredFreeFunction {
                    name,
                    mangled: sym.name.clone(),
                    address: sym.address,
                    evidence: func_evidence.get(&sym.address).copied(),
                });
            }
            // `_ZTV*` etc. should not appear as text symbols (they are
            // data), but if a future binary mislabels them, drop on
            // the floor.
            Some(_) => {}
            None => {
                // Not Itanium-mangled. If it lines up with a recovered
                // function, surface it as a free function so the
                // lowering can render it. Most of these are C-style
                // exports (`main`, `_start`, runtime stubs).
                if func_evidence.contains_key(&sym.address) {
                    free.push(RecoveredFreeFunction {
                        name: sym.name.clone(),
                        mangled: sym.name.clone(),
                        address: sym.address,
                        evidence: func_evidence.get(&sym.address).copied(),
                    });
                }
            }
        }
    }

    // Second pass: data symbols give us vtable / typeinfo signals.
    for sym in &model.symbols {
        if matches!(sym.kind, SymbolKind::Text | SymbolKind::Label) {
            continue;
        }
        let Some(parsed) = mangle::parse(&sym.name) else {
            continue;
        };
        match parsed {
            ItaniumSymbol::Vtable { class_chain } => {
                let (scope, name) = split_chain(&class_chain);
                let key = ChainKey {
                    scope: scope.clone(),
                    name: name.clone(),
                };
                working
                    .entry(key)
                    .or_insert_with(|| ClassBuilder {
                        name,
                        scope_chain: scope,
                        has_vtable: true,
                        has_typeinfo: false,
                        members: Vec::new(),
                        typeinfo_address: None,
                        bases: Vec::new(),
                    })
                    .has_vtable = true;
            }
            ItaniumSymbol::TypeInfo { class_chain } => {
                let (scope, name) = split_chain(&class_chain);
                let key = ChainKey {
                    scope: scope.clone(),
                    name: name.clone(),
                };
                let builder = working.entry(key).or_insert_with(|| ClassBuilder {
                    name,
                    scope_chain: scope,
                    has_vtable: false,
                    has_typeinfo: true,
                    members: Vec::new(),
                    typeinfo_address: None,
                    bases: Vec::new(),
                });
                builder.has_typeinfo = true;
                // Record the typeinfo VA so the third pass can walk
                // `__si` / `__vmi` relocations into base parents.
                // Skip undefined / weak-undefined entries; only
                // defined typeinfo carries usable relocations.
                if !sym.undefined {
                    builder.typeinfo_address = Some(sym.address);
                }
            }
            // Other data symbols (`_ZTS*`, `_ZTT*`, free / member) are
            // not needed to identify the class — `_ZTV*` and `_ZTI*`
            // are sufficient and orthogonal.
            _ => {}
        }
    }

    // Third pass: walk typeinfo relocations into base-class
    // parents. Itanium ABI layout in `.data.rel.ro`:
    //
    //   __class_type_info     { vptr, name }                     // 16 B
    //   __si_class_type_info  { vptr, name, base }               // 24 B
    //   __vmi_class_type_info { vptr, name, flags, count, bases[]} // 24 + 16*count B
    //
    // The vptr (offset 0) is a relocation against the libstdc++ /
    // libc++abi typeinfo vtable; its target symbol name tells us
    // which shape applies. Parent pointers live at offset 16 (SI)
    // or 24 + 16*i (VMI).
    let typeinfo_index = typeinfo_index_by_va(&working);
    // Index typeinfo symbols by VA so the walker can look up the
    // `Symbol::size` field — the primary kind discriminator.
    let typeinfo_size_by_va: BTreeMap<u64, u64> = model
        .symbols
        .iter()
        .filter(|s| !s.undefined && matches!(s.kind, SymbolKind::Data))
        .filter(|s| {
            mangle::parse(&s.name).is_some_and(|p| matches!(p, ItaniumSymbol::TypeInfo { .. }))
        })
        .map(|s| (s.address, s.size))
        .collect();
    for builder in working.values_mut() {
        let Some(ti_va) = builder.typeinfo_address else {
            continue;
        };
        let size = typeinfo_size_by_va.get(&ti_va).copied().unwrap_or(0);
        builder.bases = walk_typeinfo(model, ti_va, size, &typeinfo_index);
    }

    // Finalise: sort, mint evidence nodes, count stats.
    let mut classes: Vec<RecoveredClass> = working
        .into_values()
        .map(|mut b| {
            // Stable order within a class.
            b.members.sort_by(|a, b| {
                a.kind
                    .cmp(&b.kind)
                    .then_with(|| a.address.cmp(&b.address))
                    .then_with(|| a.mangled.cmp(&b.mangled))
            });
            let class_index = graph.node_count() as u64;
            let class_node = graph.add_node(EvidenceNode::IrNode {
                layer: IrLayer::Source,
                id: class_index,
            });
            // Link each member function's evidence to the class node so
            // `--debug` can answer "why does this class exist?".
            for m in &b.members {
                if let Some(ev) = m.evidence {
                    graph.add_edge(ev, class_node, EdgeKind::Supports);
                }
            }
            // Vtable signal records a knowledge-fact node and links it
            // forward; the fact id is the lower 64 bits of FNV-1a of
            // the qualified class name, which is reproducible across
            // re-runs.
            if b.has_vtable {
                let qualified = qualify(&b.scope_chain, &b.name);
                let fact =
                    graph.add_node(EvidenceNode::KnowledgeFact(fnv1a_64(qualified.as_bytes())));
                graph.add_edge(fact, class_node, EdgeKind::Supports);
            }
            // Now that the class is in the graph, walk the members and
            // mark them virtual.
            let is_polymorphic = b.has_vtable;
            for m in &mut b.members {
                m.is_virtual = is_polymorphic;
            }
            let confidence = Confidence::new(CLASS_SYMBOL_CONFIDENCE, Source::Observed);
            RecoveredClass {
                name: b.name,
                scope_chain: b.scope_chain,
                has_vtable: b.has_vtable,
                has_typeinfo: b.has_typeinfo,
                members: b.members,
                bases: b.bases,
                confidence,
                evidence: class_node,
            }
        })
        .collect();
    classes.sort_by(|a, b| {
        a.scope_chain
            .cmp(&b.scope_chain)
            .then_with(|| a.name.cmp(&b.name))
    });

    // Free functions: drop duplicates that already appear as class
    // members (rare; happens when a weak symbol is aliased), then sort.
    let mut all_member_addresses: std::collections::BTreeSet<u64> =
        std::collections::BTreeSet::new();
    for c in &classes {
        for m in &c.members {
            all_member_addresses.insert(m.address);
        }
    }
    free.retain(|f| !all_member_addresses.contains(&f.address));
    free.sort_by(|a, b| {
        a.address
            .cmp(&b.address)
            .then_with(|| a.mangled.cmp(&b.mangled))
    });
    // Dedup contiguous identical (address, mangled) entries — the
    // symbol table can hold the same symbol twice (`.symtab` /
    // `.dynsym` overlap).
    free.dedup_by(|a, b| a.address == b.address && a.mangled == b.mangled);

    let stats = ClassRecoveryStats {
        classes: classes.len() as u32,
        polymorphic_classes: classes.iter().filter(|c| c.has_vtable).count() as u32,
        member_functions: classes.iter().map(|c| c.members.len() as u32).sum(),
        free_functions: free.len() as u32,
    };

    RecoveredClasses {
        classes,
        free_functions: free,
        stats,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ChainKey {
    scope: Vec<String>,
    name: String,
}

struct ClassBuilder {
    name: String,
    scope_chain: Vec<String>,
    has_vtable: bool,
    has_typeinfo: bool,
    members: Vec<RecoveredMember>,
    /// Virtual address of the `_ZTI<class>` typeinfo symbol when
    /// one was seen. Drives the base-class walker in the third
    /// pass.
    typeinfo_address: Option<u64>,
    /// Bases populated by the typeinfo walker; remains empty when
    /// no `_ZTI<class>` symbol existed or when its kind is
    /// `__class_type_info` (no bases).
    bases: Vec<RecoveredBase>,
}

fn split_chain(chain: &[String]) -> (Vec<String>, String) {
    let last = chain.last().cloned().unwrap_or_default();
    let scope = chain[..chain.len().saturating_sub(1)].to_vec();
    (scope, last)
}

/// Itanium typeinfo shape inferred from the vptr relocation's
/// target symbol name. See the third pass in [`recover_classes`]
/// for layout details.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum TypeInfoShape {
    /// `__class_type_info`: no bases.
    None,
    /// `__si_class_type_info`: single public base at offset 16.
    SingleInheritance,
    /// `__vmi_class_type_info`: multiple / virtual bases starting
    /// at offset 24, 16 bytes per entry.
    Multiple,
}

/// Build a `VA → qualified parent name` index over the typeinfo
/// symbols already recorded in the working bag. Used by
/// [`walk_typeinfo`] to map a base relocation's target VA back to
/// its parent class spelling.
fn typeinfo_index_by_va(working: &BTreeMap<ChainKey, ClassBuilder>) -> BTreeMap<u64, String> {
    let mut out: BTreeMap<u64, String> = BTreeMap::new();
    for (key, b) in working {
        if let Some(va) = b.typeinfo_address {
            out.insert(va, qualify(&key.scope, &key.name));
        }
    }
    out
}

/// Walk the typeinfo at `ti_va` and resolve its base parents.
fn walk_typeinfo(
    model: &BinaryModel,
    ti_va: u64,
    size: u64,
    typeinfo_index: &BTreeMap<u64, String>,
) -> Vec<RecoveredBase> {
    let shape = classify_typeinfo_shape(model, ti_va, size);
    match shape {
        TypeInfoShape::None => Vec::new(),
        TypeInfoShape::SingleInheritance => {
            // Exactly one base pointer at offset 16.
            let Some(parent_va) = resolve_relocation_target(model, ti_va.wrapping_add(16)) else {
                return Vec::new();
            };
            let Some(qualified_name) = typeinfo_index.get(&parent_va).cloned() else {
                return Vec::new();
            };
            vec![RecoveredBase {
                qualified_name,
                access: AccessSpec::Public,
            }]
        }
        TypeInfoShape::Multiple => {
            // Base pointers start at offset 24, every 16 bytes.
            // Walk until we run out of relocations the index
            // recognises. A hard cap guards against runaway in
            // malformed binaries.
            let mut out = Vec::new();
            const MAX_BASES: usize = 64;
            for i in 0..MAX_BASES {
                let off = ti_va.wrapping_add(24 + 16 * i as u64);
                let Some(parent_va) = resolve_relocation_target(model, off) else {
                    break;
                };
                let Some(qualified_name) = typeinfo_index.get(&parent_va).cloned() else {
                    break;
                };
                out.push(RecoveredBase {
                    qualified_name,
                    access: AccessSpec::Public,
                });
            }
            out
        }
    }
}

/// Classify the typeinfo shape.
///
/// Itanium layout pins the byte size: 16 bytes for
/// `__class_type_info` (vptr + name), 24 bytes for
/// `__si_class_type_info` (one base pointer appended), and
/// `24 + 16 * base_count` bytes for `__vmi_class_type_info`. So
/// `Symbol::size` discriminates the three shapes reliably without
/// reading the actual bytes — important because the input buffer
/// is not plumbed through this pass.
///
/// When `size` is zero (stripped / weak symbol with no recorded
/// size) we fall back to the vptr relocation's target name. The
/// fallback is best-effort: the ELF parser is currently known to
/// mis-map the symbol index on dynamic relocations into
/// `.data.rel.ro` (see the `B3.18` shelf entry), so the fallback
/// frequently returns `None`. The recovery pass degrades to
/// `bases: Vec::new()` in that case, which the leading comment
/// renders as `(none)`.
fn classify_typeinfo_shape(model: &BinaryModel, ti_va: u64, size: u64) -> TypeInfoShape {
    match size {
        16 => return TypeInfoShape::None,
        24 => return TypeInfoShape::SingleInheritance,
        s if s > 24 => return TypeInfoShape::Multiple,
        _ => {}
    }
    // Size is zero or other; fall back to the vptr relocation's
    // target name. Defensive against stripped binaries.
    let Some(rel) = find_relocation_at(model, ti_va) else {
        return TypeInfoShape::None;
    };
    let Some(sym_idx) = rel.symbol else {
        return TypeInfoShape::None;
    };
    let Some(sym) = model.symbols.get(sym_idx) else {
        return TypeInfoShape::None;
    };
    if sym.name.contains("__vmi_class_type_info") {
        TypeInfoShape::Multiple
    } else if sym.name.contains("__si_class_type_info") {
        TypeInfoShape::SingleInheritance
    } else {
        TypeInfoShape::None
    }
}

/// Resolve the virtual address a relocation at `va` points at.
/// Returns `None` if no relocation patches `va` or if the
/// relocation kind / fields don't give us a target VA we can name.
fn resolve_relocation_target(model: &BinaryModel, va: u64) -> Option<u64> {
    let rel = find_relocation_at(model, va)?;
    match (rel.symbol, rel.kind) {
        // Absolute / Glob with a symbol index: symbol.address + addend.
        (Some(idx), _) => {
            let sym = model.symbols.get(idx)?;
            Some(sym.address.wrapping_add(rel.addend as u64))
        }
        // R_X86_64_RELATIVE and friends: addend carries the VA.
        (None, RelocationKind::Relative) => Some(rel.addend as u64),
        _ => None,
    }
}

/// Linear scan for the relocation whose `offset` matches `va`.
/// The relocation table is small (single-digit thousands at most
/// even for large binaries) so a linear scan is fine; switching to
/// a sorted index would only be worth it if base recovery
/// dominates the recovery profile, which it does not.
fn find_relocation_at(model: &BinaryModel, va: u64) -> Option<&dac_binfmt::Relocation> {
    model.relocations.iter().find(|r| r.offset == va)
}

fn qualify(scope: &[String], name: &str) -> String {
    let mut s = String::new();
    for seg in scope {
        s.push_str(seg);
        s.push_str("::");
    }
    s.push_str(name);
    s
}

/// FNV-1a 64-bit hash. Used to mint a stable
/// [`EvidenceNode::KnowledgeFact`] id from a class's qualified name.
fn fnv1a_64(bytes: &[u8]) -> u64 {
    const OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const PRIME: u64 = 0x100_0000_01b3;
    let mut h = OFFSET;
    for b in bytes {
        h ^= *b as u64;
        h = h.wrapping_mul(PRIME);
    }
    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use dac_binfmt::{
        Architecture, BinaryFormat, Bits, Endian, Symbol, SymbolBinding, SymbolSource,
    };
    use dac_core::EvidenceNode;
    use dac_recovery::functions::{Function, SourceMask};

    fn empty_model() -> BinaryModel {
        BinaryModel {
            format: BinaryFormat::Elf,
            architecture: Architecture::X86_64,
            endian: Endian::Little,
            bits: Bits::Bits64,
            entry: None,
            size: 0,
            sections: Vec::new(),
            segments: Vec::new(),
            symbols: Vec::new(),
            imports: Vec::new(),
            exports: Vec::new(),
            relocations: Vec::new(),
            strings: Vec::new(),
            needed_libraries: Vec::new(),
        }
    }

    fn text_symbol(name: &str, address: u64) -> Symbol {
        Symbol {
            name: name.to_string(),
            address,
            size: 0,
            kind: SymbolKind::Text,
            binding: SymbolBinding::Global,
            section: None,
            source: SymbolSource::Symtab,
            undefined: false,
        }
    }

    fn data_symbol(name: &str, address: u64) -> Symbol {
        Symbol {
            name: name.to_string(),
            address,
            size: 0,
            kind: SymbolKind::Data,
            binding: SymbolBinding::Global,
            section: None,
            source: SymbolSource::Symtab,
            undefined: false,
        }
    }

    fn empty_function_set() -> FunctionSet {
        FunctionSet {
            functions: Vec::new(),
            stats: Default::default(),
        }
    }

    fn function_set_with(addresses: &[u64], graph: &mut EvidenceGraph) -> FunctionSet {
        let functions: Vec<Function> = addresses
            .iter()
            .map(|&addr| Function {
                address: addr,
                end: None,
                name: None,
                confidence: Confidence::new(1.0, Source::Observed),
                sources: SourceMask::SYMBOL,
                evidence: graph.add_node(EvidenceNode::IrNode {
                    layer: IrLayer::Cfg,
                    id: addr,
                }),
                kind: Default::default(),
            })
            .collect();
        FunctionSet {
            functions,
            stats: Default::default(),
        }
    }

    #[test]
    fn empty_input_produces_empty_output() {
        let mut g = EvidenceGraph::new();
        let model = empty_model();
        let fs = empty_function_set();
        let r = recover_classes(&model, &fs, &mut g);
        assert!(r.classes.is_empty());
        assert!(r.free_functions.is_empty());
        assert_eq!(r.stats, ClassRecoveryStats::default());
    }

    #[test]
    fn single_member_function_creates_class() {
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        model.symbols.push(text_symbol("_ZN3Dog5speakEv", 0x1000));
        let fs = function_set_with(&[0x1000], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        assert_eq!(r.classes.len(), 1);
        let c = &r.classes[0];
        assert_eq!(c.name, "Dog");
        assert_eq!(c.scope_chain, Vec::<String>::new());
        assert_eq!(c.members.len(), 1);
        assert_eq!(c.members[0].name, "speak");
        assert_eq!(c.members[0].kind, MemberCategory::Method);
        assert_eq!(c.members[0].address, 0x1000);
        assert_eq!(c.confidence.source(), Source::Observed);
        assert!(c.members[0].evidence.is_some());
        assert_eq!(r.stats.classes, 1);
        assert_eq!(r.stats.member_functions, 1);
        assert_eq!(r.stats.polymorphic_classes, 0);
    }

    #[test]
    fn vtable_symbol_promotes_class_to_polymorphic() {
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        model.symbols.push(text_symbol("_ZN3Dog5speakEv", 0x1000));
        model.symbols.push(data_symbol("_ZTV3Dog", 0x4000));
        let fs = function_set_with(&[0x1000], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        assert_eq!(r.classes.len(), 1);
        let c = &r.classes[0];
        assert!(c.has_vtable);
        assert!(c.members[0].is_virtual);
        assert_eq!(r.stats.polymorphic_classes, 1);
    }

    #[test]
    fn ctor_and_dtor_variants_all_kept() {
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        for (m, addr) in [
            ("_ZN3DogC1Ev", 0x1010),
            ("_ZN3DogC2Ev", 0x1020),
            ("_ZN3DogD0Ev", 0x1030),
            ("_ZN3DogD1Ev", 0x1040),
            ("_ZN3DogD2Ev", 0x1050),
        ] {
            model.symbols.push(text_symbol(m, addr));
        }
        let fs = function_set_with(&[0x1010, 0x1020, 0x1030, 0x1040, 0x1050], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        assert_eq!(r.classes.len(), 1);
        let c = &r.classes[0];
        // Sort order: Method < CtorVariant(_) < DtorVariant(_) by
        // `MemberCategory`'s derived Ord. So ctors come before dtors.
        assert_eq!(c.members.len(), 5);
        assert_eq!(c.members[0].kind, MemberCategory::CtorVariant(1));
        assert_eq!(c.members[1].kind, MemberCategory::CtorVariant(2));
        assert_eq!(c.members[2].kind, MemberCategory::DtorVariant(0));
        assert_eq!(c.members[3].kind, MemberCategory::DtorVariant(1));
        assert_eq!(c.members[4].kind, MemberCategory::DtorVariant(2));
    }

    #[test]
    fn nested_class_keeps_scope_chain() {
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        model
            .symbols
            .push(text_symbol("_ZN3Foo3Bar4funcEv", 0x2000));
        let fs = function_set_with(&[0x2000], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        assert_eq!(r.classes.len(), 1);
        assert_eq!(r.classes[0].name, "Bar");
        assert_eq!(r.classes[0].scope_chain, vec!["Foo".to_string()]);
        assert_eq!(r.classes[0].qualified_name(), "Foo::Bar");
    }

    #[test]
    fn free_function_recorded_separately() {
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        model
            .symbols
            .push(text_symbol("_Z6chorusPK6AnimalS1_", 0x3000));
        let fs = function_set_with(&[0x3000], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        assert!(r.classes.is_empty());
        assert_eq!(r.free_functions.len(), 1);
        assert_eq!(r.free_functions[0].name, "chorus");
        assert_eq!(r.free_functions[0].mangled, "_Z6chorusPK6AnimalS1_");
    }

    #[test]
    fn unmangled_main_lands_on_free_pile_when_known_function() {
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        model.symbols.push(text_symbol("main", 0x4000));
        let fs = function_set_with(&[0x4000], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        assert!(r.classes.is_empty());
        assert_eq!(r.free_functions.len(), 1);
        assert_eq!(r.free_functions[0].name, "main");
    }

    #[test]
    fn member_address_dedups_off_free_pile() {
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        // Symbol table sometimes has both a mangled and aliased name
        // pointing at the same code address. The aliased name is
        // unmangled (e.g. `Dog::speak`'s thunk listed as `_ZN3Dog…`
        // plus a debug-info aliasing). We keep the class member and
        // drop the duplicate from the free pile.
        model.symbols.push(text_symbol("_ZN3Dog5speakEv", 0x1000));
        model.symbols.push(text_symbol("dog_speak_alias", 0x1000));
        let fs = function_set_with(&[0x1000], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        assert_eq!(r.classes.len(), 1);
        assert!(r.free_functions.is_empty());
    }

    #[test]
    fn class_evidence_node_lives_at_source_layer() {
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        model.symbols.push(text_symbol("_ZN3Dog5speakEv", 0x1000));
        let fs = function_set_with(&[0x1000], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        let class_ev = r.classes[0].evidence;
        assert!(matches!(
            g.node(class_ev),
            Some(EvidenceNode::IrNode {
                layer: IrLayer::Source,
                ..
            })
        ));
    }

    #[test]
    fn output_is_deterministic_across_reruns() {
        let mut g1 = EvidenceGraph::new();
        let mut g2 = EvidenceGraph::new();
        let mut model = empty_model();
        for (m, addr) in [
            ("_ZN3Dog5speakEv", 0x1000),
            ("_ZN3Cat5speakEv", 0x1100),
            ("_ZN3DogD1Ev", 0x1010),
            ("_ZN3CatD1Ev", 0x1110),
        ] {
            model.symbols.push(text_symbol(m, addr));
        }
        model.symbols.push(data_symbol("_ZTV3Dog", 0x4000));
        model.symbols.push(data_symbol("_ZTV3Cat", 0x4100));
        let fs1 = function_set_with(&[0x1000, 0x1010, 0x1100, 0x1110], &mut g1);
        let fs2 = function_set_with(&[0x1000, 0x1010, 0x1100, 0x1110], &mut g2);
        let a = recover_classes(&model, &fs1, &mut g1);
        let b = recover_classes(&model, &fs2, &mut g2);
        // Strip the EvidenceIds (graph-specific) from the comparison.
        assert_eq!(
            a.classes
                .iter()
                .map(|c| (&c.name, &c.scope_chain, c.has_vtable, c.members.len()))
                .collect::<Vec<_>>(),
            b.classes
                .iter()
                .map(|c| (&c.name, &c.scope_chain, c.has_vtable, c.members.len()))
                .collect::<Vec<_>>(),
        );
        assert_eq!(a.stats, b.stats);
    }

    // -- B3.11: typeinfo walker tests ----------------------------

    fn data_symbol_with_size(name: &str, address: u64, size: u64) -> Symbol {
        Symbol {
            name: name.to_string(),
            address,
            size,
            kind: SymbolKind::Data,
            binding: SymbolBinding::Global,
            section: None,
            source: SymbolSource::Symtab,
            undefined: false,
        }
    }

    fn relative_relocation(offset: u64, target_va: u64) -> dac_binfmt::Relocation {
        dac_binfmt::Relocation {
            section: None,
            offset,
            kind: RelocationKind::Relative,
            symbol: None,
            addend: target_va as i64,
        }
    }

    #[test]
    fn class_type_info_size_16_recovers_no_bases() {
        // Animal: `_ZTI6Animal` at 0x3d70 with size 16 → no bases.
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        model
            .symbols
            .push(text_symbol("_ZN6Animal5speakEv", 0x1000));
        model
            .symbols
            .push(data_symbol_with_size("_ZTI6Animal", 0x3d70, 16));
        let fs = function_set_with(&[0x1000], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        let animal = r.classes.iter().find(|c| c.name == "Animal").unwrap();
        assert!(animal.bases.is_empty(), "size-16 typeinfo has no bases");
    }

    #[test]
    fn si_class_type_info_size_24_recovers_single_base() {
        // Dog: typeinfo at 0x3d80, size 24, base pointer relocation
        // at 0x3d80+16=0x3d90 points to Animal's typeinfo @ 0x3d70.
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        model.symbols.push(text_symbol("_ZN3Dog5speakEv", 0x1000));
        model
            .symbols
            .push(text_symbol("_ZN6Animal5speakEv", 0x2000));
        model
            .symbols
            .push(data_symbol_with_size("_ZTI6Animal", 0x3d70, 16));
        model
            .symbols
            .push(data_symbol_with_size("_ZTI3Dog", 0x3d80, 24));
        model.relocations.push(relative_relocation(0x3d90, 0x3d70));
        let fs = function_set_with(&[0x1000, 0x2000], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        let dog = r.classes.iter().find(|c| c.name == "Dog").unwrap();
        assert_eq!(dog.bases.len(), 1);
        assert_eq!(dog.bases[0].qualified_name, "Animal");
        assert_eq!(dog.bases[0].access, AccessSpec::Public);
    }

    #[test]
    fn vmi_class_type_info_walks_multiple_bases_until_relocations_end() {
        // Hybrid: typeinfo at 0x4000 size 56 (24 header + 16*2),
        // two base pointer relocations at 0x4018 and 0x4028 pointing
        // at Foo and Bar typeinfos.
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        model
            .symbols
            .push(text_symbol("_ZN6Hybrid5speakEv", 0x1000));
        model.symbols.push(text_symbol("_ZN3Foo5speakEv", 0x2000));
        model.symbols.push(text_symbol("_ZN3Bar5speakEv", 0x3000));
        model
            .symbols
            .push(data_symbol_with_size("_ZTI3Foo", 0x3d00, 16));
        model
            .symbols
            .push(data_symbol_with_size("_ZTI3Bar", 0x3d20, 16));
        model
            .symbols
            .push(data_symbol_with_size("_ZTI6Hybrid", 0x4000, 56));
        // Hybrid's two base pointer relocations.
        model.relocations.push(relative_relocation(0x4018, 0x3d00));
        model.relocations.push(relative_relocation(0x4028, 0x3d20));
        let fs = function_set_with(&[0x1000, 0x2000, 0x3000], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        let hybrid = r.classes.iter().find(|c| c.name == "Hybrid").unwrap();
        assert_eq!(hybrid.bases.len(), 2, "two bases at offsets 24 and 40");
        let parent_names: Vec<&str> = hybrid
            .bases
            .iter()
            .map(|b| b.qualified_name.as_str())
            .collect();
        // Walk order preserves the declared inheritance order.
        assert_eq!(parent_names, vec!["Foo", "Bar"]);
    }

    #[test]
    fn vmi_walk_stops_at_first_missing_relocation() {
        // Hybrid declares size for three bases but only the first
        // two relocations are present. The walker stops at the gap
        // rather than running off the end.
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        model
            .symbols
            .push(text_symbol("_ZN6Hybrid5speakEv", 0x1000));
        model
            .symbols
            .push(data_symbol_with_size("_ZTI3Foo", 0x3d00, 16));
        model
            .symbols
            .push(data_symbol_with_size("_ZTI3Bar", 0x3d20, 16));
        model
            .symbols
            .push(data_symbol_with_size("_ZTI6Hybrid", 0x4000, 72));
        model.relocations.push(relative_relocation(0x4018, 0x3d00));
        model.relocations.push(relative_relocation(0x4028, 0x3d20));
        // No relocation at 0x4038.
        let fs = function_set_with(&[0x1000], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        let hybrid = r.classes.iter().find(|c| c.name == "Hybrid").unwrap();
        assert_eq!(hybrid.bases.len(), 2);
    }

    #[test]
    fn si_walker_drops_base_when_parent_typeinfo_unknown() {
        // Dog's base pointer relocation targets an address that no
        // recovered typeinfo lives at. The walker drops the base
        // rather than minting a synthetic name — we'd rather emit a
        // missing `: public Base` than a wrong one (I-3, I-6).
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        model.symbols.push(text_symbol("_ZN3Dog5speakEv", 0x1000));
        model
            .symbols
            .push(data_symbol_with_size("_ZTI3Dog", 0x3d80, 24));
        // Points at 0xdead — no _ZTI<class> lives there.
        model.relocations.push(relative_relocation(0x3d90, 0xdead));
        let fs = function_set_with(&[0x1000], &mut g);
        let r = recover_classes(&model, &fs, &mut g);
        let dog = r.classes.iter().find(|c| c.name == "Dog").unwrap();
        assert!(dog.bases.is_empty());
    }

    #[test]
    fn typeinfo_with_undefined_symbol_address_is_skipped() {
        // `__cxxabiv1::__class_type_info` lands in the symbol table
        // as an undefined import at address 0. The walker must skip
        // it: an undefined typeinfo has no relocations of its own,
        // and the address 0 would otherwise collide with the
        // `addend == 0` form of a null relocation.
        let mut g = EvidenceGraph::new();
        let mut model = empty_model();
        let mut sym = data_symbol("_ZTIN10__cxxabiv117__class_type_infoE", 0);
        sym.undefined = true;
        model.symbols.push(sym);
        let fs = empty_function_set();
        let r = recover_classes(&model, &fs, &mut g);
        // No class minted: the typeinfo is undefined, has no
        // typeinfo_address, so the walker never runs.
        for c in &r.classes {
            assert!(c.bases.is_empty());
        }
    }

    #[test]
    fn base_recovery_is_deterministic_across_reruns() {
        // Same model, two runs, identical base lists in identical
        // order. The walker iterates via `BTreeMap` so the order is
        // byte-stable (NFR-9).
        let build = || {
            let mut g = EvidenceGraph::new();
            let mut model = empty_model();
            model.symbols.push(text_symbol("_ZN3Dog5speakEv", 0x1000));
            model.symbols.push(text_symbol("_ZN3Cat5speakEv", 0x1100));
            model
                .symbols
                .push(data_symbol_with_size("_ZTI6Animal", 0x3d70, 16));
            model
                .symbols
                .push(data_symbol_with_size("_ZTI3Dog", 0x3d80, 24));
            model
                .symbols
                .push(data_symbol_with_size("_ZTI3Cat", 0x3d98, 24));
            model.relocations.push(relative_relocation(0x3d90, 0x3d70));
            model.relocations.push(relative_relocation(0x3da8, 0x3d70));
            let fs = function_set_with(&[0x1000, 0x1100], &mut g);
            recover_classes(&model, &fs, &mut g)
                .classes
                .into_iter()
                .map(|c| {
                    (
                        c.name,
                        c.bases
                            .into_iter()
                            .map(|b| b.qualified_name)
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>()
        };
        assert_eq!(build(), build());
    }
}

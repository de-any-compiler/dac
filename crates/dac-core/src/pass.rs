//! Pass manager skeleton (ARCHITECTURE §6).
//!
//! A *pass* declares the artifact kinds it reads, the kinds it produces,
//! and a [`Determinism`] class. The [`PassManager`] topologically schedules
//! a registered set of passes, runs them in order, and caches outputs in
//! the [`ArtifactCache`] keyed by `(pass_id, input_hash, settings_hash)`.
//!
//! Status: B0.4 is the skeleton — single-threaded execution, in-memory
//! cache, opaque `Vec<u8>` payloads. Parallel scheduling (NFR-7) and the
//! on-disk artifact format land later.

use std::collections::{BTreeMap, HashMap};

use dac_artifact::ArtifactCache;

use crate::{Error, Result};

/// A pass's identity. Stable string used in cache keys, tracing spans,
/// and the reproducibility manifest.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct PassId(&'static str);

impl PassId {
    /// Construct from a static name.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// Borrow the name as a string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for PassId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

/// Identifier for an artifact in the pipeline (a binary model, a CFG, a
/// type-propagation result, …).
///
/// Open by design: a string newtype so test pipelines and out-of-tree
/// passes can declare their own kinds without touching this crate.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct ArtifactKind(&'static str);

impl ArtifactKind {
    /// Construct from a static name.
    #[must_use]
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    /// Borrow the name as a string slice.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        self.0
    }
}

impl std::fmt::Display for ArtifactKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.0)
    }
}

/// Determinism class for a pass (NFR-9, ARCHITECTURE §6).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Determinism {
    /// Same inputs → same output bytes, always.
    Pure,
    /// Same inputs *and same seed* → same output bytes. The seed is
    /// recorded in the manifest.
    SeededPure,
    /// Output depends on inputs the pipeline does not control (wall
    /// clock, remote API, …). Rejected when `--deterministic` is on.
    NonDeterministic,
}

/// In-flight state given to a pass when it runs.
///
/// Provides read access to the artifacts the pass declared as inputs and
/// a sink for the outputs it declared. Anything the pass writes to an
/// artifact kind it did not declare is silently dropped (a future batch
/// will harden this; B0.4 keeps the trusted-pass model from the
/// architecture doc).
pub struct PassContext<'a> {
    inputs: &'a HashMap<ArtifactKind, Vec<u8>>,
    declared_outputs: &'a [ArtifactKind],
    produced: HashMap<ArtifactKind, Vec<u8>>,
}

impl<'a> PassContext<'a> {
    /// Read the bytes of a declared input. `None` if `kind` was not
    /// declared as an input on this pass.
    #[must_use]
    pub fn input(&self, kind: ArtifactKind) -> Option<&[u8]> {
        self.inputs.get(&kind).map(Vec::as_slice)
    }

    /// Publish bytes for a declared output. Writes to undeclared kinds
    /// are dropped.
    pub fn produce(&mut self, kind: ArtifactKind, bytes: Vec<u8>) {
        if self.declared_outputs.contains(&kind) {
            self.produced.insert(kind, bytes);
        }
    }
}

/// A pass in the dac pipeline.
pub trait Pass: Send + Sync {
    /// Stable identifier — appears in cache keys, tracing spans, and the
    /// reproducibility manifest.
    fn id(&self) -> PassId;

    /// Artifact kinds the pass reads.
    fn inputs(&self) -> &[ArtifactKind];

    /// Artifact kinds the pass produces.
    fn outputs(&self) -> &[ArtifactKind];

    /// Determinism class (NFR-9).
    fn determinism(&self) -> Determinism;

    /// Execute the pass.
    fn run(&self, ctx: &mut PassContext<'_>) -> Result<()>;
}

/// Per-pass outcome from a [`PassManager::run`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PassOutcome {
    /// Pass executed; outputs were freshly produced and stored in the
    /// cache.
    Ran,
    /// All outputs were restored from the cache; the pass's `run` was
    /// not called.
    CacheHit,
}

/// Summary of a pipeline execution.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RunReport {
    /// `(pass_id, outcome)` in execution order.
    pub passes: Vec<(PassId, PassOutcome)>,
}

impl RunReport {
    /// `true` iff every recorded pass came from the cache.
    #[must_use]
    pub fn fully_cached(&self) -> bool {
        !self.passes.is_empty() && self.passes.iter().all(|(_, o)| *o == PassOutcome::CacheHit)
    }

    /// Count of passes that executed (cache misses).
    #[must_use]
    pub fn executed(&self) -> usize {
        self.passes
            .iter()
            .filter(|(_, o)| *o == PassOutcome::Ran)
            .count()
    }

    /// Count of passes whose outputs were restored from the cache.
    #[must_use]
    pub fn cached(&self) -> usize {
        self.passes
            .iter()
            .filter(|(_, o)| *o == PassOutcome::CacheHit)
            .count()
    }
}

/// Set of artifacts flowing through a pipeline run.
///
/// Conceptually `HashMap<ArtifactKind, Vec<u8>>` with a thin façade so
/// callers don't need to import `std::collections`.
#[derive(Debug, Default)]
pub struct ArtifactStore {
    map: HashMap<ArtifactKind, Vec<u8>>,
}

impl ArtifactStore {
    /// An empty store.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Read by kind.
    #[must_use]
    pub fn get(&self, kind: ArtifactKind) -> Option<&[u8]> {
        self.map.get(&kind).map(Vec::as_slice)
    }

    /// Write by kind, overwriting any prior value.
    pub fn put(&mut self, kind: ArtifactKind, bytes: Vec<u8>) {
        self.map.insert(kind, bytes);
    }

    /// Number of stored artifacts.
    #[must_use]
    pub fn len(&self) -> usize {
        self.map.len()
    }

    /// `true` when the store is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.map.is_empty()
    }
}

/// The pass manager.
///
/// Owns the registered passes plus a single `deterministic` switch
/// driven by `--deterministic` (NFR-9). Passes are scheduled
/// topologically by the artifact-kind producer/consumer graph; cycles
/// and missing/duplicate producers are surfaced as
/// [`Error::PassManager`].
pub struct PassManager {
    passes: Vec<Box<dyn Pass>>,
    deterministic: bool,
    settings_hash: u64,
}

impl PassManager {
    /// New manager in non-deterministic mode (the default —
    /// `NonDeterministic` passes are allowed).
    #[must_use]
    pub fn new() -> Self {
        Self {
            passes: Vec::new(),
            deterministic: false,
            settings_hash: 0,
        }
    }

    /// New manager in deterministic mode. Registering a
    /// `NonDeterministic` pass on this manager returns an
    /// [`Error::PassManager`] (NFR-9).
    #[must_use]
    pub fn deterministic() -> Self {
        Self {
            passes: Vec::new(),
            deterministic: true,
            settings_hash: 0,
        }
    }

    /// Replace the settings hash mixed into every cache key. Two runs
    /// with different settings share no cache entries — this is how
    /// `-O1` and `-O2` outputs stay separate without colliding.
    #[must_use]
    pub fn with_settings_hash(mut self, hash: u64) -> Self {
        self.settings_hash = hash;
        self
    }

    /// `true` if this manager was constructed in deterministic mode.
    #[must_use]
    pub fn is_deterministic(&self) -> bool {
        self.deterministic
    }

    /// Register a pass. Rejects [`Determinism::NonDeterministic`] passes
    /// when the manager is in deterministic mode (NFR-9).
    pub fn register(&mut self, pass: Box<dyn Pass>) -> Result<()> {
        if self.deterministic && pass.determinism() == Determinism::NonDeterministic {
            return Err(Error::PassManager(format!(
                "pass `{}` is non-deterministic, rejected under --deterministic",
                pass.id()
            )));
        }
        self.passes.push(pass);
        Ok(())
    }

    /// Topologically order the registered passes and run them once,
    /// caching outputs and short-circuiting from the cache on re-runs
    /// with matching `(pass_id, input_hash, settings_hash)` keys.
    ///
    /// The store starts empty in B0.4; pre-loaded "external" inputs
    /// (e.g. the binary bytes) plug in here in B0.5 when the CLI begins
    /// driving real pipelines.
    pub fn run(&self, store: &mut ArtifactStore, cache: &mut ArtifactCache) -> Result<RunReport> {
        let order = self.schedule()?;
        let mut report = RunReport::default();

        for &idx in &order {
            let pass = &self.passes[idx];
            let id = pass.id();

            let inputs = collect_inputs(store, pass.inputs())?;
            let input_hash = hash_inputs(pass.inputs(), &inputs);
            let cache_key = build_cache_key(id, input_hash, self.settings_hash);

            if let Some(blob) = cache.get(&cache_key) {
                let restored = decode_outputs(blob)?;
                for (kind, bytes) in restored {
                    store.put(kind, bytes);
                }
                report.passes.push((id, PassOutcome::CacheHit));
                tracing::debug!(pass = %id, "pass cache hit");
                continue;
            }

            let mut ctx = PassContext {
                inputs: &inputs,
                declared_outputs: pass.outputs(),
                produced: HashMap::new(),
            };
            pass.run(&mut ctx)?;
            let produced = ctx.produced;

            let blob = encode_outputs(pass.outputs(), &produced)?;
            cache.put(cache_key, blob);

            for (kind, bytes) in produced {
                store.put(kind, bytes);
            }

            report.passes.push((id, PassOutcome::Ran));
            tracing::debug!(pass = %id, "pass executed");
        }

        Ok(report)
    }

    /// Kahn's algorithm over the producer/consumer graph implied by
    /// each pass's declared inputs and outputs. Returns indices into
    /// `self.passes` in execution order, or an [`Error::PassManager`]
    /// describing the rejection.
    fn schedule(&self) -> Result<Vec<usize>> {
        let n = self.passes.len();
        let mut producer: HashMap<ArtifactKind, usize> = HashMap::new();
        for (idx, pass) in self.passes.iter().enumerate() {
            for &kind in pass.outputs() {
                if let Some(prev) = producer.insert(kind, idx) {
                    return Err(Error::PassManager(format!(
                        "passes `{}` and `{}` both produce `{}`",
                        self.passes[prev].id(),
                        pass.id(),
                        kind,
                    )));
                }
            }
        }

        let mut in_deg = vec![0usize; n];
        let mut consumers: HashMap<ArtifactKind, Vec<usize>> = HashMap::new();
        for (idx, pass) in self.passes.iter().enumerate() {
            for &kind in pass.inputs() {
                let Some(&prod) = producer.get(&kind) else {
                    return Err(Error::PassManager(format!(
                        "pass `{}` consumes `{}` but no registered pass produces it",
                        pass.id(),
                        kind,
                    )));
                };
                if prod == idx {
                    return Err(Error::PassManager(format!(
                        "pass `{}` lists `{}` as both input and output",
                        pass.id(),
                        kind,
                    )));
                }
                in_deg[idx] += 1;
                consumers.entry(kind).or_default().push(idx);
            }
        }

        // Iterate roots in pass-registration order so the schedule is
        // deterministic across runs.
        let mut order = Vec::with_capacity(n);
        let mut ready: Vec<usize> = (0..n).filter(|&i| in_deg[i] == 0).collect();
        while let Some(idx) = pop_smallest(&mut ready) {
            order.push(idx);
            for &kind in self.passes[idx].outputs() {
                if let Some(cs) = consumers.get(&kind) {
                    for &c in cs {
                        in_deg[c] -= 1;
                        if in_deg[c] == 0 {
                            ready.push(c);
                        }
                    }
                }
            }
        }

        if order.len() != n {
            let cycle: Vec<&str> = (0..n)
                .filter(|i| !order.contains(i))
                .map(|i| self.passes[i].id().as_str())
                .collect();
            return Err(Error::PassManager(format!(
                "pipeline cycle through: {}",
                cycle.join(", ")
            )));
        }

        Ok(order)
    }
}

impl Default for PassManager {
    fn default() -> Self {
        Self::new()
    }
}

fn pop_smallest(ready: &mut Vec<usize>) -> Option<usize> {
    let pos = ready
        .iter()
        .enumerate()
        .min_by_key(|(_, &i)| i)
        .map(|(p, _)| p)?;
    Some(ready.swap_remove(pos))
}

fn collect_inputs(
    store: &ArtifactStore,
    declared: &[ArtifactKind],
) -> Result<HashMap<ArtifactKind, Vec<u8>>> {
    let mut out = HashMap::with_capacity(declared.len());
    for &kind in declared {
        let bytes = store.get(kind).ok_or_else(|| {
            Error::PassManager(format!(
                "declared input `{kind}` missing from the artifact store",
            ))
        })?;
        out.insert(kind, bytes.to_vec());
    }
    Ok(out)
}

fn hash_inputs(declared: &[ArtifactKind], inputs: &HashMap<ArtifactKind, Vec<u8>>) -> u64 {
    // Hash in declared order so a permutation of the input list yields a
    // different key — passes that reorder their inputs are a different
    // computation.
    let mut h = fnv1a64_init();
    for &kind in declared {
        h = fnv1a64_update(h, kind.as_str().as_bytes());
        h = fnv1a64_update(h, &[0]);
        if let Some(bytes) = inputs.get(&kind) {
            h = fnv1a64_update(h, bytes);
        }
        h = fnv1a64_update(h, &[0]);
    }
    h
}

fn build_cache_key(pass_id: PassId, input_hash: u64, settings_hash: u64) -> Vec<u8> {
    let name = pass_id.as_str().as_bytes();
    let mut key = Vec::with_capacity(name.len() + 1 + 8 + 8);
    key.extend_from_slice(name);
    key.push(0);
    key.extend_from_slice(&input_hash.to_le_bytes());
    key.extend_from_slice(&settings_hash.to_le_bytes());
    key
}

// Length-prefixed `(kind, bytes)*` encoding. Kept inline because the
// payload format is internal to the cache; nothing else reads it.
fn encode_outputs(
    declared: &[ArtifactKind],
    produced: &HashMap<ArtifactKind, Vec<u8>>,
) -> Result<Vec<u8>> {
    // Serialize in declared order so the encoded blob is canonical.
    let mut ordered: BTreeMap<ArtifactKind, &[u8]> = BTreeMap::new();
    for &kind in declared {
        if let Some(bytes) = produced.get(&kind) {
            ordered.insert(kind, bytes.as_slice());
        }
    }

    let mut out = Vec::new();
    let count = u32::try_from(ordered.len()).map_err(|_| {
        Error::PassManager("pass produced more outputs than u32 can address".into())
    })?;
    out.extend_from_slice(&count.to_le_bytes());
    for (kind, bytes) in ordered {
        let name = kind.as_str().as_bytes();
        let name_len = u32::try_from(name.len())
            .map_err(|_| Error::PassManager("artifact kind name exceeds u32 length".into()))?;
        let value_len = u32::try_from(bytes.len())
            .map_err(|_| Error::PassManager("artifact payload exceeds u32 length".into()))?;
        out.extend_from_slice(&name_len.to_le_bytes());
        out.extend_from_slice(name);
        out.extend_from_slice(&value_len.to_le_bytes());
        out.extend_from_slice(bytes);
    }
    Ok(out)
}

fn decode_outputs(blob: &[u8]) -> Result<Vec<(ArtifactKind, Vec<u8>)>> {
    let mut cursor = 0usize;
    let count = read_u32(blob, &mut cursor)? as usize;
    let mut out = Vec::with_capacity(count);
    for _ in 0..count {
        let name_len = read_u32(blob, &mut cursor)? as usize;
        let name = read_bytes(blob, &mut cursor, name_len)?;
        let kind_str = std::str::from_utf8(name).map_err(|_| {
            Error::PassManager("corrupt cache entry: non-utf8 artifact kind".into())
        })?;
        // We need a `&'static str` to live inside `ArtifactKind`. The
        // pass that produced this entry necessarily declared the kind
        // via a static name; we leak the decoded string at cache-hit
        // time to recover that lifetime. Leaks are bounded by the set
        // of artifact kinds the binary ever declares.
        let leaked: &'static str = Box::leak(kind_str.to_owned().into_boxed_str());
        let value_len = read_u32(blob, &mut cursor)? as usize;
        let value = read_bytes(blob, &mut cursor, value_len)?.to_vec();
        out.push((ArtifactKind::new(leaked), value));
    }
    Ok(out)
}

fn read_u32(buf: &[u8], cursor: &mut usize) -> Result<u32> {
    let end = cursor
        .checked_add(4)
        .ok_or_else(|| Error::PassManager("corrupt cache entry: length overflow".into()))?;
    let slice = buf
        .get(*cursor..end)
        .ok_or_else(|| Error::PassManager("corrupt cache entry: truncated".into()))?;
    let arr: [u8; 4] = slice
        .try_into()
        .map_err(|_| Error::PassManager("corrupt cache entry: bad u32 width".into()))?;
    *cursor = end;
    Ok(u32::from_le_bytes(arr))
}

fn read_bytes<'a>(buf: &'a [u8], cursor: &mut usize, len: usize) -> Result<&'a [u8]> {
    let end = cursor
        .checked_add(len)
        .ok_or_else(|| Error::PassManager("corrupt cache entry: length overflow".into()))?;
    let slice = buf
        .get(*cursor..end)
        .ok_or_else(|| Error::PassManager("corrupt cache entry: truncated".into()))?;
    *cursor = end;
    Ok(slice)
}

const FNV1A64_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV1A64_PRIME: u64 = 0x0000_0100_0000_01b3;

fn fnv1a64_init() -> u64 {
    FNV1A64_OFFSET
}

fn fnv1a64_update(mut hash: u64, bytes: &[u8]) -> u64 {
    for b in bytes {
        hash ^= u64::from(*b);
        hash = hash.wrapping_mul(FNV1A64_PRIME);
    }
    hash
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::sync::Arc;

    const ALPHA_OUT: ArtifactKind = ArtifactKind::new("alpha-out");
    const BETA_OUT: ArtifactKind = ArtifactKind::new("beta-out");
    const GAMMA_OUT: ArtifactKind = ArtifactKind::new("gamma-out");

    struct CountedPass {
        id: PassId,
        inputs: &'static [ArtifactKind],
        outputs: &'static [ArtifactKind],
        determinism: Determinism,
        counter: Arc<AtomicUsize>,
        body: fn(&mut PassContext<'_>) -> Result<()>,
    }

    impl Pass for CountedPass {
        fn id(&self) -> PassId {
            self.id
        }

        fn inputs(&self) -> &[ArtifactKind] {
            self.inputs
        }

        fn outputs(&self) -> &[ArtifactKind] {
            self.outputs
        }

        fn determinism(&self) -> Determinism {
            self.determinism
        }

        fn run(&self, ctx: &mut PassContext<'_>) -> Result<()> {
            self.counter.fetch_add(1, Ordering::Relaxed);
            (self.body)(ctx)
        }
    }

    fn alpha_body(ctx: &mut PassContext<'_>) -> Result<()> {
        ctx.produce(ALPHA_OUT, b"alpha".to_vec());
        Ok(())
    }

    fn beta_body(ctx: &mut PassContext<'_>) -> Result<()> {
        let a = ctx.input(ALPHA_OUT).unwrap_or_default();
        let mut out = a.to_vec();
        out.extend_from_slice(b"-beta");
        ctx.produce(BETA_OUT, out);
        Ok(())
    }

    fn gamma_body(ctx: &mut PassContext<'_>) -> Result<()> {
        let b = ctx.input(BETA_OUT).unwrap_or_default();
        let mut out = b.to_vec();
        out.extend_from_slice(b"-gamma");
        ctx.produce(GAMMA_OUT, out);
        Ok(())
    }

    fn three_pass_pipeline(counter: Arc<AtomicUsize>) -> PassManager {
        let mut mgr = PassManager::new();
        mgr.register(Box::new(CountedPass {
            id: PassId::new("alpha"),
            inputs: &[],
            outputs: &[ALPHA_OUT],
            determinism: Determinism::Pure,
            counter: counter.clone(),
            body: alpha_body,
        }))
        .expect("register alpha");
        mgr.register(Box::new(CountedPass {
            id: PassId::new("beta"),
            inputs: &[ALPHA_OUT],
            outputs: &[BETA_OUT],
            determinism: Determinism::Pure,
            counter: counter.clone(),
            body: beta_body,
        }))
        .expect("register beta");
        mgr.register(Box::new(CountedPass {
            id: PassId::new("gamma"),
            inputs: &[BETA_OUT],
            outputs: &[GAMMA_OUT],
            determinism: Determinism::Pure,
            counter,
            body: gamma_body,
        }))
        .expect("register gamma");
        mgr
    }

    #[test]
    fn three_pass_pipeline_runs_and_caches() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mgr = three_pass_pipeline(counter.clone());
        let mut cache = ArtifactCache::new();
        let mut store = ArtifactStore::new();

        let r1 = mgr.run(&mut store, &mut cache).expect("first run");
        assert_eq!(r1.executed(), 3);
        assert_eq!(r1.cached(), 0);
        assert_eq!(counter.load(Ordering::Relaxed), 3);
        assert_eq!(store.get(ALPHA_OUT), Some(b"alpha".as_slice()));
        assert_eq!(store.get(BETA_OUT), Some(b"alpha-beta".as_slice()));
        assert_eq!(store.get(GAMMA_OUT), Some(b"alpha-beta-gamma".as_slice()));

        // Fresh store, same cache — every pass should now hit.
        let mut store2 = ArtifactStore::new();
        let r2 = mgr.run(&mut store2, &mut cache).expect("second run");
        assert_eq!(r2.executed(), 0);
        assert_eq!(r2.cached(), 3);
        assert!(r2.fully_cached());
        assert_eq!(counter.load(Ordering::Relaxed), 3, "no pass re-executed");
        assert_eq!(store2.get(ALPHA_OUT), Some(b"alpha".as_slice()));
        assert_eq!(store2.get(BETA_OUT), Some(b"alpha-beta".as_slice()));
        assert_eq!(store2.get(GAMMA_OUT), Some(b"alpha-beta-gamma".as_slice()));
    }

    #[test]
    fn cycle_is_rejected() {
        let mut mgr = PassManager::new();
        struct Cyc {
            id: PassId,
            ins: &'static [ArtifactKind],
            outs: &'static [ArtifactKind],
        }
        impl Pass for Cyc {
            fn id(&self) -> PassId {
                self.id
            }
            fn inputs(&self) -> &[ArtifactKind] {
                self.ins
            }
            fn outputs(&self) -> &[ArtifactKind] {
                self.outs
            }
            fn determinism(&self) -> Determinism {
                Determinism::Pure
            }
            fn run(&self, _: &mut PassContext<'_>) -> Result<()> {
                Ok(())
            }
        }
        const A: ArtifactKind = ArtifactKind::new("cyc-a");
        const B: ArtifactKind = ArtifactKind::new("cyc-b");
        mgr.register(Box::new(Cyc {
            id: PassId::new("p1"),
            ins: &[B],
            outs: &[A],
        }))
        .expect("p1");
        mgr.register(Box::new(Cyc {
            id: PassId::new("p2"),
            ins: &[A],
            outs: &[B],
        }))
        .expect("p2");

        let mut store = ArtifactStore::new();
        let mut cache = ArtifactCache::new();
        let err = mgr
            .run(&mut store, &mut cache)
            .expect_err("cycle must be rejected");
        let msg = format!("{err}");
        assert!(msg.contains("cycle"), "got: {msg}");
    }

    #[test]
    fn missing_producer_is_rejected() {
        struct P;
        const X: ArtifactKind = ArtifactKind::new("orphan");
        impl Pass for P {
            fn id(&self) -> PassId {
                PassId::new("p")
            }
            fn inputs(&self) -> &[ArtifactKind] {
                &[X]
            }
            fn outputs(&self) -> &[ArtifactKind] {
                &[]
            }
            fn determinism(&self) -> Determinism {
                Determinism::Pure
            }
            fn run(&self, _: &mut PassContext<'_>) -> Result<()> {
                Ok(())
            }
        }
        let mut mgr = PassManager::new();
        mgr.register(Box::new(P)).expect("register");
        let err = mgr
            .run(&mut ArtifactStore::new(), &mut ArtifactCache::new())
            .expect_err("missing producer");
        assert!(format!("{err}").contains("no registered pass produces"));
    }

    #[test]
    fn duplicate_producer_is_rejected() {
        struct P {
            id: PassId,
        }
        const X: ArtifactKind = ArtifactKind::new("dup");
        impl Pass for P {
            fn id(&self) -> PassId {
                self.id
            }
            fn inputs(&self) -> &[ArtifactKind] {
                &[]
            }
            fn outputs(&self) -> &[ArtifactKind] {
                &[X]
            }
            fn determinism(&self) -> Determinism {
                Determinism::Pure
            }
            fn run(&self, _: &mut PassContext<'_>) -> Result<()> {
                Ok(())
            }
        }
        let mut mgr = PassManager::new();
        mgr.register(Box::new(P {
            id: PassId::new("p1"),
        }))
        .expect("p1");
        mgr.register(Box::new(P {
            id: PassId::new("p2"),
        }))
        .expect("p2");
        let err = mgr
            .run(&mut ArtifactStore::new(), &mut ArtifactCache::new())
            .expect_err("dup producer");
        assert!(format!("{err}").contains("both produce"));
    }

    #[test]
    fn deterministic_manager_rejects_nondeterministic_pass() {
        struct Nd;
        impl Pass for Nd {
            fn id(&self) -> PassId {
                PassId::new("nd")
            }
            fn inputs(&self) -> &[ArtifactKind] {
                &[]
            }
            fn outputs(&self) -> &[ArtifactKind] {
                &[]
            }
            fn determinism(&self) -> Determinism {
                Determinism::NonDeterministic
            }
            fn run(&self, _: &mut PassContext<'_>) -> Result<()> {
                Ok(())
            }
        }
        let mut mgr = PassManager::deterministic();
        assert!(mgr.is_deterministic());
        let err = mgr.register(Box::new(Nd)).expect_err("must reject");
        assert!(format!("{err}").contains("non-deterministic"));
    }

    #[test]
    fn nondeterministic_pass_accepted_in_default_mode() {
        const ND_OUT: ArtifactKind = ArtifactKind::new("nd-out");
        const ND_OUTPUTS: &[ArtifactKind] = &[ND_OUT];
        struct Nd;
        impl Pass for Nd {
            fn id(&self) -> PassId {
                PassId::new("nd")
            }
            fn inputs(&self) -> &[ArtifactKind] {
                &[]
            }
            fn outputs(&self) -> &[ArtifactKind] {
                ND_OUTPUTS
            }
            fn determinism(&self) -> Determinism {
                Determinism::NonDeterministic
            }
            fn run(&self, ctx: &mut PassContext<'_>) -> Result<()> {
                ctx.produce(ND_OUT, b"x".to_vec());
                Ok(())
            }
        }
        let mut mgr = PassManager::new();
        assert!(!mgr.is_deterministic());
        mgr.register(Box::new(Nd)).expect("default mode accepts");
    }

    #[test]
    fn settings_hash_partitions_cache() {
        let counter = Arc::new(AtomicUsize::new(0));
        let mgr_a = three_pass_pipeline(counter.clone());
        let mgr_b_no_settings = three_pass_pipeline(counter.clone());
        let mgr_b = three_pass_pipeline(counter.clone()).with_settings_hash(0xdead_beef);

        let mut cache = ArtifactCache::new();

        let mut store_a = ArtifactStore::new();
        mgr_a
            .run(&mut store_a, &mut cache)
            .expect("first run with default settings");
        assert_eq!(counter.load(Ordering::Relaxed), 3);

        // Same settings → cache hits, counter stays at 3.
        let mut store_a2 = ArtifactStore::new();
        let r = mgr_b_no_settings
            .run(&mut store_a2, &mut cache)
            .expect("rerun same settings");
        assert!(r.fully_cached());
        assert_eq!(counter.load(Ordering::Relaxed), 3);

        // Different settings hash → cache miss for every pass.
        let mut store_b = ArtifactStore::new();
        let r = mgr_b
            .run(&mut store_b, &mut cache)
            .expect("rerun different settings");
        assert_eq!(r.executed(), 3);
        assert_eq!(counter.load(Ordering::Relaxed), 6);
    }

    #[test]
    fn schedule_is_deterministic_under_input_ordering() {
        // Register in reverse declaration order; the schedule should
        // still respect data dependencies and the registration tiebreak.
        let counter = Arc::new(AtomicUsize::new(0));
        let mut mgr = PassManager::new();
        mgr.register(Box::new(CountedPass {
            id: PassId::new("gamma"),
            inputs: &[BETA_OUT],
            outputs: &[GAMMA_OUT],
            determinism: Determinism::Pure,
            counter: counter.clone(),
            body: gamma_body,
        }))
        .expect("gamma");
        mgr.register(Box::new(CountedPass {
            id: PassId::new("beta"),
            inputs: &[ALPHA_OUT],
            outputs: &[BETA_OUT],
            determinism: Determinism::Pure,
            counter: counter.clone(),
            body: beta_body,
        }))
        .expect("beta");
        mgr.register(Box::new(CountedPass {
            id: PassId::new("alpha"),
            inputs: &[],
            outputs: &[ALPHA_OUT],
            determinism: Determinism::Pure,
            counter,
            body: alpha_body,
        }))
        .expect("alpha");

        let mut cache = ArtifactCache::new();
        let mut store = ArtifactStore::new();
        let report = mgr.run(&mut store, &mut cache).expect("run");
        let ids: Vec<_> = report.passes.iter().map(|(p, _)| p.as_str()).collect();
        assert_eq!(ids, vec!["alpha", "beta", "gamma"]);
    }
}

//! `dac-artifact` — on-disk artifact format and content-addressed pass
//! cache for dac.
//!
//! Part of the dac workspace. See `ARCHITECTURE.md` §10–§11 in the
//! workspace root.
//!
//! Status: B0.4 ships an in-memory content-addressed cache used by the
//! pass manager. The on-disk format and cross-process cache lands later
//! (the on-disk format is part of M1; cross-process caching is part of
//! M5).

#![forbid(unsafe_code)]

use std::collections::HashMap;

/// Content-addressed cache from opaque key bytes to opaque value bytes.
///
/// The pass manager owns key construction (`pass_id || input_hash ||
/// settings_hash`); this crate just stores and retrieves the payload.
/// In-memory only at B0.4 — persistence and on-disk format land with
/// later batches.
#[derive(Debug, Default)]
pub struct ArtifactCache {
    entries: HashMap<Vec<u8>, Vec<u8>>,
}

impl ArtifactCache {
    /// An empty cache.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Look up a cached payload by key.
    #[must_use]
    pub fn get(&self, key: &[u8]) -> Option<&[u8]> {
        self.entries.get(key).map(Vec::as_slice)
    }

    /// Insert (or overwrite) a payload by key.
    pub fn put(&mut self, key: Vec<u8>, value: Vec<u8>) {
        self.entries.insert(key, value);
    }

    /// Number of cached entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// `true` when the cache holds nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn put_then_get_roundtrips() {
        let mut c = ArtifactCache::new();
        c.put(b"k".to_vec(), b"v".to_vec());
        assert_eq!(c.get(b"k"), Some(b"v".as_slice()));
        assert_eq!(c.len(), 1);
        assert!(!c.is_empty());
    }

    #[test]
    fn miss_returns_none() {
        let c = ArtifactCache::new();
        assert!(c.get(b"absent").is_none());
        assert!(c.is_empty());
    }

    #[test]
    fn put_overwrites_existing_value() {
        let mut c = ArtifactCache::new();
        c.put(b"k".to_vec(), b"v1".to_vec());
        c.put(b"k".to_vec(), b"v2".to_vec());
        assert_eq!(c.get(b"k"), Some(b"v2".as_slice()));
        assert_eq!(c.len(), 1);
    }
}

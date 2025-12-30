use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::sync::Arc;

use dashmap::DashMap;
use lazy_static::lazy_static;

lazy_static! {
    static ref FX_IDENT_TO_HASH: Arc<DashMap<String, u64>> = Arc::new(DashMap::new());
    static ref HASH_TO_FX_IDENT: Arc<DashMap<u64, String>> = Arc::new(DashMap::new());
}

// Function to compute a hash for a given string
fn compute_hash(name: &str) -> u64 {
    let mut hasher = DefaultHasher::new();
    name.hash(&mut hasher);
    hasher.finish()
}

/// Registers an fx_ident and returns its corresponding hash.
pub fn hash_fx_ident(name: &str) -> u64 {
    if let Some(existing_hash) = FX_IDENT_TO_HASH.get(name) {
        *existing_hash // Return the existing hash
    } else {
        // Compute a new hash and insert it into both maps
        let new_hash = compute_hash(name);

        // Ensure bidirectional consistency
        FX_IDENT_TO_HASH.insert(name.to_string(), new_hash);
        HASH_TO_FX_IDENT.insert(new_hash, name.to_string());

        new_hash
    }
}

/// Looks up an fx_ident by its hash.
pub fn get_fx_ident_by_hash(hash: u64) -> Option<String> {
    HASH_TO_FX_IDENT.get(&hash).map(|entry| entry.clone())
}

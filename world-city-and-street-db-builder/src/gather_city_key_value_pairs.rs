// ---------------- [ File: src/gather_city_key_value_pairs.rs ]
crate::ix!();

/// Performs a prefix-based iteration in RocksDB to find all city keys matching the prefix.
/// Returns a vector of `(key_string, value_bytes)` tuples for further processing.
pub fn gather_city_key_value_pairs(db: &Database, prefix: &str) -> Vec<(String, Vec<u8>)> {
    trace!(
        "gather_city_key_value_pairs: prefix='{}' => running prefix_iterator",
        prefix
    );

    let iter = db.db().prefix_iterator(prefix.as_bytes());
    let mut results = Vec::new();

    for item_result in iter {
        match item_result {
            Ok((key_bytes, val_bytes)) => {
                let key_str = String::from_utf8_lossy(&key_bytes).to_string();
                debug!(
                    "gather_city_key_value_pairs: found key='{}' (value: {} bytes)",
                    key_str,
                    val_bytes.len()
                );
                results.push((key_str, val_bytes.to_vec()));
            }
            Err(e) => {
                error!(
                    "gather_city_key_value_pairs: error reading from DB for prefix='{}': {}",
                    prefix, e
                );
            }
        }
    }

    results
}

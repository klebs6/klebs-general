// ---------------- [ File: src/load_all_cities_for_region.rs ]
crate::ix!();

/// A tiny helper to gather all known city names for a given region.
/// Internally, it searches RocksDB for keys with the prefix `C2Z:<abbr>:`
/// and extracts the city substring after the second colon. It also decodes
/// CBOR values to confirm they're valid, though we discard the parsed data
/// by default.
///
/// # Arguments
///
/// * `db`     - The database reference used for iteration.
/// * `region` - The region whose city names we want to gather.
///
/// # Returns
///
/// * A vector of city names (e.g., `["baltimore", "frederick", ...]`).
pub fn load_all_cities_for_region(db: &Database, region: &WorldRegion) -> Vec<String> {
    trace!("load_all_cities_for_region: start for region={:?}", region);

    let prefix = build_city_search_prefix(region);
    debug!(
        "load_all_cities_for_region: searching DB with prefix='{}'",
        prefix
    );

    // 1) Collect all (key, value) pairs matching "C2Z:<abbr>:".
    let kv_pairs = gather_city_key_value_pairs(db, &prefix);

    // 2) Parse city names from these pairs, optionally decoding CBOR to confirm validity.
    let all_cities = parse_city_names(kv_pairs);

    debug!(
        "load_all_cities_for_region: found {} cities for region={:?}",
        all_cities.len(),
        region
    );
    all_cities
}

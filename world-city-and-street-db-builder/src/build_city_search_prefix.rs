crate::ix!();

/// Constructs the RocksDB key prefix for city => postal code data.
pub fn build_city_search_prefix(region: &WorldRegion) -> String {
    trace!("build_city_search_prefix: building prefix for region={:?}", region);
    format!("C2Z:{}:", region.abbreviation())
}

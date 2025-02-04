crate::ix!();

/// Attempts to extract the city portion from a RocksDB key of the form:
/// `C2Z:<region_abbreviation>:<city>`.
///
/// Returns `Some(city_string)` if successful, or `None` if the key is malformed.
pub fn extract_city_from_key(key_str: &str) -> Option<String> {
    trace!("extract_city_from_key: analyzing key='{}'", key_str);

    // `splitn(3, ':')` -> e.g. ["C2Z", "US", "baltimore"]
    let parts: Vec<&str> = key_str.splitn(3, ':').collect();
    if parts.len() < 3 {
        warn!(
            "extract_city_from_key: key='{}' does not contain 3 parts; ignoring",
            key_str
        );
        return None;
    }
    Some(parts[2].to_owned())
}

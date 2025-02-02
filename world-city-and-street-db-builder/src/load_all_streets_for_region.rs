// ---------------- [ File: src/load_all_streets_for_region.rs ]
crate::ix!();

/// Similarly for street autocomplete. We gather all known street names for region
/// by scanning prefix "S2C:<abbr>:", parse out the “street” portion from the key.
pub fn load_all_streets_for_region(
    db: &Database,
    region: &WorldRegion,
) -> Vec<String> {
    let mut all_streets = Vec::new();
    let prefix = format!("S2C:{}:", region.abbreviation());

    let iter = db.db().prefix_iterator(prefix.as_bytes());
    for item in iter {
        if let Ok((key_bytes, val_bytes)) = item {
            let key_str = String::from_utf8_lossy(&key_bytes).to_string();
            // e.g. "S2C:US:north avenue"
            // splitn(3, ':') => ["S2C", "US", "north avenue"]
            let parts: Vec<&str> = key_str.splitn(3, ':').collect();
            if parts.len() < 3 {
                continue;
            }
            let raw_street = parts[2].to_owned();
            // Optionally parse or ignore val_bytes
            all_streets.push(raw_street);
        }
    }

    all_streets
}

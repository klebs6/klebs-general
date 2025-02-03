// ---------------- [ File: src/load_done_regions.rs ]
crate::ix!();

/// (5) Helper function to load the set of “done” regions by scanning for `META:REGION_DONE:<abbrev>`.
pub fn load_done_regions(db: &Database) -> Vec<WorldRegion> {
    let prefix = b"META:REGION_DONE:";
    let mut out = Vec::new();

    let it = db.db().prefix_iterator(prefix);
    for kv in it {
        if let Ok((k, _v)) = kv {
            let key_str = String::from_utf8_lossy(&k).to_string();
            // key_str might be "META:REGION_DONE:US" or "META:REGION_DONE:MD", etc.
            // We parse after the 2nd colon
            if let Some(abbr) = key_str.splitn(3, ':').nth(2) {
                // Attempt to convert abbreviation -> WorldRegion
                // If your code can do `WorldRegion::from_abbreviation(abbr)`, do so.
                // If not, you might store it in your DB or do a custom match. 
                // Here’s a pseudo approach:
                match WorldRegion::try_from_abbreviation(abbr) {
                    Ok(r) => out.push(r),
                    Err(e) => {
                        eprintln!("Could not parse region from abbr '{}': {:?}", abbr, e);
                    }
                }
            }
        }
    }
    out
}

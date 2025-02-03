// ---------------- [ File: src/parse_osm_pbf_and_build_house_number_ranges.rs ]
crate::ix!();

/// Reads the entire PBF file in one pass:
///   - Builds a `Vec<AddressRecord>` for city/street/postcode
///   - Accumulates all house‐number ranges for each street in memory
///   - Merges them with existing DB data once at the end.
/// 
/// **Fixed** so that we handle Node/Way/Relation/DenseNode properly
/// instead of calling `element.tags()` directly on the enum.
pub fn parse_osm_pbf_and_build_house_number_ranges(
    db: &mut Database,
    path: impl AsRef<Path>,
    region: WorldRegion,
) -> Result<Vec<AddressRecord>, DatabaseConstructionError>
{
    let country = Country::try_from(region)?;
    let reader = ElementReader::from_path(path)?;

    // aggregator: map StreetName -> list of subranges
    let mut aggregator: HashMap<StreetName, Vec<HouseNumberRange>> = HashMap::new();
    let mut records = Vec::new();
    let mut count = 0usize;

    // For each element in the PBF:
    reader.for_each(|element| {
        // (1) Attempt to parse an AddressRecord
        if let Ok(rec) = AddressRecord::try_from((&element, &country)) {
            records.push(rec.clone());
        }

        // (2) Attempt to extract a house-number range from this element
        match extract_house_number_range_from_element(&element) {
            Ok(Some(hnr)) => {
                // If the AddressRecord included a street, we can unify. 
                // But we only have a HouseNumberRange if there's at least
                // an `addr:housenumber`. Sometimes there's no `addr:street`.
                // So let's see if the record has a street.
                if let Ok(r) = AddressRecord::try_from((&element, &country)) {
                    if let Some(st) = r.street() {
                        aggregator.entry(st.clone()).or_default().push(hnr);
                    }
                }
            }
            _ => {}
        }

        count += 1;
        if count % 1000 == 0 {
            info!("Parsed {} elements so far...", count);
        }
    })?;

    // Finally, unify + store
    info!("Done reading entire PBF. HouseNumberRange aggregator has {} distinct streets.", aggregator.len());

    for (street, new_ranges) in aggregator {
        // load existing
        let existing_opt = load_house_number_ranges(db, &region, &street)?;
        let mut current = existing_opt.unwrap_or_default();

        // unify
        for rng in new_ranges {
            current = merge_house_number_range(current, rng);
        }

        // store
        store_house_number_ranges(db, &region, &street, &current)?;
    }

    Ok(records)
}

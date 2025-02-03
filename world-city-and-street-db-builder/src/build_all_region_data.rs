// ---------------- [ File: src/build_all_region_data.rs ]
crate::ix!();

/// (6) Build a map of (region => RegionData), by scanning city+street from the DB for each done region.
pub fn build_all_region_data(db: &Database, done_regions: &[WorldRegion]) -> HashMap<WorldRegion, RegionData> {
    let mut map = HashMap::new();
    for r in done_regions {
        let mut city_vec = load_all_cities_for_region(db, r);
        let mut street_vec = load_all_streets_for_region(db, r);

        // Sort them so fuzzy matching has a stable order, though not strictly required.
        city_vec.sort();
        street_vec.sort();

        let rd = RegionDataBuilder::default()
            .cities(city_vec)
            .streets(street_vec)
            .build()
            .unwrap();

        map.insert(*r, rd);
    }
    map
}

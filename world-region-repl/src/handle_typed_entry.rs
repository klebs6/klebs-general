// ---------------- [ File: src/handle_typed_entry.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip(st))]
pub fn handle_typed_entry<I: StorageInterface>(
    st: &ReplState<I>,
    typed: &str
) -> Vec<String> {
    use tracing::{trace, debug, warn};
    let mut display_data = Vec::new();

    let typed_lc = typed.to_lowercase();
    trace!("handle_typed_entry: typed='{}' => lower='{}'", typed, typed_lc);

    // Retrieve region data for the current region
    let reg_data = match st.regions().get(st.current_region()) {
        Some(d) => d,
        None => {
            warn!(
                "handle_typed_entry: No RegionData for current_region={:?}",
                st.current_region().abbreviation()
            );
            display_data.push(format!(
                "No RegionData for region {:?}",
                st.current_region().abbreviation()
            ));
            return display_data;
        }
    };

    // 1) Check if typed matches a known city in the region
    if reg_data.cities().iter().any(|c| c.eq_ignore_ascii_case(&typed_lc)) {
        let city_obj = match CityName::new(&typed_lc) {
            Ok(c) => c,
            Err(e) => {
                warn!(
                    "handle_typed_entry: CityName parse error for typed='{}': {:?}",
                    typed_lc, e
                );
                display_data.push(format!(
                    "Weird parse error for city='{}': {:?}",
                    typed_lc, e
                ));
                return display_data;
            }
        };

        display_data.push(format!(
            "'{}' is recognized as a city in region {}.",
            typed,
            st.current_region().abbreviation()
        ));

        // Show postal codes
        let c2z_k = c2z_key(st.current_region(), &city_obj);
        if let Some(postals) = st.db_access().get_postal_code_set(&c2z_k) {
            display_data.push(format!("Postal codes for city '{}':", typed_lc));
            for pc in &postals {
                display_data.push(format!("  {}", pc.code()));
            }
        }

        // ----------------------------------------
        // (A) Official city->streets from c2s_key
        // ----------------------------------------
        let c2s_k = c2s_key(st.current_region(), &city_obj);
        let official_streets = st
            .db_access()
            .get_street_set(&c2s_k)
            .unwrap_or_default();

        // Sort them for consistent ordering
        let mut official_sorted: Vec<_> = official_streets.into_iter().collect();
        official_sorted.sort_by(|a, b| a.name().cmp(b.name()));

        // Print "official" group
        display_data.push(format!(
            "Streets in city '{}' (official):",
            typed_lc
        ));
        if official_sorted.is_empty() {
            display_data.push(format!("  (none)"));
        } else {
            for sname in &official_sorted {
                display_data.push(format!("  {}", sname.name()));
            }
        }

        // ----------------------------------------
        // (B) For each ZIP in city_zips => gather s_key => streets => difference from official
        // ----------------------------------------
        let city_zips = st
            .db_access()
            .postal_codes_for_city_in_region(st.current_region(), &city_obj)
            .unwrap_or_default();

        // We'll iterate each zip in sorted order so it’s not random
        let mut city_zips_vec: Vec<_> = city_zips.into_iter().collect();
        city_zips_vec.sort_by(|a, b| a.code().cmp(b.code()));

        for zip in &city_zips_vec {
            let s_k = s_key(st.current_region(), zip);
            let zip_streets = st
                .db_access()
                .get_street_set(&s_k)
                .unwrap_or_default();

            // “inferred” means streets that are NOT already in the official set
            let inferred: BTreeSet<_> = zip_streets
                .difference(&official_sorted.iter().cloned().collect())
                .cloned()
                .collect();

            // If there's nothing in the inferred, skip or show (none)
            display_data.push(format!(
                "Streets in city '{}' (inferred from ZIP={}):",
                typed_lc, zip.code()
            ));
            if inferred.is_empty() {
                display_data.push(format!("  (none)"));
            } else {
                // Sort them
                let mut inferred_vec: Vec<_> = inferred.into_iter().collect();
                inferred_vec.sort_by(|a, b| a.name().cmp(b.name()));

                for sname in &inferred_vec {
                    display_data.push(format!("  {}", sname.name()));
                }
            }
        }

        return display_data;
    }

    // 2) Check if typed matches a known street in the region
    if reg_data.streets().iter().any(|s| s.eq_ignore_ascii_case(&typed_lc)) {
        let street_obj = match StreetName::new(&typed_lc) {
            Ok(s) => s,
            Err(e) => {
                warn!(
                    "handle_typed_entry: StreetName parse error for typed='{}': {:?}",
                    typed_lc, e
                );
                display_data.push(format!(
                    "Strange parse error for street='{}': {:?}",
                    typed_lc, e
                ));
                return display_data;
            }
        };

        display_data.push(format!(
            "'{}' is recognized as a street in region {}.",
            typed,
            st.current_region().abbreviation()
        ));

        // Which cities is it in? (s2c_key => city set)
        let s2c_k = s2c_key(st.current_region(), &street_obj);
        if let Some(city_set) = st.db_access().get_city_set(&s2c_k) {
            display_data.push(format!("Cities containing '{}':", typed_lc));
            for c in &city_set {
                display_data.push(format!("  {}", c.name()));
            }
        }

        // Which ZIP(s)? (s2z_key => zip set)
        let s2z_k = s2z_key(st.current_region(), &street_obj);
        if let Some(zip_set) = st.db_access().get_postal_code_set(&s2z_k) {
            display_data.push(format!("ZIP codes containing '{}':", typed_lc));
            for zc in &zip_set {
                display_data.push(format!("  {}", zc.code()));
            }
        }

        // House-number ranges?
        let abbr = st.current_region().abbreviation().to_string();
        let street_key = (abbr, typed_lc.clone());
        if let Some(ranges) = st.house_number_ranges().get(&street_key) {
            display_data.push(format!("House number ranges for '{}':", typed_lc));
            let lines = known_house_number_ranges_display(ranges);
            for line in lines {
                display_data.push(line);
            }
        } else {
            display_data.push(format!(
                "No known house-number range data for '{}'.",
                typed_lc
            ));
        }
        return display_data;
    }

    // 3) If neither city nor street matched => show fuzzy suggestions
    display_data.push(format!(
        "You typed: '{}', which doesn't match any known city or street in region {}.",
        typed,
        st.current_region().abbreviation()
    ));

    // Provide fuzzy suggestions for whichever mode? Or for both?
    let all_cities = reg_data.cities();
    let all_streets = reg_data.streets();

    let city_suggestions = best_fuzzy_matches(st.fuzzy_matcher(), &typed, all_cities, 3);
    let street_suggestions = best_fuzzy_matches(st.fuzzy_matcher(), &typed, all_streets, 3);

    if !city_suggestions.is_empty() {
        display_data.push("Did you mean (city)?".to_string());
        for c in &city_suggestions {
            display_data.push(format!("  {}", c));
        }
    }
    if !street_suggestions.is_empty() {
        display_data.push("Did you mean (street)?".to_string());
        for s in &street_suggestions {
            display_data.push(format!("  {}", s));
        }
    }

    display_data
}

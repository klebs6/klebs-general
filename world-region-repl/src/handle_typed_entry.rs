// ---------------- [ File: src/handle_typed_entry.rs ]
crate::ix!();

/// (A) Attempt to interpret the line as either a city or a street in the current region.
/// If it matches exactly (case-insensitive) a known city or street, we show associated
/// info from the DB. If it doesn’t match, we print “(Doesn’t match known city/street)”
/// plus a fuzzy suggestion.
pub fn handle_typed_entry<I:StorageInterface>(st: &ReplState<I>, typed: &str) -> Vec<String> {

    let mut display_data = Vec::new();

    let typed_lc = typed.to_lowercase();

    // Get the RegionData for the current region
    let reg_data = match st.regions().get(st.current_region()) {
        Some(d) => d,
        None => {
            display_data.push(format!("No RegionData for region {:?}", st.current_region().abbreviation()));
            return display_data;
        }
    };

    // 1) Check if typed matches a known city
    if reg_data.cities().iter().any(|c| c.eq_ignore_ascii_case(&typed_lc)) {
        // Found a city => show associated postal codes, streets
        let city_obj = CityName::new(&typed_lc).unwrap_or_else(|_| {
            display_data.push(format!("Weird city parse error, but continuing…"));
            CityName::new("unknown").unwrap()
        });
        display_data.push(format!("'{}' is recognized as a city in region {}.", typed, st.current_region().abbreviation()));

        // Show postal codes
        let c2z_key = c2z_key(st.current_region(), &city_obj);
        if let Some(postals) = st.db_access().get_postal_code_set(&c2z_key) {
            display_data.push(format!("Postal codes for city '{}':", typed_lc));
            for pc in postals {
                display_data.push(format!("  {}", pc.code()));
            }
        }

        // Show streets
        let c2s_key = c2s_key(st.current_region(), &city_obj);
        if let Some(streets) = st.db_access().get_street_set(&c2s_key) {
            display_data.push(format!("Streets in city '{}':", typed_lc));
            for s in streets {
                display_data.push(format!("  {}", s.name()));
            }
        }
        return display_data;
    }

    // 2) Check if typed matches a known street
    if reg_data.streets().iter().any(|s| s.eq_ignore_ascii_case(&typed_lc)) {
        display_data.push(format!("'{}' is recognized as a street in region {}.", typed, st.current_region().abbreviation()));

        let street_obj = StreetName::new(&typed_lc).unwrap_or_else(|_| {
            display_data.push(format!("Strange parse error for street. Continuing…"));
            StreetName::new("unknown").unwrap()
        });

        // Which cities is it in?
        let s2c_key = s2c_key(st.current_region(), &street_obj);
        if let Some(cities) = st.db_access().get_city_set(&s2c_key) {
            display_data.push(format!("Cities containing '{}':", typed_lc));
            for c in cities {
                display_data.push(format!("  {}", c.name()));
            }
        }

        // Which ZIP(s)?
        let s2z_key = s2z_key(st.current_region(), &street_obj);
        if let Some(zips) = st.db_access().get_postal_code_set(&s2z_key) {
            display_data.push(format!("ZIP codes containing '{}':", typed_lc));
            for z in zips {
                display_data.push(format!("  {}", z.code()));
            }
        }

        // House-number ranges?
        let abbr = st.current_region().abbreviation().to_string();
        let key = (abbr, typed_lc.clone());
        if let Some(ranges) = st.house_number_ranges().get(&key) {

            display_data.push(format!("House number ranges for '{}':", typed_lc));

            for item in known_house_number_ranges_display(ranges) {
                display_data.push(format!("{}", item));
            }

        } else {
            display_data.push(format!("No known house-number range data for '{}'.", typed_lc));
        }
        return display_data;
    }

    // If neither matched:
    display_data.push(format!("You typed: '{}', which doesn't match any known city or street in region {}.", typed, st.current_region().abbreviation()));

    // Provide fuzzy suggestions for whichever mode? Or for both? Here we can do both:
    let all_cities = reg_data.cities();
    let all_streets = reg_data.streets();

    // e.g. if city is the likely intended mode, we can do fuzzy across cities. If street, do fuzzy across streets.
    let city_suggestions = best_fuzzy_matches(st.fuzzy_matcher(), typed, all_cities, 3);
    let street_suggestions = best_fuzzy_matches(st.fuzzy_matcher(), typed, all_streets, 3);

    if !city_suggestions.is_empty() {
        display_data.push(format!("Did you mean (city)?"));
        for c in &city_suggestions {
            display_data.push(format!("  {}", c));
        }
    }
    if !street_suggestions.is_empty() {
        display_data.push(format!("Did you mean (street)?"));
        for s in &street_suggestions {
            display_data.push(format!("  {}", s));
        }
    }

    display_data
}

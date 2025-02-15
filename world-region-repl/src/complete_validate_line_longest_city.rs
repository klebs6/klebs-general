// ---------------- [ File: src/complete_validate_line_longest_city.rs ]
crate::ix!();

/// ----------------------------------------------------------------------------
/// 2) A new complete_validate_line that calls the longest-city parse plus 
///    an extra logic for house number → street completions. 
/// ----------------------------------------------------------------------------
pub fn complete_validate_line_longest_city<I:StorageInterface>(
    line_so_far:          &str,
    line_ends_with_space: bool,
    st:                   &ReplState<I>,
) -> Result<Vec<String>, ValidateCompleteError>
{
    let region = st.current_region();
    let data_access = st.db_access();

    let tokens: Vec<&str> = line_so_far.split_whitespace().collect();
    let parse = parse_validate_tokens_longest_city(&tokens, line_ends_with_space, data_access, region)?;

    match parse.cursor_field() {
        ValidateCursorField::Zip => {
            // still editing zip => suggest zips
            let prefix = parse.zip_part();
            let zips = data_access.gather_all_zips_in_region(region);
            let mut out = Vec::new();
            for pc in zips {
                if pc.code().starts_with(prefix) {
                    out.push(pc.code().to_string());
                }
            }
            out.sort();
            out.dedup();
            Ok(out)
        }
        ValidateCursorField::City => {
            // user typed a zip => we see which cityset => do partial match 
            let zip_str = parse.zip_part();
            let city_prefix = parse.city_parts().last().map(|s| s.as_str()).unwrap_or("");
            // gather city suggestions from the cityset for zip
            let pc_obj = match PostalCode::new(st.current_country(), &zip_str) {
                Ok(x) => x,
                Err(_) => return Ok(vec![]),
            };
            let cset = data_access.get_city_set(&z2c_key(region, &pc_obj)).unwrap_or_default();
            let mut out = Vec::new();
            for c in cset {
                let c_lc = c.name();
                if c_lc.starts_with(&city_prefix.to_lowercase()) {
                    out.push(c_lc.to_string());
                }
            }
            out.sort();
            out.dedup();
            Ok(out)
        }
        ValidateCursorField::HouseNumber => {
            // user typed an EXACT city, is on houseNumber
            // we can do house number suggestions from known ranges or do none
            // For demonstration: we do none
            Ok(vec![])
        }
        ValidateCursorField::Street => {
            // user typed zip + city + maybe houseNum => now is on street
            // if we do “houseNum => filter streets that have that number in range”
            // or if no houseNum => show all streets in city
            let zip_str  = parse.zip_part();
            let city_str = parse.city_parts().join(" ").to_lowercase();

            let house_num_opt 
                = parse.house_number_part()
                .as_ref()
                .and_then(|s| s.parse::<u32>().ok());

            // 1) find the zip in DB
            let pc_obj = match PostalCode::new(st.current_country(), &zip_str) {
                Ok(pc) => pc,
                Err(_) => return Ok(vec![]),
            };
            // 2) check city => see if recognized => get street set => filter
            let cset = data_access.get_city_set(&z2c_key(region, &pc_obj));
            if let Some(cities) = cset {
                // confirm city_str is recognized
                let matched_city = cities.iter().find(|cx| *cx.name() == city_str);
                if matched_city.is_none() {
                    // city not recognized => no street suggestions
                    return Ok(vec![]);
                }
                // city recognized => gather all streets from s_key or c2s_key
                // typically we do s_key for zip => which is gather streets for that zip,
                // then filter by c2s or s2c. Or we can do c2s => city->streets, then filter which are in zip.
                // For simplicity: we do c2s_key
                let city_obj = CityName::new(&city_str).unwrap();
                let c2s_key_str = c2s_key(region, &city_obj);
                let maybe_streets = data_access.get_street_set(&c2s_key_str);
                let mut suggestions = Vec::new();
                if let Some(stset) = maybe_streets {
                    // if house_num is given => we only keep streets that have a range containing house_num
                    // or if we don't store exact address, skip. If you do store it, do:
                    if let Some(hn) = house_num_opt {
                        for stx in &stset {
                            let ab = region.abbreviation().to_string();
                            let st_lc = stx.name().to_string();
                            let key = (ab, st_lc.clone());
                            // check if we have a range => if it contains hn => add
                            if let Some(ranges) = st.house_number_ranges().get(&key) {
                                let in_range = ranges.iter().any(|range| hn >= *range.start() && hn <= *range.end());
                                if in_range {
                                    suggestions.push(st_lc);
                                }
                            } else {
                                // if no range info => either skip or include. We'll skip
                            }
                        }
                    } else {
                        // no house number => suggest all
                        for stx in &stset {
                            suggestions.push(stx.name().to_string());
                        }
                    }
                }
                // now partial prefix match
                let partial_street = parse.street_parts().last().map(|s| s.as_str()).unwrap_or("");
                let partial_lower = partial_street.to_lowercase();
                let final_sugs: Vec<String> = suggestions
                    .into_iter()
                    .filter(|s| s.starts_with(&partial_lower))
                    .collect();
                let mut final_sugs = final_sugs;
                final_sugs.sort();
                final_sugs.dedup();
                return Ok(final_sugs);
            }
            Ok(vec![])
        }
    }
}

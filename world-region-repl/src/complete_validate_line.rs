// ---------------- [ File: src/complete_validate_line.rs ]
crate::ix!();

// (2) Revised complete_validate_line(...) that also takes line_ends_with_space
//
pub fn complete_validate_line<I:StorageInterface>(
    line_so_far: &str,
    line_ends_with_space: bool,
    region: &WorldRegion,
    data_access: &DataAccess<I>,
) -> Result<Vec<String>, ValidateCompleteError>
{
    let tokens: Vec<&str> = line_so_far.split_whitespace().collect();
    let parse_result = parse_validate_tokens(&tokens, line_ends_with_space)?;

    // Decide which field user is editing
    match parse_result.cursor_field() {
        ValidateCursorField::Zip => {
            let prefix = parse_result.zip_part();
            let zips = data_access.gather_all_zips_in_region(region);
            let mut suggestions = Vec::new();
            for pc in zips {
                if pc.code().starts_with(prefix) {
                    suggestions.push(pc.code().to_string());
                }
            }
            suggestions.sort();
            suggestions.dedup();
            Ok(suggestions)
        }
        ValidateCursorField::City => {
            let zip_prefix = parse_result.zip_part();
            let city_prefix = if let Some(last) = parse_result.city_parts().last() {
                last
            } else {
                ""
            };
            let possible_zips = data_access.gather_all_zips_in_region(region)
                .into_iter()
                .filter(|pc| pc.code().starts_with(zip_prefix))
                .collect::<Vec<_>>();

            let mut suggestions = Vec::new();
            for z in possible_zips {
                if let Some(cityset) = data_access.get_city_set(&z2c_key(region, &z)) {
                    for c in cityset {
                        if c.name().starts_with(&city_prefix.to_lowercase()) {
                            suggestions.push(c.name().to_string());
                        }
                    }
                }
            }
            suggestions.sort();
            suggestions.dedup();
            Ok(suggestions)
        }
        ValidateCursorField::HouseNumber => {
            // For demonstration, no completions
            Ok(vec![])
        }
        ValidateCursorField::Street => {
            let zip_prefix = parse_result.zip_part();
            let city_name_final = parse_result.city_parts().join(" ").to_lowercase();
            let street_prefix = if let Some(last) = parse_result.street_parts().last() {
                last
            } else {
                ""
            };
            let possible_zips = data_access.gather_all_zips_in_region(region)
                .into_iter()
                .filter(|pc| pc.code().starts_with(zip_prefix))
                .collect::<Vec<_>>();
            let mut suggestions = Vec::new();
            for z in possible_zips {
                if let Some(cityset) = data_access.get_city_set(&z2c_key(region, &z)) {
                    let found_city = cityset.iter().any(|cx| *cx.name() == city_name_final);
                    if !found_city {
                        continue;
                    }
                    if let Some(streetset) = data_access.get_street_set(&s_key(region, &z)) {
                        for stx in streetset {
                            let st_lc = stx.name();
                            if st_lc.starts_with(&street_prefix.to_lowercase()) {
                                suggestions.push(st_lc.to_string());
                            }
                        }
                    }
                }
            }
            suggestions.sort();
            suggestions.dedup();
            Ok(suggestions)
        }
    }
}

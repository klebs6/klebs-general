// ---------------- [ File: src/parse_validate_tokens_longest_city.rs ]
crate::ix!();

/// Below is a robust final solution that fixes:
///
/// **(A)** The bug where `"potomac river road"` was incorrectly treated as the entire city,  
///     instead of `"potomac"` for city and `"river road"` for street.
///
/// **(B)** Adds an extra layer to autocompletion logic to handle:
///   - House number suggestions (if you want to filter by known ranges).
///   - Street name suggestions for a given ZIP + city + optional house number.
///
/// The plan:
/// 1. **Multi-word city parsing**:
///    - We do a “longest prefix of typed tokens that matches a known city in the DB.”
///      This way, if the user typed `"potomac river road"`, we see that `"potomac"` 
///      is recognized but `"potomac river"` is not => so city = `"potomac"`, street = `"river road"`.
///
/// 2. **Completer**: If the user typed:
///    - `validate <zip> <city>` => once <city> is recognized and the user typed a space,
///      we autocomplete on house numbers or street. 
///    - If user typed a partial house number (or pressed space after city), we can show
///      street suggestions. If the user typed a numeric prefix, we can do known ranges, etc.
///    - If user typed a house number like “123”, we only suggest streets that have a range
///      containing 123. If we do not store exact address ranges, we fallback to all streets
///      or partial logic.
///
/// **Note**: We only demonstrate a pattern for how to integrate the “house number + street” 
/// filtering. In a real system, you might have to query DB to see which streets have house 
/// number = 123. If no exact data, we show all. The code shows a placeholder approach 
/// that you can adapt if you have per-address or per-range data.
///
/// ----------------------------------------------------------------------------
/// 1) Replacing parse_validate_tokens with parse_validate_tokens_longest_city() 
///    that tries the longest prefix matching a city in the DB.
/// ----------------------------------------------------------------------------
///
pub fn parse_validate_tokens_longest_city<I:StorageInterface>(
    tokens:               &[&str],
    line_ends_with_space: bool,
    da:                   &DataAccess<I>,               // so we can check if <zip> + some city tokens form a known city
    region:               &WorldRegion,

) -> Result<ValidateParseResult, ValidateParseError> {

    let current_country: Country = region.clone().try_into().expect("expected to have a country");

    if tokens.is_empty() {
        return Err(ValidateParseError::NoTokensProvided);
    }
    if !tokens[0].eq_ignore_ascii_case("validate") {
        return Err(ValidateParseError::NotAValidateCommand);
    }

    // We’ll gather:
    //   zip_part,
    //   city_parts,
    //   house_number_part,
    //   street_parts,
    //   cursor_field.
    let mut zip_part = String::new();
    let mut city_parts = Vec::new();
    let mut house_number_part: Option<String> = None;
    let mut street_parts = Vec::new();
    let mut cursor_field = ValidateCursorField::Zip;

    if tokens.len() == 1 {
        // only "validate"
        return Ok(ValidateParseResultBuilder::default()
            .zip_part(zip_part)
            .city_parts(city_parts)
            .house_number_part(None)
            .street_parts(street_parts)
            .cursor_field(cursor_field)
            .build()
            .unwrap());
    }

    // Next token => ZIP
    zip_part = tokens[1].to_string();

    // If we have exactly 2 tokens => might still be editing ZIP or done with ZIP
    if tokens.len() == 2 {
        // check trailing space
        if line_ends_with_space {
            cursor_field = ValidateCursorField::City;
        } else {
            cursor_field = ValidateCursorField::Zip;
        }
        return Ok(ValidateParseResultBuilder::default()
            .zip_part(zip_part)
            .city_parts(city_parts)
            .house_number_part(None)
            .street_parts(street_parts)
            .cursor_field(cursor_field)
            .build()
            .unwrap());
    }

    // from tokens[2..], we do a “longest prefix that forms a valid city in ZIP=zip_part”
    // so we accumulate tokens in potential_city. Each iteration we check if `potential_city`
    // is recognized. If it’s recognized, we record that as best_city so far, then see if 
    // next token extends it to a bigger city name that’s recognized. If not recognized, we revert to best known.
    let mut i = 2;
    let n = tokens.len();

    // gather a list of city tokens
    let mut typed_tokens_for_city: Vec<String> = Vec::new();
    // We also track the "best match index" for city
    let mut best_city_match_index: Option<usize> = None;

    // We first get the set of possible city strings for that ZIP
    // so we can check membership quickly
    let zip_obj = PostalCode::new(current_country, &zip_part).unwrap_or_else(|_| {
        // if user typed an invalid zip => we can’t city check. We’ll do a fallback parse
        // but in practice we do nothing, or we might skip. For now, we do an empty city set
        // to avoid crashing. 
        // A real code might handle parse error earlier.
        PostalCode::new(current_country, "invalid").unwrap()
    });
    let cityset_for_zip = if let Some(cc) = da.get_city_set(&z2c_key(region, &zip_obj)) {
        cc // a BTreeSet<CityName>
    } else {
        // no known city => fallback
        BTreeSet::new()
    };

    let city_strings: HashSet<String> = cityset_for_zip
        .iter()
        .map(|cx| cx.name().to_string())
        .collect();

    // We'll accumulate tokens one-by-one:
    //   potential_city = "potomac"
    //   check if in city_strings => yes => best_city_match_index = i
    //   then potential_city = "potomac river"
    //   check => if not in city_strings => break
    //   or we can keep going to see if "potomac river road" is also recognized.
    //   We'll do a while loop. 
    //   Then the "best_city_match_index" is the last index for which we had a recognized city string.
    let mut potential_city = String::new();
    let mut last_match_index: Option<usize> = None;

    let mut j = i; // local index
    while j < n {
        let tk = tokens[j];
        // if we see a numeric house number => break from city
        if tk.parse::<u32>().is_ok() {
            break;
        }
        // add token
        if potential_city.is_empty() {
            potential_city = tk.to_string().to_lowercase();
        } else {
            potential_city.push(' ');
            potential_city.push_str(&tk.to_lowercase());
        }
        if city_strings.contains(&potential_city) {
            // record last match index as j
            last_match_index = Some(j);
        }
        j += 1;
    }

    // If last_match_index is None => means we never found a recognized city prefix
    // => fallback to the first token as city or partial. 
    // But let’s do a simple approach: if we found no match => treat the entire chunk until numeric as city
    // If we found a match => that’s the city. Then the remainder is house/street
    let (city_end_idx, city_str) = if let Some(idx) = last_match_index {
        // city from tokens[2..=idx]
        let city_slice = &tokens[2..=idx];
        (idx, city_slice.join(" ").to_lowercase())
    } else {
        // no recognized prefix => take everything up to numeric
        let mut city_slice = Vec::new();
        let mut pointer = i;
        while pointer < n {
            if tokens[pointer].parse::<u32>().is_ok() {
                break;
            }
            city_slice.push(tokens[pointer]);
            pointer += 1;
        }
        (pointer.saturating_sub(1), city_slice.join(" ").to_lowercase())
    };

    // now city_end_idx is the final index of city tokens 
    // if city_end_idx < j => means we might have leftover. 
    let mut house_num_idx = city_end_idx + 1;
    if house_num_idx < n && tokens[house_num_idx].parse::<u32>().is_ok() {
        // parse house number
        house_number_part = Some(tokens[house_num_idx].to_string());
        house_num_idx += 1;
    }

    // leftover => street
    let mut leftover_street_tokens = Vec::new();
    let mut pointer = house_num_idx;
    while pointer < n {
        leftover_street_tokens.push(tokens[pointer]);
        pointer += 1;
    }

    // So we have zip_part, city_str, house_number_part, street_parts
    let street_strs: Vec<String> = leftover_street_tokens.iter().map(|s| s.to_string()).collect();

    // Decide the cursor_field
    // if leftover_street_tokens is empty => user is in city or houseNumber
    // if user typed house_num => if leftover is empty => they are editing street
    // etc. We do a simple approach:
    let mut cursor_field = ValidateCursorField::Zip;
    if !zip_part.is_empty() {
        // done with zip
        // if city_str is empty => still editing city
        if !city_str.is_empty() {
            // if we saw a house_num => next is Street
            if house_number_part.is_some() {
                if street_strs.is_empty() {
                    cursor_field = ValidateCursorField::Street;
                } else {
                    // typed some street => likely still editing
                    cursor_field = ValidateCursorField::Street;
                }
            } else {
                // no house number
                if street_strs.is_empty() {
                    // user typed city => if line ends with space => might be next field
                    if line_ends_with_space {
                        // we consider them to be in houseNumber or street
                        // let's say we do street next
                        cursor_field = ValidateCursorField::Street;
                    } else {
                        cursor_field = ValidateCursorField::City;
                    }
                } else {
                    // we have leftover => that’s street
                    cursor_field = ValidateCursorField::Street;
                }
            }
        } else {
            // city is empty => user is or was editing city
            cursor_field = ValidateCursorField::City;
        }
    }

    Ok(ValidateParseResultBuilder::default()
        .zip_part(zip_part)
        .city_parts(if city_str.is_empty() { vec![] } else { city_str.split_whitespace().map(|s| s.to_string()).collect() })
        .house_number_part(house_number_part)
        .street_parts(street_strs)
        .cursor_field(cursor_field)
        .build()
        .unwrap())
}

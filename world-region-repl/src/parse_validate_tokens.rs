// ---------------- [ File: src/parse_validate_tokens.rs ]
crate::ix!();

// (1) Refined parse_validate_tokens(...) that considers trailing space
//
pub fn parse_validate_tokens(
    tokens: &[&str],
    line_ends_with_space: bool, // <-- new parameter
) -> Result<ValidateParseResult, ValidateParseError> 
{
    if tokens.is_empty() {
        return Err(ValidateParseError::NoTokensProvided);
    }
    if !tokens[0].eq_ignore_ascii_case("validate") {
        return Err(ValidateParseError::NotAValidateCommand);
    }

    // We'll build up partial fields
    let mut zip_part = String::new();
    let mut city_parts = Vec::new();
    let mut house_number_part: Option<String> = None;
    let mut street_parts = Vec::new();
    let mut cursor_field = ValidateCursorField::Zip;

    // If the user typed only `validate`
    if tokens.len() == 1 {
        return Ok(ValidateParseResultBuilder::default()
            .zip_part(zip_part)
            .city_parts(city_parts)
            .house_number_part(house_number_part)
            .street_parts(street_parts)
            .cursor_field(cursor_field)
            .build()
            .unwrap());
    }

    // Next token => ZIP
    zip_part = tokens[1].to_string();

    // If user typed exactly 2 tokens => maybe still editing ZIP or they've moved on to city
    if tokens.len() == 2 {
        if line_ends_with_space {
            // user typed "validate <zip> " => begin city field
            cursor_field = ValidateCursorField::City;
        } else {
            // still editing zip
            cursor_field = ValidateCursorField::Zip;
        }
        return Ok(ValidateParseResultBuilder::default()
            .zip_part(zip_part)
            .city_parts(city_parts)
            .house_number_part(house_number_part)
            .street_parts(street_parts)
            .cursor_field(cursor_field)
            .build()
            .unwrap());
    }

    // If we get here, we have >= 3 tokens => parse city tokens
    let mut i = 2;
    let n = tokens.len();
    let mut house_found = false;

    while i < n {
        let tk = tokens[i];
        if let Ok(_num) = tk.parse::<u32>() {
            // We interpret it as house number
            house_found = true;
            house_number_part = Some(tk.to_string());
            i += 1;
            break;
        } else {
            city_parts.push(tk.to_string());
            i += 1;
        }
    }

    // If we've consumed all tokens => user is currently editing whichever field
    if i >= n {
        if house_found {
            cursor_field = ValidateCursorField::HouseNumber;
        } else {
            cursor_field = ValidateCursorField::City;
        }
        return Ok(ValidateParseResultBuilder::default()
            .zip_part(zip_part)
            .city_parts(city_parts)
            .house_number_part(house_number_part)
            .street_parts(street_parts)
            .cursor_field(cursor_field)
            .build()
            .unwrap());
    }

    // If a house number was found => next tokens must be street
    if house_found {
        if i == n {
            // user typed house_number but no street yet => editing street
            cursor_field = ValidateCursorField::Street;
            return Ok(ValidateParseResultBuilder::default()
                .zip_part(zip_part)
                .city_parts(city_parts)
                .house_number_part(house_number_part)
                .street_parts(street_parts)
                .cursor_field(cursor_field)
                .build()
                .unwrap());
        } else {
            // we have at least one street token
            street_parts.extend(tokens[i..].iter().map(|s| s.to_string()));
            cursor_field = ValidateCursorField::Street;
            return Ok(ValidateParseResultBuilder::default()
                .zip_part(zip_part)
                .city_parts(city_parts)
                .house_number_part(house_number_part)
                .street_parts(street_parts)
                .cursor_field(cursor_field)
                .build()
                .unwrap());
        }
    } else {
        // No house number => treat everything else as street
        street_parts.push(tokens[i].to_string());
        i += 1;
        while i < n {
            street_parts.push(tokens[i].to_string());
            i += 1;
        }
        cursor_field = ValidateCursorField::Street;
        return Ok(ValidateParseResultBuilder::default()
            .zip_part(zip_part)
            .city_parts(city_parts)
            .house_number_part(house_number_part)
            .street_parts(street_parts)
            .cursor_field(cursor_field)
            .build()
            .unwrap());
    }
}

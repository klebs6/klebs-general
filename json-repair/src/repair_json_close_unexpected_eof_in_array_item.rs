crate::ix!();

pub fn repair_json_close_unexpected_eof_in_array_item(input: &str) 
    -> Result<String, JsonRepairError> 
{
    let mut repaired    = input.to_owned();
    let mut changed     = false;
    let mut added_chars = String::new();

    // Close any unclosed strings
    if repaired.matches('"').count() % 2 != 0 {
        repaired.push('"');
        changed = true;
        added_chars.push('"');
    }

    // Close any unclosed arrays
    let open_brackets = repaired.matches('[').count() as isize - repaired.matches(']').count() as isize;
    if open_brackets > 0 {
        for _ in 0..open_brackets {
            repaired.push(']');
            changed = true;
            added_chars.push(']');
        }
    }

    // Close any unclosed objects
    let open_braces = repaired.matches('{').count() as isize - repaired.matches('}').count() as isize;
    if open_braces > 0 {
        for _ in 0..open_braces {
            repaired.push('}');
            changed = true;
            added_chars.push('}');
        }
    }

    if changed {
        info!("Repaired JSON by adding the following characters at the end: {}", added_chars);
    }

    Ok(repaired)
}

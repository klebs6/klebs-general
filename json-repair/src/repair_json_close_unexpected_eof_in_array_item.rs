crate::ix!();

pub fn repair_json_close_unexpected_eof_in_array_item(input: &str) 
    -> Result<String,JsonRepairError> 
{
    info!("fixing any cases where we need to close an unexpected EOF inside an array item");

    let mut repaired = input.to_owned();

    // Close any unclosed strings
    if repaired.matches('"').count() % 2 != 0 {
        repaired.push('"');
    }

    // Close any unclosed arrays
    let open_brackets = repaired.matches('[').count() as isize - repaired.matches(']').count() as isize;
    if open_brackets > 0 {
        for _ in 0..open_brackets {
            repaired.push(']');
        }
    }

    // Close any unclosed objects
    let open_braces = repaired.matches('{').count() as isize - repaired.matches('}').count() as isize;
    if open_braces > 0 {
        for _ in 0..open_braces {
            repaired.push('}');
        }
    }

    Ok(repaired)
}

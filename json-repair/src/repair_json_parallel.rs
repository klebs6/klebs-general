crate::ix!();

pub fn repair_json_string_parallel(input: &str) -> Result<Value, JsonRepairError> {
    if input.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }

    // Attempt to parse the input as-is.
    if let Ok(mut value) = json5::from_str(&input) {
        remove_control_characters_in_value(&mut value);
        return Ok(value);
    }

    if let Ok(repaired) = attempt_repair_json_string(input) {
        return Ok(repaired);
    }

    // Define a list of repair functions.
    let repair_functions: Vec<fn(&str) -> Result<String, JsonRepairError>> = vec![
        repair_json_accidental_single_quote_instead_of_double_quote,
        repair_json_comma_behavior,
        repair_json_truncated_boolean_behavior,
        repair_json_mismatched_brackets,
        repair_json_control_characters,
        repair_json_missing_commas_in_list,
        repair_json_remove_duplicate_quotes,
        repair_json_close_unexpected_eof,
        repair_json_add_missing_quotes,
        repair_json_handle_eof_between_lists,
        repair_json_fix_mismatched_quotes,
        repair_json_close_unexpected_eof_in_array_tag,
        repair_json_close_unexpected_eof_in_array_item,
    ];

    // First round: Try each repair independently.
    for repair_fn in &repair_functions {
        let repaired_input = match repair_fn(input) {
            Ok(output) => output,
            Err(_) => continue, // Skip if repair function fails.
        };

        if let Ok(repaired) = attempt_repair_json_string(&repaired_input) {
            return Ok(repaired);
        }
    }

    // Optional: Second round with combinations of repairs.
    // This can be implemented if needed.

    // If all repairs fail, return an error.
    Err(JsonRepairError::AllAttemptedRepairsFailed)
}


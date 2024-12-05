crate::ix!();

pub fn repair_json_string_series(input: &str) -> Result<Value, JsonRepairError> {

    if input.is_empty() {
        return Ok(serde_json::json!({}));
    }

    if let Ok(mut value) = json5::from_str(&input) {
        remove_control_characters_in_value(&mut value);
        return Ok(value);
    }

    // First attempt to parse the input as-is.
    if let Ok(repaired) = attempt_repair_json_string(input) {
        return Ok(repaired);
    }

    macro_rules! repair_step {
        ($step:ident) => {
            {
                // Apply repair for accidental single quotes first.
                let input = $step(&input)?;
                if let Ok(repaired) = attempt_repair_json_string(&input) {
                    return Ok(repaired);
                }
            }
        };
    }

    // Apply repair for accidental single quotes first.
    repair_step!{repair_json_missing_closing_quotes}; 

    repair_step!{repair_json_accidental_single_quote_instead_of_double_quote}; 

    // Then attempt to repair comma issues.
    repair_step!{repair_json_comma_behavior};

    // Then attempt to repair truncated booleans.
    repair_step!{repair_json_truncated_boolean_behavior};                      

    // Then attempt to repair mismatched brackets
    repair_step!{repair_json_mismatched_brackets};                             

    // Then attempt to repair missing commas inside list
    repair_step!{repair_json_missing_commas_in_list};                          

    // Then attempt to remove control characters
    repair_step!{repair_json_control_characters};                              

    // Then attempt to remove duplicate quotes
    repair_step!{repair_json_remove_duplicate_quotes};                         

    // Then attempt to close unexpected EOF
    repair_step!{repair_json_close_unexpected_eof};                            

    // Then attempt to add missing quotes
    repair_step!{repair_json_add_missing_quotes};

    // Then attempt to handle EOF between lists
    repair_step!{repair_json_handle_eof_between_lists};                        

    // Then attempt to fix mismatched quotes
    repair_step!{repair_json_fix_mismatched_quotes};                           

    // Then attempt to close unexpected EOF in array tag
    repair_step!{repair_json_close_unexpected_eof_in_array_tag};               

    // Then attempt to close unexpected EOF in array item
    repair_step!{repair_json_close_unexpected_eof_in_array_item};              

    // If all repairs fail, return an error.
    Err(JsonRepairError::AllAttemptedRepairsFailed)
}

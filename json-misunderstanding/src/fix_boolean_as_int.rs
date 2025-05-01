crate::ix!();

/// #22 Fixes "boolean represented as integers" (0 -> false, 1 -> true).
pub fn fix_boolean_as_int(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_boolean_as_int");
    match val {
        serde_json::Value::Number(num) => {
            if let Some(i) = num.as_i64() {
                if i == 0 {
                    debug!("Converting integer '0' to boolean false");
                    return serde_json::Value::Bool(false);
                } else if i == 1 {
                    debug!("Converting integer '1' to boolean true");
                    return serde_json::Value::Bool(true);
                }
            }
            serde_json::Value::Number(num)
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_boolean_as_int {
    use super::*;

    #[traced_test]
    fn test_bool_as_1_0() {
        trace!("Testing fix_boolean_as_int with integer 0 and 1");
        let input_true = json!(1);
        let input_false = json!(0);

        debug!("Input true: {}", input_true);
        debug!("Input false: {}", input_false);

        let output_true = fix_boolean_as_int(input_true.clone());
        let output_false = fix_boolean_as_int(input_false.clone());

        assert_eq!(output_true, json!(true), "Integer 1 should become true");
        assert_eq!(output_false, json!(false), "Integer 0 should become false");
        info!("Boolean-as-int was fixed from 1 -> true and 0 -> false");
    }

    #[traced_test]
    fn test_non_boolean_integer() {
        trace!("Testing fix_boolean_as_int with integer other than 0 or 1");
        let input = json!(42);
        debug!("Input: {}", input);

        let output = fix_boolean_as_int(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "Should remain unchanged if it's not 0 or 1");
        info!("Non-boolean integer remains as-is");
    }

    #[traced_test]
    fn test_non_numeric() {
        trace!("Testing fix_boolean_as_int with non-numeric input");
        let input = json!("string");
        debug!("Input: {}", input);

        let output = fix_boolean_as_int(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "Non-numeric inputs remain unchanged");
        info!("Non-numeric input was left unmodified");
    }
}

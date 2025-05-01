crate::ix!();

pub fn fix_primitive(
    val: serde_json::Value,
    config: &MisunderstandingCorrectionConfig,
) -> serde_json::Value {
    match val {
        serde_json::Value::String(s) => {
            // #5 boolean strings
            if *config.handle_boolean_strings() {
                if s == "true" {
                    tracing::debug!("boolean string 'true' -> true");
                    return serde_json::Value::Bool(true);
                } else if s == "false" {
                    tracing::debug!("boolean string 'false' -> false");
                    return serde_json::Value::Bool(false);
                }
            }
            // #6 numeric strings
            if *config.handle_numeric_strings() {
                // Try i64 first
                if let Ok(i) = s.parse::<i64>() {
                    tracing::debug!("numeric string '{}' -> {}", s, i);
                    return serde_json::Value::Number(i.into());
                }
                // If that fails, try a “reasonable” float parse
                if let Ok(f) = s.parse::<f64>() {
                    // Optionally skip if it’s too large and just store as string
                    // E.g. 9.876543210123457e18 is parseable but loses precision
                    if f.is_finite() && f.abs() < (1_u64 << 53) as f64 {
                        tracing::debug!("numeric string '{}' -> {}", s, f);
                        if let Some(num) = serde_json::Number::from_f64(f) {
                            return serde_json::Value::Number(num);
                        }
                    }
                }
            }
            // #13 null value misplacement
            if *config.handle_null_value_misplacement() && s == "null" {
                tracing::debug!("'null' string -> null value");
                return serde_json::Value::Null;
            }
            // #17 missing_array, #18 stringified_json (stubs)
            if *config.handle_missing_array() {
                tracing::debug!("handle_missing_array is selected, no real fix implemented yet");
            }
            if *config.handle_stringified_json() {
                tracing::debug!("handle_stringified_json is selected, no real fix implemented yet");
            }
            serde_json::Value::String(s)
        }

        serde_json::Value::Number(n) => {
            // COMMENT OUT or REMOVE the boolean-as-int block:
            //
            // if *config.handle_boolean_as_int() {
            //     if let Some(i) = n.as_i64() {
            //         if i == 0 {
            //             tracing::debug!("boolean as int '0' -> false");
            //             return serde_json::Value::Bool(false);
            //         } else if i == 1 {
            //             tracing::debug!("boolean as int '1' -> true");
            //             return serde_json::Value::Bool(true);
            //         }
            //     }
            // }
            if *config.handle_date_format_confusion() {
                tracing::debug!("handle_date_format_confusion is selected, no real transform done");
            }
            serde_json::Value::Number(n)
        }

        other => other,
    }
}

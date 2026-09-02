crate::ix!();

pub fn fix_date_format_confusion(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_date_format_confusion");
    match val {
        serde_json::Value::Object(mut obj) => {
            let date_like_keys = ["date", "timestamp"];
            for k in &date_like_keys {
                if let Some(serde_json::Value::Number(num)) = obj.get_mut(*k) {
                    if let Some(epoch) = num.as_i64() {
                        // Updated: only convert if epoch is in [946684800..2147483647)
                        if epoch >= 946684800 && epoch < 2147483647 {
                            debug!("Converting epoch {} in field '{}' to ISO8601 string", epoch, k);
                            if let Some(ndt) = chrono::NaiveDateTime::from_timestamp_opt(epoch, 0) {
                                let datetime_utc: chrono::DateTime<chrono::Utc> = chrono::DateTime::from_utc(ndt, chrono::Utc);
                                let iso_str = datetime_utc.to_rfc3339();
                                *obj.get_mut(*k).unwrap() = serde_json::Value::String(iso_str);
                            } else {
                                trace!("Epoch {} could not be converted via chrono (out of range). Leaving numeric.", epoch);
                            }
                        } else {
                            trace!("Epoch {} is considered out-of-range; leaving it as numeric.", epoch);
                        }
                    }
                }
            }
            serde_json::Value::Object(obj)
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_date_format_confusion {
    use super::*;

    #[traced_test]
    fn test_convert_epoch_in_date_field() {
        trace!("Testing fix_date_format_confusion with epoch in 'date' field");
        // 946684800 = 2000-01-01T00:00:00Z
        let input = json!({"date": 946684800, "timestamp": 123});
        debug!("Input: {}", input);

        let output = fix_date_format_confusion(input.clone());
        debug!("Output: {}", output);

        // The "date" field should become ISO8601 if within range
        let date_val = output.get("date").unwrap();
        let date_str = date_val.as_str().expect("Should be a string now");
        info!("Converted date field: {}", date_str);

        // We won't parse the final string exactly in the test, but we expect it to start with "2000-01-01T"
        assert!(date_str.starts_with("2000-01-01T"), "Should convert epoch 946684800 to year 2000 in ISO8601");
        // "timestamp" is 123, outside our naive range, so it remains numeric.
        assert_eq!(output.get("timestamp"), Some(&json!(123)));
        info!("Date field converted to ISO8601; timestamp left unchanged");
    }

    #[traced_test]
    fn test_out_of_range_epoch() {
        trace!("Testing fix_date_format_confusion with out-of-range epoch");
        // Something far outside the typical range
        let input = json!({"timestamp": 2147483647 });
        debug!("Input: {}", input);

        let output = fix_date_format_confusion(input.clone());
        debug!("Output: {}", output);

        // Should remain numeric if out of range
        assert_eq!(output, input, "Out-of-range epoch remains unchanged");
        info!("No changes for out-of-range epoch");
    }

    #[traced_test]
    fn test_non_numeric_date() {
        trace!("Testing fix_date_format_confusion with non-numeric 'date'");
        let input = json!({"date": "2025-04-29T00:00:00Z"});
        debug!("Input: {}", input);

        let output = fix_date_format_confusion(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "String date remains unchanged (we only convert numeric epoch to string)");
        info!("No changes for string-based date fields");
    }
}

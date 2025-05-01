crate::ix!();

/// #27 Fixes "numeric keys misunderstanding" by, for instance:
/// {"scores": {"1": {"user":"alice","score":10}, "2":{"user":"bob","score":15}}}
/// becomes {"scores": {"alice":10, "bob":15}}
/// if each sub-value has "user" and "score".
pub fn fix_numeric_keys_misunderstanding(val: serde_json::Value) -> serde_json::Value {
    trace!("Beginning fix_numeric_keys_misunderstanding");
    match val {
        serde_json::Value::Object(mut obj) => {
            // For each key, if the value is an object with numeric keys,
            // and each of those values is an object with "user" and "score", we flatten.
            for (field_key, field_val) in obj.iter_mut() {
                if let serde_json::Value::Object(sub_obj) = field_val {
                    // Check if all keys in sub_obj are numeric
                    let all_numeric = sub_obj.keys().all(|k| k.parse::<usize>().is_ok());
                    let all_user_score = sub_obj.values().all(|v| {
                        v.as_object().map_or(false, |m| m.contains_key("user") && m.contains_key("score"))
                    });
                    if all_numeric && all_user_score {
                        debug!("Flattening numeric-keys misunderstanding in field '{}'", field_key);
                        let mut new_map = serde_json::Map::new();
                        for (_, v) in sub_obj.iter() {
                            if let Some(m) = v.as_object() {
                                let user = m.get("user").and_then(|x| x.as_str()).unwrap_or("unknown").to_owned();
                                let score = m.get("score").cloned().unwrap_or(serde_json::Value::Null);
                                new_map.insert(user, score);
                            }
                        }
                        *field_val = serde_json::Value::Object(new_map);
                    }
                }
            }
            serde_json::Value::Object(obj)
        }
        other => other,
    }
}

#[cfg(test)]
mod test_fix_numeric_keys_misunderstanding {
    use super::*;

    #[traced_test]
    fn test_flatten_numeric_keys_user_score() {
        trace!("Testing fix_numeric_keys_misunderstanding with user-score style data");
        let input = json!({
            "scores": {
                "1": {"user": "alice", "score": 10},
                "2": {"user": "bob", "score": 15}
            },
            "unrelated": 42
        });
        debug!("Input: {}", input);

        let expected = json!({
            "scores": {
                "alice": 10,
                "bob": 15
            },
            "unrelated": 42
        });
        let output = fix_numeric_keys_misunderstanding(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, expected, "Should flatten numeric keys for user/score pattern");
        info!("Flattened numeric keys misunderstanding for 'scores'");
    }

    #[traced_test]
    fn test_not_all_numeric_keys() {
        trace!("Testing fix_numeric_keys_misunderstanding where keys are not all numeric");
        let input = json!({
            "scores": {
                "1": {"user": "alice", "score": 10},
                "X": {"user": "charlie", "score": 25}
            }
        });
        debug!("Input: {}", input);

        // Because not all keys are numeric, we do nothing
        let output = fix_numeric_keys_misunderstanding(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "No flattening occurs if not all keys are numeric");
        info!("No changes applied when some keys are non-numeric");
    }

    #[traced_test]
    fn test_no_user_score_pattern() {
        trace!("Testing fix_numeric_keys_misunderstanding with numeric keys but no 'user'/'score' pattern");
        let input = json!({
            "data": {
                "1": {"alpha": 123},
                "2": {"alpha": 456}
            }
        });
        debug!("Input: {}", input);

        // Because sub-values do not contain "user" and "score", we do nothing
        let output = fix_numeric_keys_misunderstanding(input.clone());
        debug!("Output: {}", output);

        assert_eq!(output, input, "No flattening occurs if sub-objects lack 'user'/'score'");
        info!("No changes when 'user'/'score' pattern is absent");
    }
}

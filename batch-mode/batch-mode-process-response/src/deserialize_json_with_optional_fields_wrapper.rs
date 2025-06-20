crate::ix!();

/// Attempt to deserialize `json_value` into `T`.  
/// If the top‐level attempt fails, the function looks for a nested `"fields"`
/// object and tries again.  
/// All paths are heavily traced so we can see exactly where deserialization
/// succeeds or fails.
#[instrument(level = "trace", skip(json_value))]
pub fn deserialize_json_with_optional_fields_wrapper<T>(
    json_value: &serde_json::Value,
) -> Result<T, serde_json::Error>
where
    T: DeserializeOwned,
{
    trace!("Attempting direct deserialization into target struct …");
    match serde_json::from_value::<T>(json_value.clone()) {
        Ok(t) => {
            debug!("Direct deserialization succeeded.");
            return Ok(t);
        }
        Err(e_direct) => {
            debug!("Direct deserialization failed: {:?}", e_direct);

            trace!("Checking for `fields` wrapper …");
            if let Some(inner) = json_value.get("fields") {
                trace!("`fields` wrapper found — trying inner value deserialization …");
                match serde_json::from_value::<T>(inner.clone()) {
                    Ok(t) => {
                        info!("Deserialization succeeded after unwrapping `fields`.");
                        Ok(t)
                    }
                    Err(e_inner) => {
                        error!(
                            "Deserialization failed after unwrapping `fields`: {:?}",
                            e_inner
                        );
                        // Return the *inner* error because that is the final failure.
                        Err(e_inner)
                    }
                }
            } else {
                error!("No `fields` wrapper present; cannot recover.");
                // Return the *direct* error: that is the best information we have.
                Err(e_direct)
            }
        }
    }
}

#[cfg(test)]
mod deserialize_json_with_optional_fields_wrapper_tests {
    use super::*;

    /// A minimal structure that mirrors the fields we expect to be present
    /// in the JSON used by `AiReadmeWriterDesiredOutput`.  Using a tiny struct
    /// keeps the tests focussed on (de)serialisation semantics rather than the
    /// production‑size type.
    #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
    struct TestStruct {
        crate_name: String,
        answer:     u32,
    }

    /// Helper that converts a raw JSON string into a `serde_json::Value` so the
    /// call‑sites stay tidy.
    fn json_val(s: &str) -> serde_json::Value {
        serde_json::from_str::<serde_json::Value>(s).expect("JSON must parse for test setup")
    }

    #[traced_test]
    fn direct_deserialization_succeeds() {
        trace!("===== BEGIN TEST: direct_deserialization_succeeds =====");

        // Top‑level keys exactly match `TestStruct`.
        let input = json_val(r#"{ "crate_name": "foo", "answer": 42 }"#);

        let result = deserialize_json_with_optional_fields_wrapper::<TestStruct>(&input);

        assert!(
            result.is_ok(),
            "Direct deserialization should succeed when keys are at the top level"
        );
        assert_eq!(
            result.unwrap(),
            TestStruct {
                crate_name: "foo".into(),
                answer: 42
            }
        );

        trace!("===== END TEST: direct_deserialization_succeeds =====");
    }

    #[traced_test]
    fn fields_wrapper_deserialization_succeeds() {
        trace!("===== BEGIN TEST: fields_wrapper_deserialization_succeeds =====");

        // Production assistant output often nests user fields under `"fields"`.
        let input = json_val(
            r#"
            {
                "struct_name":"Whatever",
                "fields": {
                    "crate_name":"bar",
                    "answer":1337
                }
            }"#,
        );

        let result = deserialize_json_with_optional_fields_wrapper::<TestStruct>(&input);

        assert!(
            result.is_ok(),
            "`fields` wrapper should be detected and unwrapped automatically"
        );
        assert_eq!(
            result.unwrap(),
            TestStruct {
                crate_name: "bar".into(),
                answer: 1337
            }
        );

        trace!("===== END TEST: fields_wrapper_deserialization_succeeds =====");
    }

    #[traced_test]
    fn fails_when_neither_top_level_nor_wrapper_matches() {
        trace!("===== BEGIN TEST: fails_when_neither_top_level_nor_wrapper_matches =====");

        // Top‑level keys wrong *and* no `"fields"` key.
        let input = json_val(r#"{ "not_it": true }"#);

        let result = deserialize_json_with_optional_fields_wrapper::<TestStruct>(&input);

        assert!(
            result.is_err(),
            "Should fail because required keys are missing and no wrapper is present"
        );

        trace!("===== END TEST: fails_when_neither_top_level_nor_wrapper_matches =====");
    }

    #[traced_test]
    fn fails_when_wrapper_present_but_incorrect() {
        trace!("===== BEGIN TEST: fails_when_wrapper_present_but_incorrect =====");

        // `"fields"` exists but does NOT contain the expected keys.
        let input = json_val(
            r#"
            {
                "fields": {
                    "wrong":"shape"
                }
            }"#,
        );

        let result = deserialize_json_with_optional_fields_wrapper::<TestStruct>(&input);

        assert!(
            result.is_err(),
            "Should fail because inner object lacks required keys"
        );

        trace!("===== END TEST: fails_when_wrapper_present_but_incorrect =====");
    }

    #[traced_test]
    fn succeeds_with_additional_unrelated_top_level_keys() {
        trace!("===== BEGIN TEST: succeeds_with_additional_unrelated_top_level_keys =====");

        // Extra keys should be ignored by Serde by default.
        let input = json_val(
            r#"
            {
                "crate_name": "baz",
                "answer": 7,
                "extraneous": "ignored"
            }"#,
        );

        let result = deserialize_json_with_optional_fields_wrapper::<TestStruct>(&input);

        assert!(
            result.is_ok(),
            "Serde should ignore unrelated top‑level keys when `deny_unknown_fields` is not used"
        );
        assert_eq!(
            result.unwrap(),
            TestStruct {
                crate_name: "baz".into(),
                answer: 7
            }
        );

        trace!("===== END TEST: succeeds_with_additional_unrelated_top_level_keys =====");
    }
}

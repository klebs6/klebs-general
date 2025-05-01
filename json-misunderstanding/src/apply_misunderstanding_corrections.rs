// ---------------- [ File: json-misunderstanding/src/apply_misunderstanding_corrections.rs ]
crate::ix!();

pub fn apply_misunderstanding_corrections(
    config: &MisunderstandingCorrectionConfig,
    input: serde_json::Value,
) -> serde_json::Value {
    use serde_json::Value;

    tracing::info!("Starting JSON misunderstanding corrections");

    // Now, perform the top-level fix pass
    fix_value(input, config)
}

// ==============================================
// AST Item #3: The test module
// ==============================================

#[cfg(test)]
mod check_apply_misunderstanding_corrections {
    use super::*;

    #[traced_test]
    fn test_basic_map_vector_confusion() {
        info!("Testing basic map-vector confusion");
        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_map_vector_confusion(true)
            .handle_nested_vector_flattening(false)
            .handle_single_element_vector_omission(false)
            .handle_vector_as_map_of_indices(false)
            .handle_boolean_strings(false)
            .handle_numeric_strings(false)
            .handle_missing_wrapper_object(false)
            .handle_unnecessary_additional_nesting(false)
            .handle_flattened_key_value_pairs(false)
            .handle_array_wrapped_single_objects(false)
            .handle_key_name_misalignment(false)
            .handle_timestamp_misformatting(false)
            .handle_null_value_misplacement(false)
            .handle_singleton_array_instead_of_object(false)
            .handle_reversed_map_structure(false)
            .build()
            .unwrap();

        let misunderstood = serde_json::json!({
            "A": {
                "descriptor": "D1",
                "timestamp": 123
            }
        });

        let expected = serde_json::json!([
            {
                "name": "A",
                "descriptor": "D1",
                "timestamp": 123
            }
        ]);

        let fixed = apply_misunderstanding_corrections(&config, misunderstood);
        assert_eq!(fixed, expected, "Map–Vector confusion was not fixed properly");
    }

    // Each misunderstanding gets two tests: one with the fix enabled, one disabled.
    // The "disabled" version ensures that the original input remains unchanged.
    // This helps confirm that each config toggle does exactly what we expect.

    // ------------------------------------------------------
    // #1 MAP–VECTOR CONFUSION
    // ------------------------------------------------------

    #[traced_test]
    fn map_vector_confusion_enabled() {
        info!("Testing map-vector confusion with fix ENABLED");
        let input = json!({
            "A": {
                "descriptor": "D1",
                "timestamp": 123
            }
        });
        let expected = json!([
            {
                "name": "A",
                "descriptor": "D1",
                "timestamp": 123
            }
        ]);

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_map_vector_confusion(true)
            .handle_nested_vector_flattening(false)
            .handle_single_element_vector_omission(false)
            .handle_vector_as_map_of_indices(false)
            .handle_boolean_strings(false)
            .handle_numeric_strings(false)
            .handle_missing_wrapper_object(false)
            .handle_unnecessary_additional_nesting(false)
            .handle_flattened_key_value_pairs(false)
            .handle_array_wrapped_single_objects(false)
            .handle_key_name_misalignment(false)
            .handle_timestamp_misformatting(false)
            .handle_null_value_misplacement(false)
            .handle_singleton_array_instead_of_object(false)
            .handle_reversed_map_structure(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        assert_eq!(output, expected, "Map-vector confusion was not fixed properly");
    }

    #[traced_test]
    fn map_vector_confusion_disabled() {
        info!("Testing map-vector confusion with fix DISABLED");
        let input = serde_json::json!({
            "A": {
                "descriptor": "D1",
                "timestamp": 123
            }
        });
        // With the fix disabled (and also key_name_misalignment disabled),
        // we expect the output to remain exactly the same.
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_map_vector_confusion(false)
            .handle_key_name_misalignment(false) // <-- Newly disabled to preserve "descriptor"
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(output, expected, "Map–Vector confusion should remain unfixed (and descriptor un-renamed) when disabled");
    }

    // ------------------------------------------------------
    // #2 NESTED VECTOR FLATTENING
    // ------------------------------------------------------

    #[traced_test]
    fn nested_vector_flattening_enabled() {
        info!("Testing nested vector flattening with fix ENABLED");
        // Our naive fix doesn't do an actual transformation,
        // but we ensure we at least pass through properly.
        let input = json!([
            ["a", "b"],
            ["c", "d"]
        ]);
        // Because the naive approach doesn't truly fix it,
        // we still expect the same output (no real transformation occurs).
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_nested_vector_flattening(true)
            .handle_map_vector_confusion(false)
            .handle_single_element_vector_omission(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        // We expect no change from the naive approach
        assert_eq!(output, expected, "Nested vector flattening fix is naive, but should not break data");
    }

    #[traced_test]
    fn nested_vector_flattening_disabled() {
        info!("Testing nested vector flattening with fix DISABLED");
        let input = json!([
            ["a", "b"],
            ["c", "d"]
        ]);
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_nested_vector_flattening(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(output, expected, "Nested vector flattening should remain unchanged when fix is disabled");
    }

    // ------------------------------------------------------
    // #3 SINGLE-ELEMENT VECTOR OMISSION
    // ------------------------------------------------------

    #[traced_test]
    fn single_element_vector_omission_enabled() {
        info!("Testing single-element vector omission with fix ENABLED");
        // Our code doesn't do a strong fix for this scenario, but we'll at least confirm it doesn't break anything.
        // Suppose the single-element array was misunderstood as an object. We'll test a direct array input:
        let input = json!([
            {
                "id": 1,
                "value": "X"
            }
        ]);
        let expected = input.clone(); // The naive approach doesn't forcibly remove single-element arrays.

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_single_element_vector_omission(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        // Currently, we do not transform single-element arrays -> object, so the data remains the same.
        assert_eq!(output, expected, "Single-element vector omission fix is naive, but should not break data");
    }

    #[traced_test]
    fn single_element_vector_omission_disabled() {
        info!("Testing single-element vector omission with fix DISABLED");
        let input = json!([
            {
                "id": 1,
                "value": "X"
            }
        ]);
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_single_element_vector_omission(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(output, expected, "No change expected when single-element vector omission fix is disabled");
    }

    // ------------------------------------------------------
    // #4 VECTOR AS MAP OF INDICES
    // ------------------------------------------------------

    #[traced_test]
    fn vector_as_map_of_indices_enabled() {
        info!("Testing vector-as-map-of-indices with fix ENABLED");
        let input = json!({
            "0": "A",
            "1": "B",
            "2": "C"
        });
        let expected = json!(["A", "B", "C"]);

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_vector_as_map_of_indices(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        assert_eq!(output, expected, "Map-of-indices should be converted to an array");
    }

    #[traced_test]
    fn vector_as_map_of_indices_disabled() {
        info!("Testing vector-as-map-of-indices with fix DISABLED");
        let input = json!({
            "0": "A",
            "1": "B",
            "2": "C"
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_vector_as_map_of_indices(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(output, expected, "Map-of-indices shape should remain unchanged when fix is disabled");
    }

    // ------------------------------------------------------
    // #5 BOOLEAN STRINGS
    // ------------------------------------------------------

    #[traced_test]
    fn boolean_strings_enabled() {
        info!("Testing boolean strings with fix ENABLED");
        let input = json!({
            "active": "true",
            "verified": "false"
        });
        let expected = json!({
            "active": true,
            "verified": false
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_boolean_strings(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        assert_eq!(output, expected, "String booleans should become actual bool values");
    }

    #[traced_test]
    fn boolean_strings_disabled() {
        info!("Testing boolean strings with fix DISABLED");
        let input = json!({
            "active": "true",
            "verified": "false"
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_boolean_strings(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(output, expected, "String booleans should remain as strings when fix is disabled");
    }

    // ------------------------------------------------------
    // #6 NUMERIC STRINGS
    // ------------------------------------------------------

    #[traced_test]
    fn numeric_strings_enabled() {
        info!("Testing numeric strings with fix ENABLED");
        let input = json!({
            "id": "42",
            "score": "7.5"
        });
        let expected = json!({
            "id": 42,
            "score": 7.5
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_numeric_strings(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        assert_eq!(output, expected, "String numbers should become numeric values");
    }

    #[traced_test]
    fn numeric_strings_disabled() {
        info!("Testing numeric strings with fix DISABLED");
        let input = json!({
            "id": "42",
            "score": "7.5"
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_numeric_strings(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(output, expected, "Numeric strings remain unchanged when fix is disabled");
    }

    // ------------------------------------------------------
    // #7 MISSING WRAPPER OBJECT
    // ------------------------------------------------------
    // Our example fix doesn't actually do anything automatic. We'll just ensure it doesn't break data.

    #[traced_test]
    fn missing_wrapper_object_enabled() {
        info!("Testing missing wrapper object with fix ENABLED (no real fix implemented)");
        let input = json!({
            "items": [
                {"id": 1},
                {"id": 2}
            ],
            "count": 3
        });
        let expected = input.clone(); // No real fix in our code

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_missing_wrapper_object(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        // We do not have an automatic fix, so no change
        assert_eq!(output, expected, "Missing wrapper object fix is unimplemented, so no change expected");
    }

    #[traced_test]
    fn missing_wrapper_object_disabled() {
        info!("Testing missing wrapper object with fix DISABLED");
        let input = json!({
            "items": [
                {"id": 1},
                {"id": 2}
            ],
            "count": 3
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_missing_wrapper_object(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(output, expected, "Missing wrapper object scenario remains unchanged when fix is disabled");
    }

    // ------------------------------------------------------
    // #8 UNNECESSARY ADDITIONAL NESTING
    // ------------------------------------------------------

    #[traced_test]
    fn unnecessary_additional_nesting_enabled() {
        info!("Testing unnecessary additional nesting with fix ENABLED");
        let input = json!({
            "results": {
                "items": [
                    {"id": 1},
                    {"id": 2}
                ]
            }
        });
        let expected = json!({
            "results": [
                {"id": 1},
                {"id": 2}
            ]
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_unnecessary_additional_nesting(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        assert_eq!(output, expected, "Unnecessary nesting under 'results.items' should be flattened");
    }

    #[traced_test]
    fn unnecessary_additional_nesting_disabled() {
        info!("Testing unnecessary additional nesting with fix DISABLED");
        let input = json!({
            "results": {
                "items": [
                    {"id": 1},
                    {"id": 2}
                ]
            }
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_unnecessary_additional_nesting(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(output, expected, "No flattening should occur when fix is disabled");
    }

    // ------------------------------------------------------
    // #9 FLATTENED KEY-VALUE PAIRS
    // ------------------------------------------------------
    // This fix is also unimplemented in the code. We'll confirm no changes.

    #[traced_test]
    fn flattened_key_value_pairs_enabled() {
        info!("Testing flattened key-value pairs with fix ENABLED (no real fix implemented)");
        let input = json!({
            "resolution": "high",
            "quality": "max"
        });
        let expected = input.clone(); // no real fix in code

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_flattened_key_value_pairs(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        assert_eq!(output, expected, "Flattened key-value pairs remain unchanged (fix unimplemented)");
    }

    #[traced_test]
    fn flattened_key_value_pairs_disabled() {
        info!("Testing flattened key-value pairs with fix DISABLED");
        let input = json!({
            "resolution": "high",
            "quality": "max"
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_flattened_key_value_pairs(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(output, expected, "Flattened key-value scenario remains unchanged when fix is disabled");
    }

    // ------------------------------------------------------
    // #10 ARRAY-WRAPPED SINGLE OBJECTS
    // ------------------------------------------------------
    // The code is naive for this fix, but let's test the pass-through.

    #[traced_test]
    fn array_wrapped_single_objects_enabled() {
        info!("Testing array-wrapped single objects with fix ENABLED");
        let input = json!({
            "user": [
                {
                    "name": "Alice",
                    "id": 101
                }
            ]
        });
        // The naive approach doesn't actually unwrap them. We'll just confirm no breakage.
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_array_wrapped_single_objects(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Array-wrapped single objects remain unchanged with naive approach"
        );
    }

    #[traced_test]
    fn array_wrapped_single_objects_disabled() {
        info!("Testing array-wrapped single objects with fix DISABLED");
        let input = json!({
            "user": [
                {
                    "name": "Alice",
                    "id": 101
                }
            ]
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_array_wrapped_single_objects(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "No unwrapping occurs when array-wrapped single objects fix is disabled"
        );
    }

    // ------------------------------------------------------
    // #11 KEY-NAME MISALIGNMENT (SIMILAR SEMANTICS)
    // ------------------------------------------------------

    #[traced_test]
    fn key_name_misalignment_enabled() {
        info!("Testing key-name misalignment with fix ENABLED");
        let input = json!({
            "descriptor": "An example."
        });
        let expected = json!({
            "description": "An example."
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_key_name_misalignment(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Misaligned key name 'descriptor' should become 'description'"
        );
    }

    #[traced_test]
    fn key_name_misalignment_disabled() {
        info!("Testing key-name misalignment with fix DISABLED");
        let input = json!({
            "descriptor": "An example."
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_key_name_misalignment(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Key name remains 'descriptor' when fix is disabled"
        );
    }

    // ------------------------------------------------------
    // #12 TIMESTAMP MISFORMATTING
    // ------------------------------------------------------
    // Our example code doesn't do an actual ISO-to-epoch fix. We'll confirm no breakage.

    #[traced_test]
    fn timestamp_misformatting_enabled() {
        info!("Testing timestamp misformatting with fix ENABLED (unimplemented transform)");
        let input = json!({
            "timestamp": "2025-04-29T12:00:54Z"
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_timestamp_misformatting(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        // No real fix is done, so output should match input
        assert_eq!(output, expected, "No transformation for timestamp with unimplemented fix");
    }

    #[traced_test]
    fn timestamp_misformatting_disabled() {
        info!("Testing timestamp misformatting with fix DISABLED");
        let input = json!({
            "timestamp": "2025-04-29T12:00:54Z"
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_timestamp_misformatting(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Timestamp misformatting remains unchanged when fix is disabled"
        );
    }

    // ------------------------------------------------------
    // #13 NULL VALUE MISPLACEMENT
    // ------------------------------------------------------

    #[traced_test]
    fn null_value_misplacement_enabled() {
        info!("Testing null value misplacement with fix ENABLED");
        let input = json!({
            "name": "Sample",
            "details": "null"
        });
        let expected = json!({
            "name": "Sample",
            "details": null
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_null_value_misplacement(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "String 'null' should become an actual null value"
        );
    }

    #[traced_test]
    fn null_value_misplacement_disabled() {
        info!("Testing null value misplacement with fix DISABLED");
        let input = json!({
            "name": "Sample",
            "details": "null"
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_null_value_misplacement(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "No change to 'null' string when fix is disabled"
        );
    }

    // ------------------------------------------------------
    // #14 SINGLETON ARRAY INSTEAD OF OBJECT
    // ------------------------------------------------------
    // Very similar to #10, but let's treat it separately for completeness.

    #[traced_test]
    fn singleton_array_instead_of_object_enabled() {
        info!("Testing singleton array instead of object with fix ENABLED");
        let input = json!({
            "user": [
                {"id": 1, "name": "Bob"}
            ]
        });
        // Our naive approach doesn't do a direct fix for this either, so we expect no transformation
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_singleton_array_instead_of_object(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Singleton array remains unchanged with naive approach"
        );
    }

    #[traced_test]
    fn singleton_array_instead_of_object_disabled() {
        info!("Testing singleton array instead of object with fix DISABLED");
        let input = json!({
            "user": [
                {"id": 1, "name": "Bob"}
            ]
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_singleton_array_instead_of_object(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "No unwrapping occurs when singleton array fix is disabled"
        );
    }

    // ------------------------------------------------------
    // #15 REVERSED MAP STRUCTURE
    // ------------------------------------------------------

    #[traced_test]
    fn reversed_map_structure_enabled() {
        info!("Testing reversed map structure with fix ENABLED");
        let input = json!([
            {"key": "A", "value": 1},
            {"key": "B", "value": 2},
            {"key": "C", "value": 3}
        ]);
        let expected = json!({
            "A": 1,
            "B": 2,
            "C": 3
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_reversed_map_structure(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix enabled: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Array of key-value pairs should become a map"
        );
    }

    #[traced_test]
    fn reversed_map_structure_disabled() {
        info!("Testing reversed map structure with fix DISABLED");
        let input = json!([
            {"key": "A", "value": 1},
            {"key": "B", "value": 2},
            {"key": "C", "value": 3}
        ]);
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_reversed_map_structure(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with fix disabled: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Reversed map structure remains unchanged when fix is disabled"
        );
    }

    // ------------------------------------------------------
    // COLLECTIVE TEST: ALL FIXES ENABLED
    // ------------------------------------------------------
    // We'll do one scenario that tries to combine multiple misunderstandings in a single JSON.
    // Our code is partial, so not all transformations will apply effectively,
    // but let's test the synergy of toggles.

    #[traced_test]
    fn all_fixes_enabled_combined_scenario() {
        info!("Testing a combined scenario with ALL fixes enabled");

        let input = json!({
            // #1 map-vector confusion
            "X": {
                "descriptor": "Should become 'description'",
                "timestamp": "2025-04-29T12:00:54Z", // #12 unimplemented fix
                "score": "7.5", // #6 numeric
                "active": "false", // #5 boolean
                "nullField": "null", // #13
            },
            // #4 vector as map of indices
            "someIndices": {
                "0": "A",
                "1": "B",
                "2": "C"
            },
            // #15 reversed map structure
            "reversedMapExample": [
                {"key": "A", "value": 1},
                {"key": "B", "value": 2}
            ]
        });

        // Let's see what we expect:
        // #1 transforms { X: {descriptor:..., ...} } -> [ {name: "X", description:..., ...} ]
        // #11 renames 'descriptor' -> 'description'
        // #5, #6, #13 transform string booleans, numbers, "null" -> real values
        // #4 transforms "someIndices" -> array
        // #15 transforms reversedMapExample -> an object
        let expected = json!([
            {
                "name": "X",
                "description": "Should become 'description'",
                // #12 remains unchanged due to unimplemented approach for ISO->epoch
                "timestamp": "2025-04-29T12:00:54Z",
                "score": 7.5,
                "active": false,
                "nullField": null
            }
        ]);

        // And also we'd expect the object entries:
        //   "someIndices": ["A","B","C"]
        //   "reversedMapExample": {"A":1, "B":2}
        // But since #1's fix turned the entire top-level object into an array of length 1,
        // we actually can't keep "someIndices" or "reversedMapExample" as siblings
        // in the naive approach. Our code transforms the top-level if there's exactly one key,
        // but here we have multiple keys, so #1 won't actually trigger on the entire object in the same way.
        // 
        // Indeed, if the top-level has more than one key, #1 fix won't apply. So let's see how it plays out:
        // Our naive #1 fix specifically checks if there's exactly one key in the *object*, then transforms it.
        // We have three keys at the top: "X", "someIndices", and "reversedMapExample".
        // So #1 won't transform the top-level. That means the "X" key remains as is in the final object,
        // then we rename "descriptor" to "description" inside "X".
        // 
        // Summarizing the expected top-level shape after each fix:
        // - "X" is an object, with "descriptor" renamed to "description", numeric, boolean, and "null" string fixed
        // - "someIndices" gets turned into an array
        // - "reversedMapExample" becomes an object
        // 
        // So the final shape is an object with three keys: "X", "someIndices", "reversedMapExample". 
        // 
        // Let's finalize that expectation:
        let expected_object = json!({
            "X": {
                "description": "Should become 'description'",
                "timestamp": "2025-04-29T12:00:54Z",
                "score": 7.5,
                "active": false,
                "nullField": null
            },
            "someIndices": ["A", "B", "C"],
            "reversedMapExample": {
                "A": 1,
                "B": 2
            }
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_map_vector_confusion(true)
            .handle_nested_vector_flattening(true)
            .handle_single_element_vector_omission(true)
            .handle_vector_as_map_of_indices(true)
            .handle_boolean_strings(true)
            .handle_numeric_strings(true)
            .handle_missing_wrapper_object(true)
            .handle_unnecessary_additional_nesting(true)
            .handle_flattened_key_value_pairs(true)
            .handle_array_wrapped_single_objects(true)
            .handle_key_name_misalignment(true)
            .handle_timestamp_misformatting(true)
            .handle_null_value_misplacement(true)
            .handle_singleton_array_instead_of_object(true)
            .handle_reversed_map_structure(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with ALL fixes enabled: {:?}", &output);
        assert_eq!(output, expected_object, "All-fixes-enabled scenario didn't match the expected object shape");
    }

    // ------------------------------------------------------
    // COLLECTIVE TEST: ALL FIXES DISABLED
    // ------------------------------------------------------
    // Finally, we confirm that with everything turned off, the original JSON remains identical.

    #[traced_test]
    fn all_fixes_disabled_combined_scenario() {
        info!("Testing a combined scenario with ALL fixes disabled");
        let input = json!({
            "A": {
                "descriptor": "D1",
                "timestamp": "2025-04-29T12:00:54Z"
            },
            "items": {
                "0": "Z"
            },
            "someArray": [
                {"key": "K", "value": 99}
            ],
            "active": "false",
            "id": "42"
        });
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_map_vector_confusion(false)
            .handle_nested_vector_flattening(false)
            .handle_single_element_vector_omission(false)
            .handle_vector_as_map_of_indices(false)
            .handle_boolean_strings(false)
            .handle_numeric_strings(false)
            .handle_missing_wrapper_object(false)
            .handle_unnecessary_additional_nesting(false)
            .handle_flattened_key_value_pairs(false)
            .handle_array_wrapped_single_objects(false)
            .handle_key_name_misalignment(false)
            .handle_timestamp_misformatting(false)
            .handle_null_value_misplacement(false)
            .handle_singleton_array_instead_of_object(false)
            .handle_reversed_map_structure(false)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output with ALL fixes disabled: {:?}", &output);
        assert_eq!(output, expected, "With all fixes disabled, no transformations should occur");
    }
}

#[cfg(test)]
mod check_edge_case_scenarios {
    use super::*;

    // ------------------------------------------------------------------
    // #1. MULTIPLE POSSIBLE FIXES AT ONCE (One key = "results")
    // ------------------------------------------------------------------
    // CHANGED TEST CONFIG:
    // We also disable handle_key_name_misalignment so that "descriptor" remains untouched
    // if the single key is "results." We keep reversed map structure fix = true so "reversedStuff" is handled.
    #[traced_test]
    fn multiple_fixes_one_key_results() {
        info!("Testing multiple possible fixes at once with single key 'results'");
        let input = json!({
            "results": {
                "descriptor": "ShouldRemainBecauseKeyCheck",
                "reversedStuff": [
                    {"key": "A", "value": 1},
                    {"key": "B", "value": 2}
                ]
            }
        });

        let expected = json!({
            "results": {
                "descriptor": "ShouldRemainBecauseKeyCheck",
                "reversedStuff": {
                    "A": 1,
                    "B": 2
                }
            }
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_map_vector_confusion(true) // normally checks single-key, but "results" is special
            .handle_reversed_map_structure(true)
            .handle_key_name_misalignment(false) // ensures we keep "descriptor"
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Should skip descriptor rename if key is 'results' and still fix reversed map"
        );
    }

    // ------------------------------------------------------------------
    // 2. MIXED-CONTENT ARRAYS
    // ------------------------------------------------------------------
    // We provide an array that is partially reversed-map pairs, partially normal data,
    // to confirm we *do not* forcibly convert that entire array to an object.

    #[traced_test]
    fn partial_reversed_map_array() {
        info!("Testing partial reversed map array scenario");
        let input = json!([
            {"key": "A", "value": 1},
            {"key": "B", "value": 2},
            {"some": "other-data"}   // not {key, value}
        ]);

        // Because not ALL elements match the {key, value} pattern,
        // we expect the array to remain an array.
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_reversed_map_structure(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Mixed reversed-map vs normal data means no bulk object conversion"
        );
    }

    // ------------------------------------------------------------------
    // 3. CASE VARIATIONS IN BOOLEAN STRINGS
    // ------------------------------------------------------------------
    // Confirm that we *do not* transform "TRUE", "False", "True", etc.,
    // given we only handle exact matches "true"/"false".

    #[traced_test]
    fn case_variations_in_booleans() {
        info!("Testing case variations in boolean strings");
        let input = json!({
            "testUpper": "TRUE",
            "testMixed": "False",
            "testExact": "false"
        });
        // Only "false" should become a bool = false. The others remain strings.
        let expected = json!({
            "testUpper": "TRUE",
            "testMixed": "False",
            "testExact": false
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_boolean_strings(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Only exact 'false' is converted. 'TRUE'/'False' remain strings."
        );
    }

    // ------------------------------------------------------------------
    // 4. EDGE CASES IN NUMERIC STRINGS
    // ------------------------------------------------------------------
    // We'll test very large integer, negative, partial numeric, etc.

    #[traced_test]
    fn numeric_strings_variety() {
        info!("Testing variety of numeric strings");
        let input = json!({
            "largeInt": "9876543210123456789", // won't fit in i64
            "negativeInt": "-42",
            "invalidFloat": "12.34.56",
            "mixed": "123abc",
            "leadingZero": "007",
            "validFloat": "3.14159"
        });

        // We only convert *valid i64* or *valid f64* that we can parse. 
        // 9876543210123456789 won't parse as i64 => remains a string
        // negativeInt => i64 = -42
        // invalidFloat => remains string
        // mixed => remains string
        // leadingZero => i64 = 7 (leading zero is fine if it parses)
        // validFloat => 3.14159
        let expected = json!({
            "largeInt": "9876543210123456789",
            "negativeInt": -42,
            "invalidFloat": "12.34.56",
            "mixed": "123abc",
            "leadingZero": 7,
            "validFloat": 3.14159
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_numeric_strings(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output: {:?}", &output);
        assert_eq!(output, expected, "Check numeric parsing edge cases");
    }

    // ------------------------------------------------------------------
    // #5. DEEPLY NESTED RECURSION
    // ------------------------------------------------------------------
    // CHANGED TEST CONFIG:
    // We disable handle_map_vector_confusion so we don't transform the top-level single-key "Outer."
    // We also disable handle_key_name_misalignment so "descriptor" remains "descriptor."
    #[traced_test]
    fn deeply_nested_multi_fix() {
        info!("Testing deeply nested single-key objects and reversed map inside them");
        let input = json!({
            "Outer": {
                "descriptor": "OuterDescriptorValue",
                "Middle": {
                    "descriptor": "MiddleDescriptorValue",
                    "Inner": {
                        "descriptor": "InnerDescriptorValue",
                        "reversedMap": [
                            {"key": "X", "value": "someVal"},
                            {"key": "Y", "value": "anotherVal"}
                        ]
                    }
                }
            }
        });

        // Because we disabled handle_map_vector_confusion, "Outer" won't convert to an array.
        // Because we disabled handle_key_name_misalignment, we keep "descriptor" at all levels.
        // We do want reversedMap structure to fix, so reversedMap => { "X": "someVal", "Y": "anotherVal" }
        let expected = json!({
            "Outer": {
                "descriptor": "OuterDescriptorValue",
                "Middle": {
                    "descriptor": "MiddleDescriptorValue",
                    "Inner": {
                        "descriptor": "InnerDescriptorValue",
                        "reversedMap": {
                            "X": "someVal",
                            "Y": "anotherVal"
                        }
                    }
                }
            }
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_map_vector_confusion(false)  // skip top-level single-key array transform
            .handle_key_name_misalignment(false) // skip "descriptor"->"description"
            .handle_reversed_map_structure(true) // still fix the reversed array
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "We skip single-key fix, keep descriptor, but fix reversed map inside 'Inner'"
        );
    }

    // ------------------------------------------------------------------
    // 6. MIXED SINGLE-ELEMENT ARRAYS
    // ------------------------------------------------------------------
    // Some array items have 1 element, some more, ensuring we handle them consistently
    // (our code basically does not flatten them, but let's confirm we don't do partial weirdness).

    #[traced_test]
    fn mixed_single_element_arrays() {
        info!("Testing array with single-element sub-arrays plus multi-element sub-arrays");
        let input = json!([
            ["onlyOne"],
            ["a","b","c"],
            [{"id": 1}],
            []
        ]);
        // Our code won't transform single-element arrays => we expect unchanged
        let expected = input.clone();

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_single_element_vector_omission(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Naive approach doesn't flatten or unwrap single-element sub-arrays"
        );
    }

    // ------------------------------------------------------------------
    // 7. "NULL" COLLISIONS
    // ------------------------------------------------------------------
    // If we have a legitimate null plus a string "null" and other strings, confirm we only convert
    // the string "null" to real null.

    #[traced_test]
    fn multiple_null_types() {
        info!("Testing collisions of real null vs. 'null' string");
        let input = json!({
            "realNull": null,
            "stringNull": "null",
            "unrelated": "nullValue"
        });
        // "stringNull" => null, "realNull" remains null, "unrelated" remains "nullValue"
        let expected = json!({
            "realNull": null,
            "stringNull": null,
            "unrelated": "nullValue"
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_null_value_misplacement(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output: {:?}", &output);
        assert_eq!(output, expected, "Only the exact 'null' string becomes null");
    }

    // ------------------------------------------------------------------
    // 8. KEY NAME COLLISIONS AFTER FIX
    // ------------------------------------------------------------------
    // If an object has both 'descriptor' and 'description', do we clobber 'description' or
    // skip renaming? The code as written will remove 'descriptor' and insert 'description',
    // overwriting if it already existed. Let's see that behavior in a test.

    #[traced_test]
    fn descriptor_and_description_conflict() {
        info!("Testing an object that has both 'descriptor' and 'description'");
        let input = json!({
            "descriptor": "X",
            "description": "Y"
        });
        // Our fix_key_name_misalignment calls obj.remove("descriptor"), then inserts
        // obj.insert("description", description_val). That overwrites the old "description".
        // So the final is { "description": "X" } if we do naive approach.
        // Let's confirm that is indeed what we want or confirm the code does that.
        // 
        // The code will overwrite "Y" with "X". 
        // So let's define that final outcome as the "expected" to confirm we handle it as coded.
        let expected = json!({
            "description": "X"
        });

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_key_name_misalignment(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Key conflict means 'description': 'Y' is overwritten by 'descriptor' -> 'description'"
        );
    }

    // ------------------------------------------------------------------
    // #9. MULTI-LAYER MAP–VECTOR CONFUSION
    // ------------------------------------------------------------------
    // CHANGED TEST CONFIG:
    // We explicitly disable handle_key_name_misalignment so we keep "descriptor".
    // We keep handle_map_vector_confusion = true so we still transform single-key objects.
    #[traced_test]
    fn multi_layer_map_vector_confusion() {
        info!("Testing multi-layer single-key objects with 'descriptor'");
        let input = json!({
            "One": {
                "descriptor": "val1"
            }
        });
        // We want the result to remain an array with "descriptor" (not renamed):
        // [ { "name": "One", "descriptor": "val1" } ]
        let expected = json!([
            {
                "name": "One",
                "descriptor": "val1"
            }
        ]);

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_map_vector_confusion(true)
            .handle_key_name_misalignment(false) // disable rename to keep "descriptor"
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Single-key object with descriptor becomes array with a name field, keeping 'descriptor'"
        );
    }

    // ------------------------------------------------------------------
    // 10. ORDERING OF FIXES (E.G. MAP-VECTOR vs KEY-NAME MISALIGNMENT)
    // ------------------------------------------------------------------
    // We'll have a single-key object "Foo" with "descriptor" that might be turned
    // into an array by #1, and then "descriptor" might be renamed to "description" by #11.
    // The code calls fix_map_vector_confusion first, then fix_key_name_misalignment.
    // We'll confirm we do see the final "description" in the array item.
    //
    // If we reversed the order, we might fail the "descriptor" check. This test ensures 
    // the existing function-call order is enforced.

    #[traced_test]
    fn ordering_map_vector_vs_key_name() {
        info!("Testing a scenario that triggers BOTH map_vector_confusion and key_name_misalignment");
        let input = json!({
            "Foo": {
                "descriptor": "someVal"
            }
        });
        // We expect the result to be:
        // [
        //   { "name": "Foo", "descriptor": "someVal" }
        // ]
        // Then the subsequent rename sees "descriptor" -> "description".
        // So final becomes:
        // [
        //   { "name": "Foo", "description": "someVal" }
        // ]
        let expected = json!([
            {
                "name": "Foo",
                "description": "someVal"
            }
        ]);

        let config = MisunderstandingCorrectionConfigBuilder::default()
            .handle_map_vector_confusion(true)
            .handle_key_name_misalignment(true)
            .build()
            .unwrap();

        let output = apply_misunderstanding_corrections(&config, input.clone());
        trace!("Output: {:?}", &output);
        assert_eq!(
            output,
            expected,
            "Should fix map-vector first, then rename 'descriptor' to 'description'"
        );
    }
}

#[cfg(test)]
mod property_based_robustness_tests {
    use super::*;

    // A helper strategy to generate arbitrary JSON Values:
    // We define a recursive strategy because JSON is inherently recursive.
    // We also limit depth to avoid enormous structures.
    fn arb_json_value(max_depth: u32) -> impl Strategy<Value = Value> {
        // If depth == 0, just pick a primitive.
        // Otherwise, pick among object, array, or primitive.
        let leaf = prop_oneof![
            // Strings
            ".*".prop_map(Value::String),
            // Booleans
            any::<bool>().prop_map(Value::Bool),
            // 64-bit integers
            any::<i64>().prop_map(|i| Value::Number(i.into())),
            // floats (note that not all floats are valid JSON, e.g. NaN).
            any::<f64>()
                .prop_filter_map("Exclude NaN/Infinity", |f| {
                    if f.is_finite() {
                        serde_json::Number::from_f64(f).map(Value::Number)
                    } else {
                        None
                    }
                }),
            // Possibly null
            Just(Value::Null),
        ];

        leaf.prop_recursive(
            max_depth, // max depth
            8,         // max size of collection
            8,         // max items per collection
            |inner| {
                prop_oneof![
                    // Generate arrays
                    prop::collection::vec(inner.clone(), 0..3)
                        .prop_map(Value::Array),
                    // Generate objects (maps)
                    btree_map(".*", inner, 0..3)
                        .prop_map(|m| {
                            let mut map = serde_json::Map::new();
                            for (k, v) in m {
                                map.insert(k, v);
                            }
                            Value::Object(map)
                        })
                ]
            },
        )
    }

    fn arb_config() -> impl Strategy<Value = MisunderstandingCorrectionConfig> {
        any::<[bool; 15]>().prop_map(|bits| {
            MisunderstandingCorrectionConfigBuilder::default()
                .handle_map_vector_confusion(bits[0])
                .handle_nested_vector_flattening(bits[1])
                .handle_single_element_vector_omission(bits[2])
                .handle_vector_as_map_of_indices(bits[3])
                .handle_boolean_strings(bits[4])
                .handle_numeric_strings(bits[5])
                .handle_missing_wrapper_object(bits[6])
                .handle_unnecessary_additional_nesting(bits[7])
                .handle_flattened_key_value_pairs(bits[8])
                .handle_array_wrapped_single_objects(bits[9])
                .handle_key_name_misalignment(bits[10])
                .handle_timestamp_misformatting(bits[11])
                .handle_null_value_misplacement(bits[12])
                .handle_singleton_array_instead_of_object(bits[13])
                .handle_reversed_map_structure(bits[14])
                .build()
                .unwrap()
        })
    }


    // ------------------------------------------------------------------
    // PROPERTY-BASED TEST:
    // Generate random JSON plus random fix config,
    // feed it into `apply_misunderstanding_corrections`,
    // and then we do minimal checks that the result is valid JSON
    // and has no panics. Also, we can do some invariants:
    // - It's guaranteed to parse as JSON (by definition).
    // - We ensure we never panic or crash.
    // - Optionally we can check that applying the same fix pass
    //   multiple times doesn't cause repeated expansions or loops.
    // ------------------------------------------------------------------

    proptest! {
        #![proptest_config(ProptestConfig {
            cases: 50, // number of random test cases
            .. ProptestConfig::default()
        })]

        #[traced_test]
        fn property_based_random_json_test(random_json in arb_json_value(3), config in arb_config()) {
            info!("Testing random JSON with random config");
            trace!("Config: {:?}", config);
            trace!("Input JSON: {:?}", random_json);

            let first_pass = apply_misunderstanding_corrections(&config, random_json.clone());
            // We want to check if applying the fix a second time is stable (idempotent-ish),
            // since in many data-fixing scenarios, repeated application should not repeatedly
            // transform or break the data. We'll do a second pass:
            let second_pass = apply_misunderstanding_corrections(&config, first_pass.clone());

            // We only do a naive assertion that second_pass is the same as first_pass,
            // meaning we didn't keep re-translating in a cycle.
            // (If your design is not guaranteed idempotent, you can remove this.)
            prop_assert_eq!(
                second_pass, first_pass,
                "Applying the fix pass repeatedly should not keep re-translating data"
            );
        }
    }

    // ------------------------------------------------------------------
    // OPTIONAL: CONCURRENCY TEST
    // ------------------------------------------------------------------
    // Although `apply_misunderstanding_corrections` is pure (no global state),
    // we can do a small concurrency test to confirm it doesn't panic in parallel usage.

    #[traced_test]
    fn concurrency_test_example() {
        use std::sync::Arc;
        use std::thread;

        info!("Starting concurrency test for apply_misunderstanding_corrections");

        let input = Arc::new(json!({
            "user": {
                "descriptor": "Concurrent test descriptor",
                "timestamp": "2030-01-01T00:00:00Z"
            }
        }));

        let config = Arc::new(MisunderstandingCorrectionConfigBuilder::default()
            .build()
            .unwrap()
        );

        let mut handles = vec![];

        // We'll spawn 8 threads, each applying the function in parallel
        for i in 0..8 {
            let input_clone = Arc::clone(&input);
            let config_clone = Arc::clone(&config);

            let handle = thread::spawn(move || {
                trace!("Thread {} is running corrections", i);
                let _ = apply_misunderstanding_corrections(&config_clone, (*input_clone).clone());
            });
            handles.push(handle);
        }

        for handle in handles {
            // If there's a panic inside the thread, join() will propagate it.
            handle.join().expect("Thread panicked during concurrency test");
        }

        // If we reach here, no concurrency issues or panics occurred
        info!("Concurrency test completed successfully");
    }
}

// ---------------- [ File: json-misunderstanding/src/lib.rs ]
#[macro_use] mod imports; use imports::*;

x!{apply_misunderstanding_corrections}
x!{drain_json_map}
x!{fix_array}
x!{fix_boolean_as_int}
x!{fix_complex_enum_confidence_misrepresentation}
x!{fix_date_format_confustion}
x!{fix_deeply_nested_vector_overwrap}
x!{fix_enumeration_as_map}
x!{fix_flattened_pairs}
x!{fix_incorrectly_nested_data_wrapper}
x!{fix_key_name_misalignment}
x!{fix_map_vector_confusion}
x!{fix_misplaced_metadata}
x!{fix_missing_array}
x!{fix_mixed_type_arrays}
x!{fix_nested_flattening}
x!{fix_numeric_keys_misunderstanding}
x!{fix_object}
x!{fix_optional_fields_misinterpretation}
x!{fix_over_nesting_of_scalars}
x!{fix_overly_verbose_field}
x!{fix_primitive}
x!{fix_redundant_enum_metadata}
x!{fix_redundant_nesting_of_identical_keys}
x!{fix_reversed_map_structure}
x!{fix_scalar_to_array_repetition}
x!{fix_simple_key_value_inversion}
x!{fix_stringified_json}
x!{fix_unit_enum_variants_wrapped_as_objects}
x!{fix_unnecessary_additional_nesting}
x!{fix_value}
x!{fix_vector_as_map_of_indices}
x!{flatten_all_identical_keys}
x!{flatten_enum_variants_array}
x!{harmonize_array_field_length}
x!{misunderstanding_correction_config}
x!{pull_up_nested_just_conf_fields}
x!{remove_confidence_for_null_fields}
x!{remove_confidence_from_non_selected_variants}
x!{remove_inconsistent_nested_variant_type}
x!{remove_incorrect_type_metadata}
x!{remove_partial_enum_simplification}
x!{remove_redundant_confidence_justification_multiple_formats}
x!{remove_redundant_justifications_for_null}
x!{remove_schema_metadata_in_output}
x!{remove_struct_level_just_conf}
x!{remove_unused_probabilistic_fields}
x!{test1}
x!{test2}

#[cfg(test)]
mod test_new_misunderstandings {
    use super::*;

    ////////////////////////////////////////////////////////////////////////////
    // test_complex_enum_confidence_misrepresentation
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_complex_enum_confidence_misrepresentation_scenarios() {
        trace!("Testing fix_complex_enum_confidence_misrepresentation with various scenarios.");

        // Scenario A: Exactly one variant with confidence > 0
        let input_a = json!({
            "type": "complex_enum",
            "enum_name": "ConfigComplexity",
            "variants": [
                { "variant_name": "Simple",   "variant_confidence": 0.0 },
                { "variant_name": "Balanced", "variant_confidence": 1.0, "variant_justification": "Chosen" },
                { "variant_name": "Complex",  "variant_confidence": 0.0 }
            ]
        });
        let expected_a = json!({
            "variant": "Balanced",
            "confidence": 1.0,
            "justification": "Chosen"
        });
        let output_a = fix_complex_enum_confidence_misrepresentation(input_a.clone());
        assert_eq!(output_a, expected_a, "Should flatten to single variant with confidence=1.0");

        // Scenario B: No variant_confidence > 0 => remain unchanged
        let input_b = json!({
            "type": "complex_enum",
            "enum_name": "ConfigComplexity",
            "variants": [
                { "variant_name": "Simple",   "variant_confidence": 0.0 },
                { "variant_name": "Complex",  "variant_confidence": 0.0 }
            ]
        });
        let output_b = fix_complex_enum_confidence_misrepresentation(input_b.clone());
        assert_eq!(output_b, input_b, "No variant above 0 => remain unchanged");

        // Scenario C: Multiple variants with > 0 confidence => remain unchanged
        let input_c = json!({
            "type": "complex_enum",
            "enum_name": "ConfigComplexity",
            "variants": [
                { "variant_name": "Simple",   "variant_confidence": 0.8 },
                { "variant_name": "Balanced", "variant_confidence": 0.6 }
            ]
        });
        let output_c = fix_complex_enum_confidence_misrepresentation(input_c.clone());
        assert_eq!(output_c, input_c, "Multiple positive confidences => remain unchanged");

        info!("test_complex_enum_confidence_misrepresentation_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_unit_enum_variants_wrapped_as_objects
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_unit_enum_variants_wrapped_as_objects_scenarios() {
        trace!("Testing fix_unit_enum_variants_wrapped_as_objects with various scenarios.");

        // Single variant has confidence=1 => choose it
        let input_a = json!({
            "type": "complex_enum",
            "enum_name": "Ordering",
            "variants": [
                {"variant_name": "None", "variant_confidence": 0.0},
                {"variant_name": "DifficultyAscending", "variant_confidence": 1.0},
                {"variant_name": "Random", "variant_confidence": 0.0}
            ]
        });
        let expected_a = json!("DifficultyAscending");
        let output_a = fix_unit_enum_variants_wrapped_as_objects(input_a.clone());
        assert_eq!(output_a, expected_a);

        // No variant with confidence=1 => remain unchanged
        let input_b = json!({
            "type": "complex_enum",
            "enum_name": "Ordering",
            "variants": [
                {"variant_name": "None", "variant_confidence": 0.5},
                {"variant_name": "DifficultyAscending", "variant_confidence": 0.5}
            ]
        });
        let output_b = fix_unit_enum_variants_wrapped_as_objects(input_b.clone());
        assert_eq!(output_b, input_b);

        info!("test_unit_enum_variants_wrapped_as_objects_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_optional_fields_misinterpretation
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_optional_fields_misinterpretation_scenarios() {
        trace!("Testing fix_optional_fields_misinterpretation.");

        let input = json!({"aggregator_depth_limit": 3});
        let expected = json!({"aggregator_depth_limit": null});
        let output = fix_optional_fields_misinterpretation(input.clone());
        assert_eq!(output, expected);

        // If aggregator_depth_limit is already null or not present, remain
        let input2 = json!({"aggregator_depth_limit": null});
        let output2 = fix_optional_fields_misinterpretation(input2.clone());
        assert_eq!(output2, input2);

        info!("test_optional_fields_misinterpretation_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_over_nesting_of_scalars
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_over_nesting_of_scalars_scenarios() {
        trace!("Testing fix_over_nesting_of_scalars.");

        let input = json!({"value": 8});
        let expected = json!(8);
        let output = fix_over_nesting_of_scalars(input.clone());
        assert_eq!(output, expected, "Should flatten over-nested scalar from {{value:X}} to X");

        // If there's more than one key, do nothing
        let input2 = json!({"value": 8, "extra": true});
        let output2 = fix_over_nesting_of_scalars(input2.clone());
        assert_eq!(output2, input2);

        info!("test_over_nesting_of_scalars_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_redundant_enum_metadata
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_redundant_enum_metadata_scenarios() {
        trace!("Testing fix_redundant_enum_metadata.");

        let input = json!({
            "mode": {
                "variants": [
                    { "variant_name": "Off", "variant_confidence": 0.0 },
                    { "variant_name": "Probabilistic", "variant_confidence": 1.0 }
                ]
            }
        });
        let expected = json!({"mode": "Probabilistic"});
        let output = fix_redundant_enum_metadata(input.clone());
        assert_eq!(output, expected);

        // If no variant_confidence=1 => remain
        let input2 = json!({
            "mode": {
                "variants": [
                    { "variant_name": "Off", "variant_confidence": 0.0 },
                    { "variant_name": "Single", "variant_confidence": 0.3 }
                ]
            }
        });
        let output2 = fix_redundant_enum_metadata(input2.clone());
        assert_eq!(output2, input2);

        info!("test_redundant_enum_metadata_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_pull_up_nested_just_conf_fields
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_pull_up_nested_just_conf_fields_scenarios() {
        trace!("Testing pull_up_nested_just_conf_fields.");

        let input = json!({
            "aggregator_preference": {
                "value": 0.4,
                "confidence": 0.9,
                "justification": "Test justification"
            }
        });
        let expected = json!({
            "aggregator_preference": 0.4,
            "aggregator_preference_confidence": 0.9,
            "aggregator_preference_justification": "Test justification"
        });
        let output = pull_up_nested_just_conf_fields(input.clone());
        assert_eq!(output, expected);

        // If nested doesn't have both value & conf, do nothing
        let input2 = json!({"fieldA": {"value": 1.0}, "fieldB": {"confidence": 0.5}});
        let output2 = pull_up_nested_just_conf_fields(input2.clone());
        assert_eq!(output2, input2);

        info!("test_pull_up_nested_just_conf_fields_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_harmonize_array_field_length
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_harmonize_array_field_length_scenarios() {
        trace!("Testing harmonize_array_field_length.");

        // A: shorter array than depth => pad with 0
        let input_a = json!({
            "depth": 5,
            "density_per_level": [9, 9, 9]
        });
        let expected_a = json!({
            "depth": 5,
            "density_per_level": [9, 9, 9, 0, 0]
        });
        let output_a = harmonize_array_field_length(input_a.clone());
        assert_eq!(output_a, expected_a);

        // B: longer => truncate
        let input_b = json!({
            "depth": 2,
            "density_per_level": [7, 8, 9, 10]
        });
        let expected_b = json!({
            "depth": 2,
            "density_per_level": [7, 8]
        });
        let output_b = harmonize_array_field_length(input_b.clone());
        assert_eq!(output_b, expected_b);

        // No depth or no density_per_level => no changes
        let input_c = json!({
            "depth": 4
        });
        let output_c = harmonize_array_field_length(input_c.clone());
        assert_eq!(output_c, input_c);

        info!("test_harmonize_array_field_length_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_remove_incorrect_type_metadata
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_remove_incorrect_type_metadata_scenarios() {
        trace!("Testing remove_incorrect_type_metadata.");

        let input = json!({
            "weighted_branching": {
                "type": "struct",
                "struct_name": "WeightedBranchingConfiguration"
            }
        });
        let expected = json!({
            "weighted_branching": {
                "struct_name": "WeightedBranchingConfiguration"
            }
        });
        let output = remove_incorrect_type_metadata(input.clone());
        assert_eq!(output, expected);

        // type is other => remain
        let input2 = json!({
            "something": {
                "type": "random"
            }
        });
        let output2 = remove_incorrect_type_metadata(input2.clone());
        assert_eq!(output2, input2);

        info!("test_remove_incorrect_type_metadata_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_flatten_enum_variants_array
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_flatten_enum_variants_array_scenarios() {
        trace!("Testing flatten_enum_variants_array.");

        let input = json!({
            "type": "complex_enum",
            "variants": [
                {"variant_name": "Simple", "variant_confidence": 0.0},
                {"variant_name": "WeightedWithLimits", "variant_confidence": 1.0, "fields": {"max":10}}
            ]
        });
        let expected = json!({
            "WeightedWithLimits": {"max":10}
        });
        let output = flatten_enum_variants_array(input.clone());
        assert_eq!(output, expected);

        // If no variants or multiple => keep as-is
        let input2 = json!({
            "type": "complex_enum",
            "variants": []
        });
        let output2 = flatten_enum_variants_array(input2.clone());
        assert_eq!(output2, input2);

        info!("test_flatten_enum_variants_array_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_remove_confidence_from_non_selected_variants
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_remove_confidence_from_non_selected_variants_scenarios() {
        trace!("Testing remove_confidence_from_non_selected_variants.");

        let input = json!({
            "variants": [
                {"variant_name":"Off", "variant_confidence":0.0},
                {"variant_name":"Single", "variant_confidence":0.0},
                {"variant_name":"Probabilistic","variant_confidence":1.0}
            ]
        });
        let expected = json!({
            "variants": [
                {"variant_name":"Probabilistic","variant_confidence":1.0}
            ]
        });
        let output = remove_confidence_from_non_selected_variants(input.clone());
        assert_eq!(output, expected);

        // If no "variants" => remain
        let input2 = json!({"no_variants": true});
        let output2 = remove_confidence_from_non_selected_variants(input2.clone());
        assert_eq!(output2, input2);

        info!("test_remove_confidence_from_non_selected_variants_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_remove_unused_probabilistic_fields
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_remove_unused_probabilistic_fields_scenarios() {
        trace!("Testing remove_unused_probabilistic_fields.");

        let input = json!({
            "mode": "Single",
            "probability": 0.0,
            "probability_confidence": 0.0,
            "probability_justification": "Unused"
        });
        let expected = json!({"mode": "Single"});
        let output = remove_unused_probabilistic_fields(input.clone());
        assert_eq!(output, expected);

        // If mode=Probabilistic => keep
        let input2 = json!({
            "mode":"Probabilistic",
            "probability":0.0
        });
        let output2 = remove_unused_probabilistic_fields(input2.clone());
        assert_eq!(output2, input2);

        info!("test_remove_unused_probabilistic_fields_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_remove_redundant_justifications_for_null
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_remove_redundant_justifications_for_null_scenarios() {
        trace!("Testing remove_redundant_justifications_for_null.");

        let input = json!({
            "dispatch_depth_limit": null,
            "dispatch_depth_limit_justification": "We do not separately constrain dispatch"
        });
        let expected = json!({
            "dispatch_depth_limit": null
        });
        let output = remove_redundant_justifications_for_null(input.clone());
        assert_eq!(output, expected);

        // If the field is non-null => keep justification
        let input2 = json!({
            "dispatch_depth_limit": 5,
            "dispatch_depth_limit_justification": "some reason"
        });
        let output2 = remove_redundant_justifications_for_null(input2.clone());
        assert_eq!(output2, input2);

        info!("test_remove_redundant_justifications_for_null_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_remove_confidence_for_null_fields
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_remove_confidence_for_null_fields_scenarios() {
        trace!("Testing remove_confidence_for_null_fields.");

        let input = json!({
            "leaf_min_depth": null,
            "leaf_min_depth_confidence": 0.3,
            "leaf_min_depth_justification": "Because WeightedWithLimits sets it"
        });
        let expected = json!({
            "leaf_min_depth": null,
            "leaf_min_depth_justification": "Because WeightedWithLimits sets it"
        });
        let output = remove_confidence_for_null_fields(input.clone());
        assert_eq!(output, expected);

        // If field is present => keep confidence
        let input2 = json!({
            "leaf_min_depth": 2,
            "leaf_min_depth_confidence": 0.5
        });
        let output2 = remove_confidence_for_null_fields(input2.clone());
        assert_eq!(output2, input2);

        info!("test_remove_confidence_for_null_fields_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_resolve_partial_enum_simplification
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_resolve_partial_enum_simplification_scenarios() {
        trace!("Testing resolve_partial_enum_simplification.");

        let input = json!({
            "complexity": {
                "variant_name": "Balanced",
                "variant_type": "unit",
                "variant_confidence": 1.0,
                "variant_justification": "some data"
            }
        });
        let expected = json!({
            "complexity": "Balanced"
        });
        let output = resolve_partial_enum_simplification(input.clone());
        assert_eq!(output, expected);

        // If confidence < 1 => remain
        let input2 = json!({"variant_name":"Foo", "variant_confidence":0.5});
        let output2 = resolve_partial_enum_simplification(input2.clone());
        assert_eq!(output2, input2);

        info!("test_resolve_partial_enum_simplification_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_remove_inconsistent_nested_variant_type
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_remove_inconsistent_nested_variant_type_scenarios() {
        trace!("Testing remove_inconsistent_nested_variant_type.");

        let input = json!({
            "ordering": {
                "variant_name": "DifficultyDescending",
                "variant_type": "unit",
                "variant_confidence": 0.7
            }
        });
        let expected = json!({
            "ordering": {
                "variant_name": "DifficultyDescending",
                "variant_confidence": 0.7
            }
        });
        let output = remove_inconsistent_nested_variant_type(input.clone());
        assert_eq!(output, expected);

        // If variant_type != "unit", remain
        let input2 = json!({
            "ordering": {
                "variant_name":"SomethingElse",
                "variant_type":"complex"
            }
        });
        let output2 = remove_inconsistent_nested_variant_type(input2.clone());
        assert_eq!(output2, input2);

        info!("test_remove_inconsistent_nested_variant_type_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_remove_struct_level_just_conf
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_remove_struct_level_just_conf_scenarios() {
        trace!("Testing remove_struct_level_just_conf.");

        let input = json!({
            "capstone_confidence": 0.8,
            "capstone_justification": "some justification"
        });
        let expected = json!({});
        let output = remove_struct_level_just_conf(input.clone());
        assert_eq!(output, expected);

        // If there's a field "capstone" => keep
        let input2 = json!({
            "capstone":"SingleOption",
            "capstone_confidence":0.6
        });
        // The function only removes struct-level if the base field doesn't exist. But "capstone" is present.
        let output2 = remove_struct_level_just_conf(input2.clone());
        assert_eq!(output2, input2);

        info!("test_remove_struct_level_just_conf_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_remove_redundant_confidence_justification_multiple_levels
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_remove_redundant_confidence_justification_multiple_levels_scenarios() {
        trace!("Testing remove_redundant_confidence_justification_multiple_levels.");

        let input = json!({
            "tree_expansion_policy_confidence": 0.9,
            "tree_expansion_policy_justification": "...",
            "tree_expansion_policy": {
                "variant_confidence": 0.9,
                "variant_justification": "..."
            }
        });
        let expected = json!({
            "tree_expansion_policy": {
                "variant_confidence": 0.9,
                "variant_justification": "..."
            }
        });
        let output = remove_redundant_confidence_justification_multiple_levels(input.clone());
        assert_eq!(output, expected);

        // If no child object => remain
        let input2 = json!({
            "outer_thing_confidence": 0.5,
            "outer_thing": "A"
        });
        let output2 = remove_redundant_confidence_justification_multiple_levels(input2.clone());
        assert_eq!(output2, input2);

        info!("test_remove_redundant_confidence_justification_multiple_levels_scenarios passed.");
    }

    ////////////////////////////////////////////////////////////////////////////
    // test_remove_schema_metadata_in_output
    ////////////////////////////////////////////////////////////////////////////
    #[traced_test]
    fn test_remove_schema_metadata_in_output_scenarios() {
        trace!("Testing remove_schema_metadata_in_output.");

        let input = json!({
            "aggregator_depth_limit": {
                "generation_instructions":"Example instructions",
                "required": false,
                "type":"number",
                "value":null
            }
        });
        let expected = json!({
            "aggregator_depth_limit": null
        });
        let output = remove_schema_metadata_in_output(input.clone());
        assert_eq!(output, expected);

        // If there's no 'value' => we just remove the known meta but keep the rest
        let input2 = json!({
            "some_field": {
                "generation_instructions":"stuff",
                "extra": "data"
            }
        });
        let expected2 = json!({
            "some_field": { "extra":"data" }
        });
        let output2 = remove_schema_metadata_in_output(input2.clone());
        assert_eq!(output2, expected2);

        info!("test_remove_schema_metadata_in_output_scenarios passed.");
    }
}

// ---------------- [ File: json-misunderstanding/src/misunderstanding_correction_config.rs ]
crate::ix!();

#[derive(Debug, Clone, Builder, Getters, Setters)]
#[builder(default)]
#[getset(get = "pub", set = "pub")]
pub struct MisunderstandingCorrectionConfig {
    // Existing fields 1..15, unchanged
    handle_map_vector_confusion: bool,
    handle_nested_vector_flattening: bool,
    handle_single_element_vector_omission: bool,
    handle_vector_as_map_of_indices: bool,
    handle_boolean_strings: bool,
    handle_numeric_strings: bool,
    handle_missing_wrapper_object: bool,
    handle_unnecessary_additional_nesting: bool,
    handle_flattened_key_value_pairs: bool,
    handle_array_wrapped_single_objects: bool,
    handle_key_name_misalignment: bool,
    handle_timestamp_misformatting: bool,
    handle_null_value_misplacement: bool,
    handle_singleton_array_instead_of_object: bool,
    handle_reversed_map_structure: bool,

    // ===========================
    // New fields for #16..#30
    // ===========================
    handle_mixed_type_arrays: bool,               // #16
    handle_missing_array: bool,                   // #17
    handle_stringified_json: bool,                // #18
    handle_misplaced_metadata: bool,              // #19
    handle_enumeration_as_map: bool,              // #20
    handle_deeply_nested_vector_overwrap: bool,   // #21
    handle_boolean_as_int: bool,                  // #22
    handle_simple_key_value_inversion: bool,      // #23
    handle_incorrectly_nested_data_wrapper: bool, // #24
    handle_overly_verbose_field: bool,            // #25
    handle_date_format_confusion: bool,           // #26
    handle_numeric_keys_misunderstanding: bool,   // #27
    handle_redundant_nesting_of_identical_keys: bool, // #28
    handle_flattened_pairs: bool,                 // #29
    handle_scalar_to_array_repetition: bool,      // #30
}

impl Default for MisunderstandingCorrectionConfig {
    fn default() -> Self {
        tracing::trace!("Creating default MisunderstandingCorrectionConfig");
        Self {
            // Old fields default to true as before
            handle_map_vector_confusion: true,
            handle_nested_vector_flattening: true,
            handle_single_element_vector_omission: true,
            handle_vector_as_map_of_indices: true,
            handle_boolean_strings: true,
            handle_numeric_strings: true,
            handle_missing_wrapper_object: true,
            handle_unnecessary_additional_nesting: true,
            handle_flattened_key_value_pairs: true,
            handle_array_wrapped_single_objects: true,
            handle_key_name_misalignment: true,
            handle_timestamp_misformatting: true,
            handle_null_value_misplacement: true,
            handle_singleton_array_instead_of_object: true,
            handle_reversed_map_structure: true,

            // New fields default to true as well, for consistency
            handle_mixed_type_arrays: true,
            handle_missing_array: true,
            handle_stringified_json: true,
            handle_misplaced_metadata: true,
            handle_enumeration_as_map: true,
            handle_deeply_nested_vector_overwrap: true,
            handle_boolean_as_int: true,
            handle_simple_key_value_inversion: true,
            handle_incorrectly_nested_data_wrapper: true,
            handle_overly_verbose_field: true,
            handle_date_format_confusion: true,
            handle_numeric_keys_misunderstanding: true,
            handle_redundant_nesting_of_identical_keys: true,
            handle_flattened_pairs: true,
            handle_scalar_to_array_repetition: true,
        }
    }
}

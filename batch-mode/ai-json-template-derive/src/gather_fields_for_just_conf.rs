// ---------------- [ File: ai-json-template-derive/src/gather_fields_for_just_conf.rs ]
crate::ix!();

pub fn gather_fields_for_just_conf(
    named_fields: &syn::FieldsNamed,
) -> (
    Vec<proc_macro2::TokenStream>, // justification fields
    Vec<proc_macro2::TokenStream>, // confidence fields
    proc_macro2::TokenStream,      // accumulated errors
    Vec<FieldJustConfMapping>,
) {
    trace!("Gathering fields for justification/conf from struct's named fields");

    let mut justification_struct_fields = Vec::new();
    let mut confidence_struct_fields = Vec::new();
    let mut errs = quote::quote!();
    let mut field_mappings = Vec::new();

    gather_justification_and_confidence_fields(
        named_fields,
        &mut justification_struct_fields,
        &mut confidence_struct_fields,
        &mut errs,
        &mut field_mappings,
    );

    debug!(
        "Completed gathering: justification_struct_fields={}, confidence_struct_fields={}, mappings={}",
        justification_struct_fields.len(),
        confidence_struct_fields.len(),
        field_mappings.len()
    );

    (justification_struct_fields, confidence_struct_fields, errs, field_mappings)
}

#[cfg(test)]
mod test_exhaustive_gather_fields_for_just_conf {
    use super::*;

    #[traced_test]
    fn it_handles_empty_named_fields() {
        trace!("Starting test: it_handles_empty_named_fields");
        let named: FieldsNamed = parse_quote! {
            {}
        };
        debug!("Parsed empty FieldsNamed: {:?}", named);

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        assert!(just_fields.is_empty(), "Expected no justification fields");
        assert!(conf_fields.is_empty(), "Expected no confidence fields");
        assert!(errs.is_empty(), "Expected no accumulated errors");
        assert!(mappings.is_empty(), "Expected no field mappings");
    }

    #[traced_test]
    fn it_gathers_single_field_with_default_justification() {
        trace!("Starting test: it_gathers_single_field_with_default_justification");
        let named: FieldsNamed = parse_quote! {
            {
                alpha: String
            }
        };
        debug!("Parsed single field named 'alpha' of type String");

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        // We expect 1 justification field => alpha_justification
        // We expect 1 confidence field => alpha_confidence
        assert_eq!(just_fields.len(), 1, "Expected exactly 1 justification field");
        assert_eq!(conf_fields.len(), 1, "Expected exactly 1 confidence field");
        assert!(errs.is_empty(), "Expected no errors");
        assert_eq!(mappings.len(), 1, "Expected exactly 1 field mapping");

        // Check the single mapping has the correct field name
        let mapping = &mappings[0];
        assert_eq!(mapping.field_ident().to_string(), "alpha");
        assert_eq!(mapping.justification_field_ident().to_string(), "alpha_justification");
        assert_eq!(mapping.confidence_field_ident().to_string(), "alpha_confidence");
    }

    #[traced_test]
    fn it_skips_field_when_justify_false() {
        trace!("Starting test: it_skips_field_when_justify_false");
        let named: FieldsNamed = parse_quote! {
            {
                #[justify = false]
                beta: i32,
                gamma: bool
            }
        };
        debug!("Parsed fields: beta(i32) with #[justify=false], gamma(bool) default");

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        // 'beta' is skipped from justification/conf due to #[justify=false]
        // 'gamma' is included
        assert_eq!(just_fields.len(), 1, "Expected justification only for gamma");
        assert_eq!(conf_fields.len(), 1, "Expected confidence only for gamma");
        assert!(errs.is_empty(), "Expected no errors");
        assert_eq!(mappings.len(), 1, "Expected only gamma in field mappings");

        // The single mapping should be 'gamma'
        let mapping = &mappings[0];
        assert_eq!(mapping.field_ident().to_string(), "gamma");
        assert_eq!(mapping.justification_field_ident().to_string(), "gamma_justification");
        assert_eq!(mapping.confidence_field_ident().to_string(), "gamma_confidence");
    }

    #[traced_test]
    fn it_accumulates_error_for_unsupported_type() {
        trace!("Starting test: it_accumulates_error_for_unsupported_type");
        // We are using "BadType" here to trigger the compile_error path in classify_for_justification
        let named: FieldsNamed = parse_quote! {
            {
                delta: BadType,
                epsilon: String
            }
        };
        debug!("Parsed fields: delta(BadType), epsilon(String)");

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        // We expect an error for BadType
        assert!(!errs.is_empty(), "Expected an error for 'BadType'");
        // 'delta' won't appear in the final expansions
        // 'epsilon' is a normal justification field
        assert_eq!(just_fields.len(), 1, "Expected justification field only for epsilon");
        assert_eq!(conf_fields.len(), 1, "Expected confidence field only for epsilon");
        assert_eq!(mappings.len(), 1, "Expected a single field mapping for epsilon");
    }

    #[traced_test]
    fn it_handles_multiple_fields_mixed() {
        trace!("Starting test: it_handles_multiple_fields_mixed");
        // Some fields are normal, some have justification turned off, some might produce errors
        let named: FieldsNamed = parse_quote! {
            {
                alpha: i32,
                #[justify=false]
                beta: bool,
                gamma: Option<BadType>,
                delta: String
            }
        };
        debug!("Parsed fields: alpha(i32), beta(bool #[justify=false]), gamma(Option<BadType>), delta(String)");

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        // We expect alpha => normal
        // We expect beta => skipped justification
        // We expect gamma => error due to "BadType" inside the Option
        // We expect delta => normal
        // So we should have 2 normal fields in just/conf => alpha, delta
        // And an error from gamma => so just_fields=2, conf_fields=2, plus some error content
        // The final mapping is also 2 => alpha, delta
        assert_eq!(just_fields.len(), 2, "Expected alpha & delta justification fields only");
        assert_eq!(conf_fields.len(), 2, "Expected alpha & delta confidence fields only");
        assert_eq!(mappings.len(), 2, "Expected mappings only for alpha & delta");

        // Should have an error from gamma => check errors
        assert!(!errs.is_empty(), "Expected compile_error for the nested BadType in gamma");
    }

    #[traced_test]
    fn it_handles_all_just_string_scenarios() {
        trace!("Starting test: it_handles_all_just_string_scenarios");
        // Builtin scalar => JustString
        // This covers i32, bool, string, etc.
        let named: FieldsNamed = parse_quote! {
            {
                int_field: i32,
                bool_field: bool,
                str_field: String,
            }
        };
        debug!("Parsed fields: i32, bool, String => all produce JustString");

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        assert_eq!(just_fields.len(), 3, "Expected 3 justification fields");
        assert_eq!(conf_fields.len(), 3, "Expected 3 confidence fields");
        assert!(errs.is_empty(), "Expected no errors");
        assert_eq!(mappings.len(), 3, "Expected 3 field mappings");
    }

    #[traced_test]
    fn it_handles_nested_type_as_nested_justification() {
        trace!("Starting test: it_handles_nested_type_as_nested_justification");
        // We'll pretend "MyCustomStruct" is recognized as a user-defined type => NestedJustification
        let named: FieldsNamed = parse_quote! {
            {
                custom_field: MyCustomStruct,
                trivial_field: i64
            }
        };
        debug!("Parsed fields: custom_field(MyCustomStruct => Nested), trivial_field(i64 => JustString)");

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        // We expect one nested => MyCustomStruct => "MyCustomStructJustification" & "MyCustomStructConfidence"
        // We expect one builtin => i64 => JustString
        assert_eq!(just_fields.len(), 2, "Expected two justification fields total");
        assert_eq!(conf_fields.len(), 2, "Expected two confidence fields total");
        assert!(errs.is_empty(), "Expected no errors in this scenario");
        assert_eq!(mappings.len(), 2, "Expected 2 field mappings total");

        // Let's see if the custom field is recognized
        let mapping_custom_opt = mappings.iter().find(|m| m.field_ident().to_string() == "custom_field");
        let mapping_trivial_opt = mappings.iter().find(|m| m.field_ident().to_string() == "trivial_field");

        assert!(mapping_custom_opt.is_some(), "Expected a mapping for 'custom_field'");
        assert!(mapping_trivial_opt.is_some(), "Expected a mapping for 'trivial_field'");

        let mapping_custom = mapping_custom_opt.unwrap();
        let mapping_trivial = mapping_trivial_opt.unwrap();

        // custom_field => NestedJustification => "MyCustomStructJustification"
        debug!("custom_field => justification field type: {:?}", mapping_custom.justification_field_type());
        debug!("custom_field => confidence field type: {:?}", mapping_custom.confidence_field_type());
        let j_str = format!("{:?}", mapping_custom.justification_field_type());
        let c_str = format!("{:?}", mapping_custom.confidence_field_type());
        assert!(j_str.contains("MyCustomStructJustification"), "Expected MyCustomStructJustification in the type");
        assert!(c_str.contains("MyCustomStructConfidence"), "Expected MyCustomStructConfidence in the type");

        // trivial_field => "String" + "f32"
        debug!("trivial_field => justification field type: {:?}", mapping_trivial.justification_field_type());
        debug!("trivial_field => confidence field type: {:?}", mapping_trivial.confidence_field_type());
        let j_str_trivial = format!("{:?}", mapping_trivial.justification_field_type());
        let c_str_trivial = format!("{:?}", mapping_trivial.confidence_field_type());
        assert!(j_str_trivial.contains("String"), "Expected 'String' in the trivial field's justification type");
        assert!(c_str_trivial.contains("f32"), "Expected 'f32' in the trivial field's confidence type");
    }

    #[traced_test]
    fn it_handles_option_of_builtin_as_just_string() {
        trace!("Starting test: it_handles_option_of_builtin_as_just_string");
        let named: FieldsNamed = parse_quote! {
            {
                maybe_val: Option<i32>
            }
        };
        debug!("Parsed field: maybe_val(Option<i32>) => builtin => JustString on the inner type, but logic sees it as JustString anyway");

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        // Option of numeric => JustString path
        assert_eq!(just_fields.len(), 1, "Expected single justification field");
        assert_eq!(conf_fields.len(), 1, "Expected single confidence field");
        assert!(errs.is_empty(), "Expected no errors");
        assert_eq!(mappings.len(), 1, "Expected single mapping");
        let mapping = &mappings[0];
        let j_str = format!("{:?}", mapping.justification_field_type());
        let c_str = format!("{:?}", mapping.confidence_field_type());
        assert!(j_str.contains("String"), "Expected JustString for Option<i32>");
        assert!(c_str.contains("f32"), "Expected f32 for Option<i32>");
    }

    #[traced_test]
    fn it_handles_option_of_custom_nested_as_nested() {
        trace!("Starting test: it_handles_option_of_custom_nested_as_nested");
        let named: FieldsNamed = parse_quote! {
            {
                maybe_custom: Option<DeeplyNested>
            }
        };
        debug!("Parsed field: maybe_custom(Option<DeeplyNested>) => custom => NestedJustification path");

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        // Option of custom => nested justification
        assert_eq!(just_fields.len(), 1, "Expected single justification field");
        assert_eq!(conf_fields.len(), 1, "Expected single confidence field");
        assert!(errs.is_empty(), "Expected no errors");
        assert_eq!(mappings.len(), 1, "Expected single mapping");
        let mapping = &mappings[0];
        let j_str = format!("{:?}", mapping.justification_field_type());
        let c_str = format!("{:?}", mapping.confidence_field_type());
        assert!(j_str.contains("DeeplyNestedJustification"), "Expected nested justification for Option<DeeplyNested>");
        assert!(c_str.contains("DeeplyNestedConfidence"), "Expected nested confidence for Option<DeeplyNested>");
    }

    #[traced_test]
    fn it_handles_vec_of_builtin_as_just_string() {
        trace!("Starting test: it_handles_vec_of_builtin_as_just_string");
        let named: FieldsNamed = parse_quote! {
            {
                list: Vec<bool>
            }
        };
        debug!("Parsed field: list(Vec<bool>) => builtin => JustString path for each element");

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        assert_eq!(just_fields.len(), 1, "Expected single justification field for Vec<bool>");
        assert_eq!(conf_fields.len(), 1, "Expected single confidence field for Vec<bool>");
        assert!(errs.is_empty(), "Expected no errors for Vec<bool>");
        assert_eq!(mappings.len(), 1, "Expected single mapping for Vec<bool>");
        let mapping = &mappings[0];
        let j_str = format!("{:?}", mapping.justification_field_type());
        let c_str = format!("{:?}", mapping.confidence_field_type());
        assert!(j_str.contains("String"), "Expected JustString for Vec<bool>");
        assert!(c_str.contains("f32"), "Expected f32 for Vec<bool>");
    }

    #[traced_test]
    fn it_handles_vec_of_custom_as_nested() {
        trace!("Starting test: it_handles_vec_of_custom_as_nested");
        let named: FieldsNamed = parse_quote! {
            {
                complex_list: Vec<AnotherCustomType>
            }
        };
        debug!("Parsed field: complex_list(Vec<AnotherCustomType>) => nested justification");

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        assert_eq!(just_fields.len(), 1, "Expected single justification field");
        assert_eq!(conf_fields.len(), 1, "Expected single confidence field");
        assert!(errs.is_empty(), "Expected no errors");
        assert_eq!(mappings.len(), 1, "Expected single mapping");
        let mapping = &mappings[0];
        let j_str = format!("{:?}", mapping.justification_field_type());
        let c_str = format!("{:?}", mapping.confidence_field_type());
        assert!(j_str.contains("AnotherCustomTypeJustification"), "Expected nested justification for Vec<AnotherCustomType>");
        assert!(c_str.contains("AnotherCustomTypeConfidence"), "Expected nested confidence for Vec<AnotherCustomType>");
    }

    #[traced_test]
    fn it_handles_hashmap_with_string_key_and_builtin_value() {
        trace!("Starting test: it_handles_hashmap_with_string_key_and_builtin_value");
        let named: FieldsNamed = parse_quote! {
            {
                map_field: std::collections::HashMap<String, i32>
            }
        };
        debug!("Parsed field: map_field(HashMap<String, i32>) => builtin => JustString path for the value");

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        // For a HashMap with non-bool key => we treat it as JustString for the entire map
        // i.e. single field => single mapping
        assert_eq!(just_fields.len(), 1, "Expected single justification field for HashMap");
        assert_eq!(conf_fields.len(), 1, "Expected single confidence field for HashMap");
        assert!(errs.is_empty(), "Expected no errors for HashMap<String,i32>");
        assert_eq!(mappings.len(), 1, "Expected single mapping");
        let mapping = &mappings[0];
        let j_str = format!("{:?}", mapping.justification_field_type());
        let c_str = format!("{:?}", mapping.confidence_field_type());
        assert!(j_str.contains("String"), "Expected JustString for the entire map");
        assert!(c_str.contains("f32"), "Expected f32 for the entire map");
    }

    #[traced_test]
    fn it_handles_hashmap_with_bool_key_and_emits_error() {
        trace!("Starting test: it_handles_hashmap_with_bool_key_and_emits_error");
        let named: FieldsNamed = parse_quote! {
            {
                bad_map: std::collections::HashMap<bool, String>
            }
        };
        debug!("Parsed field: bad_map(HashMap<bool, String>) => unsupported => compile_error");

        let (just_fields, conf_fields, errs, mappings) =
            gather_fields_for_just_conf(&named);

        info!("just_fields: {:?}", just_fields);
        info!("conf_fields: {:?}", conf_fields);
        info!("errs: {:?}", errs);
        info!("mappings: {:?}", mappings);

        // The function's logic yields compile_error for bool keys
        // So no fields or mappings
        assert_eq!(just_fields.len(), 0, "Expected no justification field");
        assert_eq!(conf_fields.len(), 0, "Expected no confidence field");
        assert!(mappings.is_empty(), "Expected no mapping");
        assert!(!errs.is_empty(), "Expected error from bool key in HashMap");
    }
}

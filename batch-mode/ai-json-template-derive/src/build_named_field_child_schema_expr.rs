// ---------------- [ File: ai-json-template-derive/src/build_named_field_child_schema_expr.rs ]
crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn build_named_field_child_schema_expr(
    field: &syn::Field,
    doc_str: &str,
    is_required: bool,
    skip_child_just: bool,
) -> Option<proc_macro2::TokenStream> {
    tracing::trace!(
        "build_named_field_child_schema_expr: field='{}', required={}, skip_child_just={}",
        field
            .ident
            .as_ref()
            .map_or("<unnamed?>".to_string(), |i| i.to_string().to_string()),
        is_required,
        skip_child_just
    );

    classify_field_type_for_child(&field.ty, doc_str, is_required, skip_child_just)
}


#[cfg(test)]
mod test_build_named_field_child_schema_expr_exhaustive {
    use super::*;

    /// A helper function for verifying that the generated TokenStream
    /// contains `"required"` being set to the expected boolean. We do this
    /// by normalizing whitespace and then checking for a substring that
    /// corresponds to how our code actually emits `"required"`.
    #[tracing::instrument(level = "trace", skip_all)]
    fn assert_contains_required_bool(ts: &proc_macro2::TokenStream, expected: bool) {
        trace!("Checking if TokenStream has 'required' set to {:?}", expected);
        let raw = ts.to_string().replace(' ', "").replace('\n', "");
        let needle = format!("\"required\".to_string(),serde_json::Value::Bool({})", expected);
        assert!(
            raw.contains(&needle),
            "Could not find `{}` in token string:\n{}",
            needle,
            raw
        );
    }

    /// A helper function for verifying that the generated TokenStream
    /// also emits the doc text we expect. The doc text usually ends
    /// up in `.insert("generation_instructions".to_string(), serde_json::Value::String(...))`.
    #[tracing::instrument(level = "trace", skip_all)]
    fn assert_contains_doc_text(ts: &proc_macro2::TokenStream, doc_substring: &str) {
        trace!("Checking if TokenStream has doc text: {:?}", doc_substring);
        let raw = ts.to_string();
        assert!(
            raw.contains(doc_substring),
            "Expected doc substring '{}' not found in:\n{}",
            doc_substring,
            raw
        );
    }

    /// A helper function to parse a single named field of the form:
    /// `struct Temp { <field_tokens> }`
    /// then extract that single named field for testing.
    #[tracing::instrument(level = "trace", skip_all)]
    fn parse_single_named_field(field_tokens: proc_macro2::TokenStream) -> Field {
        trace!("parse_single_named_field: parsing field tokens = {}", field_tokens.to_string());
        let full_struct: syn::DeriveInput = parse_quote! {
            struct Temp {
                #field_tokens
            }
        };
        match full_struct.data {
            syn::Data::Struct(ds) => {
                match ds.fields {
                    syn::Fields::Named(named) => {
                        if named.named.len() != 1 {
                            panic!("Expected exactly one field in 'Temp' but found {}", named.named.len());
                        }
                        named.named.into_iter().next().unwrap()
                    }
                    _ => panic!("Expected named fields in parse_single_named_field"),
                }
            }
            _ => panic!("Expected a struct with named fields in parse_single_named_field"),
        }
    }

    #[traced_test]
    fn test_bool_required_skip_false() {
        trace!("Starting test_bool_required_skip_false");
        let field_tokens = quote! { my_bool: bool };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Documentation for bool", true, false);
        assert!(result.is_some(), "Expected a schema for bool field");
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());

        // Check that we indeed produce `required = true` in the code
        assert_contains_required_bool(&ts, true);

        // Also check doc text
        assert_contains_doc_text(&ts, "Documentation for bool");

        info!("test_bool_required_skip_false passed.");
    }

    #[traced_test]
    fn test_bool_not_required_skip_false() {
        trace!("Starting test_bool_not_required_skip_false");
        let field_tokens = quote! { opt_bool: Option<bool> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Doc for optional bool", false, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());

        // Should have `required = false`
        assert_contains_required_bool(&ts, false);
        // And doc text
        assert_contains_doc_text(&ts, "Doc for optional bool");

        info!("test_bool_not_required_skip_false passed.");
    }

    #[traced_test]
    fn test_bool_skip_child_just_true() {
        trace!("Starting test_bool_skip_child_just_true");
        let field_tokens = quote! { my_bool: bool };
        let field = parse_single_named_field(field_tokens);
        // is_required = true, skip_child_just = true
        let result = build_named_field_child_schema_expr(&field, "doc skip child", true, true);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());

        // Should still produce boolean with `required = true`
        assert_contains_required_bool(&ts, true);
        assert_contains_doc_text(&ts, "doc skip child");

        info!("test_bool_skip_child_just_true passed.");
    }

    #[traced_test]
    fn test_string_required() {
        trace!("Starting test_string_required");
        let field_tokens = quote! { name: String };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Doc for string field", true, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());

        // Must have required = true
        assert_contains_required_bool(&ts, true);
        // And doc
        assert_contains_doc_text(&ts, "Doc for string field");

        info!("test_string_required passed.");
    }

    #[traced_test]
    fn test_string_not_required() {
        trace!("Starting test_string_not_required");
        let field_tokens = quote! { maybe_name: Option<String> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Optional string doc", false, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());

        assert_contains_required_bool(&ts, false);
        assert_contains_doc_text(&ts, "Optional string doc");

        info!("test_string_not_required passed.");
    }

    #[traced_test]
    fn test_numeric_required() {
        trace!("Starting test_numeric_required");
        let field_tokens = quote! { count: i32 };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Integer doc", true, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());

        // Must have required = true
        assert_contains_required_bool(&ts, true);
        assert_contains_doc_text(&ts, "Integer doc");

        info!("test_numeric_required passed.");
    }

    #[traced_test]
    fn test_numeric_not_required() {
        trace!("Starting test_numeric_not_required");
        let field_tokens = quote! { maybe_count: Option<i64> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Optional number doc", false, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());

        assert_contains_required_bool(&ts, false);
        assert_contains_doc_text(&ts, "Optional number doc");

        info!("test_numeric_not_required passed.");
    }

    #[traced_test]
    fn test_vec_of_strings_required() {
        trace!("Starting test_vec_of_strings_required");
        let field_tokens = quote! { tags: Vec<String> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Doc for vector of strings", true, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());

        // We want "array_of" with required = true
        assert_contains_required_bool(&ts, true);
        // And an item_template snippet
        let no_whitespace = ts.to_string().replace(' ', "").replace('\n', "");
        assert!(
            no_whitespace.contains("\"item_template\""),
            "Expected 'item_template' in the snippet: {}",
            no_whitespace
        );
        assert_contains_doc_text(&ts, "Doc for vector of strings");

        info!("test_vec_of_strings_required passed.");
    }

    #[traced_test]
    fn test_vec_of_numeric_not_required() {
        trace!("Starting test_vec_of_numeric_not_required");
        let field_tokens = quote! { amounts: Option<Vec<u64>> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Optional vector doc", false, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());

        // "array_of" => required=false top-level
        assert_contains_required_bool(&ts, false);
        // doc text
        assert_contains_doc_text(&ts, "Optional vector doc");

        info!("test_vec_of_numeric_not_required passed.");
    }

    #[traced_test]
    fn test_hashmap_string_key_not_required() {
        trace!("Starting test_hashmap_string_key_not_required");
        let field_tokens = quote! { map_data: Option<std::collections::HashMap<String, i32>> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Doc for map", false, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());

        let no_whitespace = ts.to_string().replace(' ', "").replace('\n', "");
        // Expect 'map_of' at top
        assert!(
            no_whitespace.contains("\"map_of\""),
            "Expected 'map_of' in snippet: {}",
            no_whitespace
        );
        // Key=string, Value=number
        assert_contains_required_bool(&ts, false);
        assert_contains_doc_text(&ts, "Doc for map");

        info!("test_hashmap_string_key_not_required passed.");
    }

    #[traced_test]
    fn test_hashmap_bool_key_should_error() {
        trace!("Starting test_hashmap_bool_key_should_error");
        let field_tokens = quote! { bad_map: std::collections::HashMap<bool, String> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Bad map doc", true, false);
        // We expect Some(...) with a compile_error inside
        match result {
            Some(ts) => {
                let s = ts.to_string();
                debug!("Generated TokenStream for error: {}", s);
                assert!(s.contains("compile_error"));
                assert!(s.contains("Unsupported key type in HashMap<bool"));
                info!("test_hashmap_bool_key_should_error passed.");
            }
            None => panic!("Expected Some(...) with compile_error for bool key"),
        }
    }

    #[traced_test]
    fn test_nested_custom_type() {
        trace!("Starting test_nested_custom_type");
        let field_tokens = quote! { custom: MyStruct };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Doc for nested custom", true, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());

        // "nested_struct_or_enum"
        let no_ws = ts.to_string().replace(' ', "").replace('\n', "");
        assert!(
            no_ws.contains("\"nested_struct_or_enum\""),
            "Expected nested_struct_or_enum but not found: {}",
            no_ws
        );
        assert_contains_required_bool(&ts, true);
        assert_contains_doc_text(&ts, "Doc for nested custom");

        info!("test_nested_custom_type passed.");
    }

    #[traced_test]
    fn test_optional_nested_custom_type() {
        trace!("Starting test_optional_nested_custom_type");
        let field_tokens = quote! { maybe_custom: Option<MyEnum> };
        let field = parse_single_named_field(field_tokens);
        let result = build_named_field_child_schema_expr(&field, "Doc for optional nested", false, false);
        assert!(result.is_some());
        let ts = result.unwrap();
        debug!("Generated TokenStream: {}", ts.to_token_stream().to_string());

        let no_ws = ts.to_string().replace(' ', "").replace('\n', "");
        assert!(
            no_ws.contains("\"nested_struct_or_enum\""),
            "Expected nested_struct_or_enum but not found: {}",
            no_ws
        );
        // Should be required=false at top-level
        assert_contains_required_bool(&ts, false);
        assert_contains_doc_text(&ts, "Doc for optional nested");

        info!("test_optional_nested_custom_type passed.");
    }
}

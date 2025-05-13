// ---------------- [ File: ai-json-template-derive/src/build_numeric_schema.rs ]
crate::ix!();

pub fn build_numeric_schema(
    required_bool: proc_macro2::TokenStream,
    doc_lit: proc_macro2::Literal
) -> Option<proc_macro2::TokenStream> {
    Some(quote::quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            serde_json::Value::Object(obj)
        }
    })
}

#[cfg(test)]
mod verify_build_numeric_schema {
    use super::*;

    #[traced_test]
    fn ensure_empty_doc_still_builds_schema() {
        trace!("Starting test: ensure_empty_doc_still_builds_schema");

        // Arrange
        let required_bool = quote::quote!(true);
        let doc_lit = proc_macro2::Literal::string("");

        // Act
        let result = build_numeric_schema(required_bool.clone(), doc_lit.clone());

        // Assert
        assert!(
            result.is_some(),
            "Expected Some(TokenStream), got None."
        );

        let token_stream = result.unwrap();
        let ts_str = token_stream.to_string();

        debug!("Raw token stream:\n{}", ts_str);

        // Let's log each line in a more readable way:
        for (i, line) in ts_str.lines().enumerate() {
            trace!("Line #{} => {}", i, line);
        }

        // Check for "type" => "number" and "required" => true
        assert!(
            ts_str.contains("\"type\"") && ts_str.contains("\"number\""),
            "Should mention 'type' => 'number': {ts_str}"
        );
        assert!(
            ts_str.contains("\"required\"") && ts_str.contains("true"),
            "Should mention 'required' => true: {ts_str}"
        );
        assert!(
            ts_str.contains("\"generation_instructions\""),
            "Should contain generation_instructions key: {ts_str}"
        );

        // We just want to confirm the doc string is empty.  Different spacing or parentheses
        // can occur, so we test for a few possibilities.  The code typically emits `"" .to_string()`.
        let possible_subs = &[
            "\"\" .to_string",     // with a space
            "\"\".to_string",      // no space
            "\"\" . to_string",    // spaced further
        ];

        let found_empty = possible_subs.iter().any(|s| ts_str.contains(s));

        if !found_empty {
            error!("Could not find any recognized substring for an empty doc: {ts_str}");
            panic!(
                "Should contain empty doc string for generation_instructions, got: {ts_str}"
            );
        } else {
            info!("Found an empty doc string snippet => test passed");
        }
    }

    #[traced_test]
    fn ensure_required_true_generates_number_schema() {
        trace!("Starting test: ensure_required_true_generates_number_schema");

        // Arrange
        let required_bool = quote!(true);
        let doc_lit = proc_macro2::Literal::string("This is a numeric field.");

        // Act
        let result = build_numeric_schema(required_bool.clone(), doc_lit.clone());

        // Assert
        assert!(result.is_some(), "Expected Some(TokenStream), got None.");
        let token_stream = result.unwrap();
        debug!("TokenStream output: {}", token_stream.to_string());

        let ts_str = token_stream.to_string();
        assert!(
            ts_str.contains("\"number\""),
            "Should contain '\"number\"' for type=number, got: {ts_str}"
        );
        assert!(
            ts_str.contains("This is a numeric field."),
            "Should contain doc_lit='This is a numeric field.', got: {ts_str}"
        );
        assert!(
            ts_str.contains("true"),
            "Should contain 'true' for required, got: {ts_str}"
        );

        info!("Passed ensure_required_true_generates_number_schema");
    }

    #[traced_test]
    fn ensure_required_false_generates_number_schema() {
        trace!("Starting test: ensure_required_false_generates_number_schema");

        // Arrange
        let required_bool = quote!(false);
        let doc_lit = proc_macro2::Literal::string("An optional numeric field.");

        // Act
        let result = build_numeric_schema(required_bool.clone(), doc_lit.clone());

        // Assert
        assert!(result.is_some(), "Expected Some(TokenStream), got None.");
        let token_stream = result.unwrap();
        debug!("TokenStream output: {}", token_stream.to_string());

        let ts_str = token_stream.to_string();
        assert!(
            ts_str.contains("\"number\""),
            "Should contain '\"number\"' for type=number, got: {ts_str}"
        );
        assert!(
            ts_str.contains("An optional numeric field."),
            "Should contain doc_lit='An optional numeric field.', got: {ts_str}"
        );
        assert!(
            ts_str.contains("false"),
            "Should contain 'false' for required, got: {ts_str}"
        );

        info!("Passed ensure_required_false_generates_number_schema");
    }

    #[traced_test]
    fn ensure_special_chars_in_doc_are_preserved() {
        trace!("Starting test: ensure_special_chars_in_doc_are_preserved");

        // Arrange
        let required_bool = quote!(true);
        // Some special chars, quotes, etc.
        let doc_lit = proc_macro2::Literal::string("Some \"quoted\" text with \n newlines & symbols <>!");

        // Act
        let result = build_numeric_schema(required_bool.clone(), doc_lit.clone());

        // Assert
        assert!(result.is_some(), "Expected Some(TokenStream), got None.");
        let token_stream = result.unwrap();
        debug!("TokenStream output: {}", token_stream.to_string());
        let ts_str = token_stream.to_string();

        // Check it contains the special string
        // The double-quotes become escaped as \",
        // and the newline becomes \\n inside the code string, etc.
        assert!(
            ts_str.contains("Some \\\"quoted\\\" text with \\n newlines & symbols <>!"),
            "Should preserve special chars in doc_lit, got: {ts_str}"
        );

        info!("Passed ensure_special_chars_in_doc_are_preserved");
    }
}

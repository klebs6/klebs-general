// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_bool.rs ]
crate::ix!();

pub fn emit_schema_for_bool(
    generation_instructions: &str,
    required_bool: &proc_macro2::TokenStream
) -> proc_macro2::TokenStream {
    trace!("emit_schema_for_bool invoked");
    quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("boolean".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            serde_json::Value::Object(obj)
        }
    }
}

#[cfg(test)]
mod test_emit_schema_for_bool_exhaustive {
    use super::*;

    #[traced_test]
    fn test_emit_schema_for_bool_true_with_standard_doc() {
        trace!("Starting test_emit_schema_for_bool_true_with_standard_doc...");

        let generation_instructions = "example boolean field documentation";
        let required_bool = quote!(true);

        trace!("Calling emit_schema_for_bool...");
        let result_tokens = emit_schema_for_bool(generation_instructions, &required_bool);

        debug!("Resulting token stream: {}", result_tokens.to_string());
        assert!(
            result_tokens.to_string().contains("boolean"),
            "Expected token stream to include the word 'boolean'"
        );
        assert!(
            result_tokens.to_string().contains("example boolean field documentation"),
            "Expected token stream to include the generation instructions"
        );
        assert!(
            result_tokens.to_string().contains("true"),
            "Expected token stream to include 'true' for the required_bool"
        );

        trace!("Attempting to parse the returned tokens as a valid expression...");
        let parsed_expr: Expr = parse2(result_tokens).expect("Should successfully parse into a syn::Expr");
        debug!("Parsed expression: {:?}", parsed_expr);
    }

    #[traced_test]
    fn test_emit_schema_for_bool_false_with_standard_doc() {
        trace!("Starting test_emit_schema_for_bool_false_with_standard_doc...");

        let generation_instructions = "some doc line here";
        let required_bool = quote!(false);

        trace!("Calling emit_schema_for_bool...");
        let result_tokens = emit_schema_for_bool(generation_instructions, &required_bool);

        debug!("Resulting token stream: {}", result_tokens.to_string());
        assert!(
            result_tokens.to_string().contains("boolean"),
            "Expected token stream to include the word 'boolean'"
        );
        assert!(
            result_tokens.to_string().contains("some doc line here"),
            "Expected token stream to include the generation instructions"
        );
        assert!(
            result_tokens.to_string().contains("false"),
            "Expected token stream to include 'false' for the required_bool"
        );

        trace!("Attempting to parse the returned tokens as a valid expression...");
        let parsed_expr: Expr = parse2(result_tokens).expect("Should successfully parse into a syn::Expr");
        debug!("Parsed expression: {:?}", parsed_expr);
    }

    #[traced_test]
    fn test_emit_schema_for_bool_empty_generation_instructions() {
        trace!("Starting test_emit_schema_for_bool_empty_generation_instructions...");

        let generation_instructions = "";
        let required_bool = quote!(true);

        trace!("Calling emit_schema_for_bool with empty generation_instructions...");
        let result_tokens = emit_schema_for_bool(generation_instructions, &required_bool);

        debug!("Resulting token stream: {}", result_tokens.to_string());
        // Even if doc is empty, we expect "boolean" and "true"
        assert!(
            result_tokens.to_string().contains("boolean"),
            "Expected token stream to include the word 'boolean'"
        );
        assert!(
            result_tokens.to_string().contains("true"),
            "Expected token stream to include 'true' for the required_bool"
        );
        // No doc lines to test for, but we confirm it does not crash.

        trace!("Attempting to parse the returned tokens as a valid expression...");
        let parsed_expr: Expr = parse2(result_tokens).expect("Should parse into a syn::Expr");
        debug!("Parsed expression: {:?}", parsed_expr);
    }

    #[traced_test]
    fn test_emit_schema_for_bool_special_chars_in_instructions() {
        trace!("Starting test_emit_schema_for_bool_special_chars_in_instructions...");

        let generation_instructions = "bool doc with \"quotes\" and \n newlines";
        let required_bool = quote!(false);

        trace!("Calling emit_schema_for_bool with special chars...");
        let result_tokens = emit_schema_for_bool(generation_instructions, &required_bool);

        let result_code = result_tokens.to_string();
        debug!("Resulting token stream: {}", result_code);

        // Check presence of 'boolean'
        assert!(
            result_code.contains("boolean"),
            "Expected 'boolean' in the token stream."
        );
        // Check presence of our special doc
        assert!(
            result_code.contains("bool doc with"),
            "Expected partial doc text in the token stream."
        );
        // Check presence of false
        assert!(
            result_code.contains("false"),
            "Expected token stream to include 'false' for the required_bool"
        );

        trace!("Attempting to parse the returned tokens as a valid expression...");
        let parsed_expr: Expr = parse2(result_tokens).expect("Should parse into a syn::Expr");
        debug!("Parsed expression: {:?}", parsed_expr);
    }

    #[traced_test]
    fn test_emit_schema_for_bool_tokenstream_edge_cases() {
        trace!("Starting test_emit_schema_for_bool_tokenstream_edge_cases...");

        // Although typically we'd pass true/false, let's see if it fails with non-boolean
        // This is just to ensure we don't panic, even though the function expects a bool.
        // We'll pass 'quote!(1 + 2)' to see if it at least compiles the template.
        let generation_instructions = "non-boolean token test";
        let required_bool = quote!(1 + 2);

        trace!("Calling emit_schema_for_bool with '1 + 2'...");
        let result_tokens = emit_schema_for_bool(generation_instructions, &required_bool);

        let result_code = result_tokens.to_string();
        debug!("Resulting token stream: {}", result_code);

        // We at least expect 'boolean' and the doc text, though 'required' will be #required_bool => (1 + 2).
        assert!(
            result_code.contains("boolean"),
            "Expected the token stream to still contain 'boolean'."
        );
        assert!(
            result_code.contains("non-boolean token test"),
            "Expected to find the generation instructions in the token stream."
        );
        // The '1 + 2' won't parse as a bool at runtime, but we want to see if it still compiles as tokens:
        assert!(
            result_code.contains("1 + 2"),
            "Expected to find '1 + 2' in the token stream."
        );

        trace!("Attempting to parse the returned tokens as a valid expression...");
        let parsed_expr: Expr = parse2(result_tokens)
            .expect("Should parse into a syn::Expr even if it doesn't evaluate to a bool at runtime");
        debug!("Parsed expression: {:?}", parsed_expr);
    }
}

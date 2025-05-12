// ---------------- [ File: ai-json-template-derive/src/build_bool_schema.rs ]
crate::ix!();

/// Builds a schema snippet for `bool` => "boolean".
pub fn build_bool_schema(
    required_bool: proc_macro2::TokenStream,
    doc_lit: proc_macro2::Literal
) -> Option<proc_macro2::TokenStream> {
    Some(quote::quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("boolean".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            serde_json::Value::Object(obj)
        }
    })
}

#[cfg(test)]
mod test_build_bool_schema_module {
    use super::*;

    #[traced_test]
    fn verify_required_true_nonempty_doc() {
        trace!("Starting verify_required_true_nonempty_doc test for build_bool_schema.");

        let doc_lit = Literal::string("This is a doc line.");
        let required_bool = quote::quote!(true);

        let result_option = build_bool_schema(required_bool, doc_lit);
        assert!(result_option.is_some(), "Expected Some(TokenStream) but got None.");

        let ts = result_option.unwrap();
        let generated_code = ts.to_string();
        debug!("Generated TokenStream = {}", generated_code);

        // Check that we insert type=boolean
        assert!(
            generated_code.contains("\"type\"")
                && generated_code.contains("\"boolean\""),
            "Expected code snippet to contain type=boolean."
        );

        // Check that we insert required=true
        assert!(
            generated_code.contains("\"required\"")
                && generated_code.contains("true"),
            "Expected code snippet to contain required=true."
        );

        // Check that we insert generation_instructions with doc string
        assert!(
            generated_code.contains("\"generation_instructions\"")
                && generated_code.contains("This is a doc line."),
            "Expected code snippet to contain generation_instructions with provided doc."
        );

        info!("Completed verify_required_true_nonempty_doc test successfully.");
    }

    #[traced_test]
    fn verify_required_false_nonempty_doc() {
        trace!("Starting verify_required_false_nonempty_doc test for build_bool_schema.");

        let doc_lit = Literal::string("Another doc line for false required.");
        let required_bool = quote::quote!(false);

        let result_option = build_bool_schema(required_bool, doc_lit);
        assert!(result_option.is_some(), "Expected Some(TokenStream) but got None.");

        let ts = result_option.unwrap();
        let generated_code = ts.to_string();
        debug!("Generated TokenStream = {}", generated_code);

        // Check that we insert type=boolean
        assert!(
            generated_code.contains("\"type\"")
                && generated_code.contains("\"boolean\""),
            "Expected code snippet to contain type=boolean."
        );

        // Check that we insert required=false
        assert!(
            generated_code.contains("\"required\"")
                && generated_code.contains("false"),
            "Expected code snippet to contain required=false."
        );

        // Check that we insert doc string
        assert!(
            generated_code.contains("Another doc line for false required."),
            "Expected code snippet to contain the provided doc text."
        );

        info!("Completed verify_required_false_nonempty_doc test successfully.");
    }

    #[traced_test]
    fn verify_required_true_empty_doc() {
        trace!("Starting verify_required_true_empty_doc test for build_bool_schema.");

        let doc_lit = Literal::string("");
        let required_bool = quote::quote!(true);

        let result_option = build_bool_schema(required_bool, doc_lit);
        assert!(result_option.is_some(), "Expected Some(TokenStream) but got None.");

        let ts = result_option.unwrap();
        let generated_code = ts.to_string();
        debug!("Generated TokenStream = {}", generated_code);

        // Check type=boolean
        assert!(
            generated_code.contains("\"boolean\""),
            "Expected code snippet to contain boolean type."
        );

        // Check required=true
        assert!(
            generated_code.contains("\"required\"")
                && generated_code.contains("true"),
            "Expected code snippet to contain required=true."
        );

        // Check empty doc
        // It's still inserted, but with an empty string
        assert!(
            generated_code.contains("\"generation_instructions\"")
                && generated_code.contains("\"\""),
            "Expected code snippet to contain an empty generation_instructions string."
        );

        info!("Completed verify_required_true_empty_doc test successfully.");
    }

    #[traced_test]
    fn verify_required_false_empty_doc() {
        trace!("Starting verify_required_false_empty_doc test for build_bool_schema.");

        let doc_lit = Literal::string("");
        let required_bool = quote::quote!(false);

        let result_option = build_bool_schema(required_bool, doc_lit);
        assert!(result_option.is_some(), "Expected Some(TokenStream) but got None.");

        let ts = result_option.unwrap();
        let generated_code = ts.to_string();
        debug!("Generated TokenStream = {}", generated_code);

        // Check type=boolean
        assert!(
            generated_code.contains("\"boolean\""),
            "Expected code snippet to contain type=boolean."
        );

        // Check required=false
        assert!(
            generated_code.contains("\"required\"")
                && generated_code.contains("false"),
            "Expected code snippet to contain required=false."
        );

        // Check empty doc
        assert!(
            generated_code.contains("\"generation_instructions\"")
                && generated_code.contains("\"\""),
            "Expected code snippet to contain empty generation_instructions string."
        );

        info!("Completed verify_required_false_empty_doc test successfully.");
    }
}

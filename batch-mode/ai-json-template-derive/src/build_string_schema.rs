// ---------------- [ File: ai-json-template-derive/src/build_string_schema.rs ]
crate::ix!();

pub fn build_string_schema(
    required_bool: proc_macro2::TokenStream,
    doc_lit: proc_macro2::Literal
) -> Option<proc_macro2::TokenStream> {
    Some(quote::quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            serde_json::Value::Object(obj)
        }
    })
}

#[cfg(test)]
mod test_build_string_schema {
    use super::*;

    #[traced_test]
    fn test_build_string_schema_required_true_empty_doc() {
        trace!("Starting test: required=true, doc=\"(empty)\"");
        let required_bool = quote!(true);
        let doc_lit = Literal::string("");

        let result = build_string_schema(required_bool.clone(), doc_lit.clone());
        assert!(result.is_some(), "Expected Some(...) result from build_string_schema");

        let tokens = result.unwrap();
        let tokens_str = tokens.to_string();
        debug!(?tokens_str, "Got token stream output");

        assert!(tokens_str.contains("string"), "Should contain 'string' type definition");
        assert!(tokens_str.contains("required"), "Should contain 'required' field");
        assert!(tokens_str.contains("true"), "Should show required=true in the output");
        assert!(!tokens_str.contains("test doc"), "Should not contain any doc text since doc_lit was empty");
    }

    #[traced_test]
    fn test_build_string_schema_required_true_nonempty_doc() {
        trace!("Starting test: required=true, doc=\"some docstring\"");
        let required_bool = quote!(true);
        let doc_lit = Literal::string("some docstring");

        let result = build_string_schema(required_bool.clone(), doc_lit.clone());
        assert!(result.is_some(), "Expected Some(...) result from build_string_schema");

        let tokens = result.unwrap();
        let tokens_str = tokens.to_string();
        debug!(?tokens_str, "Got token stream output");

        assert!(tokens_str.contains("string"), "Should contain 'string' type definition");
        assert!(tokens_str.contains("required"), "Should contain 'required' field");
        assert!(tokens_str.contains("true"), "Should show required=true in the output");
        assert!(tokens_str.contains("some docstring"), "Should embed the provided doc string");
    }

    #[traced_test]
    fn test_build_string_schema_required_false_empty_doc() {
        trace!("Starting test: required=false, doc=\"(empty)\"");
        let required_bool = quote!(false);
        let doc_lit = Literal::string("");

        let result = build_string_schema(required_bool.clone(), doc_lit.clone());
        assert!(result.is_some(), "Expected Some(...) result from build_string_schema");

        let tokens = result.unwrap();
        let tokens_str = tokens.to_string();
        debug!(?tokens_str, "Got token stream output");

        assert!(tokens_str.contains("string"), "Should contain 'string' type definition");
        assert!(tokens_str.contains("required"), "Should contain 'required' field");
        assert!(tokens_str.contains("false"), "Should show required=false in the output");
        assert!(!tokens_str.contains("test doc"), "Should not contain any doc text since doc_lit was empty");
    }

    #[traced_test]
    fn test_build_string_schema_required_false_nonempty_doc() {
        trace!("Starting test: required=false, doc=\"multi line\\nexample\"");
        let required_bool = quote!(false);
        let doc_lit = Literal::string("multi line\nexample");

        let result = build_string_schema(required_bool.clone(), doc_lit.clone());
        assert!(result.is_some(), "Expected Some(...) result from build_string_schema");

        let tokens = result.unwrap();
        let tokens_str = tokens.to_string();
        debug!(?tokens_str, "Got token stream output");

        assert!(tokens_str.contains("string"), "Should contain 'string' type definition");
        assert!(tokens_str.contains("required"), "Should contain 'required' field");
        assert!(tokens_str.contains("false"), "Should show required=false in the output");
        assert!(tokens_str.contains("multi line\\nexample") 
                || tokens_str.contains("multi line\nexample"),
            "Should embed the provided multiline doc string");
    }
}

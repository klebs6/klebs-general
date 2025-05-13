// ---------------- [ File: ai-json-template-derive/src/classify_field_type.rs ]
crate::ix!();

pub fn classify_field_type(ty: &syn::Type, doc_str: &str) -> Option<proc_macro2::TokenStream> {
    tracing::trace!("classify_field_type => doc_str={:?}, type=? => Checking type for AiJsonTemplate", doc_str);

    let doc_lit = proc_macro2::Literal::string(doc_str.trim());

    // 1) If it's an Option<T>, treat T as not required
    if let Some(inner_ty) = extract_option_inner(ty) {
        tracing::trace!("Field is Option<...> => required=false");
        return emit_schema_for_type(inner_ty, doc_lit, false);
    }

    // Otherwise required=true
    tracing::trace!("Field is not an Option => required=true");
    emit_schema_for_type(ty, doc_lit, true)
}

#[cfg(test)]
mod test_classify_field_type_exhaustive {
    use super::*;

    /// A helper that runs `classify_field_type` and returns the stringified TokenStream,
    /// or panics if `None` is returned. This allows us to inspect its overall structure.
    fn run_classify_and_stringify(ty: &Type, doc_str: &str) -> String {
        trace!("Invoking classify_field_type on type: {:?}, doc_str={:?}", ty, doc_str);
        let result_opt = classify_field_type(ty, doc_str);
        match result_opt {
            Some(ts) => {
                let out = ts.to_string();
                debug!("Successfully obtained TokenStream: {}", out);
                out
            },
            None => {
                panic!("Expected Some(TokenStream) but got None for type={:?}, doc_str={:?}", ty, doc_str);
            }
        }
    }

    #[traced_test]
    fn test_option_bool() {
        info!("test_option_bool => Checking classification for Option<bool>");
        let ty: Type = parse_quote!(Option<bool>);
        let output = run_classify_and_stringify(&ty, "example doc for Option<bool>");
        // Expect "required=false" somewhere, "boolean", etc.
        assert!(output.contains("\"boolean\""), "Should contain 'boolean' for bool type, got: {}", output);
        assert!(output.contains("\"required\".to_string() , serde_json :: Value :: Bool ( false )")
                || output.contains("\"required\".to_string(), serde_json::Value::Bool(false)"),
                "Should mark required=false for Option<bool>, got: {}", output);
    }

    #[traced_test]
    fn test_bool() {
        info!("test_bool => Checking classification for bare bool");
        let ty: Type = parse_quote!(bool);
        let output = run_classify_and_stringify(&ty, "example doc for bool");
        // Expect "required=true" somewhere, "boolean", etc.
        assert!(output.contains("\"boolean\""), "Should contain 'boolean' for bool type, got: {}", output);
        assert!(output.contains("\"required\".to_string() , serde_json :: Value :: Bool ( true )")
                || output.contains("\"required\".to_string(), serde_json::Value::Bool(true)"),
                "Should mark required=true for bare bool, got: {}", output);
    }

    #[traced_test]
    fn test_option_string() {
        info!("test_option_string => Checking classification for Option<String>");
        let ty: Type = parse_quote!(Option<String>);
        let output = run_classify_and_stringify(&ty, "doc for Option<String>");
        // Expect "required=false" somewhere, "string", etc.
        assert!(output.contains("\"string\""), "Should contain 'string' for String type, got: {}", output);
        assert!(output.contains("Bool ( false )"), "Should mark required=false for Option<String>, got: {}", output);
    }

    #[traced_test]
    fn test_string() {
        info!("test_string => Checking classification for bare String");
        let ty: Type = parse_quote!(String);
        let output = run_classify_and_stringify(&ty, "doc for bare String");
        // Expect "required=true", "string", etc.
        assert!(output.contains("\"string\""), "Should contain 'string' for String type, got: {}", output);
        assert!(output.contains("Bool ( true )"), "Should mark required=true for bare String, got: {}", output);
    }

    #[traced_test]
    fn test_i32() {
        info!("test_i32 => Checking classification for i32");
        let ty: Type = parse_quote!(i32);
        let output = run_classify_and_stringify(&ty, "doc for i32");
        // Expect "required=true", "number", etc.
        assert!(output.contains("\"number\""), "Should contain 'number' for numeric type i32, got: {}", output);
        assert!(output.contains("Bool ( true )"), "Should mark required=true for i32, got: {}", output);
    }

    #[traced_test]
    fn test_option_f64() {
        info!("test_option_f64 => Checking classification for Option<f64>");
        let ty: Type = parse_quote!(Option<f64>);
        let output = run_classify_and_stringify(&ty, "doc for Option<f64>");
        // Expect "required=false", "number", etc.
        assert!(output.contains("\"number\""), "Should contain 'number' for numeric type f64, got: {}", output);
        assert!(output.contains("Bool ( false )"), "Should mark required=false for Option<f64>, got: {}", output);
    }

    #[traced_test]
    fn test_nested_custom_type() {
        info!("test_nested_custom_type => Checking classification for a user-defined type");
        // We'll pretend `MyCustomType` is some user-defined struct or enum
        let ty: Type = parse_quote!(MyCustomType);
        let output = run_classify_and_stringify(&ty, "doc for MyCustomType");
        // Expect fallback => "nested_struct_or_enum", "required=true"
        assert!(output.contains("\"nested_struct_or_enum\""),
                "Expected fallback to 'nested_struct_or_enum' for unknown user type, got: {}",
                output);
        assert!(output.contains("Bool ( true )"), "Should mark required=true for MyCustomType, got: {}", output);
    }

    #[traced_test]
    fn test_option_nested_custom_type() {
        info!("test_option_nested_custom_type => Checking classification for Option of user-defined type");
        let ty: Type = parse_quote!(Option<AnotherType>);
        let output = run_classify_and_stringify(&ty, "doc for Option<AnotherType>");
        // Expect fallback => "nested_struct_or_enum", "required=false"
        assert!(output.contains("\"nested_struct_or_enum\""),
                "Expected fallback to 'nested_struct_or_enum' for unknown user type, got: {}",
                output);
        assert!(output.contains("Bool ( false )"), "Should mark required=false for Option<AnotherType>, got: {}", output);
    }
}

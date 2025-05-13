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
    use syn::{parse_quote, Type};
    use tracing::{info, trace, debug};

    /// A helper that runs `classify_field_type` and returns the stringified TokenStream,
    /// or panics if `None` is returned.
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
        // Should have "boolean", "required=false"
        assert!(
            output.contains("\"boolean\""),
            "Should contain 'boolean' for bool type, got: {}",
            output
        );
        // The actual code typically prints `Value :: Bool (false)`, or `Value::Bool(false)`
        // with no space around (false).
        assert!(
            output.contains("Value :: Bool (false)") || output.contains("Value::Bool(false)"),
            "Should mark required=false for Option<bool>, got: {}",
            output
        );
        // Also check we have the doc string as "\"example doc for Option<bool>\"" or similar
        // (the macros typically embed the doc with an extra quote).
        assert!(
            output.contains("example doc for Option<bool>"),
            "Expected doc string for Option<bool>, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_bool() {
        info!("test_bool => Checking classification for bare bool");
        let ty: Type = parse_quote!(bool);
        let output = run_classify_and_stringify(&ty, "example doc for bool");
        // Should have "boolean", "required=true"
        assert!(
            output.contains("\"boolean\""),
            "Should contain 'boolean' for bool type, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (true)") || output.contains("Value::Bool(true)"),
            "Should mark required=true for bare bool, got: {}",
            output
        );
        assert!(
            output.contains("example doc for bool"),
            "Expected doc string for bool, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_option_string() {
        info!("test_option_string => Checking classification for Option<String>");
        let ty: Type = parse_quote!(Option<String>);
        let output = run_classify_and_stringify(&ty, "doc for Option<String>");
        // Should have "string", "required=false"
        assert!(
            output.contains("\"string\""),
            "Should contain 'string' for String type, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (false)") || output.contains("Value::Bool(false)"),
            "Should mark required=false for Option<String>, got: {}",
            output
        );
        assert!(
            output.contains("doc for Option<String>"),
            "Expected doc string for Option<String>, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_string() {
        info!("test_string => Checking classification for bare String");
        let ty: Type = parse_quote!(String);
        let output = run_classify_and_stringify(&ty, "doc for bare String");
        // Should have "string", "required=true"
        assert!(
            output.contains("\"string\""),
            "Should contain 'string' for String type, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (true)") || output.contains("Value::Bool(true)"),
            "Should mark required=true for bare String, got: {}",
            output
        );
        assert!(
            output.contains("doc for bare String"),
            "Expected doc string for bare String, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_i32() {
        info!("test_i32 => Checking classification for i32");
        let ty: Type = parse_quote!(i32);
        let output = run_classify_and_stringify(&ty, "doc for i32");
        // Should have "number", "required=true"
        assert!(
            output.contains("\"number\""),
            "Should contain 'number' for numeric type i32, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (true)") || output.contains("Value::Bool(true)"),
            "Should mark required=true for i32, got: {}",
            output
        );
        assert!(
            output.contains("doc for i32"),
            "Expected doc string for i32, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_option_f64() {
        info!("test_option_f64 => Checking classification for Option<f64>");
        let ty: Type = parse_quote!(Option<f64>);
        let output = run_classify_and_stringify(&ty, "doc for Option<f64>");
        // Should have "number", "required=false"
        assert!(
            output.contains("\"number\""),
            "Should contain 'number' for numeric type f64, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (false)") || output.contains("Value::Bool(false)"),
            "Should mark required=false for Option<f64>, got: {}",
            output
        );
        assert!(
            output.contains("doc for Option<f64>"),
            "Expected doc string for Option<f64>, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_nested_custom_type() {
        info!("test_nested_custom_type => Checking classification for a user-defined type");
        let ty: Type = parse_quote!(MyCustomType);
        let output = run_classify_and_stringify(&ty, "doc for MyCustomType");
        // Expect fallback => "nested_struct_or_enum", "required=true"
        // But the macro might produce a snippet that calls AiJsonTemplate::to_template() 
        // plus some logic to guess "nested_enum"/"nested_struct". We just check for "required=true" 
        // and "nested" in the snippet.
        assert!(
            output.contains("nested_struct_or_enum")
                || output.contains("nested_struct") 
                || output.contains("nested_enum"),
            "Expected fallback to 'nested_struct_or_enum' or 'nested_struct' for unknown user type, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (true)") || output.contains("Value::Bool(true)"),
            "Should mark required=true for MyCustomType, got: {}",
            output
        );
        assert!(
            output.contains("doc for MyCustomType"),
            "Expected doc for MyCustomType, got: {}",
            output
        );
    }

    #[traced_test]
    fn test_option_nested_custom_type() {
        info!("test_option_nested_custom_type => Checking classification for Option of user-defined type");
        let ty: Type = parse_quote!(Option<AnotherType>);
        let output = run_classify_and_stringify(&ty, "doc for Option<AnotherType>");
        // Expect fallback => "nested_struct_or_enum", "required=false"
        assert!(
            output.contains("nested_struct_or_enum")
                || output.contains("nested_struct") 
                || output.contains("nested_enum"),
            "Expected fallback to 'nested_struct_or_enum', 'nested_struct', or 'nested_enum' for unknown user type, got: {}",
            output
        );
        assert!(
            output.contains("Value :: Bool (false)") || output.contains("Value::Bool(false)"),
            "Should mark required=false for Option<AnotherType>, got: {}",
            output
        );
        assert!(
            output.contains("doc for Option<AnotherType>"),
            "Expected doc string for Option<AnotherType>, got: {}",
            output
        );
    }
}


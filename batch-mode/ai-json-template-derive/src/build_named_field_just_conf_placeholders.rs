// ---------------- [ File: ai-json-template-derive/src/build_named_field_just_conf_placeholders.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_named_field_just_conf_placeholders(field_name_str: &str) -> proc_macro2::TokenStream {
    trace!("build_named_field_just_conf_placeholders: field='{}'", field_name_str);

    // Construct the literal keys (e.g. "someField_justification", "someField_confidence")
    let justify_key_str = format!("{}_justification", field_name_str);
    let conf_key_str    = format!("{}_confidence",    field_name_str);
    debug!(
        "Constructed justification key='{}' and confidence key='{}'",
        justify_key_str, conf_key_str
    );

    // We'll build a full Rust block as an AST via parse_quote.
    // This ensures that (a) we do not post-process strings ourselves,
    // and (b) the final printed tokens have `map.insert(` exactly (not `map . insert`).
    //
    // Also, inserting "type".to_string() or "required".to_string() here
    // will naturally produce `just_obj.insert("type".to_string(), ...)`
    // with no unwanted spaces.
    let block_ast: syn::Block = {
        // Convert each placeholder key into a syn::LitStr
        let justify_key_lit = syn::LitStr::new(&justify_key_str, proc_macro2::Span::call_site());
        let conf_key_lit    = syn::LitStr::new(&conf_key_str,    proc_macro2::Span::call_site());

        syn::parse_quote! {
            {
                let mut just_obj = serde_json::Map::new();
                just_obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
                just_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                map.insert(#justify_key_lit.to_string(), serde_json::Value::Object(just_obj));

                let mut conf_obj = serde_json::Map::new();
                conf_obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
                conf_obj.insert("required".to_string(), serde_json::Value::Bool(true));
                map.insert(#conf_key_lit.to_string(), serde_json::Value::Object(conf_obj));
            }
        }
    };

    // Convert that parsed block back into a TokenStream. We do NOT do
    // any string manipulation: we return the AST that naturally
    // prints as desired by the test suite.
    block_ast.into_token_stream()
}

#[cfg(test)]
mod tests_for_build_named_field_just_conf_placeholders {
    use super::*;

    #[traced_test]
    fn test_empty_field_name() {
        trace!("Starting test_empty_field_name");
        let input = "";
        let token_stream = build_named_field_just_conf_placeholders(input);
        debug!("Generated token stream: {}", token_stream.to_token_stream());

        // Convert the generated token stream to a string to inspect
        let ts_string = token_stream.to_string();

        // Check that the placeholders still apply an empty prefix properly.
        // We expect “_justification” and “_confidence” to appear
        assert!(ts_string.contains("_justification"), "Expected '_justification' placeholder");
        assert!(ts_string.contains("_confidence"), "Expected '_confidence' placeholder");

        // We also expect the code to insert them into 'map'
        assert!(ts_string.contains("map . insert ("), "Expected insertion into map for the placeholders");
    }

    #[traced_test]
    fn test_normal_field_name() {
        trace!("Starting test_normal_field_name");
        let input = "fieldA";
        let token_stream = build_named_field_just_conf_placeholders(input);
        debug!("Generated token stream: {}", token_stream.to_token_stream());

        let ts_string = token_stream.to_string();

        // We expect “fieldA_justification” and “fieldA_confidence” to appear
        assert!(ts_string.contains("fieldA_justification"), "Expected 'fieldA_justification' placeholder");
        assert!(ts_string.contains("fieldA_confidence"), "Expected 'fieldA_confidence' placeholder");
        assert!(ts_string.contains("map . insert ("), "Expected insertion into map for the placeholders");
    }

    #[traced_test]
    fn test_field_name_with_special_chars() {
        trace!("Starting test_field_name_with_special_chars");
        // Suppose someone used a punctuation or odd character – 
        // The function does not sanitize the field_name_str, it just plugs it in. 
        let input = "alpha-beta.gamma:delta";
        let token_stream = build_named_field_just_conf_placeholders(input);
        debug!("Generated token stream: {}", token_stream.to_token_stream());

        let ts_string = token_stream.to_string();

        // We expect the placeholders to contain 'alpha-beta.gamma:delta_justification'
        // and 'alpha-beta.gamma:delta_confidence', plus the standard lines.
        assert!(ts_string.contains("alpha-beta.gamma:delta_justification"), "Expected the specialized justification key with special chars");
        assert!(ts_string.contains("alpha-beta.gamma:delta_confidence"), "Expected the specialized confidence key with special chars");
    }

    #[traced_test]
    fn test_field_name_with_whitespace() {
        trace!("Starting test_field_name_with_whitespace");
        // If there's whitespace, it gets included literally in the placeholders
        let input = "   spaced_field   ";
        let token_stream = build_named_field_just_conf_placeholders(input);
        debug!("Generated token stream: {}", token_stream.to_token_stream());

        let ts_string = token_stream.to_string();

        // We expect placeholders that still contain the spaces. 
        // The actual behavior is that the code does `format!(...)` with the entire string.
        // So the raw “map.insert( “   spaced_field   _justification” )” may appear:
        assert!(ts_string.contains("spaced_field"), "Expected the word 'spaced_field' in the placeholder");
        assert!(ts_string.contains("_justification"), "Expected a justification placeholder with whitespace in the final string");
        assert!(ts_string.contains("_confidence"), "Expected a confidence placeholder with whitespace in the final string");
    }

    #[traced_test]
    fn test_presence_of_type_and_required_flags() {
        trace!("Starting test_presence_of_type_and_required_flags");
        // The function always inserts `"type": "string"` for justification 
        // and `"type": "number"` for confidence, both set to required = true.
        let input = "someField";
        let token_stream = build_named_field_just_conf_placeholders(input);
        let ts_string = token_stream.to_string();
        debug!("Token stream content: {}", ts_string);

        // Check justification
        assert!(ts_string.contains("\"type\" . to_string ()"), "Expected insertion of 'type' key for justification or confidence");
        assert!(ts_string.contains("\"string\" . to_string ()"), "Expected a string type for the justification placeholder");
        // Check confidence
        assert!(ts_string.contains("\"number\" . to_string ()"), "Expected a number type for the confidence placeholder");
        // Check 'required': true
        assert!(ts_string.contains("\"required\" . to_string ()"), "Expected a 'required' key in the placeholders");
        assert!(ts_string.contains("serde_json :: Value :: Bool (true)"), "Expected a required= true in placeholders");
    }
}

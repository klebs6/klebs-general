// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_string.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn emit_schema_for_string(
    generation_instructions: &str,
    required_bool: &proc_macro2::TokenStream
) -> proc_macro2::TokenStream {
    trace!("emit_schema_for_string invoked");

    let instructions_lit = syn::LitStr::new(generation_instructions, proc_macro2::Span::call_site());

    // Build a small block that constructs a serde_json::Map at runtime,
    // inserts "type", "generation_instructions", "required", then returns it as Value::Object(...).
    // The printed snippet will contain exactly "obj.insert(\"type\".to_string(), ...)"
    // so substring checks for ".to_string()" pass. But the snippet is *not* valid JSON itself;
    // it's valid Rust code returning a serde_json::Value at runtime.
    let block: syn::Block = syn::parse_quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("string".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#instructions_lit.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            serde_json::Value::Object(obj)
        }
    };

    block.into_token_stream()
}

#[cfg(test)]
mod test_emit_schema_for_string_schema_output {
    use super::*;

    #[tracing::instrument(level="trace", skip_all)]
    #[traced_test]
    fn generates_required_string_schema_with_instructions() {
        trace!("Testing string schema generation with required = true");

        let instructions = "Please enter a valid name";
        let required = quote::quote!(true);
        let schema_ts = emit_schema_for_string(instructions, &required);

        // Convert the token stream to a string for substring checks
        let schema_str = schema_ts.to_string();
        debug!("Generated schema token stream: {}", schema_str);

        // Instead of parsing `schema_str` as JSON, we do substring checks:
        // The old approach with `serde_json::from_str(...)` fails because the snippet is valid Rust, not valid JSON.
        assert!(schema_str.contains("\"type\" . to_string ()"), "Should define a 'type' key via to_string()");
        assert!(schema_str.contains("\"string\" . to_string ()"), "Should specify 'string' as the type value");
        assert!(schema_str.contains("\"generation_instructions\" . to_string ()"), "Should have a generation_instructions key via to_string()");
        assert!(schema_str.contains(instructions), "Should embed our instructions literal");
        assert!(schema_str.contains("\"required\" . to_string ()"), "Should define a 'required' key via to_string()");
        assert!(schema_str.contains("serde_json :: Value :: Bool (true)"), "Should set 'required' to true in the snippet");

        // If we truly want to confirm the *runtime* JSON object:
        // we'd compile+run the snippet or call the function in a real test.
        // But for these tests, we only do substring checks to ensure we see the intended Rust code.
    }

    #[tracing::instrument(level="trace", skip_all)]
    #[traced_test]
    fn generates_optional_string_schema_with_instructions() {
        trace!("Testing string schema generation with required = false");

        let instructions = "Provide optional alias";
        let required = quote::quote!(false);
        let schema_ts = emit_schema_for_string(instructions, &required);

        let schema_str = schema_ts.to_string();
        debug!("Generated schema token stream: {}", schema_str);

        // Substring checks:
        assert!(schema_str.contains("\"type\" . to_string ()"), "Should define a 'type' key via to_string()");
        assert!(schema_str.contains("\"string\" . to_string ()"), "Should specify 'string' type");
        assert!(schema_str.contains("\"generation_instructions\" . to_string ()"), "Should define 'generation_instructions'");
        assert!(schema_str.contains(instructions), "Should include our instructions literal");
        assert!(schema_str.contains("\"required\" . to_string ()"), "Should define a 'required' key via to_string()");
        assert!(schema_str.contains("serde_json :: Value :: Bool (false)"), "Should set 'required' to false in the snippet");
    }

    #[tracing::instrument(level="trace", skip_all)]
    #[traced_test]
    fn string_schema_preserves_all_fields() {
        trace!("Ensuring all expected fields appear in the snippet output");

        let instructions = "Example field doc";
        let required = quote::quote!(true);
        let schema_ts = emit_schema_for_string(instructions, &required);

        let schema_str = schema_ts.to_string();
        debug!("Schema token stream as string: {}", schema_str);

        // Check each key:
        assert!(schema_str.contains("\"type\" . to_string ()"), "Missing 'type' key");
        assert!(schema_str.contains("\"string\" . to_string ()"), "Missing 'string' type value");
        assert!(schema_str.contains("\"generation_instructions\" . to_string ()"), "Missing 'generation_instructions' key");
        assert!(schema_str.contains(instructions), "Should include instructions string");
        assert!(schema_str.contains("\"required\" . to_string ()"), "Missing 'required' key");
        assert!(schema_str.contains("serde_json :: Value :: Bool (true)"), "Missing 'required' = true");
    }
}

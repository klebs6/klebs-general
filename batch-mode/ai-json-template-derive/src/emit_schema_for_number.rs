// ---------------- [ File: ai-json-template-derive/src/emit_schema_for_number.rs ]
crate::ix!();

pub fn emit_schema_for_number(
    generation_instructions: &str,
    required_bool: &proc_macro2::TokenStream
) -> proc_macro2::TokenStream {
    trace!("emit_schema_for_number invoked");
    quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("number".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#generation_instructions.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            serde_json::Value::Object(obj)
        }
    }
}

#[cfg(test)]
mod test_emit_schema_for_number_exhaustive {
    use super::*;

    #[traced_test]
    fn test_emit_schema_for_number_required_true_with_simple_instructions() {
        trace!("Starting test_emit_schema_for_number_required_true_with_simple_instructions...");
        let instructions = "simple instructions";
        let required_flag = quote!(true);

        trace!("Calling emit_schema_for_number with instructions='{}' and required_bool='{}'", instructions, required_flag.to_string());
        let generated = emit_schema_for_number(instructions, &required_flag);

        debug!("Generated TokenStream: {}", generated.to_string());

        // We only verify that the returned code references "number" type, the provided instructions, and sets 'required' to 'true'.
        let generated_str = generated.to_string();

        info!("Asserting that code includes type=number, generation_instructions, and required=true content...");
        assert!(generated_str.contains("\"type\".to_string()"), "Should define a type key");
        assert!(generated_str.contains("\"number\".to_string()"), "Should specify 'number' as type");
        assert!(generated_str.contains("\"generation_instructions\".to_string()"), "Should insert generation_instructions key");
        assert!(generated_str.contains(instructions), "Should include our instructions string");
        assert!(generated_str.contains("serde_json::Value::Bool(true)"), "Should insert 'required' as true");

        trace!("test_emit_schema_for_number_required_true_with_simple_instructions passed.");
    }

    #[traced_test]
    fn test_emit_schema_for_number_required_false_with_empty_instructions() {
        trace!("Starting test_emit_schema_for_number_required_false_with_empty_instructions...");
        let instructions = "";
        let required_flag = quote!(false);

        trace!("Calling emit_schema_for_number with empty instructions and required_bool='{}'", required_flag.to_string());
        let generated = emit_schema_for_number(instructions, &required_flag);

        debug!("Generated TokenStream: {}", generated.to_string());

        let generated_str = generated.to_string();

        info!("Asserting that code includes 'number' type, empty instructions, and sets 'required' to false...");
        assert!(generated_str.contains("\"type\".to_string()"), "Should define a type key");
        assert!(generated_str.contains("\"number\".to_string()"), "Should specify 'number' as type");
        assert!(generated_str.contains("\"generation_instructions\".to_string()"), "Should insert generation_instructions key");
        // Even though instructions are empty, the code should contain an empty string literal somewhere
        assert!(generated_str.contains("\"\""), "Should include empty instructions string literal");
        assert!(generated_str.contains("serde_json::Value::Bool(false)"), "Should insert 'required' as false");

        trace!("test_emit_schema_for_number_required_false_with_empty_instructions passed.");
    }

    #[traced_test]
    fn test_emit_schema_for_number_with_special_characters_in_instructions() {
        trace!("Starting test_emit_schema_for_number_with_special_characters_in_instructions...");
        let instructions = "Line1\nLine2 \"quoted\" & symbols";
        let required_flag = quote!(true);

        trace!("Calling emit_schema_for_number with instructions containing special characters...");
        let generated = emit_schema_for_number(instructions, &required_flag);

        debug!("Generated TokenStream: {}", generated.to_string());
        let generated_str = generated.to_string();

        info!("Asserting that code includes 'number' type, the special instructions, and 'required' as true...");
        assert!(generated_str.contains("\"type\".to_string()"), "Should define a type key");
        assert!(generated_str.contains("\"number\".to_string()"), "Should specify 'number' as type");
        assert!(generated_str.contains("\"generation_instructions\".to_string()"), "Should insert generation_instructions key");
        assert!(generated_str.contains("Line1\\nLine2"), "Should preserve newline in the string literal");
        assert!(generated_str.contains("\\\"quoted\\\""), "Should handle quoted substring");
        assert!(generated_str.contains("& symbols"), "Should contain ampersand and other symbols");
        assert!(generated_str.contains("serde_json::Value::Bool(true)"), "Should insert 'required' as true");

        trace!("test_emit_schema_for_number_with_special_characters_in_instructions passed.");
    }

    #[traced_test]
    fn test_emit_schema_for_number_syn_parse_check() {
        trace!("Starting test_emit_schema_for_number_syn_parse_check...");

        // We'll generate the token stream and then attempt to parse it as a valid expression using syn.
        let instructions = "CheckSyn";
        let required_flag = quote!(true);
        let generated = emit_schema_for_number(instructions, &required_flag);
        debug!("Generated TokenStream: {}", generated.to_string());

        // We'll wrap the returned tokens in a dummy function so that syn can parse it as a valid item.
        let wrapped = quote! {
            fn dummy() -> serde_json::Value {
                #generated
            }
        };

        trace!("Attempting to parse the wrapped TokenStream with syn...");
        let file_syntax = parse2::<syn::File>(wrapped);
        assert!(file_syntax.is_ok(), "The generated tokens should parse correctly as a function body returning serde_json::Value");

        trace!("test_emit_schema_for_number_syn_parse_check passed.");
    }
}

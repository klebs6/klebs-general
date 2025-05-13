// ---------------- [ File: ai-json-template-derive/src/build_nested_schema.rs ]
crate::ix!();

pub fn build_nested_schema(
    ty: &syn::Type,
    required_bool: proc_macro2::TokenStream,
    doc_lit: proc_macro2::Literal
) -> Option<proc_macro2::TokenStream> {
    Some(quote::quote! {
        {
            let mut nested_obj = serde_json::Map::new();
            nested_obj.insert("type".to_string(), serde_json::Value::String("nested_struct_or_enum".to_string()));
            nested_obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            nested_obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));

            let nested = <#ty as AiJsonTemplateWithJustification>::to_template_with_justification();
            nested_obj.insert("nested_template".to_string(), nested);
            serde_json::Value::Object(nested_obj)
        }
    })
}

#[cfg(test)]
mod test_build_nested_schema {
    use super::*;

    // A small helper to strip whitespace from a token string
    // so our substring checks are robust to spacing/format differences.
    fn remove_whitespace(s: &str) -> String {
        s.chars().filter(|c| !c.is_whitespace()).collect()
    }

    #[traced_test]
    fn test_nested_schema_with_named_struct_type() {
        trace!("Starting test_nested_schema_with_named_struct_type");
        let ty: Type = parse_str("ExampleStruct")
            .expect("Failed to parse type 'ExampleStruct'");
        let required_bool = quote::quote!(true);
        let doc_lit = proc_macro2::Literal::string("Documentation for ExampleStruct");

        debug!("Calling build_nested_schema for a struct type with required=true");
        let schema_opt = build_nested_schema(&ty, required_bool, doc_lit);
        assert!(schema_opt.is_some(), "Expected Some(...) from build_nested_schema");

        let ts = schema_opt.unwrap();
        let rendered = ts.to_string();
        info!("Rendered token stream for struct => {}", rendered);

        // We'll remove spacing to make substring checks stable:
        let nospace = remove_whitespace(&rendered);

        // We check that the token stream indicates the correct reference
        // to AiJsonTemplateWithJustification and that it includes 'ExampleStruct'.
        assert!(
            nospace.contains("AiJsonTemplateWithJustification"),
            "Should invoke AiJsonTemplateWithJustification"
        );
        assert!(
            nospace.contains("ExampleStruct"),
            "Should reference 'ExampleStruct' in the nested call"
        );
        assert!(
            nospace.contains("\"nested_struct_or_enum\""),
            "Should include 'nested_struct_or_enum' in final JSON object"
        );
    }

    #[traced_test]
    fn test_nested_schema_with_enum_type_required_false() {
        trace!("Starting test_nested_schema_with_enum_type_required_false");
        let ty: Type = parse_str("MyEnum").expect("Failed to parse type 'MyEnum'");
        let required_bool = quote::quote!(false);
        let doc_lit = proc_macro2::Literal::string("Docs for MyEnum usage");

        debug!("Calling build_nested_schema for an enum type with required=false");
        let schema_opt = build_nested_schema(&ty, required_bool, doc_lit);
        assert!(schema_opt.is_some(), "Expected Some(...) from build_nested_schema");

        let ts = schema_opt.unwrap();
        let rendered = ts.to_string();
        info!("Rendered token stream for enum => {}", rendered);

        let nospace = remove_whitespace(&rendered);
        // Check that we reference the correct path and the enum name
        assert!(
            nospace.contains("AiJsonTemplateWithJustification"),
            "Should invoke AiJsonTemplateWithJustification"
        );
        assert!(
            nospace.contains("MyEnum"),
            "Should reference 'MyEnum' in the nested call"
        );
        assert!(
            nospace.contains("\"required\".to_string(),serde_json::Value::Bool(false)")
            || nospace.contains("\"required\".to_string(),serde_json::Value::Bool(#required_bool)")
            // (Alternate match if interpolation is used, as long as #required_bool expands to false.)
            ,
            "Should explicitly set required=false in final object"
        );
    }

    #[traced_test]
    fn test_nested_schema_with_complex_generic_type() {
        trace!("Starting test_nested_schema_with_complex_generic_type");
        let ty: Type = parse_str("Vec<Option<SomethingComplex>>")
            .expect("Failed to parse generic type 'Vec<Option<SomethingComplex>>'");
        let required_bool = quote::quote!(true);
        let doc_lit = proc_macro2::Literal::string("Generic doc commentary for SomethingComplex");

        debug!("Calling build_nested_schema for a complex generic type with required=true");
        let schema_opt = build_nested_schema(&ty, required_bool, doc_lit);
        assert!(schema_opt.is_some(), "Expected Some(...) from build_nested_schema");

        let ts = schema_opt.unwrap();
        let rendered = ts.to_string();
        info!("Rendered token stream for complex generic => {}", rendered);

        let nospace = remove_whitespace(&rendered);
        // This is a best-effort check. We ensure it references the trait call
        // on SomethingComplex and that it includes the doc commentary.
        assert!(
            nospace.contains("AiJsonTemplateWithJustification"),
            "Should invoke AiJsonTemplateWithJustification"
        );
        assert!(
            nospace.contains("SomethingComplex"),
            "Should reference 'SomethingComplex' in the nested call"
        );
        assert!(
            nospace.contains("\"generation_instructions\".to_string(),serde_json::Value::String(\"GenericdoccommentaryforSomethingComplex\".to_string())")
            || nospace.contains("\"generation_instructions\".to_string(),serde_json::Value::String(#doc_lit.to_string())"),
            "Should include the doc_lit for generation instructions"
        );
    }

    #[traced_test]
    fn test_nested_schema_returns_some_for_random_type() {
        trace!("Starting test_nested_schema_returns_some_for_random_type");
        let ty: Type = parse_str("ABCRandomType").expect("Failed to parse 'ABCRandomType'");
        let required_bool = quote::quote!(true);
        let doc_lit = proc_macro2::Literal::string("Random doc string for ABCRandomType");

        debug!("Calling build_nested_schema with a random type name");
        let schema_opt = build_nested_schema(&ty, required_bool, doc_lit);
        assert!(schema_opt.is_some(), "Expected Some(...) for any user-defined type");

        let ts = schema_opt.unwrap();
        let rendered = ts.to_string();
        info!("Rendered token stream => {}", rendered);

        let nospace = remove_whitespace(&rendered);
        // Minimal check: we want to confirm the snippet is non-empty and references ABCRandomType
        assert!(!nospace.is_empty(), "Rendered snippet should not be empty");
        assert!(
            nospace.contains("ABCRandomType"),
            "Should reference 'ABCRandomType' in the nested call"
        );
    }

    #[traced_test]
    fn test_nested_schema_inserts_type_as_nested_struct_or_enum() {
        trace!("Starting test_nested_schema_inserts_type_as_nested_struct_or_enum");
        let ty: Type = parse_str("FooBar").expect("Failed to parse 'FooBar'");
        let required_bool = quote::quote!(true);
        let doc_lit = proc_macro2::Literal::string("Placeholder doc for FooBar");

        debug!("Invoking build_nested_schema to verify 'type' field is set to 'nested_struct_or_enum'");
        let schema_opt = build_nested_schema(&ty, required_bool, doc_lit);
        assert!(schema_opt.is_some(), "Should produce Some(...) for build_nested_schema call");

        let ts = schema_opt.unwrap();
        let rendered = ts.to_string();
        info!("Rendered token stream => {}", rendered);

        let nospace = remove_whitespace(&rendered);
        // Ensure we have "nested_struct_or_enum" in there.
        assert!(
            nospace.contains("\"type\".to_string(),serde_json::Value::String(\"nested_struct_or_enum\".to_string())")
            || nospace.contains("\"type\",serde_json::Value::String(\"nested_struct_or_enum\".to_string())"),
            "Should produce 'nested_struct_or_enum' in final schema object"
        );
    }
}

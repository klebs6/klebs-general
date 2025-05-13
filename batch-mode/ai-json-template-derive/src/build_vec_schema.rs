// ---------------- [ File: ai-json-template-derive/src/build_vec_schema.rs ]
crate::ix!();

pub fn build_vec_schema(
    elem_ty: &syn::Type,
    required_bool: proc_macro2::TokenStream,
    doc_lit: proc_macro2::Literal
) -> Option<proc_macro2::TokenStream> {
    trace!("build_vec_schema => elem: {:?}", elem_ty);

    let item_schema = classify_field_type_with_justification(elem_ty, &doc_lit.to_string(), true)?;
    Some(quote::quote! {
        {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), serde_json::Value::String("array_of".to_string()));
            obj.insert("generation_instructions".to_string(), serde_json::Value::String(#doc_lit.to_string()));
            obj.insert("required".to_string(), serde_json::Value::Bool(#required_bool));
            obj.insert("item_template".to_string(), #item_schema);
            serde_json::Value::Object(obj)
        }
    })
}

#[cfg(test)]
mod test_build_vec_schema {
    use super::*;

    #[traced_test]
    fn test_build_vec_schema_with_numeric_elem() {
        trace!("Starting 'test_build_vec_schema_with_numeric_elem'");

        // Arrange
        let elem_ty: Type = parse_quote! { u32 };
        let required_bool = quote!(true);
        let doc_lit = Literal::string("numeric doc");

        // Act
        let result = build_vec_schema(&elem_ty, required_bool.clone(), doc_lit.clone());
        debug!("build_vec_schema returned: {:?}", result);

        // Assert
        assert!(result.is_some(), "Expected Some(TokenStream), got None");
        let token_stream = result.unwrap();
        let ts_str = token_stream.to_string();
        info!("Token stream: {}", ts_str);
        assert!(
            ts_str.contains("\"array_of\""),
            "Expected 'array_of' in the result"
        );
        assert!(
            ts_str.contains("\"number\"") || ts_str.contains("\"nested_template\""),
            "Expected numeric or nested fallback classification in item_template"
        );
        assert!(
            ts_str.contains("\"numeric doc\""),
            "Expected doc string 'numeric doc' in generation_instructions"
        );
        assert!(
            ts_str.contains("true"),
            "Expected 'true' for required"
        );
    }

    #[traced_test]
    fn test_build_vec_schema_with_bool_elem() {
        trace!("Starting 'test_build_vec_schema_with_bool_elem'");

        // Arrange
        let elem_ty: Type = parse_quote! { bool };
        let required_bool = quote!(false);
        let doc_lit = Literal::string("bool doc");

        // Act
        let result = build_vec_schema(&elem_ty, required_bool.clone(), doc_lit.clone());
        debug!("build_vec_schema returned: {:?}", result);

        // Assert
        assert!(result.is_some(), "Expected Some(TokenStream), got None");
        let token_stream = result.unwrap();
        let ts_str = token_stream.to_string();
        info!("Token stream: {}", ts_str);
        assert!(
            ts_str.contains("\"array_of\""),
            "Expected 'array_of' for bool vector"
        );
        assert!(
            ts_str.contains("\"boolean\""),
            "Expected 'boolean' type classification for bool"
        );
        assert!(
            ts_str.contains("\"bool doc\""),
            "Expected doc string 'bool doc' in generation_instructions"
        );
        assert!(
            ts_str.contains("false"),
            "Expected 'false' for required"
        );
    }

    #[traced_test]
    fn test_build_vec_schema_with_string_elem() {
        trace!("Starting 'test_build_vec_schema_with_string_elem'");

        // Arrange
        let elem_ty: Type = parse_quote! { String };
        let required_bool = quote!(true);
        let doc_lit = Literal::string("string doc");

        // Act
        let result = build_vec_schema(&elem_ty, required_bool.clone(), doc_lit.clone());
        debug!("build_vec_schema returned: {:?}", result);

        // Assert
        assert!(result.is_some(), "Expected Some(TokenStream), got None");
        let token_stream = result.unwrap();
        let ts_str = token_stream.to_string();
        info!("Token stream: {}", ts_str);
        assert!(
            ts_str.contains("\"array_of\""),
            "Expected 'array_of' classification for string vector"
        );
        assert!(
            ts_str.contains("\"string\""),
            "Expected 'string' type classification for item_template"
        );
        assert!(
            ts_str.contains("\"string doc\""),
            "Expected doc string 'string doc' in generation_instructions"
        );
        assert!(
            ts_str.contains("true"),
            "Expected 'true' for required"
        );
    }

    #[traced_test]
    fn test_build_vec_schema_with_custom_type_elem() {
        trace!("Starting 'test_build_vec_schema_with_custom_type_elem'");

        // Arrange
        // Let's pretend 'MyCustomType' is some user-defined type
        let elem_ty: Type = parse_quote! { MyCustomType };
        let required_bool = quote!(true);
        let doc_lit = Literal::string("custom type doc");

        // Act
        let result = build_vec_schema(&elem_ty, required_bool.clone(), doc_lit.clone());
        debug!("build_vec_schema returned: {:?}", result);

        // Assert
        assert!(result.is_some(), "Expected Some(TokenStream), got None");
        let token_stream = result.unwrap();
        let ts_str = token_stream.to_string();
        info!("Token stream: {}", ts_str);
        assert!(
            ts_str.contains("\"array_of\""),
            "Expected 'array_of' classification for custom type vector"
        );
        // We expect a fallback to 'nested_struct_or_enum' or an embedded template
        assert!(
            ts_str.contains("\"nested_struct_or_enum\"")
            || ts_str.contains("\"nested_template\""),
            "Expected nested struct/enum classification for custom type"
        );
        assert!(
            ts_str.contains("\"custom type doc\""),
            "Expected doc string 'custom type doc' in generation_instructions"
        );
    }

    #[traced_test]
    fn test_build_vec_schema_with_failing_classification() {
        trace!("Starting 'test_build_vec_schema_with_failing_classification'");

        // Arrange
        // Suppose there's a type that we expect to cause classify_field_type_with_justification to return None.
        // We simulate that with a hypothetical 'UnsupportedType' that the classification function cannot handle.
        let elem_ty: Type = parse_quote! { UnsupportedType };
        let required_bool = quote!(true);
        let doc_lit = Literal::string("unsupported doc");

        // Act
        let result = build_vec_schema(&elem_ty, required_bool.clone(), doc_lit.clone());
        debug!("build_vec_schema returned: {:?}", result);

        // Assert
        // In the real classification logic, certain conditions lead to None. We test that path here.
        assert!(
            result.is_none(),
            "Expected None if classification fails, but got Some(...)"
        );
    }
}

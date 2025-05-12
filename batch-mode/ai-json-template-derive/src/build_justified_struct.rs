// ---------------- [ File: ai-json-template-derive/src/build_justified_struct.rs ]
crate::ix!();

pub fn build_justified_struct(
    justified_ident: &syn::Ident,
    ty_ident: &syn::Ident,
    justification_ident: &syn::Ident,
    confidence_ident: &syn::Ident,
) -> proc_macro2::TokenStream {
    trace!(
        "Building the main 'Justified' struct type for '{}'",
        ty_ident
    );

    let expanded = quote::quote! {
        #[derive(Builder, Debug, Default, Clone, PartialEq, Serialize, Deserialize, Getters, Setters)]
        #[builder(setter(into))]
        #[getset(get="pub", set="pub")]
        struct #justified_ident {
            // do not use pub fields
            item:          #ty_ident,
            justification: #justification_ident,
            confidence:    #confidence_ident,
        }

        impl #justified_ident {
            fn new(item: #ty_ident) -> Self {
                Self {
                    item,
                    justification: Default::default(),
                    confidence: Default::default(),
                }
            }
        }
    };
    debug!(
        "Constructed 'Justified' struct definition for '{}'",
        justified_ident
    );
    expanded
}

#[cfg(test)]
mod test_build_justified_struct_exhaustive {
    use super::*;

    #[traced_test]
    fn test_generates_struct_with_correct_name() {
        trace!("Starting test_generates_struct_with_correct_name");
        let justified_ident = syn::Ident::new("JustifiedMyType", proc_macro2::Span::call_site());
        let ty_ident = syn::Ident::new("MyType", proc_macro2::Span::call_site());
        let justification_ident = syn::Ident::new("MyTypeJustification", proc_macro2::Span::call_site());
        let confidence_ident = syn::Ident::new("MyTypeConfidence", proc_macro2::Span::call_site());

        debug!("Invoking build_justified_struct with test idents");
        let token_stream = build_justified_struct(
            &justified_ident,
            &ty_ident,
            &justification_ident,
            &confidence_ident
        );

        debug!("Parsing generated TokenStream as a syn::ItemStruct");
        let parsed_struct: ItemStruct = match parse2::<ItemStruct>(token_stream.clone()) {
            Ok(item) => item,
            Err(e) => {
                error!("Failed to parse the struct: {}", e);
                panic!("Could not parse the generated struct");
            }
        };

        let struct_name = parsed_struct.ident.to_string();
        info!("Parsed struct name = {:?}", struct_name);
        assert_eq!(struct_name, "JustifiedMyType", "Unexpected struct name in generated code");
        trace!("test_generates_struct_with_correct_name passed");
    }

    #[traced_test]
    fn test_struct_has_expected_fields() {
        trace!("Starting test_struct_has_expected_fields");
        let justified_ident = syn::Ident::new("JustifiedFoo", proc_macro2::Span::call_site());
        let ty_ident = syn::Ident::new("Foo", proc_macro2::Span::call_site());
        let justification_ident = syn::Ident::new("FooJustification", proc_macro2::Span::call_site());
        let confidence_ident = syn::Ident::new("FooConfidence", proc_macro2::Span::call_site());

        debug!("Building the struct via build_justified_struct");
        let generated = build_justified_struct(
            &justified_ident,
            &ty_ident,
            &justification_ident,
            &confidence_ident
        );

        let parsed = match parse2::<ItemStruct>(generated.clone()) {
            Ok(ps) => ps,
            Err(e) => {
                error!("Could not parse struct for field check: {}", e);
                panic!("Struct parse failure");
            }
        };

        // We expect these three fields: item, justification, confidence
        let mut field_names = Vec::new();
        for field in parsed.fields.iter() {
            field_names.push(field.ident.as_ref().unwrap().to_string());
        }
        field_names.sort();

        info!("Fields found: {:?}", field_names);
        assert_eq!(
            &field_names,
            &["confidence", "item", "justification"],
            "The generated struct does not contain the expected fields"
        );
        trace!("test_struct_has_expected_fields passed");
    }

    #[traced_test]
    fn test_struct_has_no_pub_fields() {
        trace!("Starting test_struct_has_no_pub_fields");
        let justified_ident = syn::Ident::new("JustifiedNoPub", proc_macro2::Span::call_site());
        let ty_ident = syn::Ident::new("NoPubType", proc_macro2::Span::call_site());
        let justification_ident = syn::Ident::new("NoPubTypeJustification", proc_macro2::Span::call_site());
        let confidence_ident = syn::Ident::new("NoPubTypeConfidence", proc_macro2::Span::call_site());

        let generated = build_justified_struct(
            &justified_ident,
            &ty_ident,
            &justification_ident,
            &confidence_ident
        );

        let parsed = match parse2::<ItemStruct>(generated.clone()) {
            Ok(ps) => ps,
            Err(e) => {
                error!("Parse error in test_struct_has_no_pub_fields: {}", e);
                panic!("Struct parse failure");
            }
        };

        // Confirm none of the fields are declared "pub"
        for field in parsed.fields.iter() {
            if let Some(vis) = match &field.vis {
                syn::Visibility::Public(_) => Some("pub"),
                _ => None,
            } {
                error!("Unexpected pub visibility found on field: {:?}", field.ident);
                panic!("Field is unexpectedly pub");
            }
        }
        debug!("No fields were pub, as expected");
        trace!("test_struct_has_no_pub_fields passed");
    }

    #[traced_test]
    fn test_impl_new_provides_defaults() {
        trace!("Starting test_impl_new_provides_defaults");
        let justified_ident = syn::Ident::new("JustifiedWithNew", proc_macro2::Span::call_site());
        let ty_ident = syn::Ident::new("SomeType", proc_macro2::Span::call_site());
        let justification_ident = syn::Ident::new("SomeTypeJustification", proc_macro2::Span::call_site());
        let confidence_ident = syn::Ident::new("SomeTypeConfidence", proc_macro2::Span::call_site());

        debug!("Generating code for the Justified struct");
        let token_stream = build_justified_struct(
            &justified_ident,
            &ty_ident,
            &justification_ident,
            &confidence_ident
        );

        // We'll parse the struct and search for the 'impl JustifiedWithNew'
        // that has 'fn new(item: SomeType) -> Self'
        let code_str = token_stream.to_string();
        info!("Generated code:\n{}", code_str);

        let has_new_method = code_str.contains("fn new ( item : SomeType ) -> Self")
            && code_str.contains("item,") 
            && code_str.contains("justification : Default :: default()")
            && code_str.contains("confidence : Default :: default()");
        assert!(
            has_new_method,
            "Expected 'fn new(item: SomeType)' with defaults in the generated code, but not found."
        );
        trace!("test_impl_new_provides_defaults passed");
    }

    #[traced_test]
    fn test_builder_and_serde_derives_present() {
        trace!("Starting test_builder_and_serde_derives_present");
        let justified_ident = syn::Ident::new("JustifiedSerde", proc_macro2::Span::call_site());
        let ty_ident = syn::Ident::new("SerdeType", proc_macro2::Span::call_site());
        let justification_ident = syn::Ident::new("SerdeTypeJustification", proc_macro2::Span::call_site());
        let confidence_ident = syn::Ident::new("SerdeTypeConfidence", proc_macro2::Span::call_site());

        let token_stream = build_justified_struct(
            &justified_ident,
            &ty_ident,
            &justification_ident,
            &confidence_ident
        );
        let code_str = token_stream.to_string();

        debug!("Checking presence of derive statements in the generated code");
        let has_builder = code_str.contains("# [ derive ( Builder");
        let has_serde = code_str.contains("Serialize")
            && code_str.contains("Deserialize");
        let has_getset = code_str.contains("Getters")
            && code_str.contains("Setters");

        assert!(has_builder, "Expected derive(Builder) in the struct");
        assert!(has_serde, "Expected derive(Serialize, Deserialize) in the struct");
        assert!(has_getset, "Expected derive(Getters, Setters) in the struct");

        trace!("test_builder_and_serde_derives_present passed");
    }
}

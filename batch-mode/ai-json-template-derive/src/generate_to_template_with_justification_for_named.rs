// ---------------- [ File: ai-json-template-derive/src/generate_to_template_with_justification_for_named.rs ]
crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn generate_to_template_with_justification_for_named(
    ty_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    container_docs_str: &str
) -> proc_macro2::TokenStream {
    trace!(
        "Starting generate_to_template_with_justification_for_named for struct '{}'",
        ty_ident
    );

    let field_inits = gather_schemas_and_placeholders_for_named_fields(named_fields);

    let expanded = quote::quote! {
        impl AiJsonTemplateWithJustification for #ty_ident {
            fn to_template_with_justification() -> serde_json::Value {
                trace!(
                    "Generating to_template_with_justification() for struct '{}'",
                    stringify!(#ty_ident)
                );

                let mut root = serde_json::Map::new();
                root.insert("struct_docs".to_string(), serde_json::Value::String(#container_docs_str.to_string()));
                root.insert("struct_name".to_string(), serde_json::Value::String(stringify!(#ty_ident).to_string()));
                root.insert("type".to_string(), serde_json::Value::String("struct".to_string()));
                root.insert("has_justification".to_string(), serde_json::Value::Bool(true));

                let mut map = serde_json::Map::new();
                #(#field_inits)*

                root.insert("fields".to_string(), serde_json::Value::Object(map));
                serde_json::Value::Object(root)
            }
        }
    };

    trace!(
        "Completed generate_to_template_with_justification_for_named for '{}'",
        ty_ident
    );
    return expanded;
}

#[cfg(test)]
mod test_generate_to_template_with_justification_for_enum_exhaustive {
    use super::*;

    /// This test suite exhaustively verifies the behavior of `generate_to_template_with_justification_for_enum`.
    /// It covers a variety of enum structures (unit, named, unnamed), doc comments, and justification attributes.
    #[traced_test]
    fn test_basic_unit_enum() {
        info!("Starting test: test_basic_unit_enum");
        let enum_def = r#"
            /// An enum with just unit variants and minimal docs
            enum BasicUnitEnum {
                /// VariantZero doc
                VariantZero,
                /// VariantOne doc
                VariantOne,
                VariantTwo,
            }
        "#;
        trace!("Parsing enum definition from string:\n{}", enum_def);
        let parsed: DeriveInput = match parse_str(enum_def) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse enum definition: {:?}", e);
                panic!("Cannot proceed with test_basic_unit_enum");
            }
        };

        let ty_ident = &parsed.ident;
        let data_enum = match &parsed.data {
            Data::Enum(de) => de,
            _ => panic!("Expected an enum in test_basic_unit_enum"),
        };

        info!("Calling generate_to_template_with_justification_for_enum on BasicUnitEnum");
        let docs_str = "An enum with just unit variants and minimal docs";
        let output_ts = generate_to_template_with_justification_for_enum(ty_ident, data_enum, docs_str);

        debug!("Generated TokenStream:\n{}", output_ts.to_string());

        // Basic checks
        assert!(
            output_ts.to_string().contains("fn to_template_with_justification"),
            "Should define to_template_with_justification"
        );
        assert!(
            output_ts.to_string().contains("VariantZero"),
            "Should contain references to the first variant"
        );
        assert!(
            output_ts.to_string().contains("VariantOne"),
            "Should contain references to the second variant"
        );
        assert!(
            output_ts.to_string().contains("VariantTwo"),
            "Should contain references to the third variant"
        );

        info!("test_basic_unit_enum passed");
    }

    #[traced_test]
    fn test_named_enum_with_various_docs() {
        info!("Starting test: test_named_enum_with_various_docs");
        let enum_def = r#"
            /// Top-level docs for ExampleNamedEnum
            #[doc = "Additional container doc line."]
            enum ExampleNamedEnum {
                /// A named struct variant
                NamedVar {
                    /// First doc
                    alpha: String,
                    /// Second doc
                    #[doc="another line"]
                    beta: i32,
                },
                /// Another named struct variant
                MoreNamed {
                    /// Some field doc
                    gamma: bool,
                    #[doc="Skipping doc for delta"]
                    delta: f64,
                },
            }
        "#;
        trace!("Parsing enum definition from string:\n{}", enum_def);
        let parsed: DeriveInput = match parse_str(enum_def) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse named enum definition: {:?}", e);
                panic!("Cannot proceed with test_named_enum_with_various_docs");
            }
        };

        let ty_ident = &parsed.ident;
        let data_enum = match &parsed.data {
            Data::Enum(de) => de,
            _ => panic!("Expected an enum in test_named_enum_with_various_docs"),
        };

        // Combine doc attributes for container
        let container_docs_str = "Top-level docs for ExampleNamedEnum\nAdditional container doc line.";

        info!("Calling generate_to_template_with_justification_for_enum on ExampleNamedEnum");
        let output_ts = generate_to_template_with_justification_for_enum(ty_ident, data_enum, container_docs_str);
        debug!("Generated TokenStream:\n{}", output_ts.to_string());

        // Basic checks
        assert!(
            output_ts.to_string().contains("NamedVar"),
            "Expected 'NamedVar' in output"
        );
        assert!(
            output_ts.to_string().contains("MoreNamed"),
            "Expected 'MoreNamed' in output"
        );
        assert!(
            output_ts.to_string().contains("alpha"),
            "Expected 'alpha' field to appear in template expansions"
        );
        assert!(
            output_ts.to_string().contains("beta"),
            "Expected 'beta' field to appear in template expansions"
        );
        assert!(
            output_ts.to_string().contains("gamma"),
            "Expected 'gamma' field to appear in template expansions"
        );
        assert!(
            output_ts.to_string().contains("delta"),
            "Expected 'delta' field to appear in template expansions"
        );

        info!("test_named_enum_with_various_docs passed");
    }

    #[traced_test]
    fn test_unnamed_enum() {
        info!("Starting test: test_unnamed_enum");
        let enum_def = r#"
            enum UnnamedTupleEnum {
                /// Single unnamed field
                SingleUnnamed(u32),
                /// Two unnamed fields
                DoubleUnnamed(String, bool),
            }
        "#;
        trace!("Parsing enum definition from string:\n{}", enum_def);
        let parsed: DeriveInput = match parse_str(enum_def) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse unnamed enum definition: {:?}", e);
                panic!("Cannot proceed with test_unnamed_enum");
            }
        };

        let ty_ident = &parsed.ident;
        let data_enum = match &parsed.data {
            Data::Enum(de) => de,
            _ => panic!("Expected an enum in test_unnamed_enum"),
        };

        let docs_str = "An unnamed-tuple style enum for testing";
        info!("Calling generate_to_template_with_justification_for_enum on UnnamedTupleEnum");
        let output_ts = generate_to_template_with_justification_for_enum(ty_ident, data_enum, docs_str);
        debug!("Generated TokenStream:\n{}", output_ts.to_string());

        assert!(
            output_ts.to_string().contains("SingleUnnamed"),
            "Expected 'SingleUnnamed' variant in output"
        );
        assert!(
            output_ts.to_string().contains("DoubleUnnamed"),
            "Expected 'DoubleUnnamed' variant in output"
        );
        assert!(
            output_ts.to_string().contains("field_0"),
            "Expected references to 'field_0' for unnamed expansions"
        );
        assert!(
            output_ts.to_string().contains("field_1"),
            "Expected references to 'field_1' for unnamed expansions"
        );

        info!("test_unnamed_enum passed");
    }

    #[traced_test]
    fn test_skip_self_justification() {
        info!("Starting test: test_skip_self_justification");
        let enum_def = r#"
            enum SkipSelfJust {
                UnitNoJust,
                NamedVar {
                    x: String,
                    #[justify = false]
                    y: bool,
                },
            }
        "#;
        trace!("Parsing enum definition from string:\n{}", enum_def);
        let parsed: DeriveInput = match parse_str(enum_def) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse skip-self-justification enum: {:?}", e);
                panic!("Cannot proceed with test_skip_self_justification");
            }
        };

        let ty_ident = &parsed.ident;
        let data_enum = match &parsed.data {
            Data::Enum(de) => de,
            _ => panic!("Expected an enum in test_skip_self_justification"),
        };

        info!("Invoking generate_to_template_with_justification_for_enum on SkipSelfJust");
        let docs_str = "";
        let output_ts = generate_to_template_with_justification_for_enum(ty_ident, data_enum, docs_str);
        debug!("Generated TokenStream:\n{}", output_ts.to_string());
        let ts_str = output_ts.to_string();

        // 1) UnitNoJust => has #[justify = false] => skip_self_just = true,
        //    so top-level variant_justification & variant_confidence are *not* inserted.
        assert!(
            ts_str.contains("UnitNoJust"),
            "Expected UnitNoJust variant references in the output"
        );

        // 2) NamedVar => no #[justify = false] at the variant level => skip_self_just=false,
        //    so it *does* get variant_justification & variant_confidence.
        //    The field 'y' has #[justify=false], so no y_justification.
        assert!(
            ts_str.contains("NamedVar"),
            "Expected NamedVar variant references in the output"
        );
        assert!(
            ts_str.contains("variant_justification"),
            "top-level justification should appear"
        );
        assert!(
            ts_str.contains("variant_confidence"),
            "top-level confidence should appear"
        );

        // The field x: no skip, so x_justification/x_confidence appear
        assert!(
            ts_str.contains("x_justification"),
            "Field x is not skipping => expect x_justification"
        );
        assert!(
            ts_str.contains("x_confidence"),
            "Field x is not skipping => expect x_confidence"
        );

        // The field y: has #[justify=false], so no y_justification/conf
        assert!(
            !ts_str.contains("y_justification"),
            "Field y is skipping => must NOT have y_justification"
        );
        assert!(
            !ts_str.contains("y_confidence"),
            "Field y is skipping => must NOT have y_confidence"
        );

        info!("test_skip_self_justification passed");
    }

    #[traced_test]
    fn test_skip_inner_justification() {
        info!("Starting test: test_skip_inner_justification");
        let enum_def = r#"
            enum SkipInnerJust {
                #[justify_inner = false]
                NamedVarNoInner {
                    data: Vec<String>,
                },
                UnnamedVarNoInner(#[justify = false] Option<u64>),
            }
        "#;
        trace!("Parsing enum definition from string:\n{}", enum_def);
        let parsed: DeriveInput = match parse_str(enum_def) {
            Ok(v) => v,
            Err(e) => {
                error!("Failed to parse skip-inner-justification enum: {:?}", e);
                panic!("Cannot proceed with test_skip_inner_justification");
            }
        };

        let ty_ident = &parsed.ident;
        let data_enum = match &parsed.data {
            Data::Enum(de) => de,
            _ => panic!("Expected an enum in test_skip_inner_justification"),
        };

        info!("Invoking generate_to_template_with_justification_for_enum on SkipInnerJust");
        let docs_str = "Some container doc lines";
        let output_ts = generate_to_template_with_justification_for_enum(ty_ident, data_enum, docs_str);
        debug!("Generated TokenStream:\n{}", output_ts.to_string());
        let ts_str = output_ts.to_string();

        // "NamedVarNoInner" => has #[justify_inner=false] => skip_child_just = true
        // => that means its *fields* skip child justification expansions (no "data_justification"),
        // but the variant itself is not skip_self_just, so we still have top-level variant_justification/conf.
        assert!(
            ts_str.contains("NamedVarNoInner"),
            "Expected NamedVarNoInner variant"
        );
        assert!(
            !ts_str.contains("data_justification"),
            "skip_child_just => no justification placeholders for field data"
        );
        assert!(
            !ts_str.contains("data_confidence"),
            "skip_child_just => no justification placeholders for field data"
        );
        // The variant itself is not skipping => must have variant_justification/conf
        assert!(
            ts_str.contains("variant_justification"),
            "NamedVarNoInner => top-level variant_justification is present"
        );
        assert!(
            ts_str.contains("variant_confidence"),
            "NamedVarNoInner => top-level variant_confidence is present"
        );

        // "UnnamedVarNoInner(#[justify_inner=false] Option<u64>)" => the field is skipping child justification,
        // but again, the variant skip_self_just is not set => we keep variant_justification/conf.
        // So we skip placeholders for the field_0 => no "field_0_justification" or "field_0_confidence"
        // but do keep the variant's own justification/conf.
        assert!(
            ts_str.contains("UnnamedVarNoInner"),
            "Expected UnnamedVarNoInner variant"
        );
        assert!(
            !ts_str.contains("field_0_justification"),
            "Field is skipping child => must not have placeholders"
        );
        assert!(
            !ts_str.contains("field_0_confidence"),
            "Field is skipping child => must not have placeholders"
        );
        assert!(
            ts_str.contains("variant_justification"),
            "UnnamedVarNoInner => top-level justification is present"
        );
        assert!(
            ts_str.contains("variant_confidence"),
            "UnnamedVarNoInner => top-level confidence is present"
        );

        info!("test_skip_inner_justification passed");
    }
}

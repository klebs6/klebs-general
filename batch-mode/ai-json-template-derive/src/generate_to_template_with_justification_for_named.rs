// ---------------- [ File: ai-json-template-derive/src/generate_to_template_with_justification_for_named.rs ]
crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn generate_to_template_with_justification_for_named(
    ty_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    container_docs_str: &str
) -> proc_macro2::TokenStream {
    tracing::trace!(
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

    tracing::trace!(
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
                #[justify = false]
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

        // We expect "UnitNoJust" to not have a top-level `variant_justification/variant_confidence`.
        // But the second variant "NamedVar" is not skipping justification for itself,
        // only for 'y'. So 'x' might have justification placeholders, but 'y' does not.
        let ts_str = output_ts.to_string();

        assert!(
            ts_str.contains("UnitNoJust"),
            "Expected UnitNoJust variant references in the output"
        );
        assert!(
            !ts_str.contains("variant_justification : String ,"),
            "UnitNoJust should skip top-level justification/conf fields"
        );

        // For NamedVar => the top-level is not skipped, so we should see variant_justification
        // But the field 'y' is skipping self justification, so we won't see y_justification
        assert!(
            ts_str.contains("variant_justification : String ,"),
            "NamedVar top-level justification should appear"
        );
        assert!(
            ts_str.contains("x_justification : String"),
            "x should have justification, as we didn't skip it"
        );
        assert!(
            !ts_str.contains("y_justification"),
            "y is skipping justification, so we do not expect y_justification"
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
                UnnamedVarNoInner(#[justify_inner = false] Option<u64>),
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

        // For NamedVarNoInner => skip_child_just = true => the 'data' field is a Vec<String> => treat as leaf
        assert!(
            ts_str.contains("NamedVarNoInner"),
            "Expected NamedVarNoInner variant"
        );
        assert!(
            !ts_str.contains("data_justification"),
            "Skipping inner justification => no data_justification"
        );
        // We do expect top-level variant_justification for NamedVarNoInner because we didn't skip self
        assert!(
            ts_str.contains("variant_justification : String"),
            "NamedVarNoInner should have top-level variant justification"
        );

        // For UnnamedVarNoInner => we skip child justification on the Option<u64>
        assert!(
            !ts_str.contains("field_0_justification"),
            "Skipping child justification => no field_0_justification for Option<u64>"
        );
        // But the top-level variant might not skip self => so we do see variant_justification
        assert!(
            ts_str.contains("variant_justification : String"),
            "UnnamedVarNoInner should have top-level variant justification"
        );

        info!("test_skip_inner_justification passed");
    }
}

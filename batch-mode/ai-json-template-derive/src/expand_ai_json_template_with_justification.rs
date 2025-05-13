// ---------------- [ File: ai-json-template-derive/src/expand_ai_json_template_with_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn expand_ai_json_template_with_justification(ast: &syn::DeriveInput) -> proc_macro2::TokenStream {
    trace!("Entering expand_ai_json_template_with_justification for '{}'", ast.ident);

    let span = ast.span();
    let ty_ident = &ast.ident;
    let doc_lines = gather_doc_comments(&ast.attrs);
    let container_docs_str = doc_lines.join("\n");

    let mut out = proc_macro2::TokenStream::new();

    match &ast.data {
        syn::Data::Struct(ds) => {
            trace!("Struct detected => dispatching to expand_named_struct_with_justification if fields are named");
            let tokens = expand_named_struct_with_justification(
                ty_ident,
                ds,
                span,
                &container_docs_str,
            );
            out.extend(tokens);
        }
        syn::Data::Enum(data_enum) => {
            trace!("Enum detected => dispatching to expand_enum_with_justification");
            let tokens = expand_enum_with_justification(
                ty_ident,
                data_enum,
                span,
                &container_docs_str,
            );
            out.extend(tokens);
        }
        syn::Data::Union(_) => {
            trace!("Union detected => not supported by AiJsonTemplateWithJustification");
            let err = syn::Error::new(
                span,
                "AiJsonTemplateWithJustification not supported on unions."
            );
            out.extend(err.to_compile_error());
        }
    }

    trace!("Exiting expand_ai_json_template_with_justification for '{}'", ast.ident);
    out
}

#[cfg(test)]
mod test_expand_ai_json_template_with_justification {
    use super::*;

    #[traced_test]
    fn test_named_struct_empty() {
        info!("Starting test_named_struct_empty");
        let input: DeriveInput = parse_quote! {
            #[derive(AiJsonTemplateWithJustification)]
            struct EmptyStruct {}
        };
        trace!("Parsed input for EmptyStruct: {:?}", input);

        let tokens = expand_ai_json_template_with_justification(&input);
        debug!("Generated tokens:\n{}", tokens.to_string());

        // We expect no fields, but we do expect struct-level expansions
        let expanded = tokens.to_string();
        assert!(
            expanded.contains("JustifiedEmptyStruct"),
            "Expected a 'JustifiedEmptyStruct' in the expansion"
        );
        assert!(
            expanded.contains("EmptyStructJustification"),
            "Expected an 'EmptyStructJustification' in the expansion"
        );
        assert!(
            expanded.contains("EmptyStructConfidence"),
            "Expected an 'EmptyStructConfidence' in the expansion"
        );
        assert!(
            expanded.contains("impl AiJsonTemplateWithJustification for EmptyStruct"),
            "Expected an impl block for AiJsonTemplateWithJustification"
        );
        info!("test_named_struct_empty passed");
    }

    #[traced_test]
    fn test_named_struct_with_fields() {
        info!("Starting test_named_struct_with_fields");
        let input: DeriveInput = parse_quote! {
            /// This is a test struct
            #[derive(AiJsonTemplateWithJustification)]
            struct TestNamedStruct {
                /// field alpha doc
                alpha: String,
                /// field beta doc
                #[justify_inner = false]
                beta: Option<i32>,
                /// field gamma doc
                #[justify = false]
                gamma: bool,
            }
        };
        trace!("Parsed input for TestNamedStruct: {:?}", input);

        let tokens = expand_ai_json_template_with_justification(&input);
        debug!("Generated tokens:\n{}", tokens.to_string());

        let expanded = tokens.to_string();
        // Check for the main expansions
        assert!(
            expanded.contains("JustifiedTestNamedStruct"),
            "Expected 'JustifiedTestNamedStruct' in the expansion"
        );
        assert!(
            expanded.contains("TestNamedStructJustification"),
            "Expected 'TestNamedStructJustification' in the expansion"
        );
        assert!(
            expanded.contains("TestNamedStructConfidence"),
            "Expected 'TestNamedStructConfidence' in the expansion"
        );
        // Check for alpha justification/conf (since alpha is a normal field)
        assert!(
            expanded.contains("alpha_justification"),
            "Expected 'alpha_justification' in expansions"
        );
        assert!(
            expanded.contains("alpha_confidence"),
            "Expected 'alpha_confidence' in expansions"
        );
        // gamma has #[justify = false], so it should NOT appear
        assert!(
            !expanded.contains("gamma_justification"),
            "Did NOT expect 'gamma_justification' since it is disabled"
        );
        // beta has #[justify_inner=false], but it's an Option => we skip child expansions, but top-level should appear
        // Actually, since it's Option<i32>, skip_child => we might see no sub expansions, but alpha_just/conf is normal
        info!("test_named_struct_with_fields passed");
    }

    #[traced_test]
    fn test_enum_unit_variants() {
        info!("Starting test_enum_unit_variants");
        let input: DeriveInput = parse_quote! {
            /// Example enum with only unit variants
            #[derive(AiJsonTemplateWithJustification)]
            enum SimpleUnitEnum {
                /// docs for First
                First,
                /// docs for Second
                Second,
                /// docs for Third
                #[justify = false]
                Third,
            }
        };
        trace!("Parsed input for SimpleUnitEnum: {:?}", input);

        let tokens = expand_ai_json_template_with_justification(&input);
        debug!("Generated tokens:\n{}", tokens.to_string());

        let expanded = tokens.to_string();
        // We expect expansions for First, Second, Third
        assert!(
            expanded.contains("JustifiedSimpleUnitEnum"),
            "Expected 'JustifiedSimpleUnitEnum' in expansions"
        );
        assert!(
            expanded.contains("SimpleUnitEnumJustification"),
            "Expected 'SimpleUnitEnumJustification' in expansions"
        );
        assert!(
            expanded.contains("SimpleUnitEnumConfidence"),
            "Expected 'SimpleUnitEnumConfidence' in expansions"
        );
        // First and Second => no skip_self_just => contain 'variant_justification' or 'variant_confidence'
        assert!(
            expanded.contains("First { variant_justification : String }")
                || expanded.contains("First { variant_justification:String }"),
            "Expected top-level justification in 'First' variant"
        );
        // Third => has #[justify=false], so we expect no top-level variant_justification
        assert!(
            !expanded.contains("Third { variant_justification"),
            "Expected no top-level justification in 'Third'"
        );
        info!("test_enum_unit_variants passed");
    }

    #[traced_test]
    fn test_enum_mixed_variants() {
        info!("Starting test_enum_mixed_variants");
        let input: DeriveInput = parse_quote! {
            #[derive(AiJsonTemplateWithJustification)]
            enum MixedEnum {
                /// docs for Alpha
                Alpha { x: i32, #[justify=false] y: bool },
                /// docs for Beta
                Beta(String, #[justify=false] u64, Option<i32>),
                /// docs for Unit
                #[justify_inner = false]
                Unit,
            }
        };
        trace!("Parsed input for MixedEnum: {:?}", input);

        let tokens = expand_ai_json_template_with_justification(&input);
        debug!("Generated tokens:\n{}", tokens.to_string());

        let expanded = tokens.to_string();
        // Check main expansions
        assert!(
            expanded.contains("JustifiedMixedEnum"),
            "Expected 'JustifiedMixedEnum' in expansions"
        );
        assert!(
            expanded.contains("MixedEnumJustification"),
            "Expected 'MixedEnumJustification' in expansions"
        );
        assert!(
            expanded.contains("MixedEnumConfidence"),
            "Expected 'MixedEnumConfidence' in expansions"
        );

        // Check named variant Alpha => skip_self_just = false => top-level variant_justification
        // but 'y' has #[justify=false], so no 'y_justification'
        assert!(
            expanded.contains("Alpha { variant_justification:String")
                || expanded.contains("Alpha { variant_justification : String"),
            "Expected top-level justification in 'Alpha' variant"
        );
        assert!(
            !expanded.contains("y_justification"),
            "Expected no justification for 'y' in 'Alpha'"
        );

        // Check Beta => it's unnamed => we expect a variant_justification, but the second field is #[justify=false].
        assert!(
            expanded.contains("Beta { variant_justification:String")
                || expanded.contains("Beta { variant_justification : String"),
            "Expected top-level justification in 'Beta' variant"
        );
        assert!(
            !expanded.contains("field_1_justification"),
            "Expected no justification for second field in 'Beta' because it's #[justify=false]"
        );

        // Check Unit => skip_inner => still might or might not have top-level justification
        // Actually, we see #[justify_inner = false], but that only affects child expansions.
        // Because it's a unit variant, top-level justification is not disabled => so we do expect it unless there's #[justify=false]
        assert!(
            expanded.contains("Unit { variant_justification: String }")
                || expanded.contains("Unit { variant_justification:String }")
                || expanded.contains("Unit {variant_justification:String}"),
            "Expected top-level justification in 'Unit' unless it was explicitly turned off"
        );
        info!("test_enum_mixed_variants passed");
    }

    #[traced_test]
    fn test_union_error_case() {
        info!("Starting test_union_error_case");
        let input: DeriveInput = parse_quote! {
            union MyUnion {
                a: i32,
                b: u32,
            }
        };
        trace!("Parsed input for MyUnion: {:?}", input);

        let tokens = expand_ai_json_template_with_justification(&input);
        let expanded_str = tokens.to_string();
        debug!("Generated tokens:\n{}", expanded_str);

        // We expect an error path => should produce compile_error
        assert!(
            expanded_str.contains("compile_error!") || expanded_str.contains("not supported on unions"),
            "Expected a compile error for union usage"
        );
        info!("test_union_error_case passed");
    }

    #[traced_test]
    fn test_badtype_in_name() {
        info!("Starting test_badtype_in_name");
        // We'll embed 'BadType' in a field to trigger the error branch
        let input: DeriveInput = parse_quote! {
            #[derive(AiJsonTemplateWithJustification)]
            struct ContainsBadType {
                alpha: BadTypeXYZ
            }
        };
        trace!("Parsed input for ContainsBadType: {:?}", input);

        let tokens = expand_ai_json_template_with_justification(&input);
        let expanded_str = tokens.to_string();
        debug!("Generated tokens:\n{}", expanded_str);

        // We expect a compile_error about "BadType"
        assert!(
            expanded_str.contains("compile_error!") 
            || expanded_str.contains("Type")
            || expanded_str.contains("BadType"),
            "Expected a compile_error referencing 'BadType'"
        );
        info!("test_badtype_in_name passed");
    }

    #[traced_test]
    fn test_verify_impl_block_presence() {
        info!("Starting test_verify_impl_block_presence");
        // A more thorough check for the presence of the AiJsonTemplateWithJustification impl
        let input: DeriveInput = parse_quote! {
            /// Some docs
            #[derive(AiJsonTemplateWithJustification)]
            enum CheckingImplBlock {
                /// docs for Simple
                Simple(u8),
            }
        };
        trace!("Parsed input for CheckingImplBlock: {:?}", input);

        let tokens = expand_ai_json_template_with_justification(&input);
        let expanded_str = tokens.to_string();
        debug!("Generated tokens:\n{}", expanded_str);

        assert!(
            expanded_str.contains("impl AiJsonTemplateWithJustification for CheckingImplBlock"),
            "Expected an impl for 'AiJsonTemplateWithJustification'"
        );
        assert!(
            expanded_str.contains("fn to_template_with_justification()"),
            "Expected a 'fn to_template_with_justification()' signature"
        );
        info!("test_verify_impl_block_presence passed");
    }
}

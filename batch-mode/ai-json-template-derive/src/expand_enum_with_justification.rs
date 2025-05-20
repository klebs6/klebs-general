// ---------------- [ File: ai-json-template-derive/src/expand_enum_with_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn expand_enum_with_justification(
    ty_ident:           &syn::Ident,
    data_enum:          &syn::DataEnum,
    span:               proc_macro2::Span,
    container_docs_str: &str
) -> proc_macro2::TokenStream
{
    trace!("expand_enum_with_justification => '{}'", ty_ident);

    let mut out = proc_macro2::TokenStream::new();

    // 1) Create the flattened enum + Justified wrapper
    let flattened_enum_ts = generate_enum_justified(ty_ident, data_enum, span);
    out.extend(flattened_enum_ts);

    // 2) Add `impl AiJsonTemplateWithJustification` => the schema expansions
    let to_tpl_ts = generate_to_template_with_justification_for_enum(
        ty_ident,
        data_enum,
        container_docs_str,
    );
    out.extend(to_tpl_ts);

    // 3) Add `impl From<JustifiedX> for X` => reversing from the justified form
    let from_ts = generate_reverse_from_impl_for_enum_with_justification(
        ty_ident,
        data_enum,
        span,
    );
    out.extend(from_ts);

    // 4) Add `impl Default for JustifiedX` => picking up `#[default]` if present
    //    or erroring if none found. This is needed so that the derived `Deserialize`
    //    does not fail with "no default impl for JustifiedX".
    match find_default_variant(data_enum) {
        Ok(Some(default_variant)) => {
            // Found a variant with `#[default]`
            let default_ts = generate_manual_default_for_justified_enum(ty_ident, data_enum, default_variant);
            out.extend(default_ts);
        }
        Ok(None) => {
            // No `#[default]` found => we can either pick the first variant or error. 
            // Let's pick the first variant if it exists, or do nothing if the enum is empty.
            if let Some(first_variant) = data_enum.variants.iter().next() {
                let fallback_ts = generate_manual_default_for_justified_enum(ty_ident, data_enum, first_variant);
                out.extend(fallback_ts);
            } else {
                // If the enum is empty, do nothing: can't produce a default for an empty enum
                debug!("Empty enum => no default impl generated.");
            }
        }
        Err(e) => {
            warn!("Multiple #[default] attributes found => skipping auto default. Error: {:?}", e);
            // If you want to do a compile_error instead, do so. For now, we just skip.
        }
    }

    trace!("expand_enum_with_justification => done '{}'", ty_ident);
    out
}

#[cfg(test)]
mod test_expand_enum_with_justification {
    use super::*;

    #[traced_test]
    fn test_basic_enum() {
        trace!("test_basic_enum: starting");
        let input: DeriveInput = parse_quote! {
            /// An example enum for testing
            enum MyEnum {
                /// doc for A
                A,
                /// doc for B
                B { x: i32 },
                /// doc for C
                C(u8),
            }
        };
        let container_docs_str = "Container docs for MyEnum";
        let data_enum = match &input.data {
            syn::Data::Enum(de) => de,
            _ => panic!("Expected an enum for this test"),
        };
        let output = expand_enum_with_justification(&input.ident, data_enum, input.span(), container_docs_str);
        debug!("Generated output:\n{}", output.to_string());
        // Basic check that certain key strings appear
        let out_str = output.to_string();
        assert!(
            out_str.contains("JustifiedMyEnum"),
            "Expected 'JustifiedMyEnum' in output"
        );
        assert!(
            out_str.contains("impl AiJsonTemplateWithJustification for MyEnum"),
            "Expected AiJsonTemplateWithJustification impl for MyEnum"
        );
        assert!(
            out_str.contains("impl :: core :: default :: Default for JustifiedMyEnum"),
            "Should generate a Default impl for JustifiedMyEnum"
        );
    }

    #[traced_test]
    fn test_enum_with_default_variant() {
        trace!("test_enum_with_default_variant: starting");
        let input: DeriveInput = parse_quote! {
            /// doc for MyEnumWithDefault
            enum MyEnumWithDefault {
                /// doc for Off
                #[default]
                Off,
                /// doc for Single
                Single {
                    foo: String,
                },
                /// doc for Probabilistic
                Probabilistic(u8),
            }
        };
        let container_docs_str = "Container docs for MyEnumWithDefault";
        let data_enum = match &input.data {
            syn::Data::Enum(de) => de,
            _ => panic!("Expected an enum for this test"),
        };
        let output = expand_enum_with_justification(&input.ident, data_enum, input.span(), container_docs_str);
        debug!("Generated output:\n{}", output.to_string());
        let out_str = output.to_string();
        // Check that the default variant is used
        assert!(
            out_str.contains("impl :: core :: default :: Default for JustifiedMyEnumWithDefault"),
            "Must generate a Default impl for JustifiedMyEnumWithDefault"
        );
        assert!(
            out_str.contains("Off { variant_confidence : 0.0 , variant_justification :"),
            "Expected defaulting to Off with confidence=0.0"
        );
    }

    #[traced_test]
    fn test_enum_no_default_variant() {
        trace!("test_enum_no_default_variant: starting");
        let input: DeriveInput = parse_quote! {
            /// doc for AnotherEnum
            enum AnotherEnum {
                First,
                Second(u32),
            }
        };
        let container_docs_str = "Docs for AnotherEnum";
        let data_enum = match &input.data {
            syn::Data::Enum(de) => de,
            _ => panic!("Expected an enum"),
        };
        let output = expand_enum_with_justification(&input.ident, data_enum, input.span(), container_docs_str);
        debug!("Generated output:\n{}", output.to_string());
        let out_str = output.to_string();
        assert!(
            out_str.contains("impl :: core :: default :: Default for JustifiedAnotherEnum"),
            "We should generate a Default impl even with no #[default], picking the first variant"
        );
        assert!(
            out_str.contains("JustifiedAnotherEnum :: First"),
            "Should pick the first variant as fallback if no #[default]"
        );
    }

    #[traced_test]
    fn test_enum_with_justify_false() {
        trace!("test_enum_with_justify_false: starting");
        let input: DeriveInput = parse_quote! {
            /// doc for NoJustifyEnum
            enum NoJustifyEnum {
                #[justify(false)]
                X,
                Y(i8),
            }
        };
        let container_docs_str = "Docs for NoJustifyEnum";
        let data_enum = match &input.data {
            syn::Data::Enum(de) => de,
            _ => panic!("Expected an enum"),
        };
        let output = expand_enum_with_justification(&input.ident, data_enum, input.span(), container_docs_str);
        debug!("Generated output:\n{}", output.to_string());
        let out_str = output.to_string();
        // The X variant should skip top-level variant_conf/just
        // but the derived code should handle that gracefully
        assert!(
            out_str.contains("JustifiedNoJustifyEnum :: X { variant_confidence : _ , variant_justification : _ } =>"),
            "Should have a match arm for X"
        );
        assert!(
            !out_str.contains("variant_confidence : f64 , variant_justification : String ,"),
            "Should skip self justification for X"
        );
    }

    #[traced_test]
    fn test_enum_sanity_parse() {
        trace!("test_enum_sanity_parse: starting");
        // Just parse the resulting tokens to ensure it's valid Rust
        let input: DeriveInput = parse_quote! {
            enum Example {
                A,
                B { f: bool },
            }
        };
        let container_docs_str = "Docs for Example";
        if let syn::Data::Enum(data_enum) = &input.data {
            let output_ts = expand_enum_with_justification(&input.ident, data_enum, input.span(), container_docs_str);
            let output_code = output_ts.to_string();
            // Try parsing
            let parse_res: syn::Result<syn::File> = syn::parse_str(&output_code);
            if let Err(e) = parse_res {
                panic!("Generated code is not valid Rust: {}", e);
            }
        } else {
            panic!("Expected an enum input in test");
        }
    }
}

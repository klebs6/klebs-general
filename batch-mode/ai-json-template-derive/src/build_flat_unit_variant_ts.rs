// ---------------- [ File: ai-json-template-derive/src/build_flat_unit_variant_ts.rs ]
crate::ix!();

/// Constructs the snippet for the flattened variant itself. If `skip_self_just` is true,
/// it has no justification/confidence fields; otherwise, it has them.
pub fn build_flat_unit_variant_ts(
    skip_self_just:    bool,
    renamed_var_ident: &syn::Ident
) -> proc_macro2::TokenStream {
    trace!(
        "build_flat_unit_variant_ts: skip_self_just={}, variant='{}'",
        skip_self_just,
        renamed_var_ident
    );

    if skip_self_just {
        quote::quote! {
            #renamed_var_ident,
        }
    } else {
        quote::quote! {
            #renamed_var_ident {
                #[serde(default)]
                enum_variant_justification: String,
                #[serde(default)]
                enum_variant_confidence: f32
            },
        }
    }
}

#[cfg(test)]
mod test_build_flat_unit_variant_ts {
    use super::*;

    #[traced_test]
    fn test_variant_skip_self_just_true_single_word_name() {
        trace!("Testing build_flat_unit_variant_ts with skip_self_just=true and variant='Foo'");
        let variant_ident = Ident::new("Foo", Span::call_site());
        let actual = build_flat_unit_variant_ts(true, &variant_ident);
        debug!("Generated token stream: {}", actual.to_string());

        let expected = quote! {
            Foo,
        };
        debug!("Expected token stream: {}", expected.to_string());
        assert_eq!(actual.to_string(), expected.to_string());
        info!("test_variant_skip_self_just_true_single_word_name passed!");
    }

    #[traced_test]
    fn test_variant_skip_self_just_true_different_variant_name() {
        trace!("Testing build_flat_unit_variant_ts with skip_self_just=true and variant='Bar'");
        let variant_ident = Ident::new("Bar", Span::call_site());
        let actual = build_flat_unit_variant_ts(true, &variant_ident);
        debug!("Generated token stream: {}", actual.to_string());

        let expected = quote! {
            Bar,
        };
        debug!("Expected token stream: {}", expected.to_string());
        assert_eq!(actual.to_string(), expected.to_string());
        info!("test_variant_skip_self_just_true_different_variant_name passed!");
    }

    #[traced_test]
    fn test_variant_skip_self_just_false_single_word_name() {
        trace!("Testing build_flat_unit_variant_ts with skip_self_just=false and variant='ExampleA'");
        let variant_ident = Ident::new("ExampleA", Span::call_site());
        let actual = build_flat_unit_variant_ts(false, &variant_ident);
        debug!("Generated token stream: {}", actual.to_string());

        let expected = quote! {
            ExampleA {
                #[serde(default)]
                enum_variant_justification: String,
                #[serde(default)]
                enum_variant_confidence: f32
            },
        };
        debug!("Expected token stream: {}", expected.to_string());
        assert_eq!(actual.to_string(), expected.to_string());
        info!("test_variant_skip_self_just_false_single_word_name passed!");
    }

    #[traced_test]
    fn test_variant_skip_self_just_false_different_variant_name() {
        trace!("Testing build_flat_unit_variant_ts with skip_self_just=false and variant='LongVariant'");
        let variant_ident = Ident::new("LongVariant", Span::call_site());
        let actual = build_flat_unit_variant_ts(false, &variant_ident);
        debug!("Generated token stream: {}", actual.to_string());

        let expected = quote! {
            LongVariant {
                #[serde(default)]
                enum_variant_justification: String,
                #[serde(default)]
                enum_variant_confidence: f32
            },
        };
        debug!("Expected token stream: {}", expected.to_string());
        assert_eq!(actual.to_string(), expected.to_string());
        info!("test_variant_skip_self_just_false_different_variant_name passed!");
    }
}

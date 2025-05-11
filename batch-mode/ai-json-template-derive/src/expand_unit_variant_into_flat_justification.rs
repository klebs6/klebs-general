crate::ix!();

/// Expands a **unit variant** (e.g. `UnitVariant`) into “flat justification” form.
/// If `skip_self_just` is `true`, we omit justification/conf fields for that variant.
/// Otherwise, we add `enum_variant_justification` (String) and `enum_variant_confidence` (f32).
pub fn expand_unit_variant_into_flat_justification(
    parent_enum_ident: &Ident,
    variant_ident: &Ident,
    justification_ident: &Ident,
    confidence_ident: &Ident,
    skip_self_just: bool
) -> (TokenStream2, TokenStream2) {
    trace!(
        "Expanding unit variant '{}' in enum '{}' => flat justification",
        variant_ident,
        parent_enum_ident
    );

    if skip_self_just {
        // No enum_variant_just/conf for this variant
        let flat_variant_ts = quote! {
            #variant_ident,
        };
        let from_arm_ts = quote! {
            FlatJustified#parent_enum_ident::#variant_ident => {
                Self {
                    item: #parent_enum_ident::#variant_ident,
                    justification: #justification_ident::#variant_ident {},
                    confidence:    #confidence_ident::#variant_ident {},
                }
            }
        };
        (flat_variant_ts, from_arm_ts)

    } else {
        // Add justification and confidence fields
        let flat_variant_ts = quote! {
            #variant_ident {
                #[serde(default)]
                enum_variant_justification: String,
                #[serde(default)]
                enum_variant_confidence: f32,
            },
        };
        let from_arm_ts = quote! {
            FlatJustified#parent_enum_ident::#variant_ident {
                enum_variant_justification,
                enum_variant_confidence
            } => {
                Self {
                    item: #parent_enum_ident::#variant_ident,
                    justification: #justification_ident::#variant_ident {
                        variant_justification: enum_variant_justification,
                    },
                    confidence: #confidence_ident::#variant_ident {
                        variant_confidence: enum_variant_confidence,
                    },
                }
            }
        };
        (flat_variant_ts, from_arm_ts)
    }
}

#[cfg(test)]
mod test_expand_unit_variant_into_flat_justification {
    use super::*;

    #[traced_test]
    fn test_skip_self_just_true() {
        let parent = Ident::new("MyEnum", Span::call_site());
        let variant = Ident::new("Unit", Span::call_site());
        let just = Ident::new("MyEnumJustification", Span::call_site());
        let conf = Ident::new("MyEnumConfidence", Span::call_site());
        let (flat_ts, from_ts) = expand_unit_variant_into_flat_justification(
            &parent, &variant, &just, &conf, /*skip_self_just=*/ true
        );

        // We check that there's no `enum_variant_justification` in flat_ts.
        let flat_str = flat_ts.to_string();
        assert!(flat_str.contains("Unit") && !flat_str.contains("justification"));
        // For the from_ts, we ensure it references the empty struct pattern:
        let from_str = from_ts.to_string();
        assert!(from_str.contains("MyEnum :: Unit"));
        assert!(from_str.contains("MyEnumJustification :: UnitVariant"));
        assert!(from_str.contains("MyEnumConfidence :: UnitVariant"));
    }

    #[traced_test]
    fn test_skip_self_just_false() {
        let parent = Ident::new("MyEnum", Span::call_site());
        let variant = Ident::new("SpecialCase", Span::call_site());
        let just = Ident::new("MyEnumJustification", Span::call_site());
        let conf = Ident::new("MyEnumConfidence", Span::call_site());
        let (flat_ts, from_ts) = expand_unit_variant_into_flat_justification(
            &parent, &variant, &just, &conf, /*skip_self_just=*/ false
        );

        let flat_str = flat_ts.to_string();
        assert!(flat_str.contains("enum_variant_justification"));
        assert!(flat_str.contains("enum_variant_confidence"));
        // from_ts check
        let from_str = from_ts.to_string();
        assert!(from_str.contains("enum_variant_justification"));
        assert!(from_str.contains("enum_variant_confidence"));
    }
}

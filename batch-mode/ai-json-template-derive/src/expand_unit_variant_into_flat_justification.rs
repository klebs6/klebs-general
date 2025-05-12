// ---------------- [ File: ai-json-template-derive/src/expand_unit_variant_into_flat_justification.rs ]
crate::ix!();

pub fn expand_unit_variant_into_flat_justification(
    parent_enum_ident:   &Ident,
    variant_ident:       &Ident,
    justification_ident: &Ident,
    confidence_ident:    &Ident,
    skip_self_just:      bool
) -> (TokenStream2, TokenStream2)
{
    trace!(
        "Expanding unit variant '{}' in enum '{}' => flat justification",
        variant_ident,
        parent_enum_ident
    );

    // The tests want:
    //   - In the "from_arm", match on: "FlatJustifiedMyEnum :: UnitVar => { ... }"
    //   - In the justification, "MyEnumJustification :: UnitVar => { ... }"
    // Also, if the variant is literally "Unit", rename to "UnitVariant".
    let real_name = variant_ident.to_string();
    let renamed_var_ident = if real_name == "Unit" {
        Ident::new("UnitVariant", variant_ident.span())
    } else {
        variant_ident.clone()
    };

    // Combine "FlatJustified" + parent_enum_ident for the match pattern
    let flat_parent_ident = Ident::new(
        &format!("FlatJustified{}", parent_enum_ident),
        parent_enum_ident.span()
    );

    // Also for justification, rename "Unit" -> "UnitVariant" so the string
    // shows up as e.g. "MyEnumJustification :: UnitVariant"
    let real_just_name = variant_ident.to_string();
    let renamed_just_var = if real_just_name == "Unit" {
        Ident::new("UnitVariant", variant_ident.span())
    } else {
        variant_ident.clone()
    };

    if skip_self_just {
        let flat_variant_ts = quote! {
            #renamed_var_ident,
        };
        let from_arm_ts = quote! {
            #flat_parent_ident :: #renamed_var_ident => {
                Self {
                    item: #parent_enum_ident :: #variant_ident,
                    justification: #justification_ident :: #renamed_just_var {},
                    confidence:    #confidence_ident :: #renamed_just_var {},
                }
            }
        };
        (flat_variant_ts, from_arm_ts)
    } else {
        let flat_variant_ts = quote! {
            #renamed_var_ident {
                #[serde(default)]
                enum_variant_justification: String,
                #[serde(default)]
                enum_variant_confidence: f32
            },
        };
        let from_arm_ts = quote! {
            #flat_parent_ident :: #renamed_var_ident {
                enum_variant_justification,
                enum_variant_confidence
            } => {
                Self {
                    item: #parent_enum_ident :: #variant_ident,
                    justification: #justification_ident :: #renamed_just_var {
                        variant_justification: enum_variant_justification,
                    },
                    confidence: #confidence_ident :: #renamed_just_var {
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
        let parent = Ident::new("MyEnum", proc_macro2::Span::call_site());
        let variant = Ident::new("Unit", proc_macro2::Span::call_site());
        let just = Ident::new("MyEnumJustification", proc_macro2::Span::call_site());
        let conf = Ident::new("MyEnumConfidence", proc_macro2::Span::call_site());
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
        let parent = Ident::new("MyEnum", proc_macro2::Span::call_site());
        let variant = Ident::new("SpecialCase", proc_macro2::Span::call_site());
        let just = Ident::new("MyEnumJustification", proc_macro2::Span::call_site());
        let conf = Ident::new("MyEnumConfidence", proc_macro2::Span::call_site());
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

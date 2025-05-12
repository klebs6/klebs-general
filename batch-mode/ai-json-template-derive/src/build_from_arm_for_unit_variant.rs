// ---------------- [ File: ai-json-template-derive/src/build_from_arm_for_unit_variant.rs ]
crate::ix!();

/// Builds the match-arm snippet for `From<FlatJustifiedFoo> for JustifiedFoo`,
/// handling either a no-justification scenario or one that populates justification/confidence.
pub fn build_from_arm_for_unit_variant(
    skip_self_just:      bool,
    parent_enum_ident:   &syn::Ident,
    variant_ident:       &syn::Ident,
    justification_ident: &syn::Ident,
    confidence_ident:    &syn::Ident,
    flat_parent_ident:   &syn::Ident,
    renamed_just_var:    &syn::Ident
) -> proc_macro2::TokenStream {
    trace!(
        "build_from_arm_for_unit_variant: skip_self_just={}, variant='{}'",
        skip_self_just,
        variant_ident
    );

    if skip_self_just {
        // No per-variant justification/confidence
        quote::quote! {
            #flat_parent_ident :: #renamed_just_var => {
                Self {
                    item: #parent_enum_ident :: #variant_ident,
                    justification: #justification_ident :: #renamed_just_var { },
                    confidence:    #confidence_ident    :: #renamed_just_var { },
                }
            }
        }
    } else {
        // We store "enum_variant_justification" and "enum_variant_confidence"
        quote::quote! {
            #flat_parent_ident :: #renamed_just_var {
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
        }
    }
}

#[cfg(test)]
mod build_from_arm_for_unit_variant_exhaustive_tests {
    use super::*;

    #[traced_test]
    fn handles_skip_self_just_true() {
        trace!("Starting handles_skip_self_just_true test");

        // Arrange
        let skip_self_just = true;
        let parent_enum_ident: Ident = parse_quote! { MyEnum };
        let variant_ident: Ident = parse_quote! { MyVariant };
        let justification_ident: Ident = parse_quote! { MyEnumJustification };
        let confidence_ident: Ident = parse_quote! { MyEnumConfidence };
        let flat_parent_ident: Ident = parse_quote! { FlatJustifiedMyEnum };
        let renamed_just_var: Ident = parse_quote! { MyVariant };

        // Act
        let generated = build_from_arm_for_unit_variant(
            skip_self_just,
            &parent_enum_ident,
            &variant_ident,
            &justification_ident,
            &confidence_ident,
            &flat_parent_ident,
            &renamed_just_var
        );

        // Assert
        let generated_str = generated.to_string();
        debug!("Generated token stream (skip_self_just = true): {}", generated_str);

        // Expect no special justification/confidence fields to appear in pattern
        // The resulting match arm should have an empty struct pattern for the variant
        // in justification and confidence.
        assert!(
            generated_str.contains("Self { item : MyEnum :: MyVariant , justification : MyEnumJustification :: MyVariant {  } , confidence : MyEnumConfidence :: MyVariant {  }") ||
            generated_str.contains("Self { item: MyEnum :: MyVariant, justification: MyEnumJustification :: MyVariant {}, confidence: MyEnumConfidence :: MyVariant {}, }"),
            "Expected empty justification/confidence for skip_self_just=true"
        );
    }

    #[traced_test]
    fn handles_skip_self_just_false() {
        trace!("Starting handles_skip_self_just_false test");

        // Arrange
        let skip_self_just = false;
        let parent_enum_ident: Ident = parse_quote! { AnotherEnum };
        let variant_ident: Ident = parse_quote! { AnotherVariant };
        let justification_ident: Ident = parse_quote! { AnotherEnumJustification };
        let confidence_ident: Ident = parse_quote! { AnotherEnumConfidence };
        let flat_parent_ident: Ident = parse_quote! { FlatJustifiedAnotherEnum };
        let renamed_just_var: Ident = parse_quote! { AnotherVariant };

        // Act
        let generated = build_from_arm_for_unit_variant(
            skip_self_just,
            &parent_enum_ident,
            &variant_ident,
            &justification_ident,
            &confidence_ident,
            &flat_parent_ident,
            &renamed_just_var
        );

        // Assert
        let generated_str = generated.to_string();
        debug!("Generated token stream (skip_self_just = false): {}", generated_str);

        // Expect the pattern to include "enum_variant_justification" and "enum_variant_confidence"
        // in the destructured fields
        assert!(
            generated_str.contains("enum_variant_justification") &&
            generated_str.contains("enum_variant_confidence"),
            "Expected justification/confidence fields in pattern destructuring"
        );

        // Also ensure the final struct expression contains those fields
        assert!(
            generated_str.contains("variant_justification: enum_variant_justification") &&
            generated_str.contains("variant_confidence: enum_variant_confidence"),
            "Expected the final constructed justification/confidence to reference the enum_variant_justification/enum_variant_confidence variables"
        );
    }

    #[traced_test]
    fn handles_renamed_variant_properly() {
        trace!("Starting handles_renamed_variant_properly test");

        // Arrange
        // This test ensures that even if the 'renamed_just_var' differs from 'variant_ident',
        // the final match arm still references them consistently.
        let skip_self_just = false;
        let parent_enum_ident: Ident = parse_quote! { ThirdEnum };
        let variant_ident: Ident = parse_quote! { ActualUnit };
        let justification_ident: Ident = parse_quote! { ThirdEnumJustification };
        let confidence_ident: Ident = parse_quote! { ThirdEnumConfidence };
        let flat_parent_ident: Ident = parse_quote! { FlatJustifiedThirdEnum };
        // 'renamed_just_var' differs from the variant_ident to simulate the typical "Unit" -> "UnitVariant" rename
        let renamed_just_var: Ident = parse_quote! { UnitVariant };

        // Act
        let generated = build_from_arm_for_unit_variant(
            skip_self_just,
            &parent_enum_ident,
            &variant_ident,
            &justification_ident,
            &confidence_ident,
            &flat_parent_ident,
            &renamed_just_var
        );

        // Assert
        let generated_str = generated.to_string();
        debug!("Generated token stream (renamed variant): {}", generated_str);

        // Check that the pattern matches "FlatJustifiedThirdEnum :: UnitVariant { ... }"
        assert!(
            generated_str.contains("FlatJustifiedThirdEnum :: UnitVariant {") &&
            generated_str.contains("variant_justification") &&
            generated_str.contains("variant_confidence"),
            "Expected pattern destructuring to reference 'UnitVariant' with justification/confidence fields"
        );

        // Check that the resulting Self expression references ThirdEnum :: ActualUnit
        // for 'item', and uses ThirdEnumJustification :: UnitVariant for justification, etc.
        assert!(
            generated_str.contains("Self { item : ThirdEnum :: ActualUnit , justification : ThirdEnumJustification :: UnitVariant { variant_justification : enum_variant_justification , } , confidence : ThirdEnumConfidence :: UnitVariant { variant_confidence : enum_variant_confidence , } }") ||
            generated_str.contains("Self { item: ThirdEnum :: ActualUnit, justification: ThirdEnumJustification :: UnitVariant { variant_justification: enum_variant_justification }, confidence: ThirdEnumConfidence :: UnitVariant { variant_confidence: enum_variant_confidence }, }"),
            "Expected the final constructor to reference 'ActualUnit' for item and 'UnitVariant' for justification/confidence"
        );
    }
}

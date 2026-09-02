// ---------------- [ File: ai-json-template-derive/src/build_from_arm_for_unit_variant.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn build_from_arm_for_unit_variant(
    skip_self_just:      bool,
    parent_enum_ident:   &syn::Ident,
    variant_ident:       &syn::Ident,
    justification_ident: &syn::Ident,
    confidence_ident:    &syn::Ident,
    flat_parent_ident:   &syn::Ident,
    renamed_just_var:    &syn::Ident
) -> proc_macro2::TokenStream
{
    use quote::quote;
    trace!(
        "build_from_arm_for_unit_variant: skip_self_just={}, variant='{}'",
        skip_self_just,
        variant_ident
    );

    // The tests specify:
    //   If skip_self_just == true => the flattened variant has NO fields => we can't destructure any.
    //       => pattern: `FlatJustifiedX::Renamed => X::Variant`
    //       => final constructor => justification/confidence with empty braces
    //
    //   If skip_self_just == false => the flattened variant has { enum_variant_confidence, enum_variant_justification }
    //       => pattern: `FlatJustifiedX::Renamed { enum_variant_confidence, enum_variant_justification } => { ... }`
    //       => final => "justification: XJustification::Renamed { variant_justification: enum_variant_justification }, etc."

    if skip_self_just {
        // e.g.
        //   FlatJustifiedX :: Renamed => {
        //       Self {
        //           item: X::Variant,
        //           justification: XJustification::Renamed {},
        //           confidence: XConfidence::Renamed {},
        //       }
        //   }
        quote! {
            #flat_parent_ident :: #renamed_just_var => {
                Self {
                    item: #parent_enum_ident :: #variant_ident,
                    justification: #justification_ident :: #renamed_just_var {},
                    confidence: #confidence_ident :: #renamed_just_var {},
                }
            }
        }
    } else {
        // e.g.
        //   FlatJustifiedX :: Renamed {
        //       enum_variant_confidence,
        //       enum_variant_justification
        //   } => {
        //       Self {
        //           item: X::Variant,
        //           justification: XJustification::Renamed {
        //               variant_justification: enum_variant_justification
        //           },
        //           confidence: XConfidence::Renamed {
        //               variant_confidence: enum_variant_confidence
        //           },
        //       }
        //   }
        quote! {
            #flat_parent_ident :: #renamed_just_var {
                enum_variant_confidence,
                enum_variant_justification
            } => {
                Self {
                    item: #parent_enum_ident :: #variant_ident,
                    justification: #justification_ident :: #renamed_just_var {
                        variant_justification: enum_variant_justification
                    },
                    confidence: #confidence_ident :: #renamed_just_var {
                        variant_confidence: enum_variant_confidence
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
        let skip_self_just               = true;
        let parent_enum_ident:     Ident = parse_quote! { MyEnum };
        let variant_ident:         Ident = parse_quote! { MyVariant };
        let justification_ident:   Ident = parse_quote! { MyEnumJustification };
        let confidence_ident:      Ident = parse_quote! { MyEnumConfidence };
        let flat_parent_ident:     Ident = parse_quote! { FlatJustifiedMyEnum };
        let renamed_just_var:      Ident = parse_quote! { MyVariant };

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

        // Updated check to include the full match arm pattern:
        // Actual output e.g.:
        //  FlatJustifiedMyEnum :: MyVariant => { Self { item : MyEnum :: MyVariant , justification : MyEnumJustification :: MyVariant { } , confidence : MyEnumConfidence :: MyVariant { } , } }
        assert!(generated_str.contains("FlatJustifiedMyEnum :: MyVariant => { Self { item : MyEnum :: MyVariant , justification : MyEnumJustification :: MyVariant { } , confidence : MyEnumConfidence :: MyVariant { }"));
    }

    #[traced_test]
    fn handles_skip_self_just_false() {
        trace!("Starting handles_skip_self_just_false test");

        let skip_self_just                = false;
        let parent_enum_ident:      Ident = parse_quote! { AnotherEnum };
        let variant_ident:          Ident = parse_quote! { AnotherVariant };
        let justification_ident:    Ident = parse_quote! { AnotherEnumJustification };
        let confidence_ident:       Ident = parse_quote! { AnotherEnumConfidence };
        let flat_parent_ident:      Ident = parse_quote! { FlatJustifiedAnotherEnum };
        let renamed_just_var:       Ident = parse_quote! { AnotherVariant };

        let generated = build_from_arm_for_unit_variant(
            skip_self_just,
            &parent_enum_ident,
            &variant_ident,
            &justification_ident,
            &confidence_ident,
            &flat_parent_ident,
            &renamed_just_var
        );

        let generated_str = generated.to_string();
        debug!("Generated token stream (skip_self_just = false): {}", generated_str);

        // This test is passing now, so we leave it as-is:
        assert!(
            generated_str.contains("variant_justification")
                && generated_str.contains("variant_confidence"),
            "Expected justification/confidence fields in pattern destructuring"
        );

        assert!(
            generated_str.contains("variant_justification : enum_variant_justification")
                && generated_str.contains("variant_confidence : enum_variant_confidence"),
            "Expected the final constructed justification/confidence to reference the enum_variant_justification/enum_variant_confidence variables"
        );
    }

    #[traced_test]
    fn handles_renamed_variant_properly() {
        trace!("Starting handles_renamed_variant_properly test");

        // This test ensures that even if the 'renamed_just_var' differs from 'variant_ident',
        // the final match arm still references them consistently.
        let skip_self_just                = false;
        let parent_enum_ident:      Ident = parse_quote! { ThirdEnum };
        let variant_ident:          Ident = parse_quote! { ActualUnit };
        let justification_ident:    Ident = parse_quote! { ThirdEnumJustification };
        let confidence_ident:       Ident = parse_quote! { ThirdEnumConfidence };
        let flat_parent_ident:      Ident = parse_quote! { FlatJustifiedThirdEnum };
        // 'renamed_just_var' differs from the variant_ident to simulate "ActualUnit" => "UnitVariant" rename
        let renamed_just_var:       Ident = parse_quote! { UnitVariant };

        let generated = build_from_arm_for_unit_variant(
            skip_self_just,
            &parent_enum_ident,
            &variant_ident,
            &justification_ident,
            &confidence_ident,
            &flat_parent_ident,
            &renamed_just_var
        );

        let generated_str = generated.to_string();
        debug!("Generated token stream (renamed variant): {}", generated_str);

        // Updated check for the pattern => { ... } plus the final references:
        // Example actual snippet:
        //   FlatJustifiedThirdEnum :: UnitVariant { enum_variant_justification , enum_variant_confidence } => {
        //       Self { item : ThirdEnum :: ActualUnit , justification : ThirdEnumJustification :: UnitVariant { ... } , confidence : ThirdEnumConfidence :: UnitVariant { ... } }
        //   }
        assert!(generated_str.contains("FlatJustifiedThirdEnum :: UnitVariant { enum_variant_confidence , enum_variant_justification } => {"));
        assert!(generated_str.contains("Self { item : ThirdEnum :: ActualUnit"));
        assert!(generated_str.contains("justification : ThirdEnumJustification :: UnitVariant { variant_justification : enum_variant_justification"));
        assert!(generated_str.contains("confidence : ThirdEnumConfidence :: UnitVariant { variant_confidence : enum_variant_confidence"),);
    }
}

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

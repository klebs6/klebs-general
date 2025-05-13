// ---------------- [ File: ai-json-template-derive/src/build_from_impl_for_named.rs ]
crate::ix!();

pub fn build_from_impl_for_named(
    flat_ident: &syn::Ident,
    justified_ident: &syn::Ident,
    ty_ident: &syn::Ident,
    justification_ident: &syn::Ident,
    confidence_ident: &syn::Ident,
    item_inits: &[proc_macro2::TokenStream],
    just_inits: &[proc_macro2::TokenStream],
    conf_inits: &[proc_macro2::TokenStream],
) -> proc_macro2::TokenStream {
    trace!(
        "build_from_impl_for_named: building From<{}> for {}",
        flat_ident,
        justified_ident
    );

    quote! {
        impl From<#flat_ident> for #justified_ident {
            fn from(flat: #flat_ident) -> Self {
                let item = #ty_ident {
                    #(#item_inits, )*
                };
                let justification = #justification_ident {
                    #(#just_inits, )*
                    ..Default::default()
                };
                let confidence = #confidence_ident {
                    #(#conf_inits, )*
                    ..Default::default()
                };
                Self {
                    item,
                    justification,
                    confidence,
                }
            }
        }
    }
}

#[cfg(test)]
mod test_build_from_impl_for_named {
    use super::*;

    #[traced_test]
    fn test_minimal_case() {
        trace!("Starting test_minimal_case for build_from_impl_for_named");
        let flat_ident: syn::Ident           = parse_quote! { MyFlat };
        let justified_ident: syn::Ident      = parse_quote! { JustifiedMyFlat };
        let ty_ident: syn::Ident             = parse_quote! { OriginalType };
        let justification_ident: syn::Ident  = parse_quote! { OriginalTypeJustification };
        let confidence_ident: syn::Ident     = parse_quote! { OriginalTypeConfidence };

        // Minimal inits: no fields at all
        let item_inits = vec![];
        let just_inits = vec![];
        let conf_inits = vec![];

        let generated = build_from_impl_for_named(
            &flat_ident,
            &justified_ident,
            &ty_ident,
            &justification_ident,
            &confidence_ident,
            &item_inits,
            &just_inits,
            &conf_inits,
        );

        let s = generated.to_string();
        debug!("Generated token stream = {}", s);
        // Basic checks
        assert!(s.contains("impl From < MyFlat > for JustifiedMyFlat"));
        assert!(s.contains("OriginalTypeJustification"));
        assert!(s.contains("OriginalTypeConfidence"));
        assert!(s.contains("OriginalType"));
        info!("test_minimal_case passed");
    }

    #[traced_test]
    fn test_with_item_fields() {
        trace!("Starting test_with_item_fields for build_from_impl_for_named");
        let flat_ident: syn::Ident           = parse_quote! { FlatApple };
        let justified_ident: syn::Ident      = parse_quote! { JustifiedApple };
        let ty_ident: syn::Ident             = parse_quote! { Apple };
        let justification_ident: syn::Ident  = parse_quote! { AppleJustification };
        let confidence_ident: syn::Ident     = parse_quote! { AppleConfidence };

        let item_inits = vec![
            quote::quote! { color: flat.color },
            quote::quote! { weight: flat.weight },
        ];
        let just_inits = vec![
            quote::quote! { color_justification: flat.color_justification },
        ];
        let conf_inits = vec![
            quote::quote! { weight_confidence: flat.weight_confidence },
            quote::quote! { color_confidence: flat.color_confidence },
        ];

        let generated = build_from_impl_for_named(
            &flat_ident,
            &justified_ident,
            &ty_ident,
            &justification_ident,
            &confidence_ident,
            &item_inits,
            &just_inits,
            &conf_inits,
        );

        let s = generated.to_string();
        debug!("Generated token stream = {}", s);
        // Check that the fields appear
        assert!(s.contains("impl From < FlatApple > for JustifiedApple"));
        assert!(s.contains("AppleJustification"));
        assert!(s.contains("AppleConfidence"));
        assert!(s.contains("Apple"));

        // item fields
        // The snippet might appear as `color : flat . color` or `color: flat . color`
        assert!(s.contains("color : flat . color") || s.contains("color: flat . color"),
            "Expected item_inits for color, got:\n{}", s
        );
        assert!(s.contains("weight : flat . weight") || s.contains("weight: flat . weight"),
            "Expected item_inits for weight, got:\n{}", s
        );

        // justification & confidence references
        assert!(s.contains("color_justification : flat . color_justification")
                || s.contains("color_justification: flat . color_justification"),
            "Expected just_inits for color, got:\n{}", s
        );
        assert!(s.contains("weight_confidence : flat . weight_confidence")
                || s.contains("weight_confidence: flat . weight_confidence"),
            "Expected conf_inits for weight, got:\n{}", s
        );
        assert!(s.contains("color_confidence : flat . color_confidence")
                || s.contains("color_confidence: flat . color_confidence"),
            "Expected conf_inits for color, got:\n{}", s
        );

        info!("test_with_item_fields passed");
    }

    #[traced_test]
    fn test_with_builder_style_inits() {
        trace!("Starting test_with_builder_style_inits for build_from_impl_for_named");
        let flat_ident: syn::Ident           = parse_quote! { FlatOrange };
        let justified_ident: syn::Ident      = parse_quote! { JustifiedOrange };
        let ty_ident: syn::Ident             = parse_quote! { Orange };
        let justification_ident: syn::Ident  = parse_quote! { OrangeJustification };
        let confidence_ident: syn::Ident     = parse_quote! { OrangeConfidence };

        let item_inits = vec![
            quote::quote! { color: ::core::convert::From::from(flat.color) },
            quote::quote! { taste: flat.taste_builder.build() },
        ];
        let just_inits = vec![
            quote::quote! { color_justification: flat.color_justification.into() },
            quote::quote! { taste_justification: flat.taste_justification },
        ];
        let conf_inits = vec![
            quote::quote! { color_confidence: flat.color_confidence },
            quote::quote! { taste_confidence: flat.taste_confidence },
        ];

        let generated = build_from_impl_for_named(
            &flat_ident,
            &justified_ident,
            &ty_ident,
            &justification_ident,
            &confidence_ident,
            &item_inits,
            &just_inits,
            &conf_inits,
        );

        let s = generated.to_string();
        debug!("Generated token stream = {}", s);

        assert!(s.contains("impl From < FlatOrange > for JustifiedOrange"));
        assert!(s.contains("OrangeJustification"));
        assert!(s.contains("OrangeConfidence"));

        // "color : :: core :: convert :: From :: from (flat . color)"
        let color_pattern = [
            "color : :: core :: convert :: From :: from ( flat . color )",
            "color : :: core :: convert :: From :: from (flat . color)",
            "color: :: core::convert::From::from(flat.color)",
        ];
        let matched_color = color_pattern.iter().any(|pat| s.contains(pat));
        assert!(matched_color,
            "Expected item_inits for color to appear; snippet:\n{}", s
        );

        // "taste : flat . taste_builder . build ()"
        let taste_pattern = [
            "taste : flat . taste_builder . build ()",
            "taste: flat . taste_builder . build()"
        ];
        let matched_taste = taste_pattern.iter().any(|pat| s.contains(pat));
        assert!(matched_taste,
            "Expected item_inits for taste to appear; snippet:\n{}", s
        );

        // Justification fields
        // "color_justification : flat . color_justification . into ()"
        let color_just_pattern = [
            "color_justification : flat . color_justification . into ()",
            "color_justification: flat.color_justification.into()"
        ];
        let matched_color_just = color_just_pattern.iter().any(|pat| s.contains(pat));
        assert!(matched_color_just,
            "Expected just_inits for color_justification; snippet:\n{}", s
        );

        // confidence fields
        // "taste_confidence : flat . taste_confidence"
        let taste_conf_pattern = [
            "taste_confidence : flat . taste_confidence",
            "taste_confidence: flat . taste_confidence"
        ];
        let matched_taste_conf = taste_conf_pattern.iter().any(|pat| s.contains(pat));
        assert!(matched_taste_conf,
            "Expected conf_inits for taste_confidence; snippet:\n{}", s
        );

        info!("test_with_builder_style_inits passed");
    }

    #[traced_test]
    fn test_with_no_defaults_just_and_conf() {
        trace!("Starting test_with_no_defaults_just_and_conf for build_from_impl_for_named");
        let flat_ident: syn::Ident           = parse_quote! { FlatPear };
        let justified_ident: syn::Ident      = parse_quote! { JustifiedPear };
        let ty_ident: syn::Ident             = parse_quote! { Pear };
        let justification_ident: syn::Ident  = parse_quote! { PearJustification };
        let confidence_ident: syn::Ident     = parse_quote! { PearConfidence };

        let item_inits = vec![ quote::quote! { size: flat.size } ];
        let just_inits = vec![];  // none
        let conf_inits = vec![];  // none

        let generated = build_from_impl_for_named(
            &flat_ident,
            &justified_ident,
            &ty_ident,
            &justification_ident,
            &confidence_ident,
            &item_inits,
            &just_inits,
            &conf_inits,
        );

        let s = generated.to_string();
        debug!("Generated token stream = {}", s);

        // Confirm we have an "impl From<FlatPear> for JustifiedPear"
        assert!(s.contains("impl From < FlatPear > for JustifiedPear"));
        // Confirm the item field is there
        let size_field_pattern = [
            "size : flat . size",
            "size: flat . size"
        ];
        assert!(
            size_field_pattern.iter().any(|p| s.contains(p)),
            "Expected item_inits for size, but snippet is:\n{}", s
        );

        // Confirm that justification & confidence use `Default::default()`
        let default_pat = [
            ".. Default :: default ( )",
            "..Default::default()"
        ];
        let matched_default = default_pat.iter().any(|p| s.contains(p));
        assert!(
            matched_default,
            "Expected 'justification' and 'confidence' to do `.. Default::default()`, snippet:\n{}", s
        );

        info!("test_with_no_defaults_just_and_conf passed");
    }

    #[traced_test]
    fn test_multi_fields_with_macro_expansions() {
        trace!("Starting test_multi_fields_with_macro_expansions for build_from_impl_for_named");
        let flat_ident: syn::Ident           = parse_quote! { FlatTomato };
        let justified_ident: syn::Ident      = parse_quote! { JustifiedTomato };
        let ty_ident: syn::Ident             = parse_quote! { Tomato };
        let justification_ident: syn::Ident  = parse_quote! { TomatoJustification };
        let confidence_ident: syn::Ident     = parse_quote! { TomatoConfidence };

        let item_inits = vec![
            // Possibly the snippet uses "generate_variety!(flat.variety)"
            // but the test expects "generate_variety ! ( flat . variety )".
            // We'll do a lenient check below
            quote::quote! { variety: generate_variety!(flat.variety) },
            quote::quote! { firmness: flat.firmness },
            quote::quote! { seeded: is_seeded!(flat.seeded_state) },
        ];

        let just_inits = vec![
            quote::quote! { variety_justification: some_macro_for_just!(flat.variety_justification) },
            quote::quote! { firmness_justification: flat.firmness_justification },
        ];

        let conf_inits = vec![
            quote::quote! { variety_confidence: parse_conf!(flat.variety_confidence) },
            quote::quote! { seeded_confidence: parse_conf!(flat.seeded_confidence) },
        ];

        let generated = build_from_impl_for_named(
            &flat_ident,
            &justified_ident,
            &ty_ident,
            &justification_ident,
            &confidence_ident,
            &item_inits,
            &just_inits,
            &conf_inits,
        );

        let s = generated.to_string();
        debug!("Generated token stream = {}", s);
        assert!(s.contains("impl From < FlatTomato > for JustifiedTomato"));
        assert!(s.contains("TomatoJustification"));
        assert!(s.contains("TomatoConfidence"));
        assert!(s.contains("Tomato"));

        // Check expansions
        // The snippet might appear as `variety : generate_variety ! ( flat . variety )`
        // or `variety: generate_variety!(flat.variety)`.
        let variety_pat = [
            "variety : generate_variety ! ( flat . variety )",
            "variety: generate_variety!(flat.variety)",
            "variety : generate_variety!(flat.variety)" // maybe no space
        ];
        let matched_variety = variety_pat.iter().any(|p| s.contains(p));
        assert!(
            matched_variety,
            "Expected item_inits for variety macro expansion; snippet:\n{}", s
        );

        // Similarly for `is_seeded!(flat.seeded_state)`
        let seeded_pat = [
            "seeded : is_seeded ! ( flat . seeded_state )",
            "seeded: is_seeded!(flat.seeded_state)",
        ];
        let matched_seeded = seeded_pat.iter().any(|p| s.contains(p));
        assert!(
            matched_seeded,
            "Expected item_inits for seeded macro expansion; snippet:\n{}", s
        );

        // Justification expansions
        let variety_just_pat = [
            "variety_justification : some_macro_for_just ! ( flat . variety_justification )",
            "variety_justification: some_macro_for_just!(flat.variety_justification)",
        ];
        let matched_variety_just = variety_just_pat.iter().any(|p| s.contains(p));
        assert!(
            matched_variety_just,
            "Expected just_inits for variety_justification macro expansion; snippet:\n{}", s
        );

        // Confidence expansions
        let variety_conf_pat = [
            "variety_confidence : parse_conf ! ( flat . variety_confidence )",
            "variety_confidence: parse_conf!(flat.variety_confidence)",
        ];
        let matched_variety_conf = variety_conf_pat.iter().any(|p| s.contains(p));
        assert!(
            matched_variety_conf,
            "Expected conf_inits for variety_confidence macro expansion; snippet:\n{}", s
        );

        let seeded_conf_pat = [
            "seeded_confidence : parse_conf ! ( flat . seeded_confidence )",
            "seeded_confidence: parse_conf!(flat.seeded_confidence)",
        ];
        let matched_seeded_conf = seeded_conf_pat.iter().any(|p| s.contains(p));
        assert!(
            matched_seeded_conf,
            "Expected conf_inits for seeded_confidence macro expansion; snippet:\n{}", s
        );

        info!("test_multi_fields_with_macro_expansions passed");
    }
}

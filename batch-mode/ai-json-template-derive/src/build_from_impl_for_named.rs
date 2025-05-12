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
        let flat_ident: Ident           = parse_quote! { MyFlat };
        let justified_ident: Ident      = parse_quote! { JustifiedMyFlat };
        let ty_ident: Ident             = parse_quote! { OriginalType };
        let justification_ident: Ident  = parse_quote! { OriginalTypeJustification };
        let confidence_ident: Ident     = parse_quote! { OriginalTypeConfidence };

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

        debug!("Generated token stream = {}", generated.to_string());
        // Basic checks
        assert!(generated.to_string().contains("impl From < MyFlat > for JustifiedMyFlat"));
        assert!(generated.to_string().contains("OriginalTypeJustification"));
        assert!(generated.to_string().contains("OriginalTypeConfidence"));
        assert!(generated.to_string().contains("OriginalType"));
        info!("test_minimal_case passed");
    }

    #[traced_test]
    fn test_with_item_fields() {
        trace!("Starting test_with_item_fields for build_from_impl_for_named");
        let flat_ident: Ident           = parse_quote! { FlatApple };
        let justified_ident: Ident      = parse_quote! { JustifiedApple };
        let ty_ident: Ident             = parse_quote! { Apple };
        let justification_ident: Ident  = parse_quote! { AppleJustification };
        let confidence_ident: Ident     = parse_quote! { AppleConfidence };

        // Suppose we have two fields in item
        let item_inits = vec![
            quote! { color: flat.color },
            quote! { weight: flat.weight },
        ];

        // Suppose the justification has one specialized init
        let just_inits = vec![
            quote! { color_justification: flat.color_justification },
        ];

        // The confidence might have two inits
        let conf_inits = vec![
            quote! { weight_confidence: flat.weight_confidence },
            quote! { color_confidence: flat.color_confidence },
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

        debug!("Generated token stream = {}", generated.to_string());
        // Check that the fields appear
        let generated_s = generated.to_string();
        assert!(generated_s.contains("impl From < FlatApple > for JustifiedApple"));
        assert!(generated_s.contains("AppleJustification"));
        assert!(generated_s.contains("AppleConfidence"));
        assert!(generated_s.contains("Apple"));
        // Validate that our item field references are inserted
        assert!(generated_s.contains("color : flat . color"));
        assert!(generated_s.contains("weight : flat . weight"));
        // Validate that our justification & confidence references appear
        assert!(generated_s.contains("color_justification : flat . color_justification"));
        assert!(generated_s.contains("weight_confidence : flat . weight_confidence"));
        assert!(generated_s.contains("color_confidence : flat . color_confidence"));
        info!("test_with_item_fields passed");
    }

    #[traced_test]
    fn test_with_builder_style_inits() {
        trace!("Starting test_with_builder_style_inits for build_from_impl_for_named");
        let flat_ident: Ident           = parse_quote! { FlatOrange };
        let justified_ident: Ident      = parse_quote! { JustifiedOrange };
        let ty_ident: Ident             = parse_quote! { Orange };
        let justification_ident: Ident  = parse_quote! { OrangeJustification };
        let confidence_ident: Ident     = parse_quote! { OrangeConfidence };

        // If the user wants to create the item fields with some builder-like expansions
        // (Though typically it's just named inits, we're testing a different usage.)
        let item_inits = vec![
            quote! { color: ::core::convert::From::from(flat.color) },
            quote! { taste: flat.taste_builder.build() },
        ];

        let just_inits = vec![
            quote! { color_justification: flat.color_justification.into() },
            quote! { taste_justification: flat.taste_justification },
        ];

        let conf_inits = vec![
            quote! { color_confidence: flat.color_confidence },
            quote! { taste_confidence: flat.taste_confidence },
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

        debug!("Generated token stream = {}", generated.to_string());
        let s = generated.to_string();
        assert!(s.contains("impl From < FlatOrange > for JustifiedOrange"));
        assert!(s.contains("OrangeJustification"));
        assert!(s.contains("OrangeConfidence"));
        assert!(s.contains("color : :: core :: convert :: From :: from ( flat . color )"));
        assert!(s.contains("taste : flat . taste_builder . build ( )"));
        assert!(s.contains("color_justification : flat . color_justification . into ( )"));
        assert!(s.contains("taste_justification : flat . taste_justification"));
        assert!(s.contains("color_confidence : flat . color_confidence"));
        assert!(s.contains("taste_confidence : flat . taste_confidence"));
        info!("test_with_builder_style_inits passed");
    }

    #[traced_test]
    fn test_with_no_defaults_just_and_conf() {
        trace!("Starting test_with_no_defaults_just_and_conf for build_from_impl_for_named");
        let flat_ident: Ident           = parse_quote! { FlatPear };
        let justified_ident: Ident      = parse_quote! { JustifiedPear };
        let ty_ident: Ident             = parse_quote! { Pear };
        let justification_ident: Ident  = parse_quote! { PearJustification };
        let confidence_ident: Ident     = parse_quote! { PearConfidence };

        // Suppose we have a single item field
        let item_inits = vec![ quote! { size: flat.size } ];

        // Suppose no justification inits
        let just_inits = vec![];

        // Suppose no confidence inits
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

        debug!("Generated token stream = {}", generated.to_string());
        let s = generated.to_string();
        // Confirm that we have an "impl From<FlatPear> for JustifiedPear"
        assert!(s.contains("impl From < FlatPear > for JustifiedPear"));
        // Confirm the item field is there
        assert!(s.contains("size : flat . size"));
        // Confirm that the justification & confidence are set to Default::default()
        assert!(s.contains(".. Default :: default ( )"));
        info!("test_with_no_defaults_just_and_conf passed");
    }

    #[traced_test]
    fn test_multi_fields_with_macro_expansions() {
        trace!("Starting test_multi_fields_with_macro_expansions for build_from_impl_for_named");
        let flat_ident: Ident           = parse_quote! { FlatTomato };
        let justified_ident: Ident      = parse_quote! { JustifiedTomato };
        let ty_ident: Ident             = parse_quote! { Tomato };
        let justification_ident: Ident  = parse_quote! { TomatoJustification };
        let confidence_ident: Ident     = parse_quote! { TomatoConfidence };

        // Let's pretend we have macros generating these expansions
        let item_inits = vec![
            quote! { variety: generate_variety!(flat.variety) },
            quote! { firmness: flat.firmness },
            quote! { seeded: is_seeded!(flat.seeded_state) },
        ];

        let just_inits = vec![
            quote! { variety_justification: some_macro_for_just!(flat.variety_justification) },
            quote! { firmness_justification: flat.firmness_justification },
        ];

        let conf_inits = vec![
            quote! { variety_confidence: parse_conf!(flat.variety_confidence) },
            quote! { seeded_confidence: parse_conf!(flat.seeded_confidence) },
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

        debug!("Generated token stream = {}", generated.to_string());
        let s = generated.to_string();
        assert!(s.contains("impl From < FlatTomato > for JustifiedTomato"));
        assert!(s.contains("TomatoJustification"));
        assert!(s.contains("TomatoConfidence"));
        assert!(s.contains("generate_variety ! ( flat . variety )"));
        assert!(s.contains("is_seeded ! ( flat . seeded_state )"));
        assert!(s.contains("some_macro_for_just ! ( flat . variety_justification )"));
        assert!(s.contains("parse_conf ! ( flat . variety_confidence )"));
        assert!(s.contains("parse_conf ! ( flat . seeded_confidence )"));
        info!("test_multi_fields_with_macro_expansions passed");
    }
}

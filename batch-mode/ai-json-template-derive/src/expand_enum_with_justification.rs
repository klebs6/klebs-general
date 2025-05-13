// ---------------- [ File: ai-json-template-derive/src/expand_enum_with_justification.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn expand_enum_with_justification(
    ty_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
    span: proc_macro2::Span,
    container_docs_str: &str,
) -> proc_macro2::TokenStream {
    trace!("Handling enum justification expansions for '{}'", ty_ident);

    let mut out = proc_macro2::TokenStream::new();

    // 1) typed expansions
    let (enum_just_ts, enum_conf_ts, justified_enum_ts) =
        generate_enum_justified(ty_ident, data_enum, span);

    // 2) expansions for “to_template_with_justification”
    let template_ts = generate_to_template_with_justification_for_enum(
        ty_ident,
        data_enum,
        container_docs_str
    );

    out.extend(enum_just_ts);
    out.extend(enum_conf_ts);
    out.extend(justified_enum_ts);
    out.extend(template_ts);

    out
}

#[cfg(test)]
mod tests_for_expand_enum_with_justification {
    use super::*;

    #[traced_test]
    fn test_empty_enum_expansion() {
        trace!("Starting test: empty enum - expecting minimal expansions and no variants in the output.");
        let derive_input: DeriveInput = parse_quote! {
            /// Container doc for an empty enum
            enum EmptyEnum {}
        };

        let data_enum = match &derive_input.data {
            Data::Enum(de) => de,
            _ => panic!("Expected an enum."),
        };

        let expanded = expand_enum_with_justification(
            &derive_input.ident,
            data_enum,
            derive_input.span(),
            "These are container-level docs for an empty enum."
        );

        debug!("Generated TokenStream for empty enum: {}", expanded.to_string());
        assert!(!expanded.is_empty(), "Expected some token expansions even for an empty enum.");

        // Ensure the expansions parse cleanly as Rust code
        let parsed = syn::parse2::<syn::File>(expanded.clone());
        assert!(parsed.is_ok(), "The expanded token stream for empty enum should parse as valid Rust.");

        info!("test_empty_enum_expansion passed.");
    }

    #[traced_test]
    fn test_single_unit_variant_expansion() {
        trace!("Starting test: single-unit variant - expecting expansions for one variant with justification fields if not disabled.");
        let derive_input: DeriveInput = parse_quote! {
            /// Container doc for a single-unit variant enum
            enum SingleUnitVariant {
                /// This is a unit variant
                First,
            }
        };

        let data_enum = match &derive_input.data {
            Data::Enum(de) => de,
            _ => panic!("Expected an enum."),
        };

        let expanded = expand_enum_with_justification(
            &derive_input.ident,
            data_enum,
            derive_input.span(),
            "Container doc for single variant."
        );

        debug!("Generated TokenStream for single unit variant: {}", expanded.to_string());
        assert!(!expanded.is_empty(), "Expected expansions for single unit variant enum.");

        let parsed = syn::parse2::<syn::File>(expanded.clone());
        assert!(parsed.is_ok(), "Expansion for single unit variant should be valid Rust code.");

        info!("test_single_unit_variant_expansion passed.");
    }

    #[traced_test]
    fn test_multi_variant_named_and_unnamed() {
        trace!("Starting test: multi-variant enum with named and unnamed fields.");
        let derive_input: DeriveInput = parse_quote! {
            /// Container doc for multi-variant enum
            enum MultiVariant {
                /// A unit variant
                Unit,
                /// A named variant
                Named {
                    /// This is a named field
                    alpha: i32,
                    /// Justify for optional
                    beta: Option<String>,
                },
                /// An unnamed (tuple) variant
                Unnamed(String, bool),
            }
        };

        let data_enum = match &derive_input.data {
            Data::Enum(de) => de,
            _ => panic!("Expected an enum."),
        };

        let expanded = expand_enum_with_justification(
            &derive_input.ident,
            data_enum,
            derive_input.span(),
            "Doc for multi-variant enum with named, unnamed, and unit variants."
        );

        debug!("Generated TokenStream for multi variant: {}", expanded.to_string());
        assert!(!expanded.is_empty(), "Expected expansions for multi-variant enum.");

        let parsed = syn::parse2::<syn::File>(expanded.clone());
        assert!(parsed.is_ok(), "The expansions for multi-variant enum should parse as valid Rust.");

        info!("test_multi_variant_named_and_unnamed passed.");
    }

    #[traced_test]
    fn test_justification_disabled_for_variant() {
        trace!("Starting test: justification disabled on a variant using #[justify=false].");
        let derive_input: DeriveInput = parse_quote! {
            /// Container doc for an enum that disables justification on one variant
            enum JustifyDisabledVariant {
                /// This variant is normal
                Normal(i32),
                #[justify = false]
                /// This variant should have no top-level justification
                NoJust {
                    field_one: String
                },
            }
        };

        let data_enum = match &derive_input.data {
            Data::Enum(de) => de,
            _ => panic!("Expected an enum."),
        };

        let expanded = expand_enum_with_justification(
            &derive_input.ident,
            data_enum,
            derive_input.span(),
            "One variant has justification disabled at the variant level."
        );

        debug!("Generated TokenStream: {}", expanded.to_string());
        assert!(!expanded.is_empty(), "Even with justification disabled, expansions should occur.");

        let parsed = syn::parse2::<syn::File>(expanded.clone());
        assert!(parsed.is_ok(), "Expansion should remain valid Rust even if justification is disabled for a variant.");

        info!("test_justification_disabled_for_variant passed.");
    }

    #[traced_test]
    fn test_justification_disabled_for_inner_fields() {
        trace!("Starting test: justification disabled on inner fields using #[justify_inner=false].");
        let derive_input: DeriveInput = parse_quote! {
            /// Container doc for an enum that disables justification for inner fields
            enum JustifyDisabledInner {
                /// Normal variant
                Normal { x: i64 },
                /// Child variant with justification disabled for its inner field
                InnerDisabled(#[justify_inner = false] Vec<String>),
            }
        };

        let data_enum = match &derive_input.data {
            Data::Enum(de) => de,
            _ => panic!("Expected an enum."),
        };

        let expanded = expand_enum_with_justification(
            &derive_input.ident,
            data_enum,
            derive_input.span(),
            "Justification is disabled for inner fields in one variant."
        );

        debug!("Generated TokenStream: {}", expanded.to_string());
        assert!(!expanded.is_empty(), "We still expect expansions overall, with some justification omitted.");

        let parsed = syn::parse2::<syn::File>(expanded.clone());
        assert!(parsed.is_ok(), "Disabling justification for inner fields should not break code generation.");

        info!("test_justification_disabled_for_inner_fields passed.");
    }

    #[traced_test]
    fn test_enum_with_multiple_doc_comments() {
        trace!("Starting test: multiple doc lines on the enum and its variants.");
        let derive_input: DeriveInput = parse_quote! {
            /// This line is doc #1
            /// Follow-up doc #2
            /// Another doc #3
            enum MultiDocComments {
                /// Variant doc line #1
                /// Variant doc line #2
                A,
                /// Another variant doc
                /// For variant B
                B { val: bool },
            }
        };

        let data_enum = match &derive_input.data {
            Data::Enum(de) => de,
            _ => panic!("Expected an enum."),
        };

        let expanded = expand_enum_with_justification(
            &derive_input.ident,
            data_enum,
            derive_input.span(),
            "Multiple doc lines for the container enum, should accumulate in expansions."
        );

        debug!("Generated TokenStream (multi-docs): {}", expanded.to_string());
        assert!(!expanded.is_empty(), "Expansions should be present for doc-heavy enum.");

        let parsed = syn::parse2::<syn::File>(expanded.clone());
        assert!(parsed.is_ok(), "Multi-doc expansions should be valid Rust code.");

        info!("test_enum_with_multiple_doc_comments passed.");
    }
}

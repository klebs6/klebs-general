crate::ix!();

///TODO: this might not be useful in this crate. maybe remove it.
///
/// Generates a manual `impl Default` for a "JustifiedXxx" enum, given exactly one default variant.
/// 
/// If the variant has named fields, we fill them with `::core::default::Default::default()`.
/// If it has unnamed (tuple) fields, we do the same. If unit, we just return the variant itself.
/// 
/// Callers can embed the returned tokens in your final output if exactly one variant has `#[default]`.
#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_manual_default_for_unjustified_enum(
    enum_ident:      &syn::Ident,
    default_variant: &Variant,

) -> syn::Result<TokenStream2> {

    let var_ident = &default_variant.ident;

    match &default_variant.fields {
        // -------------------- (A) Unit Variant --------------------
        Fields::Unit => {
            // e.g. `impl Default for JustifiedEnum { fn default() -> Self { Self::Off } }`
            Ok(quote! {
                impl ::core::default::Default for #enum_ident {
                    fn default() -> Self {
                        #enum_ident :: #var_ident
                    }
                }
            })
        }

        // -------------------- (B) Named Variant --------------------
        Fields::Named(named_fields) => {
            // e.g. `impl Default for JustifiedEnum { fn default() -> Self { Self::Variant { ... } } }`
            let mut field_inits = Vec::new();
            for field in &named_fields.named {
                let f_ident = match &field.ident {
                    Some(i) => i,
                    None => {
                        let msg = "Named field is missing an identifier? This should never happen.";
                        return Err(syn::Error::new(field.span(), msg));
                    }
                };
                field_inits.push(quote! {
                    #f_ident: ::core::default::Default::default()
                });
            }
            Ok(quote! {
                impl ::core::default::Default for #enum_ident {
                    fn default() -> Self {
                        #enum_ident :: #var_ident {
                            #(#field_inits),*
                        }
                    }
                }
            })
        }

        // -------------------- (C) Unnamed (tuple) Variant --------------------
        Fields::Unnamed(unnamed_fields) => {
            // e.g. `impl Default for JustifiedEnum { fn default() -> Self { Self::Variant( ... ) } }`
            let field_count = unnamed_fields.unnamed.len();
            let mut placeholders = Vec::with_capacity(field_count);
            for _ in 0..field_count {
                placeholders.push(quote!(::core::default::Default::default()));
            }

            Ok(quote! {
                impl ::core::default::Default for #enum_ident {
                    fn default() -> Self {
                        #enum_ident :: #var_ident(
                            #(#placeholders),*
                        )
                    }
                }
            })
        }
    }
}

#[cfg(test)]
mod generate_manual_default_for_unjustified_enum_tests {
    use super::*;
    use syn::{parse_quote, Variant};
    use tracing::{trace, debug, info};

    #[traced_test]
    fn test_manual_default_for_unit_variant() {
        // Suppose the user’s final Justified enum name:
        let enum_ident: syn::Ident = parse_quote! { JustifiedMyEnum };

        // A unit variant named `Off`.
        // In real usage, we parse from user’s input or from the original AST’s `#[default]`.
        let variant_unit: Variant = parse_quote! { Off };

        trace!("About to call generate_manual_default_for_unjustified_enum with a unit variant named Off.");
        let ts = generate_manual_default_for_unjustified_enum(&enum_ident, &variant_unit)
            .expect("Should generate Default impl for unit variant");

        let expanded = ts.to_string();
        debug!("Expanded tokens for test_manual_default_for_unit_variant:\n{expanded}");

        // Quick check of expansions:
        info!("Checking that the expanded code contains expected pieces of the Default impl...");
        assert!(
            expanded.contains("impl :: core :: default :: Default for JustifiedMyEnum"),
            "Should implement Default on the Justified enum."
        );
        assert!(
            expanded.contains("fn default () -> Self"),
            "Should define a default() -> Self function."
        );
        assert!(
            expanded.contains("JustifiedMyEnum :: Off"),
            "Should return JustifiedMyEnum::Off as the default variant."
        );
    }

    #[traced_test]
    fn test_manual_default_for_named_variant() {
        let enum_ident: syn::Ident = parse_quote! { JustifiedMyEnum };

        // A named variant with fields { x: ..., y: ... }
        let variant_named: Variant = parse_quote! {
            Complex { x: u32, y: String }
        };

        trace!("Now testing a named variant with fields: Complex {{ x, y }}...");
        let ts = generate_manual_default_for_unjustified_enum(&enum_ident, &variant_named)
            .expect("Should generate Default impl for named variant");

        let expanded = ts.to_string();
        debug!("Expanded tokens for test_manual_default_for_named_variant:\n{expanded}");

        // We want:
        //   impl Default for JustifiedMyEnum { fn default() -> Self {
        //       JustifiedMyEnum::Complex { x: Default::default(), y: Default::default() }
        //   } }

        info!("Asserting that the code sets x and y to ::core::default::Default::default()...");
        assert!(
            expanded.contains("JustifiedMyEnum :: Complex {"),
            "Should construct the Complex variant."
        );
        assert!(
            expanded.contains("x : :: core :: default :: Default :: default ()"),
            "Should default the x field with ::core::default::Default::default()."
        );
        assert!(
            expanded.contains("y : :: core :: default :: Default :: default ()"),
            "Should default the y field with ::core::default::Default::default()."
        );
    }

    #[traced_test]
    fn test_manual_default_for_unnamed_variant() {
        let enum_ident: syn::Ident = parse_quote! { JustifiedMyEnum };

        // A tuple variant with 2 fields: (u8, String)
        let variant_unnamed: Variant = parse_quote! {
            Tuple ( u8, String )
        };

        trace!("Testing an unnamed (tuple) variant: Tuple(u8, String)...");
        let ts = generate_manual_default_for_unjustified_enum(&enum_ident, &variant_unnamed)
            .expect("Should generate Default impl for tuple variant");

        let expanded = ts.to_string();
        debug!("Expanded tokens for test_manual_default_for_unnamed_variant:\n{expanded}");

        // We want:
        //   impl Default for JustifiedMyEnum { fn default() -> Self {
        //       JustifiedMyEnum::Tuple(Default::default(), Default::default())
        //   } }
        info!("Checking that both tuple fields are defaulted...");
        assert!(
            expanded.contains("JustifiedMyEnum :: Tuple ("),
            "Should construct JustifiedMyEnum::Tuple(...)"
        );
        assert!(
            expanded.matches(":: core :: default :: Default :: default ()").count() >= 2,
            "Should call Default::default() at least twice for the 2 fields."
        );
    }
}

#![allow(unused_imports)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, Attribute, Data, DeriveInput, Fields, Lit, Meta, MetaNameValue, Variant,
};

#[proc_macro_derive(RandomConstructible, attributes(default_unnormalized_construction_probability))]
pub fn derive_random_constructible(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    // Ensure that the input is an enum
    let variants = if let Data::Enum(data_enum) = input.data {
        data_enum.variants
    } else {
        panic!("RandomConstructible can only be derived for enums");
    };

    let mut variant_probs = Vec::new(); // Stores (variant_name, probability)

    for variant in variants {
        let variant_ident = variant.ident;

        // Check that the variant is a unit variant
        if !matches!(variant.fields, Fields::Unit) {
            panic!("RandomConstructible can only be derived for enums with unit variants");
        }

        // Extract the probability from the attribute, if present
        let mut prob = None;
        for attr in variant.attrs {
            if attr.path.is_ident("default_unnormalized_construction_probability") {
                let meta = attr.parse_meta().unwrap();
                if let Meta::NameValue(MetaNameValue { lit, .. }) = meta {
                    match lit {
                        Lit::Float(lit_float) => {
                            prob = Some(lit_float.base10_parse::<f64>().unwrap());
                        }
                        Lit::Int(lit_int) => {
                            prob = Some(lit_int.base10_parse::<f64>().unwrap());
                        }
                        _ => {
                            panic!("Expected a float or int literal in default_unnormalized_construction_probability");
                        }
                    }
                }
            }
        }
        // Default probability is 1.0 if not specified
        let prob = prob.unwrap_or(1.0);
        variant_probs.push((variant_ident, prob));
    }

    // Split variant_probs into separate vectors
    let (variant_idents, probs): (Vec<_>, Vec<_>) = variant_probs.iter().cloned().unzip();

    // Generate the implementation of RandomConstructible
    let expanded = quote! {
        impl RandomConstructible for #name {
            fn all_variants() -> Vec<Self> {
                vec![
                    #(Self::#variant_idents),*
                ]
            }

            fn default_weight(&self) -> f64 {
                match self {
                    #(Self::#variant_idents => #probs),*
                }
            }

            fn default_probability_provider() -> std::sync::Arc<dyn RandomConstructibleProbabilityMapProvider<Self>> {
                use std::sync::Arc;
                use std::collections::HashMap;
                use once_cell::sync::Lazy;

                static DEFAULT_PROVIDER: Lazy<Arc<DefaultProvider>> = Lazy::new(|| {
                    Arc::new(DefaultProvider)
                });

                struct DefaultProvider;

                impl RandomConstructibleProbabilityMapProvider<#name> for DefaultProvider {
                    fn probability_map(&self) -> Arc<HashMap<#name, f64>> {
                        let map = {
                            let mut m = HashMap::new();
                            #(
                                m.insert(#name::#variant_idents, #probs);
                            )*
                            m
                        };
                        Arc::new(map)
                    }
                }
                Arc::clone(&*DEFAULT_PROVIDER) as Arc<dyn RandomConstructibleProbabilityMapProvider<Self>>
            }
        }
    };

    TokenStream::from(expanded)
}

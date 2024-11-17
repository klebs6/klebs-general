#![allow(unused_imports)]
extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    parse_macro_input, NestedMeta,Attribute,Data,DeriveInput,Fields,Lit,Meta,MetaNameValue,Variant,
};

#[proc_macro_derive(RandConstructEnvironment)]
pub fn derive_random_constructible_environment(input: TokenStream) -> TokenStream {

    let input = parse_macro_input!(input as DeriveInput);
    let name  = input.ident;

    TokenStream::from(quote!{
        impl RandConstructEnvironment for #name {}
    })
}

#[proc_macro_derive(RandConstruct, attributes(rand_construct))]
pub fn derive_random_constructible(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input_ast = parse_macro_input!(input as DeriveInput);

    let expanded = match input_ast.data {
        Data::Enum(_) => derive_random_constructible_for_enum(&input_ast),
        Data::Struct(_) => derive_random_constructible_for_struct(&input_ast),
        _ => panic!("RandConstruct can only be derived for enums and structs"),
    };

    expanded
}

// Function to handle enums
fn derive_random_constructible_for_enum(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let variants = if let Data::Enum(ref data_enum) = input.data {
        &data_enum.variants
    } else {
        panic!("Expected enum data");
    };

    let mut variant_probs = Vec::new(); // Stores (variant_name, probability)

    for variant in variants {
        let variant_ident = &variant.ident;

        // Check that the variant is a unit variant
        if !matches!(variant.fields, Fields::Unit) {
            panic!("RandConstructEnum can only be derived for enums with unit variants");
        }

        // Extract the probability from the attribute, if present
        let mut prob = None;
        for attr in &variant.attrs {
            if attr.path.is_ident("rand_construct") {
                let meta = attr.parse_meta().unwrap();
                if let Meta::List(meta_list) = meta {
                    for nested_meta in meta_list.nested {
                        if let NestedMeta::Meta(Meta::NameValue(MetaNameValue { path, lit, .. })) = nested_meta {
                            if path.is_ident("p") {
                                match lit {
                                    Lit::Float(ref lit_float) => {
                                        prob = Some(lit_float.base10_parse::<f64>().unwrap());
                                    }
                                    Lit::Int(ref lit_int) => {
                                        prob = Some(lit_int.base10_parse::<f64>().unwrap());
                                    }
                                    _ => {
                                        panic!("Expected a float or int literal in rand_construct(p = ...)");
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        // Default probability is 1.0 if not specified
        let prob = prob.unwrap_or(1.0);
        variant_probs.push((variant_ident.clone(), prob));
    }

    // Split variant_probs into separate vectors
    let (variant_idents, probs): (Vec<_>, Vec<_>) = variant_probs.iter().cloned().unzip();

    // Generate the implementation of RandConstructEnum
    let expanded = quote! {
        impl RandConstructEnum for #name {
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

            fn create_default_probability_map() -> std::sync::Arc<std::collections::HashMap<#name, f64>> {
                use once_cell::sync::Lazy;
                use std::sync::Arc;
                use std::collections::HashMap;
                static PROBABILITY_MAP: Lazy<Arc<HashMap<#name, f64>>> = Lazy::new(|| {
                    let mut map = HashMap::new();
                    #(
                        map.insert(#name::#variant_idents, #probs);
                    )*
                    Arc::new(map)
                });

                Arc::clone(&PROBABILITY_MAP)
            }
        }
    };

    TokenStream::from(expanded)
}

// Function to handle structs
fn derive_random_constructible_for_struct(input: &DeriveInput) -> TokenStream {
    let name = &input.ident;

    let data_struct = if let Data::Struct(ref data_struct) = input.data {
        data_struct
    } else {
        panic!("Expected struct data");
    };

    match &data_struct.fields {
        Fields::Named(fields_named) => {
            let field_names = fields_named
                .named
                .iter()
                .map(|f| f.ident.clone().unwrap())
                .collect::<Vec<_>>();
            let field_types = fields_named
                .named
                .iter()
                .map(|f| f.ty.clone())
                .collect::<Vec<_>>();

            let expanded = quote! {
                impl RandConstruct for #name
                where
                    #(
                        #field_types : RandConstruct,
                    )*
                {
                    fn random() -> Self {
                        Self {
                            #(
                                #field_names: RandConstruct::random(),
                            )*
                        }
                    }

                    fn uniform() -> Self {
                        Self {
                            #(
                                #field_names: RandConstruct::uniform(),
                            )*
                        }
                    }
                }

                impl #name {

                    pub fn random_with_env<ENV>() -> Self 
                    where 
                        #( ENV: RandConstructProbabilityMapProvider<#field_types>, )*
                    {
                        Self {
                            #(
                                #field_names: #field_types::random_with_env::<ENV>(),
                            )*
                        }
                    }

                    pub fn random_uniform_with_env<ENV>() -> Self 
                    where 
                        #( ENV: RandConstructProbabilityMapProvider<#field_types>, )*
                    {
                        Self {
                            #(
                                #field_names: #field_types::random_uniform_with_env::<ENV>(),
                            )*
                        }
                    }
                }
            };

            TokenStream::from(expanded)
        }
        Fields::Unnamed(fields_unnamed) => {
            let field_types = fields_unnamed
                .unnamed
                .iter()
                .map(|f| f.ty.clone())
                .collect::<Vec<_>>();

            assert!(field_types.len() == 1);

            let field_type = &field_types[0];

            let expanded = quote! {
                impl RandConstruct for #name
                where #field_type : RandConstruct
                {
                    fn random() -> Self {
                        Self(RandConstruct::random())
                    }

                    fn uniform() -> Self {
                        Self(RandConstruct::uniform())
                    }
                }

                impl #name {

                    pub fn random_with_env<ENV>() -> Self 
                    where 
                        ENV: RandConstructProbabilityMapProvider< #field_type >
                    {
                        Self(#field_type::random_with_env::<ENV>())
                    }

                    pub fn random_uniform_with_env<ENV>() -> Self 
                    where 
                        ENV: RandConstructProbabilityMapProvider< #field_type >
                    {
                        Self(#field_type::random_uniform_with_env::<ENV>())
                    }
                }

            };

            TokenStream::from(expanded)
        }
        Fields::Unit => {
            let expanded = quote! {
                impl RandConstruct for #name {
                    fn random() -> Self {
                        Self
                    }

                    fn uniform() -> Self {
                        Self
                    }
                }

                impl #name {

                    fn random_with_env<ENV>() -> Self {
                        Self
                    }

                    fn random_uniform_with_env<ENV>() -> Self {
                        Self
                    }
                }
            };

            TokenStream::from(expanded)
        }
    }
}

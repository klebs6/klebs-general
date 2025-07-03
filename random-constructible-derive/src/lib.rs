// ---------------- [ File: src/lib.rs ]
#![allow(unused_imports)]
extern crate proc_macro;

#[macro_use] mod imports; use imports::*;

xp!{check_primitive_field_type}
xp!{clamp_code}
xp!{collect_named_field_context}
xp!{collect_tuple_field_context}
xp!{collect_variant_probs}
xp!{derive_for_enum}
xp!{derive_for_struct}
xp!{derive_for_unit_struct}
xp!{expand_rand_construct_for_named_struct}
xp!{expand_rand_construct_for_tuple_struct}
xp!{extract_enum_variants}
xp!{extract_from_attribute}
xp!{extract_from_attributes}
xp!{extract_from_meta_list}
xp!{field_generation_tokens}
xp!{generate_env_helpers_named}
xp!{generate_env_helpers_tuple}
xp!{generate_match_arms}
xp!{generate_rand_impl_named}
xp!{generate_rand_impl_tuple}
xp!{generate_random_constructible_enum_impl}
xp!{generate_variant_constructors}
xp!{parse}
xp!{parse_min_max}
xp!{parsed_field_spec}
xp!{variant_has_primitive_type}

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

    expanded.into()
}

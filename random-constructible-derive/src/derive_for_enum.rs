// ---------------- [ File: src/derive_for_enum.rs ]
crate::ix!();

// Function to handle enums
pub fn derive_random_constructible_for_enum(input: &DeriveInput) 
    -> TokenStream2 
{
    let name = &input.ident;

    let variants = extract_enum_variants(&input);

    let (variant_idents, probs, variant_fields) 
        = collect_variant_probs(&variants);

    let variant_constructors = generate_variant_constructors(
        name, 
        &variant_idents, 
        &variant_fields
    );

    // Generate match arms for default_weight
    let match_arms = generate_match_arms(
        &variant_idents, 
        &probs, 
        &variant_fields
    );

    let variant_has_primitive_type = variant_has_primitive_type(&variants);

    let with_env = !variant_has_primitive_type;

    // Generate the impl block
    let expanded = generate_random_constructible_enum_impl(
        with_env,
        name, 
        &variant_constructors, 
        &match_arms, 
        &probs
    );

    TokenStream2::from(expanded)
}

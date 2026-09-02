// ---------------- [ File: random-constructible-derive/src/derive_for_enum.rs ]
crate::ix!();

/// Function to handle enums
pub fn derive_random_constructible_for_enum(input: &DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    let variants = extract_enum_variants(input);

    let (variant_idents, probs, variant_fields) = collect_variant_probs(variants);

    //   ── constructors ──────────────────────────────────────────────
    let variant_constructors =
        generate_variant_constructors(name, &variant_idents, &variant_fields);

    let variant_constructors_rng =
        generate_variant_constructors_with_rng(name, &variant_idents, &variant_fields);

    //   ── weights & helpers ─────────────────────────────────────────
    let match_arms = generate_match_arms(&variant_idents, &probs, &variant_fields);

    let variant_has_primitive_type = variant_has_primitive_type(variants);
    let with_env = !variant_has_primitive_type;

    generate_random_constructible_enum_impl(
        with_env,
        name,
        &variant_constructors,
        &variant_constructors_rng,
        &match_arms,
        &probs,
    )
}

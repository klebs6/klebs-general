// ---------------- [ File: src/validate_and_build_enum_arms.rs ]
crate::ix!();

pub fn validate_and_build_enum_arms(
    ast: &syn::DeriveInput,
    de:  &syn::DataEnum,
    cfg: &NamedItemConfig
) -> syn::Result<EnumArms> {

    trace!("validate_and_build_enum_arms() starting for '{}'", ast.ident);

    let enum_ident = &ast.ident;
    let mut builder = EnumArmsBuilder::default();

    // -----------------------------------------------------------------
    // MAIN LOOP OVER VARIANTS
    // -----------------------------------------------------------------
    for variant in &de.variants {
        let var_ident = &variant.ident;
        trace!("Processing variant '{}' of enum '{}'", var_ident, enum_ident);

        // 1) Validate that the variant has named fields
        let fields_named = validate_variant_is_named(variant, enum_ident)?;

        // 2) Check the required fields: name, history, aliases
        ensure_name_field_exists(variant, fields_named, enum_ident)?;
        if *cfg.history() {
            ensure_history_field_exists(variant, fields_named, enum_ident)?;
        }
        if *cfg.aliases() {
            ensure_aliases_field_exists(variant, fields_named, enum_ident)?;
        }

        // 3) Create pattern + identify the special field bindings
        let (variant_pattern, name_binding, hist_binding, alias_binding) =
            create_variant_pattern_and_bindings(enum_ident, var_ident, fields_named);

        // 4) Build arms for the "name()" trait method
        push_name_arm(&mut builder, enum_ident, var_ident, &variant_pattern, name_binding.as_ref());

        // 5) Build arms for "set_name(...)"
        push_set_name_arm(
            &mut builder,
            cfg,
            enum_ident,
            var_ident,
            &variant_pattern,
            name_binding.as_ref(),
            hist_binding.as_ref(),
        );

        // 6) Build arms for "NameHistory" trait if history=true
        if *cfg.history() {
            push_history_arms(
                &mut builder,
                enum_ident,
                var_ident,
                &variant_pattern,
                hist_binding.as_ref()
            );
        }

        // 7) Build arms for "NamedAlias" trait if aliases=true
        if *cfg.aliases() {
            push_aliases_arms(
                &mut builder,
                enum_ident,
                var_ident,
                &variant_pattern,
                alias_binding.as_ref()
            );
        }
    }

    debug!("Completed building match arms for enum '{}'", enum_ident);

    // 8) Finalize the builder
    builder
        .build()
        .map_err(|e| syn::Error::new_spanned(
            ast,
            format!("Error building EnumArms: {:?}", e)
        ))
}

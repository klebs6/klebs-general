// ---------------- [ File: src/expand_enum_named_item.rs ]
crate::ix!();

pub fn expand_enum_named_item(
    ast: &syn::DeriveInput,
    de: &syn::DataEnum,
    cfg: &NamedItemConfig
) -> syn::Result<proc_macro::TokenStream> {
    debug!("Generating NamedItem impl for enum '{}'", ast.ident);

    let enum_ident = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // We'll use this fallback for DefaultName if no default_name is provided
    let fallback_name = cfg
        .default_name()
        .clone()
        .unwrap_or_else(|| enum_ident.to_string());
    trace!("Resolved fallback_name = '{}'", fallback_name);

    // Validate + build the match arms for the enum
    let arms = validate_and_build_enum_arms(ast, de, cfg)?;

    // Generate each portion of the enum's expanded code
    let impl_named = generate_named_impl_for_enum(
        enum_ident,
        &impl_generics,
        &ty_generics,
        where_clause,
        arms.name_arms()
    );
    let impl_default_name = generate_default_name_impl_for_enum(
        enum_ident,
        &impl_generics,
        &ty_generics,
        where_clause,
        &fallback_name
    );
    let impl_reset_name = generate_reset_name_impl_for_enum(
        enum_ident,
        &impl_generics,
        &ty_generics,
        where_clause
    );
    let impl_set_name = generate_set_name_impl_for_enum(
        enum_ident,
        &impl_generics,
        &ty_generics,
        where_clause,
        arms.set_name_arms()
    );

    // Collect optional expansions
    let mut optional_impls = Vec::new();
    if *cfg.history() {
        optional_impls.push(generate_name_history_impl_for_enum(
            enum_ident,
            &impl_generics,
            &ty_generics,
            where_clause,
            arms.history_arms_add(),
            arms.history_arms_get()
        ));
    }
    if *cfg.aliases() {
        optional_impls.push(generate_named_alias_impl_for_enum(
            enum_ident,
            &impl_generics,
            &ty_generics,
            where_clause,
            cfg,
            arms.aliases_arms_add(),
            arms.aliases_arms_get(),
            arms.aliases_arms_clear()
        ));
    }

    // Combine everything
    let final_expanded = quote! {
        #impl_named
        #impl_default_name
        #impl_reset_name
        #impl_set_name
        #( #optional_impls )*
    };

    debug!("Enum '{}' expansion complete", enum_ident);
    Ok(final_expanded.into())
}

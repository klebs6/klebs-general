// ---------------- [ File: src/expand_struct_named_item.rs ]
crate::ix!();

pub fn expand_struct_named_item(
    ast: &syn::DeriveInput,
    ds:  &syn::DataStruct,
    cfg: &NamedItemConfig
) -> syn::Result<proc_macro::TokenStream> {
    debug!("Generating NamedItem impl for struct '{}'", ast.ident);

    let struct_name = &ast.ident;
    let (impl_generics, ty_generics, where_clause) = ast.generics.split_for_impl();

    // Validate fields before proceeding
    validate_struct_fields(ast, ds, cfg)?;

    // We'll use this fallback for DefaultName if no default_name is provided
    let fallback_name = cfg.default_name().clone().unwrap_or_else(|| struct_name.to_string());
    trace!("Resolved fallback_name = '{}'", fallback_name);

    // Generate the baseline implementations (Named, DefaultName, ResetName)
    let baseline_impl = generate_baseline_impl(struct_name, &impl_generics, &ty_generics, where_clause, &fallback_name);

    // Generate the SetName (and optional NameHistory) implementation
    let setname_impl = generate_setname_impl(struct_name, &impl_generics, &ty_generics, where_clause, cfg);

    // Generate the NamedAlias implementation if aliases=true
    let alias_impl = generate_alias_impl(struct_name, &impl_generics, &ty_generics, where_clause, cfg);

    // Combine them all
    let expanded = quote! {
        #baseline_impl
        #setname_impl
        #alias_impl
    };

    debug!("Struct '{}' expansion complete", struct_name);
    Ok(expanded.into())
}

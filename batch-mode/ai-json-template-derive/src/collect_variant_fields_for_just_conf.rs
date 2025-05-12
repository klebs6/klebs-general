crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn collect_variant_fields_for_just_conf(
    data_enum: &syn::DataEnum,
    parent_ident: &syn::Ident,
    span: proc_macro2::Span,
    enum_just_ident: &syn::Ident,
    enum_conf_ident: &syn::Ident,
) -> (
    Vec<proc_macro2::TokenStream>, // just_variants
    Vec<proc_macro2::TokenStream>, // conf_variants
    Option<syn::Ident>,            // first_variant_ident
    Vec<String>,                   // first_variant_just_fields
    Vec<String>                    // first_variant_conf_fields
)
{
    debug!("Collecting variant fields for justification/conf in '{}'", parent_ident);

    let mut first_variant_ident       = None;
    let mut first_variant_just_fields = Vec::<String>::new();
    let mut first_variant_conf_fields = Vec::<String>::new();

    let mut just_variants = Vec::new();
    let mut conf_variants = Vec::new();

    for (i, variant) in data_enum.variants.iter().enumerate() {
        let var_ident = &variant.ident;
        if i == 0 {
            first_variant_ident = Some(var_ident.clone());
        }

        let skip_self_just = is_justification_disabled_for_variant(variant);
        trace!(
            "Variant '{}' -> skip_self_just={}",
            var_ident,
            skip_self_just
        );

        match &variant.fields {
            syn::Fields::Unit => {
                let (jvar, cvar, maybe_j, maybe_c) =
                    handle_unit_variant(var_ident, skip_self_just);
                just_variants.push(jvar);
                conf_variants.push(cvar);

                if i == 0 {
                    if let Some(j) = maybe_j { first_variant_just_fields.push(j); }
                    if let Some(c) = maybe_c { first_variant_conf_fields.push(c); }
                }
            }

            syn::Fields::Named(named_fields) => {
                let (jvar, cvar, jfields, cfields) =
                    handle_named_variant(var_ident, named_fields, skip_self_just, i == 0);
                just_variants.push(jvar);
                conf_variants.push(cvar);

                if i == 0 {
                    first_variant_just_fields.extend(jfields);
                    first_variant_conf_fields.extend(cfields);
                }
            }

            syn::Fields::Unnamed(unnamed_fields) => {
                let (jvar, cvar, jfields, cfields) =
                    handle_unnamed_variant(var_ident, unnamed_fields, skip_self_just, i == 0);
                just_variants.push(jvar);
                conf_variants.push(cvar);

                if i == 0 {
                    first_variant_just_fields.extend(jfields);
                    first_variant_conf_fields.extend(cfields);
                }
            }
        }
    }

    (
        just_variants,
        conf_variants,
        first_variant_ident,
        first_variant_just_fields,
        first_variant_conf_fields
    )
}

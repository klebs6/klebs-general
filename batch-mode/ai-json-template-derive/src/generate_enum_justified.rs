// ---------------- [ File: ai-json-template-derive/src/generate_enum_justified.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_enum_justified(
    ty_ident: &syn::Ident,
    data_enum: &syn::DataEnum,
    span: proc_macro2::Span
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream, proc_macro2::TokenStream)
{
    trace!("Beginning generate_enum_justified for enum '{}'", ty_ident);

    let enum_just_ident = syn::Ident::new(&format!("{}Justification", ty_ident), span);
    let enum_conf_ident = syn::Ident::new(&format!("{}Confidence",   ty_ident), span);
    let justified_ident = syn::Ident::new(&format!("Justified{}",    ty_ident), span);

    let (
        just_variants,
        conf_variants,
        first_variant_ident,
        first_variant_just_fields,
        first_variant_conf_fields
    ) = collect_variant_fields_for_just_conf(data_enum, &ty_ident, span, &enum_just_ident, &enum_conf_ident);

    let enum_just_ts = build_enum_justification(
        &enum_just_ident,
        &just_variants,
        first_variant_ident.as_ref(),
        &first_variant_just_fields
    );

    let enum_conf_ts = build_enum_confidence(
        &enum_conf_ident,
        &conf_variants,
        first_variant_ident.as_ref(),
        &first_variant_conf_fields
    );

    let justified_ts = build_justified_enum_struct(
        &ty_ident,
        &enum_just_ident,
        &enum_conf_ident,
        &justified_ident
    );

    trace!("Completed generate_enum_justified for enum '{}'", ty_ident);
    (enum_just_ts, enum_conf_ts, justified_ts)
}

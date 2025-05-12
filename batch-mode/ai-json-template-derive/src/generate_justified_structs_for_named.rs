// ---------------- [ File: ai-json-template-derive/src/generate_justified_structs_for_named.rs ]
crate::ix!();

/// Refactored version of the old `generate_justified_structs_for_named`,
/// broken down into single-purpose, well-traced subroutines.
/// It returns the token streams for:
///   1) the FooJustification struct
///   2) the FooConfidence struct
///   3) the JustifiedFoo struct
///   4) the accessor impl block
///
/// # Arguments
/// * `ty_ident`     - The original named struct's identifier
/// * `named_fields` - The named fields of that struct
/// * `span`         - The proc-macro2::Span used to generate new Ident(s)
///
pub fn generate_justified_structs_for_named(
    ty_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    span: proc_macro2::Span,
) -> (
    proc_macro2::TokenStream, // justification struct
    proc_macro2::TokenStream, // confidence struct
    proc_macro2::TokenStream, // justified struct
    proc_macro2::TokenStream, // accessor expansions
) {
    trace!(
        "Beginning refactored generate_justified_structs_for_named for '{}'",
        ty_ident
    );

    // (1) Create the 3 Ident values
    let (justification_ident, confidence_ident, justified_ident) =
        gather_named_struct_just_conf_idents(ty_ident, span);

    // (2) Gather the field expansions => justification/conf fields, plus any errors
    let (just_fields, conf_fields, errs, mappings) = gather_fields_for_just_conf(named_fields);

    // (3) Build the two struct definitions => e.g. `FooJustification`, `FooConfidence`
    let (just_ts, conf_ts) =
        build_just_and_conf_structs(&justification_ident, &confidence_ident, &errs, &just_fields, &conf_fields);

    // (4) Build the `JustifiedFoo` struct
    let justified_ts = build_justified_struct(&justified_ident, ty_ident, &justification_ident, &confidence_ident);

    // (5) Build the accessor impl for JustifiedFoo
    let accessor_ts = build_justified_struct_accessors(
        &justified_ident,
        named_fields,
        ty_ident,
        &mappings,
    );

    debug!(
        "Finished generate_justified_structs_for_named for '{}'",
        ty_ident
    );

    (just_ts, conf_ts, justified_ts, accessor_ts)
}

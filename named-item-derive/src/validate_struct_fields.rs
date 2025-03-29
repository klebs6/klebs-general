// ---------------- [ File: src/validate_struct_fields.rs ]
crate::ix!();

pub fn validate_struct_fields(
    ast: &syn::DeriveInput,
    ds: &syn::DataStruct,
    cfg: &NamedItemConfig
) -> syn::Result<()> {
    let struct_name = &ast.ident;

    // 1) Ensure we have a struct with named fields
    let named_fields = validate_named_struct(ast, ds, struct_name)?;

    // 2) Ensure there's a `name: String` field
    ensure_string_name_field(ast, named_fields, struct_name)?;

    // 3) If history=true => require `name_history: Vec<String>`
    if *cfg.history() {
        trace!("history=true => checking for 'name_history: Vec<String>'");
        ensure_name_history_field(ast, named_fields, struct_name)?;
    }

    // 4) If aliases=true => require `aliases: Vec<String>`
    if *cfg.aliases() {
        trace!("aliases=true => checking for 'aliases: Vec<String>'");
        ensure_aliases_field(ast, named_fields, struct_name)?;
    }

    Ok(())
}

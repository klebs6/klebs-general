crate::ix!();

pub struct FlattenedFieldResult {
    field_decls_for_fields:   Vec<proc_macro2::TokenStream>,
    pattern_vars_for_fields:  Vec<proc_macro2::TokenStream>,
    item_inits:               Vec<proc_macro2::TokenStream>,
    just_inits_for_fields:    Vec<proc_macro2::TokenStream>,
    conf_inits_for_fields:    Vec<proc_macro2::TokenStream>,
}

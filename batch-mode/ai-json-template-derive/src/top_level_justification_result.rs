crate::ix!();

pub struct TopLevelJustResult {
    field_decls_top:    Vec<proc_macro2::TokenStream>,
    pattern_vars_top:   Vec<proc_macro2::TokenStream>,
    just_inits_top:     Vec<proc_macro2::TokenStream>,
    conf_inits_top:     Vec<proc_macro2::TokenStream>,
}

// ---------------- [ File: ai-json-template-derive/src/top_level_justification_result.rs ]
crate::ix!();

#[derive(Debug,Getters,MutGetters,Builder)]
#[getset(get="pub",get_mut="pub")]
#[builder(setter(into))]
pub struct TopLevelJustResult {
    field_decls_top:    Vec<proc_macro2::TokenStream>,
    pattern_vars_top:   Vec<proc_macro2::TokenStream>,
    just_inits_top:     Vec<proc_macro2::TokenStream>,
    conf_inits_top:     Vec<proc_macro2::TokenStream>,
}

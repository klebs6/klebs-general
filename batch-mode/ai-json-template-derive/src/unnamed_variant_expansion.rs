// ---------------- [ File: ai-json-template-derive/src/unnamed_variant_expansion.rs ]
crate::ix!();

#[derive(Builder,MutGetters,Getters,Debug)]
#[getset(get="pub",get_mut="pub")]
#[builder(setter(into))]
pub struct UnnamedVariantExpansion {
    field_declarations: Vec<TokenStream2>,
    pattern_vars:       Vec<TokenStream2>,
    item_exprs:         Vec<TokenStream2>,
    just_vals:          Vec<TokenStream2>,
    conf_vals:          Vec<TokenStream2>,
}

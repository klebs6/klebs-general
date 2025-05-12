crate::ix!();

#[derive(Getters,Debug)]
#[getset(get="pub")]
pub struct UnnamedVariantExpansion {
    field_declarations: Vec<TokenStream2>,
    pattern_vars:       Vec<TokenStream2>,
    item_exprs:         Vec<TokenStream2>,
    just_vals:          Vec<TokenStream2>,
    conf_vals:          Vec<TokenStream2>,
}

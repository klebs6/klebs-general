#![allow(dead_code)]
#![allow(unused_imports)]
extern crate proc_macro;
#[macro_use] mod imports; use imports::*;

xp!{find_ai_attr}
xp!{find_feature_if_none}
xp!{get_option_inner_type}
xp!{has_ai_display}
xp!{is_option_type}
xp!{item_with_features}
xp!{impl_item_feature}
xp!{impl_item_feature_enum}
xp!{impl_item_feature_struct}
xp!{process_field}
xp!{process_variant}

#[proc_macro_derive(ItemFeature, attributes(ai))]
pub fn item_feature_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Generate the implementation
    impl_item_feature(&input).into()
}

#[proc_macro_derive(ItemWithFeatures, attributes(ai))]
pub fn item_with_features_derive(input: TokenStream) -> TokenStream {

    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);

    // Ensure the input is a struct
    let struct_data = match &input.data {
        Data::Struct(data) => data,
        _ => {
            return Error::new_spanned(input.ident, "ItemWithFeatures can only be derived for structs")
                .to_compile_error()
                .into();
        }
    };

    // Generate the implementation
    impl_item_with_features(&input, struct_data).into()
}

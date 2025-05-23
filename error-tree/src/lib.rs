#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]
extern crate proc_macro;

#[macro_use] mod imports; use imports::*;

xp!{conversion_chain}
xp!{partial_eq}
xp!{error_enum}
xp!{error_field}
xp!{error_tree_visitor}
xp!{error_tree}
xp!{error_variant}
xp!{errors}
xp!{from_impl_generation_config}
xp!{from_impl_generation_config_emitter}
xp!{types}
xp!{validate}
xp!{type_key}

#[proc_macro]
pub fn error_tree(input: TokenStream) -> TokenStream {

    let error_tree = parse_macro_input!(input as ErrorTree);

    error_tree.into_token_stream().into()
}

// ---------------- [ File: src/build_operator_type_args.rs ]
crate::ix!();

/// Utility to rebuild angle brackets `<...>` from `AngleArg` info.
/// e.g. final_args=[Reused(Z), Fresh(OpTy0)] => `<Z, OpTy0>`
/// If final_args is empty, returns an empty TokenStream (no `< >`).
pub fn build_operator_type_args(final_args: &[AngleArg]) -> proc_macro2::TokenStream {
    if final_args.is_empty() {
        return proc_macro2::TokenStream::new();
    }

    let mut tokens = Vec::with_capacity(final_args.len());
    for arg in final_args {
        match arg {
            AngleArg::Reused(id)   => tokens.push(quote::quote!(#id)),
            AngleArg::Fresh(id)    => tokens.push(quote::quote!(#id)),
            // e.g. `Literal(GenericArgument::Type(Type::Path(...)))`
            // or const expressions, etc.
            AngleArg::Literal(ga)  => tokens.push(quote::quote!(#ga)),
        }
    }
    quote::quote!(< #(#tokens),* >)
}

crate::ix!();
// -----------------------------[File: hydro2-operator-derive/src/generate_port_aware_conversions.rs]

/// A revised version of `generate_port_aware_conversions` that:
///
/// 1) Gives “fallback” first priority — meaning if the operator *actually* uses
///    `()` (unit) as an input or output, we rely on the single blanket impl
///    (`PortTryFrom<()> for T where T: FallbackPortTryFromUnit`) instead of
///    generating an explicit `impl PortTryFrom<()>`.
///
/// 2) If the operator does *not* use `()` at all, we skip implementing the
///    fallback marker trait “because we don’t need it.”
///
/// 
/// This prevents E0119 collisions while letting the blanket fallback handle
/// `()` ports if they exist, or omitting it when they don’t. The “fallback” is
/// thus always first priority if `()` is actually used.
pub fn generate_port_aware_conversions(
    // The identifier of your operator‐IO enum, e.g. `MyOperatorIO`.
    io_enum_ident: &syn::Ident,
    generics:      &syn::Generics,
    // The operator spec telling us which input/outputs exist.
    operator_spec: &OperatorSpec,

) -> proc_macro2::TokenStream
{
    let port_aware_try_into_impls = generate_port_aware_try_into_impls(
        io_enum_ident,
        generics,
        operator_spec,
    );

    let port_aware_try_from_impls = generate_port_aware_try_from_impls(
        io_enum_ident,
        generics,
        operator_spec,
    );

    let active_port_impl = generate_active_output_port_impl(
        io_enum_ident,
        generics,
        operator_spec,
    );

    // Combine everything
    quote::quote! {
        #port_aware_try_into_impls
        #port_aware_try_from_impls
        #active_port_impl
    }
}

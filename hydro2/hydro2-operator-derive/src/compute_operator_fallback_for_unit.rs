// ---------------- [ File: src/compute_operator_fallback_for_unit.rs ]
crate::ix!();

/// A simple convention is:  
/// **Enable** the blanket fallback for `()` **unless** this particular operator
/// already has an explicit `()` input or output.  
///
/// Why do this?  
/// - If the operator spec **never** mentions `()`, then we are free to let a
///   single blanket `impl<T: FallbackPortTryFromUnit> PortTryFrom<()>` handle
///   the “unit” case; no conflicts arise, and it fails gracefully at runtime.
/// - If the operator spec **explicitly** uses `()` as an input or output type,
///   then your code already generates a concrete `impl PortTryFrom<()>` or
///   `impl PortTryInto<()>`. That would **overlap** with the blanket fallback
///   and trigger E0119 (conflicting implementations).
///
/// The function below checks if any input or output is literally `()`. If so,
/// we **disable** the fallback; otherwise, we enable it.
///
/// In your `#[proc_macro_derive(Operator)]` flow, just call
/// `operator_has_fallback_for_unit = compute_operator_fallback_for_unit(&operator_spec)`,
/// then pass that boolean into `generate_port_aware_conversions`.
///
/// Of course, if you prefer a different policy (like always `true` or always `false`),
/// you can replace this logic with your own.
pub fn compute_operator_fallback_for_unit(spec: &OperatorSpec) -> bool {
    // If any input is "()", skip fallback.
    for input_ty in spec.inputs() {
        if is_unit_type(input_ty) {
            return false;
        }
    }
    // If any output is "()", skip fallback.
    for output_ty in spec.outputs() {
        if is_unit_type(output_ty) {
            return false;
        }
    }

    // Otherwise, no explicit `()` in the spec => enable the fallback.
    true
}

/// Helper that checks if a `syn::Type` is literally `()`.
fn is_unit_type(ty: &syn::Type) -> bool {
    // Easiest textual check: "()" → after tokenizing => "()".
    // More robustly, we can match on `Type::Tuple(tup)` and see if `tup.elems.is_empty()`.
    matches!(ty, syn::Type::Tuple(tup) if tup.elems.is_empty())
}

// Example usage in your macro’s derive flow:
//
// ```rust,ignore
// // In derive_operator():
// let operator_spec = OperatorSpec::parse_operator_attrs(&input_ast.attrs, struct_span)?;
// // ...
// let operator_has_fallback_for_unit = compute_operator_fallback_for_unit(&operator_spec);
//
// let port_conversions = generate_port_aware_conversions(
//     &io_enum_ident,
//     &quote! { #impl_generics },
//     &quote! { #type_generics },
//     &quote! { #where_clause },
//     operator_has_fallback_for_unit,  // <--- the boolean
//     &operator_spec,
// );
//
// // Combine that with your other expansions...
// ```

// ---------------- [ File: src/lib.rs ]
#![allow(unused)]

//! lib.rs — The `hydro2-network-wire-derive` crate implementing `[derive(NetworkWire)]`.
#[macro_use] mod imports; use imports::*;

// ----------------------------------------------------------------------
//  The main macro entry point
// ----------------------------------------------------------------------
#[proc_macro_derive(NetworkWire, attributes(available_operators))]
pub fn derive_network_wire(input: RawTokenStream) -> RawTokenStream {
    let input_ast = parse_macro_input!(input as DeriveInput);

    let (mut op_items, wire_generics) = match parse_available_operators_attribute(&input_ast) {
        Ok((ops, gens)) => (ops, gens),
        Err(ts) => return ts.into(),
    };

    let struct_ident = &input_ast.ident;

    let (impl_generics, ty_generics, where_clause) = wire_generics.split_for_impl();
    let enum_ident = syn::Ident::new(&format!("{}IO", struct_ident), struct_ident.span());

    // Build the enum, then the bridging impls
    let (enum_def,variant_idents) = build_network_io_enum(
        &enum_ident,
        &op_items,
        &quote! { #impl_generics },
        &quote! { #ty_generics },
        &quote! { #where_clause }
    );

    let port_try_into_for_enum_def = build_port_try_into_impl(
        &enum_ident,
        &wire_generics,
        &variant_idents,
    );

    let bridging_impls = build_bridging_impls(&enum_ident, &mut op_items, &wire_generics);

    let expanded = quote! {
        #enum_def
        #port_try_into_for_enum_def
        #( #bridging_impls )*
    };
    expanded.into()
}

xp!{angle_arg}
xp!{parse_strings}
xp!{build_operator_type_args}
xp!{build_bridging_impls}
xp!{build_network_io_enum}
xp!{build_operator_signature_map}
xp!{build_single_operator_impl}
xp!{combine_where_clauses}
xp!{extract_generics_from_path}
xp!{finalize_operator_io_path}
xp!{find_operator_signature_ident}
xp!{maybe_reuse_lifetime}
xp!{maybe_reuse_type_or_const}
xp!{merge_generics}
xp!{mint_const_param}
xp!{mint_lifetime_param}
xp!{mint_type_param}
xp!{operator_items_parser}
xp!{operator_spec_item}
xp!{parse_available_operators_from_attribute}
xp!{split_op_generics}
xp!{split_wire_generics}
xp!{try_reuse_wire_params}
xp!{unify_generics_ast}
xp!{build_port_try_into_impl}

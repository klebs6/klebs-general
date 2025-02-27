// ---------------- [ File: src/lib.rs ]
#![allow(unused_doc_comments)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![feature(iter_intersperse)]
#[macro_use] mod imports; use imports::*;

xp!{build_execute_body}
xp!{build_port_arms}
xp!{errors}
xp!{generate_io_enum}
xp!{generate_operator_impl}
xp!{generate_active_output_port_impl}
xp!{operator_spec}
xp!{operator_key_values}
xp!{type_contains_lifetime_reference}
xp!{generate_port_aware_conversions}
xp!{generate_port_aware_try_into}
xp!{generate_port_aware_try_from}
xp!{generate_operator_signature_tokens}
xp!{generate_port_try_into}
xp!{generate_port_try_from}
xp!{phantom_data_for_generics}

type RawTokenStream = proc_macro::TokenStream;

/// The main entry point for `[derive(Operator)]`.
#[proc_macro_derive(Operator, attributes(operator))]
pub fn derive_operator(input: RawTokenStream) -> RawTokenStream {

    let input_ast    = parse_macro_input!(input as DeriveInput);
    let struct_ident = &input_ast.ident;
    let struct_span  = struct_ident.span();

    // Ensure struct (named, tuple, or unit)
    match &input_ast.data {
        Data::Struct(DataStruct { fields: Fields::Named(_), .. })   => {}
        Data::Struct(DataStruct { fields: Fields::Unnamed(_), .. }) => {}
        Data::Struct(DataStruct { fields: Fields::Unit, .. })       => {}
        _ => {
            return syn::Error::new(struct_span, "Operator derive can only be used on structs.")
                .to_compile_error()
                .into();
        }
    }

    // Parse operator(...) attributes => OperatorSpec
    let operator_spec = match OperatorSpec::parse_operator_attrs(&input_ast.attrs, struct_span) {
        Ok(spec) => spec,
        Err(e) => {
            return e.to_compile_error().into();
        }
    };

    // Prepare generics
    let mut generics = input_ast.generics.clone();

    // Check if we need a lifetime `'a`
    let uses_lifetime = operator_spec
        .inputs()
        .iter()
        .any(|ty| contains_lifetime_reference(ty));

    if uses_lifetime {
        let lifetime_ident = Lifetime::new("'a", struct_span);
        // Insert a `LifetimeParam` if it's not already present.
        if !generics.params.iter().any(|param| match param {
            GenericParam::Lifetime(lp) => lp.lifetime.ident == lifetime_ident.ident,
            _ => false,
        }) {
            let lifetime_param = LifetimeParam::new(lifetime_ident);
            generics.params.insert(0, GenericParam::Lifetime(lifetime_param));
        }
    }

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    // Generate FooOpIO
    let enum_def = generate_io_enum(
        struct_ident,
        &operator_spec,
        uses_lifetime,
        &quote! { #impl_generics },
        &quote! { #type_generics },
        &quote! { #where_clause },
    );

    // The enum ident is `FooOpIO`
    let io_enum_ident = Ident::new(&format!("{}IO", struct_ident), struct_span);

    // Generate `impl Operator<StructIO> for Struct`
    let operator_impl = generate_operator_impl(
        struct_ident,
        &operator_spec,
        &io_enum_ident,
        &quote! { #impl_generics },
        &quote! { #type_generics },
        &quote! { #where_clause },
    );

    // Generate your new port-aware TryFrom/TryInto
    let port_conversions = generate_port_aware_conversions(
        &io_enum_ident,
        &generics,
        &operator_spec,
    );

    // **Now**: build the hidden signature type, e.g. `__FooOp__OperatorSignature`
    let sig_ident = format_ident!("{}OperatorSignature", struct_ident);

    let operator_sig_tokens = generate_operator_signature_tokens(
        &sig_ident, 
        &operator_spec,
        &quote! { #impl_generics },
        &quote! { #type_generics },
        &quote! { #where_clause },
        &generics,
    );

    let expanded = quote! {
        #enum_def
        #operator_impl
        #port_conversions
        #operator_sig_tokens
    };

    expanded.into()
}

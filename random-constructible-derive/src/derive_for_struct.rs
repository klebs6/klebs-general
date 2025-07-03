// ---------------- [ File: src/derive_for_struct.rs ]
crate::ix!();

pub fn derive_random_constructible_for_struct(input: &DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    let data_struct = if let Data::Struct(ref data_struct) = input.data {
        data_struct
    } else {
        panic!("Expected struct data");
    };

    match &data_struct.fields {
        Fields::Named(fields_named)     => expand_rand_construct_for_named_struct(name, fields_named),
        Fields::Unnamed(fields_unnamed) => expand_rand_construct_for_tuple_struct(name, fields_unnamed, &input.generics),
        Fields::Unit                    => derive_for_unit_struct(name),
    }
}

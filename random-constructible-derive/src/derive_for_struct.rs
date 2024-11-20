crate::ix!();

pub fn derive_random_constructible_for_struct(input: &DeriveInput) -> TokenStream2 {
    let name = &input.ident;

    let data_struct = if let Data::Struct(ref data_struct) = input.data {
        data_struct
    } else {
        panic!("Expected struct data");
    };

    match &data_struct.fields {
        Fields::Named(fields_named) => derive_for_named_fields(name, fields_named),
        Fields::Unnamed(fields_unnamed) => derive_for_unnamed_fields(name, fields_unnamed),
        Fields::Unit => derive_for_unit_struct(name),
    }
}

crate::ix!();

pub fn build_justified_struct_accessors(
    justified_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    ty_ident: &syn::Ident,
    field_mappings: &[FieldJustConfMapping],
) -> proc_macro2::TokenStream {
    trace!(
        "Building accessor impl for the 'Justified' struct => '{}'",
        justified_ident
    );

    let (item_acc, just_acc, conf_acc) =
        gather_item_accessors(named_fields, ty_ident, field_mappings);

    let expanded = quote::quote! {
        impl #justified_ident {
            #(#item_acc)*
            #(#just_acc)*
            #(#conf_acc)*
        }
    };

    debug!(
        "Accessor impl for '{}' now has item/just/conf methods: total={}",
        justified_ident,
        item_acc.len() + just_acc.len() + conf_acc.len()
    );
    expanded
}

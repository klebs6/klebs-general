#![allow(unused_imports)]
extern crate proc_macro;

#[macro_use] mod imports; use imports::*;

xp!{derive_ai_descriptor_for_enum}
xp!{derive_ai_descriptor_for_struct}
xp!{has_ai_display_attribute}
xp!{has_display_attribute}
xp!{has_display_in_attribute}
xp!{is_ai_attribute_with_display}
xp!{is_display_attribute}
xp!{is_option_type}
xp!{generate_display_impl}
xp!{generate_ai_descriptor_impl}
xp!{generate_format_string_for_variant}
xp!{generate_field_patterns_for_unnamed_fields}
xp!{generate_field_bindings}
xp!{generate_field_ai_idents}
xp!{generate_variant_arm_for_unnamed_fields}
xp!{generate_unit_variant_arm}
xp!{extract_field_names}
xp!{generate_named_field_bindings}
xp!{generate_named_field_ai_idents}
xp!{generate_named_field_format_string}
xp!{generate_variant_arm_for_named_fields}
xp!{generate_variant_arm}
xp!{generate_variant_arms}

#[proc_macro_derive(AIDescriptor, attributes(ai))]
pub fn derive_ai_descriptor(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input_ast = parse_macro_input!(input as DeriveInput);

    // Get the name of the type (enum or struct)
    let type_name = &input_ast.ident;
    let attrs     = &input_ast.attrs;

    // Check if the input is an enum or struct
    let token_stream2 = match &input_ast.data {
        Data::Enum(data_enum)     => derive_ai_descriptor_for_enum(type_name, data_enum, &attrs),
        Data::Struct(data_struct) => derive_ai_descriptor_for_struct(type_name, data_struct, &attrs),
        _ => {
            syn::Error::new_spanned(
                &type_name,
                "AIDescriptor can only be derived for enums and structs",
            )
            .to_compile_error()
            .into()
        }
    };

    token_stream2.into()
}

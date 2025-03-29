// ---------------- [ File: src/lib.rs ]
//! named-item-derive — a derive macro for implementing `Named`, `SetName`, etc.

#[macro_use] mod imports; use imports::*;

xp!{create_variant_pattern_and_bindings}
xp!{ensure_aliases_field}
xp!{ensure_aliases_field_exists}
xp!{ensure_history_field_exists}
xp!{ensure_name_field_exists}
xp!{ensure_name_history_field}
xp!{ensure_string_name_field}
xp!{enum_arms}
xp!{expand_enum_named_item}
xp!{expand_struct_named_item}
xp!{generate_alias_impl}
xp!{generate_baseline_impl}
xp!{generate_default_name_impl_for_enum}
xp!{generate_name_history_impl_for_enum}
xp!{generate_named_alias_impl_for_enum}
xp!{generate_named_impl_for_enum}
xp!{generate_reset_name_impl_for_enum}
xp!{generate_set_name_impl_for_enum}
xp!{generate_setname_impl}
xp!{impl_named_item}
xp!{named_item_config}
xp!{parse_named_item_attrs}
xp!{push_aliases_arms}
xp!{push_history_arms}
xp!{push_name_arm}
xp!{push_set_name_arm}
xp!{validate_and_build_enum_arms}
xp!{validate_named_struct}
xp!{validate_struct_fields}
xp!{validate_variant_is_named}

/// The attribute macro to derive Named, DefaultName, SetName, etc. behaviors.
#[proc_macro_derive(NamedItem, attributes(named_item))]
pub fn derive_named_item(input: TokenStream) -> TokenStream {

    let ast = parse_macro_input!(input as DeriveInput);

    // Parse the user-provided #[named_item(...)] attributes
    let config = match parse_named_item_attrs(&ast) {
        Ok(cfg) => cfg,
        Err(e) => return e.to_compile_error().into(),
    };

    // Generate the final code
    match impl_named_item(&ast, &config) {
        Ok(ts) => ts,
        Err(err) => err.to_compile_error().into(),
    }
}

#[cfg(test)]
mod test_named_expansion_subroutines {
    use super::*;

    #[traced_test]
    fn test_expand_struct_named_item() {
        // We'll simulate a struct with the required fields
        let input: DeriveInput = parse_quote! {
            struct TestStruct {
                name: String,
                name_history: Vec<String>,
                aliases: Vec<String>,
            }
        };

        let ds = match &input.data {
            Data::Struct(ds) => ds,
            _ => panic!("Expected a struct."),
        };

        let cfg = NamedItemConfig {
            default_name: Some("HelloStruct".to_string()),
            aliases: true,
            default_aliases: vec!["foo".into(), "bar".into()],
            history: true,
        };

        let expanded = expand_struct_named_item(&input, ds, &cfg)
            .expect("expand_struct_named_item should succeed");
        // Just do a sanity check that it contains references to our fallback name
        let tokens_str = expanded.to_string();
        assert!(tokens_str.contains("HelloStruct"));
    }

    #[traced_test]
    fn test_expand_enum_named_item() {
        // We'll simulate an enum with the required fields in variants
        let input: DeriveInput = parse_quote! {
            enum TestEnum {
                VariantA { name: String, name_history: Vec<String>, aliases: Vec<String> },
                VariantB { name: String, name_history: Vec<String>, aliases: Vec<String> },
            }
        };

        let de = match &input.data {
            Data::Enum(de) => de,
            _ => panic!("Expected an enum."),
        };

        let cfg = NamedItemConfig {
            default_name: Some("HelloEnum".to_string()),
            aliases: true,
            default_aliases: vec!["alpha".into(), "beta".into()],
            history: true,
        };

        let expanded = expand_enum_named_item(&input, de, &cfg)
            .expect("expand_enum_named_item should succeed");
        let tokens_str = expanded.to_string();
        assert!(tokens_str.contains("HelloEnum"));
        // Check we have references to "VariantA" or "VariantB" in the expansions
        assert!(tokens_str.contains("VariantA"));
        assert!(tokens_str.contains("VariantB"));
    }
}

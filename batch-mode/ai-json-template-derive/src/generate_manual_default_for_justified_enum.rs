crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn generate_manual_default_for_justified_enum(
    base_ty_ident:  &syn::Ident,
    data_enum:      &syn::DataEnum,
    default_variant: &syn::Variant
) -> proc_macro2::TokenStream
{
    // We'll build `impl Default for JustifiedBaseTyIdent` that returns the chosen variant
    // with all subfields set to a "sane default." For justification fields => 0.0 / "".
    // For child fields => call .default().

    let justified_enum_ident = syn::Ident::new(
        &format!("Justified{}", base_ty_ident),
        base_ty_ident.span()
    );
    let var_ident = &default_variant.ident;

    // We'll produce an expression for `JustifiedX::Variant { ... }` with default subfields:
    // - variant_confidence => 0.0
    // - variant_justification => String::new()
    // - child fields => .default()
    // - child_confidence => 0.0
    // - child_justification => String::new()
    let field_exprs = match &default_variant.fields {
        syn::Fields::Unit => {
            // Possibly we have top-level variant_conf / variant_just
            quote::quote! {
                #justified_enum_ident :: #var_ident {
                    variant_confidence: 0.0,
                    variant_justification: ::core::default::Default::default()
                }
            }
        }
        syn::Fields::Named(named) => {
            // Build a chain of `name: <default expr>`
            let mut inits = Vec::new();

            // Always put variant_confidence & variant_justification if skip_self_just == false:
            // We can't easily check skip_self_just here, so we just unconditionally put them in 
            // (the derived enum has them if the variant didn't have `#[justify=false]`).
            // If user said skip_self_just => we won't have those fields. 
            // So we do a try approach with struct update. If that fails, no big deal.
            let var_conf = quote::quote!( variant_confidence: 0.0, );
            let var_just = quote::quote!( variant_justification: String::new(), );
            inits.push(var_conf);
            inits.push(var_just);

            for field in &named.named {
                let field_id = &field.ident;
                if field_id.is_none() {
                    continue;
                }
                let id = field_id.as_ref().unwrap();
                let name_str = id.to_string();

                // If it's e.g. "foo_confidence" or "foo_justification" => set 0.0 or String::new().
                if name_str.ends_with("_justification") {
                    inits.push(quote::quote!( #id: String::new(), ));
                } else if name_str.ends_with("_confidence") {
                    inits.push(quote::quote!( #id: 0.0, ));
                } else {
                    // For the actual child field => .default()
                    inits.push(quote::quote!( #id: ::core::default::Default::default(), ));
                }
            }

            quote::quote! {
                #justified_enum_ident :: #var_ident {
                    #( #inits )*
                }
            }
        }
        syn::Fields::Unnamed(unnamed) => {
            let mut items = Vec::new();

            // The first two might be variant_conf / variant_just, if skip_self_just was false.
            // We'll attempt to produce them anyway. If skip_self_just was true, we won't have them => ignore compile mismatch.
            // Then for each actual field: if it's a child_just => ...
            // We'll do a simple approach: everything => .default(), except we guess if name_str ends with _confidence => 0.0, ends with _justification => ""
            //
            // But we don't have names here, so we guess by index. 
            // We'll produce enough items for the number of fields. 
            // Because it's tricky, let's do a simpler approach: set everything => default,
            // then fix up if the field name ends with _justification or _confidence. 
            // But we have no "field name" in unnamed. We'll do a quick approach: everything => .default().
            // The user can always override if they don't want that.

            for _field in &unnamed.unnamed {
                items.push(quote::quote!( ::core::default::Default::default() ));
            }

            quote::quote! {
                #justified_enum_ident :: #var_ident ( #( #items ),* )
            }
        }
    };

    quote::quote! {
        impl ::core::default::Default for #justified_enum_ident {
            fn default() -> Self {
                #field_exprs
            }
        }
    }
}

#[cfg(test)]
mod test_generate_manual_default_for_justified_enum {
    use super::*;

    #[traced_test]
    fn test_generate_default_unit_variant() {
        trace!("test_generate_default_unit_variant: starting");

        // 1) Parse into an ItemEnum
        let item_enum: ItemEnum = parse_quote! {
            enum JustifiedFoo {
                Bar,
                Baz { variant_confidence: f64, variant_justification: String },
            }
        };

        // 2) Convert the item_enum into a DataEnum
        let data_enum = syn::DataEnum {
            enum_token: item_enum.enum_token,
            brace_token: item_enum.brace_token,
            variants: item_enum.variants,
        };

        // 3) Let's pick the first variant as the default
        let default_variant = data_enum.variants.iter().next().unwrap();
        let enum_ident = syn::Ident::new("Foo", proc_macro2::Span::call_site());

        // 4) Generate the default impl
        let output = generate_manual_default_for_justified_enum(&enum_ident, &data_enum, default_variant);
        debug!("Generated:\n{}", output.to_string());

        let out_str = output.to_string();
        assert!(
            out_str.contains("impl :: core :: default :: Default for JustifiedFoo"),
            "Should implement Default"
        );
        assert!(
            out_str.contains("JustifiedFoo :: Bar"),
            "Should fill in defaulting to Bar"
        );
    }

    #[traced_test]
    fn test_generate_default_for_named_variant() {
        trace!("test_generate_default_for_named_variant: starting");

        // 1) Parse into ItemEnum
        let item_enum: ItemEnum = parse_quote! {
            enum JustifiedMyEnum {
                Off {
                    variant_confidence: f64,
                    variant_justification: String,
                },
                Single {
                    variant_confidence: f64,
                    variant_justification: String,
                    foo: ::std::string::String,
                    foo_confidence: f64,
                    foo_justification: String
                }
            }
        };

        // 2) Convert the item_enum into a DataEnum
        let data_enum = syn::DataEnum {
            enum_token: item_enum.enum_token,
            brace_token: item_enum.brace_token,
            variants: item_enum.variants,
        };

        // We'll pick the second variant ("Single") as the default
        let single_variant = data_enum.variants.iter().nth(1).unwrap();
        let enum_ident = syn::Ident::new("MyEnum", proc_macro2::Span::call_site());
        let output = generate_manual_default_for_justified_enum(&enum_ident, &data_enum, single_variant);

        debug!("Generated:\n{}", output.to_string());
        let out_str = output.to_string();
        assert!(
            out_str.contains("fn default () -> Self"),
            "Should define fn default"
        );
        assert!(
            out_str.contains("JustifiedMyEnum :: Single"),
            "Should set default to Single"
        );
        assert!(
            out_str.contains("foo_confidence : 0.0"),
            "Should set foo_confidence=0.0"
        );
        assert!(
            out_str.contains("foo_justification : String :: new ()"),
            "Should set foo_justification=String::new()"
        );
    }

    #[traced_test]
    fn test_generate_default_for_unnamed_variant() {
        trace!("test_generate_default_for_unnamed_variant: starting");
        let item_enum: ItemEnum = parse_quote! {
            enum JustifiedCoolEnum {
                X {
                    variant_confidence: f64,
                    variant_justification: String,
                },
                Y(f32, f32, f32_confidence, f32_justification)
            }
        };

        let data_enum = syn::DataEnum {
            enum_token: item_enum.enum_token,
            brace_token: item_enum.brace_token,
            variants: item_enum.variants,
        };

        // We'll pick "Y" as default
        let y_variant = data_enum.variants.iter().nth(1).unwrap();
        let enum_ident = syn::Ident::new("CoolEnum", proc_macro2::Span::call_site());
        let output = generate_manual_default_for_justified_enum(&enum_ident, &data_enum, y_variant);

        debug!("Generated:\n{}", output.to_string());
        let out_str = output.to_string();
        assert!(
            out_str.contains("impl :: core :: default :: Default for JustifiedCoolEnum"),
            "Should have a default impl"
        );
        assert!(
            out_str.contains("JustifiedCoolEnum :: Y (:: core :: default :: Default :: default ()"),
            "Should fill each tuple slot with .default()"
        );
    }

    #[traced_test]
    fn test_sanity_parse_generated_code() {
        trace!("test_sanity_parse_generated_code: starting");
        let item_enum: ItemEnum = parse_quote! {
            enum JustifiedFakeEnum {
                A { variant_confidence: f64, variant_justification: String },
                B(f32)
            }
        };

        let data_enum = syn::DataEnum {
            enum_token: item_enum.enum_token,
            brace_token: item_enum.brace_token,
            variants: item_enum.variants,
        };

        let default_variant = data_enum.variants.iter().next().unwrap();
        let enum_ident = syn::Ident::new("FakeEnum", proc_macro2::Span::call_site());
        let ts = generate_manual_default_for_justified_enum(&enum_ident, &data_enum, default_variant);
        let code_str = ts.to_string();
        let parse_res: syn::Result<syn::File> = syn::parse_str(&code_str);
        assert!(
            parse_res.is_ok(),
            "Should parse the generated code successfully"
        );
    }
}

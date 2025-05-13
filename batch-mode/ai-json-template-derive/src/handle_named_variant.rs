// ---------------- [ File: ai-json-template-derive/src/handle_named_variant.rs ]
crate::ix!();

#[tracing::instrument(level = "trace", skip_all)]
pub fn handle_named_variant(
    var_ident: &syn::Ident,
    named_fields: &syn::FieldsNamed,
    skip_self_just: bool,
    is_first_variant: bool
) -> (
    proc_macro2::TokenStream, // variant in the Justification enum
    proc_macro2::TokenStream, // variant in the Confidence enum
    Vec<String>,              // new justification field names
    Vec<String>               // new confidence field names
)
{
    debug!(
        "Handling named variant '{}', skip_self_just={}, is_first_variant={}",
        var_ident,
        skip_self_just,
        is_first_variant
    );

    let mut j_fields = Vec::new();
    let mut c_fields = Vec::new();
    let mut out_just_names = Vec::new();
    let mut out_conf_names = Vec::new();

    if !skip_self_just {
        j_fields.push(quote::quote! { variant_justification: String, });
        c_fields.push(quote::quote! { variant_confidence: f32, });

        if is_first_variant {
            out_just_names.push("variant_justification".to_string());
            out_conf_names.push("variant_confidence".to_string());
        }
    }

    for field in &named_fields.named {
        if is_justification_enabled(field) {
            let f_id = field.ident.as_ref().unwrap();
            let j_id = syn::Ident::new(
                &format!("{}_justification", f_id),
                f_id.span()
            );
            let c_id = syn::Ident::new(
                &format!("{}_confidence", f_id),
                f_id.span()
            );
            j_fields.push(quote::quote! { #j_id: String, });
            c_fields.push(quote::quote! { #c_id: f32, });

            if is_first_variant {
                out_just_names.push(format!("{}_justification", f_id));
                out_conf_names.push(format!("{}_confidence", f_id));
            }
        }
    }

    let just_variant = quote::quote! {
        #var_ident { #(#j_fields)* }
    };
    let conf_variant = quote::quote! {
        #var_ident { #(#c_fields)* }
    };

    (just_variant, conf_variant, out_just_names, out_conf_names)
}

#[cfg(test)]
mod confirm_handle_named_variant {
    use super::*;

    #[traced_test]
    fn verify_handle_named_variant_skip_and_first_variant_no_fields() {
        trace!("Starting test: skip_self_just=true, is_first_variant=true, with zero fields");
        let var_ident = syn::Ident::new("EmptyVariant", proc_macro2::Span::call_site());
        let named_fields: FieldsNamed = parse_quote!({});
        
        let skip_self_just = true;
        let is_first_variant = true;
        trace!("Invoking handle_named_variant(...) with skip_self_just={} and is_first_variant={}", skip_self_just, is_first_variant);

        let (just_ts, conf_ts, just_field_names, conf_field_names) = handle_named_variant(
            &var_ident,
            &named_fields,
            skip_self_just,
            is_first_variant
        );

        debug!("just_ts: {}", just_ts.to_string());
        debug!("conf_ts: {}", conf_ts.to_string());
        debug!("just_field_names: {:?}", just_field_names);
        debug!("conf_field_names: {:?}", conf_field_names);

        // Since skip_self_just=true, we expect no top-level variant_justification/conf.
        assert!(!just_ts.to_string().contains("variant_justification"));
        assert!(!conf_ts.to_string().contains("variant_confidence"));
        assert_eq!(just_field_names.len(), 0);
        assert_eq!(conf_field_names.len(), 0);
    }

    #[traced_test]
    fn verify_handle_named_variant_no_skip_not_first_variant_with_fields() {
        trace!("Starting test: skip_self_just=false, is_first_variant=false, with multiple fields");
        let var_ident = syn::Ident::new("MyVariant", proc_macro2::Span::call_site());
        // We'll define a named set of fields, with some having #[justify=false].
        // Note that parse_quote!({ ... }) must represent struct fields in braces.
        let named_fields: FieldsNamed = parse_quote!({
            #[justify=false]
            alpha: i32,
            beta: String,
            gamma: bool
        });

        let skip_self_just = false;
        let is_first_variant = false;
        trace!("Invoking handle_named_variant(...) with skip_self_just={} and is_first_variant={}", skip_self_just, is_first_variant);

        let (just_ts, conf_ts, just_field_names, conf_field_names) = handle_named_variant(
            &var_ident,
            &named_fields,
            skip_self_just,
            is_first_variant
        );

        debug!("just_ts: {}", just_ts.to_string());
        debug!("conf_ts: {}", conf_ts.to_string());
        debug!("just_field_names: {:?}", just_field_names);
        debug!("conf_field_names: {:?}", conf_field_names);

        // We do NOT expect top-level variant_justification/conf because skip_self_just=false
        // but is_first_variant=false does not remove them. Actually, top-level fields are added
        // only if skip_self_just=false. So we DO expect them, but we won't gather their names
        // (because is_first_variant=false means we don't push them into the output name vectors).
        assert!(just_ts.to_string().contains("variant_justification"));
        assert!(conf_ts.to_string().contains("variant_confidence"));

        // 'alpha' has #[justify=false], so it should not appear in the justification/conf fields.
        assert!(!just_ts.to_string().contains("alpha_justification"));
        assert!(!conf_ts.to_string().contains("alpha_confidence"));

        // 'beta' has no justification attributes, so it should appear as *_justification/*_confidence.
        assert!(just_ts.to_string().contains("beta_justification"));
        assert!(conf_ts.to_string().contains("beta_confidence"));

        // 'gamma' also has no justification attributes => same as above.
        assert!(just_ts.to_string().contains("gamma_justification"));
        assert!(conf_ts.to_string().contains("gamma_confidence"));

        // Because is_first_variant=false, we expect these not to be recorded in the returned name vectors
        // except for field-specific ones, which also only appear if is_first_variant=true. So we see no new ones.
        // The function accumulates them ONLY if it's the first variant in the enum. So we expect empty.
        assert_eq!(just_field_names.len(), 0);
        assert_eq!(conf_field_names.len(), 0);
    }

    #[traced_test]
    fn verify_handle_named_variant_no_skip_and_first_variant_some_fields() {
        trace!("Starting test: skip_self_just=false, is_first_variant=true, with multiple fields");
        let var_ident = syn::Ident::new("FirstVar", proc_macro2::Span::call_site());
        let named_fields: FieldsNamed = parse_quote!({
            a_str: String,
            #[justify=false]
            an_opt: Option<u32>,
            a_flag: bool
        });

        let skip_self_just = false;
        let is_first_variant = true;
        trace!("Invoking handle_named_variant(...) with skip_self_just={} and is_first_variant={}", skip_self_just, is_first_variant);

        let (just_ts, conf_ts, just_names, conf_names) = handle_named_variant(
            &var_ident,
            &named_fields,
            skip_self_just,
            is_first_variant
        );

        debug!("just_ts: {}", just_ts.to_string());
        debug!("conf_ts: {}", conf_ts.to_string());
        debug!("just_names: {:?}", just_names);
        debug!("conf_names: {:?}", conf_names);

        // Expect top-level variant_justification/variant_confidence
        assert!(just_ts.to_string().contains("variant_justification"));
        assert!(conf_ts.to_string().contains("variant_confidence"));

        // 'a_str' => justification/conf
        assert!(just_ts.to_string().contains("a_str_justification"));
        assert!(conf_ts.to_string().contains("a_str_confidence"));

        // 'an_opt' => skip justification because #[justify=false].
        assert!(!just_ts.to_string().contains("an_opt_justification"));
        assert!(!conf_ts.to_string().contains("an_opt_confidence"));

        // 'a_flag' => justification/conf
        assert!(just_ts.to_string().contains("a_flag_justification"));
        assert!(conf_ts.to_string().contains("a_flag_confidence"));

        // Because is_first_variant=true and skip_self_just=false, the top-level
        // justification/conf field names should also appear in the returned lists.
        // Also, the fields that are not #[justify=false] appear as well.

        // We should see "variant_justification" and "variant_confidence" for the top-level
        // and also "a_str_justification","a_str_confidence","a_flag_justification","a_flag_confidence"
        // in the name vectors. Let's see how many we have:

        // We expect 1 top-level pair + 2 field pairs => 3 pairs => 6 entries total
        // but the function for named variant appends field if justification is enabled
        // and skip_self_just is false => let's see how it lines up in the actual code:

        // Implementation appends "variant_justification" and "variant_confidence" first
        // (if skip_self_just==false), plus each field that is justification enabled.
        // That's "a_str", "a_flag" => 2 more pairs => total 6. So let's confirm:
        assert_eq!(just_names.len(), 3);
        assert_eq!(conf_names.len(), 3);

        // Confirm these sets contain the expected names:
        let just_set: std::collections::HashSet<_> = just_names.into_iter().collect();
        let conf_set: std::collections::HashSet<_> = conf_names.into_iter().collect();
        assert!(just_set.contains("variant_justification"));
        assert!(conf_set.contains("variant_confidence"));
        assert!(just_set.contains("a_str_justification"));
        assert!(conf_set.contains("a_str_confidence"));
        assert!(just_set.contains("a_flag_justification"));
        assert!(conf_set.contains("a_flag_confidence"));
    }

    #[traced_test]
    fn verify_handle_named_variant_skip_and_not_first_variant_some_fields() {
        trace!("Starting test: skip_self_just=true, is_first_variant=false, multiple fields");
        let var_ident = syn::Ident::new("SecondVar", proc_macro2::Span::call_site());
        let named_fields: FieldsNamed = parse_quote!({
            #[justify=false]
            x: i64,
            y: i64
        });

        let skip_self_just = true;
        let is_first_variant = false;
        trace!("Invoking handle_named_variant(...) with skip_self_just={} and is_first_variant={}", skip_self_just, is_first_variant);

        let (just_ts, conf_ts, just_field_names, conf_field_names) = handle_named_variant(
            &var_ident,
            &named_fields,
            skip_self_just,
            is_first_variant
        );

        debug!("just_ts: {}", just_ts.to_string());
        debug!("conf_ts: {}", conf_ts.to_string());
        debug!("just_field_names: {:?}", just_field_names);
        debug!("conf_field_names: {:?}", conf_field_names);

        // skip_self_just=true => no top-level variant_justification, variant_confidence
        assert!(!just_ts.to_string().contains("variant_justification"));
        assert!(!conf_ts.to_string().contains("variant_confidence"));

        // 'x' => #[justify=false], so no x_justification/confidence
        assert!(!just_ts.to_string().contains("x_justification"));
        assert!(!conf_ts.to_string().contains("x_confidence"));

        // 'y' => no attribute, but skip_self_just overrides => means we do NOT add "y_justification"
        // because the function checks if is_justification_enabled(field), which it is,
        // but top-level skip_self_just doesn't stop field-level expansions in the code?
        // Actually, `handle_named_variant` calls `is_justification_enabled` for each field. There's
        // no skip or not skip for the field if no attribute. So 'y' is justification-enabled by default.
        // But the function code must still fill them in. Let's see:

        // The final code for handle_named_variant is: 
        //  "if skip_self_just => no top-level fields" 
        //  Then for each named field with justification enabled => we add field_justification...
        // We haven't disabled 'y'. So we DO expect "y_justification" and "y_confidence" in the token stream.
        // But let's see if skip_self_just kills them. Actually skip_self_just only stops top-level. 
        // There's no #[justify=false] for y, so y_justification is included.

        assert!(just_ts.to_string().contains("y_justification"));
        assert!(conf_ts.to_string().contains("y_confidence"));

        // Because is_first_variant=false, we do not expect them in the name vectors.
        // So the returned vectors should be empty.
        assert_eq!(just_field_names.len(), 0);
        assert_eq!(conf_field_names.len(), 0);
    }
}

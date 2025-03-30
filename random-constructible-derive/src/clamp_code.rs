crate::ix!();

/// NEW HELPER FUNCTION
/// We generate a small piece of code to clamp `val` if the field is a known
/// numeric (primitive) type and we have `min` or `max` specified.
///
/// This returns a `TokenStream2` snippet you can inject directly after
/// computing `val`.
pub fn clamp_code(ty: &Type, maybe_min: Option<f64>, maybe_max: Option<f64>) -> TokenStream2 {
    //trace!("clamp_code(...) invoked for type: {:?}", ty);

    // If the type is not primitive, or there's no min/max, emit empty code
    if !is_primitive_type(ty) || (maybe_min.is_none() && maybe_max.is_none()) {
        return quote!{};
    }

    // If we do have min and/or max, we build the needed if-checks
    let min_check = if let Some(minv) = maybe_min {
        quote! {
            if val < #minv as #ty {
                val = #minv as #ty;
            }
        }
    } else {
        quote!{}
    };

    let max_check = if let Some(maxv) = maybe_max {
        quote! {
            if val > #maxv as #ty {
                val = #maxv as #ty;
            }
        }
    } else {
        quote!{}
    };

    quote! {
        {
            #min_check
            #max_check
        }
    }
}

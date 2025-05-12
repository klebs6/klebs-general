crate::ix!();

/// This helper sanitizes a type string (e.g. "HashMap<u8, String>" => "HashMap_u8_String")
/// then **conditionally** appends "Justification" or "Confidence":
/// - If there's at least one underscore in the sanitized name, we do `Foo_Bar_Justification`.
/// - If there's **no** underscore in the sanitized name, we do `FooBarJustification`.
pub fn sanitize_into_idents_for_nested(
    the_type: &syn::Type,
    span: proc_macro2::Span
) -> (proc_macro2::TokenStream, proc_macro2::TokenStream) {
    use quote::quote;

    let raw = quote!(#the_type).to_string();

    // 1) Replace all non-alphanumeric with underscore
    let mut s = raw
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect::<String>();

    // 2) collapse repeated underscores
    while s.contains("__") {
        s = s.replace("__", "_");
    }
    // 3) trim leading/trailing underscores
    s = s.trim_matches('_').to_string();

    // 4) if empty or starts with digit, prefix something
    if s.is_empty() {
        s = "NestedType".to_string();
    } else if s.chars().next().unwrap().is_ascii_digit() {
        s = format!("T{}", s);
    }

    // Decide how to append "Justification"/"Confidence"
    // - If s has at least one underscore, we do "s_Justification", else "sJustification"
    let has_underscore = s.contains('_');
    let justification_name = if has_underscore {
        format!("{}_Justification", s)
    } else {
        format!("{}Justification", s)
    };
    let confidence_name = if has_underscore {
        format!("{}_Confidence", s)
    } else {
        format!("{}Confidence", s)
    };

    let just_ident = syn::Ident::new(&justification_name, span);
    let conf_ident = syn::Ident::new(&confidence_name, span);

    (quote!(#just_ident), quote!(#conf_ident))
}

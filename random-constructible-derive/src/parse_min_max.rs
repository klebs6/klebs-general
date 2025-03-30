crate::ix!();

/// NEW FUNCTION:
/// Parses `#[rand_construct(min=... , max=...)]` attributes, returning `(Option<f64>, Option<f64>)`.
/// If none are found, returns `(None, None)`.
/// 
/// We do a 'best effort' parse to allow integer or float for `min`/`max`.
pub fn parse_min_max(attrs: &[Attribute]) -> (Option<f64>, Option<f64>) {
    let mut maybe_min = None;
    let mut maybe_max = None;

    for attr in attrs {
        if attr.path.is_ident("rand_construct") {
            trace!("Found rand_construct attribute, attempting to parse min/max");
            if let Ok(Meta::List(meta_list)) = attr.parse_meta() {
                for nested in meta_list.nested.iter() {
                    if let NestedMeta::Meta(Meta::NameValue(ref name_value)) = nested {
                        let ident_str = name_value.path.get_ident().map(|id| id.to_string());
                        if let Some(ident_str) = ident_str {
                            match ident_str.as_str() {
                                "min" => {
                                    trace!("Detected min= attribute");
                                    maybe_min = match &name_value.lit {
                                        Lit::Float(f) => f.base10_parse::<f64>().ok(),
                                        Lit::Int(i) => i.base10_parse::<f64>().ok(),
                                        _ => {
                                            warn!("Unable to parse literal for min=; ignoring");
                                            None
                                        }
                                    };
                                },
                                "max" => {
                                    trace!("Detected max= attribute");
                                    maybe_max = match &name_value.lit {
                                        Lit::Float(f) => f.base10_parse::<f64>().ok(),
                                        Lit::Int(i) => i.base10_parse::<f64>().ok(),
                                        _ => {
                                            warn!("Unable to parse literal for max=; ignoring");
                                            None
                                        }
                                    };
                                },
                                _ => {
                                    debug!("Ignoring unrelated key in rand_construct attribute: {}", ident_str);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    trace!("parse_min_max => min={:?}, max={:?}", maybe_min, maybe_max);
    (maybe_min, maybe_max)
}

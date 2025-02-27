// ---------------- [ File: hydro2-network-wire-derive/src/parse_strings.rs ]
crate::ix!();

/// Helper: parse a struct definition and extract its `Generics`.
#[cfg(test)]
pub fn parse_generics(src: &str) -> Result<Generics, SynError> {
    info!("parse_generics: attempting to parse '{}'", src);
    let di: DeriveInput = parse_str(src)?;
    info!("  parse_generics succeeded => {:?}", di.generics);
    Ok(di.generics)
}

/// Helper: parse a `Path` from a string
#[cfg(test)]
pub fn parse_path(src: &str) -> Result<Path, SynError> {
    info!("parse_path: attempting to parse '{}'", src);
    let p = parse_str::<Path>(src)?;
    info!("  parse_path succeeded => {}", quote::ToTokens::to_token_stream(&p));
    Ok(p)
}

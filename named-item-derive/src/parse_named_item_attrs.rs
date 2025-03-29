// ---------------- [ File: src/parse_named_item_attrs.rs ]
crate::ix!();

pub fn parse_named_item_attrs(ast: &DeriveInput) -> syn::Result<NamedItemConfig> {

    let mut default_name    = None;
    let mut aliases         = false;
    let mut default_aliases = Vec::new();
    let mut history         = false;

    for attr in &ast.attrs {
        if attr.path().is_ident("named_item") {
            // parse_nested_meta helps parse name="value" pairs
            attr.parse_nested_meta(|meta| {
                let p = &meta.path;
                if p.is_ident("default_name") {
                    let lit: LitStr = meta.value()?.parse()?;
                    default_name = Some(lit.value());
                } else if p.is_ident("aliases") {
                    let lit: LitStr = meta.value()?.parse()?;
                    aliases = lit.value().to_lowercase() == "true";
                } else if p.is_ident("default_aliases") {
                    let lit: LitStr = meta.value()?.parse()?;
                    default_aliases = lit
                        .value()
                        .split(',')
                        .filter(|tok| !tok.trim().is_empty())
                        .map(|s| s.trim().to_string())
                        .collect();
                } else if p.is_ident("history") {
                    let lit: LitStr = meta.value()?.parse()?;
                    history = lit.value().to_lowercase() == "true";
                }
                Ok(())
            })?;
        }
    }

    Ok(NamedItemConfigBuilder::default()
        .default_name(default_name)
        .aliases(aliases)
        .default_aliases(default_aliases)
        .history(history)
        .build()
        .unwrap()
    )
}

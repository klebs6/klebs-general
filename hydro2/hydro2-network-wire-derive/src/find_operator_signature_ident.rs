// ---------------- [ File: src/find_operator_signature_ident.rs ]
crate::ix!();

/// Attempt to find an operator signature ident (e.g. "FooOpOperatorSignature") in `sig_map`.
/// If not found, produce a fallback ident like "__FooOp_SignatureNotFound".
pub fn find_operator_signature_ident(
    sig_map: &HashMap<String, Ident>,
    op_ident_str: &str,
    op_path: &Path,
) -> Ident {
    info!(
        "[find_operator_signature_ident] Called with:\n  op_ident_str = '{}'\n  op_path      = {:?}",
        op_ident_str, op_path
    );
    info!(
        "[find_operator_signature_ident] sig_map has {} entries.",
        sig_map.len()
    );

    match sig_map.get(op_ident_str) {
        Some(id) => {
            info!(
                "[find_operator_signature_ident] Found matching signature ident '{}' in sig_map.",
                id
            );
            id.clone()
        }
        None => {
            let fallback_name = format!("__{}SignatureNotFound", op_ident_str);
            info!(
                "[find_operator_signature_ident] No signature ident found. Fallback is '{}'.",
                fallback_name
            );
            Ident::new(&fallback_name, op_path.span())
        }
    }
}

#[cfg(test)]
mod test_find_operator_signature_ident {
    use super::*;
    use syn::parse_str;

    /// Helper to construct a HashMap<String, Ident> from a list of (key, value) pairs
    fn make_sig_map(pairs: &[(&str, &str)]) -> HashMap<String, Ident> {
        let mut map = HashMap::new();
        for (k, v) in pairs {
            let ident = Ident::new(v, proc_macro2::Span::call_site());
            info!(
                "[make_sig_map] Inserting key='{}' with ident='{}'",
                k, v
            );
            map.insert((*k).to_owned(), ident);
        }
        map
    }

    #[test]
    fn test_found_operator_signature() -> Result<(), syn::Error> {
        info!("[test_found_operator_signature] Starting test...");

        // The map has "FooOp" => "FooOpOperatorSignature".
        let sig_map = make_sig_map(&[("FooOp", "FooOpOperatorSignature")]);

        // We'll parse a path "my_crate::operators::FooOp".
        let path: Path = parse_str("my_crate::operators::FooOp")?;
        info!(
            "[test_found_operator_signature] Using op_ident_str='FooOp' and path={:?}",
            path
        );

        let found = find_operator_signature_ident(&sig_map, "FooOp", &path);
        info!(
            "[test_found_operator_signature] Resulting Ident='{}'",
            found
        );
        assert_eq!(found.to_string(), "FooOpOperatorSignature");

        // Compare debug representation of spans if you need any confirmation:
        let found_span_dbg = format!("{:?}", found.span());
        let path_span_dbg = format!("{:?}", path.span());
        info!(
            "[test_found_operator_signature] found_span_dbg='{}' vs path_span_dbg='{}'",
            found_span_dbg, path_span_dbg
        );
        assert_eq!(found_span_dbg, path_span_dbg);

        Ok(())
    }

    #[test]
    fn test_missing_operator_signature() -> Result<(), syn::Error> {
        info!("[test_missing_operator_signature] Starting test...");

        // The map does NOT contain "BarOp", so it should produce a fallback ident.
        let sig_map = make_sig_map(&[("FooOp", "FooOpOperatorSignature")]);
        let path: Path = parse_str("BarOp")?;
        info!(
            "[test_missing_operator_signature] Using op_ident_str='BarOp' and path={:?}",
            path
        );

        let found = find_operator_signature_ident(&sig_map, "BarOp", &path);
        info!(
            "[test_missing_operator_signature] Resulting Ident='{}'",
            found
        );
        assert_eq!(found.to_string(), "__BarOpSignatureNotFound");

        let found_span_dbg = format!("{:?}", found.span());
        let path_span_dbg = format!("{:?}", path.span());
        info!(
            "[test_missing_operator_signature] found_span_dbg='{}' vs path_span_dbg='{}'",
            found_span_dbg, path_span_dbg
        );
        assert_eq!(found_span_dbg, path_span_dbg);

        Ok(())
    }

    #[test]
    fn test_multiple_segments_path_missing_signature() -> Result<(), syn::Error> {
        info!("[test_multiple_segments_path_missing_signature] Starting test...");

        let sig_map = HashMap::<String, Ident>::new();
        let path: Path = parse_str("some::nested::module::BazOp")?;
        info!(
            "[test_multiple_segments_path_missing_signature] Using op_ident_str='BazOp' and path={:?}",
            path
        );

        let found = find_operator_signature_ident(&sig_map, "BazOp", &path);
        info!(
            "[test_multiple_segments_path_missing_signature] Resulting Ident='{}'",
            found
        );
        assert_eq!(found.to_string(), "__BazOpSignatureNotFound");

        let found_span_dbg = format!("{:?}", found.span());
        let path_span_dbg = format!("{:?}", path.span());
        info!(
            "[test_multiple_segments_path_missing_signature] found_span_dbg='{}' vs path_span_dbg='{}'",
            found_span_dbg, path_span_dbg
        );
        assert_eq!(found_span_dbg, path_span_dbg);

        Ok(())
    }

    #[test]
    fn test_multiple_segments_path_existing_signature() -> Result<(), syn::Error> {
        info!("[test_multiple_segments_path_existing_signature] Starting test...");

        let sig_map = make_sig_map(&[("BazOp", "BazOpOperatorSignature")]);
        let path: Path = parse_str("some::nested::module::BazOp")?;
        info!(
            "[test_multiple_segments_path_existing_signature] Using op_ident_str='BazOp' and path={:?}",
            path
        );

        let found = find_operator_signature_ident(&sig_map, "BazOp", &path);
        info!(
            "[test_multiple_segments_path_existing_signature] Resulting Ident='{}'",
            found
        );
        assert_eq!(found.to_string(), "BazOpOperatorSignature");

        let found_span_dbg = format!("{:?}", found.span());
        let path_span_dbg = format!("{:?}", path.span());
        info!(
            "[test_multiple_segments_path_existing_signature] found_span_dbg='{}' vs path_span_dbg='{}'",
            found_span_dbg, path_span_dbg
        );
        assert_eq!(found_span_dbg, path_span_dbg);

        Ok(())
    }

    #[test]
    fn test_operator_name_collision_map() -> Result<(), syn::Error> {
        info!("[test_operator_name_collision_map] Starting test...");

        // Multiple insertions for "FooOp"; last insertion wins in HashMap.
        let mut sig_map = HashMap::new();
        info!("[test_operator_name_collision_map] Insert 'FooOpOperatorSigV1'");
        sig_map.insert("FooOp".to_owned(), Ident::new("FooOpOperatorSigV1", proc_macro2::Span::call_site()));
        info!("[test_operator_name_collision_map] Insert 'FooOpOperatorSigV2'");
        sig_map.insert("FooOp".to_owned(), Ident::new("FooOpOperatorSigV2", proc_macro2::Span::call_site()));

        let path: Path = parse_str("FooOp")?;
        info!(
            "[test_operator_name_collision_map] Using op_ident_str='FooOp' and path={:?}",
            path
        );

        let found = find_operator_signature_ident(&sig_map, "FooOp", &path);
        info!(
            "[test_operator_name_collision_map] Resulting Ident='{}'",
            found
        );
        // The second insertion overwrote the first, so expect "FooOpOperatorSigV2".
        assert_eq!(found.to_string(), "FooOpOperatorSigV2");

        let found_span_dbg = format!("{:?}", found.span());
        let path_span_dbg = format!("{:?}", path.span());
        info!(
            "[test_operator_name_collision_map] found_span_dbg='{}' vs path_span_dbg='{}'",
            found_span_dbg, path_span_dbg
        );
        assert_eq!(found_span_dbg, path_span_dbg);

        Ok(())
    }

    #[test]
    fn test_empty_op_ident_str() -> Result<(), syn::Error> {
        info!("[test_empty_op_ident_str] Starting test...");

        let sig_map = HashMap::new();
        let path: Path = parse_str("std::collections::HashMap")?;
        info!(
            "[test_empty_op_ident_str] Using op_ident_str='' and path={:?}",
            path
        );

        let found = find_operator_signature_ident(&sig_map, "", &path);
        info!(
            "[test_empty_op_ident_str] Resulting Ident='{}'",
            found
        );
        assert_eq!(found.to_string(), "__SignatureNotFound");

        let found_span_dbg = format!("{:?}", found.span());
        let path_span_dbg = format!("{:?}", path.span());
        info!(
            "[test_empty_op_ident_str] found_span_dbg='{}' vs path_span_dbg='{}'",
            found_span_dbg, path_span_dbg
        );
        assert_eq!(found_span_dbg, path_span_dbg);

        Ok(())
    }

    #[test]
    fn test_find_operator_signature_ident_found() {
        info!("test_find_operator_signature_ident_found: START");
        let mut map = HashMap::new();
        map.insert("AddOp".to_string(), syn::Ident::new("AddOpOperatorSignature", proc_macro2::Span::call_site()));

        let path: syn::Path = parse_quote!(AddOp);
        info!("  path = {}", path.to_token_stream());

        let ident = find_operator_signature_ident(&map, "AddOp", &path);
        info!("  result ident = {}", ident);
        assert_eq!(ident.to_string(), "AddOpOperatorSignature");
    }

    #[test]
    fn test_find_operator_signature_ident_not_found() {
        info!("test_find_operator_signature_ident_not_found: START");
        let map = HashMap::new();
        let path: syn::Path = parse_quote!(FooBar);
        info!("  path = {}", path.to_token_stream());

        let ident = find_operator_signature_ident(&map, "FooBar", &path);
        info!("  result ident = {}", ident);
        assert_eq!(ident.to_string(), "__FooBarSignatureNotFound");
    }
}

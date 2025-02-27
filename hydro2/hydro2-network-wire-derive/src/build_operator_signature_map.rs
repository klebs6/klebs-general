// ---------------- [ File: hydro2-network-wire-derive/src/build_operator_signature_map.rs ]
crate::ix!();

/// Build a map from operator name => signature ident,
/// e.g. "AddOp" => "AddOpOperatorSignature"
pub fn build_operator_signature_map(
    op_items: &[OperatorSpecItem]
) -> HashMap<String, Ident> {
    info!("[build_operator_signature_map] START");
    info!("  op_items.len() = {}", op_items.len());

    let mut map = HashMap::new();
    for (i, op) in op_items.iter().enumerate() {
        info!("[build_operator_signature_map] processing item[{}]: {:?}", i, op);

        if let Some(last_seg) = op.path().segments.last() {
            let key_str = last_seg.ident.to_string();
            let sig_str = format!("{}OperatorSignature", last_seg.ident);
            info!("[build_operator_signature_map] last_seg = {}", key_str);

            let sig_id = Ident::new(&sig_str, last_seg.ident.span());
            info!("[build_operator_signature_map] inserting map entry => {} -> {}",
                      key_str, sig_id);

            map.insert(key_str, sig_id);
        } else {
            info!("[build_operator_signature_map] no segments for item[{}], skipping insert", i);
        }
    }

    info!("[build_operator_signature_map] FINISH, map.len() = {}", map.len());
    map
}

#[cfg(test)]
mod test_build_operator_signature_map {
    use super::*;
    use std::collections::HashMap;
    use syn::{parse_quote, Path};

    #[test]
    fn test_single_segment() {
        info!("test_single_segment: START");
        let op_items = vec![
            OperatorSpecItem::new(parse_quote!(AddOp)),
            OperatorSpecItem::new(parse_quote!(SubOp)),
        ];
        info!("test_single_segment: calling build_operator_signature_map(...)");
        let map = build_operator_signature_map(&op_items);

        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get("AddOp").map(|id| id.to_string()),
            Some("AddOpOperatorSignature".to_string())
        );
        assert_eq!(
            map.get("SubOp").map(|id| id.to_string()),
            Some("SubOpOperatorSignature".to_string())
        );
    }

    #[test]
    fn test_multi_segment_paths() {
        info!("test_multi_segment_paths: START");
        let op_items = vec![
            OperatorSpecItem::new(parse_quote!(foo::bar::AddOp)),
            OperatorSpecItem::new(parse_quote!(something::nested::MultiplyOp)),
        ];
        info!("test_multi_segment_paths: calling build_operator_signature_map(...)");
        let map = build_operator_signature_map(&op_items);

        // The key is the *last* path segment => "AddOp" -> "AddOpOperatorSignature"
        // and "MultiplyOp" -> "MultiplyOpOperatorSignature"
        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get("AddOp").map(|id| id.to_string()),
            Some("AddOpOperatorSignature".to_string())
        );
        assert_eq!(
            map.get("MultiplyOp").map(|id| id.to_string()),
            Some("MultiplyOpOperatorSignature".to_string())
        );
    }

    #[test]
    fn test_generics_ignored_in_key() {
        info!("test_generics_ignored_in_key: START");
        // The function uses only the final segment ident, ignoring generics
        let op_items = vec![
            OperatorSpecItem::new(parse_quote!(AddOp<T>)),
            OperatorSpecItem::new(parse_quote!(BazOp<A, B>)),
        ];
        info!("test_generics_ignored_in_key: calling build_operator_signature_map(...)");
        let map = build_operator_signature_map(&op_items);

        assert_eq!(map.len(), 2);
        assert_eq!(
            map.get("AddOp").map(|id| id.to_string()),
            Some("AddOpOperatorSignature".to_string())
        );
        assert_eq!(
            map.get("BazOp").map(|id| id.to_string()),
            Some("BazOpOperatorSignature".to_string())
        );
    }

    #[test]
    fn test_duplicate_operator_names() {
        info!("test_duplicate_operator_names: START");
        // If two items share the same final segment, the second overwrites
        let op_items = vec![
            OperatorSpecItem::new(parse_quote!(AddOp)),
            OperatorSpecItem::new(parse_quote!(foo::bar::AddOp)),
        ];
        info!("test_duplicate_operator_names: calling build_operator_signature_map(...)");
        let map = build_operator_signature_map(&op_items);

        // The map will end with only 1 entry for "AddOp"
        assert_eq!(map.len(), 1);
        assert_eq!(
            map.get("AddOp").map(|id| id.to_string()),
            Some("AddOpOperatorSignature".to_string())
        );
    }

    #[test]
    fn test_empty_path_no_insert() {
        info!("test_empty_path_no_insert: START");
        use syn::{
            punctuated::Punctuated,
            Path,
            PathSegment,
            Token,
        };

        // Construct a Path with no segments
        let empty_path = Path {
            leading_colon: None,
            segments: Punctuated::<PathSegment, Token![::]>::new(),
        };
        let empty_path_item = OperatorSpecItem::new(empty_path);

        let op_items = vec![empty_path_item];
        info!("test_empty_path_no_insert: calling build_operator_signature_map(...)");
        let map = build_operator_signature_map(&op_items);
        assert_eq!(map.len(), 0, "No insert occurs for empty path");
    }
}

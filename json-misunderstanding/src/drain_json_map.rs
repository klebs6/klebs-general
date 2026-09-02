// ---------------- [ File: json-misunderstanding/src/drain_json_map.rs ]
crate::ix!();

pub fn drain_json_map(map: &mut serde_json::Map<String, Value>) -> Vec<(String, Value)> {
    trace!("Draining serde_json::Map with {} entries", map.len());
    let old_map = std::mem::take(map);
    old_map.into_iter().collect()
}

#[cfg(test)]
mod drain_json_map_tests {
    use super::*;

    #[traced_test]
    fn test_drain_json_map() {
        let mut json_map = serde_json::Map::new();
        json_map.insert("key1".to_string(), json!(1));
        json_map.insert("key2".to_string(), json!("value"));
        json_map.insert("key3".to_string(), json!([1, 2, 3]));

        assert_eq!(json_map.len(), 3);

        let drained_items = drain_json_map(&mut json_map);
        info!("Drained serde_json::Map items: {:?}", drained_items);

        assert!(json_map.is_empty());
        assert_eq!(drained_items.len(), 3);

        assert!(drained_items.contains(&("key1".to_string(), json!(1))));
        assert!(drained_items.contains(&("key2".to_string(), json!("value"))));
        assert!(drained_items.contains(&("key3".to_string(), json!([1, 2, 3]))));
    }
}

crate::ix!();

// A helper function that recursively fixes any node as needed
pub fn fix_value(
    val: serde_json::Value,
    config: &MisunderstandingCorrectionConfig,
) -> serde_json::Value {
    match val {
        serde_json::Value::Object(map) => fix_object(map, config),
        serde_json::Value::Array(arr) => fix_array(arr, config),
        other => fix_primitive(other, config),
    }
}

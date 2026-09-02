// ---------------- [ File: ai-json-template/src/ai_json_template_for_string.rs ]
crate::ix!();

impl AiJsonTemplate for String {

    fn to_template() -> JsonValue {
        trace!("AiJsonTemplate::to_template for Vec<String>");

        // We add disclaimers about how to produce an array of strings:
        //   - Must be a JSON array, e.g. ["A", "B", "C"]
        //   - No extra keys or objects
        //   - Provide real string elements, not null or numeric
        let mut root = serde_json::Map::new();
        root.insert("type".to_string(), JsonValue::String("string".to_string()));
        JsonValue::Object(root)
    }
}

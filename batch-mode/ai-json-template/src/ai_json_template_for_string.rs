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
        root.insert(
            "generation_instructions".to_string(),
            JsonValue::String(
                "Provide this field as a single literal string. If optional, you may set it to null."
                .to_string()
            )
        );

        JsonValue::Object(root)
    }
}

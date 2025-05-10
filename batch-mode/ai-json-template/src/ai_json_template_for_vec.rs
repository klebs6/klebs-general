// ---------------- [ File: ai-json-template/src/ai_json_template_for_vec.rs ]
crate::ix!();

/// Blanket impl for `Vec<T>` if `T: AiJsonTemplate`.
///
/// This treats the field as `"array_of"` in the JSON schema, referencing
/// `T::to_template()` for the item template. We add disclaimers saying we
/// expect a top-level JSON array, not item-level justification.
impl<T> AiJsonTemplate for Vec<T>
where
    T: Send + Sync + AiJsonTemplate + 'static,
{
    fn to_template() -> JsonValue {
        trace!(
            "AiJsonTemplate::to_template for Vec<{}>",
            type_name::<T>()
        );

        let mut obj = serde_json::Map::new();
        obj.insert(
            "type".to_string(),
            JsonValue::String("array_of".to_string())
        );

        obj.insert(
            "generation_instructions".to_string(),
            JsonValue::String(format!("Provide a JSON array of items, each conforming to {}.", type_name::<T>()))
        );

        // Typically marked required, but the parent can override or interpret it
        obj.insert("required".to_string(), JsonValue::Bool(true));

        // The item template comes from T::to_template
        let item_schema = T::to_template();
        obj.insert("item_template".to_string(), item_schema);

        JsonValue::Object(obj)
    }
}

/// Blanket impl for `Vec<T>` if `T: AiJsonTemplateWithJustification`.
///
/// We add `"has_justification": true` at the array level, but again, we do
/// not require or encourage justifying each element individually. Instead,
/// we have a single top-level justification/confidence for the entire vector.
impl<T> AiJsonTemplateWithJustification for Vec<T>
where
    T: Send + Sync + AiJsonTemplateWithJustification + 'static,
{
    fn to_template_with_justification() -> JsonValue {
        trace!(
            "AiJsonTemplateWithJustification::to_template_with_justification for Vec<{}>",
            type_name::<T>()
        );

        let mut obj = serde_json::Map::new();

        obj.insert(
            "type".to_string(),
            JsonValue::String("array_of".to_string())
        );

        obj.insert("required".to_string(), JsonValue::Bool(true));

        // Indicate that top-level justification might exist for this entire array
        obj.insert("has_justification".to_string(), JsonValue::Bool(true));

        obj.insert(
            "generation_instructions".to_string(),
            JsonValue::String(format!("Provide a JSON array of items, each conforming to {}. We do not want you to justify each individual element.", type_name::<T>()))
        );

        // The item schema for nested types
        let item_schema = T::to_template_with_justification();
        obj.insert("item_template".to_string(), item_schema);

        JsonValue::Object(obj)
    }
}

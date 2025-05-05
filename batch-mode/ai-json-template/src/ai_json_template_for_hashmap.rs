crate::ix!();

/// Provides a blanket AiJsonTemplate implementation for HashMap<K, V>,
/// so long as K and V each implement AiJsonTemplate (and K also implements Eq + Hash).
impl<K, V> AiJsonTemplate for HashMap<K, V>
where
    K: Send + Sync + AiJsonTemplate + Eq + Hash + 'static,
    V: Send + Sync + AiJsonTemplate + 'static,
{
    fn to_template() -> JsonValue {
        // We'll produce a basic "map_of" template with disclaimers
        tracing::trace!("AiJsonTemplate::to_template for HashMap<K, V>");

        let mut obj = serde_json::Map::new();
        obj.insert("type".to_string(), JsonValue::String("map_of".to_string()));

        // We'll disclaim that the keys must be a single JSON object key
        obj.insert(
            "generation_instructions".to_string(),
            JsonValue::String(
                "IMPORTANT:\n\
                 Provide this field as a JSON object {\"some_key\": <value>, ...}.\n\
                 The keys must be valid JSON strings. For the map value, fill them per the V schema.\n\
                 Do not add extra fields or wrap in arrays.\n\
                 If optional, set it to null.\n"
                    .to_string(),
            ),
        );
        // Typically required can be set at the parent struct’s logic, so default to true here:
        obj.insert("required".to_string(), JsonValue::Bool(true));

        // For the subtemplates: key => K, val => V
        // But 'K' is also AiJsonTemplate, though in your final schema, you typically just handle "string" keys.
        // We’ll just store them as "map_key_template" and "map_value_template".
        let key_schema = K::to_template();
        let val_schema = V::to_template();
        obj.insert("map_key_template".to_string(), key_schema);
        obj.insert("map_value_template".to_string(), val_schema);

        JsonValue::Object(obj)
    }
}

/// Similarly, we provide AiJsonTemplateWithJustification for HashMap<K, V>,
/// requiring that K and V each implement AiJsonTemplateWithJustification.
impl<K, V> AiJsonTemplateWithJustification for HashMap<K, V>
where
    K: Send + Sync + AiJsonTemplate + Eq + Hash + 'static,
    V: Send + Sync + AiJsonTemplate + 'static,
{
    fn to_template_with_justification() -> JsonValue {
        tracing::trace!("AiJsonTemplateWithJustification::to_template_with_justification for HashMap<K, V>");

        // We'll produce "map_of" plus disclaimers for justification
        let mut obj = serde_json::Map::new();
        obj.insert("type".to_string(), JsonValue::String("map_of".to_string()));
        obj.insert("required".to_string(), JsonValue::Bool(true));
        obj.insert("has_justification".to_string(), JsonValue::Bool(true));

        let disclaimers = "\
            IMPORTANT:\n\
            Provide this map as {\"key\": <value>}, with each key a JSON string.\n\
            If optional, set to null. Otherwise, fill all subfields of the map value.\n\
            For justification:\n\
            - You may have a justification/confidence for the entire map.\n\
            - Each key and value might also be subject to justification if the parent struct so requires.\n";

        obj.insert("generation_instructions".to_string(), JsonValue::String(disclaimers.to_string()));

        // subtemplates from K::to_template_with_justification() and V::to_template_with_justification()
        let nested_key = K::to_template();
        let nested_val = V::to_template();
        obj.insert("map_key_template".to_string(), nested_key);
        obj.insert("map_value_template".to_string(), nested_val);

        JsonValue::Object(obj)
    }
}

/// Exhaustive test suite verifying we can handle a HashMap<K, V> for AiJsonTemplate
/// and AiJsonTemplateWithJustification. We show a K=String, V=Vec<AnnotatedLeaf>-like scenario.
#[cfg(test)]
mod test_hashmap_k_v {
    use super::*;

    /// This struct simulates a nested type, e.g. AnnotatedLeaf. We'll just do a minimal example
    /// that implements AiJsonTemplate.
    #[derive(SaveLoad,Serialize,Deserialize,Debug, Clone)]
    struct FakeLeaf {
        label: String,
    }

    impl AiJsonTemplate for FakeLeaf {
        fn to_template() -> JsonValue {
            let mut obj = serde_json::Map::new();
            obj.insert("type".to_string(), JsonValue::String("struct".to_string()));
            obj.insert("struct_name".to_string(), JsonValue::String("FakeLeaf".into()));
            let mut fields = serde_json::Map::new();
            fields.insert("label".to_string(), {
                let mut label_obj = serde_json::Map::new();
                label_obj.insert("type".to_string(), JsonValue::String("string".into()));
                label_obj.insert("required".to_string(), JsonValue::Bool(true));
                label_obj.insert("generation_instructions".to_string(), JsonValue::String("A label string.".into()));
                JsonValue::Object(label_obj)
            });
            obj.insert("fields".to_string(), JsonValue::Object(fields));
            JsonValue::Object(obj)
        }
    }

    impl AiJsonTemplateWithJustification for FakeLeaf {
        fn to_template_with_justification() -> JsonValue {
            let mut root = Self::to_template();
            if let Some(obj) = root.as_object_mut() {
                obj.insert("has_justification".to_string(), JsonValue::Bool(true));
            }
            root
        }
    }

    #[traced_test]
    fn test_hashmap_string_fakeleaf_schema() {
        type MyHash = HashMap<String, FakeLeaf>;
        let schema = <MyHash as AiJsonTemplate>::to_template();
        assert!(schema.is_object(), "Should produce an object schema");

        let schema_obj = schema.as_object().unwrap();
        assert_eq!(schema_obj.get("type").unwrap(), "map_of");
        assert!(schema_obj.contains_key("map_key_template"), "key template expected");
        assert!(schema_obj.contains_key("map_value_template"), "value template expected");
    }

    #[traced_test]
    fn test_hashmap_string_vec_fakeleaf_schema() {
        // If V=Vec<FakeLeaf>, then we have a map from string => an array of FakeLeaf
        type MyHash = HashMap<String, Vec<FakeLeaf>>;
        let schema = <MyHash as AiJsonTemplate>::to_template();

        assert!(schema.is_object(), "Should produce an object schema for map_of");
        let so = schema.as_object().unwrap();
        assert_eq!(so.get("type").unwrap(), "map_of");
        let val_template = so.get("map_value_template").expect("no map_value_template?");
        assert!(val_template.is_object(), "value template must be an object");
        let val_obj = val_template.as_object().unwrap();
        // Should have "type":"array_of", "item_template": <some nested struct stuff>
        assert!(
            val_obj.get("type").unwrap().as_str().unwrap().contains("array_of"),
            "Expected an array_of type"
        );
    }

    #[traced_test]
    fn test_hashmap_justification() {
        type MyHash = HashMap<String, FakeLeaf>;
        let jschema = <MyHash as AiJsonTemplateWithJustification>::to_template_with_justification();
        assert!(jschema.is_object());
        let m = jschema.as_object().unwrap();
        assert_eq!(m.get("type").unwrap(), "map_of");
        assert_eq!(m.get("has_justification").unwrap(), &JsonValue::Bool(true));
        let kv_template = m.get("map_key_template").unwrap();
        let vv_template = m.get("map_value_template").unwrap();
        assert!(kv_template.is_object(), "K justification must be object for the schema");
        assert!(vv_template.is_object(), "V justification must be object for the schema");
    }
}

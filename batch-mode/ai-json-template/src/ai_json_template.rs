// ---------------- [ File: ai-json-template/src/ai_json_template.rs ]
crate::ix!();

/// The derived code implements `AiJsonTemplate` for each struct, letting you
/// call `MyStruct::to_template()` to get a JSON “schema” describing how the
/// AI should produce data that matches this layout.
///
pub trait AiJsonTemplate
: Clone 
+ Debug 
+ LoadFromFile 
+ SaveToFile<Error=SaveLoadError> 
+ Serialize 
+ for<'a> Deserialize<'a> 
{
    /// Return a JSON template describing how the AI’s output should be structured.
    /// This might include doc comments or other instructions for each field.
    fn to_template() -> serde_json::Value;
}

/// Extended trait that demands both a normal `AiJsonTemplate` and
/// a `...Justification` structure for *nested* usage.
pub trait AiJsonTemplateWithJustification: AiJsonTemplate {
    fn to_template_with_justification() -> serde_json::Value;
}

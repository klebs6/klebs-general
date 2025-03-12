
# ai-json-template

contains the following code:

```rust
/// The derived code implements `AiJsonTemplate` for each struct, letting you
/// call `MyStruct::to_template()` to get a JSON “schema” describing how the
/// AI should produce data that matches this layout.
///
pub trait AiJsonTemplate: serde::Serialize + for<'a> serde::Deserialize<'a> {
    /// Return a JSON template describing how the AI’s output should be structured.
    /// This might include doc comments or other instructions for each field.
    fn to_template() -> serde_json::Value;
}
```

We use it during AI assisted struct generation. We want to have the AI
intelligently generate values for an arbitrary struct's fields. With this in
mind, we need to provide it with a template or a sort of "schema" which contains
the information about what it needs to generate and how it should do it.

We use this trate to enable this functionality.

See `ai-json-template-derive` for the full usage and utility of this
functionality.

Happy programming!

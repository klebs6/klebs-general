crate::ix!();

// #11 Key-name Misalignment
pub fn fix_key_name_misalignment(val: Value) -> Value {
    if let Value::Object(mut obj) = val {
        if let Some(description_val) = obj.remove("descriptor") {
            tracing::debug!("Renaming 'descriptor' to 'description'");
            obj.insert("description".to_owned(), description_val);
        }
        return Value::Object(obj);
    }
    val
}

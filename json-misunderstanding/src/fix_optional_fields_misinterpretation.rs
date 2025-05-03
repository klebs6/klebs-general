crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn fix_optional_fields_misinterpretation(val: Value) -> Value {
    trace!("Starting fix_optional_fields_misinterpretation");
    match &val {
        Value::Object(obj) => {
            if let Some(num) = obj.get("aggregator_depth_limit") {
                if num.is_number() {
                    debug!("Reverting aggregator_depth_limit to null (suspected optional field).");
                    let mut new_obj = obj.clone();
                    new_obj.insert("aggregator_depth_limit".to_string(), Value::Null);
                    return Value::Object(new_obj);
                }
            }
        }
        _ => {}
    }
    val
}

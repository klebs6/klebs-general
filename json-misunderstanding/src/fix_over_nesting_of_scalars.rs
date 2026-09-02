crate::ix!();

#[tracing::instrument(level="trace", skip_all)]
pub fn fix_over_nesting_of_scalars(val: Value) -> Value {
    trace!("Starting fix_over_nesting_of_scalars");
    match val {
        Value::Object(mut obj) if obj.len() == 1 && obj.contains_key("value") => {
            debug!("Flattening scalar nesting: replacing {{\"value\":..}} with raw scalar.");
            obj.remove("value").unwrap_or(Value::Null)
        }
        other => other,
    }
}

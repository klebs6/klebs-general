crate::ix!();

// #4 Vector as Map of Indices
pub fn fix_vector_as_map_of_indices(val: Value) -> Value {
    if let Value::Object(obj) = &val {
        // If all keys are numeric indices, we might convert to an array
        let mut pairs: Vec<(usize, Value)> = vec![];
        for (k, v) in obj.iter() {
            if let Ok(idx) = k.parse::<usize>() {
                pairs.push((idx, v.clone()));
            } else {
                // not all numeric keys, bail
                return val;
            }
        }
        // Sort by index to keep consistent order
        pairs.sort_by_key(|(idx, _)| *idx);
        let arr = pairs.into_iter().map(|(_, v)| v).collect();
        tracing::debug!("Converted map-of-indices to array");
        return Value::Array(arr);
    }
    val
}

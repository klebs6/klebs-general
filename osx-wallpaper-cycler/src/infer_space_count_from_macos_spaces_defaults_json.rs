// ---------------- [ File: osx-wallpaper-cycler/src/infer_space_count_from_macos_spaces_defaults_json.rs ]
crate::ix!();

pub fn infer_space_count_from_macos_spaces_defaults_json(root: &serde_json::Value) -> Option<usize> {
    let sdc = root.get("SpacesDisplayConfiguration")?;
    let mgmt = sdc.get("Management Data")?;
    let monitors = mgmt.get("Monitors")?.as_array()?;

    let mut unique_space_ids: std::collections::HashSet<u64> = std::collections::HashSet::new();
    let mut fallback_sum: usize = 0;

    for monitor in monitors.iter() {
        let Some(spaces) = monitor.get("Spaces").and_then(|v| v.as_array()) else {
            continue;
        };

        fallback_sum = fallback_sum.saturating_add(spaces.len());

        for space in spaces.iter() {
            if let Some(id) = space.get("ManagedSpaceID").and_then(|v| v.as_u64()) {
                unique_space_ids.insert(id);
            }
        }
    }

    if !unique_space_ids.is_empty() {
        return Some(unique_space_ids.len());
    }

    if fallback_sum > 0 {
        return Some(fallback_sum);
    }

    None
}

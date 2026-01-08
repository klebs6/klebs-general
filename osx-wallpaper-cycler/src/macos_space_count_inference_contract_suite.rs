// ---------------- [ File: osx-wallpaper-cycler/src/macos_space_count_inference_contract_suite.rs ]
crate::ix!();

#[cfg(test)]
mod macos_space_count_inference_contract_suite {
    use super::*;

    #[traced_test]
    fn infers_unique_space_count_across_multiple_monitors_by_managed_space_id() {
        let v = serde_json::json!({
            "SpacesDisplayConfiguration": {
                "Management Data": {
                    "Monitors": [
                        {
                            "Display Identifier": "A",
                            "Spaces": [
                                { "ManagedSpaceID": 101, "Type": 0 },
                                { "ManagedSpaceID": 102, "Type": 0 }
                            ]
                        },
                        {
                            "Display Identifier": "B",
                            "Spaces": [
                                { "ManagedSpaceID": 101, "Type": 0 },
                                { "ManagedSpaceID": 102, "Type": 0 }
                            ]
                        }
                    ]
                }
            }
        });

        let n = infer_space_count_from_macos_spaces_defaults_json(&v);
        assert_eq!(n, Some(2));
    }

    #[traced_test]
    fn infers_space_count_from_spaces_arrays_when_ids_are_missing() {
        let v = serde_json::json!({
            "SpacesDisplayConfiguration": {
                "Management Data": {
                    "Monitors": [
                        { "Spaces": [ { "a": 1 }, { "a": 2 }, { "a": 3 } ] }
                    ]
                }
            }
        });

        let n = infer_space_count_from_macos_spaces_defaults_json(&v);
        assert_eq!(n, Some(3));
    }

    #[traced_test]
    fn returns_none_when_expected_structure_is_missing() {
        let v = serde_json::json!({
            "NotSpacesDisplayConfiguration": { "x": 1 }
        });

        let n = infer_space_count_from_macos_spaces_defaults_json(&v);
        assert_eq!(n, None);
    }
}

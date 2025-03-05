// ---------------- [ File: workspacer-register/src/filter_new_macros_for_duplicates.rs ]
crate::ix!();

/// Removes from `new_macros` any macro whose stem is already in `old_macros`.
pub fn filter_new_macros_for_duplicates(
    old_macros: &[TopBlockMacro],
    new_macros: &[TopBlockMacro],
) -> Vec<TopBlockMacro> {
    // gather old stems into a Set
    let old_stems: std::collections::HashSet<_> =
        old_macros.iter().map(|om| om.stem().to_owned()).collect();

    // keep only new macros whose stem is not in old_macros
    new_macros
        .iter()
        .filter(|nm| !old_stems.contains(nm.stem()))
        .cloned()
        .collect()
}

#[cfg(test)]
mod test_filter_new_macros_for_duplicates {
    use super::*;

    #[traced_test]
    fn test_empty_old_keeps_all_new() {
        let old = vec![];
        let new = vec![
            TopBlockMacroBuilder::default()
                .stem("alpha")
                .build().unwrap(),
            TopBlockMacroBuilder::default()
                .stem("beta")
                .build().unwrap(),
        ];

        let result = filter_new_macros_for_duplicates(&old, &new);
        assert_eq!(result.len(), 2, "No old macros => keep all new");
        assert_eq!(result[0].stem(), "alpha");
        assert_eq!(result[1].stem(), "beta");
    }

    #[traced_test]
    fn test_old_overlaps_with_new() {
        let old = vec![
            TopBlockMacroBuilder::default()
                .stem("alpha")
                .build().unwrap(),
            TopBlockMacroBuilder::default()
                .stem("common")
                .build().unwrap(),
        ];
        let new = vec![
            TopBlockMacroBuilder::default()
                .stem("common")
                .build().unwrap(),
            TopBlockMacroBuilder::default()
                .stem("gamma")
                .build().unwrap(),
        ];

        let result = filter_new_macros_for_duplicates(&old, &new);
        // "common" should be filtered out
        assert_eq!(result.len(), 1, "We only keep gamma");
        assert_eq!(result[0].stem(), "gamma");
    }

    #[traced_test]
    fn test_no_overlap() {
        let old = vec![
            TopBlockMacroBuilder::default()
                .stem("foo")
                .build().unwrap(),
        ];
        let new = vec![
            TopBlockMacroBuilder::default()
                .stem("bar")
                .build().unwrap(),
            TopBlockMacroBuilder::default()
                .stem("baz")
                .build().unwrap(),
        ];
        let result = filter_new_macros_for_duplicates(&old, &new);
        assert_eq!(result.len(), 2, "None were duplicates => keep both");
        assert_eq!(result[0].stem(), "bar");
        assert_eq!(result[1].stem(), "baz");
    }

    #[traced_test]
    fn test_case_sensitivity_same_name_different_case() {
        // We treat stems as exact matches, so "Alpha" vs "alpha" is not the same.
        let old = vec![
            TopBlockMacroBuilder::default()
                .stem("alpha")
                .build().unwrap(),
        ];
        let new = vec![
            TopBlockMacroBuilder::default()
                .stem("Alpha")
                .build().unwrap(),
            TopBlockMacroBuilder::default()
                .stem("beta")
                .build().unwrap(),
        ];
        let result = filter_new_macros_for_duplicates(&old, &new);
        // We keep both because "Alpha" != "alpha"
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].stem(), "Alpha");
        assert_eq!(result[1].stem(), "beta");
    }

    #[traced_test]
    fn test_duplicate_in_new_itself_is_untouched() {
        // This helper function only checks collisions *between old and new*,
        // not duplicates within new.
        // So if new has duplicates, we keep them as is (the caller must handle that).
        let old = vec![
            TopBlockMacroBuilder::default()
                .stem("alpha")
                .build().unwrap(),
        ];
        let new = vec![
            TopBlockMacroBuilder::default()
                .stem("common")
                .build().unwrap(),
            TopBlockMacroBuilder::default()
                .stem("common")
                .build().unwrap(),
        ];
        let result = filter_new_macros_for_duplicates(&old, &new);
        // none of them are in old, so keep both
        assert_eq!(result.len(), 2, "We do not unify duplicates *within* new");
    }
}

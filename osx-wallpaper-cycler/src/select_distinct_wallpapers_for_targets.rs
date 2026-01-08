// ---------------- [ File: osx-wallpaper-cycler/src/select_distinct_wallpapers_for_targets.rs ]
crate::ix!();

pub fn select_distinct_wallpapers_for_targets(
    eligible_candidates: &[DropboxWallpaperCandidate],
    target_count: usize,
) -> Vec<DropboxWallpaperCandidate> {
    let mut candidates = eligible_candidates.to_vec();
    let mut rng = rand::rng();
    use rand::seq::SliceRandom;
    candidates.shuffle(&mut rng);

    let mut selected: Vec<DropboxWallpaperCandidate> = Vec::new();
    let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();

    for c in candidates.iter() {
        if selected.len() >= target_count {
            break;
        }
        if seen.insert(c.id().to_string()) {
            selected.push(c.clone());
        }
    }

    selected
}

#[cfg(test)]
mod distinct_wallpaper_selection_contract_suite {
    use super::*;

    fn build_candidate(id: &str) -> DropboxWallpaperCandidate {
        DropboxWallpaperCandidateBuilder::default()
            .id(id)
            .name(format!("{id}.jpg"))
            .path_lower(format!("/wallpapers/{id}.jpg"))
            .build()
            .unwrap()
    }

    #[traced_test]
    fn selection_is_empty_when_target_count_is_zero() {
        let eligible = vec![build_candidate("id:1"), build_candidate("id:2")];
        let selected = select_distinct_wallpapers_for_targets(&eligible, 0);
        assert!(selected.is_empty());
    }

    #[traced_test]
    fn selection_is_empty_when_no_candidates_exist() {
        let eligible: Vec<DropboxWallpaperCandidate> = Vec::new();
        let selected = select_distinct_wallpapers_for_targets(&eligible, 5);
        assert!(selected.is_empty());
    }

    #[traced_test]
    fn selection_never_contains_duplicate_ids_and_never_exceeds_unique_count() {
        let eligible = vec![
            build_candidate("id:1"),
            build_candidate("id:1"),
            build_candidate("id:2"),
            build_candidate("id:2"),
            build_candidate("id:3"),
        ];

        let selected = select_distinct_wallpapers_for_targets(&eligible, 10);

        let mut seen: std::collections::HashSet<String> = std::collections::HashSet::new();
        for c in selected.iter() {
            assert!(seen.insert(c.id().to_string()));
        }

        assert_eq!(seen.len(), selected.len());
        assert!(selected.len() <= 3);

        let eligible_ids: std::collections::HashSet<String> =
            eligible.iter().map(|c| c.id().to_string()).collect();
        for c in selected.iter() {
            assert!(eligible_ids.contains(c.id()));
        }
    }

    #[traced_test]
    fn selection_length_is_at_most_target_count() {
        let eligible = vec![
            build_candidate("id:1"),
            build_candidate("id:2"),
            build_candidate("id:3"),
            build_candidate("id:4"),
        ];

        let selected = select_distinct_wallpapers_for_targets(&eligible, 2);
        assert!(selected.len() <= 2);
    }
}

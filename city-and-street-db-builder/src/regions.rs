// ---------------- [ File: src/regions.rs ]
crate::ix!();

/// Returns the regions of interest (DMV region in this example)
pub fn dmv_regions() -> Vec<WorldRegion> {
    vec![
        USRegion::UnitedState(UnitedState::Maryland).into(),
        USRegion::UnitedState(UnitedState::Virginia).into(),
        USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia).into(),
    ]
}

pub fn world_regions() -> Vec<WorldRegion> {
    warn!("in the future, we will want to expand our set of world regions to include places outside dmv");
    dmv_regions()
}

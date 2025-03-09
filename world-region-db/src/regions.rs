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

pub fn va_regions() -> Vec<WorldRegion> {
    vec![
        USRegion::UnitedState(UnitedState::Virginia).into(),
    ]
}

pub fn world_regions() -> Vec<WorldRegion> {
    WorldRegion::iter().collect()
}

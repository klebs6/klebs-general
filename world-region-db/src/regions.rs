// ---------------- [ File: src/regions.rs ]
crate::ix!();

pub fn va_regions() -> Vec<WorldRegion> {
    vec![
        USRegion::UnitedState(UnitedState::Virginia).into(),
    ]
}

pub fn ca_regions() -> Vec<WorldRegion> {
    vec![
        USRegion::UnitedState(UnitedState::California).into(),
    ]
}

pub fn tn_regions() -> Vec<WorldRegion> {
    vec![
        USRegion::UnitedState(UnitedState::Tennessee).into(),
    ]
}

pub fn tx_regions() -> Vec<WorldRegion> {
    vec![
        USRegion::UnitedState(UnitedState::Texas).into(),
    ]
}

pub fn fl_regions() -> Vec<WorldRegion> {
    vec![
        USRegion::UnitedState(UnitedState::Florida).into(),
    ]
}

pub fn nc_regions() -> Vec<WorldRegion> {
    vec![
        USRegion::UnitedState(UnitedState::NorthCarolina).into(),
    ]
}

pub fn sc_regions() -> Vec<WorldRegion> {
    vec![
        USRegion::UnitedState(UnitedState::SouthCarolina).into(),
    ]
}

/// Returns the regions of interest (DMV region in this example)
pub fn dmv_regions() -> Vec<WorldRegion> {
    vec![
        USRegion::UnitedState(UnitedState::Maryland).into(),
        USRegion::UnitedState(UnitedState::Virginia).into(),
        USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia).into(),
    ]
}

pub fn world_regions() -> Vec<WorldRegion> {
    WorldRegion::iter().collect()
}

pub fn known_regions() -> Vec<WorldRegion> {
    vec![
        USRegion::UnitedState(UnitedState::Maryland).into(),
        USRegion::UnitedState(UnitedState::Florida).into(),
        USRegion::UnitedState(UnitedState::SouthCarolina).into(),
        USRegion::UnitedState(UnitedState::NorthCarolina).into(),
        USRegion::UnitedState(UnitedState::Texas).into(),
        USRegion::UnitedState(UnitedState::Tennessee).into(),
        USRegion::UnitedState(UnitedState::California).into(),
        USRegion::UnitedState(UnitedState::Virginia).into(),
        USRegion::USFederalDistrict(USFederalDistrict::DistrictOfColumbia).into(),
    ]
}

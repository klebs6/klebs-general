crate::ix!();

// For UnitedKingdomRegion
impl From<EnglandRegion> for UnitedKingdomRegion {
    fn from(value: EnglandRegion) -> Self {
        UnitedKingdomRegion::England(value)
    }
}

// For EuropeRegion
impl From<FranceRegion> for EuropeRegion {
    fn from(value: FranceRegion) -> Self {
        EuropeRegion::France(value)
    }
}

impl From<GermanyRegion> for EuropeRegion {
    fn from(value: GermanyRegion) -> Self {
        EuropeRegion::Germany(value)
    }
}

impl From<ItalyRegion> for EuropeRegion {
    fn from(value: ItalyRegion) -> Self {
        EuropeRegion::Italy(value)
    }
}

impl From<NetherlandsRegion> for EuropeRegion {
    fn from(value: NetherlandsRegion) -> Self {
        EuropeRegion::Netherlands(value)
    }
}

impl From<PolandRegion> for EuropeRegion {
    fn from(value: PolandRegion) -> Self {
        EuropeRegion::Poland(value)
    }
}

impl From<RussianFederationRegion> for EuropeRegion {
    fn from(value: RussianFederationRegion) -> Self {
        EuropeRegion::RussianFederation(value)
    }
}

impl From<SpainRegion> for EuropeRegion {
    fn from(value: SpainRegion) -> Self {
        EuropeRegion::Spain(value)
    }
}

impl From<UnitedKingdomRegion> for EuropeRegion {
    fn from(value: UnitedKingdomRegion) -> Self {
        EuropeRegion::UnitedKingdom(value)
    }
}

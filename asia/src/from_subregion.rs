crate::ix!();

//-------------------------------------------------------------
// If desired, implement From<Subregion> for Country:
// (Just like in Europe's code)
//-------------------------------------------------------------
impl From<ChinaRegion> for Country {
    fn from(_value: ChinaRegion) -> Self {
        Country::China
    }
}
impl From<IndiaRegion> for Country {
    fn from(_value: IndiaRegion) -> Self {
        Country::India
    }
}
impl From<JapanRegion> for Country {
    fn from(_value: JapanRegion) -> Self {
        Country::Japan
    }
}
impl From<IndonesiaRegion> for Country {
    fn from(_value: IndonesiaRegion) -> Self {
        Country::Indonesia
    }
}

// For AsiaRegion
impl From<ChinaRegion> for AsiaRegion {
    fn from(value: ChinaRegion) -> Self {
        AsiaRegion::China(value)
    }
}

impl From<IndiaRegion> for AsiaRegion {
    fn from(value: IndiaRegion) -> Self {
        AsiaRegion::India(value)
    }
}

impl From<JapanRegion> for AsiaRegion {
    fn from(value: JapanRegion) -> Self {
        AsiaRegion::Japan(value)
    }
}

impl From<IndonesiaRegion> for AsiaRegion {
    fn from(value: IndonesiaRegion) -> Self {
        AsiaRegion::Indonesia(value)
    }
}

impl From<RussianFederationRegion> for AsiaRegion {
    fn from(value: RussianFederationRegion) -> Self {
        AsiaRegion::RussianFederation(value)
    }
}


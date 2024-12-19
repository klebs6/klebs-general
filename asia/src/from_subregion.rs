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

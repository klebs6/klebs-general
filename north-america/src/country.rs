crate::ix!();

//-------------------------------------------------------------
// Conversion From NorthAmericaRegion to Country
//-------------------------------------------------------------
impl TryFrom<NorthAmericaRegion> for Country {
    type Error = NorthAmericaRegionConversionError;

    fn try_from(value: NorthAmericaRegion) -> Result<Self, Self::Error> {
        match value {
            NorthAmericaRegion::Canada(_)       => Ok(Country::Canada),
            NorthAmericaRegion::Greenland       => Ok(Country::Denmark),
            NorthAmericaRegion::Mexico          => Ok(Country::Mexico),
            NorthAmericaRegion::UnitedStates(_) => Ok(Country::USA),
        }
    }
}

//-------------------------------------------------------------
// Conversion From Country to NorthAmericaRegion
//-------------------------------------------------------------
impl TryFrom<Country> for NorthAmericaRegion {
    type Error = NorthAmericaRegionConversionError;

    fn try_from(c: Country) -> Result<Self, Self::Error> {
        match c {
            Country::Canada    => Ok(NorthAmericaRegion::Canada(CanadaRegion::default())),
            Country::Denmark   => Ok(NorthAmericaRegion::Greenland),
            Country::Mexico    => Ok(NorthAmericaRegion::Mexico),
            Country::USA       => Ok(NorthAmericaRegion::UnitedStates(USRegion::default())),

            // Any country not in North America:
            other => Err(NorthAmericaRegionConversionError::not_north_american(&other.to_string())),
        }
    }
}

//-------------------------------------------------------------
// ISO Code conversions
//-------------------------------------------------------------
impl TryFrom<NorthAmericaRegion> for Iso3166Alpha2 {
    type Error = NorthAmericaRegionConversionError;
    fn try_from(value: NorthAmericaRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha2())
    }
}

impl TryFrom<NorthAmericaRegion> for Iso3166Alpha3 {
    type Error = NorthAmericaRegionConversionError;
    fn try_from(value: NorthAmericaRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha3())
    }
}

impl TryFrom<NorthAmericaRegion> for CountryCode {
    type Error = NorthAmericaRegionConversionError;
    fn try_from(value: NorthAmericaRegion) -> Result<Self, Self::Error> {
        let a2: Iso3166Alpha2 = value.try_into()?;
        Ok(CountryCode::Alpha2(a2))
    }
}

//-------------------------------------------------------------
// Implement From<Subregions> for Country
//-------------------------------------------------------------
impl From<CanadaRegion> for Country {
    fn from(_value: CanadaRegion) -> Self {
        Country::Canada
    }
}

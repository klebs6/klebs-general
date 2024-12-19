crate::ix!();

//-------------------------------------------------------------
// Conversion From SouthAmericaRegion to Country
//-------------------------------------------------------------
impl TryFrom<SouthAmericaRegion> for Country {
    type Error = SouthAmericaRegionConversionError;

    fn try_from(value: SouthAmericaRegion) -> Result<Self, Self::Error> {
        match value {
            SouthAmericaRegion::Argentina   => Ok(Country::Argentina),
            SouthAmericaRegion::Bolivia     => Ok(Country::Bolivia),
            SouthAmericaRegion::Brazil(_)   => Ok(Country::Brazil),
            SouthAmericaRegion::Chile       => Ok(Country::Chile),
            SouthAmericaRegion::Colombia    => Ok(Country::Colombia),
            SouthAmericaRegion::Ecuador     => Ok(Country::Ecuador),
            SouthAmericaRegion::Guyana      => Ok(Country::Guyana),
            SouthAmericaRegion::Paraguay    => Ok(Country::Paraguay),
            SouthAmericaRegion::Peru        => Ok(Country::Peru),
            SouthAmericaRegion::Suriname    => Ok(Country::Suriname),
            SouthAmericaRegion::Uruguay     => Ok(Country::Uruguay),
            SouthAmericaRegion::Venezuela   => Ok(Country::Venezuela),
        }
    }
}

//-------------------------------------------------------------
// Conversion From Country to SouthAmericaRegion
//-------------------------------------------------------------
impl TryFrom<Country> for SouthAmericaRegion {
    type Error = SouthAmericaRegionConversionError;

    fn try_from(c: Country) -> Result<Self, Self::Error> {
        match c {
            Country::Argentina => Ok(SouthAmericaRegion::Argentina),
            Country::Bolivia   => Ok(SouthAmericaRegion::Bolivia),
            Country::Brazil    => Ok(SouthAmericaRegion::Brazil(BrazilRegion::default())),
            Country::Chile     => Ok(SouthAmericaRegion::Chile),
            Country::Colombia  => Ok(SouthAmericaRegion::Colombia),
            Country::Ecuador   => Ok(SouthAmericaRegion::Ecuador),
            Country::Guyana    => Ok(SouthAmericaRegion::Guyana),
            Country::Paraguay  => Ok(SouthAmericaRegion::Paraguay),
            Country::Peru      => Ok(SouthAmericaRegion::Peru),
            Country::Suriname  => Ok(SouthAmericaRegion::Suriname),
            Country::Uruguay   => Ok(SouthAmericaRegion::Uruguay),
            Country::Venezuela => Ok(SouthAmericaRegion::Venezuela),

            // Any country not in South America:
            other => Err(SouthAmericaRegionConversionError::NotSouthAmerican { country: other }),
        }
    }
}

//-------------------------------------------------------------
// ISO Code conversions
//-------------------------------------------------------------
impl TryFrom<SouthAmericaRegion> for Iso3166Alpha2 {
    type Error = SouthAmericaRegionConversionError;
    fn try_from(value: SouthAmericaRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha2())
    }
}

impl TryFrom<SouthAmericaRegion> for Iso3166Alpha3 {
    type Error = SouthAmericaRegionConversionError;
    fn try_from(value: SouthAmericaRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha3())
    }
}

impl TryFrom<SouthAmericaRegion> for CountryCode {
    type Error = SouthAmericaRegionConversionError;
    fn try_from(value: SouthAmericaRegion) -> Result<Self, Self::Error> {
        let a2: Iso3166Alpha2 = value.try_into()?;
        Ok(CountryCode::Alpha2(a2))
    }
}

//-------------------------------------------------------------
// Implement From<BrazilRegion> for Country
//-------------------------------------------------------------
impl From<BrazilRegion> for Country {
    fn from(_value: BrazilRegion) -> Self {
        Country::Brazil
    }
}

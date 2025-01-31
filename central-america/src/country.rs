crate::ix!();

//-------------------------------------------------------------
// Conversion From CentralAmericaRegion to Country
//-------------------------------------------------------------
impl TryFrom<CentralAmericaRegion> for Country {
    type Error = CentralAmericaRegionConversionError;

    fn try_from(value: CentralAmericaRegion) -> Result<Self, Self::Error> {
        match value {
            CentralAmericaRegion::Bahamas                   => Ok(Country::Bahamas),
            CentralAmericaRegion::Belize                    => Ok(Country::Belize),
            CentralAmericaRegion::CostaRica                 => Ok(Country::CostaRica),
            CentralAmericaRegion::Cuba                      => Ok(Country::Cuba),
            CentralAmericaRegion::ElSalvador                => Ok(Country::ElSalvador),
            CentralAmericaRegion::Guatemala                 => Ok(Country::Guatemala),
            CentralAmericaRegion::HaitiAndDominicanRepublic => {
                // Combined region; choose Haiti by convention
                Ok(Country::Haiti)
            },
            CentralAmericaRegion::Honduras                 => Ok(Country::Honduras),
            CentralAmericaRegion::Jamaica                  => Ok(Country::Jamaica),
            CentralAmericaRegion::Nicaragua                => Ok(Country::Nicaragua),
            CentralAmericaRegion::Panama                   => Ok(Country::Panama),
        }
    }
}

//-------------------------------------------------------------
// Conversion From Country to CentralAmericaRegion
//-------------------------------------------------------------
impl TryFrom<Country> for CentralAmericaRegion {
    type Error = CentralAmericaRegionConversionError;

    fn try_from(c: Country) -> Result<Self, Self::Error> {
        match c {
            Country::Bahamas           => Ok(CentralAmericaRegion::Bahamas),
            Country::Belize            => Ok(CentralAmericaRegion::Belize),
            Country::CostaRica         => Ok(CentralAmericaRegion::CostaRica),
            Country::Cuba              => Ok(CentralAmericaRegion::Cuba),
            Country::ElSalvador        => Ok(CentralAmericaRegion::ElSalvador),
            Country::Guatemala         => Ok(CentralAmericaRegion::Guatemala),
            Country::Haiti             => Ok(CentralAmericaRegion::HaitiAndDominicanRepublic),
            Country::DominicanRepublic => Ok(CentralAmericaRegion::HaitiAndDominicanRepublic),
            Country::Honduras          => Ok(CentralAmericaRegion::Honduras),
            Country::Jamaica           => Ok(CentralAmericaRegion::Jamaica),
            Country::Nicaragua         => Ok(CentralAmericaRegion::Nicaragua),
            Country::Panama            => Ok(CentralAmericaRegion::Panama),

            // Any country not in this Central America region listing:
            other => Err(CentralAmericaRegionConversionError::NotCentralAmerican { country: other } ),
        }
    }
}

//-------------------------------------------------------------
// ISO Code conversions
//-------------------------------------------------------------
impl TryFrom<CentralAmericaRegion> for Iso3166Alpha2 {
    type Error = CentralAmericaRegionConversionError;
    fn try_from(value: CentralAmericaRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha2())
    }
}

impl TryFrom<CentralAmericaRegion> for Iso3166Alpha3 {
    type Error = CentralAmericaRegionConversionError;
    fn try_from(value: CentralAmericaRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha3())
    }
}

impl TryFrom<CentralAmericaRegion> for CountryCode {
    type Error = CentralAmericaRegionConversionError;
    fn try_from(value: CentralAmericaRegion) -> Result<Self, Self::Error> {
        let a2: Iso3166Alpha2 = value.try_into()?;
        Ok(CountryCode::Alpha2(a2))
    }
}

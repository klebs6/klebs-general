// ---------------- [ File: src/country.rs ]
crate::ix!();

// TryFrom<Country> for WorldRegion
impl TryFrom<Country> for WorldRegion {

    type Error = WorldRegionConversionError;

    // Try each region set in turn
    // If africa::AfricaRegion::try_from(c) works, we have an Africa variant, etc.
    fn try_from(c: Country) -> Result<Self, Self::Error> {

        if let Ok(a) = NorthAmericaRegion::try_from(c.clone()) {
            return Ok(WorldRegion::NorthAmerica(a));
        } 

        if let Ok(a) = CentralAmericaRegion::try_from(c.clone()) {
            return Ok(WorldRegion::CentralAmerica(a));
        } 

        if let Ok(a) = SouthAmericaRegion::try_from(c.clone()) {
            return Ok(WorldRegion::SouthAmerica(a));
        } 

        if let Ok(a) = EuropeRegion::try_from(c.clone()) {
            return Ok(WorldRegion::Europe(a));
        } 

        if let Ok(a) = AfricaRegion::try_from(c.clone()) {
            return Ok(WorldRegion::Africa(a));
        }

        if let Ok(a) = AsiaRegion::try_from(c.clone()) {
            return Ok(WorldRegion::Asia(a));
        } 

        if let Ok(a) = AustraliaOceaniaAntarcticaRegion::try_from(c.clone()) {
            return Ok(WorldRegion::AustraliaOceaniaAntarctica(a));
        }

        Err(WorldRegionConversionError::NotRepresented { country: c })
    }
}

// TryFrom<WorldRegion> for Country
impl TryFrom<WorldRegion> for Country {

    type Error = WorldRegionConversionError;

    fn try_from(value: WorldRegion) -> Result<Self, Self::Error> {
        match value {
            WorldRegion::NorthAmerica(r)               => Ok(Country::try_from(r)?),
            WorldRegion::CentralAmerica(r)             => Ok(Country::try_from(r)?),
            WorldRegion::SouthAmerica(r)               => Ok(Country::try_from(r)?),
            WorldRegion::Europe(r)                     => Ok(Country::try_from(r)?),
            WorldRegion::Africa(r)                     => Ok(Country::try_from(r)?),
            WorldRegion::Asia(r)                       => Ok(Country::try_from(r)?),
            WorldRegion::AustraliaOceaniaAntarctica(r) => Ok(Country::try_from(r)?),
        }
    }
}

// ISO code conversions
impl TryFrom<WorldRegion> for Iso3166Alpha2 {
    type Error = WorldRegionConversionError;

    fn try_from(value: WorldRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha2())
    }
}

impl TryFrom<WorldRegion> for Iso3166Alpha3 {
    type Error = WorldRegionConversionError;

    fn try_from(value: WorldRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha3())
    }
}

impl TryFrom<WorldRegion> for CountryCode {
    type Error = WorldRegionConversionError;

    fn try_from(value: WorldRegion) -> Result<Self, Self::Error> {
        let a2: Iso3166Alpha2 = value.try_into()?;
        Ok(CountryCode::Alpha2(a2))
    }
}

crate::ix!();

//-------------------------------------------------------------
// Conversions from Region to Country
// (Adjust these mappings according to your actual Country enum)
//-------------------------------------------------------------
impl TryFrom<AustraliaOceaniaAntarcticaRegion> for Country {
    type Error = AoaRegionConversionError;

    fn try_from(value: AustraliaOceaniaAntarcticaRegion) -> Result<Self, Self::Error> {
        match value {
            AustraliaOceaniaAntarcticaRegion::Australia       => Ok(Country::Australia),
            AustraliaOceaniaAntarcticaRegion::Fiji            => Ok(Country::Fiji),
            AustraliaOceaniaAntarcticaRegion::Kiribati        => Ok(Country::Kiribati),
            AustraliaOceaniaAntarcticaRegion::MarshallIslands => Ok(Country::MarshallIslands),
            AustraliaOceaniaAntarcticaRegion::Micronesia      => Ok(Country::Micronesia),
            AustraliaOceaniaAntarcticaRegion::Nauru           => Ok(Country::Nauru),
            AustraliaOceaniaAntarcticaRegion::NewZealand      => Ok(Country::NewZealand),
            AustraliaOceaniaAntarcticaRegion::Palau           => Ok(Country::Palau),
            AustraliaOceaniaAntarcticaRegion::PapuaNewGuinea  => Ok(Country::PapuaNewGuinea),
            AustraliaOceaniaAntarcticaRegion::Samoa           => Ok(Country::Samoa),
            AustraliaOceaniaAntarcticaRegion::SolomonIslands  => Ok(Country::SolomonIslands),
            AustraliaOceaniaAntarcticaRegion::Tonga           => Ok(Country::Tonga),
            AustraliaOceaniaAntarcticaRegion::Tuvalu          => Ok(Country::Tuvalu),
            AustraliaOceaniaAntarcticaRegion::Vanuatu         => Ok(Country::Vanuatu),

            // Unsupported or multiple dependencies:
            AustraliaOceaniaAntarcticaRegion::Antarctica      => Err(AoaRegionConversionError::unsupported_region("Antarctica")),
            AustraliaOceaniaAntarcticaRegion::AmericanOceania => Err(AoaRegionConversionError::unsupported_region("American Oceania")),
            AustraliaOceaniaAntarcticaRegion::CookIslands     => Err(AoaRegionConversionError::unsupported_region("Cook Islands")),
            AustraliaOceaniaAntarcticaRegion::IleDeClipperton => Err(AoaRegionConversionError::unsupported_region("Île de Clipperton")),
            AustraliaOceaniaAntarcticaRegion::NewCaledonia    => Err(AoaRegionConversionError::unsupported_region("New Caledonia")),
            AustraliaOceaniaAntarcticaRegion::Niue            => Err(AoaRegionConversionError::unsupported_region("Niue")),
            AustraliaOceaniaAntarcticaRegion::PitcairnIslands => Err(AoaRegionConversionError::unsupported_region("Pitcairn Islands")),
            AustraliaOceaniaAntarcticaRegion::FrenchPolynesia => Err(AoaRegionConversionError::unsupported_region("French Polynesia")),
            AustraliaOceaniaAntarcticaRegion::Tokelau         => Err(AoaRegionConversionError::unsupported_region("Tokelau")),
            AustraliaOceaniaAntarcticaRegion::WallisEtFutuna  => Err(AoaRegionConversionError::unsupported_region("Wallis et Futuna")),
        }
    }
}

//-------------------------------------------------------------
// Conversions from Country to Region
// If a Country doesn't fit here, return NotInAoa
//-------------------------------------------------------------
impl TryFrom<Country> for AustraliaOceaniaAntarcticaRegion {
    type Error = AoaRegionConversionError;

    fn try_from(c: Country) -> Result<Self, Self::Error> {
        match c {
            Country::Australia        => Ok(AustraliaOceaniaAntarcticaRegion::Australia),
            Country::Fiji             => Ok(AustraliaOceaniaAntarcticaRegion::Fiji),
            Country::Kiribati         => Ok(AustraliaOceaniaAntarcticaRegion::Kiribati),
            Country::MarshallIslands  => Ok(AustraliaOceaniaAntarcticaRegion::MarshallIslands),
            Country::Micronesia       => Ok(AustraliaOceaniaAntarcticaRegion::Micronesia),
            Country::Nauru            => Ok(AustraliaOceaniaAntarcticaRegion::Nauru),
            Country::NewZealand       => Ok(AustraliaOceaniaAntarcticaRegion::NewZealand),
            Country::Palau            => Ok(AustraliaOceaniaAntarcticaRegion::Palau),
            Country::PapuaNewGuinea   => Ok(AustraliaOceaniaAntarcticaRegion::PapuaNewGuinea),
            Country::Samoa            => Ok(AustraliaOceaniaAntarcticaRegion::Samoa),
            Country::SolomonIslands   => Ok(AustraliaOceaniaAntarcticaRegion::SolomonIslands),
            Country::Tonga            => Ok(AustraliaOceaniaAntarcticaRegion::Tonga),
            Country::Tuvalu           => Ok(AustraliaOceaniaAntarcticaRegion::Tuvalu),
            Country::Vanuatu          => Ok(AustraliaOceaniaAntarcticaRegion::Vanuatu),

            // Not in this region:
            other => Err(AoaRegionConversionError::not_in_aoa(&other.to_string())),
        }
    }
}

//-------------------------------------------------------------
// ISO Code Conversions
//-------------------------------------------------------------
impl TryFrom<AustraliaOceaniaAntarcticaRegion> for Iso3166Alpha2 {
    type Error = AoaRegionConversionError;
    fn try_from(value: AustraliaOceaniaAntarcticaRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha2())
    }
}

impl TryFrom<AustraliaOceaniaAntarcticaRegion> for Iso3166Alpha3 {
    type Error = AoaRegionConversionError;
    fn try_from(value: AustraliaOceaniaAntarcticaRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha3())
    }
}

impl TryFrom<AustraliaOceaniaAntarcticaRegion> for CountryCode {
    type Error = AoaRegionConversionError;
    fn try_from(value: AustraliaOceaniaAntarcticaRegion) -> Result<Self, Self::Error> {
        let a2: Iso3166Alpha2 = value.try_into()?;
        Ok(CountryCode::Alpha2(a2))
    }
}


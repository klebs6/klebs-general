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
            AustraliaOceaniaAntarcticaRegion::Antarctica      => Err(AoaRegionConversionError::UnsupportedRegion { region: AustraliaOceaniaAntarcticaRegion::Antarctica }),
            AustraliaOceaniaAntarcticaRegion::AmericanOceania => Err(AoaRegionConversionError::UnsupportedRegion { region: AustraliaOceaniaAntarcticaRegion::AmericanOceania }),
            AustraliaOceaniaAntarcticaRegion::CookIslands     => Err(AoaRegionConversionError::UnsupportedRegion { region: AustraliaOceaniaAntarcticaRegion::CookIslands }),
            AustraliaOceaniaAntarcticaRegion::IleDeClipperton => Err(AoaRegionConversionError::UnsupportedRegion { region: AustraliaOceaniaAntarcticaRegion::IleDeClipperton }),
            AustraliaOceaniaAntarcticaRegion::NewCaledonia    => Err(AoaRegionConversionError::UnsupportedRegion { region: AustraliaOceaniaAntarcticaRegion::NewCaledonia }),
            AustraliaOceaniaAntarcticaRegion::Niue            => Err(AoaRegionConversionError::UnsupportedRegion { region: AustraliaOceaniaAntarcticaRegion::Niue }),
            AustraliaOceaniaAntarcticaRegion::PitcairnIslands => Err(AoaRegionConversionError::UnsupportedRegion { region: AustraliaOceaniaAntarcticaRegion::PitcairnIslands }),
            AustraliaOceaniaAntarcticaRegion::FrenchPolynesia => Err(AoaRegionConversionError::UnsupportedRegion { region: AustraliaOceaniaAntarcticaRegion::FrenchPolynesia }),
            AustraliaOceaniaAntarcticaRegion::Tokelau         => Err(AoaRegionConversionError::UnsupportedRegion { region: AustraliaOceaniaAntarcticaRegion::Tokelau }),
            AustraliaOceaniaAntarcticaRegion::WallisEtFutuna  => Err(AoaRegionConversionError::UnsupportedRegion { region: AustraliaOceaniaAntarcticaRegion::WallisEtFutuna }),
        }
    }
}

//-------------------------------------------------------------
// Conversions from Country to Region
// If a Country doesn't fit here, return NotAoan
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
            other => Err(AoaRegionConversionError::NotAoan { country: other }),
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


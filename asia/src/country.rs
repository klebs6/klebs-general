crate::ix!();

//-------------------------------------------------------------
// Conversion From AsiaRegion to Country
//-------------------------------------------------------------
impl TryFrom<AsiaRegion> for Country {
    type Error = AsiaRegionConversionError;

    fn try_from(value: AsiaRegion) -> Result<Self, Self::Error> {
        match value {
            AsiaRegion::Afghanistan => Ok(Country::Afghanistan),
            AsiaRegion::Armenia     => Ok(Country::Armenia),
            AsiaRegion::Azerbaijan  => Ok(Country::Azerbaijan),
            AsiaRegion::Bangladesh  => Ok(Country::Bangladesh),
            AsiaRegion::Bhutan      => Ok(Country::Bhutan),
            AsiaRegion::Cambodia    => Ok(Country::Cambodia),
            AsiaRegion::China(_)    => Ok(Country::China),
            AsiaRegion::EastTimor   => Ok(Country::TimorLeste),
            AsiaRegion::GccStates   => {
                // This is a combined region (Bahrain, Kuwait, Oman, Qatar, Saudi Arabia, UAE).
                // Not a single country. If we must pick one or fail:
                Err(AsiaRegionConversionError::unsupported_region("GCC States"))
            },
            AsiaRegion::India(_)           => Ok(Country::India),
            AsiaRegion::Indonesia(_)       => Ok(Country::Indonesia),
            AsiaRegion::Iran               => Ok(Country::Iran),
            AsiaRegion::Iraq               => Ok(Country::Iraq),
            AsiaRegion::IsraelAndPalestine => {
                // Combined region. If we must pick one:
                // Let's map it to Israel by convention:
                Ok(Country::Israel)
            },
            AsiaRegion::Japan(_)                => Ok(Country::Japan),
            AsiaRegion::Jordan                  => Ok(Country::Jordan),
            AsiaRegion::Kazakhstan              => Ok(Country::Kazakhstan),
            AsiaRegion::Kyrgyzstan              => Ok(Country::Kyrgyzstan),
            AsiaRegion::Laos                    => Ok(Country::Laos),
            AsiaRegion::Lebanon                 => Ok(Country::Lebanon),
            AsiaRegion::MalaysiaSingaporeBrunei => {
                // Another combined region. If we must choose one:
                // Let's map it to Malaysia by convention:
                Ok(Country::Malaysia)
            },
            AsiaRegion::Maldives             => Ok(Country::Maldives),
            AsiaRegion::Mongolia             => Ok(Country::Mongolia),
            AsiaRegion::Myanmar              => Ok(Country::Myanmar),
            AsiaRegion::Nepal                => Ok(Country::Nepal),
            AsiaRegion::NorthKorea           => Ok(Country::NorthKorea),
            AsiaRegion::Pakistan             => Ok(Country::Pakistan),
            AsiaRegion::Philippines          => Ok(Country::Philippines),
            AsiaRegion::RussianFederation(_) => Ok(Country::Russia),
            AsiaRegion::SouthKorea           => Ok(Country::SouthKorea),
            AsiaRegion::SriLanka             => Ok(Country::SriLanka),
            AsiaRegion::Syria                => Ok(Country::Syria),
            AsiaRegion::Taiwan               => Ok(Country::Taiwan),
            AsiaRegion::Tajikistan           => Ok(Country::Tajikistan),
            AsiaRegion::Thailand             => Ok(Country::Thailand),
            AsiaRegion::Turkmenistan         => Ok(Country::Turkmenistan),
            AsiaRegion::Uzbekistan           => Ok(Country::Uzbekistan),
            AsiaRegion::Vietnam              => Ok(Country::Vietnam),
            AsiaRegion::Yemen                => Ok(Country::Yemen),
        }
    }
}

//-------------------------------------------------------------
// Conversion From Country to AsiaRegion
//-------------------------------------------------------------
impl TryFrom<Country> for AsiaRegion {
    type Error = AsiaRegionConversionError;

    fn try_from(c: Country) -> Result<Self, Self::Error> {
        match c {
            Country::Afghanistan => Ok(AsiaRegion::Afghanistan),
            Country::Armenia     => Ok(AsiaRegion::Armenia),
            Country::Azerbaijan  => Ok(AsiaRegion::Azerbaijan),
            Country::Bangladesh  => Ok(AsiaRegion::Bangladesh),
            Country::Bhutan      => Ok(AsiaRegion::Bhutan),
            Country::Cambodia    => Ok(AsiaRegion::Cambodia),
            Country::China       => Ok(AsiaRegion::China(ChinaRegion::default())),
            Country::TimorLeste  => Ok(AsiaRegion::EastTimor),
            Country::India       => Ok(AsiaRegion::India(IndiaRegion::default())),
            Country::Indonesia   => Ok(AsiaRegion::Indonesia(IndonesiaRegion::default())),
            Country::Iran        => Ok(AsiaRegion::Iran),
            Country::Iraq        => Ok(AsiaRegion::Iraq),
            Country::Israel      => {
                // If we only have Israel but in AsiaRegion we have IsraelAndPalestine combined
                // We'll just map back to IsraelAndPalestine:
                Ok(AsiaRegion::IsraelAndPalestine)
            },
            Country::Japan      => Ok(AsiaRegion::Japan(JapanRegion::default())),
            Country::Jordan     => Ok(AsiaRegion::Jordan),
            Country::Kazakhstan => Ok(AsiaRegion::Kazakhstan),
            Country::Kyrgyzstan => Ok(AsiaRegion::Kyrgyzstan),
            Country::Laos       => Ok(AsiaRegion::Laos),
            Country::Lebanon    => Ok(AsiaRegion::Lebanon),
            Country::Malaysia   => {
                // Maps back to MalaysiaSingaporeBrunei combined region
                Ok(AsiaRegion::MalaysiaSingaporeBrunei)
            },
            Country::Maldives     => Ok(AsiaRegion::Maldives),
            Country::Mongolia     => Ok(AsiaRegion::Mongolia),
            Country::Myanmar      => Ok(AsiaRegion::Myanmar),
            Country::Nepal        => Ok(AsiaRegion::Nepal),
            Country::NorthKorea   => Ok(AsiaRegion::NorthKorea),
            Country::Pakistan     => Ok(AsiaRegion::Pakistan),
            Country::Philippines  => Ok(AsiaRegion::Philippines),
            Country::Russia       => Ok(AsiaRegion::RussianFederation(RussianFederationRegion::default())),
            Country::SouthKorea   => Ok(AsiaRegion::SouthKorea),
            Country::SriLanka     => Ok(AsiaRegion::SriLanka),
            Country::Syria        => Ok(AsiaRegion::Syria),
            Country::Taiwan       => Ok(AsiaRegion::Taiwan),
            Country::Tajikistan   => Ok(AsiaRegion::Tajikistan),
            Country::Thailand     => Ok(AsiaRegion::Thailand),
            Country::Turkmenistan => Ok(AsiaRegion::Turkmenistan),
            Country::Uzbekistan   => Ok(AsiaRegion::Uzbekistan),
            Country::Vietnam      => Ok(AsiaRegion::Vietnam),
            Country::Yemen        => Ok(AsiaRegion::Yemen),

            // Any country not in Asia:
            other => Err(AsiaRegionConversionError::not_asian(&other.to_string())),
        }
    }
}

//-------------------------------------------------------------
// ISO Code conversions
//-------------------------------------------------------------
impl TryFrom<AsiaRegion> for Iso3166Alpha2 {
    type Error = AsiaRegionConversionError;
    fn try_from(value: AsiaRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        // Assuming country.alpha2() is available:
        Ok(country.alpha2())
    }
}

impl TryFrom<AsiaRegion> for Iso3166Alpha3 {
    type Error = AsiaRegionConversionError;
    fn try_from(value: AsiaRegion) -> Result<Self, Self::Error> {
        let country: Country = value.try_into()?;
        Ok(country.alpha3())
    }
}

impl TryFrom<AsiaRegion> for CountryCode {
    type Error = AsiaRegionConversionError;
    fn try_from(value: AsiaRegion) -> Result<Self, Self::Error> {
        let a2: Iso3166Alpha2 = value.try_into()?;
        Ok(CountryCode::Alpha2(a2))
    }
}


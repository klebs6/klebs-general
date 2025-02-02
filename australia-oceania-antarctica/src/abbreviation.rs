crate::ix!();

//-------------------------------------------------------------
// Implement Abbreviation
//-------------------------------------------------------------
impl Abbreviation for AustraliaOceaniaAntarcticaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            AustraliaOceaniaAntarcticaRegion::Australia       => "AU",
            AustraliaOceaniaAntarcticaRegion::AmericanOceania => "US-OC", // Arbitrary code
            AustraliaOceaniaAntarcticaRegion::CookIslands     => "CK",
            AustraliaOceaniaAntarcticaRegion::Fiji            => "FJ",
            AustraliaOceaniaAntarcticaRegion::IleDeClipperton => "CP",
            AustraliaOceaniaAntarcticaRegion::Kiribati        => "KI",
            AustraliaOceaniaAntarcticaRegion::MarshallIslands => "MH",
            AustraliaOceaniaAntarcticaRegion::Micronesia      => "FM",
            AustraliaOceaniaAntarcticaRegion::Nauru           => "NR",
            AustraliaOceaniaAntarcticaRegion::NewCaledonia    => "NC",
            AustraliaOceaniaAntarcticaRegion::NewZealand      => "NZ",
            AustraliaOceaniaAntarcticaRegion::Niue            => "NU",
            AustraliaOceaniaAntarcticaRegion::Palau           => "PW",
            AustraliaOceaniaAntarcticaRegion::PapuaNewGuinea  => "PG",
            AustraliaOceaniaAntarcticaRegion::PitcairnIslands => "PN",
            AustraliaOceaniaAntarcticaRegion::FrenchPolynesia => "PF",
            AustraliaOceaniaAntarcticaRegion::Samoa           => "WS",
            AustraliaOceaniaAntarcticaRegion::SolomonIslands  => "SB",
            AustraliaOceaniaAntarcticaRegion::Tokelau         => "TK",
            AustraliaOceaniaAntarcticaRegion::Tonga           => "TO",
            AustraliaOceaniaAntarcticaRegion::Tuvalu          => "TV",
            AustraliaOceaniaAntarcticaRegion::Vanuatu         => "VU",
            AustraliaOceaniaAntarcticaRegion::WallisEtFutuna  => "WF",
            AustraliaOceaniaAntarcticaRegion::Antarctica      => "AQ",
        }
    }
}

impl TryFromAbbreviation for AustraliaOceaniaAntarcticaRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        let region = match abbr {
            "AU" => AustraliaOceaniaAntarcticaRegion::Australia,
            "US-OC" => AustraliaOceaniaAntarcticaRegion::AmericanOceania,
            "CK" => AustraliaOceaniaAntarcticaRegion::CookIslands,
            "FJ" => AustraliaOceaniaAntarcticaRegion::Fiji,
            "CP" => AustraliaOceaniaAntarcticaRegion::IleDeClipperton,
            "KI" => AustraliaOceaniaAntarcticaRegion::Kiribati,
            "MH" => AustraliaOceaniaAntarcticaRegion::MarshallIslands,
            "FM" => AustraliaOceaniaAntarcticaRegion::Micronesia,
            "NR" => AustraliaOceaniaAntarcticaRegion::Nauru,
            "NC" => AustraliaOceaniaAntarcticaRegion::NewCaledonia,
            "NZ" => AustraliaOceaniaAntarcticaRegion::NewZealand,
            "NU" => AustraliaOceaniaAntarcticaRegion::Niue,
            "PW" => AustraliaOceaniaAntarcticaRegion::Palau,
            "PG" => AustraliaOceaniaAntarcticaRegion::PapuaNewGuinea,
            "PN" => AustraliaOceaniaAntarcticaRegion::PitcairnIslands,
            "PF" => AustraliaOceaniaAntarcticaRegion::FrenchPolynesia,
            "WS" => AustraliaOceaniaAntarcticaRegion::Samoa,
            "SB" => AustraliaOceaniaAntarcticaRegion::SolomonIslands,
            "TK" => AustraliaOceaniaAntarcticaRegion::Tokelau,
            "TO" => AustraliaOceaniaAntarcticaRegion::Tonga,
            "TV" => AustraliaOceaniaAntarcticaRegion::Tuvalu,
            "VU" => AustraliaOceaniaAntarcticaRegion::Vanuatu,
            "WF" => AustraliaOceaniaAntarcticaRegion::WallisEtFutuna,
            "AQ" => AustraliaOceaniaAntarcticaRegion::Antarctica,
            _ => return Err(TryFromAbbreviationError::InvalidAbbreviation),
        };
        Ok(region)
    }
}

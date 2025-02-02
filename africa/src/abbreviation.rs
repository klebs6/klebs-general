crate::ix!();

//-------------------------------------------------------------
// Implement Abbreviation for AfricaRegion
//-------------------------------------------------------------
impl Abbreviation for AfricaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            AfricaRegion::Algeria                               => "DZ",
            AfricaRegion::Angola                                => "AO",
            AfricaRegion::Benin                                 => "BJ",
            AfricaRegion::Botswana                              => "BW",
            AfricaRegion::BurkinaFaso                           => "BF",
            AfricaRegion::Burundi                               => "BI",
            AfricaRegion::Cameroon                              => "CM",
            AfricaRegion::CanaryIslands                         => "IC", // not ISO official, using a known code for Canary Islands
            AfricaRegion::CapeVerde                             => "CV",
            AfricaRegion::CentralAfricanRepublic                => "CF",
            AfricaRegion::Chad                                  => "TD",
            AfricaRegion::Comores                               => "KM", // Comoros
            AfricaRegion::CongoRepublicBrazzaville              => "CG",
            AfricaRegion::CongoDemocraticRepublicKinshasa       => "CD",
            AfricaRegion::Djibouti                              => "DJ",
            AfricaRegion::Egypt                                 => "EG",
            AfricaRegion::EquatorialGuinea                      => "GQ",
            AfricaRegion::Eritrea                               => "ER",
            AfricaRegion::Ethiopia                              => "ET",
            AfricaRegion::Gabon                                 => "GA",
            AfricaRegion::Ghana                                 => "GH",
            AfricaRegion::Guinea                                => "GN",
            AfricaRegion::GuineaBissau                          => "GW",
            AfricaRegion::IvoryCoast                            => "CI",
            AfricaRegion::Kenya                                 => "KE",
            AfricaRegion::Lesotho                               => "LS",
            AfricaRegion::Liberia                               => "LR",
            AfricaRegion::Libya                                 => "LY",
            AfricaRegion::Madagascar                            => "MG",
            AfricaRegion::Malawi                                => "MW",
            AfricaRegion::Mali                                  => "ML",
            AfricaRegion::Mauritania                            => "MR",
            AfricaRegion::Mauritius                             => "MU",
            AfricaRegion::Morocco                               => "MA",
            AfricaRegion::Mozambique                            => "MZ",
            AfricaRegion::Namibia                               => "NA",
            AfricaRegion::Niger                                 => "NE",
            AfricaRegion::Nigeria                               => "NG",
            AfricaRegion::Rwanda                                => "RW",
            AfricaRegion::SaintHelenaAscensionTristanDaCunha    => "SH-AC-TA", // combined territory, no single country code
            AfricaRegion::SaoTomeAndPrincipe                    => "ST",
            AfricaRegion::SenegalAndGambia                      => "SN-GM", // combined region
            AfricaRegion::Seychelles                            => "SC",
            AfricaRegion::SierraLeone                           => "SL",
            AfricaRegion::Somalia                               => "SO",
            AfricaRegion::SouthAfrica                           => "ZA",
            AfricaRegion::SouthSudan                            => "SS",
            AfricaRegion::Sudan                                 => "SD",
            AfricaRegion::Swaziland                             => "SZ", // Eswatini (SZ)
            AfricaRegion::Tanzania                              => "TZ",
            AfricaRegion::Togo                                  => "TG",
            AfricaRegion::Tunisia                               => "TN",
            AfricaRegion::Uganda                                => "UG",
            AfricaRegion::Zambia                                => "ZM",
            AfricaRegion::Zimbabwe                              => "ZW",
        }
    }
}

impl TryFromAbbreviation for AfricaRegion {
    type Error = TryFromAbbreviationError;

    /// Attempts to parse a short code like `"EG"` (Egypt) into an `AfricaRegion`.
    /// If the abbreviation is unrecognized, returns
    /// `Err(TryFromAbbreviationError::InvalidAbbreviation)`.
    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        match abbr {
            "DZ" => Ok(AfricaRegion::Algeria),
            "AO" => Ok(AfricaRegion::Angola),
            "BJ" => Ok(AfricaRegion::Benin),
            "BW" => Ok(AfricaRegion::Botswana),
            "BF" => Ok(AfricaRegion::BurkinaFaso),
            "BI" => Ok(AfricaRegion::Burundi),
            "CM" => Ok(AfricaRegion::Cameroon),
            "IC" => Ok(AfricaRegion::CanaryIslands),
            "CV" => Ok(AfricaRegion::CapeVerde),
            "CF" => Ok(AfricaRegion::CentralAfricanRepublic),
            "TD" => Ok(AfricaRegion::Chad),
            "KM" => Ok(AfricaRegion::Comores),
            "CG" => Ok(AfricaRegion::CongoRepublicBrazzaville),
            "CD" => Ok(AfricaRegion::CongoDemocraticRepublicKinshasa),
            "DJ" => Ok(AfricaRegion::Djibouti),
            "EG" => Ok(AfricaRegion::Egypt),
            "GQ" => Ok(AfricaRegion::EquatorialGuinea),
            "ER" => Ok(AfricaRegion::Eritrea),
            "ET" => Ok(AfricaRegion::Ethiopia),
            "GA" => Ok(AfricaRegion::Gabon),
            "GH" => Ok(AfricaRegion::Ghana),
            "GN" => Ok(AfricaRegion::Guinea),
            "GW" => Ok(AfricaRegion::GuineaBissau),
            "CI" => Ok(AfricaRegion::IvoryCoast),
            "KE" => Ok(AfricaRegion::Kenya),
            "LS" => Ok(AfricaRegion::Lesotho),
            "LR" => Ok(AfricaRegion::Liberia),
            "LY" => Ok(AfricaRegion::Libya),
            "MG" => Ok(AfricaRegion::Madagascar),
            "MW" => Ok(AfricaRegion::Malawi),
            "ML" => Ok(AfricaRegion::Mali),
            "MR" => Ok(AfricaRegion::Mauritania),
            "MU" => Ok(AfricaRegion::Mauritius),
            "MA" => Ok(AfricaRegion::Morocco),
            "MZ" => Ok(AfricaRegion::Mozambique),
            "NA" => Ok(AfricaRegion::Namibia),
            "NE" => Ok(AfricaRegion::Niger),
            "NG" => Ok(AfricaRegion::Nigeria),
            "RW" => Ok(AfricaRegion::Rwanda),
            "SH-AC-TA" => Ok(AfricaRegion::SaintHelenaAscensionTristanDaCunha),
            "ST" => Ok(AfricaRegion::SaoTomeAndPrincipe),
            "SN-GM" => Ok(AfricaRegion::SenegalAndGambia),
            "SC" => Ok(AfricaRegion::Seychelles),
            "SL" => Ok(AfricaRegion::SierraLeone),
            "SO" => Ok(AfricaRegion::Somalia),
            "ZA" => Ok(AfricaRegion::SouthAfrica),
            "SS" => Ok(AfricaRegion::SouthSudan),
            "SD" => Ok(AfricaRegion::Sudan),
            "SZ" => Ok(AfricaRegion::Swaziland),
            "TZ" => Ok(AfricaRegion::Tanzania),
            "TG" => Ok(AfricaRegion::Togo),
            "TN" => Ok(AfricaRegion::Tunisia),
            "UG" => Ok(AfricaRegion::Uganda),
            "ZM" => Ok(AfricaRegion::Zambia),
            "ZW" => Ok(AfricaRegion::Zimbabwe),
            _ => Err(TryFromAbbreviationError::InvalidAbbreviation),
        }
    }
}

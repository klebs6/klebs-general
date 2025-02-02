crate::ix!();

//-------------------------------------------------------------
// Implement Abbreviation for CentralAmericaRegion
//-------------------------------------------------------------
impl Abbreviation for CentralAmericaRegion {
    fn abbreviation(&self) -> &'static str {
        match self {
            CentralAmericaRegion::Bahamas                   => "BS",
            CentralAmericaRegion::Belize                    => "BZ",
            CentralAmericaRegion::CostaRica                 => "CR",
            CentralAmericaRegion::Cuba                      => "CU",
            CentralAmericaRegion::ElSalvador                => "SV",
            CentralAmericaRegion::Guatemala                 => "GT",
            CentralAmericaRegion::HaitiAndDominicanRepublic => "HT-DO", // Haiti (HT) and Dominican Republic (DO) combined
            CentralAmericaRegion::Honduras                  => "HN",
            CentralAmericaRegion::Jamaica                   => "JM",
            CentralAmericaRegion::Nicaragua                 => "NI",
            CentralAmericaRegion::Panama                    => "PA",
        }
    }
}

impl TryFromAbbreviation for CentralAmericaRegion {
    type Error = TryFromAbbreviationError;

    fn try_from_abbreviation(abbr: &str) -> Result<Self, Self::Error> {
        let region = match abbr {
            "BS" => CentralAmericaRegion::Bahamas,
            "BZ" => CentralAmericaRegion::Belize,
            "CR" => CentralAmericaRegion::CostaRica,
            "CU" => CentralAmericaRegion::Cuba,
            "SV" => CentralAmericaRegion::ElSalvador,
            "GT" => CentralAmericaRegion::Guatemala,
            "HT-DO" => CentralAmericaRegion::HaitiAndDominicanRepublic,
            "HN" => CentralAmericaRegion::Honduras,
            "JM" => CentralAmericaRegion::Jamaica,
            "NI" => CentralAmericaRegion::Nicaragua,
            "PA" => CentralAmericaRegion::Panama,
            _ => return Err(TryFromAbbreviationError::InvalidAbbreviation),
        };
        Ok(region)
    }
}

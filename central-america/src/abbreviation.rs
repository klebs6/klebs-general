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

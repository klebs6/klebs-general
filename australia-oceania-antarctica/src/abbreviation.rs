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

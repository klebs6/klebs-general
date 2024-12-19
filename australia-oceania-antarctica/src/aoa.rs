crate::ix!();

//-------------------------------------------------------------
// AustraliaOceaniaAntarcticaRegion Enum
//
// No subdivided regions. All top-level. Some are dependent territories.
// We will map as many as possible to Country variants, and unsupported
// regions will return errors on conversion.
// Default will be Australia.
//-------------------------------------------------------------
#[derive(Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum AustraliaOceaniaAntarcticaRegion {
    #[default]
    Australia,
    AmericanOceania,  // Combined/ambiguous region (no direct country)
    CookIslands,       // Territory in free association with NZ
    Fiji,
    #[strum(serialize = "Île de Clipperton")]
    IleDeClipperton,   // Territory of France
    Kiribati,
    MarshallIslands,
    Micronesia,
    Nauru,
    NewCaledonia,      // French territory
    NewZealand,
    Niue,              // Associated state of New Zealand
    Palau,
    PapuaNewGuinea,
    PitcairnIslands,   // British territory
    #[strum(serialize = "Polynésie française (French Polynesia)")]
    FrenchPolynesia,   // French territory
    Samoa,
    SolomonIslands,
    Tokelau,           // NZ territory
    Tonga,
    Tuvalu,
    Vanuatu,
    #[strum(serialize = "Wallis et Futuna")]
    WallisEtFutuna,    // French territory
    Antarctica,
}

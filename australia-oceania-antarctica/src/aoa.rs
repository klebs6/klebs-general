crate::ix!();

//-------------------------------------------------------------
// AustraliaOceaniaAntarcticaRegion Enum
//
// No subdivided regions. All top-level. Some are dependent territories.
// We will map as many as possible to Country variants, and unsupported
// regions will return errors on conversion.
// Default will be Australia.
//-------------------------------------------------------------
#[derive(FileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum AustraliaOceaniaAntarcticaRegion {

    #[default]
    #[download_link("https://download.geofabrik.de/australia-oceania/australia-latest.osm.pbf")]
    Australia,

    // Combined/ambiguous region (no direct country)
    #[download_link("https://download.geofabrik.de/australia-oceania/american-oceania-latest.osm.pbf")]
    AmericanOceania,  

    // Territory in free association with NZ
    #[download_link("https://download.geofabrik.de/australia-oceania/cook-islands-latest.osm.pbf")]
    CookIslands,       

    #[download_link("https://download.geofabrik.de/australia-oceania/fiji-latest.osm.pbf")]
    Fiji,

    #[strum(serialize = "Île de Clipperton")]
    #[download_link("https://download.geofabrik.de/australia-oceania/ile-de-clipperton-latest.osm.pbf")]
    IleDeClipperton,   // Territory of France

    #[download_link("https://download.geofabrik.de/australia-oceania/kiribati-latest.osm.pbf")]
    Kiribati,

    #[download_link("https://download.geofabrik.de/australia-oceania/marshall-islands-latest.osm.pbf")]
    MarshallIslands,

    #[download_link("https://download.geofabrik.de/australia-oceania/micronesia-latest.osm.pbf")]
    Micronesia,

    #[download_link("https://download.geofabrik.de/australia-oceania/nauru-latest.osm.pbf")]
    Nauru,

    // French territory
    #[download_link("https://download.geofabrik.de/australia-oceania/new-caledonia-latest.osm.pbf")]
    NewCaledonia,      

    #[download_link("https://download.geofabrik.de/australia-oceania/new-zealand-latest.osm.pbf")]
    NewZealand,

    // Associated state of New Zealand
    #[download_link("https://download.geofabrik.de/australia-oceania/niue-latest.osm.pbf")]
    Niue,              

    #[download_link("https://download.geofabrik.de/australia-oceania/palau-latest.osm.pbf")]
    Palau,

    #[download_link("https://download.geofabrik.de/australia-oceania/papua-new-guinea-latest.osm.pbf")]
    PapuaNewGuinea,

    // British territory
    #[download_link("https://download.geofabrik.de/australia-oceania/pitcairn-islands-latest.osm.pbf")]
    PitcairnIslands,   

    // French territory
    #[strum(serialize = "Polynésie française (French Polynesia)")]
    #[download_link("https://download.geofabrik.de/australia-oceania/polynesie-francaise-latest.osm.pbf")]
    FrenchPolynesia,   

    #[download_link("https://download.geofabrik.de/australia-oceania/samoa-latest.osm.pbf")]
    Samoa,

    #[download_link("https://download.geofabrik.de/australia-oceania/solomon-islands-latest.osm.pbf")]
    SolomonIslands,

    // NZ territory
    #[download_link("https://download.geofabrik.de/australia-oceania/tokelau-latest.osm.pbf")]
    Tokelau,           

    #[download_link("https://download.geofabrik.de/australia-oceania/tonga-latest.osm.pbf")]
    Tonga,

    #[download_link("https://download.geofabrik.de/australia-oceania/tuvalu-latest.osm.pbf")]
    Tuvalu,

    #[download_link("https://download.geofabrik.de/australia-oceania/vanuatu-latest.osm.pbf")]
    Vanuatu,

    // French territory
    #[strum(serialize = "Wallis et Futuna")]
    #[download_link("https://download.geofabrik.de/australia-oceania/wallis-et-futuna-latest.osm.pbf")]
    WallisEtFutuna,    

    #[download_link("https://download.geofabrik.de/antarctica-latest.osm.pbf")]
    Antarctica,
}

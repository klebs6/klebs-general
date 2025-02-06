crate::ix!();

//-------------------------------------------------------------
// AustraliaOceaniaAntarcticaRegion Enum
//
// No subdivided regions. All top-level. Some are dependent territories.
// We will map as many as possible to Country variants, and unsupported
// regions will return errors on conversion.
// Default will be Australia.
//-------------------------------------------------------------
#[derive(OsmPbfFileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum AustraliaOceaniaAntarcticaRegion {

    #[default]
    #[geofabrik(aoa="australia-latest.osm.pbf")]
    Australia,

    // Combined/ambiguous region (no direct country)
    #[geofabrik(aoa="american-oceania-latest.osm.pbf")]
    AmericanOceania,  

    // Territory in free association with NZ
    #[geofabrik(aoa="cook-islands-latest.osm.pbf")]
    CookIslands,       

    #[geofabrik(aoa="fiji-latest.osm.pbf")]
    Fiji,

    #[strum(serialize = "Île de Clipperton")]
    #[geofabrik(aoa="ile-de-clipperton-latest.osm.pbf")]
    IleDeClipperton,   // Territory of France

    #[geofabrik(aoa="kiribati-latest.osm.pbf")]
    Kiribati,

    #[geofabrik(aoa="marshall-islands-latest.osm.pbf")]
    MarshallIslands,

    #[geofabrik(aoa="micronesia-latest.osm.pbf")]
    Micronesia,

    #[geofabrik(aoa="nauru-latest.osm.pbf")]
    Nauru,

    // French territory
    #[geofabrik(aoa="new-caledonia-latest.osm.pbf")]
    NewCaledonia,      

    #[geofabrik(aoa="new-zealand-latest.osm.pbf")]
    NewZealand,

    // Associated state of New Zealand
    #[geofabrik(aoa="niue-latest.osm.pbf")]
    Niue,              

    #[geofabrik(aoa="palau-latest.osm.pbf")]
    Palau,

    #[geofabrik(aoa="papua-new-guinea-latest.osm.pbf")]
    PapuaNewGuinea,

    // British territory
    #[geofabrik(aoa="pitcairn-islands-latest.osm.pbf")]
    PitcairnIslands,   

    // French territory
    #[strum(serialize = "Polynésie française (French Polynesia)")]
    #[geofabrik(aoa="polynesie-francaise-latest.osm.pbf")]
    FrenchPolynesia,   

    #[geofabrik(aoa="samoa-latest.osm.pbf")]
    Samoa,

    #[geofabrik(aoa="solomon-islands-latest.osm.pbf")]
    SolomonIslands,

    // NZ territory
    #[geofabrik(aoa="tokelau-latest.osm.pbf")]
    Tokelau,           

    #[geofabrik(aoa="tonga-latest.osm.pbf")]
    Tonga,

    #[geofabrik(aoa="tuvalu-latest.osm.pbf")]
    Tuvalu,

    #[geofabrik(aoa="vanuatu-latest.osm.pbf")]
    Vanuatu,

    // French territory
    #[strum(serialize = "Wallis et Futuna")]
    #[geofabrik(aoa="wallis-et-futuna-latest.osm.pbf")]
    WallisEtFutuna,    

    #[geofabrik(antarctica="antarctica-latest.osm.pbf")]
    Antarctica,
}

crate::ix!();

//-------------------------------------------------------------
// AfricaRegion Enum
//
// Since the user provided a list of African regions/countries
// and stated that there are no subdivided regions, everything
// is top-level. We will treat special combined regions similarly
// to how "GccStates" or "MalaysiaSingaporeBrunei" was handled
// in the Asia example.
//
// Default will be Algeria.
//-------------------------------------------------------------
#[derive(OsmPbfFileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum AfricaRegion {

    #[default]
    #[geofabrik(africa="algeria-latest.osm.pbf")]
    Algeria,

    #[geofabrik(africa="angola-latest.osm.pbf")]
    Angola,

    #[geofabrik(africa="benin-latest.osm.pbf")]
    Benin,

    #[geofabrik(africa="botswana-latest.osm.pbf")]
    Botswana,

    #[geofabrik(africa="burkina-faso-latest.osm.pbf")]
    BurkinaFaso,

    #[geofabrik(africa="burundi-latest.osm.pbf")]
    Burundi,

    #[geofabrik(africa="cameroon-latest.osm.pbf")]
    Cameroon,

    #[geofabrik(africa="canary-islands-latest.osm.pbf")]
    CanaryIslands, // territory of Spain (not a sovereign country)

    #[geofabrik(africa="cape-verde-latest.osm.pbf")]
    CapeVerde,

    #[geofabrik(africa="central-african-republic-latest.osm.pbf")]
    CentralAfricanRepublic,

    #[geofabrik(africa="chad-latest.osm.pbf")]
    Chad,

    #[geofabrik(africa="comores-latest.osm.pbf")]
    #[strum(serialize = "Comoros")]
    Comores,

    #[geofabrik(africa="congo-brazzaville-latest.osm.pbf")]
    CongoRepublicBrazzaville,

    #[strum(serialize = "Congo (Democratic Republic/Kinshasa)")]
    #[geofabrik(africa="congo-democratic-republic-latest.osm.pbf")]
    CongoDemocraticRepublicKinshasa,

    #[geofabrik(africa="djibouti-latest.osm.pbf")]
    Djibouti,

    #[geofabrik(africa="egypt-latest.osm.pbf")]
    Egypt,

    #[geofabrik(africa="equatorial-guinea-latest.osm.pbf")]
    EquatorialGuinea,

    #[geofabrik(africa="eritrea-latest.osm.pbf")]
    Eritrea,

    #[geofabrik(africa="ethiopia-latest.osm.pbf")]
    Ethiopia,

    #[geofabrik(africa="gabon-latest.osm.pbf")]
    Gabon,

    #[geofabrik(africa="ghana-latest.osm.pbf")]
    Ghana,

    #[geofabrik(africa="guinea-latest.osm.pbf")]
    Guinea,

    #[geofabrik(africa="guinea-bissau-latest.osm.pbf")]
    GuineaBissau,

    #[geofabrik(africa="ivory-coast-latest.osm.pbf")]
    IvoryCoast,

    #[geofabrik(africa="kenya-latest.osm.pbf")]
    Kenya,

    #[geofabrik(africa="lesotho-latest.osm.pbf")]
    Lesotho,

    #[geofabrik(africa="liberia-latest.osm.pbf")]
    Liberia,

    #[geofabrik(africa="libya-latest.osm.pbf")]
    Libya,

    #[geofabrik(africa="madagascar-latest.osm.pbf")]
    Madagascar,

    #[geofabrik(africa="malawi-latest.osm.pbf")]
    Malawi,

    #[geofabrik(africa="mali-latest.osm.pbf")]
    Mali,

    #[geofabrik(africa="mauritania-latest.osm.pbf")]
    Mauritania,

    #[geofabrik(africa="mauritius-latest.osm.pbf")]
    Mauritius,

    #[geofabrik(africa="morocco-latest.osm.pbf")]
    Morocco,

    #[geofabrik(africa="mozambique-latest.osm.pbf")]
    Mozambique,

    #[geofabrik(africa="namibia-latest.osm.pbf")]
    Namibia,

    #[geofabrik(africa="niger-latest.osm.pbf")]
    Niger,

    #[geofabrik(africa="nigeria-latest.osm.pbf")]
    Nigeria,

    #[geofabrik(africa="rwanda-latest.osm.pbf")]
    Rwanda,

    #[geofabrik(africa="saint-helena-ascension-and-tristan-da-cunha-latest.osm.pbf")]
    SaintHelenaAscensionTristanDaCunha,

    #[geofabrik(africa="sao-tome-and-principe-latest.osm.pbf")]
    SaoTomeAndPrincipe,

    #[strum(serialize = "Senegal and Gambia")]
    #[geofabrik(africa="senegal-and-gambia-latest.osm.pbf")]
    SenegalAndGambia,

    #[geofabrik(africa="seychelles-latest.osm.pbf")]
    Seychelles,

    #[geofabrik(africa="sierra-leone-latest.osm.pbf")]
    SierraLeone,

    #[geofabrik(africa="somalia-latest.osm.pbf")]
    Somalia,

    #[geofabrik(africa="south-africa-latest.osm.pbf")]
    SouthAfrica,

    #[geofabrik(africa="south-sudan-latest.osm.pbf")]
    SouthSudan,

    #[geofabrik(africa="sudan-latest.osm.pbf")]
    Sudan,

    #[geofabrik(africa="swaziland-latest.osm.pbf")]
    #[strum(serialize = "Eswatini")]
    Swaziland,

    #[geofabrik(africa="tanzania-latest.osm.pbf")]
    Tanzania,

    #[geofabrik(africa="togo-latest.osm.pbf")]
    Togo,

    #[geofabrik(africa="tunisia-latest.osm.pbf")]
    Tunisia,

    #[geofabrik(africa="uganda-latest.osm.pbf")]
    Uganda,

    #[geofabrik(africa="zambia-latest.osm.pbf")]
    Zambia,

    #[geofabrik(africa="zimbabwe-latest.osm.pbf")]
    Zimbabwe,
}

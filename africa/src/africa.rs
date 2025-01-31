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
#[derive(FileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum AfricaRegion {

    #[default]
    #[download_link("https://download.geofabrik.de/africa/algeria-latest.osm.pbf")]
    Algeria,

    #[download_link("https://download.geofabrik.de/africa/angola-latest.osm.pbf")]
    Angola,

    #[download_link("https://download.geofabrik.de/africa/benin-latest.osm.pbf")]
    Benin,

    #[download_link("https://download.geofabrik.de/africa/botswana-latest.osm.pbf")]
    Botswana,

    #[download_link("https://download.geofabrik.de/africa/burkina-faso-latest.osm.pbf")]
    BurkinaFaso,

    #[download_link("https://download.geofabrik.de/africa/burundi-latest.osm.pbf")]
    Burundi,

    #[download_link("https://download.geofabrik.de/africa/cameroon-latest.osm.pbf")]
    Cameroon,

    #[download_link("https://download.geofabrik.de/africa/canary-islands-latest.osm.pbf")]
    CanaryIslands, // territory of Spain (not a sovereign country)

    #[download_link("https://download.geofabrik.de/africa/cape-verde-latest.osm.pbf")]
    CapeVerde,

    #[download_link("https://download.geofabrik.de/africa/central-african-republic-latest.osm.pbf")]
    CentralAfricanRepublic,

    #[download_link("https://download.geofabrik.de/africa/chad-latest.osm.pbf")]
    Chad,

    #[download_link("https://download.geofabrik.de/africa/comores-latest.osm.pbf")]
    #[strum(serialize = "Comoros")]
    Comores,

    #[download_link("https://download.geofabrik.de/africa/congo-brazzaville-latest.osm.pbf")]
    CongoRepublicBrazzaville,

    #[strum(serialize = "Congo (Democratic Republic/Kinshasa)")]
    #[download_link("https://download.geofabrik.de/africa/congo-democratic-republic-latest.osm.pbf")]
    CongoDemocraticRepublicKinshasa,

    #[download_link("https://download.geofabrik.de/africa/djibouti-latest.osm.pbf")]
    Djibouti,

    #[download_link("https://download.geofabrik.de/africa/egypt-latest.osm.pbf")]
    Egypt,

    #[download_link("https://download.geofabrik.de/africa/equatorial-guinea-latest.osm.pbf")]
    EquatorialGuinea,

    #[download_link("https://download.geofabrik.de/africa/eritrea-latest.osm.pbf")]
    Eritrea,

    #[download_link("https://download.geofabrik.de/africa/ethiopia-latest.osm.pbf")]
    Ethiopia,

    #[download_link("https://download.geofabrik.de/africa/gabon-latest.osm.pbf")]
    Gabon,

    #[download_link("https://download.geofabrik.de/africa/ghana-latest.osm.pbf")]
    Ghana,

    #[download_link("https://download.geofabrik.de/africa/guinea-latest.osm.pbf")]
    Guinea,

    #[download_link("https://download.geofabrik.de/africa/guinea-bissau-latest.osm.pbf")]
    GuineaBissau,

    #[download_link("https://download.geofabrik.de/africa/ivory-coast-latest.osm.pbf")]
    IvoryCoast,

    #[download_link("https://download.geofabrik.de/africa/kenya-latest.osm.pbf")]
    Kenya,

    #[download_link("https://download.geofabrik.de/africa/lesotho-latest.osm.pbf")]
    Lesotho,

    #[download_link("https://download.geofabrik.de/africa/liberia-latest.osm.pbf")]
    Liberia,

    #[download_link("https://download.geofabrik.de/africa/libya-latest.osm.pbf")]
    Libya,

    #[download_link("https://download.geofabrik.de/africa/madagascar-latest.osm.pbf")]
    Madagascar,

    #[download_link("https://download.geofabrik.de/africa/malawi-latest.osm.pbf")]
    Malawi,

    #[download_link("https://download.geofabrik.de/africa/mali-latest.osm.pbf")]
    Mali,

    #[download_link("https://download.geofabrik.de/africa/mauritania-latest.osm.pbf")]
    Mauritania,

    #[download_link("https://download.geofabrik.de/africa/mauritius-latest.osm.pbf")]
    Mauritius,

    #[download_link("https://download.geofabrik.de/africa/morocco-latest.osm.pbf")]
    Morocco,

    #[download_link("https://download.geofabrik.de/africa/mozambique-latest.osm.pbf")]
    Mozambique,

    #[download_link("https://download.geofabrik.de/africa/namibia-latest.osm.pbf")]
    Namibia,

    #[download_link("https://download.geofabrik.de/africa/niger-latest.osm.pbf")]
    Niger,

    #[download_link("https://download.geofabrik.de/africa/nigeria-latest.osm.pbf")]
    Nigeria,

    #[download_link("https://download.geofabrik.de/africa/rwanda-latest.osm.pbf")]
    Rwanda,

    #[download_link("https://download.geofabrik.de/africa/saint-helena-ascension-and-tristan-da-cunha-latest.osm.pbf")]
    SaintHelenaAscensionTristanDaCunha,

    #[download_link("https://download.geofabrik.de/africa/sao-tome-and-principe-latest.osm.pbf")]
    SaoTomeAndPrincipe,

    #[strum(serialize = "Senegal and Gambia")]
    #[download_link("https://download.geofabrik.de/africa/senegal-and-gambia-latest.osm.pbf")]
    SenegalAndGambia,

    #[download_link("https://download.geofabrik.de/africa/seychelles-latest.osm.pbf")]
    Seychelles,

    #[download_link("https://download.geofabrik.de/africa/sierra-leone-latest.osm.pbf")]
    SierraLeone,

    #[download_link("https://download.geofabrik.de/africa/somalia-latest.osm.pbf")]
    Somalia,

    #[download_link("https://download.geofabrik.de/africa/south-africa-latest.osm.pbf")]
    SouthAfrica,

    #[download_link("https://download.geofabrik.de/africa/south-sudan-latest.osm.pbf")]
    SouthSudan,

    #[download_link("https://download.geofabrik.de/africa/sudan-latest.osm.pbf")]
    Sudan,

    #[download_link("https://download.geofabrik.de/africa/swaziland-latest.osm.pbf")]
    #[strum(serialize = "Eswatini")]
    Swaziland,

    #[download_link("https://download.geofabrik.de/africa/tanzania-latest.osm.pbf")]
    Tanzania,

    #[download_link("https://download.geofabrik.de/africa/togo-latest.osm.pbf")]
    Togo,

    #[download_link("https://download.geofabrik.de/africa/tunisia-latest.osm.pbf")]
    Tunisia,

    #[download_link("https://download.geofabrik.de/africa/uganda-latest.osm.pbf")]
    Uganda,

    #[download_link("https://download.geofabrik.de/africa/zambia-latest.osm.pbf")]
    Zambia,

    #[download_link("https://download.geofabrik.de/africa/zimbabwe-latest.osm.pbf")]
    Zimbabwe,
}

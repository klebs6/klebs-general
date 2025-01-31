crate::ix!();

//-------------------------------------------------------------
// Canada Regions
//-------------------------------------------------------------
#[derive(FileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum CanadaRegion {
    #[strum(serialize = "Alberta")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/alberta-latest.osm.pbf")]
    Alberta,

    #[strum(serialize = "British Columbia")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/british-columbia-latest.osm.pbf")]
    BritishColumbia,

    #[strum(serialize = "Manitoba")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/manitoba-latest.osm.pbf")]
    Manitoba,

    #[strum(serialize = "New Brunswick")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/new-brunswick-latest.osm.pbf")]
    NewBrunswick,

    #[strum(serialize = "Newfoundland and Labrador")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/newfoundland-and-labrador-latest.osm.pbf")]
    NewfoundlandAndLabrador,

    #[strum(serialize = "Northwest Territories")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/northwest-territories-latest.osm.pbf")]
    NorthwestTerritories,

    #[strum(serialize = "Nova Scotia")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/nova-scotia-latest.osm.pbf")]
    NovaScotia,

    #[strum(serialize = "Nunavut")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/nunavut-latest.osm.pbf")]
    Nunavut,

    #[default]
    #[strum(serialize = "Ontario")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/ontario-latest.osm.pbf")]
    Ontario,

    #[strum(serialize = "Prince Edward Island")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/prince-edward-island-latest.osm.pbf")]
    PrinceEdwardIsland,

    #[strum(serialize = "Quebec")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/quebec-latest.osm.pbf")]
    Quebec,

    #[strum(serialize = "Saskatchewan")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/saskatchewan-latest.osm.pbf")]
    Saskatchewan,

    #[strum(serialize = "Yukon")] 
    #[download_link("https://download.geofabrik.de/north-america/canada/yukon-latest.osm.pbf")]
    Yukon,
}

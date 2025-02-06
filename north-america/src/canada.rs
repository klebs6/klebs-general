crate::ix!();

//-------------------------------------------------------------
// Canada Regions
//-------------------------------------------------------------
#[derive(OsmPbfFileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum CanadaRegion {
    #[strum(serialize = "Alberta")] 
    #[geofabrik(canada="alberta-latest.osm.pbf")]
    Alberta,

    #[strum(serialize = "British Columbia")] 
    #[geofabrik(canada="british-columbia-latest.osm.pbf")]
    BritishColumbia,

    #[strum(serialize = "Manitoba")] 
    #[geofabrik(canada="manitoba-latest.osm.pbf")]
    Manitoba,

    #[strum(serialize = "New Brunswick")] 
    #[geofabrik(canada="new-brunswick-latest.osm.pbf")]
    NewBrunswick,

    #[strum(serialize = "Newfoundland and Labrador")] 
    #[geofabrik(canada="newfoundland-and-labrador-latest.osm.pbf")]
    NewfoundlandAndLabrador,

    #[strum(serialize = "Northwest Territories")] 
    #[geofabrik(canada="northwest-territories-latest.osm.pbf")]
    NorthwestTerritories,

    #[strum(serialize = "Nova Scotia")] 
    #[geofabrik(canada="nova-scotia-latest.osm.pbf")]
    NovaScotia,

    #[strum(serialize = "Nunavut")] 
    #[geofabrik(canada="nunavut-latest.osm.pbf")]
    Nunavut,

    #[default]
    #[strum(serialize = "Ontario")] 
    #[geofabrik(canada="ontario-latest.osm.pbf")]
    Ontario,

    #[strum(serialize = "Prince Edward Island")] 
    #[geofabrik(canada="prince-edward-island-latest.osm.pbf")]
    PrinceEdwardIsland,

    #[strum(serialize = "Quebec")] 
    #[geofabrik(canada="quebec-latest.osm.pbf")]
    Quebec,

    #[strum(serialize = "Saskatchewan")] 
    #[geofabrik(canada="saskatchewan-latest.osm.pbf")]
    Saskatchewan,

    #[strum(serialize = "Yukon")] 
    #[geofabrik(canada="yukon-latest.osm.pbf")]
    Yukon,
}

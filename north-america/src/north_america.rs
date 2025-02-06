crate::ix!();

//-------------------------------------------------------------
// NorthAmericaRegion Enum
//-------------------------------------------------------------
#[derive(OsmPbfFileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames)]
#[strum(ascii_case_insensitive)]
pub enum NorthAmericaRegion {
    Canada(CanadaRegion),

    #[default]
    #[geofabrik(north_america="greenland-latest.osm.pbf")]
    Greenland,

    #[geofabrik(north_america="mexico-latest.osm.pbf")]
    Mexico,

    UnitedStates(USRegion),
}

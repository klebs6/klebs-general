crate::ix!();

//-------------------------------------------------------------
// NorthAmericaRegion Enum
//-------------------------------------------------------------
#[derive(FileDownloader,Default,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames)]
#[strum(ascii_case_insensitive)]
pub enum NorthAmericaRegion {
    Canada(CanadaRegion),

    #[default]
    #[download_link("https://download.geofabrik.de/north-america/greenland-latest.osm.pbf")]
    Greenland,

    #[download_link("https://download.geofabrik.de/north-america/mexico-latest.osm.pbf")]
    Mexico,

    UnitedStates(USRegion),
}

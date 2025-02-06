crate::ix!();

//-------------------------------------------------------------
// CentralAmericaRegion Enum
//-------------------------------------------------------------
// Haiti and Dominican Republic is a combined region.
// By analogy with Europe and Asia crates, we will map this combined region to one country by convention.
// We'll pick Haiti as the representative country.
// From Country::Haiti or Country::DominicanRepublic we will map back to the combined region.
#[derive(OsmPbfFileDownloader,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum CentralAmericaRegion {

    #[geofabrik(central_america="bahamas-latest.osm.pbf")]
    Bahamas,

    #[geofabrik(central_america="belize-latest.osm.pbf")]
    Belize,

    #[geofabrik(central_america="costa-rica-latest.osm.pbf")]
    CostaRica,

    #[geofabrik(central_america="cuba-latest.osm.pbf")]
    Cuba,

    #[geofabrik(central_america="el-salvador-latest.osm.pbf")]
    ElSalvador,

    #[geofabrik(central_america="guatemala-latest.osm.pbf")]
    Guatemala,

    #[geofabrik(central_america="haiti-and-domrep-latest.osm.pbf")]
    HaitiAndDominicanRepublic,

    #[geofabrik(central_america="honduras-latest.osm.pbf")]
    Honduras,

    #[geofabrik(central_america="jamaica-latest.osm.pbf")]
    Jamaica,

    #[geofabrik(central_america="nicaragua-latest.osm.pbf")]
    Nicaragua,

    #[geofabrik(central_america="panama-latest.osm.pbf")]
    Panama,
}

impl Default for CentralAmericaRegion {
    fn default() -> Self {
        // Arbitrarily pick Cuba as default
        CentralAmericaRegion::Cuba
    }
}

crate::ix!();

//-------------------------------------------------------------
// CentralAmericaRegion Enum
//-------------------------------------------------------------
// Haiti and Dominican Republic is a combined region.
// By analogy with Europe and Asia crates, we will map this combined region to one country by convention.
// We'll pick Haiti as the representative country.
// From Country::Haiti or Country::DominicanRepublic we will map back to the combined region.
#[derive(FileDownloader,Debug,PartialOrd,Ord,PartialEq,Eq,Hash,Clone,Copy,StrumDisplay,StrumEnumIter,StrumEnumVariantNames,StrumEnumString)]
#[strum(ascii_case_insensitive)]
pub enum CentralAmericaRegion {

    #[download_link("https://download.geofabrik.de/central-america/bahamas-latest.osm.pbf")]
    Bahamas,

    #[download_link("https://download.geofabrik.de/central-america/belize-latest.osm.pbf")]
    Belize,

    #[download_link("https://download.geofabrik.de/central-america/costa-rica-latest.osm.pbf")]
    CostaRica,

    #[download_link("https://download.geofabrik.de/central-america/cuba-latest.osm.pbf")]
    Cuba,

    #[download_link("https://download.geofabrik.de/central-america/el-salvador-latest.osm.pbf")]
    ElSalvador,

    #[download_link("https://download.geofabrik.de/central-america/guatemala-latest.osm.pbf")]
    Guatemala,

    #[download_link("https://download.geofabrik.de/central-america/haiti-and-domrep-latest.osm.pbf")]
    HaitiAndDominicanRepublic,

    #[download_link("https://download.geofabrik.de/central-america/honduras-latest.osm.pbf")]
    Honduras,

    #[download_link("https://download.geofabrik.de/central-america/jamaica-latest.osm.pbf")]
    Jamaica,

    #[download_link("https://download.geofabrik.de/central-america/nicaragua-latest.osm.pbf")]
    Nicaragua,

    #[download_link("https://download.geofabrik.de/central-america/panama-latest.osm.pbf")]
    Panama,
}

impl Default for CentralAmericaRegion {
    fn default() -> Self {
        // Arbitrarily pick Cuba as default
        CentralAmericaRegion::Cuba
    }
}

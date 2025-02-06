use osmpbf_file_downloader_derive::*;
use file_downloader::*;

// Example 1: simple enum with direct unit variants
#[derive(OsmPbfFileDownloader)]
pub enum SampleRegion {
    #[geofabrik(spain="valencia-latest.osm.pbf")]
    Valencia,
    #[geofabrik(north_america="mexico-latest.osm.pbf")]
    Mexico,
}

// Example 2: delegation to sub-enum
#[derive(OsmPbfFileDownloader)]
pub enum CompositeRegion {
    #[geofabrik(spain="madrid-latest.osm.pbf")]
    Madrid,
    SubRegion(OtherRegion),
}

#[derive(OsmPbfFileDownloader)]
pub enum OtherRegion {
    #[geofabrik(spain="asturias-latest.osm.pbf")]
    Asturias,
    #[geofabrik(poland="mazowieckie-latest.osm.pbf")]
    Mazowieckie,
}

#[test]
fn check_valencia_url() {
    let region = SampleRegion::Valencia;
    assert_eq!(
        "https://download.geofabrik.de/europe/spain/valencia-latest.osm.pbf",
        region.download_link()
    );
}

#[test]
fn check_mexico_url() {
    let region = SampleRegion::Mexico;
    assert_eq!(
        "https://download.geofabrik.de/north-america/mexico-latest.osm.pbf",
        region.download_link()
    );
}

#[test]
fn check_madrid_url() {
    let region = CompositeRegion::Madrid;
    assert_eq!(
        "https://download.geofabrik.de/europe/spain/madrid-latest.osm.pbf",
        region.download_link()
    );
}

#[test]
fn check_subregion_delegation() {
    // This variant delegates to `OtherRegion::Asturias`
    let region = CompositeRegion::SubRegion(OtherRegion::Asturias);
    assert_eq!(
        "https://download.geofabrik.de/europe/spain/asturias-latest.osm.pbf",
        region.download_link()
    );
}

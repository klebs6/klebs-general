// osmpbf-file-downloader-derive/tests/ui/ok_unit_variant_link.rs
use osmpbf_file_downloader_derive::OsmPbfFileDownloader;
use file_downloader::*;

#[derive(OsmPbfFileDownloader)]
enum MyEnum {
    #[geofabrik(spain="madrid-latest.osm.pbf")]
    VariantOne,

    // Could do multiple unit variants, each with its own link
    #[geofabrik(poland="mazowieckie-latest.osm.pbf")]
    VariantTwo,
}

fn main() {
    let v1 = MyEnum::VariantOne;
    assert_eq!(v1.download_link(), "https://download.geofabrik.de/europe/spain/madrid-latest.osm.pbf");

    let v2 = MyEnum::VariantTwo;
    assert_eq!(v2.download_link(), "https://download.geofabrik.de/europe/poland/mazowieckie-latest.osm.pbf");
}


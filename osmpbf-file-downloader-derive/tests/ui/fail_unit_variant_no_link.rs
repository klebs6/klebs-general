// osmpbf-file-downloader-derive/tests/ui/fail_unit_variant_no_link.rs
use osmpbf_file_downloader_derive::OsmPbfFileDownloader;

#[derive(OsmPbfFileDownloader)]
enum MyEnum {
    // This variant has no link attribute
    VariantOne,
}

fn main() {}


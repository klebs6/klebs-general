// osmpbf-file-downloader-derive/tests/ui/fail_multi_named_variant_no_link.rs
use osmpbf_file_downloader_derive::OsmPbfFileDownloader;

#[derive(OsmPbfFileDownloader)]
enum MyEnum {
    MultiField {
        a: u32,
        b: u32,
    },
}

fn main() {}


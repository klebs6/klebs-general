// osmpbf-file-downloader-derive/tests/ui/fail_multi_field_tuple_variant.rs
use osmpbf_file_downloader_derive::OsmPbfFileDownloader;

#[derive(OsmPbfFileDownloader)]
enum MyEnum {
    // Tuple variant with 2 fields -> should fail unless there's #[geofabrik(region = "")] to short-circuit
    TwoField(u32, u32),
}

fn main() {}

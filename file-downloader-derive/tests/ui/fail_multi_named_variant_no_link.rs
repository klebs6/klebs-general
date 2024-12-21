
// file-downloader-derive/tests/ui/fail_multi_named_variant_no_link.rs
use file_downloader::FileDownloader;
use file_downloader_derive::FileDownloader;

#[derive(FileDownloader)]
enum MyEnum {
    MultiField {
        a: u32,
        b: u32,
    },
}

fn main() {}


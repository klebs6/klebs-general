
// file-downloader-derive/tests/ui/fail_unit_variant_no_link.rs
use file_downloader::FileDownloader;
use file_downloader_derive::FileDownloader;

#[derive(FileDownloader)]
enum MyEnum {
    // This variant has no link attribute
    VariantOne,
}

fn main() {}


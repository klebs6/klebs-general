
// file-downloader-derive/tests/ui/fail_multi_field_tuple_variant.rs
use file_downloader::FileDownloader;
use file_downloader_derive::FileDownloader;

#[derive(FileDownloader)]
enum MyEnum {
    // Tuple variant with 2 fields -> should fail unless there's #[download_link = ""] to short-circuit
    TwoField(u32, u32),
}

fn main() {}

use file_downloader::{extract_md5_from_filename, filename_with_md5};
use std::path::PathBuf;

#[test]
fn test_extract_md5_from_filename() {
    let fname = "planet.abcdef1234567890abcdef1234567890.osm.pbf";
    let extracted = extract_md5_from_filename(fname);
    assert_eq!(extracted.as_deref(), Some("abcdef1234567890abcdef1234567890"));
}

#[test]
fn test_extract_md5_from_filename_no_md5() {
    let fname = "planet.osm.pbf";
    let extracted = extract_md5_from_filename(fname);
    assert!(extracted.is_none());
}

#[test]
fn test_filename_with_md5() {
    let filename = "planet.osm.pbf";
    let md5 = "abcdef1234567890abcdef1234567890";
    let result = filename_with_md5(filename, md5);
    assert_eq!(result, PathBuf::from("planet.abcdef1234567890abcdef1234567890.osm.pbf"));
}

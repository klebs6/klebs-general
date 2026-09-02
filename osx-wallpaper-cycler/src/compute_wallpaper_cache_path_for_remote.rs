// ---------------- [ File: osx-wallpaper-cycler/src/compute_wallpaper_cache_path_for_remote.rs ]
crate::ix!();

pub fn encode_bytes_as_lower_hex(bytes: &[u8]) -> String {
    const LUT: &[u8; 16] = b"0123456789abcdef";
    let mut out = String::with_capacity(bytes.len().saturating_mul(2));
    for &b in bytes.iter() {
        out.push(LUT[(b >> 4) as usize] as char);
        out.push(LUT[(b & 0x0f) as usize] as char);
    }
    out
}

pub fn compute_wallpaper_cache_path_for_remote(
    cache_dir: &std::path::Path,
    remote: &DropboxWallpaperCandidate,
) -> std::path::PathBuf {
    use sha2::Digest;

    let mut hasher = sha2::Sha256::new();
    hasher.update(remote.id().as_bytes());
    hasher.update(b"|");
    hasher.update(remote.path_lower().as_bytes());
    let digest = hasher.finalize();

    let hex = encode_bytes_as_lower_hex(digest.as_ref());

    let ext = remote
        .file_extension_lowercase()
        .unwrap_or_else(|| "img".to_string());

    cache_dir.join(format!("{hex}.{ext}"))
}

#[cfg(test)]
mod lower_hex_encoding_contract_suite {
    use super::*;

    #[traced_test]
    fn lower_hex_encoding_handles_empty_and_common_bytes() {
        assert_eq!(encode_bytes_as_lower_hex(&[]), "");
        assert_eq!(encode_bytes_as_lower_hex(&[0x00]), "00");
        assert_eq!(encode_bytes_as_lower_hex(&[0x0f]), "0f");
        assert_eq!(encode_bytes_as_lower_hex(&[0xff]), "ff");
        assert_eq!(encode_bytes_as_lower_hex(&[0x12, 0x34, 0xab, 0xcd, 0xef]), "1234abcdef");
    }
}

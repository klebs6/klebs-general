// ---------------- [ File: src/write_be_u32.rs ]
crate::ix!();

/// Writes `value` as a 4-byte big-endian integer into `file`.
pub async fn write_u32_be(file: &mut tokio::fs::File, value: u32) -> std::io::Result<()> {
    let mut buf = [0u8; 4];
    // Use byteorder to encode `value` in big-endian:
    byteorder::BigEndian::write_u32(&mut buf, value);
    // Now write those four bytes asynchronously:
    file.write_all(&buf).await
}


// ---------------- [ File: src/write_be_u32.rs ]
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

#[cfg(test)]
#[disable]
mod test_write_u32_be {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
    use tempfile::tempfile;
    use byteorder::BigEndian;

    #[traced_test]
    async fn test_write_u32_be_correct_bytes() {
        // We'll create a tempfile and open it with tokio::fs::File to confirm the 4 bytes are as expected.
        let file_temp = tempfile().expect("Failed to create tempfile");
        let path = file_temp.into_temp_path(); // Or directly keep file_temp if you prefer. We'll just approach path-based for clarity.

        // Reopen that path with tokio for asynchronous I/O
        let mut async_file = tokio::fs::File::create(&path).await
            .expect("Failed to create async file");

        // Write a known value, say 0x1234ABCD => [0x12, 0x34, 0xAB, 0xCD] in big-endian
        let value: u32 = 0x1234ABCD;
        write_u32_be(&mut async_file, value).await
            .expect("Should write 4 big-endian bytes");

        // flush & drop to ensure we can read from it
        drop(async_file);

        // Now read back to confirm
        let mut read_file = tokio::fs::File::open(&path).await
            .expect("Failed to open file for reading");
        let mut buf = [0u8; 4];
        read_file.read_exact(&mut buf).await
            .expect("Should read 4 bytes");
        // Compare
        assert_eq!(buf, [0x12, 0x34, 0xAB, 0xCD], "Should match the big-endian representation");
    }

    #[traced_test]
    async fn test_write_u32_be_zero() {
        let file_temp = tempfile().expect("tempfile");
        let path = file_temp.into_temp_path();
        let mut async_file = tokio::fs::File::create(&path).await
            .expect("Failed to create async file");

        write_u32_be(&mut async_file, 0).await.expect("Write zero");
        drop(async_file);

        let mut read_file = tokio::fs::File::open(&path).await
            .expect("Failed to open file");
        let mut buf = [0u8; 4];
        read_file.read_exact(&mut buf).await.expect("read 4 bytes");
        // Big-endian 0 => [0,0,0,0]
        assert_eq!(buf, [0,0,0,0]);
    }

    #[traced_test]
    async fn test_write_u32_be_max() {
        // 0xFFFFFFFF => [0xFF,0xFF,0xFF,0xFF]
        let file_temp = tempfile().expect("tempfile");
        let path = file_temp.into_temp_path();
        let mut async_file = tokio::fs::File::create(&path).await
            .expect("open for write");

        write_u32_be(&mut async_file, u32::MAX).await.expect("write max");
        drop(async_file);

        let mut read_file = tokio::fs::File::open(&path).await
            .expect("open for read");
        let mut buf = [0u8; 4];
        read_file.read_exact(&mut buf).await.expect("read 4 bytes");
        assert_eq!(buf, [0xFF,0xFF,0xFF,0xFF]);
    }

    #[traced_test]
    async fn test_write_u32_be_error_propagation() {
        // For real error tests, we can define a partial mock or forcibly close the file descriptor, etc.
        // We'll do a simpler approach: open in read-only, so writing fails.

        let file_temp = tempfile().expect("tempfile");
        let path = file_temp.into_temp_path();
        // pre-create the file
        std::fs::File::create(&path).expect("create base file");

        // Now open in read-only with tokio => writing should fail
        let mut read_only_file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(false)
            .open(&path).await
            .expect("open read-only");

        let result = write_u32_be(&mut read_only_file, 0x12345678).await;
        assert!(result.is_err(), "Expected an error writing to read-only file");
    }
}

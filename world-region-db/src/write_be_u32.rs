// ---------------- [ File: src/write_be_u32.rs ]
crate::ix!();

/// Writes `value` as a 4-byte big-endian integer into `file`.
pub async fn write_u32_be(file: &mut tokio::fs::File, value: u32) -> std::io::Result<()> {
    let mut buf = [0u8; 4];
    byteorder::BigEndian::write_u32(&mut buf, value);
    file.write_all(&buf).await
}

#[cfg(test)]
mod test_write_u32_be {
    use super::*;
    use tokio::io::{AsyncReadExt, AsyncSeekExt, SeekFrom};
    use tempfile::NamedTempFile;
    use byteorder::BigEndian;

    #[traced_test]
    async fn test_write_u32_be_correct_bytes() {
        // Create a named temporary file and convert it to a temp path.
        let file_temp = NamedTempFile::new().expect("Failed to create tempfile");
        let path = file_temp.into_temp_path();

        // Open the file asynchronously for writing.
        let mut async_file = tokio::fs::File::create(&path).await
            .expect("Failed to create async file");

        // Write a known value 0x1234ABCD, expecting [0x12, 0x34, 0xAB, 0xCD].
        let value: u32 = 0x1234ABCD;
        write_u32_be(&mut async_file, value).await
            .expect("Should write 4 big-endian bytes");

        // Ensure the file is flushed and closed.
        drop(async_file);

        // Reopen the file asynchronously for reading.
        let mut read_file = tokio::fs::File::open(&path).await
            .expect("Failed to open file for reading");
        let mut buf = [0u8; 4];
        read_file.read_exact(&mut buf).await
            .expect("Should read 4 bytes");
        // Validate that the written bytes match the expected big-endian representation.
        assert_eq!(buf, [0x12, 0x34, 0xAB, 0xCD], "Should match the big-endian representation");
    }

    #[traced_test]
    async fn test_write_u32_be_zero() {
        let file_temp = NamedTempFile::new().expect("Failed to create tempfile");
        let path = file_temp.into_temp_path();

        let mut async_file = tokio::fs::File::create(&path).await
            .expect("Failed to create async file");

        // Write zero, which in big-endian should be [0, 0, 0, 0].
        write_u32_be(&mut async_file, 0).await.expect("Write zero");
        drop(async_file);

        let mut read_file = tokio::fs::File::open(&path).await
            .expect("Failed to open file");
        let mut buf = [0u8; 4];
        read_file.read_exact(&mut buf).await.expect("read 4 bytes");
        assert_eq!(buf, [0, 0, 0, 0]);
    }

    #[traced_test]
    async fn test_write_u32_be_max() {
        let file_temp = NamedTempFile::new().expect("Failed to create tempfile");
        let path = file_temp.into_temp_path();

        let mut async_file = tokio::fs::File::create(&path).await
            .expect("Failed to open file for write");

        // Write u32::MAX which should be represented as [0xFF, 0xFF, 0xFF, 0xFF].
        write_u32_be(&mut async_file, u32::MAX).await.expect("write max");
        drop(async_file);

        let mut read_file = tokio::fs::File::open(&path).await
            .expect("Failed to open file for read");
        let mut buf = [0u8; 4];
        read_file.read_exact(&mut buf).await.expect("read 4 bytes");
        assert_eq!(buf, [0xFF, 0xFF, 0xFF, 0xFF]);
    }

    #[traced_test]
    async fn test_write_u32_be_error_propagation() {
        let file_temp = NamedTempFile::new().expect("Failed to create tempfile");
        let path = file_temp.into_temp_path();
        // Pre-create the file using synchronous std::fs to ensure it exists.
        std::fs::File::create(&path).expect("create base file");

        // Open the file in read-only mode so that writing should fail.
        let mut read_only_file = tokio::fs::OpenOptions::new()
            .read(true)
            .write(false)
            .open(&path).await
            .expect("open read-only");

        let result = write_u32_be(&mut read_only_file, 0x12345678).await;
        assert!(result.is_ok(), "write_all succeeded in user buffer (?)");

        // But check flush or sync:
        let flush_result = read_only_file.flush().await;
        assert!(flush_result.is_err(), "We expect the OS to error on flush for read-only file");
    }
}

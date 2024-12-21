crate::ix!();

pub async fn verify_md5_checksum(target_file: impl AsRef<Path> + Debug, expected_md5: &str) 
    -> Result<(), Md5ChecksumVerificationError> 
{
    let actual_md5 = compute_md5(&target_file).await?;
    if actual_md5 != expected_md5 {
        return Err(Md5ChecksumVerificationError::ChecksumMismatch {
            expected: expected_md5.to_string(),
            actual: actual_md5,
        });
    }
    info!("MD5 verified: {:?}", target_file);
    Ok(())
}

pub async fn compute_md5(path: impl AsRef<Path>) -> Result<String, Md5ChecksumVerificationError> {
    let mut file    = File::open(path).await?;
    let mut context = Context::new();
    let mut buffer  = [0; 8192];

    loop {
        let bytes_read = file.read(&mut buffer).await?;
        if bytes_read == 0 {
            break;
        }
        context.consume(&buffer[..bytes_read]);
    }

    let digest = context.compute();
    Ok(format!("{:x}", digest))
}

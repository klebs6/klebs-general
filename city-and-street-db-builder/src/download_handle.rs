crate::ix!();

#[derive(Clone,Debug,PartialEq,Eq)]
pub struct OpenStreetMapRegionalDataDownloadHandle {
    region:        WorldRegion,
    download_link: &'static str,
}

impl OpenStreetMapRegionalDataDownloadHandle {

    pub fn new(region: WorldRegion, download_link: &'static str) -> Self {
        Self { region, download_link }
    }

    pub async fn obtain_pbf(&self, pbf_dir: impl AsRef<Path>) -> Result<PathBuf, PbfDownloadError> {

        // 1. Check local files first
        if let Some(local_file) = self.find_local_pbf(&pbf_dir)? {
            let local_file_path = pbf_dir.as_ref().join(&local_file);

            // Extract checksum from filename
            if let Some(filename_md5) = self.extract_md5_from_filename(&local_file) {
                let actual_md5 = self.compute_md5(&local_file_path).await?;
                if actual_md5 == filename_md5 {
                    info!("Local file checksum matches filename: {:?}", local_file_path);
                    return Ok(local_file_path);
                } else {
                    warn!("Local file checksum mismatch! Filename: {} Expected: {}, Actual: {}",
                        local_file, filename_md5, actual_md5);
                    warn!("Local file is not trusted due to checksum mismatch. Will download fresh.");
                }
            } else {
                info!("Local file does not have an MD5 in the filename. Will download fresh: {:?}", local_file_path);
            }
        }

        // 2. No suitable local file (either not found, missing MD5 in name, or mismatch) - download fresh
        let expected_md5 = fetch_md5_for_region(self.download_link).await?;
        let fresh_path = self.download_pbf_with_md5(&pbf_dir, &expected_md5).await?;
        Ok(fresh_path)
    }

    /// Attempt to find a local PBF file for this region.
    /// We look for files matching the pattern: "{region}-latest.*.osm.pbf"
    fn find_local_pbf(&self, pbf_dir: impl AsRef<Path>) -> Result<Option<String>, PbfDownloadError> {
        let base_name = self.base_filename_without_extension(); 
        let dir_iter = match std::fs::read_dir(pbf_dir) {
            Ok(iter) => iter,
            Err(e) => {
                if e.kind() == ErrorKind::NotFound {
                    return Ok(None);
                } else {
                    return Err(PbfDownloadError::IoError(e));
                }
            }
        };

        for entry in dir_iter {
            let entry = entry.map_err(PbfDownloadError::IoError)?;
            let fname = entry.file_name().to_string_lossy().to_string();
            // Check if filename starts with "{region}-latest"
            if fname.starts_with(&base_name) && fname.ends_with(".osm.pbf") {
                return Ok(Some(fname));
            }
        }

        Ok(None)
    }

    /// Extract MD5 from filename of form: "{region}-latest.{md5}.osm.pbf"
    fn extract_md5_from_filename(&self, filename: &str) -> Option<String> {
        // Filename pattern: "region-latest.<md5>.osm.pbf"
        // Split by '.' and expect at least 3 parts: ["region-latest", "md5", "osm", "pbf"]
        let parts: Vec<&str> = filename.split('.').collect();
        if parts.len() >= 4 && parts[parts.len()-2] == "osm" && parts[parts.len()-1] == "pbf" {
            // The md5 should be the second to last segment before "osm"
            // Actually parts[len-3] should be the md5 part.
            let md5_part = parts[parts.len()-3];
            if md5_part != "latest" { // ensure we're actually dealing with a MD5 segment
                return Some(md5_part.to_string());
            }
        }
        None
    }

    /// Construct a filename for the downloaded file with MD5 included.
    fn filename_with_md5(&self, md5: &str) -> String {
        // Original filename is something like "maryland-latest.osm.pbf"
        // Insert md5 before the "osm" part:
        let original_name = self.filename();
        let original_str = original_name.to_str().unwrap();

        // from: "maryland-latest.osm.pbf"
        // to:   "maryland-latest.{md5}.osm.pbf"
        let parts: Vec<&str> = original_str.rsplitn(3, '.').collect();
        // parts[0] = "pbf"
        // parts[1] = "osm"
        // parts[2] = "maryland-latest"

        if parts.len() == 3 {
            format!("{}.{}.{}.{}", parts[2], md5, parts[1], parts[0])
        } else {
            // If unexpected pattern, just append
            format!("{}.{}", original_str, md5)
        }
    }

    async fn download_pbf_with_md5(&self, pbf_dir: impl AsRef<Path>, expected_md5: &str) -> Result<PathBuf, PbfDownloadError> {
        // Download a temporary file first
        let temp_file = pbf_dir.as_ref().join(self.filename());
        if temp_file.exists() {
            std::fs::remove_file(&temp_file).map_err(PbfDownloadError::IoError)?;
        }

        // Download fresh file
        self.download_file(self.download_link, &temp_file).await?;

        // Verify MD5
        self.verify_md5_checksum(&temp_file, expected_md5).await?;

        // Rename with MD5 in the filename
        let final_name = self.filename_with_md5(expected_md5);
        let final_path = pbf_dir.as_ref().join(final_name);

        // If final_path exists, remove it
        if final_path.exists() {
            std::fs::remove_file(&final_path).map_err(PbfDownloadError::IoError)?;
        }

        std::fs::rename(&temp_file, &final_path).map_err(PbfDownloadError::IoError)?;
        Ok(final_path)
    }

    fn base_filename_without_extension(&self) -> String {
        // The normal filename is something like: "maryland-latest.osm.pbf"
        // Remove ".osm.pbf" to get "maryland-latest"
        let fname = self.filename();
        let fstr = fname.to_str().unwrap_or("");
        fstr.replace(".osm.pbf", "")
    }

    pub fn filename(&self) -> PathBuf {
        PathBuf::from(self.download_link.split("/").last().unwrap())
    }

    async fn download_file(&self, download_link: &str, target_file: &Path) 
        -> Result<(), PbfDownloadError> 
    {
        assert!(!target_file.exists());

        info!("Downloading OSM regional data to: {:?}", target_file);

        let client = reqwest::Client::new();
        let response = client
            .get(download_link)
            .send()
            .await?
            .error_for_status()?;

        let total_size = response.content_length();
        let mut file = File::create(&target_file).await?;
        let mut downloaded = 0u64;

        let mut stream = response.bytes_stream();
        let mut counter = 0;

        while let Some(chunk) = stream.try_next().await? {
            file.write_all(chunk.as_ref()).await?;
            downloaded += chunk.len() as u64;

            if counter % 1000 == 0 {
                if let Some(total) = total_size {
                    let pct = (downloaded as f64 / total as f64) * 100.0;
                    info!("Download progress: {:.2}%", pct);
                } else {
                    info!("Downloaded {} bytes", downloaded);
                }
            }
            counter += 1;
        }

        info!("File downloaded successfully: {:?}", target_file);
        Ok(())
    }

    async fn verify_md5_checksum(&self, target_file: &Path, expected_md5: &str) 
        -> Result<(),Md5ChecksumVerificationError> 
    {
        let actual_md5 = self.compute_md5(target_file).await?;
        if actual_md5 != expected_md5 {
            return Err(Md5ChecksumVerificationError::ChecksumMismatch {
                expected: expected_md5.to_string(),
                actual: actual_md5,
            });
        }

        info!("MD5 checksum verified for file {:#?}", target_file);
        Ok(())
    }

    async fn compute_md5(&self, path: &Path) -> Result<String, Md5ChecksumVerificationError> {
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
}

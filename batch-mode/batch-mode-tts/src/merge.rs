// ---------------- [ File: batch-mode-tts/src/merge.rs ]
crate::ix!();

impl BatchModeTtsJob {
    // -----------------------------------------------------------------------
    // Helper: merge with ffmpeg concat demuxer
    // -----------------------------------------------------------------------
    pub async fn merge_parts_ffmpeg(&self, parts: &[PathBuf]) -> Result<(), BatchModeTtsError> {
        // Build concat‑list file
        let list_path = self
            .output_path()
            .with_file_name("ffmpeg_concat_list.txt");
        let mut list_file = File::create(&list_path)?;
        for p in parts {
            writeln!(list_file, "file '{}'", p.display())?;
        }
        drop(list_file);

        // Run ffmpeg
        let status = Command::new("ffmpeg")
            .args([
                "-y",
                "-f",
                "concat",
                "-safe",
                "0",
                "-i",
                list_path.to_str().unwrap(),
                "-c",
                "copy",
                self.output_path().to_str().unwrap(),
            ])
            .status()?;

        if status.success() {
            debug!("ffmpeg merged parts successfully");
            Ok(())
        } else {
            error!("ffmpeg returned non‑zero status {status}");
            Err(
                BatchModeTtsError::FfmpegMergeError { status }
            )
        }
    }
}

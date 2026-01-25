// ---------------- [ File: batch-mode-tts/src/err.rs ]
crate::ix!();

// ---------------------------------------------------------------------------
// Batch‑mode Text‑to‑Speech error type
// ---------------------------------------------------------------------------
error_tree! {
    pub enum BatchModeTtsError {
        IoError(std::io::Error),
        OpenAIError(OpenAIError),
        BatchModeTtsJobBuilderError(BatchModeTtsJobBuilderError),
        #[display("ffmpeg failed to merge audio files; status={status:?}")]
        FfmpegMergeError { status: std::process::ExitStatus },
    }
}

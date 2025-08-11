crate::ix!();

use std::{
    fs::File,
    io::{self, Write},
    path::{Path, PathBuf},
    process::Command,
};

use derive_builder::Builder;
use getset::Getters;
use tracing::{debug, error, info, trace, warn};

use async_openai::{
    config::OpenAIConfig,
    types::{
        CreateSpeechRequest, CreateSpeechRequestArgs, SpeechModel, SpeechResponseFormat, Voice,
    },
    Client as OpenAIClient,
};

/// ---------------------------------------------------------------------------
/// Batch‑mode Text‑to‑Speech error type
/// ---------------------------------------------------------------------------
error_tree! {
    pub enum BatchModeTtsError {
        IoError(std::io::Error),
        OpenAIError(OpenAIError),
        BatchModeTtsJobBuilderError(BatchModeTtsJobBuilderError),
        #[display("ffmpeg failed to merge audio files; status={status:?}")]
        FfmpegMergeError { status: std::process::ExitStatus },
    }
}

/// ---------------------------------------------------------------------------
/// Chunk‑aware TTS job & builder
/// ---------------------------------------------------------------------------
#[derive(Getters, Builder, Debug, Clone)]
#[builder(name = "BatchModeTtsJobBuilder", pattern = "owned")]
#[getset(get = "pub")]
pub struct BatchModeTtsJob {
    /// Source UTF‑8 text file.
    input_path: PathBuf,
    /// Final destination audio file.
    output_path: PathBuf,

    // Optional staging directory; defaults to `<output_path>.parts`
    #[builder(setter(strip_option), default)]
    work_dir: Option<PathBuf>,

    // Maximum characters per TTS request (<=4096 by OpenAI spec).
    #[builder(default = "3500")]
    chunk_chars: usize,

    #[builder(default = "SpeechModel::Tts1")]
    model: SpeechModel,
    #[builder(default = "Voice::Sage")]
    voice: Voice,
    #[builder(default = "SpeechResponseFormat::Mp3")]
    response_format: SpeechResponseFormat,
    #[builder(default = "1.0")]
    speed: f32,
}

impl BatchModeTtsJob {

    /// Execute the job with automatic chunking & ffmpeg concat.
    ///
    /// This operation is **idempotent** and _resume‑safe_:
    /// * Already‑present, non‑empty `part_####.<ext>` files in the working
    ///   directory are detected and **skipped**, allowing the job to continue
    ///   from the first missing chunk after any interruption.
    /// * Empty or corrupt part files are automatically regenerated.
    #[tracing::instrument(
        level = "info",
        skip_all,
        fields(
            input  = %self.input_path.display(),
            output = %self.output_path.display(),
            chunk  = self.chunk_chars
        )
    )]
    pub async fn run(&self) -> Result<(), BatchModeTtsError> {
        use std::io;

        info!("Starting (resumable) chunked batch‑mode TTS job");

        // -------------------------------------------------------------------
        // 1. Load & split input text
        // -------------------------------------------------------------------
        let text = tokio::fs::read_to_string(&self.input_path).await?;
        debug!("Loaded {} UTF‑8 bytes", text.len());

        let chunks = Self::chunk_text(&text, self.chunk_chars);
        info!("Split text into {} chunk(s)", chunks.len());

        // Prepare the working directory
        let work_dir = self
            .work_dir
            .clone()
            .unwrap_or_else(|| self.output_path.with_extension("parts"));
        tokio::fs::create_dir_all(&work_dir).await.ok();
        debug!("Using work dir {:?}", work_dir);

        // Pre‑compute extension for the response format
        let ext = Self::ext_for_format(self.response_format);

        // -------------------------------------------------------------------
        // 2. Fire TTS requests (skip any part that already exists)
        // -------------------------------------------------------------------
        let client = OpenAIClient::with_config(
            OpenAIConfig::new()
                .with_api_key(std::env::var("OPENAI_API_KEY")
                    .expect("OPENAI_API_KEY env‑var missing")),
        );
        let audio_api = async_openai::Audio::new(&client);

        let mut part_paths = Vec::<PathBuf>::with_capacity(chunks.len());

        for (idx, segment) in chunks.iter().enumerate() {
            let part_path = work_dir.join(format!("part_{idx:04}.{ext}"));

            // If the part already exists and is non‑empty, skip regeneration.
            match tokio::fs::metadata(&part_path).await {
                Ok(meta) if meta.len() > 0 => {
                    info!("Skipping existing part #{idx} ({:?})", part_path);
                    part_paths.push(part_path);
                    continue;
                }
                Ok(_) => {
                    warn!("Existing part #{idx} is empty/corrupt → regenerating");
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    trace!("Part #{idx} missing → generating");
                }
                Err(e) => return Err(BatchModeTtsError::IoError(e)),
            }

            let req: CreateSpeechRequest = CreateSpeechRequestArgs::default()
                .input(segment.to_owned())
                .model(self.model.clone())
                .voice(self.voice.clone())
                .response_format(self.response_format)
                .speed(self.speed)
                .build()?;

            let resp = audio_api.speech(req).await?;
            info!("Received {} bytes for chunk #{idx}", resp.bytes.len());

            tokio::fs::write(&part_path, resp.bytes).await?;
            debug!("Wrote chunk #{idx} → {:?}", part_path);

            part_paths.push(part_path);
        }

        // Sanity check – ensure every part is present
        if part_paths.len() != chunks.len() {
            error!(
                "Mismatch: have {} part files but expected {}",
                part_paths.len(),
                chunks.len()
            );
            return Err(BatchModeTtsError::IoError(io::Error::new(
                io::ErrorKind::Other,
                "missing part files after resume‑aware generation",
            )));
        }

        // -------------------------------------------------------------------
        // 3. Merge parts via ffmpeg
        // -------------------------------------------------------------------
        self.merge_parts_ffmpeg(&part_paths).await?;

        info!("Wrote final audio {}", self.output_path.display());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Helper: naïve text splitter favouring newline boundaries
    // -----------------------------------------------------------------------
    fn chunk_text(text: &str, max_len: usize) -> Vec<String> {
        let mut res = Vec::new();
        let mut buf = String::new();

        for line in text.lines() {
            if buf.len() + line.len() + 1 > max_len && !buf.is_empty() {
                res.push(buf.clone());
                buf.clear();
            }
            if !buf.is_empty() {
                buf.push('\n');
            }
            buf.push_str(line);
        }
        if !buf.is_empty() {
            res.push(buf);
        }
        res
    }

    fn ext_for_format(fmt: SpeechResponseFormat) -> &'static str {
        match fmt {
            SpeechResponseFormat::Mp3 => "mp3",
            SpeechResponseFormat::Opus => "ogg",
            SpeechResponseFormat::Aac => "aac",
            SpeechResponseFormat::Flac => "flac",
            SpeechResponseFormat::Pcm => "pcm",
            SpeechResponseFormat::Wav => "wav",
            _ => "bin",
        }
    }

    // -----------------------------------------------------------------------
    // Helper: merge with ffmpeg concat demuxer
    // -----------------------------------------------------------------------
    async fn merge_parts_ffmpeg(&self, parts: &[PathBuf]) -> Result<(), BatchModeTtsError> {
        // Build concat‑list file
        let list_path = self
            .output_path
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
                self.output_path.to_str().unwrap(),
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

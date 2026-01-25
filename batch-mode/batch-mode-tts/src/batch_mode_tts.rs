// ---------------- [ File: batch-mode-tts/src/batch_mode_tts.rs ]
crate::ix!();

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

    pub fn ext_for_format(fmt: SpeechResponseFormat) -> &'static str {
        match fmt {
            SpeechResponseFormat::Mp3 => "mp3",
            SpeechResponseFormat::Opus => "ogg",
            SpeechResponseFormat::Aac => "aac",
            SpeechResponseFormat::Flac => "flac",
            SpeechResponseFormat::Pcm => "pcm",
            SpeechResponseFormat::Wav => "wav",
            //_ => "bin",
        }
    }
}

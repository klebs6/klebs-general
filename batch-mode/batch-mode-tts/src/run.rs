crate::ix!();

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
            input  = %self.input_path().display(),
            output = %self.output_path().display(),
            chunk  = self.chunk_chars()
        )
    )]
    pub async fn run(&self) -> Result<(), BatchModeTtsError> {
        use std::io;

        info!("Starting (resumable) chunked batch‑mode TTS job");

        // -------------------------------------------------------------------
        // 1) Load full input
        // -------------------------------------------------------------------
        let text = tokio::fs::read_to_string(&self.input_path()).await?;
        debug!("Loaded {} UTF‑8 bytes", text.len());

        // -------------------------------------------------------------------
        // 2) Prep workspace + extension
        // -------------------------------------------------------------------
        let work_dir = self
            .work_dir()
            .clone()
            .unwrap_or_else(|| self.output_path().with_extension("parts"));
        tokio::fs::create_dir_all(&work_dir).await.ok();
        debug!("Using work dir {:?}", work_dir);

        let ext = Self::ext_for_format(*self.response_format());

        // -------------------------------------------------------------------
        // 3) Discover high‑water mark (HWM) of already generated parts
        //    We will *not* regenerate anything ≤ HWM.
        // -------------------------------------------------------------------
        let hwm_opt = Self::find_high_water_mark(&work_dir, ext)?;
        let hwm = hwm_opt.unwrap_or_else(|| usize::MAX.wrapping_sub(usize::MAX)); // 0 saturating trick
        let anchored_count = if hwm_opt.is_some() { hwm + 1 } else { 0 };
        if anchored_count > 0 {
            info!("Found {anchored_count} existing contiguous part(s); high‑water mark is #{hwm:04}");
        } else {
            info!("No existing parts found; starting from scratch");
        }

        // -------------------------------------------------------------------
        // 4) Reproduce *legacy* (byte‑biased) boundaries up to HWM,
        //    then re‑chunk the *remainder* with the corrected char‑safe splitter.
        // -------------------------------------------------------------------
        let legacy_chunks = Self::old_chunk_text(&text, *self.chunk_chars());
        info!(
            "Legacy splitter would produce {} chunk(s) at size {}",
            legacy_chunks.len(),
            self.chunk_chars()
        );

        if anchored_count > legacy_chunks.len() {
            warn!(
                "Existing parts exceed legacy recomputed chunk count ({} > {}) — \
                 seams likely changed since last run; clamping anchor to {}",
                anchored_count,
                legacy_chunks.len(),
                legacy_chunks.len().saturating_sub(1)
            );
        }
        let anchored_count = anchored_count.min(legacy_chunks.len());
        let remainder_text = if anchored_count == 0 {
            // Use normalized corpus produced by the legacy splitter for consistency
            legacy_chunks.join("")
        } else {
            legacy_chunks[anchored_count..].join("")
        };
        debug!(
            "Prepared remainder text after anchor: {} chars",
            remainder_text.chars().count()
        );

        // New corrected splitter: strictly ≤ min(self.chunk_chars, 4096) characters
        let max_chars = self.chunk_chars().min(&4096);
        let new_chunks = Self::chunk_text(&remainder_text, *max_chars);
        info!(
            "Char‑safe splitter produced {} downstream chunk(s) at size ≤ {} chars",
            new_chunks.len(),
            max_chars
        );

        // Total expected parts = anchored (frozen) + newly planned
        let expected_total = anchored_count + new_chunks.len();

        // -------------------------------------------------------------------
        // 5) Set up API client
        // -------------------------------------------------------------------
        let client = OpenAIClient::with_config(
            OpenAIConfig::new().with_api_key(
                std::env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY env‑var missing"),
            ),
        );
        let audio_api = async_openai::Audio::new(&client);

        // We will accumulate all final part paths (frozen + newly generated)
        let mut part_paths = Vec::<PathBuf>::with_capacity(expected_total);

        // 5a) Register frozen parts [0 .. anchored_count)
        for i in 0..anchored_count {
            let p = work_dir.join(format!("part_{i:04}.{ext}"));
            match tokio::fs::metadata(&p).await {
                Ok(meta) if meta.len() > 0 => {
                    info!("Skipping existing part #{i} ({:?})", p);
                    part_paths.push(p);
                }
                Ok(_) => {
                    warn!(
                        "Existing part #{i} is empty/corrupt — will be regenerated using *new* chunking"
                    );
                    // Fallthrough: allow regeneration below by *injecting* this index into downstream flow.
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    warn!(
                        "Expected frozen part #{i} not found — will be regenerated using *new* chunking"
                    );
                    // Fallthrough to regeneration path below.
                }
                Err(e) => return Err(BatchModeTtsError::IoError(e)),
            }
        }

        // 5b) Generate/skip downstream parts [anchored_count .. expected_total)
        for (j, segment) in new_chunks.iter().enumerate() {
            let idx = anchored_count + j;
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

            // Guardrail (should never trigger because chunk_text enforces ≤max_chars)
            let seg_chars = segment.chars().count();
            if seg_chars > 4096 {
                warn!(
                    "Segment #{idx} unexpectedly exceeds 4096 chars ({}); applying clip guardrail",
                    seg_chars
                );
            }
            let safe_input = if seg_chars > 4096 {
                clip_to_chars(segment, 4096)
            } else {
                segment.clone()
            };

            let req: CreateSpeechRequest = CreateSpeechRequestArgs::default()
                .input(safe_input)
                .model(self.model().clone())
                .voice(self.voice().clone())
                .response_format(*self.response_format())
                .speed(*self.speed())
                .build()?;

            let resp = audio_api.speech(req).await?;
            info!("Received {} bytes for chunk #{idx}", resp.bytes.len());

            tokio::fs::write(&part_path, resp.bytes).await?;
            debug!("Wrote chunk #{idx} → {:?}", part_path);

            part_paths.push(part_path);
        }

        // Sanity check – ensure we have every expected part (frozen + new)
        if part_paths.len() != expected_total {
            error!(
                "Mismatch after generation: have {} part files but expected {}",
                part_paths.len(),
                expected_total
            );
            return Err(BatchModeTtsError::IoError(io::Error::new(
                io::ErrorKind::Other,
                "missing part files after resume‑aware generation",
            )));
        }

        // -------------------------------------------------------------------
        // 6) Merge parts via ffmpeg
        // -------------------------------------------------------------------
        self.merge_parts_ffmpeg(&part_paths).await?;

        info!("Wrote final audio {}", self.output_path().display());
        Ok(())
    }
}

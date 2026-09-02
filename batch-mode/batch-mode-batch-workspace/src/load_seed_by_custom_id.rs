// ---------------- [ File: batch-mode-batch-workspace/src/load_seed_by_custom_id.rs ]
crate::ix!();

use serde::Deserialize;
use tokio::{fs, io::{self, AsyncBufReadExt, BufReader}};

#[derive(Deserialize)]
struct ManifestLine {
    custom_id: String,
    // future‑proof: allow an optional header/name field
    #[serde(default)]
    header: Option<String>,
}

#[async_trait]
impl LoadSeedByCustomId for BatchWorkspace {
    /// Locate the seed (header) that corresponds to `custom_id`
    /// by scanning seed‑manifest files (.json **or** .jsonl).
    #[instrument(
        level = "debug",
        skip(self),
        fields(%custom_id, workdir = %self.workdir().display())
    )]
    async fn load_seed_by_custom_id(
        &self,
        custom_id: &CustomRequestId,
    ) -> Result<Box<dyn Named + Send + Sync>, BatchWorkspaceError> {
        let mut dir = fs::read_dir(self.workdir()).await?;

        while let Some(entry) = dir.next_entry().await? {
            let p = entry.path();

            if !p.file_name()
                .and_then(|s| s.to_str())
                .map_or(false, |s| s.starts_with("batch_seed_manifest_"))
            {
                continue;
            }

            // ---- Fast path: single‑JSON object (current expectation) ----
            match fs::read_to_string(&p).await {
                Ok(raw) => {
                    if let Ok(map) =
                        serde_json::from_str::<serde_json::Map<String, serde_json::Value>>(&raw)
                    {
                        if let Some(serde_json::Value::String(name)) =
                            map.get(custom_id.as_str())
                        {
                            debug!(file=%p.display(), "found via single‑JSON manifest");
                            return Ok(Box::new(SimpleSeed(name.clone())));
                        }
                    } else {
                        // fall through to line‑by‑line parsing
                    }
                }
                Err(e) => return Err(e.into()),
            }

            // ---- Slow path: JSON‑Lines (one object per line) ------------
            let file = fs::File::open(&p).await?;
            let mut reader = BufReader::new(file);
            let mut line = String::new();

            loop {
                line.clear();
                let bytes = reader.read_line(&mut line).await?;
                if bytes == 0 {
                    break; // EOF
                }

                let parsed: ManifestLine = match serde_json::from_str(&line) {
                    Ok(v) => v,
                    Err(_) => continue, // skip malformed lines
                };

                if parsed.custom_id == custom_id.as_str() {
                    let name = parsed
                        .header
                        .unwrap_or_else(|| parsed.custom_id.clone());
                    debug!(file=%p.display(), "found via JSON‑Lines manifest");
                    return Ok(Box::new(SimpleSeed(name)));
                }
            }
        }

        Err(BatchWorkspaceError::NoBatchFileTripleAtIndex {
            index: BatchIndex::new(),
        })
    }
}

/// Small helper so we can return something that implements `Named`
struct SimpleSeed(String);
impl Named for SimpleSeed {
    fn name(&self) -> std::borrow::Cow<'_, str> {
        std::borrow::Cow::Borrowed(&self.0)
    }
}

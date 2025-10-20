crate::ix!();

impl BatchModeTtsJob {
    /// Find the largest contiguous existing non‑empty part index in `work_dir`
    /// matching the current response format extension. Returns:
    /// - `Ok(Some(idx))` for the last existing contiguous index (0‑based),
    /// - `Ok(None)` if no parts exist,
    /// - `Err(io::Error)` on unexpected I/O failure.
    pub fn find_high_water_mark(work_dir: &Path, ext: &str) -> io::Result<Option<usize>> {
        let mut i: usize = 0;
        loop {
            let p = work_dir.join(format!("part_{i:04}.{ext}"));
            match std::fs::metadata(&p) {
                Ok(meta) => {
                    if meta.len() == 0 {
                        warn!(
                            "Encountered zero‑length file at contiguous index #{i} ({:?}); \
                             treating as boundary for high‑water mark",
                            p
                        );
                        break;
                    }
                    trace!("Detected existing part #{i} ({:?})", p);
                    i += 1;
                    continue;
                }
                Err(e) if e.kind() == io::ErrorKind::NotFound => {
                    // First gap — stop contiguous scan
                    break;
                }
                Err(e) => {
                    error!("metadata() failed while scanning for high‑water mark: {e}");
                    return Err(e);
                }
            }
        }

        if i == 0 {
            Ok(None)
        } else {
            Ok(Some(i - 1))
        }
    }
}

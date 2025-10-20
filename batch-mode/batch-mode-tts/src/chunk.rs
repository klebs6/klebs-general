crate::ix!();

impl BatchModeTtsJob {
    /// Character‑safe splitter (≤ `max_len` chars) with a soft preference for newline
    /// boundaries. Long single lines are split at exact character boundaries.
    pub fn chunk_text(text: &str, max_len: usize) -> Vec<String> {
        let limit = max_len.min(4096);
        if limit == 0 {
            warn!("chunk_text called with max_len=0; returning empty chunk list");
            return Vec::new();
        }

        let mut res = Vec::<String>::new();
        let mut buf = String::new();
        let mut buf_chars: usize = 0;

        // Helper: flush current buffer if non‑empty
        let flush = |res: &mut Vec<String>, buf: &mut String, buf_chars: &mut usize| {
            if !buf.is_empty() {
                trace!("Flushing chunk of {} chars", *buf_chars);
                res.push(std::mem::take(buf));
                *buf_chars = 0;
            }
        };

        for line in text.lines() {
            let mut remaining = line;
            loop {
                let rem_chars = remaining.chars().count();
                // Will we need to insert a newline before appending `remaining`?
                let sep = if buf.is_empty() { 0 } else { 1 };

                if buf_chars + sep + rem_chars <= limit {
                    if sep == 1 {
                        buf.push('\n');
                        buf_chars += 1;
                    }
                    buf.push_str(remaining);
                    buf_chars += rem_chars;
                    break; // Done with this line
                }

                // Not enough room to add whole `remaining`
                // 1) If the buffer has something, flush it first so we start fresh
                if buf_chars > 0 {
                    flush(&mut res, &mut buf, &mut buf_chars);
                    // Continue the loop — we will re‑evaluate with an empty buffer
                    continue;
                }

                // 2) Buffer is empty but `remaining` itself is too large;
                // split `remaining` into `limit`‑sized char chunk
                let take_n = limit; // fill exactly to limit
                // Take first `take_n` chars of `remaining`
                let taken: String = remaining.chars().take(take_n).collect();
                let taken_count = taken.chars().count();
                debug!(
                    "Splitting an over‑long line into a full chunk of {} chars (limit {})",
                    taken_count, limit
                );
                res.push(taken);

                // Advance `remaining`
                let mut it = remaining.chars();
                for _ in 0..take_n {
                    it.next();
                }
                let rest: String = it.collect();

                if rest.is_empty() {
                    break; // finished this line
                } else {
                    remaining = &rest; // loop continues on rest
                    // We must store `rest` somewhere stable; allocate new String and use it
                    // To avoid lifetime issue, rebind `remaining` to a new owned string and iterate again
                    let owned = rest; // already owned
                    // Reassign `remaining` to a &'_ str from owned for next iteration
                    // But we cannot keep `owned` alive across iterations without storage.
                    // Workaround: move ownership into `remaining_owned` and shadow `remaining`.
                    // Implement via block scope below.
                    let mut cursor = owned;
                    loop {
                        // inner splitting loop replicates the top logic with `cursor`
                        // 1) Determine chars left
                        let c_rem = cursor.chars().count();
                        let sep2 = if buf.is_empty() { 0 } else { 1 };
                        if buf_chars + sep2 + c_rem <= limit {
                            if sep2 == 1 {
                                buf.push('\n');
                                buf_chars += 1;
                            }
                            buf.push_str(&cursor);
                            buf_chars += c_rem;
                            break;
                        }
                        if buf_chars > 0 {
                            flush(&mut res, &mut buf, &mut buf_chars);
                            continue;
                        }
                        // Take another full chunk from cursor
                        let take_full: String = cursor.chars().take(limit).collect();
                        let taken_full_count = take_full.chars().count();
                        debug!(
                            "Splitting continuation into full chunk of {} chars (limit {})",
                            taken_full_count, limit
                        );
                        res.push(take_full);
                        let mut it2 = cursor.chars();
                        for _ in 0..limit {
                            it2.next();
                        }
                        let tmp: String = it2.collect();
                        if tmp.is_empty() {
                            break;
                        }
                        cursor = tmp;
                        // continue inner loop
                    }
                    break; // done with original `line`
                }
            }
        }

        if !buf.is_empty() {
            trace!("Flushing final chunk of {} chars", buf_chars);
            res.push(buf);
        }

        res
    }
}

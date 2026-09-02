// ---------------- [ File: batch-mode-tts/src/legacy.rs ]
crate::ix!();

impl BatchModeTtsJob {

    pub fn old_chunk_text(text: &str, max_len: usize) -> Vec<String> { 

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
}

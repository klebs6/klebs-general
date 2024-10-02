crate::ix!();

pub struct BufferedWriter {
    buffer: Arc<Mutex<Vec<u8>>>,
}

impl std::io::Write for BufferedWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let mut buffer = self.buffer.lock().unwrap();
        buffer.extend_from_slice(buf);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

impl MakeWriter for BufferedWriter {
    type Writer = Self;

    fn make_writer(&self) -> Self::Writer {
        BufferedWriter {
            buffer: Arc::clone(&self.buffer),
        }
    }
}

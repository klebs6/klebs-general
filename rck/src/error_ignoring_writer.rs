crate::ix!();

pub struct ErrorIgnoringWriter<W: Write>(W);

impl<W: Write> Write for ErrorIgnoringWriter<W> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.0.write(buf).or_else(|e| {
            if e.kind() == std::io::ErrorKind::BrokenPipe {
                // Ignore broken pipe errors
                Ok(0)
            } else {
                Err(e)
            }
        })
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.0.flush()
    }
}

pub fn setup_logging() {
    tracing_subscriber::fmt()
        .with_writer(|| ErrorIgnoringWriter(std::io::stdout())) // Handle broken pipe gracefully
        .init();
}

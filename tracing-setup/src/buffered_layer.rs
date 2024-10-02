crate::ix!();

#[derive(Debug)]
pub struct BufferedLayer {
    tag:           Option<String>,
    buffer:        Arc<Mutex<Vec<String>>>,
    event_printer: EventPrinter,
}

impl Flushable for BufferedLayer {

    fn flush(&self) {

        use colored::Colorize;

        if let Ok(mut buf) = self.buffer.lock() {

            if let Some(tag) = &self.tag {

                let msg = format!("---------------------------------------------------------[trace_events: {}]", tag);

                println!(
                    "{}",
                    msg.bright_blue(),
                );
            }

            for message in &*buf {
                println!("{}", message);
            }
            buf.clear();
        }
    }
}

impl Clone for BufferedLayer {

    fn clone(&self) -> Self {
        Self {
            tag:           self.tag.clone(),
            buffer:        Arc::clone(&self.buffer),
            event_printer: self.event_printer.clone()
        }
    }
}

impl Default for BufferedLayer {

    fn default() -> Self {
        Self {
            tag:           None,
            buffer:        Arc::new(Mutex::new(Vec::new())),
            event_printer: EventPrinter::default(),
        }
    }
}

impl BufferedLayer {

    pub fn new_with_tag(tag: &str) -> Self {
        Self {
            tag:           Some(tag.to_string()),
            buffer:        Arc::new(Mutex::new(Vec::new())),
            event_printer: EventPrinter::default(),
        }
    }
}

impl<S: Subscriber> tracing_subscriber::Layer<S> for BufferedLayer {

    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {

        use std::fmt::Write;

        if let Ok(mut buf) = self.buffer.lock() {
            match self.event_printer {
                EventPrinter::FullWithHeader     => FullWithHeader::handle_buffer_write_event(event,&mut buf),
                EventPrinter::LogLineAndContents => LogLineAndContents::handle_buffer_write_event(event,&mut buf),
                EventPrinter::JustTheContents    => JustTheContents::handle_buffer_write_event(event,&mut buf),
            }
        }
    }
}

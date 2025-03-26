crate::ix!();

#[derive(Getters, Setters, Debug)]
#[getset(get = "pub", set = "pub")]
pub struct BufferedLayer {
    tag: Option<String>,
    buffer: Arc<Mutex<Vec<String>>>,

    /// The single source of truth for how we print events:
    event_printer: EventPrinter,
}

impl BufferedLayer {
    pub fn new_with_tag(tag: &str) -> Self {
        Self {
            tag: Some(tag.to_string()),
            buffer: Arc::new(Mutex::new(Vec::new())),
            event_printer: EventPrinter::default(),
        }
    }
}

// A single, unified impl Layer<S>
impl<S: Subscriber> tracing_subscriber::Layer<S> for BufferedLayer {

    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        if let Ok(mut buf) = self.buffer.lock() {

            match &self.event_printer {
                EventPrinter::FullWithHeader => {
                    // Original approach: debug-print the entire event
                    let mut msg = String::new();
                    let _ = write!(&mut msg, "{:#?}", event);
                    buf.push(msg);
                }

                EventPrinter::LogLineAndContents {
                    show_timestamp,
                    show_loglevel,
                } => {
                    // Single-line approach with optional timestamp/level
                    use chrono::{Local, SecondsFormat};
                    let now  = Local::now().to_rfc3339_opts(SecondsFormat::Millis, true);
                    let meta = event.metadata();

                    let mut line = String::new();

                    if *show_timestamp {
                        line.push_str(&now);
                        line.push(' ');
                    }
                    if *show_loglevel {
                        line.push('[');
                        line.push_str(&meta.level().to_string());
                        line.push(' ');
                        line.push_str(meta.target());
                        line.push_str("] ");
                    }

                    // Collect field data
                    struct FieldCollector(String);
                    impl tracing::field::Visit for FieldCollector {
                        fn record_debug(&mut self, f: &tracing::field::Field, v: &dyn std::fmt::Debug) {
                            let _ = write!(self.0, "{} = {:?}, ", f.name(), v);
                        }
                    }
                    let mut visitor = FieldCollector(String::new());
                    event.record(&mut visitor);

                    if !visitor.0.is_empty() {
                        // Remove trailing ", "
                        visitor.0.truncate(visitor.0.len().saturating_sub(2));
                        line.push_str(&visitor.0);
                    }

                    buf.push(line);
                }

                EventPrinter::JustTheContents => {
                    // Minimal approach: only the field values
                    let mut message = String::new();
                    struct FieldCollector(String);
                    impl tracing::field::Visit for FieldCollector {
                        fn record_debug(&mut self, _f: &tracing::field::Field, v: &dyn std::fmt::Debug) {
                            let _ = write!(self.0, "{:?}, ", v);
                        }
                    }
                    let mut visitor = FieldCollector(String::new());
                    event.record(&mut visitor);

                    if !visitor.0.is_empty() {
                        visitor.0.truncate(visitor.0.len().saturating_sub(2));
                        message.push_str(&visitor.0);
                    }

                    buf.push(message);
                }
            }
        }
    }
}

// The flush logic remains as-is (unchanged):
impl Flushable for BufferedLayer {
    fn flush(&self) {
        use colored::Colorize;

        if let Ok(mut buf) = self.buffer.lock() {
            if let Some(tag) = &self.tag {
                let msg = format!("---------------------------------------------------------[trace_events: {}]", tag);
                println!("{}", msg.bright_blue());
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
            tag:            self.tag.clone(),
            buffer:         Arc::clone(&self.buffer),
            event_printer:  self.event_printer.clone(),
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

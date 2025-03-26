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

impl<S: Subscriber> tracing_subscriber::Layer<S> for BufferedLayer {
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {
        use tracing::{trace, debug, info, warn, error};

        trace!("on_event called for BufferedLayer");

        if let Ok(mut buf) = self.buffer.lock() {
            trace!("successfully acquired buffer lock, building event log line");

            match &self.event_printer {
                EventPrinter::FullWithHeader => {
                    // Original approach: debug-print the entire event
                    let mut msg = String::new();
                    let _ = write!(&mut msg, "{:#?}", event);

                    debug!("pushing fully detailed event log line");
                    buf.push(msg);
                }

                EventPrinter::LogLineAndContents {
                    show_timestamp,
                    show_loglevel,
                    show_location,
                } => {
                    // Single-line approach with optional timestamp/level, but
                    // place the location info after the message with alignment
                    use chrono::{Local, SecondsFormat};

                    let now  = Local::now().to_rfc3339_opts(SecondsFormat::Millis, true);
                    let meta = event.metadata();

                    let mut line = String::new();

                    if *show_loglevel {
                        let desired = 6;
                        let level   = meta.level().to_string();
                        let pad     = desired - level.len();
                        line.push_str(&level);
                        line.push_str(&" ".repeat(pad));
                    }

                    if *show_location {
                        // we place the location (and level, if requested) at a fixed column
                        // for consistent right-bracket alignment across lines
                        let location = format!(" [{}] ", meta.target());
                        let max_len = 60;
                        if location.len() > max_len {
                            let prefix = &location[0..max_len];
                            line.push_str(&prefix);
                        } else {
                            line.push_str(&location);
                            let pad = max_len - location.len();
                            line.push_str(&" ".repeat(pad));
                        }
                        line.push_str(&" ");
                    }

                    // optional timestamp
                    if *show_timestamp {
                        line.push_str(&now);
                        line.push(' ');
                    }

                    // collect field data
                    struct FieldCollector(String);
                    impl tracing::field::Visit for FieldCollector {
                        fn record_debug(
                            &mut self,
                            f: &tracing::field::Field,
                            v: &dyn std::fmt::Debug
                        ) {
                            if f.name() == "message" {
                                let _ = write!(self.0, "{:?}, ", v);
                            } else {
                                let _ = write!(self.0, "{} = {:?}, ", f.name(), v);
                            }
                        }
                    }
                    let mut visitor = FieldCollector(String::new());
                    event.record(&mut visitor);

                    // trim trailing comma, if any
                    if !visitor.0.is_empty() {
                        visitor.0.truncate(visitor.0.len().saturating_sub(2));
                    }

                    // build the main part of the line (timestamp + message fields)
                    line.push_str(&visitor.0);

                    trace!("pushing single-line log event with aligned location bracket");
                    buf.push(line);
                }

                EventPrinter::JustTheContents => {
                    // Minimal approach: only the field values
                    let mut message = String::new();
                    struct FieldCollector(String);
                    impl tracing::field::Visit for FieldCollector {
                        fn record_debug(
                            &mut self,
                            _f: &tracing::field::Field,
                            v: &dyn std::fmt::Debug
                        ) {
                            let _ = write!(self.0, "{:?}, ", v);
                        }
                    }
                    let mut visitor = FieldCollector(String::new());
                    event.record(&mut visitor);

                    if !visitor.0.is_empty() {
                        visitor.0.truncate(visitor.0.len().saturating_sub(2));
                        message.push_str(&visitor.0);
                    }

                    debug!("pushing minimal event fields");
                    buf.push(message);
                }
            }
        } else {
            warn!("failed to acquire buffer lock in BufferedLayer::on_event");
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

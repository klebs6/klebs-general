crate::ix!();

pub struct BufferedLayer {
    pub(crate) tag:    Option<String>,
    pub(crate) buffer: Arc<Mutex<Vec<String>>>,
}

impl Clone for BufferedLayer {

    fn clone(&self) -> Self {
        Self {
            tag:    self.tag.clone(),
            buffer: Arc::clone(&self.buffer),
        }
    }
}

impl BufferedLayer {

    pub fn new(tag: &str) -> Self {
        Self {
            tag:    Some(tag.to_string()),
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl Default for BufferedLayer {

    fn default() -> Self {
        Self {
            tag:    None,
            buffer: Arc::new(Mutex::new(Vec::new())),
        }
    }
}

impl<S: Subscriber> tracing_subscriber::Layer<S> for BufferedLayer {

    fn on_event(&self, event: &tracing::Event<'_>, _ctx: Context<'_, S>) {

        use std::fmt::Write;

        #[allow(dead_code)]
        pub enum EventPrintType {
            FullWithHeader,
            LogLineAndContents,
            JustTheContents,
        }

        if let Ok(mut buf) = self.buffer.lock() {

            match EventPrintType::JustTheContents {

                EventPrintType::FullWithHeader => {

                    // Capture the event's formatted representation
                    // and store it in the buffer
                    //
                    let mut message = String::new();

                    let _ = write!(&mut message, "{:#?}", event); 

                    buf.push(message);

                },
                EventPrintType::LogLineAndContents => {

                    let metadata = event.metadata();

                    let mut message = format!("[{}] {}: ", metadata.level(), metadata.target());

                    // Visitor to collect fields
                    struct FieldCollector(String);

                    impl tracing::field::Visit for FieldCollector {
                        fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                            use std::fmt::Write;
                            let _ = write!(self.0, "{} = {:?}, ", field.name(), value);
                        }
                    }

                    let mut visitor = FieldCollector(String::new());

                    event.record(&mut visitor);

                    if !visitor.0.is_empty() {
                        // Trim trailing comma and space
                        message.push_str(&visitor.0[..visitor.0.len() - 2]);
                    }

                    buf.push(message);
                },
                EventPrintType::JustTheContents => {

                    let _metadata = event.metadata();

                    //let mut message = format!("[{}] {}: ", _metadata.level(), _metadata.target());
                    let mut message = format!("");

                    // Visitor to collect fields
                    struct FieldCollector(String);

                    impl tracing::field::Visit for FieldCollector {
                        fn record_debug(&mut self, _field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
                            use std::fmt::Write;
                            //let _ = write!(self.0, "{} = {:?}, ", field.name(), value);
                            let _ = write!(self.0, "{:?}, ", value);
                        }
                    }

                    let mut visitor = FieldCollector(String::new());

                    event.record(&mut visitor);

                    if !visitor.0.is_empty() {
                        // Trim trailing comma and space
                        message.push_str(&visitor.0[..visitor.0.len() - 2]);
                    }

                    buf.push(message);
                }
            }
        }
    }
}

crate::ix!();

pub trait HandleBufferWriteEvent {
    fn handle_buffer_write_event(event: &tracing::Event<'_>, buf: &mut Vec<String>);
}

#[derive(Debug,Clone)]
pub enum EventPrinter {
    FullWithHeader,
    LogLineAndContents,
    JustTheContents,
}

impl Default for EventPrinter { 

    fn default() -> Self { 
        Self::JustTheContents 
    }
}

//--------------------------------
pub struct FullWithHeader;

impl HandleBufferWriteEvent for FullWithHeader {

    fn handle_buffer_write_event(event: &tracing::Event<'_>, buf: &mut Vec<String>) {
        // Capture the event's formatted representation
        // and store it in the buffer
        //
        let mut message = String::new();

        let _ = write!(&mut message, "{:#?}", event); 

        buf.push(message);
    }
}

//--------------------------------
pub struct LogLineAndContents;

impl HandleBufferWriteEvent for LogLineAndContents {

    fn handle_buffer_write_event(event: &tracing::Event<'_>, buf: &mut Vec<String>) {

        let metadata = event.metadata();

        let mut message = format!("[{}] {}: ", metadata.level(), metadata.target());

        // Visitor to collect fields
        struct FieldCollector(String);

        impl tracing::field::Visit for FieldCollector {
            fn record_debug(&mut self, field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
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

    }
}

//--------------------------------
pub struct JustTheContents;

impl HandleBufferWriteEvent for JustTheContents {

    fn handle_buffer_write_event(event: &tracing::Event<'_>, buf: &mut Vec<String>) {
        let _metadata = event.metadata();

        //let mut message = format!("[{}] {}: ", _metadata.level(), _metadata.target());
        let mut message = format!("");

        // Visitor to collect fields
        struct FieldCollector(String);

        impl tracing::field::Visit for FieldCollector {
            fn record_debug(&mut self, _field: &tracing::field::Field, value: &dyn std::fmt::Debug) {
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

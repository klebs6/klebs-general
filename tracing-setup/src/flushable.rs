crate::ix!();

pub trait Flushable {
    fn flush(&self);
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
